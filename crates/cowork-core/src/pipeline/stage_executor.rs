// Stage Executor - Configuration-driven stage execution
//
// This module provides a unified execution framework for all stages:
// - Uses configuration registry to create agents
// - Supports both Simple and Actor-Critic stage types
// - Handles feedback and iteration
// - Saves artifacts
// - Sends real-time streaming output

use crate::config::{get_language_instruction};
use crate::config_definition::{global_registry, create_agent_for_stage};
use crate::interaction::{InteractiveBackend, MessageContext};
use crate::llm::{create_llm_client, get_execution_llm};
use crate::llm::config::load_config;
use crate::pipeline::{PipelineContext, StageResult, clear_goto_stage_signal, take_goto_stage_signal};
use crate::persistence::{set_iteration_id, load_feedback_history};
use crate::tools::set_current_agent_name;
use adk_core::{Content, Event};
use futures::StreamExt;
use std::sync::Arc;

fn check_event_for_goto_stage(event: &Event) -> Option<(String, String)> {
    if event.actions.escalate {
        if let Some(target) = event.actions.state_delta.get("goto_stage").and_then(|v| v.as_str()) {
            let reason = event.actions.state_delta.get("goto_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("Stage jump requested");
            return Some((target.to_string(), reason.to_string()));
        }
    }
    take_goto_stage_signal()
}

/// Map stage name to the corresponding save tool name
fn get_save_tool_name(stage_name: &str) -> &'static str {
    match stage_name {
        "idea" => "save_idea",
        "prd" => "save_prd_doc",
        "design" => "save_design_doc",
        "plan" => "save_plan_doc",
        "check" => "save_check_report",
        "delivery" => "save_delivery_report",
        _ => "save_idea", // fallback
    }
}

/// Map stage name to the artifact filename used by save_* tools in persistence/iteration_data.rs
/// Returns None for stages that don't have a single artifact file (e.g., coding)
fn get_artifact_filename(stage_name: &str) -> Option<&'static str> {
    match stage_name {
        "idea" => Some("idea.md"),
        "prd" => Some("prd.md"),
        "design" => Some("design.md"),
        "plan" => Some("plan.md"),
        "check" => Some("check_report.md"),
        "delivery" => Some("delivery_report.md"),
        "coding" => None, // Coding stage doesn't have a single artifact file
        _ => None,
    }
}

/// Map internal agent names to user-friendly display names
fn get_display_name(agent_name: &str) -> String {
    // Try to get agent name from registry first
    let registry = global_registry();
    if let Some(agent_def) = registry.get_agent(agent_name) {
        return agent_def.name.clone();
    }

    // Fallback to hardcoded names for system agents
    match agent_name {
        // System agents
        "Pipeline Controller" => "Pipeline".to_string(),
        "Memory System" => "Memory".to_string(),
        "Knowledge System" => "Knowledge".to_string(),
        // Fallback
        _ => format!("{} Agent", agent_name.replace("_", " ").split_whitespace().map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }).collect::<Vec<_>>().join(" ")),
    }
}

fn check_pending_critic_feedback(stage_name: &str) -> Option<String> {
    if let Ok(history) = load_feedback_history() {
        if let Some(fb) = history.feedbacks.iter()
            .filter(|f| f.stage == stage_name)
            .max_by_key(|f| f.timestamp)
        {
            return Some(fb.details.clone());
        }
    }
    None
}

/// Execute a stage using real adk-rust Agent
pub async fn execute_stage_with_instruction(
    ctx: &PipelineContext,
    interaction: Arc<dyn InteractiveBackend>,
    stage_name: &str,
    instruction: &str,
    feedback: Option<&str>,
) -> StageResult {
    execute_stage_with_instruction_and_context(ctx, interaction, stage_name, instruction, feedback, None).await
}

pub async fn execute_stage_with_instruction_and_context(
    ctx: &PipelineContext,
    interaction: Arc<dyn InteractiveBackend>,
    stage_name: &str,
    _instruction: &str,
    feedback: Option<&str>,
    extra_context: Option<&str>,
) -> StageResult {
    // Set iteration ID for data tools (V2 architecture)
    set_iteration_id(ctx.iteration.id.clone());
    clear_goto_stage_signal();

    // Check for restart mode (GotoStage mechanism)
    if let Ok(Some(session_meta)) = crate::persistence::load_session_meta()
        && let Some(restart_reason) = session_meta.restart_reason
    {
        // This is a restart from a previous stage
        interaction
            .show_message(
                crate::interaction::MessageLevel::Warning,
                format!(
                    "🔄 RESTART MODE: Restarting {} stage due to: {}",
                    stage_name, restart_reason
                ),
            )
            .await;

        // Clear the restart reason after displaying it
        if let Ok(mut meta) = crate::persistence::load_session_meta()
            && let Some(ref mut m) = meta
        {
            m.restart_reason = None;
            let _ = crate::persistence::save_session_meta(m);
        }
    }

    // Wrap everything in an async block to handle ? operator
    let result = async {
        // Get iteration directory (parent of workspace)
        // workspace_path is .cowork-v2/iterations/{id}/workspace
        // artifacts should be in .cowork-v2/iterations/{id}/artifacts
        let iteration_dir = ctx.workspace_path.parent().unwrap_or(&ctx.workspace_path);

        // Ensure artifacts directory exists - use PathBuf for proper path handling
        let artifacts_dir = iteration_dir.join("artifacts");
        if let Err(e) = std::fs::create_dir_all(&artifacts_dir) {
            return Err(format!("Failed to create artifacts directory: {}", e));
        }

        // Prepare artifact path - V2 architecture: .cowork-v2/iterations/{iteration_id}/artifacts/{artifact_filename}
        // Must match the filename used by the save_* tools in persistence/iteration_data.rs
        // Some stages (e.g., coding) don't have a single artifact file
        let artifact_filename = get_artifact_filename(stage_name);
        let artifact_path = artifact_filename.map(|f| artifacts_dir.join(f));

        // Load LLM client - reuse execution-scoped client if available, otherwise create new
        let model = if let Some(cached) = get_execution_llm() {
            cached
        } else {
            let llm_config = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
            create_llm_client(&llm_config.llm)
                .map_err(|e| format!("Failed to create LLM client: {}", e))?
        };

        // Create agent using configuration registry
        let agent = create_agent_for_stage(stage_name, model, ctx.iteration.id.clone())
            .map_err(|e| format!("Failed to create agent for stage '{}': {}", stage_name, e))?;

        Ok((agent, artifact_path))
    }
    .await;

    let (agent, artifact_path) = match result {
        Ok(v) => v,
        Err(e) => return StageResult::Failed(e),
    };

    // Get the actual agent name and map to user-friendly display name
    let internal_name = agent.name();
    let display_name = get_display_name(internal_name);
    set_current_agent_name(&display_name);

    // Build prompt with context
    let prompt = build_prompt(ctx, stage_name, feedback, extra_context);

    // Execute agent - send start notification with user-friendly name
    let status_msg = if feedback.is_some() {
        format!("Regenerating {}...", stage_name.to_uppercase())
    } else {
        format!("Generating {}...", stage_name.to_uppercase())
    };

    interaction
        .show_message_with_context(
            crate::interaction::MessageLevel::Info,
            status_msg,
            MessageContext::new(&display_name).with_stage(stage_name),
        )
        .await;

    // Create initial content with the prompt
    let initial_content = Content::new("user").with_text(prompt);

    // Execute agent - Agent::run() takes Arc<dyn InvocationContext>
    let invocation_ctx = Arc::new(SimpleInvocationContext::new(
        ctx,
        &initial_content,
        agent.clone(),
    ));
    let stream = match agent.run(invocation_ctx).await {
        Ok(s) => s,
        Err(e) => {
            let err_msg = format!("{}", e);
            if err_msg.contains("GOTO_STAGE_REQUESTED") {
                if let Some((target_stage, reason)) = take_goto_stage_signal() {
                    interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            format!("🔄 Stage jump requested: {} → {}", stage_name, target_stage),
                            MessageContext::new(&display_name).with_stage(stage_name),
                        )
                        .await;
                    return StageResult::GotoStage(target_stage, reason);
                }
            }
            return StageResult::Failed(format!("Agent execution failed: {}", e));
        }
    };

    let mut generated_text = String::new();
    let mut event_count = 0u32;
    let mut text_event_count = 0u32;
    let mut tool_call_count = 0u32;

    let mut stream = std::pin::pin!(stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                event_count += 1;
                if let Some(content) = event.content() {
                    if let Some(text) = extract_text_from_content(content) {
                        if !text.trim().is_empty() {
                            text_event_count += 1;
                            generated_text.push_str(&text);
                            interaction
                                .send_streaming(text.clone(), &display_name, false)
                                .await;
                        }
                    } else {
                        tool_call_count += 1;
                        tracing::debug!(
                            "[StageExecutor] Event #{} has content but no text part (likely tool call)",
                            event_count
                        );
                    }
                } else if let Some(text) = extract_text_from_event(&event) {
                    if !text.trim().is_empty() {
                        text_event_count += 1;
                        generated_text.push_str(&text);
                        interaction.send_streaming(text, &display_name, false).await;
                    }
                }

                if let Some((target_stage, reason)) = check_event_for_goto_stage(&event) {
                    interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            format!("🔄 Stage jump requested: {} → {}", stage_name, target_stage),
                            MessageContext::new(&display_name).with_stage(stage_name),
                        )
                        .await;
                    return StageResult::GotoStage(target_stage, reason);
                }
            }
            Err(e) => {
                let err_msg = format!("{}", e);
                if err_msg.contains("GOTO_STAGE_REQUESTED") {
                    if let Some((target_stage, reason)) = take_goto_stage_signal() {
                        interaction
                            .show_message_with_context(
                                crate::interaction::MessageLevel::Warning,
                                format!("🔄 Stage jump requested: {} → {}", stage_name, target_stage),
                                MessageContext::new(&display_name).with_stage(stage_name),
                            )
                            .await;
                        return StageResult::GotoStage(target_stage, reason);
                    }
                }

                interaction
                    .show_message_with_context(
                        crate::interaction::MessageLevel::Error,
                        format!("Stream error: {}", e),
                        MessageContext::new(&display_name).with_stage(stage_name),
                    )
                    .await;
            }
        }
    }

    // Send completion notification
    if generated_text.is_empty() {
        // Check if the agent saved the artifact via a tool call (e.g., save_idea)
        // even though it didn't produce any text output in the stream
        if let Some(ref path) = artifact_path {
            if path.exists()
                && let Ok(content) = std::fs::read_to_string(path)
                && !content.trim().is_empty()
            {
                if let Err(e) = validate_artifact_content(stage_name, &content) {
                    return StageResult::Failed(e);
                }
                tracing::info!(
                    "[StageExecutor] Agent produced no text in stream, but artifact was saved via tool call ({:?}, {} chars)",
                    path, content.len()
                );
                if let Some(feedback_msg) = check_pending_critic_feedback(stage_name) {
                    interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            format!("🔄 Critic found issues, triggering revision..."),
                            MessageContext::new(&display_name).with_stage(stage_name),
                        )
                        .await;
                    return StageResult::NeedsRevision(feedback_msg);
                }
                interaction
                    .show_message_with_context(
                        crate::interaction::MessageLevel::Success,
                        format!("✓ Completed (artifact saved via tool, {} chars)", content.len()),
                        MessageContext::new(&display_name).with_stage(stage_name),
                    )
                    .await;
                return StageResult::Success(Some(path.to_string_lossy().to_string()));
            }
        } else {
            if let Some(feedback_msg) = check_pending_critic_feedback(stage_name) {
                interaction
                    .show_message_with_context(
                        crate::interaction::MessageLevel::Warning,
                        format!("🔄 Critic found issues, triggering revision..."),
                        MessageContext::new(&display_name).with_stage(stage_name),
                    )
                    .await;
                return StageResult::NeedsRevision(feedback_msg);
            }
            tracing::info!(
                "[StageExecutor] Stage '{}' has no artifact file, treating empty output as acceptable",
                stage_name
            );
            return StageResult::Success(None);
        }

        tracing::warn!(
            "[StageExecutor] Agent produced no text output and no artifact was saved. total_events={}, text_events={}, tool_calls={}. \
             The LLM likely only made tool calls without generating any text content or saving artifacts.",
            event_count, text_event_count, tool_call_count
        );
        return StageResult::Failed(format!(
            "Agent produced no text output and no artifact was saved ({} events, {} tool calls)",
            event_count, tool_call_count
        ));
    }

    // Show summary of what was generated
    let summary_msg = format!("✓ Completed ({} chars generated)", generated_text.len());

    interaction
        .show_message_with_context(
            crate::interaction::MessageLevel::Success,
            summary_msg,
            MessageContext::new(&display_name).with_stage(stage_name),
        )
        .await;

    // Stages without a single artifact file (e.g., coding) are considered successful
    // once they produce text output
    let artifact_path = match artifact_path {
        Some(p) => p,
        None => {
            if let Some(feedback_msg) = check_pending_critic_feedback(stage_name) {
                interaction
                    .show_message_with_context(
                        crate::interaction::MessageLevel::Warning,
                        format!("🔄 Critic found issues, triggering revision..."),
                        MessageContext::new(&display_name).with_stage(stage_name),
                    )
                    .await;
                return StageResult::NeedsRevision(feedback_msg);
            }
            tracing::info!(
                "[StageExecutor] Stage '{}' has no artifact file, text output is sufficient",
                stage_name
            );
            return StageResult::Success(None);
        }
    };

    if artifact_path.exists()
        && let Ok(content) = std::fs::read_to_string(&artifact_path)
        && !content.trim().is_empty()
    {
        if let Err(e) = validate_artifact_content(stage_name, &content) {
            return StageResult::Failed(e);
        }
        if let Some(feedback_msg) = check_pending_critic_feedback(stage_name) {
            interaction
                .show_message_with_context(
                    crate::interaction::MessageLevel::Warning,
                    format!("🔄 Critic found issues, triggering revision..."),
                    MessageContext::new(&display_name).with_stage(stage_name),
                )
                .await;
            return StageResult::NeedsRevision(feedback_msg);
        }
        tracing::info!(
            "[StageExecutor] Artifact saved via tool call ({:?}, {} chars)",
            artifact_path, content.len()
        );
        return StageResult::Success(Some(artifact_path.to_string_lossy().to_string()));
    }

    // Agent produced text output but didn't call the save tool.
    // Send a follow-up message to the same agent to save the artifact.
    tracing::warn!(
        "[StageExecutor] Agent completed but artifact not saved. Sending follow-up to prompt save tool call."
    );

    interaction
        .show_message_with_context(
            crate::interaction::MessageLevel::Warning,
            "⚠️ Artifact not saved, prompting agent to save...".to_string(),
            MessageContext::new(&display_name).with_stage(stage_name),
        )
        .await;

    // Build follow-up prompt
    let save_tool_name = get_save_tool_name(stage_name);
    let followup_prompt = format!(
        "CRITICAL: You completed the {} stage but did NOT call the {} tool to save your artifact. \
         Your work will be LOST unless you call {} now. \
         Call {}(content=<your complete {} document in markdown>) IMMEDIATELY. \
         Do NOT output any more text — just call the save tool with your complete document content.",
        stage_name, save_tool_name, save_tool_name, save_tool_name, stage_name
    );

    let followup_content = Content::new("user").with_text(followup_prompt);
    let followup_ctx = Arc::new(SimpleInvocationContext::new(
        ctx,
        &followup_content,
        agent.clone(),
    ));

    let followup_stream = match agent.run(followup_ctx).await {
        Ok(s) => s,
        Err(e) => {
            let err_msg = format!("{}", e);
            if err_msg.contains("GOTO_STAGE_REQUESTED") {
                if let Some((target_stage, reason)) = take_goto_stage_signal() {
                    interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            format!("🔄 Stage jump requested: {} → {}", stage_name, target_stage),
                            MessageContext::new(&display_name).with_stage(stage_name),
                        )
                        .await;
                    return StageResult::GotoStage(target_stage, reason);
                }
            }
            tracing::warn!("[StageExecutor] Follow-up agent run failed: {}", e);
            return StageResult::Failed(format!(
                "Agent completed but did not save artifact, and follow-up failed: {}", e
            ));
        }
    };

    let mut followup_stream = std::pin::pin!(followup_stream);
    while let Some(result) = followup_stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = event.content()
                    && let Some(text) = extract_text_from_content(content)
                    && !text.trim().is_empty()
                {
                    interaction.send_streaming(text, &display_name, false).await;
                }
                if let Some((target_stage, reason)) = check_event_for_goto_stage(&event) {
                    interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            format!("🔄 Stage jump requested: {} → {}", stage_name, target_stage),
                            MessageContext::new(&display_name).with_stage(stage_name),
                        )
                        .await;
                    return StageResult::GotoStage(target_stage, reason);
                }
            }
            Err(e) => {
                let err_msg = format!("{}", e);
                if err_msg.contains("GOTO_STAGE_REQUESTED") {
                    if let Some((target_stage, reason)) = take_goto_stage_signal() {
                        interaction
                            .show_message_with_context(
                                crate::interaction::MessageLevel::Warning,
                                format!("🔄 Stage jump requested: {} → {}", stage_name, target_stage),
                                MessageContext::new(&display_name).with_stage(stage_name),
                            )
                            .await;
                        return StageResult::GotoStage(target_stage, reason);
                    }
                }
                tracing::debug!("[StageExecutor] Follow-up stream error: {}", e);
            }
        }
    }

    // Check artifact again after follow-up
    if artifact_path.exists()
        && let Ok(content) = std::fs::read_to_string(&artifact_path)
        && !content.trim().is_empty()
    {
        if let Err(e) = validate_artifact_content(stage_name, &content) {
            return StageResult::Failed(e);
        }
        if let Some(feedback_msg) = check_pending_critic_feedback(stage_name) {
            interaction
                .show_message_with_context(
                    crate::interaction::MessageLevel::Warning,
                    format!("🔄 Critic found issues, triggering revision..."),
                    MessageContext::new(&display_name).with_stage(stage_name),
                )
                .await;
            return StageResult::NeedsRevision(feedback_msg);
        }
        tracing::info!(
            "[StageExecutor] Artifact saved after follow-up ({:?}, {} chars)",
            artifact_path, content.len()
        );
        interaction
            .show_message_with_context(
                crate::interaction::MessageLevel::Success,
                format!("✓ Artifact saved ({} chars)", content.len()),
                MessageContext::new(&display_name).with_stage(stage_name),
            )
            .await;
        return StageResult::Success(Some(artifact_path.to_string_lossy().to_string()));
    }

    tracing::warn!(
        "[StageExecutor] Agent still did not save artifact after follow-up. Artifact path: {:?}",
        artifact_path
    );
    StageResult::Failed(format!(
        "Agent completed but did not save artifact via {} tool",
        save_tool_name
    ))
}

/// Maximum characters for fully pre-injected artifacts.
/// Artifacts below this size are injected in their entirety.
const FULL_INJECTION_MAX_CHARS: usize = 12000;

/// Maximum characters for artifact previews when the artifact is too large
/// to inject fully. The agent MUST use the load tool to get the full content.
const PREVIEW_MAX_CHARS: usize = 2000;

/// Get truncated message in current language
fn get_truncated_message() -> String {
    let locale = crate::config::get_system_locale();
    if locale.starts_with("zh") {
        "...[已截断，完整内容可通过工具加载]".to_string()
    } else if locale.starts_with("ja") {
        "...[一部切り捨て、完全な内容はツールで読み込めます]".to_string()
    } else {
        "...[truncated, full content available via tool]".to_string()
    }
}

/// Truncate content to a maximum number of characters (UTF-8 safe).
/// This is kept for backward compatibility; new code should use
/// `format_artifact_block` so callers can distinguish full vs. preview content.
fn truncate_content(content: &str, max_chars: usize) -> String {
    if content.chars().count() <= max_chars {
        content.to_string()
    } else {
        let truncated: String = content.chars().take(max_chars).collect();
        format!("{}{}", truncated, get_truncated_message())
    }
}

/// Format an artifact block for injection into the agent prompt.
///
/// - If the artifact is small enough, inject it in full and tell the agent it
///   is pre-loaded.
/// - If it is too large, inject only a preview and explicitly instruct the
///   agent to use `load_tool` to read the complete document. This prevents
///   the agent from making decisions on a silently truncated artifact.
fn format_artifact_block(label: &str, content: &str, load_tool: &str) -> String {
    let char_count = content.chars().count();

    if char_count <= FULL_INJECTION_MAX_CHARS {
        format!(
            "═══════════════════════════════════════════════════════════════\n\
             📋 PRE-LOADED: {} ({} characters, complete)\n\
             ═══════════════════════════════════════════════════════════════\n\
             {}\n\
             ═══════════════════════════════════════════════════════════════\n\n",
            label, char_count, content
        )
    } else {
        let preview: String = content.chars().take(PREVIEW_MAX_CHARS).collect();
        format!(
            "═══════════════════════════════════════════════════════════════\n\
             📋 PREVIEW: {} ({} characters total — ONLY FIRST {} CHARACTERS SHOWN)\n\
             ═══════════════════════════════════════════════════════════════\n\
             {}\n\
             ...[TRUNCATED]\n\
             ═══════════════════════════════════════════════════════════════\n\
             ⚠️ CRITICAL: The full {} is too large to pre-load. You MUST call `{}` \
             to read the complete document before making any decisions. Do NOT assume \
             the preview above contains all requirements or details.\n\n",
            label, char_count, PREVIEW_MAX_CHARS, preview, label, load_tool
        )
    }
}

/// Load artifact content from the artifacts directory
fn load_artifact_content(ctx: &PipelineContext, artifact_name: &str) -> Option<String> {
    let iteration_dir = ctx.workspace_path.parent().unwrap_or(&ctx.workspace_path);
    let artifact_path = iteration_dir.join("artifacts").join(artifact_name);

    if artifact_path.exists() {
        match std::fs::read_to_string(&artifact_path) {
            Ok(content) if !content.trim().is_empty() => {
                return Some(content);
            }
            _ => {}
        }
    }
    None
}

/// Build prompt with iteration context and pre-injected artifacts
fn build_prompt(
    ctx: &PipelineContext,
    stage_name: &str,
    feedback: Option<&str>,
    extra_context: Option<&str>,
) -> String {
    let mut prompt = format!(
        "You are working on iteration #{} - '{}'.\n",
        ctx.iteration.number, ctx.iteration.title
    );

    prompt.push_str(&format!("Iteration ID: {}\n\n", ctx.iteration.id));

    // For evolution iterations, inject strong EVOLUTION context at the beginning
    // This is CRITICAL for agents to understand they should NOT rewrite from scratch
    if let Some(base_id) = &ctx.iteration.base_iteration_id {
        let inheritance_mode_name = match ctx.iteration.inheritance {
            crate::domain::InheritanceMode::None => "None",
            crate::domain::InheritanceMode::Full => "Full",
            crate::domain::InheritanceMode::Partial => "Partial",
        };

        prompt.push_str("═══════════════════════════════════════════════════════════════\n");
        prompt.push_str("🚨🚨🚨 CRITICAL: THIS IS AN EVOLUTION ITERATION 🚨🚨🚨\n");
        prompt.push_str("═══════════════════════════════════════════════════════════════\n");
        prompt.push('\n');
        prompt.push_str("⚠️ DO NOT CREATE NEW PROJECT - BUILD ON EXISTING CODE ⚠️\n\n");
        prompt.push_str(&format!("Base Iteration: {}\n", base_id));
        prompt.push_str(&format!("Inheritance Mode: {}\n\n", inheritance_mode_name));
        
        match ctx.iteration.inheritance {
            crate::domain::InheritanceMode::Partial => {
                prompt.push_str("📋 PARTIAL INHERITANCE:\n");
                prompt.push_str("- Code files from the base iteration have been COPIED to the workspace\n");
                prompt.push_str("- You MUST preserve existing code and add new features incrementally\n");
                prompt.push_str("- DO NOT rewrite the project from scratch\n");
                prompt.push_str("- DO NOT delete existing files unless absolutely necessary\n\n");
            }
            crate::domain::InheritanceMode::Full => {
                prompt.push_str("📋 FULL INHERITANCE:\n");
                prompt.push_str("- All files (code + artifacts) from base iteration are available\n");
                prompt.push_str("- You MUST preserve existing code and only make necessary modifications\n");
                prompt.push_str("- DO NOT rewrite the project from scratch\n\n");
            }
            crate::domain::InheritanceMode::None => {}
        }
        
        prompt.push_str("🎯 YOUR APPROACH FOR THIS ITERATION:\n");
        prompt.push_str("1. FIRST: Use list_files() to see the existing project structure\n");
        prompt.push_str("2. Read relevant existing files before making ANY changes\n");
        prompt.push_str("3. Identify where new features should be added\n");
        prompt.push_str("4. Add new features incrementally - DO NOT regenerate existing code\n");
        prompt.push_str("5. Only modify files that need changes for the new features\n\n");
        prompt.push_str("═══════════════════════════════════════════════════════════════\n\n");
        
        // Also try to load project_context insight if available
        if let Ok(iter_memory) = crate::persistence::MemoryStore::new()
            .load_iteration_memory(&ctx.iteration.id)
        {
            for insight in &iter_memory.insights {
                if insight.stage == "project_context" {
                    prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                    prompt.push_str("📋 PROJECT CONTEXT (from base iteration)\n");
                    prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                    prompt.push_str(&truncate_content(&insight.content, 2000));
                    prompt.push_str("\n═══════════════════════════════════════════════════════════════\n\n");
                    break;
                }
            }
        }
    }

    // Inject iteration goal/description for ALL stages (not just idea)
    prompt.push_str("═══════════════════════════════════════════════════════════════\n");
    prompt.push_str("🎯 ITERATION GOAL\n");
    prompt.push_str("═══════════════════════════════════════════════════════════════\n");
    prompt.push_str(&ctx.iteration.description);
    prompt.push_str("\n═══════════════════════════════════════════════════════════════\n\n");

    // Pre-inject artifacts from previous stages (Optimization: reduces tool calls)
    // We also track which artifacts were injected only as previews so the agent
    // knows it must load the full document before making decisions.
    let mut injected_artifacts: Vec<&'static str> = Vec::new();
    let mut preview_artifacts: Vec<&'static str> = Vec::new();

    // Helper to inject an artifact and track whether it was a preview.
    let mut inject = |filename: &'static str, label: &'static str, load_tool: &'static str| {
        if let Some(content) = load_artifact_content(ctx, filename) {
            let was_preview = content.chars().count() > FULL_INJECTION_MAX_CHARS;
            prompt.push_str(&format_artifact_block(label, &content, load_tool));
            injected_artifacts.push(filename);
            if was_preview {
                preview_artifacts.push(filename);
            }
        }
    };

    match stage_name {
        "prd" => {
            // PRD needs Idea
            inject("idea.md", "Idea Document (from previous stage)", "load_idea()");
        }
        "design" => {
            // Design needs PRD
            inject("prd.md", "PRD Document (from previous stage)", "load_prd_doc()");
        }
        "plan" => {
            // Plan needs Design (PRD can be loaded if needed)
            inject("design.md", "Design Document (from previous stage)", "load_design_doc()");
        }
        "coding" => {
            // Coding needs Plan (most important) and Design for architecture context
            inject("plan.md", "Implementation Plan (from previous stage)", "load_plan_doc()");
            inject("design.md", "Design Document (architecture reference)", "load_design_doc()");
        }
        "check" | "delivery" => {
            // Check and Delivery need all artifacts
            inject("idea.md", "Idea Document", "load_idea()");
            inject("prd.md", "PRD Document", "load_prd_doc()");
            inject("design.md", "Design Document", "load_design_doc()");
            inject("plan.md", "Implementation Plan", "load_plan_doc()");
        }
        _ => {}
    }

    // Build a human-readable note about which artifacts were preview-only.
    let preview_note = if preview_artifacts.is_empty() {
        String::new()
    } else {
        format!(
            " NOTE: The following pre-loaded documents were truncated previews: {}. \
             Use the corresponding load tool to read the full content before making decisions.",
            preview_artifacts.join(", ")
        )
    };

    // Provide stage-specific guidance
    match stage_name {
        "idea" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Idea\n");
            prompt.push_str("========================================\n");
            prompt.push_str("The iteration goal is provided above.\n\n");
            prompt.push_str("YOUR TASK:\n");
            prompt.push_str("1. Read and understand the iteration goal\n");
            prompt.push_str("2. Generate a structured idea document\n");
            prompt.push_str("3. SAVE IT using the save_idea() tool (MANDATORY)\n\n");
        }
        "prd" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: PRD (Product Requirements Document)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.contains(&"idea.md") {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str(&format!(
                    "1. The Idea document is provided above (pre-loaded or preview).{}\n",
                    preview_note
                ));
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load idea using load_idea() tool\n");
            }
            prompt.push_str("2. Analyze the idea and create requirements\n");
            prompt.push_str("3. SAVE PRD using save_prd_doc() tool (MANDATORY)\n\n");
        }
        "design" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Design (System Architecture)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.contains(&"prd.md") {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str(&format!(
                    "1. The PRD document is provided above (pre-loaded or preview).{}\n",
                    preview_note
                ));
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load requirements using get_requirements() tool\n");
            }
            prompt.push_str("2. Design system architecture (2-4 components max)\n");
            prompt.push_str("3. SAVE DESIGN using save_design_doc() tool (MANDATORY)\n\n");
        }
        "plan" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Plan (Implementation Tasks)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.contains(&"design.md") {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str(&format!(
                    "1. The Design document is provided above (pre-loaded or preview).{}\n",
                    preview_note
                ));
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load design using get_design() tool\n");
            }
            prompt.push_str("2. Create 5-12 simple implementation tasks\n");
            prompt.push_str("3. SAVE PLAN using save_plan_doc() tool (MANDATORY)\n\n");
        }
        "coding" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Coding (Implementation)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.contains(&"plan.md") {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str(&format!(
                    "1. The Plan and Design documents are provided above (pre-loaded or preview).{}\n",
                    preview_note
                ));
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load plan using get_plan() tool\n");
            }
            prompt.push_str("2. Implement tasks one by one\n");
            prompt.push_str("3. Update task status using update_task_status() tool\n\n");
        }
        "check" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Check (Quality Assurance)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.len() >= 4 {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str(&format!(
                    "1. All artifacts are provided above (pre-loaded or preview).{}\n",
                    preview_note
                ));
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load all artifacts (requirements, design, plan)\n");
            }
            prompt.push_str("2. Run quality checks\n");
            prompt.push_str("3. Use goto_stage() if issues found\n\n");
        }
        "delivery" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Delivery (Final Report)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.len() >= 4 {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str(&format!(
                    "1. All artifacts are provided above (pre-loaded or preview).{}\n",
                    preview_note
                ));
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load all artifacts\n");
            }
            prompt.push_str("2. Generate delivery report\n");
            prompt.push_str("3. SAVE using save_delivery_report() tool\n");
            prompt.push_str("4. Copy files using copy_workspace_to_project() tool\n\n");
        }
        _ => {
            prompt.push_str(&format!(
                "Original request: {}\n\n",
                ctx.iteration.description
            ));
        }
    }

    prompt.push_str(&format!("Workspace: {}\n\n", ctx.workspace_path.display()));

    // Add artifact path information
    prompt.push_str(&format!(
        "Artifacts directory: .cowork-v2/iterations/{}/artifacts/\n\n",
        ctx.iteration.id
    ));

    // Add explicit instruction to use tools
    prompt.push_str("IMPORTANT: You have access to tools and MUST use them to save your work.\n");
    prompt.push_str("For the ");
    prompt.push_str(stage_name);
    prompt.push_str(
        " stage, you MUST use the appropriate save tool (e.g., save_idea for idea stage).\n\n",
    );

    if let Some(feedback_text) = feedback {
        prompt.push_str(&format!("USER FEEDBACK: {}\n\n", feedback_text));
        prompt.push_str("Please revise your previous work based on this feedback.\n");
    }

    if let Some(extra) = extra_context {
        prompt.push_str("\n═══════════════════════════════════════════════════════════════\n");
        prompt.push_str("📋 ADDITIONAL CONTEXT FROM EXTERNAL AGENT FALLBACK\n");
        prompt.push_str("═══════════════════════════════════════════════════════════════\n");
        prompt.push_str(extra);
        prompt.push_str("\n═══════════════════════════════════════════════════════════════\n");
    }

    // Add language preference instruction
    let lang_instruction = get_language_instruction();
    prompt.push_str(&format!("\n{}\n", lang_instruction));

    prompt
}

/// Simple InvocationContext implementation
pub struct SimpleInvocationContext {
    invocation_id: String,
    agent_name: String,
    user_id: String,
    app_name: String,
    session_id: String,
    branch: String,
    user_content: Content,
    agent: Arc<dyn adk_core::Agent>,
    memory: Option<Arc<dyn adk_core::Memory>>,
    session: Box<dyn adk_core::Session>,
    run_config: adk_core::RunConfig,
    ended: std::sync::atomic::AtomicBool,
    artifacts: Option<Arc<dyn adk_core::Artifacts>>,
}

impl SimpleInvocationContext {
    pub fn new(ctx: &PipelineContext, content: &Content, agent: Arc<dyn adk_core::Agent>) -> Self {
        Self {
            invocation_id: uuid::Uuid::new_v4().to_string(),
            agent_name: agent.name().to_string(),
            user_id: "default_user".to_string(),
            app_name: "cowork_forge".to_string(),
            session_id: ctx.iteration.id.clone(),
            branch: "main".to_string(),
            user_content: content.clone(),
            agent,
            // Memory and Artifacts are ALSO available through dedicated tools
            // (QueryMemoryTool, LoadArtifactTool, etc.). We wire them into the
            // InvocationContext so that framework callbacks, plugins, or future
            // agents that rely on ctx.memory() / ctx.artifacts() work correctly.
            memory: Some(Arc::new(SimpleMemory::new(&ctx.iteration.id))),
            session: Box::new(SimpleSession::new(&ctx.iteration.id, content.clone())),
            run_config: adk_core::RunConfig {
                streaming_mode: adk_core::StreamingMode::SSE,
                ..adk_core::RunConfig::default()
            },
            ended: std::sync::atomic::AtomicBool::new(false),
            artifacts: Some(Arc::new(SimpleArtifacts::new(&ctx.iteration.id))),
        }
    }
}

// Implement Clone for SimpleInvocationContext
impl Clone for SimpleInvocationContext {
    fn clone(&self) -> Self {
        Self {
            invocation_id: self.invocation_id.clone(),
            agent_name: self.agent_name.clone(),
            user_id: self.user_id.clone(),
            app_name: self.app_name.clone(),
            session_id: self.session_id.clone(),
            branch: self.branch.clone(),
            user_content: self.user_content.clone(),
            agent: self.agent.clone(),
            memory: self.memory.clone(),
            // session can't be cloned, create a new one
            session: Box::new(SimpleSession::new(
                &self.session_id,
                self.user_content.clone(),
            )),
            run_config: self.run_config.clone(),
            ended: std::sync::atomic::AtomicBool::new(
                self.ended.load(std::sync::atomic::Ordering::SeqCst),
            ),
            artifacts: self.artifacts.clone(),
        }
    }
}

// Implement InvocationContext trait and its supertraits
#[async_trait::async_trait]
impl adk_core::InvocationContext for SimpleInvocationContext {
    fn agent(&self) -> Arc<dyn adk_core::Agent> {
        self.agent.clone()
    }

    fn memory(&self) -> Option<Arc<dyn adk_core::Memory>> {
        self.memory.clone()
    }

    fn session(&self) -> &dyn adk_core::Session {
        self.session.as_ref()
    }

    fn run_config(&self) -> &adk_core::RunConfig {
        &self.run_config
    }

    fn end_invocation(&self) {
        self.ended.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    fn ended(&self) -> bool {
        self.ended.load(std::sync::atomic::Ordering::SeqCst)
    }
}

// Implement CallbackContext
#[async_trait::async_trait]
impl adk_core::CallbackContext for SimpleInvocationContext {
    fn artifacts(&self) -> Option<Arc<dyn adk_core::Artifacts>> {
        self.artifacts.clone()
    }
}

// Implement ReadonlyContext
#[async_trait::async_trait]
impl adk_core::ReadonlyContext for SimpleInvocationContext {
    fn invocation_id(&self) -> &str {
        &self.invocation_id
    }

    fn agent_name(&self) -> &str {
        &self.agent_name
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn app_name(&self) -> &str {
        &self.app_name
    }

    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn branch(&self) -> &str {
        &self.branch
    }

    fn user_content(&self) -> &Content {
        &self.user_content
    }
}



/// Simple Session implementation that persists conversation history to disk.
///
/// History is stored as newline-delimited JSON in
/// `.cowork-v2/iterations/{session_id}/session_history.jsonl` so that retries,
/// actor-critic loops, and feedback revisions can see prior turns instead of
/// starting from scratch.
///
/// Truncation: to prevent unbounded context growth across many retries /
/// feedback revisions, `conversation_history()` applies a sliding window of
/// `MAX_HISTORY_MESSAGES` messages (keeping the most recent ones, plus the
/// initial user prompt for context). Persistence is untouched; truncation is
/// only applied to the view returned to the LLM.
struct SimpleSession {
    session_id: String,
    app_name: String,
    user_id: String,
    simple_state: SimpleState,
    messages: std::sync::Mutex<Vec<Content>>,
    history_path: std::path::PathBuf,
}

/// Maximum number of messages returned by `conversation_history()`.
///
/// This is a sliding-window cap: when the in-memory history exceeds this many
/// messages, only the most recent `MAX_HISTORY_MESSAGES` are returned to the
/// LLM (the very first user prompt is also preserved so the agent never loses
/// the original task context). The full history continues to be persisted to
/// disk for debugging/audit.
const MAX_HISTORY_MESSAGES: usize = 60;

impl SimpleSession {
    fn new(session_id: &str, initial_message: Content) -> Self {
        let history_path = crate::persistence::get_cowork_dir()
            .map(|dir| dir.join("iterations").join(session_id).join("session_history.jsonl"))
            .unwrap_or_else(|_| {
                std::path::PathBuf::from(".cowork-v2")
                    .join("iterations")
                    .join(session_id)
                    .join("session_history.jsonl")
            });

        // Ensure parent directory exists and load any prior history.
        let mut messages = Vec::new();
        if let Some(parent) = history_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if history_path.exists()
            && let Ok(contents) = std::fs::read_to_string(&history_path)
        {
            for line in contents.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(content) = serde_json::from_str::<Content>(line) {
                    messages.push(content);
                }
            }
        }

        // Persist the current prompt so subsequent agent invocations (within
        // the same stage run, or across retries / actor-critic iterations that
        // reuse this session) can see the original user turn. Without this,
        // a fresh agent run would only see prior assistant responses and lose
        // the user's original instruction.
        if let Err(e) = Self::append_message_to_file(&history_path, &initial_message) {
            tracing::warn!("Failed to persist initial prompt to session history: {}", e);
        }
        messages.push(initial_message);

        Self {
            session_id: session_id.to_string(),
            app_name: "cowork_forge".to_string(),
            user_id: "default_user".to_string(),
            simple_state: SimpleState::new(),
            messages: std::sync::Mutex::new(messages),
            history_path,
        }
    }

    fn append_message_to_file(
        path: &std::path::PathBuf,
        content: &Content,
    ) -> anyhow::Result<()> {
        let line = serde_json::to_string(content)?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Apply a sliding-window truncation so the LLM context stays bounded.
    ///
    /// Returns the full message list when it is small enough; otherwise
    /// returns the first message (the original user prompt, to preserve the
    /// task framing) followed by the most recent `MAX_HISTORY_MESSAGES - 1`
    /// messages.
    fn truncate_for_view(messages: &[Content]) -> Vec<Content> {
        if messages.len() <= MAX_HISTORY_MESSAGES {
            return messages.to_vec();
        }
        let mut view = Vec::with_capacity(MAX_HISTORY_MESSAGES);
        // Always keep the very first user prompt for task context.
        view.push(messages[0].clone());
        let tail_start = messages.len().saturating_sub(MAX_HISTORY_MESSAGES - 1);
        view.extend_from_slice(&messages[tail_start..]);
        tracing::debug!(
            "SimpleSession: truncated history from {} to {} messages (cap={})",
            messages.len(), view.len(), MAX_HISTORY_MESSAGES
        );
        view
    }
}

impl adk_core::Session for SimpleSession {
    fn id(&self) -> &str {
        &self.session_id
    }

    fn app_name(&self) -> &str {
        &self.app_name
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn state(&self) -> &dyn adk_core::State {
        &self.simple_state
    }

    fn conversation_history(&self) -> Vec<Content> {
        self.messages
            .lock()
            .map(|m| Self::truncate_for_view(&m))
            .unwrap_or_default()
    }

    fn append_to_history(&self, content: Content) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push(content.clone());
        }
        if let Err(e) = Self::append_message_to_file(&self.history_path, &content) {
            tracing::warn!("Failed to persist session history: {}", e);
        }
    }
}

/// Minimal in-memory + file-backed artifact store for the InvocationContext.
///
/// This is a thin adapter over the iteration's artifacts directory. Most agents
/// should continue to use dedicated artifact tools (load_artifact, save_artifact)
/// for validation and schema enforcement; this store exists so that framework
/// callbacks and plugins can access artifacts through `CallbackContext::artifacts()`.
struct SimpleArtifacts {
    artifacts_dir: std::path::PathBuf,
}

impl SimpleArtifacts {
    fn new(iteration_id: &str) -> Self {
        let artifacts_dir = crate::persistence::get_cowork_dir()
            .map(|dir| dir.join("iterations").join(iteration_id).join("artifacts"))
            .unwrap_or_else(|_| {
                std::path::PathBuf::from(".cowork-v2")
                    .join("iterations")
                    .join(iteration_id)
                    .join("artifacts")
            });
        let _ = std::fs::create_dir_all(&artifacts_dir);
        Self { artifacts_dir }
    }

    fn safe_name(name: &str) -> Option<String> {
        let sanitized: String = name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect();
        if sanitized.is_empty() || sanitized != name {
            None
        } else {
            Some(sanitized)
        }
    }

    fn path_for(&self, name: &str) -> Option<std::path::PathBuf> {
        Self::safe_name(name).map(|n| self.artifacts_dir.join(n))
    }
}

#[async_trait::async_trait]
impl adk_core::Artifacts for SimpleArtifacts {
    async fn save(&self, name: &str, data: &adk_core::Part) -> adk_core::Result<i64> {
        let path = self.path_for(name).ok_or_else(|| {
            adk_core::AdkError::tool(format!("Invalid artifact name: {}", name))
        })?;

        match data {
            adk_core::Part::Text { text } => {
                std::fs::write(&path, text).map_err(|e| {
                    adk_core::AdkError::tool(format!("Failed to write artifact {}: {}", name, e))
                })?;
            }
            adk_core::Part::InlineData { mime_type, data } => {
                let ext = match mime_type.as_str() {
                    "text/markdown" | "text/plain" => "txt",
                    "application/json" => "json",
                    "image/png" => "png",
                    "image/jpeg" => "jpg",
                    _ => "bin",
                };
                let path = path.with_extension(ext);
                std::fs::write(&path, data).map_err(|e| {
                    adk_core::AdkError::tool(format!("Failed to write artifact {}: {}", name, e))
                })?;
            }
            adk_core::Part::FileData { file_uri, .. } => {
                // We cannot meaningfully save a URI reference to local disk.
                return Err(adk_core::AdkError::tool(format!(
                    "Cannot save FileData artifact with URI: {}",
                    file_uri
                )));
            }
            _ => {
                return Err(adk_core::AdkError::tool(
                    "Unsupported artifact part type".to_string(),
                ));
            }
        }
        Ok(1)
    }

    async fn load(&self, name: &str) -> adk_core::Result<adk_core::Part> {
        let path = self.path_for(name).ok_or_else(|| {
            adk_core::AdkError::tool(format!("Invalid artifact name: {}", name))
        })?;

        let data = std::fs::read(&path).map_err(|e| {
            adk_core::AdkError::tool(format!("Failed to read artifact {}: {}", name, e))
        })?;

        // Try to return as text for UTF-8 content, otherwise inline binary.
        if let Ok(text) = String::from_utf8(data.clone()) {
            Ok(adk_core::Part::Text { text })
        } else {
            let mime_type = match path.extension().and_then(|e| e.to_str()) {
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                _ => "application/octet-stream",
            }
            .to_string();
            Ok(adk_core::Part::InlineData { mime_type, data })
        }
    }

    async fn list(&self) -> adk_core::Result<Vec<String>> {
        let mut names = Vec::new();
        if self.artifacts_dir.exists() {
            for entry in std::fs::read_dir(&self.artifacts_dir).map_err(|e| {
                adk_core::AdkError::tool(format!("Failed to list artifacts: {}", e))
            })? {
                let entry = entry.map_err(|e| adk_core::AdkError::tool(format!("Failed to read artifact entry: {}", e)))?;
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false)
                    && let Some(name) = entry.file_name().to_str()
                {
                    names.push(name.to_string());
                }
            }
        }
        Ok(names)
    }
}

/// Minimal memory adapter for the InvocationContext.
///
/// Delegates to the project's persisted memory store. Agents should prefer the
/// dedicated `query_memory` tool for richer filtering; this adapter lets the
/// framework call `InvocationContext::memory()` without returning `None`.
struct SimpleMemory {
    iteration_id: String,
}

impl SimpleMemory {
    fn new(iteration_id: &str) -> Self {
        Self {
            iteration_id: iteration_id.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl adk_core::Memory for SimpleMemory {
    async fn search(&self, query: &str) -> adk_core::Result<Vec<adk_core::MemoryEntry>> {
        let store = crate::persistence::MemoryStore::new();
        let memory_query = crate::domain::MemoryQuery {
            scope: crate::domain::MemoryScope::Smart,
            query_type: crate::domain::MemoryQueryType::All,
            keywords: query.split_whitespace().map(|s| s.to_string()).collect(),
            limit: Some(20),
        };

        let result = store.query(&memory_query, Some(&self.iteration_id)).map_err(|e| {
            adk_core::AdkError::memory(format!("Failed to query memory: {}", e))
        })?;

        let mut entries = Vec::new();
        for decision in result.decisions {
            let decision_text = format!(
                "Decision: {}\nContext: {}\nOutcome: {}\nConsequences: {}",
                decision.title,
                decision.context,
                decision.decision,
                if decision.consequences.is_empty() {
                    "None recorded".to_string()
                } else {
                    decision.consequences.join(", ")
                }
            );
            entries.push(adk_core::MemoryEntry {
                content: adk_core::Content::new("model").with_text(decision_text),
                author: "project".to_string(),
            });
        }
        for pattern in result.patterns {
            let pattern_text = format!(
                "Pattern: {}\nDescription: {}\nUsage: {}\nTags: {}",
                pattern.name,
                pattern.description,
                if pattern.usage.is_empty() {
                    "Not specified".to_string()
                } else {
                    pattern.usage.join(", ")
                },
                if pattern.tags.is_empty() {
                    "None".to_string()
                } else {
                    pattern.tags.join(", ")
                }
            );
            entries.push(adk_core::MemoryEntry {
                content: adk_core::Content::new("model").with_text(pattern_text),
                author: "project".to_string(),
            });
        }
        for insight in result.insights {
            entries.push(adk_core::MemoryEntry {
                content: adk_core::Content::new("model").with_text(insight.content),
                author: insight.stage,
            });
        }
        Ok(entries)
    }
}

/// Simple State implementation for Session
struct SimpleState {
    data: std::collections::HashMap<String, serde_json::Value>,
}

impl SimpleState {
    fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
}

impl adk_core::State for SimpleState {
    fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    fn all(&self) -> std::collections::HashMap<String, serde_json::Value> {
        self.data.clone()
    }
}

/// Helper to extract text from Content object
pub fn extract_text_from_content(content: &Content) -> Option<String> {
    let mut text = String::new();
    for part in &content.parts {
        if let Some(part_text) = part.text() {
            text.push_str(part_text);
        }
    }
    if text.is_empty() { None } else { Some(text) }
}

/// Helper to extract text from Event
pub fn extract_text_from_event(event: &Event) -> Option<String> {
    // Event has methods to extract different types of content
    // Only extract text content, ignore tool calls/results/errors
    if let Some(content) = event.content() {
        // Extract text from Content parts
        let mut text = String::new();
        for part in &content.parts {
            if let Some(part_text) = part.text() {
                text.push_str(part_text);
            }
        }
        Some(text)
    } else {
        // Not a content event (could be tool call, result, or error)
        None
    }
}

/// Validate that a stage artifact contains meaningful content.
///
/// This prevents downstream stages from consuming placeholder output such as
/// "TODO", "FIXME", or near-empty documents. Returns `Ok(())` when the
/// content looks valid, otherwise returns an error message describing the issue.
fn validate_artifact_content(stage_name: &str, content: &str) -> std::result::Result<(), String> {
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return Err(format!("Stage '{}' produced an empty artifact", stage_name));
    }

    if trimmed.chars().count() < 50 {
        return Err(format!(
            "Stage '{}' produced an artifact that is too short ({} chars). Provide a complete document.",
            stage_name,
            trimmed.chars().count()
        ));
    }

    // Reject documents that are only placeholders.
    let upper = trimmed.to_uppercase();
    let placeholder_only = ["TODO", "FIXME", "TBD", "PLACEHOLDER", "NOT IMPLEMENTED"]
        .iter()
        .any(|p| upper.contains(p));
    if placeholder_only {
        return Err(format!(
            "Stage '{}' artifact appears to contain only placeholder text (TODO/FIXME/TBD). Provide complete content.",
            stage_name
        ));
    }

    // Markdown artifacts should contain at least one heading.
    if stage_name != "coding" && !content.contains('#') {
        return Err(format!(
            "Stage '{}' markdown artifact is missing headings. Use proper markdown structure.",
            stage_name
        ));
    }

    Ok(())
}
