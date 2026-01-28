use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

use crate::artifacts::*;
use crate::hitl::HitlController;
use crate::config::LlmConfig;
use crate::tools::{create_file_tools, create_command_tools};
use adk_rust::model::{OpenAIClient, OpenAIConfig};
use adk_rust::prelude::*;
use adk_rust::runner::{Runner, RunnerConfig};
use adk_rust::session::{InMemorySessionService, CreateRequest, SessionService};
use futures::StreamExt;

/// Code Executor - 使用 LLM Agent + file tools 自动实现代码
/// 
/// 核心思想：
/// 1. 创建一个 LlmAgent，挂载文件操作工具
/// 2. 给 Agent 提供变更计划和需求描述
/// 3. Agent 自己决定如何调用工具来实现代码
pub struct CodeExecutor {
    model: Arc<OpenAIClient>,
}

impl CodeExecutor {
    pub fn new(llm_config: &LlmConfig) -> Result<Self> {
        let config = OpenAIConfig::compatible(
            llm_config.api_key.clone(),
            llm_config.api_base_url.clone(),
            llm_config.model_name.clone(),
        );
        
        let client = OpenAIClient::new(config)?;
        
        Ok(Self {
            model: Arc::new(client),
        })
    }

    /// 执行代码变更计划（便捷方法）
    pub async fn execute(
        &self,
        code_artifact: &CodeChangeArtifact,
        hitl: &HitlController
    ) -> Result<ExecutionReport> {
        // 便捷方法：不追踪 TodoList
        self.execute_with_todo(code_artifact, hitl, None, None).await
    }
    
    /// 执行代码变更计划（完整版本，支持 TodoList 追踪和 WatchDog）
    pub async fn execute_with_todo(
        &self,
        code_artifact: &CodeChangeArtifact,
        hitl: &HitlController,
        prd_summary: Option<&str>,
        todo_list: Option<&mut TodoList>,
    ) -> Result<ExecutionReport> {
        tracing::info!("Starting AI-powered code execution with batch sub-agents...");
        
        println!("\n╔═══════════════════════════════════════╗");
        println!("║   AI 代码生成与执行                   ║");
        println!("╚═══════════════════════════════════════╝\n");

        println!("📋 计划执行 {} 个代码变更：", code_artifact.data.changes.len());
        for (i, change) in code_artifact.data.changes.iter().enumerate() {
            println!("  {}. [{}] {} - {}", 
                i + 1, 
                change.kind, 
                change.path, 
                change.note
            );
        }
        println!();

        if !hitl.confirm("是否让 AI Agent 自动实现代码并写入文件？")? {
            tracing::info!("Code execution cancelled by user");
            return Ok(ExecutionReport {
                total_changes: code_artifact.data.changes.len(),
                successful: 0,
                failed: 0,
                skipped: code_artifact.data.changes.len(),
                details: Vec::new(),
            });
        }        // 决策：根据文件数量选择策略
        let changes = &code_artifact.data.changes;
        if changes.len() <= 3 {
            // 少于等于 3 个文件：单个 Agent 处理
            println!("📝 使用单个 Agent 模式（文件数 <= 3）\n");
            self.execute_single_agent(code_artifact, hitl, prd_summary, todo_list).await
        } else {
            // 3 个以上文件：分批处理
            println!("📦 使用分批 Sub-Agent 模式（文件数 > 3）\n");
            self.execute_with_batches(code_artifact, hitl, prd_summary, todo_list).await
        }
    }

    /// 分批处理模式（带上下文传递和 WatchDog）
    async fn execute_with_batches(
        &self,
        code_artifact: &CodeChangeArtifact,
        _hitl: &HitlController,
        prd_summary: Option<&str>,
        todo_list: Option<&mut TodoList>,
    ) -> Result<ExecutionReport> {
        const BATCH_SIZE: usize = 3;  // 每批处理 3 个文件
        
        let changes = &code_artifact.data.changes;
        let batches: Vec<&[crate::artifacts::Change]> = changes.chunks(BATCH_SIZE).collect();
        
        println!("📦 将 {} 个文件分成 {} 批处理（每批最多 {} 个文件）",
            changes.len(),
            batches.len(),
            BATCH_SIZE
        );
        println!();
        
        let mut all_details = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;
        
        // 构建原始需求描述（用于 WatchDog）
        let original_requirements = prd_summary
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.build_requirements_summary(code_artifact));
        
        // 批次上下文（包含文件摘要）
        let mut batch_context = crate::agents::BatchContext::new();
        
        // 逐批处理
        for (batch_idx, batch) in batches.iter().enumerate() {
            println!("╔═══════════════════════════════════════╗");
            println!("║   批次 {}/{}                         ", batch_idx + 1, batches.len());
            println!("╚═══════════════════════════════════════╝\n");
            
            println!("📝 批次 {} 包含 {} 个文件：", batch_idx + 1, batch.len());
            for (i, change) in batch.iter().enumerate() {
                println!("  {}. [{}] {}", i + 1, change.kind, change.path);
            }
            println!();
            
            // 显示批次上下文
            if !batch_context.completed_files.is_empty() {
                println!("📚 已完成的文件 ({} 个):", batch_context.completed_files.len());
                for file_ctx in &batch_context.completed_files {
                    println!("  - {} ({})", file_ctx.path, file_ctx.summary);
                    if !file_ctx.exports.is_empty() {
                        println!("    Exports: {}", file_ctx.exports.iter().take(3).cloned().collect::<Vec<_>>().join(", "));
                    }
                }
                println!();
            }
            
            // 为这一批创建独立的 Sub-Agent，传入 WatchDog 需求和上下文摘要
            let batch_result = self.execute_batch(
                batch_idx,
                batch,
                &code_artifact.data.target,
                Some(&original_requirements),  // 启用 WatchDog
                &batch_context,  // 批次间上下文摘要
            ).await?;
            
            // 生成文件上下文并添加到批次上下文
            for detail in &batch_result.details {
                if detail.status == ChangeStatus::Success {
                    // 读取文件内容并生成摘要
                    if let Ok(content) = std::fs::read_to_string(&detail.change.path) {
                        let file_ctx = crate::agents::FileSummaryGenerator::generate(
                            &detail.change.path,
                            &content,
                            &code_artifact.data.target.lang
                        );
                        batch_context.add_file(file_ctx);
                    }
                }
            }
            
            successful_count += batch_result.successful;
            failed_count += batch_result.failed;
            all_details.extend(batch_result.details);
            
            println!("✅ 批次 {} 完成: {} 成功, {} 失败\n",
                batch_idx + 1,
                batch_result.successful,
                batch_result.failed
            );
        }
        
        println!("╔═══════════════════════════════════════╗");
        println!("║   总执行摘要                          ║");
        println!("╚═══════════════════════════════════════╝");
        println!("总批次: {}", batches.len());
        println!("计划变更: {}", changes.len());
        println!("✅ 成功: {}", successful_count);
        println!("❌ 失败: {}", failed_count);
        
        // 更新 TodoList（如果提供了）
        if let Some(todo_list) = todo_list {
            let successful_files: Vec<String> = all_details.iter()
                .filter(|d| d.status == ChangeStatus::Success)
                .map(|d| d.change.path.clone())
                .collect();
            
            let failed_files: Vec<String> = all_details.iter()
                .filter(|d| d.status == ChangeStatus::Failed)
                .map(|d| d.change.path.clone())
                .collect();
            
            crate::agents::TodoListManager::update_from_execution(
                todo_list,
                &code_artifact.data.changes,
                &successful_files,
                &failed_files,
            );
            
            // 打印 TodoList 状态
            crate::agents::TodoListManager::print_status(todo_list);
        }
        
        Ok(ExecutionReport {
            total_changes: changes.len(),
            successful: successful_count,
            failed: failed_count,
            skipped: 0,
            details: all_details,
        })
    }

    /// 执行单个批次（集成 WatchDog 和上下文传递）
    async fn execute_batch(
        &self,
        batch_idx: usize,
        batch: &[crate::artifacts::Change],
        target: &TargetProject,
        original_requirements: Option<&str>,
        batch_context: &crate::agents::BatchContext,  // 批次上下文摘要
    ) -> Result<BatchExecutionReport> {
        // 创建文件操作工具
        let file_tools = create_file_tools();
        let command_tools = create_command_tools();
        
        // 构建批次任务描述
        let task_description = format!(
            "Please implement the following {} code changes:\n\n{}",
            batch.len(),
            batch.iter()
                .enumerate()
                .map(|(i, change)| format!(
                    "{}. [{}] {} - {}",
                    i + 1,
                    change.kind,
                    change.path,
                    change.note
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );
        
        // 为每个批次创建独立的 Agent（上下文隔离）+ WatchDog 提醒 + 上下文传递
        let agent = Arc::new(
            LlmAgentBuilder::new(format!("batch_{}_executor", batch_idx))
                .description("Batch code executor")
                .instruction(&self.build_batch_instruction(
                    target, 
                    batch.len(), 
                    original_requirements,
                    batch_context
                ))
                .model(self.model.clone())
                // 挂载所有文件工具（10 个）
                .tool(file_tools.write_file.clone())
                .tool(file_tools.read_file.clone())
                .tool(file_tools.list_dir.clone())
                .tool(file_tools.file_exists.clone())
                .tool(file_tools.create_dir.clone())
                .tool(file_tools.read_file_range.clone())
                .tool(file_tools.replace_line_range.clone())
                .tool(file_tools.insert_lines.clone())
                .tool(file_tools.delete_line_range.clone())
                .tool(file_tools.append_to_file.clone())
                // 命令执行工具（用于 build/test/check 等验证）
                .tool(command_tools.run_command.clone())
                .build()?
        );
        
        // 创建独立的 Session
        let session_service = Arc::new(InMemorySessionService::new());
        let app_name = "cowork_batch_executor".to_string();
        let session_id = format!("batch_{}_{}", batch_idx, uuid::Uuid::new_v4());
        let user_id = "batch_executor".to_string();
        
        session_service.create(CreateRequest {
            app_name: app_name.clone(),
            user_id: user_id.clone(),
            session_id: Some(session_id.clone()),
            state: HashMap::new(),
        }).await?;
        
        let runner = Runner::new(RunnerConfig {
            app_name,
            agent,
            session_service,
            artifact_service: None,
            memory_service: None,
            run_config: None,
        })?;
        
        println!("🤖 Sub-Agent 开始执行批次 {}...\n", batch_idx + 1);
        
        // 执行
        let mut event_stream = runner.run(
            user_id,
            session_id,
            Content::new("user").with_text(&task_description)
        ).await?;
        
        while let Some(event_result) = event_stream.next().await {
            if let Err(e) = event_result {
                tracing::error!("Error in batch {}: {}", batch_idx, e);
                return Ok(BatchExecutionReport {
                    successful: 0,
                    failed: batch.len(),
                    details: vec![ChangeResult {
                        change: Change {
                            path: format!("batch_{}", batch_idx),
                            kind: "batch".to_string(),
                        },
                        status: ChangeStatus::Failed,
                        message: format!("Batch {} failed: {}", batch_idx, e),
                    }],
                });
            }
        }
        
        println!("✅ Sub-Agent 批次 {} 执行完成\n", batch_idx + 1);
        
        // 验证文件是否存在
        let mut successful = 0;
        let mut failed = 0;
        let mut details = Vec::new();
        
        for change in batch {
            let file_exists = std::path::Path::new(&change.path).exists();
            if file_exists {
                successful += 1;
                details.push(ChangeResult {
                    change: Change {
                        path: change.path.clone(),
                        kind: change.kind.clone(),
                    },
                    status: ChangeStatus::Success,
                    message: format!("File created: {}", change.path),
                });
            } else {
                failed += 1;
                details.push(ChangeResult {
                    change: Change {
                        path: change.path.clone(),
                        kind: change.kind.clone(),
                    },
                    status: ChangeStatus::Failed,
                    message: format!("File not found after execution: {}", change.path),
                });
            }
        }
        
        Ok(BatchExecutionReport {
            successful,
            failed,
            details,
        })
    }
    
    /// 单个 Agent 处理（原有逻辑，用于少量文件）
    async fn execute_single_agent(
        &self,
        code_artifact: &CodeChangeArtifact,
        _hitl: &HitlController,
        _prd_summary: Option<&str>,
        todo_list: Option<&mut TodoList>,
    ) -> Result<ExecutionReport> {
        // 创建文件操作工具
        let file_tools = create_file_tools();
        let command_tools = create_command_tools();

        // 构建任务描述
        let task_description = self.build_task_description(code_artifact);

        // 创建执行 Agent（带文件工具）
        let agent = Arc::new(
            LlmAgentBuilder::new("code_executor_agent")
                .description("AI agent that implements code changes by calling file tools")
                .instruction(&self.build_instruction(&code_artifact.data))
                .model(self.model.clone())
                .tool(file_tools.write_file.clone())
                .tool(file_tools.read_file.clone())
                .tool(file_tools.list_dir.clone())
                .tool(file_tools.file_exists.clone())
                .tool(file_tools.create_dir.clone())
                // 增量编辑工具（用于大文件）
                .tool(file_tools.read_file_range.clone())
                .tool(file_tools.replace_line_range.clone())
                .tool(file_tools.insert_lines.clone())
                .tool(file_tools.delete_line_range.clone())
                .tool(file_tools.append_to_file.clone())
                // 命令执行工具（用于 build/test/check 等验证）
                .tool(command_tools.run_command.clone())
                .build()?
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let app_name = "cowork_executor".to_string();
        let session_id = format!("exec_{}", uuid::Uuid::new_v4().to_string());
        let user_id = "code_executor".to_string();

        session_service.create(CreateRequest {
            app_name: app_name.clone(),
            user_id: user_id.clone(),
            session_id: Some(session_id.clone()),
            state: HashMap::new(),
        }).await?;

        let runner = Runner::new(RunnerConfig {
            app_name: app_name.clone(),
            agent,
            session_service: session_service.clone(),
            artifact_service: None,
            memory_service: None,
            run_config: None,
        })?;

        let input_content = Content::new("user").with_text(&task_description);

        println!("🤖 AI Agent 开始执行任务...\n");
        
        // 执行
        let mut event_stream = runner.run(user_id, session_id, input_content).await?;
        
        while let Some(event_result) = event_stream.next().await {
            if let Err(e) = event_result {
                tracing::error!("Error during execution: {}", e);
                return Err(anyhow::anyhow!("Code execution failed: {}", e));
            }
        }
        
        println!("✅ AI Agent 执行完成\n");

        // 验证文件是否存在
        let mut successful = 0;
        let mut failed = 0;
        let mut details = Vec::new();

        for change in &code_artifact.data.changes {
            let file_exists = std::path::Path::new(&change.path).exists();
            if file_exists {
                successful += 1;
                details.push(ChangeResult {
                    change: Change {
                        path: change.path.clone(),
                        kind: change.kind.clone(),
                    },
                    status: ChangeStatus::Success,
                    message: format!("File created: {}", change.path),
                });
            } else {
                failed += 1;
                details.push(ChangeResult {
                    change: Change {
                        path: change.path.clone(),
                        kind: change.kind.clone(),
                    },
                    status: ChangeStatus::Failed,
                    message: format!("File not found after execution: {}", change.path),
                });
            }
        }
        
        // 更新 TodoList（如果提供了）
        if let Some(todo_list) = todo_list {
            let successful_files: Vec<String> = details.iter()
                .filter(|d| d.status == ChangeStatus::Success)
                .map(|d| d.change.path.clone())
                .collect();
            
            let failed_files: Vec<String> = details.iter()
                .filter(|d| d.status == ChangeStatus::Failed)
                .map(|d| d.change.path.clone())
                .collect();
            
            crate::agents::TodoListManager::update_from_execution(
                todo_list,
                &code_artifact.data.changes,
                &successful_files,
                &failed_files,
            );
            
            // 打印 TodoList 状态
            crate::agents::TodoListManager::print_status(todo_list);
        }

        Ok(ExecutionReport {
            total_changes: code_artifact.data.changes.len(),
            successful,
            failed,
            skipped: 0,
            details,
        })
    }
    
    /// 构建原始需求摘要（用于 WatchDog）
    fn build_requirements_summary(&self, code_artifact: &CodeChangeArtifact) -> String {
        let lang = &code_artifact.data.target.lang;
        let stack = code_artifact.data.target.stack.join(", ");
        
        format!(
            "Target Language: {}\nTech Stack: {}\nTotal Files: {}",
            lang,
            stack,
            code_artifact.data.changes.len()
        )
    }
    
    /// 构建批次指令（集成 WatchDog 提醒和上下文传递）
    fn build_batch_instruction(
        &self, 
        target: &TargetProject, 
        file_count: usize, 
        original_requirements: Option<&str>,
        batch_context: &crate::agents::BatchContext
    ) -> String {
        // WatchDog 提醒
        let watchdog_reminder = if let Some(reqs) = original_requirements {
            format!(
                r#"

**⚠️  WATCHDOG REMINDER: Original User Requirements**
{}

**Self-Check Questions (review every 3 tool calls):**
1. Am I still aligned with the user's original requirements?
2. Am I generating files in the correct language ({})?
3. Am I creating production-ready code (no TODOs, no placeholders)?
"#,
                reqs,
                target.lang
            )
        } else {
            String::new()
        };
        
        // 上下文传递：使用详细的文件摘要
        let context_info = batch_context.generate_summary();
        
        format!(
            r#"You are a professional software developer.

**Your Task**: Implement {} code file(s) for a {} project.

**Technology Context**:
- Language: {}
- Tech Stack: {}
{}{}

**Instructions**:
1. For each file change:
   - Generate COMPLETE, PRODUCTION-READY code (no TODO, no placeholders)
   - Call write_file to save the code
   
2. File Size Strategy:
   - For small files (< 500 lines): use write_file with complete content
   - For large files (> 500 lines): use incremental tools (read_file_range, replace_line_range)
   
3. Code Quality:
   - Include all necessary imports and dependencies
   - Follow best practices for {}
   - Add clear comments
   - Code should be ready to run/compile
   
4. Consistency:
   - If referencing previously generated files, read them first to understand their structure
   - Maintain consistent naming, types, and patterns

5. **Progressive Verification (IMPORTANT - use run_command tool):**
   - After generating all files in this batch, VERIFY your work:
     a) If CodePlan provides "cmds", execute them in order using run_command
     b) If no cmds provided, auto-discover verification based on project type:
        * Node/JS/TS: check for package.json scripts (npm test, npm run build)
        * Python: try "python -m py_compile *.py" or "pytest"
        * Rust: try "cargo check" or "cargo build"
        * Other: check for Makefile, README instructions, or common CI patterns
     c) If verification fails:
        * Read the error output carefully
        * Identify which file(s) caused the error
        * Fix the file(s) and re-run verification
        * Retry up to 2 times per batch
     d) If verification passes: proceed to next batch
   
6. Work systematically through each file in the list.

**Available Tools:**
- write_file, read_file, list_directory, file_exists, create_dir
- Incremental editing: read_file_range, replace_line_range, insert_lines, delete_line_range, append_to_file
- **run_command(cmd, cwd, env)** - Execute shell commands for verification

IMPORTANT: This is a batch of {} files. Complete them, verify with run_command, then stop."#,
            file_count,
            target.lang,
            target.lang,
            target.stack.join(", "),
            watchdog_reminder,
            context_info,
            target.lang,
            file_count
        )
    }

    /// 构建 Agent 指令
    fn build_instruction(&self, code_plan: &CodeChange) -> String {
        let lang = &code_plan.target.lang;
        let tech_stack = code_plan.target.stack.join(", ");

        format!(
            r#"You are an expert software developer with access to file system tools AND command execution.

**Your Task:** Implement the code changes described by the user.

**Technology Context:**
- Language: {}
- Tech Stack: {}

**Available Tools:**
1. write_file(path, content) - Write complete code to a file
2. read_file(path) - Read entire file content
3. list_directory(path, recursive) - List files in a directory
4. file_exists(path) - Check if a file exists
5. create_dir(path, recursive) - Create directories

**For Large Files (to avoid context overflow):**
6. read_file_range(path, start_line, end_line) - Read specific lines
7. replace_line_range(path, start_line, end_line, new_content) - Replace specific lines
8. insert_lines(path, after_line, content) - Insert lines after a specific position
9. delete_line_range(path, start_line, end_line) - Delete specific lines
10. append_to_file(path, content) - Append to end of file

**For Verification:**
11. run_command(cmd, cwd, env) - Execute shell commands (build/test/check)

**Instructions:**
1. For each file change requested by the user:
   - If file is small (<500 lines): use write_file with complete code
   - If file is large (>500 lines): use incremental editing tools (read_file_range, replace_line_range, etc.)
   - Generate COMPLETE, WORKING code (no TODO comments, no placeholders)
   
2. Code Quality Requirements:
   - Write complete, working code that focuses on CORE functionality
   - Include all necessary imports and dependencies
   - Follow best practices for {}
   - Add clear comments for complex logic ONLY (avoid over-commenting)
   - The code should be ready to run immediately
   - **KEEP IT SIMPLE** - avoid over-engineering

3. For HTML files:
   - Include complete HTML5 structure
   - Embed CSS in <style> tags or separate file (keep it simple)
   - Add responsive design with meta viewport if needed
   - Include basic JavaScript if needed (no complex frameworks unless required)

4. For configuration files:
   - Use appropriate format (JSON, TOML, etc.)
   - Include ONLY necessary fields
   - Avoid adding unused configurations

5. **Simplicity Guidelines (IMPORTANT):**
   - Do NOT add testing frameworks, test files, or test infrastructure unless explicitly requested
   - Do NOT add CI/CD configurations, GitHub Actions, or deployment scripts
   - Do NOT add linting configurations, formatters, or code quality tools
   - Do NOT add logging frameworks, monitoring, or analytics unless required
   - Focus ONLY on making the core functionality work
   - User can add these later if needed

5. **Progressive Verification (OPTIONAL - Keep It Simple):**
   After generating all files, you MAY verify your work using run_command:
   a) If CodePlan provided verification commands ("cmds"), execute them in priority order
   b) For simple projects, verification may not be necessary
   c) If verification fails:
      * Analyze error output to identify problematic files
      * Fix the issues
      * Re-run verification (max 2 retries)
   d) Only declare success after verification passes OR max retries reached

6. Work systematically:
   - Process one file at a time
   - Confirm each file is written before moving to the next
   - If you encounter errors, explain what went wrong
   - Focus on making code work, not making it perfect

**IMPORTANT:**
- Generate REAL, WORKING code - not templates, not TODOs
- Use the write_file tool to save every file
- Focus on SIMPLICITY and FUNCTIONALITY
- Avoid adding unnecessary complexity (testing, monitoring, etc.)"#,
            lang,
            tech_stack,
            lang
        )
    }

    /// 构建任务描述
    fn build_task_description(&self, code_artifact: &CodeChangeArtifact) -> String {
        let changes_list = code_artifact.data.changes.iter()
            .map(|change| {
                format!("- [{}] {}: {}", change.kind, change.path, change.note)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Please implement the following code changes:

{}

For each file:
1. Generate complete, production-ready code based on the description
2. Use write_file tool to save the code to the specified path
3. Ensure all code is complete and ready to run

Start implementing now. Work through each file systematically."#,
            changes_list
        )
    }
}

/// 执行报告
#[derive(Debug, Clone)]
pub struct ExecutionReport {
    pub total_changes: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub details: Vec<ChangeResult>,
}

/// 单个变更的执行结果
#[derive(Debug, Clone)]
pub struct ChangeResult {
    pub change: Change,
    pub status: ChangeStatus,
    pub message: String,
}

/// 变更状态
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeStatus {
    Success,
    Failed,
    Skipped,
}

/// 简化的变更信息（用于报告）
#[derive(Debug, Clone)]
pub struct Change {
    pub path: String,
    pub kind: String,
}

/// 批次执行报告（内部使用）
#[derive(Debug)]
struct BatchExecutionReport {
    successful: usize,
    failed: usize,
    details: Vec<ChangeResult>,
}
