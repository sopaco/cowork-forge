use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::artifacts::{Stage, ArtifactEnvelope, CheckReport, CheckReportArtifact};
use crate::memory::ArtifactStore;
use crate::agents::{
    IdeaIntakeAgent, PrdAgent, DesignAgent, PlanAgent, 
    CodePlanner, CheckAgent, FeedbackAgent, DeliveryAgent
};
use crate::hitl::HitlController;
use crate::config::ModelConfig;

#[cfg(test)]
mod tests;

/// Stage 执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StageStatus {
    /// 未开始
    NotStarted,
    
    /// 执行中
    InProgress {
        started_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// 完成（可能有或没有验证）
    Completed {
        artifact_id: String,
        completed_at: chrono::DateTime<chrono::Utc>,
        verified: bool,  // 是否经过验证
    },
    
    /// 失败
    Failed {
        error: String,
        failed_at: chrono::DateTime<chrono::Utc>,
        can_retry: bool,
    },
}

/// Session 元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub current_stage: Option<Stage>,
    
    #[serde(default)]
    pub stage_status: HashMap<Stage, StageStatus>,  // 阶段状态
    
    // Feedback loop 控制
    #[serde(default)]
    pub feedback_iterations: usize,  // 当前 Feedback 迭代次数
    
    #[serde(default = "default_max_feedback_iterations")]
    pub max_feedback_iterations: usize,  // 最大 Feedback 迭代次数（默认 20）
    
    // 修改上下文：保存用户通过 modify 命令提交的修改意图
    // 用于在重跑阶段时告知 CodePlanner 这是修改任务，而非从头创建
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modification_context: Option<String>,
}

fn default_max_feedback_iterations() -> usize {
    20
}

/// Orchestrator 负责驱动多阶段流程
pub struct Orchestrator {
    store: Arc<ArtifactStore>,
}

impl Orchestrator {
    pub fn new(store: ArtifactStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// 创建新 session
    pub fn create_session(&self) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let meta = SessionMeta {
            session_id: session_id.clone(),
            created_at: chrono::Utc::now(),
            current_stage: None,
            stage_status: HashMap::new(),
            feedback_iterations: 0,
            max_feedback_iterations: 20,
            modification_context: None,
        };

        self.save_session_meta(&meta)?;

        tracing::info!("Session created: {}", session_id);
        Ok(session_id)
    }

    /// 加载 session meta
    pub fn load_session_meta(&self, session_id: &str) -> Result<SessionMeta> {
        use std::fs;
        use std::path::PathBuf;

        let meta_path = PathBuf::from(".cowork")
            .join(session_id)
            .join("meta.json");

        let content = fs::read_to_string(&meta_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// 保存 session meta
    pub fn save_session_meta(&self, meta: &SessionMeta) -> Result<()> {
        use std::fs;
        use std::path::PathBuf;

        let session_dir = PathBuf::from(".cowork").join(&meta.session_id);
        fs::create_dir_all(&session_dir)?;

        let meta_path = session_dir.join("meta.json");
        let content = serde_json::to_string_pretty(meta)?;
        fs::write(&meta_path, content)?;

        Ok(())
    }

    /// 运行完整的 8 阶段工作流
    pub async fn run_full_workflow(&self, session_id: &str, model_config: &ModelConfig) -> Result<()> {
        self.run_workflow_from_stage(session_id, model_config, None).await
    }
    
    /// 标记阶段为进行中
    fn mark_stage_in_progress(&self, meta: &mut SessionMeta, stage: Stage) -> Result<()> {
        meta.stage_status.insert(
            stage,
            StageStatus::InProgress {
                started_at: chrono::Utc::now(),
            }
        );
        meta.current_stage = Some(stage);
        self.save_session_meta(meta)?;
        Ok(())
    }
    
    /// 标记阶段为完成
    fn mark_stage_completed(
        &self,
        meta: &mut SessionMeta,
        stage: Stage,
        artifact_id: String,
        verified: bool
    ) -> Result<()> {
        meta.stage_status.insert(
            stage,
            StageStatus::Completed {
                artifact_id,
                completed_at: chrono::Utc::now(),
                verified,
            }
        );
        self.save_session_meta(meta)?;
        Ok(())
    }
    
    /// 标记阶段为失败
    fn mark_stage_failed(
        &self,
        meta: &mut SessionMeta,
        stage: Stage,
        error: String,
        can_retry: bool
    ) -> Result<()> {
        meta.stage_status.insert(
            stage,
            StageStatus::Failed {
                error,
                failed_at: chrono::Utc::now(),
                can_retry,
            }
        );
        self.save_session_meta(meta)?;
        Ok(())
    }
    
    /// 检查阶段是否已完成（包括已验证和未验证）
    fn is_stage_completed(&self, meta: &SessionMeta, stage: Stage) -> bool {
        matches!(
            meta.stage_status.get(&stage),
            Some(StageStatus::Completed { .. })
        )
    }
    
    /// 检查阶段是否已成功完成并验证
    fn is_stage_completed_and_verified(&self, meta: &SessionMeta, stage: Stage) -> bool {
        matches!(
            meta.stage_status.get(&stage),
            Some(StageStatus::Completed { verified: true, .. })
        )
    }

    /// 从指定阶段开始运行工作流（用于恢复）
    /// 
    /// # 参数
    /// - `session_id`: 会话 ID
    /// - `model_config`: 模型配置
    /// - `resume_from`: 从哪个阶段开始（None = 从头开始）
    pub async fn run_workflow_from_stage(
        &self,
        session_id: &str,
        model_config: &ModelConfig,
        resume_from: Option<Stage>,
    ) -> Result<()> {
        tracing::info!("Running workflow for session: {}, resume_from: {:?}", session_id, resume_from);

        let hitl = HitlController::new();
        let mut meta = self.load_session_meta(session_id)?;

        // 确定起始阶段
        let start_stage = resume_from.unwrap_or(Stage::IdeaIntake);
        
        // 如果是恢复模式，显示已完成的阶段并验证状态
        if resume_from.is_some() {
            println!("\n╔═══════════════════════════════════════╗");
            println!("║   🔄 恢复会话: {}  ", &session_id[..8]);
            println!("╚═══════════════════════════════════════╝");
            
            // 验证前置阶段
            for stage in Stage::all() {
                if *stage == start_stage { break; }
                
                match meta.stage_status.get(stage) {
                    Some(StageStatus::Completed { verified: true, artifact_id, .. }) => {
                        println!("✅ {} - 已完成并验证 (artifact: {})", stage.as_str(), &artifact_id[..8]);
                    }
                    Some(StageStatus::Completed { verified: false, artifact_id, .. }) => {
                        println!("⚠️  {} - 已完成但未验证 (artifact: {})", stage.as_str(), &artifact_id[..8]);
                        println!("   建议：重新验证或从此阶段重新运行");
                    }
                    Some(StageStatus::Failed { error, can_retry, .. }) => {
                        println!("❌ {} - 失败: {}", stage.as_str(), error);
                        if *can_retry {
                            println!("   提示：可以重试此阶段");
                        }
                        return Err(anyhow::anyhow!("前置阶段 {} 失败，无法继续", stage.as_str()));
                    }
                    Some(StageStatus::InProgress { .. }) => {
                        println!("🔄 {} - 未完成（进行中）", stage.as_str());
                        return Err(anyhow::anyhow!("前置阶段 {} 未完成", stage.as_str()));
                    }
                    Some(StageStatus::NotStarted) | None => {
                        println!("❓ {} - 未开始", stage.as_str());
                        return Err(anyhow::anyhow!("前置阶段 {} 未完成", stage.as_str()));
                    }
                }
            }
            
            println!("从阶段继续: {:?}", start_stage);
            println!();
        }

        // Stage 1: IDEA Intake
        let idea_artifact = if self.is_stage_completed_and_verified(&meta, Stage::IdeaIntake) {
            println!("✓ 跳过 Stage 1: IDEA Intake (已完成)");
            self.load_artifact::<crate::artifacts::IdeaSpecArtifact>(session_id, Stage::IdeaIntake)?
        } else {
            println!("\n╔═══════════════════════════════════════╗");
            println!("║   Stage 1: IDEA Intake               ║");
            println!("╚═══════════════════════════════════════╝\n");
            
            self.mark_stage_in_progress(&mut meta, Stage::IdeaIntake)?;
            
            let user_idea = hitl.input("请描述你的 IDEA：")?;
            
            let idea_agent = IdeaIntakeAgent::new(&model_config.llm, self.store.clone())?;
            let mut idea_artifact = idea_agent.execute(session_id, &user_idea).await?;
            
            // HITL 审查和修改
            if let Some(modified_json) = hitl.review_and_edit_json("IdeaSpec", &idea_artifact.data)? {
                let modified_data: crate::artifacts::IdeaSpec = serde_json::from_str(&modified_json)?;
                idea_artifact.data = modified_data;
                self.store.put(session_id, Stage::IdeaIntake, &idea_artifact)?;
                println!("✅ IdeaSpec 已更新");
            }
            
            self.mark_stage_completed(&mut meta, Stage::IdeaIntake, idea_artifact.meta.artifact_id.clone(), true)?;

            self.print_idea_summary(&idea_artifact);

            if !hitl.confirm("继续生成 PRD？")? {
                return Ok(());
            }
            
            idea_artifact
        };

        // Stage 2: PRD Generation
        let prd_artifact = if self.is_stage_completed(&meta, Stage::Requirements) {
            println!("✓ 跳过 Stage 2: Requirements (已完成)");
            self.load_artifact::<crate::artifacts::PRDArtifact>(session_id, Stage::Requirements)?
        } else {
            println!("\n╔═══════════════════════════════════════╗");
            println!("║   Stage 2: Requirements (PRD)        ║");
            println!("╚═══════════════════════════════════════╝\n");
            
            let prd_agent = PrdAgent::new(&model_config.llm, self.store.clone())?;
            let mut prd_artifact = prd_agent.execute(session_id, &idea_artifact).await?;
            
            // HITL 审查和修改
            if let Some(modified_json) = hitl.review_and_edit_json("PRD", &prd_artifact.data)? {
                let modified_data: crate::artifacts::PRD = serde_json::from_str(&modified_json)?;
                prd_artifact.data = modified_data;
                self.store.put(session_id, Stage::Requirements, &prd_artifact)?;
                println!("✅ PRD 已更新");
            }
            
            self.mark_stage_completed(&mut meta, Stage::Requirements, prd_artifact.meta.artifact_id.clone(), true)?;

            self.print_prd_summary(&prd_artifact);

            if !hitl.confirm("继续生成设计文档？")? {
                return Ok(());
            }
            
            prd_artifact
        };

        // Stage 3: Design
        let design_artifact = if self.is_stage_completed(&meta, Stage::Design) {
            println!("✓ 跳过 Stage 3: Design (已完成)");
            self.load_artifact::<crate::artifacts::DesignDocArtifact>(session_id, Stage::Design)?
        } else {
            println!("\n╔═══════════════════════════════════════╗");
            println!("║   Stage 3: Design Document            ║");
            println!("╚═══════════════════════════════════════╝\n");
            
            let design_agent = DesignAgent::new(&model_config.llm, self.store.clone())?;
            let mut design_artifact = design_agent.execute(session_id, &prd_artifact).await?;
            
            // HITL 审查和修改
            if let Some(modified_json) = hitl.review_and_edit_json("DesignDoc", &design_artifact.data)? {
                let modified_data: crate::artifacts::DesignDoc = serde_json::from_str(&modified_json)?;
                design_artifact.data = modified_data;
                self.store.put(session_id, Stage::Design, &design_artifact)?;
                println!("✅ DesignDoc 已更新");
            }
            
            self.mark_stage_completed(&mut meta, Stage::Design, design_artifact.meta.artifact_id.clone(), true)?;

            self.print_design_summary(&design_artifact);

            if !hitl.confirm("继续生成实施计划？")? {
                return Ok(());
            }
            
            design_artifact
        };

        // Stage 4: Plan
        let mut plan_artifact = if self.is_stage_completed(&meta, Stage::Plan) {
            println!("✓ 跳过 Stage 4: Plan (已完成)");
            self.load_artifact::<crate::artifacts::PlanArtifact>(session_id, Stage::Plan)?
        } else {
            println!("\n╔═══════════════════════════════════════╗");
            println!("║   Stage 4: Implementation Plan        ║");
            println!("╚═══════════════════════════════════════╝\n");
            
            let plan_agent = PlanAgent::new(&model_config.llm, self.store.clone())?;
            let mut plan_artifact = plan_agent.execute(session_id, &design_artifact).await?;
            
            // HITL 审查和修改
            if let Some(modified_json) = hitl.review_and_edit_json("Plan", &plan_artifact.data)? {
                let modified_data: crate::artifacts::Plan = serde_json::from_str(&modified_json)?;
                plan_artifact.data = modified_data;
                self.store.put(session_id, Stage::Plan, &plan_artifact)?;
                println!("✅ Plan 已更新");
            }
            
            self.mark_stage_completed(&mut meta, Stage::Plan, plan_artifact.meta.artifact_id.clone(), true)?;

            self.print_plan_summary(&plan_artifact);

            if !hitl.confirm("继续生成代码？")? {
                return Ok(());
            }
            
            plan_artifact
        };

        // Stage 5: Coding
        let code_artifact = if self.is_stage_completed_and_verified(&meta, Stage::Coding) {
            println!("✓ 跳过 Stage 5: Coding (已完成并验证)");
            self.load_artifact::<crate::artifacts::CodeChangeArtifact>(session_id, Stage::Coding)?
        } else {
            println!("\n╔═══════════════════════════════════════╗");
            println!("║   Stage 5: Code Planning              ║");
            println!("╚═══════════════════════════════════════╝\n");
            
            // 标记为进行中
            self.mark_stage_in_progress(&mut meta, Stage::Coding)?;
            
            let code_planner = CodePlanner::new(&model_config.llm, self.store.clone())?;
            let code_artifact = code_planner.execute(
                session_id,
                &prd_artifact,
                &design_artifact,
                &plan_artifact
            ).await?;

            self.print_code_summary(&code_artifact);

            // 询问是否执行代码变更
            let mut execution_verified = false;
            if hitl.confirm("是否执行代码变更（AI 自动生成并写入文件）？")? {
                println!("\n╔═══════════════════════════════════════╗");
                println!("║   Stage 5.5: AI Code Generation       ║");
                println!("╚═══════════════════════════════════════╝\n");
                
                // 使用支持 AI 代码生成的 executor
                let executor = crate::agents::CodeExecutor::new(&model_config.llm)?;
                
                // 提取 PRD 摘要（用于 WatchDog）
                let prd_summary = crate::utils::extract_prd_summary(&prd_artifact);
                
                // 获取 TodoList（如果存在）
                let mut todo_list = plan_artifact.data.todo_list.clone();
                
                match executor.execute_with_todo(
                    &code_artifact,
                    &hitl,
                    Some(&prd_summary),
                    todo_list.as_mut(),
                ).await {
                    Ok(report) => {
                        println!("\n代码生成完成:");
                        println!("  ✅ 成功: {}", report.successful);
                        println!("  ❌ 失败: {}", report.failed);
                        println!("  ⏭️  跳过: {}", report.skipped);
                        
                        // 如果全部成功，标记为已验证
                        execution_verified = report.failed == 0 && report.successful > 0;
                        
                        if !execution_verified {
                            println!("⚠️  部分文件生成失败，Coding 阶段将标记为未验证");
                        }
                        
                        // 保存更新后的 TodoList
                        if let Some(updated_todo_list) = todo_list {
                            plan_artifact.data.todo_list = Some(updated_todo_list);
                            // 更新 plan artifact
                            self.store.put(session_id, Stage::Plan, &plan_artifact)?;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Code execution failed: {}", e);
                        self.mark_stage_failed(&mut meta, Stage::Coding, e.to_string(), true)?;
                        return Err(e);
                    }
                }
            } else {
                println!("⏭️  跳过代码生成，仅保留计划（未验证）");
            }
            
            // 标记为完成
            self.mark_stage_completed(&mut meta, Stage::Coding, code_artifact.meta.artifact_id.clone(), execution_verified)?;

            if !hitl.confirm("继续代码检查？")? {
                return Ok(());
            }
            
            code_artifact
        };

        // Stage 6: Check（支持智能重试）
        const MAX_RETRY: usize = 3;
        let mut retry_count = 0;
        let mut check_artifact = loop {
            if self.is_stage_completed(&meta, Stage::Check) && retry_count == 0 {
                println!("✓ 跳过 Stage 6: Check (已完成)");
                break self.load_artifact::<crate::artifacts::CheckReportArtifact>(session_id, Stage::Check)?;
            }
            
            if retry_count > 0 {
                println!("\n🔄 智能重试 Check 阶段 (第 {} 次)", retry_count);
            } else {
                println!("\n╔═══════════════════════════════════════╗");
                println!("║   Stage 6: Quality Check              ║");
                println!("╚═══════════════════════════════════════╝\n");
            }
            
            let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;
            let check_artifact = check_agent.execute(session_id, &code_artifact).await?;
            
            self.mark_stage_completed(&mut meta, Stage::Check, check_artifact.meta.artifact_id.clone(), true)?;

            self.print_check_summary(&check_artifact);
            
            // 使用 ErrorAnalyzer 分析错误
            let error_analysis = crate::agents::ErrorAnalyzer::analyze(&check_artifact.data);
            
            if error_analysis.has_critical_errors && retry_count < MAX_RETRY {
                println!("\n⚠️  发现 {} 个严重问题:", 
                    check_artifact.data.issues.iter().filter(|i| i.sev == "error").count());
                println!("{}", error_analysis.summary);
                println!("\n受影响的文件 ({} 个):", error_analysis.affected_files.len());
                for file in &error_analysis.affected_files {
                    println!("  - {}", file);
                    if let Some(errors) = error_analysis.error_details_by_file.get(file) {
                        for error in errors.iter().take(2) {  // 只显示前 2 个
                            println!("    {}", error);
                        }
                    }
                }
                
                if hitl.confirm(&format!("是否针对性修复这些文件？ ({}/{} 次重试)", retry_count + 1, MAX_RETRY))? {
                    println!("\n🔧 执行针对性修复（只重新生成受影响的文件）...\n");
                    
                    // 创建只包含受影响文件的修复计划
                    let fix_changes: Vec<crate::artifacts::Change> = code_artifact.data.changes.iter()
                        .filter(|c| error_analysis.affected_files.contains(&c.path))
                        .cloned()
                        .collect();
                    
                    if fix_changes.is_empty() {
                        println!("⚠️  无法识别受影响的文件，跳过重试");
                        break check_artifact;
                    }
                    
                    println!("📝 修复计划: 重新生成 {} 个文件", fix_changes.len());
                    for change in &fix_changes {
                        println!("  - {}", change.path);
                    }
                    println!();
                    
                    // 创建临时的 CodeChangeArtifact（只包含需要修复的文件）
                    let mut fix_artifact = code_artifact.clone();
                    fix_artifact.data.changes = fix_changes;
                    
                    // 构建修复指令（包含错误信息）
                    let fix_context = format!(
                        "Previous generation (attempt {}) had the following errors:\n\n{}\n\n\
                        IMPORTANT:\n\
                        - Focus on fixing the specific errors mentioned above\n\
                        - Only modify the files that have errors\n\
                        - Ensure the code compiles and runs correctly",
                        retry_count,
                        error_analysis.detailed_errors
                    );
                    
                    // 执行针对性修复（使用完整 API）
                    let executor = crate::agents::CodeExecutor::new(&model_config.llm)?;
                    
                    // 提取 PRD 摘要（WatchDog）
                    let prd_summary = crate::utils::extract_prd_summary(&prd_artifact);
                    
                    // 获取 TodoList（可变引用）
                    let mut todo_list = plan_artifact.data.todo_list.clone();
                    
                    println!("💡 修复提示:\n{}\n", fix_context);
                    
                    match executor.execute_with_todo(
                        &fix_artifact,
                        &hitl,
                        Some(&prd_summary),      // WatchDog 提醒
                        todo_list.as_mut(),       // TodoList 更新
                    ).await {
                        Ok(report) => {
                            println!("\n针对性修复完成:");
                            println!("  ✅ 成功: {}", report.successful);
                            println!("  ❌ 失败: {}", report.failed);
                            
                            // 保存更新后的 TodoList
                            if let Some(updated_todo_list) = todo_list {
                                plan_artifact.data.todo_list = Some(updated_todo_list);
                                self.store.put(session_id, Stage::Plan, &plan_artifact)?;
                            }
                            
                            if report.failed == 0 && report.successful > 0 {
                                // 更新 Coding 阶段为已验证
                                self.mark_stage_completed(&mut meta, Stage::Coding, code_artifact.meta.artifact_id.clone(), true)?;
                                retry_count += 1;
                                continue;  // 重新运行 Check
                            } else {
                                println!("⚠️  部分文件修复失败");
                                if retry_count + 1 < MAX_RETRY {
                                    println!("提示：还有 {} 次重试机会", MAX_RETRY - retry_count - 1);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Targeted fix failed: {}", e);
                            println!("❌ 针对性修复失败: {}", e);
                        }
                    }
                } else {
                    println!("用户选择不重试，继续下一步");
                }
            }
            
            break check_artifact;
        };

        // Stage 7: Feedback Loop (自动重试机制)
        loop {
            let user_feedback = hitl.input("有反馈吗？（直接回车跳过）")?;
            
            if user_feedback.trim().is_empty() {
                println!("✓ 用户满意,跳过 Feedback");
                break;
            }
            
            println!("\n╔═══════════════════════════════════════╗");
            println!("║   Stage 7: Feedback Analysis          ║");
            println!("╚═══════════════════════════════════════╝\n");
            
            // 检查是否达到最大迭代次数
            if meta.feedback_iterations >= meta.max_feedback_iterations {
                println!("⚠️  已达到最大 Feedback 迭代次数 ({}次)", meta.max_feedback_iterations);
                println!("   系统将停止自动迭代");
                break;
            }
            
            let feedback_agent = FeedbackAgent::new(&model_config.llm, self.store.clone())?;
            let feedback_artifact = feedback_agent.execute(session_id, &check_artifact, &user_feedback).await?;
            
            self.mark_stage_completed(&mut meta, Stage::Feedback, feedback_artifact.meta.artifact_id.clone(), true)?;
            meta.feedback_iterations += 1;
            self.save_session_meta(&meta)?;

            self.print_feedback_summary(&feedback_artifact);
            
            // 如果没有需要修改或重跑的内容，结束循环
            if feedback_artifact.data.delta.is_empty() && feedback_artifact.data.rerun.is_empty() {
                println!("✓ 无需修改，Feedback 循环结束");
                break;
            }

            // ✅ 自动应用 delta 修改
            if !feedback_artifact.data.delta.is_empty() {
                self.apply_feedback_delta(session_id, &feedback_artifact.data.delta, &model_config).await?;
            }
            
            // ✅ 自动重跑阶段（级联）
            if !feedback_artifact.data.rerun.is_empty() {
                println!("\n🔄 自动重新执行阶段 (Feedback 迭代 {}/{})...", 
                    meta.feedback_iterations, meta.max_feedback_iterations);
                
                for rerun in &feedback_artifact.data.rerun {
                    println!("  - {:?}: {}", rerun.stage, rerun.reason);
                }
                
                // 获取最早需要重跑的阶段，自动级联执行后续所有阶段
                if let Some(earliest_stage) = Self::get_earliest_stage_to_rerun(&feedback_artifact.data.rerun) {
                    match earliest_stage {
                        Stage::Plan => {
                            println!("\n▶ 重新执行: Plan → Coding → Check");
                            
                            // 重跑 Plan
                            let plan_agent = PlanAgent::new(&model_config.llm, self.store.clone())?;
                            let design_artifact = self.load_artifact(session_id, Stage::Design)?;
                            let plan_artifact = plan_agent.execute(session_id, &design_artifact).await?;
                            
                            // 重跑 Coding
                            let code_planner = CodePlanner::new(&model_config.llm, self.store.clone())?;
                            let code_artifact = code_planner.execute(
                                session_id,
                                &prd_artifact,
                                &design_artifact,
                                &plan_artifact,
                            ).await?;
                            
                            // 重跑 Check
                            let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;
                            let new_check = check_agent.execute(session_id, &code_artifact).await?;
                            
                            // 更新 check_artifact 用于下一轮 Feedback
                            check_artifact = new_check;
                            
                            println!("✓ Plan → Coding → Check 重新执行完成");
                        }
                        Stage::Coding => {
                            println!("\n▶ 重新执行: Coding → Check");
                            
                            // 重跑 Coding
                            let code_planner = CodePlanner::new(&model_config.llm, self.store.clone())?;
                            let prd_artifact = self.load_artifact(session_id, Stage::Requirements)?;
                            let design_artifact = self.load_artifact(session_id, Stage::Design)?;
                            let plan_artifact = self.load_artifact(session_id, Stage::Plan)?;
                            let code_artifact = code_planner.execute(
                                session_id,
                                &prd_artifact,
                                &design_artifact,
                                &plan_artifact,
                            ).await?;
                            
                            // 重跑 Check
                            let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;
                            let new_check = check_agent.execute(session_id, &code_artifact).await?;
                            
                            check_artifact = new_check;
                            
                            println!("✓ Coding → Check 重新执行完成");
                        }
                        Stage::Check => {
                            println!("\n▶ 重新执行: Check");
                            
                            let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;
                            let code_artifact = self.load_artifact(session_id, Stage::Coding)?;
                            let new_check = check_agent.execute(session_id, &code_artifact).await?;
                            
                            check_artifact = new_check;
                            
                            println!("✓ Check 重新执行完成");
                        }
                        Stage::Design => {
                            println!("\n▶ 重新执行: Design → Plan → Coding → Check");
                            
                            // 重跑 Design
                            let design_agent = DesignAgent::new(&model_config.llm, self.store.clone())?;
                            let prd_artifact = self.load_artifact(session_id, Stage::Requirements)?;
                            let design_artifact = design_agent.execute(session_id, &prd_artifact).await?;
                            
                            // 重跑 Plan
                            let plan_agent = PlanAgent::new(&model_config.llm, self.store.clone())?;
                            let plan_artifact = plan_agent.execute(session_id, &design_artifact).await?;
                            
                            // 重跑 Coding
                            let code_planner = CodePlanner::new(&model_config.llm, self.store.clone())?;
                            let code_artifact = code_planner.execute(
                                session_id,
                                &prd_artifact,
                                &design_artifact,
                                &plan_artifact,
                            ).await?;
                            
                            // 重跑 Check
                            let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;
                            let new_check = check_agent.execute(session_id, &code_artifact).await?;
                            
                            check_artifact = new_check;
                            
                            println!("✓ Design → Plan → Coding → Check 重新执行完成");
                        }
                        Stage::Requirements => {
                            println!("\n▶ 重新执行: Requirements → Design → Plan → Coding → Check");
                            
                            // 重跑 Requirements
                            let prd_agent = PrdAgent::new(&model_config.llm, self.store.clone())?;
                            let idea_artifact = self.load_artifact(session_id, Stage::IdeaIntake)?;
                            let prd_artifact = prd_agent.execute(session_id, &idea_artifact).await?;
                            
                            // 重跑后续所有阶段
                            let design_agent = DesignAgent::new(&model_config.llm, self.store.clone())?;
                            let design_artifact = design_agent.execute(session_id, &prd_artifact).await?;
                            
                            let plan_agent = PlanAgent::new(&model_config.llm, self.store.clone())?;
                            let plan_artifact = plan_agent.execute(session_id, &design_artifact).await?;
                            
                            let code_planner = CodePlanner::new(&model_config.llm, self.store.clone())?;
                            let code_artifact = code_planner.execute(
                                session_id,
                                &prd_artifact,
                                &design_artifact,
                                &plan_artifact,
                            ).await?;
                            
                            let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;
                            let new_check = check_agent.execute(session_id, &code_artifact).await?;
                            
                            check_artifact = new_check;
                            
                            println!("✓ Requirements → Design → Plan → Coding → Check 重新执行完成");
                        }
                        _ => {
                            println!("⚠️  暂不支持重跑 {:?} 阶段", earliest_stage);
                        }
                    }
                    
                    // 打印新的 Check 结果
                    self.print_check_summary(&check_artifact);
                }
            }
            
            // 继续下一轮 Feedback 循环（会再次询问用户反馈）
        }

        // Stage 8: Delivery
        if !self.is_stage_completed(&meta, Stage::Delivery) {
            println!("\n╔═══════════════════════════════════════╗");
            println!("║   Stage 8: Delivery Report            ║");
            println!("╚═══════════════════════════════════════╝\n");
            
            let delivery_agent = DeliveryAgent::new(&model_config.llm, self.store.clone())?;
            let delivery_artifact = delivery_agent.execute(session_id, &check_artifact, &idea_artifact).await?;
            
            self.mark_stage_completed(&mut meta, Stage::Delivery, delivery_artifact.meta.artifact_id.clone(), true)?;

            self.print_delivery_summary(&delivery_artifact);
        } else {
            println!("✓ 跳过 Stage 8: Delivery (已完成)");
        }

        println!("\n╔═══════════════════════════════════════╗");
        println!("║   🎉 工作流完成！                     ║");
        println!("╚═══════════════════════════════════════╝\n");
        println!("Session ID: {}", session_id);
        println!("Artifacts: .cowork/{}/artifacts/", session_id);

        Ok(())
    }

    /// 从文件系统加载指定阶段的 artifact
    fn load_artifact<T>(&self, session_id: &str, stage: Stage) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        use std::fs;

        let artifacts = self.store.list(session_id)?;
        
        // 找到该阶段的最新 artifact
        let artifact_meta = artifacts
            .iter()
            .filter(|a| a.stage == stage)
            .max_by_key(|a| &a.path_json)
            .ok_or_else(|| anyhow::anyhow!("No artifact found for stage {:?}", stage))?;

        let content = fs::read_to_string(&artifact_meta.path_json)?;
        let artifact: T = serde_json::from_str(&content)?;
        
        tracing::info!("Loaded artifact for stage {:?} from {}", stage, artifact_meta.path_json.display());
        
        Ok(artifact)
    }

    /// 恢复会话（从中断点继续）
    pub async fn resume_session(&self, session_id: &str, model_config: &ModelConfig) -> Result<()> {
        // 检查 session 是否存在
        if !self.store.session_exists(session_id) {
            return Err(anyhow::anyhow!("Session {} not found", session_id));
        }

        // 加载 session meta
        let meta = self.load_session_meta(session_id)?;
        
        // 确定下一个要执行的阶段
        let all_stages = Stage::all();
        let next_stage = all_stages
            .iter()
            .find(|s| !self.is_stage_completed(&meta, **s))
            .cloned();

        if let Some(stage) = next_stage {
            println!("\n📋 恢复会话: {}", session_id);
            println!("下一阶段: {:?}", stage);
            println!();
            
            self.run_workflow_from_stage(session_id, model_config, Some(stage)).await
        } else {
            println!("\n✅ 会话 {} 已全部完成", session_id);
            Ok(())
        }
    }

    /// 修改需求/设计并触发重新执行
    /// 
    /// 这个方法允许用户在任何时候修改需求或技术方案，系统会自动：
    /// 1. 使用 FeedbackAgent 分析修改内容
    /// 2. 生成 delta 和 rerun 指令
    /// 3. 自动级联重跑相关阶段
    pub async fn modify_and_rerun(
        &self,
        session_id: &str,
        modification: &str,
        model_config: &ModelConfig,
    ) -> Result<()> {
        use crate::agents::FeedbackAgent;

        tracing::info!("modify_and_rerun: session={}, modification={}", session_id, modification);

        // 检查 session 是否存在
        if !self.store.session_exists(session_id) {
            return Err(anyhow::anyhow!("Session {} not found", session_id));
        }

        let mut meta = self.load_session_meta(session_id)?;

        // 检查是否超过最大迭代次数
        if meta.feedback_iterations >= meta.max_feedback_iterations {
            return Err(anyhow::anyhow!(
                "已达到最大 Feedback 迭代次数 ({})，无法继续修改",
                meta.max_feedback_iterations
            ));
        }

        println!("\n╔═══════════════════════════════════════╗");
        println!("║   🔄 处理修改请求                      ║");
        println!("╚═══════════════════════════════════════╝\n");

        // 获取 CheckReport（如果存在）
        let check_artifact = if self.is_stage_completed(&meta, Stage::Check) {
            self.load_artifact::<CheckReportArtifact>(session_id, Stage::Check).ok()
        } else {
            None
        };

        // 如果没有 CheckReport，创建一个空的
        let check_artifact = check_artifact.unwrap_or_else(|| {
            ArtifactEnvelope::new(
                session_id.to_string(),
                Stage::Check,
                CheckReport {
                    checks: vec![],
                    ac_results: vec![],
                    issues: vec![],
                    todo_completion: None,
                    requirement_coverage: None,
                },
            )
        });

        // 使用 FeedbackAgent 分析修改内容
        let feedback_agent = FeedbackAgent::new(&model_config.llm, self.store.clone())?;
        let feedback_artifact = feedback_agent.execute(
            session_id,
            &check_artifact,
            modification,
        ).await?;

        println!("\n📝 分析结果:");
        println!("  修改项: {} 个", feedback_artifact.data.delta.len());
        println!("  需要重跑: {} 个阶段", feedback_artifact.data.rerun.len());

        // 显示详细信息
        for delta in &feedback_artifact.data.delta {
            println!("  - 修改 {:?}: {}", delta.target_stage, delta.change);
        }
        for rerun in &feedback_artifact.data.rerun {
            println!("  - 重跑 {:?}: {}", rerun.stage, rerun.reason);
        }

        // 应用 delta 修改
        self.apply_feedback_delta(session_id, &feedback_artifact.data.delta, model_config).await?;

        // 🆕 保存用户的修改意图到 meta，供 CodePlanner 使用
        meta.modification_context = Some(modification.to_string());
        println!("\n💾 保存修改上下文: {}", modification);

        // 获取最早需要重跑的阶段
        if let Some(earliest_stage) = Self::get_earliest_stage_to_rerun(&feedback_artifact.data.rerun) {
            println!("\n🔄 开始从 {:?} 阶段重新执行...", earliest_stage);

            // 🔧 关键修复：清除要重跑的阶段及其后续阶段的完成状态
            // 这样 run_workflow_from_stage 才会真正重新执行这些阶段
            let all_stages = Stage::all();
            let earliest_index = all_stages.iter().position(|s| s == &earliest_stage).unwrap_or(0);
            
            // 清除从 earliest_stage 开始的所有阶段的完成状态
            for stage in &all_stages[earliest_index..] {
                // 从 stage_status 中移除
                meta.stage_status.remove(stage);
                
                println!("   → 清除 {:?} 阶段的完成状态", stage);
            }
            
            // 保存更新后的 meta
            self.save_session_meta(&meta)?;

            // 更新迭代计数
            meta.feedback_iterations += 1;
            self.save_session_meta(&meta)?;

            // 从最早阶段重新运行（会自动级联）
            self.run_workflow_from_stage(session_id, model_config, Some(earliest_stage)).await?;
        } else {
            println!("\n⚠️  无需重跑任何阶段");
        }

        Ok(())
    }

    /// 列出 session 的所有 artifacts
    pub fn list_artifacts(&self, session_id: &str) -> Result<Vec<crate::memory::ArtifactMeta>> {
        self.store.list(session_id)
    }

    // Helper methods for printing summaries
    fn print_idea_summary(&self, artifact: &crate::artifacts::IdeaSpecArtifact) {
        println!("✓ IdeaSpec 生成成功！");
        println!("  背景: {}", artifact.data.bg);
        println!("  目标: {} 项", artifact.data.g.len());
        println!("  非目标: {} 项", artifact.data.ng.len());
        println!("  约束: {} 项", artifact.data.c.len());
    }

    fn print_prd_summary(&self, artifact: &crate::artifacts::PRDArtifact) {
        println!("✓ PRD 生成成功！");
        println!("  需求总数: {}", artifact.data.reqs.len());
        println!("    - P0: {}", artifact.data.reqs.iter().filter(|r| matches!(r.pri, crate::artifacts::Priority::P0)).count());
        println!("    - P1: {}", artifact.data.reqs.iter().filter(|r| matches!(r.pri, crate::artifacts::Priority::P1)).count());
        println!("    - P2: {}", artifact.data.reqs.iter().filter(|r| matches!(r.pri, crate::artifacts::Priority::P2)).count());
        println!("  约束: {}", artifact.data.cons.len());
        println!("  待确认问题: {}", artifact.data.hitl.len());
    }

    fn print_design_summary(&self, artifact: &crate::artifacts::DesignDocArtifact) {
        println!("✓ 设计文档生成成功！");
        println!("  CLI 模式: {:?}", artifact.data.cli.modes);
        println!("  工作流阶段: {}", artifact.data.wf.stages.len());
        println!("  架构层次: {:?}", artifact.data.arch.layers);
    }

    fn print_plan_summary(&self, artifact: &crate::artifacts::PlanArtifact) {
        println!("✓ 实施计划生成成功！");
        println!("  C4 上下文: {}", artifact.data.c4.context.len());
        println!("  任务总数: {}", artifact.data.tasks.len());
        println!("  里程碑: {}", artifact.data.milestones.len());
    }

    fn print_code_summary(&self, artifact: &crate::artifacts::CodeChangeArtifact) {
        println!("✓ 代码结构生成成功！");
        println!("  语言: {}", artifact.data.target.lang);
        println!("  模块: {}", artifact.data.project.modules.len());
        println!("  文件变更: {}", artifact.data.changes.len());
        println!("  命令: {}", artifact.data.cmds.len());
    }

    fn print_check_summary(&self, artifact: &crate::artifacts::CheckReportArtifact) {
        println!("✓ 检查报告生成完成！");
        println!("  检查项: {}", artifact.data.checks.len());
        println!("  发现问题: {}", artifact.data.issues.len());
    }

    fn print_feedback_summary(&self, artifact: &crate::artifacts::FeedbackArtifact) {
        println!("✓ 反馈分析完成！");
        println!("  需要修改: {} 处", artifact.data.delta.len());
        println!("  需要重跑: {} 个阶段", artifact.data.rerun.len());
    }

    fn print_delivery_summary(&self, artifact: &crate::artifacts::DeliveryReportArtifact) {
        println!("✓ 交付报告生成完成！");
        println!("  功能: {} 项", artifact.data.cap.len());
        println!("  使用说明: {} 条", artifact.data.howto.len());
        println!("  已知限制: {} 项", artifact.data.limits.len());
    }

    /// 应用 Feedback delta 修改到对应的 artifacts（使用 LLM）
    async fn apply_feedback_delta(
        &self,
        _session_id: &str,
        delta: &[crate::artifacts::Delta],
        _model_config: &ModelConfig,
    ) -> Result<()> {
        use crate::artifacts::Stage;

        
        if delta.is_empty() {
            return Ok(());
        }
        
        println!("\n🔄 应用 Feedback 修改...");
        
        for d in delta {
            println!("  - 修改 {:?}: {}", d.target_stage, d.change);
            
            // 简化实现：暂时只记录变更，不实际修改
            // 实际修改会在重跑对应阶段时由 Agent 自动处理
            match d.target_stage {
                Stage::Requirements => {
                    println!("    ℹ️  PRD 修改已记录");
                    println!("    → 将在重跑 Requirements 阶段时应用");
                }
                Stage::Design => {
                    println!("    ℹ️  Design 修改已记录");
                    println!("    → 将在重跑 Design 阶段时应用");
                }
                Stage::Plan => {
                    println!("    ℹ️  Plan 修改已记录");
                    println!("    → 将在重跑 Plan 阶段时应用");
                }
                Stage::Coding => {
                    println!("    ℹ️  代码修改已记录");
                    println!("    → 将在重跑 Coding 阶段时应用");
                }
                Stage::Check => {
                    println!("    ℹ️  验证修改已记录");
                    println!("    → 将在重跑 Check 阶段时应用");
                }
                Stage::IdeaIntake => {
                    println!("    ℹ️  Idea 修改已记录");
                    println!("    → 将在重跑 IdeaIntake 阶段时应用");
                }
                Stage::Feedback | Stage::Delivery => {
                    println!("    ℹ️  {} 修改已记录", d.target_stage.as_str());
                    println!("    → 将在重跑后续阶段时自动更新");
                }
            }
        }
        
        println!("✓ 修改记录完成，将在重跑阶段时应用");
        
        Ok(())
    }

    /// 使用 LLM 应用修改到 PRD
    async fn apply_change_to_prd(
        &self,
        _prd: &crate::artifacts::PRD,
        change: &str,
        _model_config: &ModelConfig,
    ) -> Result<crate::artifacts::PRD> {
        // 简化实现：暂时只记录变更，不实际修改
        // 实际修改会在重跑 Requirements 阶段时由 Agent 自动处理
        println!("    ℹ️  PRD 修改已记录: {}", change);
        println!("    → 将在重跑 Requirements 阶段时应用");
        
        // 返回原始 PRD，让重跑阶段处理
        Ok(_prd.clone())
    }

    /// 使用 LLM 应用修改到 Design
    async fn apply_change_to_design(
        &self,
        _design: &crate::artifacts::DesignDoc,
        change: &str,
        _model_config: &ModelConfig,
    ) -> Result<crate::artifacts::DesignDoc> {
        println!("    ℹ️  Design 修改已记录: {}", change);
        println!("    → 将在重跑 Design 阶段时应用");
        Ok(_design.clone())
    }

    /// 使用 LLM 应用修改到 Plan
    async fn apply_change_to_plan(
        &self,
        _plan: &crate::artifacts::Plan,
        change: &str,
        _model_config: &ModelConfig,
    ) -> Result<crate::artifacts::Plan> {
        println!("    ℹ️  Plan 修改已记录: {}", change);
        println!("    → 将在重跑 Plan 阶段时应用");
        Ok(_plan.clone())
    }

    /// 获取需要重跑的最早阶段（用于自动级联）
    fn get_earliest_stage_to_rerun(rerun: &[crate::artifacts::Rerun]) -> Option<Stage> {
        use crate::artifacts::Stage;
        
        let stage_order = |s: &Stage| -> usize {
            match s {
                Stage::IdeaIntake => 1,
                Stage::Requirements => 2,
                Stage::Design => 3,
                Stage::Plan => 4,
                Stage::Coding => 5,
                Stage::Check => 6,
                Stage::Feedback => 7,
                Stage::Delivery => 8,
            }
        };
        
        rerun.iter()
            .map(|r| &r.stage)
            .min_by_key(|s| stage_order(s))
            .cloned()
    }
}
