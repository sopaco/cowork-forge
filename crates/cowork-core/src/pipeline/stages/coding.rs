use std::sync::Arc;

use crate::agents::{ExternalCodingAgent, StreamingTask};
use crate::interaction::{InteractiveBackend, MessageContext, MessageLevel};
use crate::llm::config::load_config;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::coding::CODING_ACTOR_INSTRUCTION;
use crate::pipeline::stage_executor::{execute_stage_with_instruction, execute_stage_with_instruction_and_context};
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
        // Set iteration ID for storage operations (must be set before any storage access)
        crate::persistence::set_iteration_id(ctx.iteration.id.clone());
        
        let workspace = ctx.workspace_path.clone();
        
        interaction
            .show_message_with_context(
                MessageLevel::Info,
                "🚀 Using External Coding Agent (ACP)".to_string(),
                MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
            )
            .await;

        // Build task description
        // Priority: parameter feedback (from executor) > stored feedback > plan artifact
        // Note: Executor now auto-loads feedback from storage before calling execute_with_feedback
        let task_description = if let Some(fb) = feedback {
            // Use parameter feedback (from executor auto-load or stage review loop)
            tracing::debug!("using parameter feedback: {}", fb.chars().take(100).collect::<String>());
            format!(
                "## ⚠️ USER REPORTED ISSUE - REQUIRES FIX\n\n\
                The user has reported the following problems with the project:\n\n\
                \"\"\"\n{}\n\"\"\"\n\n\
                ## Your Task\n\
                1. Read and understand the user's issues above\n\
                2. Find the relevant code files\n\
                3. Fix each issue one by one\n\
                4. Verify your fixes work correctly",
                fb
            )
        } else {
            // Try to load feedback from storage as fallback (for edge cases)
            let stored_feedback = crate::persistence::load_feedback_history()
                .ok()
                .and_then(|history| {
                    history.feedbacks
                        .into_iter()
                        .filter(|f| f.stage == "coding")
                        .max_by_key(|f| f.timestamp)
                });

            if let Some(ref fb) = stored_feedback {
                tracing::debug!("found fallback feedback from storage: {}", fb.details.chars().take(100).collect::<String>());
                format!("Fix issues based on feedback: {}", fb.details)
            } else {
                // Load plan artifact to get tasks
                tracing::debug!("no feedback found, loading plan...");
                let iteration_dir = workspace.parent().unwrap_or(&workspace);
                let plan_artifact = iteration_dir.join("artifacts").join("plan.md");
                
                if let Ok(content) = std::fs::read_to_string(&plan_artifact) {
                    format!("Implement the tasks from the plan:\n\n{}", content)
                } else {
                    "Implement the planned features.".to_string()
                }
            }
        };

        // Build project context
        let project_context = format!(
            "Project: {}\nDescription: {}",
            ctx.iteration.title,
            ctx.iteration.description
        );

        // Create external agent with iteration context for evolution iterations
        tracing::debug!(workspace = %workspace.display(), "creating ExternalCodingAgent");
        tracing::debug!(iteration_id = %ctx.iteration.id, base_id = ?ctx.iteration.base_iteration_id, inheritance = ?ctx.iteration.inheritance, "iteration context");
        let agent = match ExternalCodingAgent::new_with_iteration(&workspace, Some(ctx.iteration.clone())).await {
            Ok(agent) => agent,
            Err(e) => {
                interaction
                    .show_message_with_context(
                        MessageLevel::Error,
                        format!("Failed to start external agent: {}", e),
                        MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
                    )
                    .await;
                // Fall back to built-in agent. Preserve the external agent's
                // task description (plan + feedback context) so the built-in
                // agent does not start from a blank slate.
                tracing::warn!("Falling back to built-in coding agent");
                let fallback_feedback = feedback;
                return execute_stage_with_instruction_and_context(
                    ctx,
                    interaction,
                    "coding",
                    CODING_ACTOR_INSTRUCTION,
                    fallback_feedback,
                    Some(&task_description),
                )
                .await;
            }
        };

        // Create message context for external agent
        let ctx_external = MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding");

        // Execute with streaming messages
        let StreamingTask { mut messages, result } = agent.execute_task_stream(&task_description, &project_context);

        // Display messages in real-time while waiting for result
        let interaction_clone = interaction.clone();
        
        // Use tokio::spawn with scoped lifetime to handle the receiver properly.
        // Note: messages is UnboundedReceiver, we need to use it in the same runtime.
        //
        // Track completion via a Notify so the outer wait can distinguish "agent still
        // working" from "agent finished, cleanup pending". This is critical: a fixed
        // timeout on result.await would kill legitimate long-running tasks (e.g., when
        // the user is scanning a QR code for auth, or the agent is producing a lot of
        // output). We only want a short cleanup timeout AFTER Completed is received.
        let completed_received = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let completed_flag = completed_received.clone();
        let completed_notify = Arc::new(tokio::sync::Notify::new());
        let completed_notify_clone = completed_notify.clone();

        let message_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = messages.recv() => {
                        match msg {
                            Some(AgentMessage::Thinking(text)) => {
                                // Stream thinking directly — the frontend aggregates
                                // chunks into a single collapsible thinking message.
                                // No truncation, no prefix labels.
                                if !text.is_empty() {
                                    interaction_clone
                                        .send_streaming(text, AGENT_NAME_EXTERNAL, true)
                                        .await;
                                }
                            }
                            Some(AgentMessage::Output(text)) => {
                                // Stream output text directly — the frontend aggregates
                                // chunks into one streaming agent message. No truncation,
                                // no "📝 Output:" prefix (that prefix was the cause of the
                                // ugly duplicated labels mid-paragraph reported by users).
                                if !text.is_empty() {
                                    interaction_clone
                                        .send_streaming(text, AGENT_NAME_EXTERNAL, false)
                                        .await;
                                }
                            }
                            Some(AgentMessage::Status(text)) => {
                                // Brief, discrete status line — keep as a separate Info
                                // message so it shows as a distinct UI element.
                                interaction_clone.show_message_with_context(MessageLevel::Info, format!("⏳ {}", text), ctx_external.clone()).await;
                            }
                            Some(AgentMessage::Error(text)) => {
                                interaction_clone.show_message_with_context(MessageLevel::Error, format!("❌ {}", text), ctx_external.clone()).await;
                            }
                            Some(AgentMessage::Completed) => {
                                interaction_clone.show_message_with_context(MessageLevel::Info, "✅ Task completed".to_string(), ctx_external.clone()).await;
                                // Record that Completed was received so the outer code
                                // can fall back to Success if result.await hangs.
                                completed_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                                // Wake the outer select! so it switches from "wait for
                                // agent" to "wait for cleanup". This is the key signal:
                                // before this fires, the outer code waits indefinitely
                                // (no spurious timeout); after, it gives the result
                                // future a short window to clean up.
                                completed_notify_clone.notify_one();
                                // Exit the loop — the ACP client sends Completed after
                                // the prompt finishes and the agent process is cleaned up.
                                break;
                            }
                            None => {
                                // Channel closed, exit loop
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        // Idle heartbeat — no recent message. Keep as a discrete Info
                        // line so the user knows the agent is still working.
                        interaction_clone.show_message_with_context(MessageLevel::Info, "⏳ Waiting for agent...".to_string(), ctx_external.clone()).await;
                    }
                }
            }
        });

        // Wait for the result future. CRITICAL: do NOT apply a short timeout here.
        // The agent may legitimately run for a long time (auth QR scan, long output,
        // file operations). The ACP SDK already has a 3000s timeout on conn.prompt()
        // (PROMPT_TIMEOUT_SECONDS) which will return Err and unblock `result`.
        //
        // We use a select! between:
        //   (a) result returns on its own — normal completion or ACP-level error
        //   (b) Completed notification fires — agent finished, give cleanup a short
        //       window then proceed to Success even if result stalls (teardown bug)
        // Before (b) fires, there is NO timeout — we wait as long as the agent needs.
        tracing::info!("Awaiting external agent result (no timeout until Completed)");
        let mut result = std::pin::pin!(result);

        let outcome = tokio::select! {
            // (a) result returned first: either the agent finished cleanly and the
            // ACP thread exited, or the ACP SDK's own timeout fired (→ Err).
            res = &mut result => {
                tracing::info!("External agent result received directly");
                res
            }
            // (b) AgentMessage::Completed was received — the agent's work is done.
            // Give the result future a short window to wrap up (process kill,
            // stderr drain, runtime teardown). If it doesn't return in time, the
            // stall is in cleanup, not in the agent's work — proceed to Success.
            _ = completed_notify.notified() => {
                tracing::info!("Completed received, waiting for result cleanup (30s)");
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(30),
                    &mut result,
                ).await {
                    Ok(res) => {
                        tracing::info!("Result cleanup completed within window");
                        res
                    }
                    Err(_) => {
                        // Cleanup stalled after Completed — the task itself finished.
                        // Proceed to Success so the user can review via HITL.
                        tracing::warn!("Result cleanup timed out after Completed, proceeding to Success");
                        interaction
                            .show_message_with_context(
                                MessageLevel::Info,
                                "External coding agent completed (cleanup timed out, proceeding)".to_string(),
                                MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
                            )
                            .await;
                        // Drain the message loop (it already exited on Completed).
                        let _ = tokio::time::timeout(
                            tokio::time::Duration::from_secs(5),
                            message_handle,
                        ).await;
                        return StageResult::Success(None);
                    }
                }
            }
        };

        // Handle the result (from either branch above).
        match outcome {
            Ok(Ok(_output)) => {
                tracing::info!("External agent result Ok");
                let _ = tokio::time::timeout(
                    tokio::time::Duration::from_secs(10),
                    message_handle,
                ).await;
                interaction
                    .show_message_with_context(
                        MessageLevel::Info,
                        "External coding agent completed successfully".to_string(),
                        MessageContext::new(AGENT_NAME_EXTERNAL).with_stage("coding"),
                    )
                    .await;
                StageResult::Success(None)
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "External agent returned error");
                message_handle.abort();
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
            Err(e) => {
                tracing::warn!(error = %e, "External agent channel error");
                message_handle.abort();
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