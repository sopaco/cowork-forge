use std::sync::Arc;

use crate::agents::ExternalCodingAgent;
use crate::interaction::{InteractiveBackend, MessageLevel};
use crate::llm::config::load_config;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::coding::CODING_ACTOR_INSTRUCTION;
use crate::pipeline::stage_executor::execute_stage_with_instruction;

/// Coding Stage - Generate code implementation using Agent with Instructions + Tools
/// 
/// This stage supports two modes:
/// 1. Built-in Agent: Uses adk-rust based coding agent (default)
/// 2. External Agent: Uses external CLI-based agent via ACP (when configured)
pub struct CodingStage;

impl CodingStage {
    /// Check if external coding agent is enabled
    fn is_external_enabled() -> bool {
        match load_config() {
            Ok(config) => config.coding_agent.enabled,
            Err(_) => false,
        }
    }

    /// Execute using external coding agent via ACP
    async fn execute_external(
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
        feedback: Option<&str>,
    ) -> StageResult {
        let workspace = ctx.workspace_path.clone();
        
        interaction
            .show_message(
                MessageLevel::Info,
                "🚀 Using External Coding Agent (ACP)".to_string(),
            )
            .await;

        // Build task description
        let task_description = if let Some(fb) = feedback {
            format!("Fix issues based on feedback: {}", fb)
        } else {
            // Load plan artifact to get tasks
            let iteration_dir = workspace.parent().unwrap_or(&workspace);
            let plan_artifact = format!("{}/artifacts/plan.md", iteration_dir.display());
            
            if let Ok(content) = std::fs::read_to_string(&plan_artifact) {
                format!("Implement the tasks from the plan:\n\n{}", content)
            } else {
                "Implement the planned features.".to_string()
            }
        };

        // Build project context
        let project_context = format!(
            "Project: {}\nDescription: {}",
            ctx.iteration.title,
            ctx.iteration.description
        );

        // Create and execute external agent
        eprintln!("DEBUG: Creating ExternalCodingAgent for workspace: {}", workspace.display());
        match ExternalCodingAgent::new(&workspace).await {
            Ok(mut agent) => {
                let result = agent.execute_task(&task_description, &project_context).await;
                
                match result {
                    Ok(task_result) => {
                        if let Some(err) = task_result.error {
                            interaction
                                .show_message(
                                    MessageLevel::Error,
                                    format!("External agent error: {}", err),
                                )
                                .await;
                            return StageResult::Failed(err);
                        }
                        
                        interaction
                            .show_message(
                                MessageLevel::Info,
                                "External coding agent completed successfully".to_string(),
                            )
                            .await;
                        StageResult::Success(None)
                    }
                    Err(e) => {
                        let error_msg = format!("External agent error details: {}", e);
                        interaction
                            .show_message(
                                MessageLevel::Error,
                                error_msg.clone(),
                            )
                            .await;
                        StageResult::Failed(e.to_string())
                    }
                }
            }
            Err(e) => {
                interaction
                    .show_message(
                        MessageLevel::Error,
                        format!("Failed to start external agent: {}", e),
                    )
                    .await;
                // Fall back to built-in agent
                tracing::warn!("Falling back to built-in coding agent");
                if let Some(fb) = feedback {
                    execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, Some(fb)).await
                } else {
                    execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, None).await
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Stage for CodingStage {
    fn name(&self) -> &str {
        "coding"
    }

    fn description(&self) -> &str {
        "Coding - Generate code implementation using Agent with Memory and Tools"
    }

    fn needs_confirmation(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        // Check if external coding agent is enabled
        if Self::is_external_enabled() {
            return Self::execute_external(ctx, interaction, None).await;
        }
        
        execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, None).await
    }

    async fn execute_with_feedback(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
        feedback: &str,
    ) -> StageResult {
        interaction
            .show_message(
                MessageLevel::Info,
                "Regenerating code based on your feedback...".to_string(),
            )
            .await;

        // Check if external coding agent is enabled
        if Self::is_external_enabled() {
            return Self::execute_external(ctx, interaction, Some(feedback)).await;
        }
        
        execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, Some(feedback)).await
    }
}