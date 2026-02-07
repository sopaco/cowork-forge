// Tauri implementation of InteractiveBackend (placeholder)
// Actual implementation will be in cowork-gui crate

use super::{InteractiveBackend, InputOption, InputResponse, MessageLevel, ProgressInfo};
use anyhow::Result;
use async_trait::async_trait;

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