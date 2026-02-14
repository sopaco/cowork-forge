// Stage Executor - Uses real adk-rust Agents (LlmAgentBuilder, LoopAgent)
//
// This module provides a unified execution framework for all stages:
// - Uses real adk-rust Agents created with LlmAgentBuilder
// - Invokes agent functions from agents/mod.rs
// - Handles feedback and iteration
// - Saves artifacts
// - Sends real-time streaming output

use crate::agents::*;
use crate::interaction::{InteractiveBackend, MessageContext};
use crate::llm::{ModelConfig, create_llm_client};
use crate::pipeline::{PipelineContext, StageResult};
use crate::storage::set_iteration_id;
use adk_core::{Content, Event};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

/// Map internal agent names to user-friendly display names
fn get_display_name(agent_name: &str) -> &'static str {
    match agent_name {
        // Individual agent names
        "idea_agent" => "Idea Agent",
        "prd_actor" => "PRD Agent",
        "prd_critic" => "PRD Reviewer",
        "design_actor" => "Design Agent",
        "design_critic" => "Design Reviewer",
        "plan_actor" => "Plan Agent",
        "plan_critic" => "Plan Reviewer",
        "coding_actor" => "Coding Agent",
        "coding_critic" => "Code Reviewer",
        "check_agent" => "Validation Agent",
        "delivery_agent" => "Delivery Agent",
        "summary_agent" => "Summary Agent",
        "knowledge_gen_agent" => "Knowledge Agent",

        // Loop agent names (composite)
        "prd_loop" => "PRD Agent",
        "design_loop" => "Design Agent",
        "plan_loop" => "Plan Agent",
        "coding_loop" => "Coding Agent",

        // System agents
        "Pipeline Controller" => "Pipeline",
        "Memory System" => "Memory",
        "Knowledge System" => "Knowledge",

        // Fallback
        _ => "AI Agent",
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
    if let Ok(Some(session_meta)) = crate::storage::load_session_meta() {
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
            if let Ok(mut meta) = crate::storage::load_session_meta() {
                if let Some(ref mut m) = meta {
                    m.restart_reason = None;
                    let _ = crate::storage::save_session_meta(m);
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

        // Prepare artifact path - V2 architecture: .cowork-v2/iterations/{iteration_id}/artifacts/{stage_name}.md
        let artifact_path = format!("{}/artifacts/{}.md", iteration_dir.display(), stage_name);

        // Ensure artifacts directory exists
        let artifacts_dir = format!("{}/artifacts", iteration_dir.display());
        if let Err(e) = std::fs::create_dir_all(&artifacts_dir) {
            return Err(format!("Failed to create artifacts directory: {}", e));
        }

        // Load LLM client
        let llm_config = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
        let model = create_llm_client(&llm_config.llm)
            .map_err(|e| format!("Failed to create LLM client: {}", e))?;

        // Create appropriate agent based on stage name
        let agent = match stage_name {
            "idea" => create_idea_agent_with_id(model, ctx.iteration.id.clone())
                .map_err(|e| format!("Failed to create idea agent: {}", e))?,
            "prd" => create_prd_loop_with_id(model, ctx.iteration.id.clone())
                .map_err(|e| format!("Failed to create PRD agent: {}", e))?,
            "design" => create_design_loop_with_id(model, ctx.iteration.id.clone())
                .map_err(|e| format!("Failed to create design agent: {}", e))?,
            "plan" => create_plan_loop_with_id(model, ctx.iteration.id.clone())
                .map_err(|e| format!("Failed to create plan agent: {}", e))?,
            "coding" => create_coding_loop_with_id(model, ctx.iteration.id.clone())
                .map_err(|e| format!("Failed to create coding agent: {}", e))?,
            "check" => create_check_agent_with_id(model, ctx.iteration.id.clone())
                .map_err(|e| format!("Failed to create check agent: {}", e))?,
            "delivery" => create_delivery_agent_with_id(model, ctx.iteration.id.clone())
                .map_err(|e| format!("Failed to create delivery agent: {}", e))?,
            _ => return Err(format!("Unknown stage: {}", stage_name)),
        };

        Ok((agent, artifact_path))
    }
    .await;

    let (agent, _artifact_path) = match result {
        Ok(v) => v,
        Err(e) => return StageResult::Failed(e),
    };

    // Get the actual agent name and map to user-friendly display name
    let internal_name = agent.name();
    let display_name = get_display_name(internal_name).to_string();

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

    let mut stream = std::pin::pin!(stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                // Extract content from the event using the event's content() method
                if let Some(content) = event.content() {
                    if let Some(text) = extract_text_from_content(content) {
                        if !text.trim().is_empty() {
                            generated_text.push_str(&text);
                            // Send content in real-time with display name
                            interaction
                                .send_streaming(text.clone(), &display_name, false)
                                .await;
                        }
                    }
                } else if let Some(text) = extract_text_from_event(&event) {
                    // Fallback: use helper function
                    if !text.trim().is_empty() {
                        generated_text.push_str(&text);
                        interaction.send_streaming(text, &display_name, false).await;
                    }
                }
            }
            Err(e) => {
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
        return StageResult::Failed("Agent produced no output".to_string());
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

    StageResult::Success(None)
}

/// Build prompt with iteration context
fn build_prompt(ctx: &PipelineContext, stage_name: &str, feedback: Option<&str>) -> String {
    let mut prompt = format!(
        "You are working on iteration #{} - '{}'.\n",
        ctx.iteration.number, ctx.iteration.title
    );

    prompt.push_str(&format!("Iteration ID: {}\n\n", ctx.iteration.id));

    // Provide stage-specific guidance
    match stage_name {
        "idea" => {
            prompt.push_str("========================================\n");
            prompt.push_str("USER'S PROJECT IDEA (ALREADY PROVIDED):\n");
            prompt.push_str("========================================\n");
            prompt.push_str(&ctx.iteration.description);
            prompt.push_str("\n========================================\n\n");
            prompt.push_str("YOUR TASK:\n");
            prompt.push_str("1. Read and understand the project idea above\n");
            prompt.push_str("2. Generate a structured idea document\n");
            prompt.push_str("3. SAVE IT using the save_idea() tool (MANDATORY)\n\n");
        }
        "prd" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: PRD (Product Requirements Document)\n");
            prompt.push_str("========================================\n");
            prompt.push_str("YOUR TASK:\n");
            prompt.push_str("1. Load idea using load_idea() tool\n");
            prompt.push_str("2. Analyze the idea and create requirements\n");
            prompt.push_str("3. SAVE PRD using save_prd_doc() tool (MANDATORY)\n\n");
            prompt.push_str(&format!(
                "Original request: {}\n\n",
                ctx.iteration.description
            ));
        }
        "design" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Design (System Architecture)\n");
            prompt.push_str("========================================\n");
            prompt.push_str("YOUR TASK:\n");
            prompt.push_str("1. Load requirements using get_requirements() tool\n");
            prompt.push_str("2. Design system architecture (2-4 components max)\n");
            prompt.push_str("3. SAVE DESIGN using save_design_doc() tool (MANDATORY)\n\n");
        }
        "plan" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Plan (Implementation Tasks)\n");
            prompt.push_str("========================================\n");
            prompt.push_str("YOUR TASK:\n");
            prompt.push_str("1. Load design using get_design() tool\n");
            prompt.push_str("2. Create 5-12 simple implementation tasks\n");
            prompt.push_str("3. SAVE PLAN using save_plan_doc() tool (MANDATORY)\n\n");
        }
        "coding" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Coding (Implementation)\n");
            prompt.push_str("========================================\n");
            prompt.push_str("YOUR TASK:\n");
            prompt.push_str("1. Load plan using get_plan() tool\n");
            prompt.push_str("2. Implement tasks one by one\n");
            prompt.push_str("3. Update task status using update_task_status() tool\n\n");
        }
        "check" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Check (Quality Assurance)\n");
            prompt.push_str("========================================\n");
            prompt.push_str("YOUR TASK:\n");
            prompt.push_str("1. Load all artifacts (requirements, design, plan)\n");
            prompt.push_str("2. Run quality checks\n");
            prompt.push_str("3. Use goto_stage() if issues found\n\n");
        }
        "delivery" => {
            prompt.push_str("========================================\n");
            prompt.push_str("STAGE: Delivery (Final Report)\n");
            prompt.push_str("========================================\n");
            prompt.push_str("YOUR TASK:\n");
            prompt.push_str("1. Load all artifacts\n");
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

    prompt
}

/// Load config from file or environment
fn load_config() -> Result<ModelConfig, String> {
    use std::path::Path;

    // Try loading from config.toml
    if Path::new("config.toml").exists() {
        ModelConfig::from_file("config.toml").map_err(|e| format!("Failed to load config: {}", e))
    } else if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(&exe_path);
        let config_path = exe_dir.join("config.toml");
        if config_path.exists() {
            ModelConfig::from_file(config_path.to_str().unwrap())
                .map_err(|e| format!("Failed to load config: {}", e))
        } else {
            ModelConfig::from_env().map_err(|e| format!("Failed to load config from env: {}", e))
        }
    } else {
        ModelConfig::from_env().map_err(|e| format!("Failed to load config from env: {}", e))
    }
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
            memory: None, // TODO: implement memory
            session: Box::new(SimpleSession::new(&ctx.iteration.id, content.clone())),
            run_config: adk_core::RunConfig {
                streaming_mode: adk_core::StreamingMode::None,
                tool_confirmation_decisions: HashMap::new(),
            },
            ended: std::sync::atomic::AtomicBool::new(false),
            artifacts: None, // TODO: implement artifacts
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
