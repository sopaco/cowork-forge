// Tauri implementation of InteractiveBackend (placeholder)
// Actual implementation will be in cowork-gui crate

use super::{InteractiveBackend, InputOption, InputResponse, MessageLevel, MessageContext, ProgressInfo};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Tauri backend placeholder - will be properly implemented in cowork-gui crate
pub struct TauriBackend {
    // event_bus removed in V2
}

impl TauriBackend {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl InteractiveBackend for TauriBackend {
    async fn show_message(&self, level: MessageLevel, content: String) {
        // Tauri implementation will send events to frontend
        println!("{} [Tauri]: {}", level.emoji(), content);
    }

    async fn show_message_with_context(&self, level: MessageLevel, content: String, context: MessageContext) {
        // Display agent name prefix for better clarity
        let prefix = match &context.stage_name {
            Some(stage) => format!("[{}:{}]", context.agent_name, stage),
            None => format!("[{}]", context.agent_name),
        };
        println!("{} {} {}", level.emoji(), prefix, content);
    }

    async fn send_streaming(&self, content: String, agent_name: &str, is_thinking: bool) {
        let prefix = if is_thinking { "💭" } else { "📝" };
        println!("{} [{}] {}", prefix, agent_name, content);
    }

    async fn send_tool_call(&self, tool_name: &str, arguments: &Value, agent_name: &str) {
        println!("🔧 [{}] Calling tool: {}", agent_name, tool_name);
    }

    async fn send_tool_result(&self, tool_name: &str, result: &str, success: bool, agent_name: &str) {
        let status = if success { "✓" } else { "✗" };
        println!("{} [{}] Tool {} completed", status, agent_name, tool_name);
    }

    async fn request_input(&self, _prompt: &str, _options: Vec<InputOption>, _initial_content: Option<String>) -> Result<InputResponse> {
        // Tauri implementation will send HITL request event and wait for response
        // For now, return a placeholder
        Ok(InputResponse::Cancel)
    }

    async fn show_progress(&self, task_id: String, progress: ProgressInfo) {
        // Tauri implementation will send progress event to frontend
        let percentage = if progress.total > 0 {
            (progress.current as f64 / progress.total as f64 * 100.0) as u32
        } else {
            0
        };
        println!("[Tauri Progress] [{}%] {}: {}/{}", percentage, task_id, progress.current, progress.total);
    }

    async fn submit_response(&self, request_id: String, response: String) -> Result<()> {
        // Tauri implementation will handle async HITL responses
        println!("[Tauri HITL] Response for {}: {}", request_id, response);
        Ok(())
    }
}