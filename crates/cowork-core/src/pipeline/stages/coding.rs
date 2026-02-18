use std::sync::Arc;

use crate::agents::{ExternalCodingAgent, StreamingTask};
use crate::interaction::{InteractiveBackend, MessageContext, MessageLevel};
use crate::llm::config::load_config;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::coding::CODING_ACTOR_INSTRUCTION;
use crate::pipeline::stage_executor::execute_stage_with_instruction;
use crate::acp::AgentMessage;

/// Coding Stage - Generate code implementation using Agent with Instructions + Tools
/// 
/// This stage supports two modes:
/// 1. Built-in Agent: Uses adk-rust based coding agent (default)
/// 2. External Agent: Uses external CLI-based agent via ACP (when configured)
pub struct CodingStage;

/// Agent names for message context
const AGENT_NAME_BUILTIN: &str = "Code Agent";
const AGENT_NAME_EXTERNAL: &str = "Code Agent (External)";

impl CodingStage {
    /// Check if external coding agent is enabled
    fn is_external_enabled() -> bool {
        match load_config() {
            Ok(config) => config.coding_agent.enabled,
            Err(_) => false,
        }
    }

    /// Execute using external coding agent via ACP with streaming messages
    async fn execute_external(
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
        feedback: Option<&str>,
    ) -> StageResult {
        let workspace = ctx.workspace_path.clone();
        
        interaction
            .show_message_with_context(
                MessageLevel::Info,
                "🚀 Using External Coding Agent (ACP)".to_string(),
                MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
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

        // Create external agent
        eprintln!("DEBUG: Creating ExternalCodingAgent for workspace: {}", workspace.display());
        let agent = match ExternalCodingAgent::new(&workspace).await {
            Ok(agent) => agent,
            Err(e) => {
                interaction
                    .show_message_with_context(
                        MessageLevel::Error,
                        format!("Failed to start external agent: {}", e),
                        MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
                    )
                    .await;
                // Fall back to built-in agent
                tracing::warn!("Falling back to built-in coding agent");
                return if let Some(fb) = feedback {
                    execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, Some(fb)).await
                } else {
                    execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, None).await
                };
            }
        };

        // Create message context for external agent
        let ctx_external = MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding");

        // Execute with streaming messages
        let StreamingTask { mut messages, result } = agent.execute_task_stream(&task_description, &project_context);

        // Display messages in real-time while waiting for result
        let interaction_clone = interaction.clone();
        
        // Use tokio::spawn with scoped lifetime to handle the receiver properly
        // Note: messages is UnboundedReceiver, we need to use it in the same runtime
        let message_handle = tokio::spawn(async move {
            let mut thinking_buffer = String::new();
            let mut output_buffer = String::new();
            
            loop {
                tokio::select! {
                    msg = messages.recv() => {
                        match msg {
                            Some(AgentMessage::Thinking(text)) => {
                                // Accumulate thinking for display
                                thinking_buffer.push_str(&text);
                                // Show thinking as it comes (truncated for UI)
                                if thinking_buffer.len() > 100 {
                                    let display = format!("💭 Thinking: {}...", &thinking_buffer[..100]);
                                    interaction_clone.show_message_with_context(MessageLevel::Info, display, ctx_external.clone()).await;
                                    thinking_buffer.clear();
                                }
                            }
                            Some(AgentMessage::Output(text)) => {
                                output_buffer.push_str(&text);
                                // Show significant output chunks
                                if output_buffer.len() > 200 {
                                    let display = format!("📝 Output: {}...", &output_buffer[..200.min(output_buffer.len())]);
                                    interaction_clone.show_message_with_context(MessageLevel::Info, display, ctx_external.clone()).await;
                                    output_buffer.clear();
                                }
                            }
                            Some(AgentMessage::Status(text)) => {
                                interaction_clone.show_message_with_context(MessageLevel::Info, format!("⏳ {}", text), ctx_external.clone()).await;
                            }
                            Some(AgentMessage::Error(text)) => {
                                interaction_clone.show_message_with_context(MessageLevel::Error, format!("❌ {}", text), ctx_external.clone()).await;
                            }
                            Some(AgentMessage::Completed) => {
                                interaction_clone.show_message_with_context(MessageLevel::Info, "✅ Task completed".to_string(), ctx_external.clone()).await;
                            }
                            None => {
                                // Channel closed, exit loop
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        // Timeout after 60 seconds of no messages
                        interaction_clone.show_message_with_context(MessageLevel::Info, "⏳ Waiting for agent...".to_string(), ctx_external.clone()).await;
                    }
                }
            }
        });

        // Wait for result - result is Result<Result<String>>
        match result.await {
            // Inner Ok: ACP execution succeeded
            Ok(Ok(_output)) => {
                // Wait for message handling to finish
                let _ = message_handle.await;
                
                interaction
                    .show_message_with_context(
                        MessageLevel::Info,
                        "External coding agent completed successfully".to_string(),
                        MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
                    )
                    .await;
                StageResult::Success(None)
            }
            // Inner Err: ACP execution failed
            Ok(Err(e)) => {
                let error_msg = format!("External agent execution error: {}", e);
                interaction
                    .show_message_with_context(
                        MessageLevel::Error,
                        error_msg.clone(),
                        MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
                    )
                    .await;
                StageResult::Failed(e.to_string())
            }
            // Outer Err: Channel/thread error
            Err(e) => {
                let error_msg = format!("External agent error: {}", e);
                interaction
                    .show_message_with_context(
                        MessageLevel::Error,
                        error_msg.clone(),
                        MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
                    )
                    .await;
                StageResult::Failed(e.to_string())
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
        // Determine which agent is being used
        let agent_name = if Self::is_external_enabled() {
            AGENT_NAME_EXTERNAL
        } else {
            AGENT_NAME_BUILTIN
        };

        interaction
            .show_message_with_context(
                MessageLevel::Info,
                "Regenerating code based on your feedback...".to_string(),
                MessageContext::new(agent_name).with_stage("coding"),
            )
            .await;

        // Check if external coding agent is enabled
        if Self::is_external_enabled() {
            return Self::execute_external(ctx, interaction, Some(feedback)).await;
        }
        
        execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, Some(feedback)).await
    }
}