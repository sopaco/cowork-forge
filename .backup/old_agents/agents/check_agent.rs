use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::artifacts::*;
use crate::memory::ArtifactStore;
use crate::config::LlmConfig;
use crate::agents::{StageAgent, StageAgentContext, StageAgentResult};
#[path = "check_agent_verification.rs"]
mod check_agent_verification;
#[path = "check_agent_verification_impl.rs"]
mod check_agent_verification_impl;

/// Check Agent - 检查代码质量和完整性
pub struct CheckAgent {
    store: Arc<ArtifactStore>,
}

impl CheckAgent {
    pub fn new(_llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        tracing::info!("Creating Check Agent");
        
        Ok(Self {
            store,
        })
    }

    async fn perform_checks(&self, session_id: &str, code_artifact: &CodeChangeArtifact) -> Result<CheckReportArtifact> {
        tracing::info!("CheckAgent: checking code for session {}", session_id);

        // 尝试加载 PRD artifact（包含 requirements）
        let prd_artifact_result = self.load_prd_artifact(session_id);
        
        // 验证需求覆盖度
        let requirement_coverage = if let Ok(prd_artifact) = prd_artifact_result {
            self.verify_requirement_coverage(&prd_artifact.data, &code_artifact.data).await
        } else {
            tracing::warn!("PRD artifact not found, skipping requirement coverage verification");
            None
        };
        
        // 基础检查
        let mut checks = Vec::new();
        let mut issues = Vec::new();
        
        // 1. 文件存在性检查
        self.check_file_existence(&code_artifact.data, &mut checks, &mut issues);
        
        // 2. 文件内容质量检查
        self.check_file_content_quality(&code_artifact.data, &mut checks, &mut issues);
        
        // 3. 编译/语法检查（根据语言类型）
        self.check_compilation(&code_artifact.data, &mut checks, &mut issues).await;
        
        // 4. 执行验证命令（build/test/run）
        check_agent_verification_impl::run_verification_commands(&code_artifact.data, &mut checks, &mut issues).await;
        
        // 创建初步的 CheckReport
        let mut check_report = CheckReport {
            checks,
            ac_results: vec![],
            issues,
            todo_completion: None,
            requirement_coverage,
        };
        
        // 验证 TodoList 完成度并更新状态
        let todo_completion = if let Ok(mut plan_artifact) = self.load_plan_artifact(session_id) {
            if let Some(ref mut todo_list) = plan_artifact.data.todo_list {
                // 根据验证结果更新 TodoList 状态
                crate::agents::TodoListManager::verify_from_check(todo_list, &check_report);
                
                // 生成状态报告（在保存前）
                let report = crate::agents::TodoListManager::generate_status_report(todo_list);
                
                // 保存更新后的 TodoList（移动到后面，避免借用冲突）
                self.store.put(session_id, Stage::Plan, &plan_artifact)?;
                
                Some(TodoCompletion {
                    total: report.total,
                    completed: report.completed,
                    pending: report.pending,
                    blocked: report.blocked,
                })
            } else {
                None
            }
        } else {
            tracing::warn!("Plan artifact not found, skipping TodoList verification");
            None
        };
        
        // 更新 check_report 的 todo_completion
        check_report.todo_completion = todo_completion;

        let summary = vec![
            format!("Checks: {}", check_report.checks.len()),
            format!("Issues: {}", check_report.issues.len()),
            if let Some(ref tc) = check_report.todo_completion {
                format!("Todo: {}/{} completed", tc.completed, tc.total)
            } else {
                "Todo: N/A".to_string()
            },
            if let Some(ref rc) = check_report.requirement_coverage {
                format!("Coverage: {:.1}%", rc.coverage_percentage)
            } else {
                "Coverage: N/A".to_string()
            },
        ];

        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Check, check_report)
            .with_summary(summary)
            .with_prev(vec![code_artifact.meta.artifact_id.clone()]);

        self.store.put(session_id, Stage::Check, &artifact)?;

        tracing::info!("Check report saved successfully");

        Ok(artifact)
    }
    
    /// 加载 Plan artifact
    fn load_plan_artifact(&self, session_id: &str) -> Result<PlanArtifact> {
        // 列出所有 artifacts，找到 plan stage 的
        let artifacts = self.store.list(session_id)?;
        
        for meta in artifacts {
            if meta.stage == Stage::Plan {
                return self.store.get(session_id, &meta.artifact_id);
            }
        }
        
        Err(anyhow::anyhow!("Plan artifact not found"))
    }
    
    /// 加载 PRD artifact
    fn load_prd_artifact(&self, session_id: &str) -> Result<PRDArtifact> {
        let artifacts = self.store.list(session_id)?;
        
        for meta in artifacts {
            if meta.stage == Stage::Requirements {
                return self.store.get(session_id, &meta.artifact_id);
            }
        }
        
        Err(anyhow::anyhow!("PRD artifact not found"))
    }
    
    /// 验证需求覆盖度
    async fn verify_requirement_coverage(&self, prd: &PRD, code_change: &CodeChange) -> Option<RequirementCoverage> {
        let mut verified = 0;
        let mut not_verified = 0;
        
        for req in &prd.reqs {
            // 查找对应的文件映射
            if let Some(mapping) = code_change.requirement_mapping.iter()
                .find(|m| m.req_id == req.id) 
            {
                // 检查映射的文件是否都存在
                let all_files_exist = mapping.files.iter()
                    .all(|file| std::path::Path::new(file).exists());
                
                if all_files_exist {
                    verified += 1;
                } else {
                    not_verified += 1;
                }
            } else {
                not_verified += 1;
            }
        }
        
        let total = prd.reqs.len();
        let coverage_percentage = if total > 0 {
            (verified as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        Some(RequirementCoverage {
            total_requirements: total,
            verified,
            partially_verified: 0,
            not_verified,
            failed: 0,
            coverage_percentage,
        })
    }
    
    /// 检查文件存在性
    fn check_file_existence(&self, code_change: &CodeChange, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {
        for change in &code_change.changes {
            let file_exists = std::path::Path::new(&change.path).exists();
            
            if file_exists {
                checks.push(CheckResult {
                    id: format!("FILE-EXIST-{}", change.path),
                    cmd: format!("check file exists: {}", change.path),
                    status: "passed".to_string(),
                    out_ref: "".to_string(),
                    notes: vec![format!("File {} exists", change.path)],
                    phase: Phase::Check,
                });
            } else {
                checks.push(CheckResult {
                    id: format!("FILE-EXIST-{}", change.path),
                    cmd: format!("check file exists: {}", change.path),
                    status: "failed".to_string(),
                    out_ref: "".to_string(),
                    notes: vec![format!("File {} does not exist", change.path)],
                    phase: Phase::Check,
                });
                
                issues.push(Issue {
                    id: format!("ISSUE-FILE-{}", change.path),
                    sev: "error".to_string(),
                    desc: format!("File not found: {}", change.path),
                    fix_hint: format!("Create file: {}", change.path),
                });
            }
        }
    }
    
    /// 检查文件内容质量（检测空文件、TODO、placeholder等）
    fn check_file_content_quality(&self, code_change: &CodeChange, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {
        use std::fs;
        
        for change in &code_change.changes {
            let path = std::path::Path::new(&change.path);
            
            if !path.exists() {
                continue;  // 已在上一步检查
            }
            
            // 读取文件内容
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    issues.push(Issue {
                        id: format!("ISSUE-READ-{}", change.path),
                        sev: "warning".to_string(),
                        desc: format!("Cannot read file {}: {}", change.path, e),
                        fix_hint: "Check file permissions".to_string(),
                    });
                    continue;
                }
            };
            
            let lines: Vec<&str> = content.lines().collect();
            let non_empty_lines: Vec<&str> = lines.iter()
                .filter(|line| !line.trim().is_empty())
                .copied()
                .collect();
            
            // 检查 1: 空文件
            if non_empty_lines.is_empty() {
                checks.push(CheckResult {
                    id: format!("CONTENT-QUALITY-{}", change.path),
                    cmd: format!("check file content: {}", change.path),
                    status: "failed".to_string(),
                    out_ref: "".to_string(),
                    notes: vec!["File is empty".to_string()],
                    phase: Phase::Check,
                });
                
                issues.push(Issue {
                    id: format!("ISSUE-EMPTY-{}", change.path),
                    sev: "error".to_string(),
                    desc: format!("File {} is empty", change.path),
                    fix_hint: "Generate actual code content".to_string(),
                });
                continue;
            }
            
            // 检查 2: TODO/FIXME/placeholder
            let todo_count = content.matches("TODO").count() + 
                            content.matches("FIXME").count() +
                            content.matches("placeholder").count();
            
            if todo_count > 0 {
                checks.push(CheckResult {
                    id: format!("CONTENT-QUALITY-{}", change.path),
                    cmd: format!("check for TODOs: {}", change.path),
                    status: "warning".to_string(),
                    out_ref: "".to_string(),
                    notes: vec![format!("Found {} TODO/FIXME/placeholder markers", todo_count)],
                    phase: Phase::Check,
                });
                
                issues.push(Issue {
                    id: format!("ISSUE-TODO-{}", change.path),
                    sev: "warning".to_string(),
                    desc: format!("File {} contains {} incomplete markers (TODO/FIXME/placeholder)", change.path, todo_count),
                    fix_hint: "Complete the implementation".to_string(),
                });
            } else {
                // 内容质量通过
                checks.push(CheckResult {
                    id: format!("CONTENT-QUALITY-{}", change.path),
                    cmd: format!("check file content: {}", change.path),
                    status: "passed".to_string(),
                    out_ref: "".to_string(),
                    notes: vec![format!("File has {} lines of content", non_empty_lines.len())],
                    phase: Phase::Check,
                });
            }
        }
    }
    
    /// 编译/语法检查
    async fn check_compilation(&self, code_change: &CodeChange, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {
        let lang = &code_change.target.lang;
        
        match lang.as_str() {
            "rust" => self.check_rust_compilation(checks, issues).await,
            "python" => self.check_python_syntax(code_change, checks, issues).await,
            "javascript" | "typescript" => self.check_js_syntax(code_change, checks, issues).await,
            "html" | "web" => {
                // HTML 不需要编译，但可以检查基本结构
                tracing::info!("HTML project - skipping compilation check");
            }
            _ => {
                tracing::warn!("Unknown language {}, skipping compilation check", lang);
            }
        }
    }
    
    /// Rust 编译检查
    async fn check_rust_compilation(&self, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {
        use std::process::Command;
        
        tracing::info!("Running cargo check...");
        
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=short")
            .output();
        
        match output {
            Ok(result) => {
                let _stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                if result.status.success() {
                    checks.push(CheckResult {
                        id: "COMPILE-RUST".to_string(),
                        cmd: "cargo check".to_string(),
                        status: "passed".to_string(),
                        out_ref: "".to_string(),
                        notes: vec!["Compilation successful".to_string()],
                        phase: Phase::Check,
                    });
                } else {
                    checks.push(CheckResult {
                        id: "COMPILE-RUST".to_string(),
                        cmd: "cargo check".to_string(),
                        status: "failed".to_string(),
                        out_ref: "".to_string(),
                        notes: vec![format!("Compilation failed:\n{}", stderr)],
                        phase: Phase::Check,
                    });
                    
                    issues.push(Issue {
                        id: "ISSUE-COMPILE-RUST".to_string(),
                        sev: "error".to_string(),
                        desc: "Rust compilation failed".to_string(),
                        fix_hint: format!("Fix compilation errors:\n{}", stderr.lines().take(10).collect::<Vec<_>>().join("\n")),
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Failed to run cargo check: {}", e);
                checks.push(CheckResult {
                    id: "COMPILE-RUST".to_string(),
                    cmd: "cargo check".to_string(),
                    status: "skipped".to_string(),
                    out_ref: "".to_string(),
                    notes: vec![format!("Cannot run cargo: {}", e)],
                    phase: Phase::Check,
                });
            }
        }
    }
    
    /// Python 语法检查
    async fn check_python_syntax(&self, code_change: &CodeChange, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {
        use std::process::Command;
        
        for change in &code_change.changes {
            if !change.path.ends_with(".py") {
                continue;
            }
            
            let output = Command::new("python3")
                .arg("-m")
                .arg("py_compile")
                .arg(&change.path)
                .output();
            
            match output {
                Ok(result) => {
                    if result.status.success() {
                        checks.push(CheckResult {
                            id: format!("SYNTAX-PY-{}", change.path),
                            cmd: format!("python3 -m py_compile {}", change.path),
                            status: "passed".to_string(),
                            out_ref: "".to_string(),
                            notes: vec!["Syntax check passed".to_string()],
                            phase: Phase::Check,
                        });
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        checks.push(CheckResult {
                            id: format!("SYNTAX-PY-{}", change.path),
                            cmd: format!("python3 -m py_compile {}", change.path),
                            status: "failed".to_string(),
                            out_ref: "".to_string(),
                            notes: vec![format!("Syntax error:\n{}", stderr)],
                            phase: Phase::Check,
                        });
                        
                        issues.push(Issue {
                            id: format!("ISSUE-SYNTAX-PY-{}", change.path),
                            sev: "error".to_string(),
                            desc: format!("Python syntax error in {}", change.path),
                            fix_hint: stderr.to_string(),
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to check Python syntax for {}: {}", change.path, e);
                }
            }
        }
    }
    
    /// JavaScript/TypeScript 语法检查
    async fn check_js_syntax(&self, _code_change: &CodeChange, _checks: &mut Vec<CheckResult>, _issues: &mut Vec<Issue>) {
        // 简化版：检查是否有 package.json，如果有则运行 npm run build/check
        let has_package_json = std::path::Path::new("package.json").exists();
        
        if !has_package_json {
            tracing::info!("No package.json found, skipping JS build check");
            return;
        }
        
        // 这里可以扩展为实际的 npm build 检查
        tracing::info!("JavaScript project detected, consider adding npm build check");
    }
}

#[async_trait]
impl StageAgent for CheckAgent {
    fn stage(&self) -> Stage {
        Stage::Check
    }
    
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult> {
        // 1. 加载 CodeChange artifact
        let code_artifact: CodeChangeArtifact = context.load_artifact(Stage::Coding)?;
        
        // 2. 执行检查
        let artifact = self.perform_checks(&context.session_id, &code_artifact).await?;
        
        // 3. 打印检查结果
        println!("\n📊 检查结果:");
        println!("  总检查数: {}", artifact.data.checks.len());
        println!("  问题数: {}", artifact.data.issues.len());
        if let Some(ref cov) = artifact.data.requirement_coverage {
            println!("  需求覆盖率: {:.1}%", cov.coverage_percentage);
        }
        if let Some(ref todo) = artifact.data.todo_completion {
            println!("  Todo完成度: {}/{}", todo.completed, todo.total);
        }
        
        // 4. 返回结果（不需要额外的 HITL）
        let summary = vec![
            format!("Checks: {}", artifact.data.checks.len()),
            format!("Issues: {}", artifact.data.issues.len()),
            format!("Coverage: {:.1}%", 
                artifact.data.requirement_coverage.as_ref().map(|c| c.coverage_percentage).unwrap_or(0.0)),
        ];
        
        Ok(StageAgentResult::new(artifact.meta.artifact_id, Stage::Check)
            .with_verified(true)
            .with_summary(summary))
    }
    
    fn dependencies(&self) -> Vec<Stage> {
        vec![Stage::Coding]
    }
    
    fn requires_hitl_review(&self) -> bool {
        false  // Check 阶段不需要 HITL
    }
    
    fn description(&self) -> &str {
        "检查代码质量和完整性"
    }
}

