// Agents module - Agent builders using adk-rust
//
// Actor-Critic Loop Design (config-driven, see `config_definition/agent_factory.rs`):
// - LoopAgent runs Actor then Critic for up to `max_iterations` iterations.
// - Each iteration: Actor generates/updates content → Critic reviews.
// - Actor sees prior turns (including Critic feedback) via `IncludeContents::Default`,
//   which the LoopAgent's HistoryTrackingSession preserves across iterations.
// - Critic has two exit signals (both set `EventActions.escalate = true`):
//     * `exit_loop`        — satisfied with the work; loop exits, stage succeeds.
//     * `provide_feedback` — records structured feedback and exits the loop so
//       the executor can retry the stage with the feedback (StageResult::NeedsRevision).
// - For minor issues the Critic can also just describe them in its response
//   WITHOUT calling any tool; the Actor will pick them up from conversation
//   history in the next loop iteration.
// - If the loop exhausts `max_iterations` without an early exit, the
//   stage_executor falls back to checking pending feedback to decide
//   Success vs NeedsRevision.
//
// Anti-loop protection: MAX_STAGE_RETRIES=3 (executor level) + Critic anti-loop rules.
//
// NOTE: Stage-specific agent construction is config-driven. See
// `config_definition/agent_factory.rs::create_agent_for_stage` and the JSON
// definitions under `config_definition/default_configs/agents/built-in/`.

use crate::instructions::*;
use crate::tools::*;
use crate::IterationStore;
use adk_agent::LlmAgentBuilder;
use adk_core::{Llm, IncludeContents};
use anyhow::Result;
use std::sync::Arc;

// External Coding Agent (ACP-based)
pub mod external_coding_agent;
pub use external_coding_agent::{ExternalCodingAgent, StreamingTask};

// Legacy Project Analyzer Agent
pub mod legacy_project_analyzer;
pub use legacy_project_analyzer::{create_legacy_project_analyzer, create_legacy_project_analyzer_with_id, create_legacy_project_analyzer_with_context};

// ============================================================================
// Summary Agent - Generates summaries of iteration documents
// ============================================================================

pub fn create_summary_agent(model: Arc<dyn Llm>, iteration_id: String, iteration_number: u32) -> Result<Arc<dyn adk_core::Agent>> {
    let instruction = SUMMARY_AGENT_INSTRUCTION
        .replace("{iteration_id}", &iteration_id)
        .replace("{iteration_number}", &iteration_number.to_string());

    let agent = LlmAgentBuilder::new(SUMMARY_AGENT_NAME)
        .instruction(&instruction)
        .model(model)
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// Knowledge Generation Agent - Extracts project-level knowledge from iterations
// ============================================================================

pub fn create_knowledge_generation_agent(
    model: Arc<dyn Llm>,
    iteration_id: String,
    iteration_number: u32,
    base_iteration_id: Option<String>
) -> Result<Arc<dyn adk_core::Agent>> {
    let instruction = KNOWLEDGE_GEN_AGENT_INSTRUCTION
        .replace("{iteration_id}", &iteration_id)
        .replace("{iteration_number}", &iteration_number.to_string());

    let read_file_with_limit = Arc::new(ReadFileWithLimitTool::new(10)); // Limit to 10 calls

    let mut builder = LlmAgentBuilder::new(KNOWLEDGE_GEN_AGENT_NAME)
        .instruction(&instruction)
        .model(model)
        .tool(Arc::new(LoadDocumentSummaryTool::new(iteration_id.clone())))
        .tool(read_file_with_limit.clone())
        .tool(Arc::new(ListFilesWorkspaceTool))
        .tool(Arc::new(SaveKnowledgeSnapshotTool::new(iteration_id.clone(), iteration_number)))
        .include_contents(IncludeContents::None);

    // Add base knowledge tool if this is an evolution iteration
    if let Some(base_id) = base_iteration_id {
        builder = builder.tool(Arc::new(LoadBaseKnowledgeTool::new(base_id)));
    }

    let agent = builder.build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// Project Manager Agent - Post-delivery chat agent
// ============================================================================

pub fn create_project_manager_agent(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    let instruction = PROJECT_MANAGER_AGENT_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);

    let mut builder = LlmAgentBuilder::new("project_manager_agent")
        .instruction(&instruction)
        .model(model)
        .tool(Arc::new(PMGotoStageTool::new(iteration_id.clone())))
        .tool(Arc::new(PMCreateIterationTool::new(iteration_id.clone())))
        .tool(Arc::new(PMRespondTool))
        .tool(Arc::new(PMSaveDecisionTool::new(iteration_id.clone())))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(ListFilesTool))  // Allow PM to see project files
        .tool(Arc::new(ReadFileTool))   // Allow PM to read files
        .include_contents(IncludeContents::None);

    // Add MCP toolsets if available
    builder = crate::config_definition::agent_factory::add_mcp_toolsets_to_builder(builder);

    let agent = builder.build()?;

    Ok(Arc::new(agent))
}

/// Load artifacts summary for a given iteration
fn load_artifacts_summary_for_pm(iteration_store: &IterationStore, iteration_id: &str) -> Result<String, String> {
    use std::fs;

    let iteration_dir = iteration_store.iteration_path(iteration_id)
        .map_err(|e| format!("Failed to get iteration path: {}", e))?;

    let mut summary = String::new();

    // Load key artifacts
    let artifacts_to_load = [
        ("idea", "idea.md"),
        ("prd", "prd.md"),
        ("design", "design.md"),
        ("plan", "plan.md"),
    ];

    for (name, filename) in artifacts_to_load.iter() {
        let path = iteration_dir.join("artifacts").join(filename);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                // Only include first 500 chars of each artifact (UTF-8 safe)
                let truncated = if content.chars().count() > 500 {
                    format!("{}...[truncated]", content.chars().take(500).collect::<String>())
                } else {
                    content
                };
                summary.push_str(&format!("\n\n## {} ({}):\n{}", name.to_uppercase(), filename, truncated));
            }
        }
    }

    // Add code structure info
    let code_dir = iteration_dir.join("workspace");
    if code_dir.exists() {
        summary.push_str("\n\n## Project Files:\n");
        if let Ok(entries) = fs::read_dir(&code_dir) {
            for entry in entries.flatten().take(20) {
                if let Ok(name) = entry.file_name().into_string() {
                    summary.push_str(&format!("- {}\n", name));
                }
            }
        }
    }

    Ok(summary)
}

/// PM Agent execution result containing response and detected actions
#[derive(Debug, Clone)]
pub struct PMAgentResult {
    /// The agent's text response
    pub message: String,
    /// Actions detected from tool calls (pm_goto_stage, pm_create_iteration)
    pub actions: Vec<PMAgentAction>,
    /// Raw parts from the response (for debugging)
    pub parts: Vec<adk_core::Part>,
}

/// Actions that the PM Agent can trigger
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "action_type")]
pub enum PMAgentAction {
    /// Jump to a specific pipeline stage
    #[serde(rename = "pm_goto_stage")]
    GotoStage {
        target_stage: String,
        reason: String,
    },
    /// Create a new iteration
    #[serde(rename = "pm_create_iteration")]
    CreateIteration {
        iteration_id: String,
        title: String,
        description: String,
        inheritance: String,
    },
}

/// Callback trait for streaming PM agent responses
#[async_trait::async_trait]
pub trait PMAgentStreamCallback: Send + Sync {
    /// Called for each text chunk during streaming
    async fn on_text_chunk(&self, text: &str, is_first: bool, is_last: bool);
    /// Called when a tool is invoked
    async fn on_tool_call(&self, tool_name: &str, args: &serde_json::Value);
}

/// Execute a PM agent message with streaming support
pub async fn execute_pm_agent_message_streaming(
    model: Arc<dyn Llm>,
    iteration_id: String,
    message: String,
    history: Vec<serde_json::Value>,
    stream_callback: Option<Arc<dyn PMAgentStreamCallback>>,
) -> Result<PMAgentResult, String> {
    use adk_core::Content;
    use futures::StreamExt;
    use crate::pipeline::stage_executor::{SimpleInvocationContext, extract_text_from_event};
    use crate::pipeline::PipelineContext;
    use crate::persistence::{ProjectStore, IterationStore};
    use std::sync::Arc as StdArc;

    // Load project and iteration
    let project_store = ProjectStore::new();
    let project = project_store.load()
        .map_err(|e| format!("Failed to load project: {}", e))?
        .ok_or_else(|| "No project found".to_string())?;

    let iteration_store = IterationStore::new();
    let iteration = iteration_store.load(&iteration_id)
        .map_err(|e| format!("Failed to load iteration: {}", e))?;

    // Get workspace path
    let workspace_path = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace path: {}", e))?;

    // Load artifacts summary for context
    let artifacts_summary = load_artifacts_summary_for_pm(&iteration_store, &iteration_id)
        .unwrap_or_else(|e| {
            tracing::warn!("[PM Agent] Failed to load artifacts: {}", e);
            String::new()
        });

    // Load memory/decisions
    let memory_store = crate::persistence::MemoryStore::new();
    let project_memory = memory_store.load_project_memory()
        .map_err(|e| format!("Failed to load memory: {}", e))
        .unwrap_or_default();

    let decisions_summary = if !project_memory.decisions.is_empty() {
        let mut summary = String::from("\n\n## Previous Decisions:\n");
        for decision in project_memory.decisions.iter().take(10) {
            summary.push_str(&format!("- {}: {}\n", decision.title, decision.decision));
        }
        summary
    } else {
        String::new()
    };

    // Create PM Agent
    let pm_agent = create_project_manager_agent(model, iteration_id.clone())
        .map_err(|e| format!("Failed to create PM agent: {}", e))?;

    // Build conversation history string
    let conversation_history = if !history.is_empty() {
        let mut history_str = String::from("\n\n## Conversation History:\n");
        for msg in history.iter() {
            let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let role = if msg_type == "user" { "User" } else { "Assistant" };
            history_str.push_str(&format!("{}: {}\n", role, content));
        }
        history_str
    } else {
        String::new()
    };

    // Build language instruction from global config
    let language_instruction = crate::config::get_language_instruction();

    // Build prompt with context
    let prompt = format!(
        "User message: {}\n\n\
        ## Current Iteration Info:\n\
        - Title: {}\n\
        - Description: {}\n\
        - Status: {}\n\
        - Current Stage: {}\n\
        {}\
        {}\
        {}\
        \n\nPlease analyze the user's request and respond appropriately. \
        If the user wants to fix a bug or make changes, use the appropriate tool (pm_goto_stage or pm_create_iteration). \
        If you need more information, use pm_respond to ask for clarification.\n\n{}",
        message,
        iteration.title,
        iteration.description,
        format!("{:?}", iteration.status),
        iteration.current_stage.clone().unwrap_or_default(),
        artifacts_summary,
        decisions_summary,
        conversation_history,
        language_instruction
    );

    // Create content
    let content = Content::new("user").with_text(prompt);

    // Create context
    let ctx = PipelineContext::new(project, iteration, workspace_path);

    // Create invocation context
    let invocation_ctx = StdArc::new(SimpleInvocationContext::new(
        &ctx,
        &content,
        pm_agent.clone(),
    ));

    // Execute agent
    let mut stream = pm_agent.run(invocation_ctx)
        .await
        .map_err(|e| format!("Agent execution failed: {}", e))?;

    // Collect response with streaming
    let mut agent_message = String::new();
    let mut all_parts: Vec<adk_core::Part> = Vec::new();
    let mut detected_actions: Vec<PMAgentAction> = Vec::new();
    let mut is_first_chunk = true;
    let mut pending_create_iteration: Option<(String, String, String)> = None;

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                // Extract text content and stream it
                if let Some(text) = extract_text_from_event(&event) {
                    if !text.trim().is_empty() {
                        agent_message.push_str(&text);

                        // Call streaming callback if provided
                        if let Some(ref callback) = stream_callback {
                            callback.on_text_chunk(&text, is_first_chunk, false).await;
                        }
                        is_first_chunk = false;
                    }
                }

                // Collect all parts (includes function calls)
                if let Some(content) = event.content() {
                    for part in &content.parts {
                        // Check for function calls
                        if let adk_core::Part::FunctionCall { name, args, .. } = part {
                            // Handle known tool calls
                            match name.as_str() {
                                "pm_goto_stage" => {
                                    if let (Some(stage), Some(reason)) = (
                                        args.get("stage").and_then(|v| v.as_str()),
                                        args.get("reason").and_then(|v| v.as_str()),
                                    ) {
                                        detected_actions.push(PMAgentAction::GotoStage {
                                            target_stage: stage.to_string(),
                                            reason: reason.to_string(),
                                        });
                                    }
                                }
                                "pm_create_iteration" => {
                                    // Store the parameters for later (we'll check if iteration was created)
                                    let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let inheritance = args.get("inheritance").and_then(|v| v.as_str()).unwrap_or("partial").to_string();
                                    pending_create_iteration = Some((title, description, inheritance));
                                }
                                _ => {}
                            }

                            // Notify callback about tool call
                            if let Some(ref callback) = stream_callback {
                                callback.on_tool_call(name, args).await;
                            }
                        }

                        all_parts.push(part.clone());
                    }
                }
            }
            Err(e) => {
                tracing::warn!("[PM Agent] Event error: {}", e);
            }
        }
    }

    // Send final streaming callback
    if let Some(ref callback) = stream_callback {
        callback.on_text_chunk("", false, true).await;
    }

    // Check if pm_create_iteration was called and iteration was created
    if let Some((title, description, inheritance)) = pending_create_iteration {
        // Check if a new iteration was created by looking for the most recent one
        // The PMCreateIterationTool saves the iteration, so we need to find it
        let iteration_store = crate::persistence::IterationStore::new();
        if let Ok(iterations) = iteration_store.load_all() {
            // Find the most recently created iteration (should be the one just created)
            if let Some(new_iteration) = iterations.iter().max_by_key(|i| i.started_at) {
                // Verify it's a new iteration (not the current one)
                if new_iteration.id != iteration_id {
                    detected_actions.push(PMAgentAction::CreateIteration {
                        iteration_id: new_iteration.id.clone(),
                        title,
                        description,
                        inheritance,
                    });
                }
            }
        }
    }

    // Fallback: if no actions detected but message contains tool references
    if detected_actions.is_empty() {
        let msg_lower = agent_message.to_lowercase();

        if msg_lower.contains("goto_stage") || msg_lower.contains("跳转") || msg_lower.contains("返回") {
            // Try to extract stage from message
            for stage in &["coding", "design", "plan", "prd", "idea"] {
                if msg_lower.contains(stage) {
                    detected_actions.push(PMAgentAction::GotoStage {
                        target_stage: stage.to_string(),
                        reason: "Detected from message".to_string(),
                    });
                    break;
                }
            }
        }
    }

    if agent_message.is_empty() {
        agent_message = "处理完成".to_string();
    }

    // Deduplicate actions - keep only the first occurrence of each unique action
    let mut seen_stages: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_iterations: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unique_actions: Vec<PMAgentAction> = Vec::new();

    for action in detected_actions {
        match &action {
            PMAgentAction::GotoStage { target_stage, .. } => {
                if !seen_stages.contains(target_stage) {
                    seen_stages.insert(target_stage.clone());
                    unique_actions.push(action);
                }
            }
            PMAgentAction::CreateIteration { iteration_id, .. } => {
                if !seen_iterations.contains(iteration_id) {
                    seen_iterations.insert(iteration_id.clone());
                    unique_actions.push(action);
                }
            }
        }
    }

    Ok(PMAgentResult {
        message: agent_message,
        actions: unique_actions,
        parts: all_parts,
    })
}

/// Execute a PM agent message and return the response and function calls (non-streaming version)
pub async fn execute_pm_agent_message(
    model: Arc<dyn Llm>,
    iteration_id: String,
    message: String,
    history: Vec<serde_json::Value>,
) -> Result<(String, Vec<adk_core::Part>), String> {
    let result = execute_pm_agent_message_streaming(model, iteration_id, message, history, None).await?;
    Ok((result.message, result.parts))
}
