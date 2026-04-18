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
use crate::llm::{create_llm_client};
use crate::llm::config::load_config;
use crate::pipeline::{PipelineContext, StageResult};
use crate::persistence::set_iteration_id;
use adk_core::{Content, Event};
use futures::StreamExt;
use std::sync::Arc;

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

/// Execute a stage using real adk-rust Agent
pub async fn execute_stage_with_instruction(
    ctx: &PipelineContext,
    interaction: Arc<dyn InteractiveBackend>,
    stage_name: &str,
    _instruction: &str,
    feedback: Option<&str>,
) -> StageResult {
    // Set iteration ID for data tools (V2 architecture)
    set_iteration_id(ctx.iteration.id.clone());

    // Check for restart mode (GotoStage mechanism)
    if let Ok(Some(session_meta)) = crate::persistence::load_session_meta() {
        if let Some(restart_reason) = session_meta.restart_reason {
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
            if let Ok(mut meta) = crate::persistence::load_session_meta() {
                if let Some(ref mut m) = meta {
                    m.restart_reason = None;
                    let _ = crate::persistence::save_session_meta(m);
                }
            }
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

        // Load LLM client
        let llm_config = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
        let model = create_llm_client(&llm_config.llm)
            .map_err(|e| format!("Failed to create LLM client: {}", e))?;

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

    // Build prompt with context
    let prompt = build_prompt(ctx, stage_name, feedback);

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
            // Check if this is a goto_stage signal
            if err_msg.starts_with("GOTO_STAGE:") {
                // Parse the target stage and reason
                let parts: Vec<&str> = err_msg.strip_prefix("GOTO_STAGE:").unwrap().splitn(2, ':').collect();
                if parts.len() == 2 {
                    let target_stage = parts[0].to_string();
                    let reason = parts[1].to_string();

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
                // Extract content from the event using the event's content() method
                if let Some(content) = event.content() {
                    if let Some(text) = extract_text_from_content(content) {
                        if !text.trim().is_empty() {
                            text_event_count += 1;
                            generated_text.push_str(&text);
                            // Send content in real-time with display name
                            interaction
                                .send_streaming(text.clone(), &display_name, false)
                                .await;
                        }
                    } else {
                        // Content exists but no text part — likely a function call
                        tool_call_count += 1;
                        tracing::debug!(
                            "[StageExecutor] Event #{} has content but no text part (likely tool call)",
                            event_count
                        );
                    }
                } else if let Some(text) = extract_text_from_event(&event) {
                    // Fallback: use helper function
                    if !text.trim().is_empty() {
                        text_event_count += 1;
                        generated_text.push_str(&text);
                        interaction.send_streaming(text, &display_name, false).await;
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("{}", e);
                // Check if this is a goto_stage signal from a tool
                if err_msg.contains("GOTO_STAGE:") {
                    // Extract the GOTO_STAGE message - format: "Tool execution failed: GOTO_STAGE:stage:reason"
                    // or just "GOTO_STAGE:stage:reason"
                    if let Some(goto_msg) = err_msg.split("GOTO_STAGE:").nth(1) {
                        let parts: Vec<&str> = goto_msg.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            let target_stage = parts[0].to_string();
                            let reason = parts[1].to_string();

                            interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Warning,
                                    format!("🔄 Stage jump requested: {} → {}", stage_name, target_stage),
                                    MessageContext::new(&display_name).with_stage(stage_name),
                                )
                                .await;

                            // Return immediately to trigger stage jump
                            return StageResult::GotoStage(target_stage, reason);
                        }
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
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if !content.trim().is_empty() {
                        tracing::info!(
                            "[StageExecutor] Agent produced no text in stream, but artifact was saved via tool call ({:?}, {} chars)",
                            path, content.len()
                        );
                        // Artifact exists and has content — stage is successful
                        interaction
                            .show_message_with_context(
                                crate::interaction::MessageLevel::Success,
                                format!("✓ Completed (artifact saved via tool, {} chars)", content.len()),
                                MessageContext::new(&display_name).with_stage(stage_name),
                            )
                            .await;
                        return StageResult::Success(Some(path.to_string_lossy().to_string()));
                    }
                }
            }
        } else {
            // No artifact expected (e.g., coding stage) — just check that agent ran
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
            tracing::info!(
                "[StageExecutor] Stage '{}' has no artifact file, text output is sufficient",
                stage_name
            );
            return StageResult::Success(None);
        }
    };

    // Check if artifact was saved via tool call (e.g., save_idea)
    if artifact_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&artifact_path) {
            if !content.trim().is_empty() {
                tracing::info!(
                    "[StageExecutor] Artifact saved via tool call ({:?}, {} chars)",
                    artifact_path, content.len()
                );
                return StageResult::Success(Some(artifact_path.to_string_lossy().to_string()));
            }
        }
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
            tracing::warn!("[StageExecutor] Follow-up agent run failed: {}", e);
            return StageResult::Failed(format!(
                "Agent completed but did not save artifact, and follow-up failed: {}", e
            ));
        }
    };

    // Process follow-up stream (only look for save tool execution, don't collect text)
    let mut followup_stream = std::pin::pin!(followup_stream);
    while let Some(result) = followup_stream.next().await {
        match result {
            Ok(event) => {
                // Stream any text content from the follow-up
                if let Some(content) = event.content() {
                    if let Some(text) = extract_text_from_content(content) {
                        if !text.trim().is_empty() {
                            interaction.send_streaming(text, &display_name, false).await;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::debug!("[StageExecutor] Follow-up stream error: {}", e);
            }
        }
    }

    // Check artifact again after follow-up
    if artifact_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&artifact_path) {
            if !content.trim().is_empty() {
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
        }
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

/// Maximum characters for pre-injected artifacts (to avoid token limits)
const MAX_ARTIFACT_CHARS: usize = 3000;

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

/// Truncate content to a maximum number of characters (UTF-8 safe)
fn truncate_content(content: &str, max_chars: usize) -> String {
    if content.chars().count() <= max_chars {
        content.to_string()
    } else {
        let truncated: String = content.chars().take(max_chars).collect();
        format!("{}{}", truncated, get_truncated_message())
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
fn build_prompt(ctx: &PipelineContext, stage_name: &str, feedback: Option<&str>) -> String {
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
        prompt.push_str("\n");
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
    let mut injected_artifacts = Vec::new();

    match stage_name {
        "prd" => {
            // PRD needs Idea
            if let Some(idea) = load_artifact_content(ctx, "idea.md") {
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str("📋 PRE-LOADED: Idea Document (from previous stage)\n");
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str(&truncate_content(&idea, MAX_ARTIFACT_CHARS));
                prompt.push_str("\n═══════════════════════════════════════════════════════════════\n\n");
                injected_artifacts.push("idea.md");
            }
        }
        "design" => {
            // Design needs PRD
            if let Some(prd) = load_artifact_content(ctx, "prd.md") {
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str("📋 PRE-LOADED: PRD Document (from previous stage)\n");
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str(&truncate_content(&prd, MAX_ARTIFACT_CHARS));
                prompt.push_str("\n═══════════════════════════════════════════════════════════════\n\n");
                injected_artifacts.push("prd.md");
            }
        }
        "plan" => {
            // Plan needs Design and PRD
            if let Some(design) = load_artifact_content(ctx, "design.md") {
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str("📋 PRE-LOADED: Design Document (from previous stage)\n");
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str(&truncate_content(&design, MAX_ARTIFACT_CHARS));
                prompt.push_str("\n═══════════════════════════════════════════════════════════════\n\n");
                injected_artifacts.push("design.md");
            }
        }
        "coding" => {
            // Coding needs Plan (most important) and Design
            if let Some(plan) = load_artifact_content(ctx, "plan.md") {
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str("📋 PRE-LOADED: Implementation Plan (from previous stage)\n");
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str(&truncate_content(&plan, MAX_ARTIFACT_CHARS));
                prompt.push_str("\n═══════════════════════════════════════════════════════════════\n\n");
                injected_artifacts.push("plan.md");
            }
            // Also include design for architecture context
            if let Some(design) = load_artifact_content(ctx, "design.md") {
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str("📋 PRE-LOADED: Design Document (architecture reference)\n");
                prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                prompt.push_str(&truncate_content(&design, 2000)); // Smaller for coding
                prompt.push_str("\n═══════════════════════════════════════════════════════════════\n\n");
                injected_artifacts.push("design.md");
            }
        }
        "check" | "delivery" => {
            // Check and Delivery need all artifacts
            let artifacts = [
                ("idea.md", "Idea Document"),
                ("prd.md", "PRD Document"),
                ("design.md", "Design Document"),
                ("plan.md", "Implementation Plan"),
            ];

            for (filename, label) in artifacts {
                if let Some(content) = load_artifact_content(ctx, filename) {
                    prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                    prompt.push_str(&format!("📋 PRE-LOADED: {}\n", label));
                    prompt.push_str("═══════════════════════════════════════════════════════════════\n");
                    prompt.push_str(&truncate_content(&content, 2000)); // Smaller for all artifacts
                    prompt.push_str("\n═══════════════════════════════════════════════════════════════\n\n");
                    injected_artifacts.push(filename);
                }
            }
        }
        _ => {}
    }

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
            if injected_artifacts.is_empty() {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load idea using load_idea() tool\n");
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. The Idea document is provided above (pre-loaded)\n");
            }
            prompt.push_str("2. Analyze the idea and create requirements\n");
            prompt.push_str("3. SAVE PRD using save_prd_doc() tool (MANDATORY)\n\n");
        }
        "design" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Design (System Architecture)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.is_empty() {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load requirements using get_requirements() tool\n");
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. The PRD document is provided above (pre-loaded)\n");
            }
            prompt.push_str("2. Design system architecture (2-4 components max)\n");
            prompt.push_str("3. SAVE DESIGN using save_design_doc() tool (MANDATORY)\n\n");
        }
        "plan" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Plan (Implementation Tasks)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.is_empty() {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load design using get_design() tool\n");
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. The Design document is provided above (pre-loaded)\n");
            }
            prompt.push_str("2. Create 5-12 simple implementation tasks\n");
            prompt.push_str("3. SAVE PLAN using save_plan_doc() tool (MANDATORY)\n\n");
        }
        "coding" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Coding (Implementation)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.is_empty() {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load plan using get_plan() tool\n");
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. The Plan and Design documents are provided above (pre-loaded)\n");
            }
            prompt.push_str("2. Implement tasks one by one\n");
            prompt.push_str("3. Update task status using update_task_status() tool\n\n");
        }
        "check" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Check (Quality Assurance)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.is_empty() {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load all artifacts (requirements, design, plan)\n");
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. All artifacts are provided above (pre-loaded)\n");
            }
            prompt.push_str("2. Run quality checks\n");
            prompt.push_str("3. Use goto_stage() if issues found\n\n");
        }
        "delivery" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Delivery (Final Report)\n");
            prompt.push_str("========================================\n");
            if injected_artifacts.is_empty() {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. Load all artifacts\n");
            } else {
                prompt.push_str("YOUR TASK:\n");
                prompt.push_str("1. All artifacts are provided above (pre-loaded)\n");
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
            // Memory and Artifacts are accessed via Tools (QueryMemoryTool, LoadArtifactTool, etc.)
            // rather than through InvocationContext. This is intentional - tools provide more
            // flexible access with proper validation and error handling.
            memory: None,
            session: Box::new(SimpleSession::new(&ctx.iteration.id, content.clone())),
            run_config: adk_core::RunConfig {
                streaming_mode: adk_core::StreamingMode::SSE,
                ..adk_core::RunConfig::default()
            },
            ended: std::sync::atomic::AtomicBool::new(false),
            artifacts: None,
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

/// Simple Session implementation
struct SimpleSession {
    session_id: String,
    app_name: String,
    user_id: String,
    simple_state: SimpleState,
    messages: Vec<Content>,
}

impl SimpleSession {
    fn new(session_id: &str, initial_message: Content) -> Self {
        Self {
            session_id: session_id.to_string(),
            app_name: "cowork_forge".to_string(),
            user_id: "default_user".to_string(),
            simple_state: SimpleState::new(),
            messages: vec![initial_message],
        }
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
        self.messages.clone()
    }

    fn append_to_history(&self, _content: Content) {
        // Simple implementation - doesn't store history
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
