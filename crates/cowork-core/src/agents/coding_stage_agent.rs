use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::artifacts::{Stage, PRDArtifact, DesignDocArtifact, PlanArtifact};
use crate::memory::ArtifactStore;
use crate::config::LlmConfig;
use crate::agents::{StageAgent, StageAgentContext, StageAgentResult, CodePlanner, CodeExecutor};
use crate::utils;

/// Coding Stage Agent - 代码生成阶段（包装 CodePlanner + CodeExecutor）
pub struct CodingStageAgent {
    code_planner: CodePlanner,
    llm_config: LlmConfig,
}

impl CodingStageAgent {
    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        let code_planner = CodePlanner::new(llm_config, store)?;
        Ok(Self {
            code_planner,
            llm_config: llm_config.clone(),
        })
    }
}

#[async_trait]
impl StageAgent for CodingStageAgent {
    fn stage(&self) -> Stage {
        Stage::Coding
    }
    
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult> {
        // 1. 加载依赖的 artifacts
        let prd_artifact: PRDArtifact = context.load_artifact(Stage::Requirements)?;
        let design_artifact: DesignDocArtifact = context.load_artifact(Stage::Design)?;
        let mut plan_artifact: PlanArtifact = context.load_artifact(Stage::Plan)?;
        
        // 2. 生成代码变更计划
        let code_artifact = self.code_planner.execute(
            &context.session_id,
            &prd_artifact,
            &design_artifact,
            &plan_artifact
        ).await?;
        
        println!("\n📋 代码变更计划：");
        println!("  语言: {}", code_artifact.data.target.lang);
        println!("  文件数: {}", code_artifact.data.changes.len());
        println!("  命令数: {}", code_artifact.data.cmds.len());
        
        // 3. 询问是否执行代码生成
        let mut execution_verified = false;
        if context.hitl.confirm("是否执行代码变更（AI 自动生成并写入文件）？")? {
            println!("\n🤖 开始 AI 代码生成...\n");
            
            let executor = CodeExecutor::new(&self.llm_config)?;
            let prd_summary = utils::extract_prd_summary(&prd_artifact);
            let mut todo_list = plan_artifact.data.todo_list.clone();
            
            match executor.execute_with_todo(
                &code_artifact,
                context.hitl.as_ref(),
                Some(&prd_summary),
                todo_list.as_mut(),
            ).await {
                Ok(report) => {
                    println!("\n代码生成完成:");
                    println!("  ✅ 成功: {}", report.successful);
                    println!("  ❌ 失败: {}", report.failed);
                    println!("  ⏭️  跳过: {}", report.skipped);
                    
                    execution_verified = report.failed == 0 && report.successful > 0;
                    
                    // 保存更新后的 TodoList
                    if let Some(updated_todo_list) = todo_list {
                        plan_artifact.data.todo_list = Some(updated_todo_list);
                        context.store.put(&context.session_id, Stage::Plan, &plan_artifact)?;
                    }
                }
                Err(e) => {
                    tracing::error!("Code execution failed: {}", e);
                    return Err(e);
                }
            }
        } else {
            println!("⏭️  跳过代码生成，仅保留计划（未验证）");
        }
        
        // 4. 返回结果
        let summary = vec![
            format!("Language: {}", code_artifact.data.target.lang),
            format!("Changes: {}", code_artifact.data.changes.len()),
            format!("Commands: {}", code_artifact.data.cmds.len()),
            format!("Verified: {}", if execution_verified { "Yes" } else { "No" }),
        ];
        
        Ok(StageAgentResult::new(code_artifact.meta.artifact_id, Stage::Coding)
            .with_verified(execution_verified)
            .with_summary(summary))
    }
    
    fn dependencies(&self) -> Vec<Stage> {
        vec![Stage::Requirements, Stage::Design, Stage::Plan]
    }
    
    fn requires_hitl_review(&self) -> bool {
        true
    }
    
    fn description(&self) -> &str {
        "生成代码变更计划并执行代码生成"
    }
}
