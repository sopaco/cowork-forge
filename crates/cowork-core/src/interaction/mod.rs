// Interaction abstraction layer - decouples core engine from UI implementations
// This module defines the trait that different UI backends (CLI, GUI) must implement

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod cli;
pub mod tauri;

/// Message level for UI feedback
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
    Debug,
}

impl MessageLevel {
    pub fn emoji(&self) -> &str {
        match self {
            MessageLevel::Info => "ℹ️",
            MessageLevel::Success => "✅",
            MessageLevel::Warning => "⚠️",
            MessageLevel::Error => "❌",
            MessageLevel::Debug => "🔍",
        }
    }
}

/// Input option for user selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
}

/// User response to an input request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputResponse {
    Text(String),
    Selection(String),
    Cancel,
}

/// Progress information for long-running tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current: u32,
    pub total: u32,
    pub message: String,
}

/// Interaction backend trait - must be implemented by all UI frontends
#[async_trait]
pub trait InteractiveBackend: Send + Sync {
    /// Show a message to the user
    async fn show_message(&self, level: MessageLevel, content: String);

    /// Request user input - either text or selection
    /// initial_content: Optional initial content to display in editor (for CLI edit mode)
    async fn request_input(&self, prompt: &str, options: Vec<InputOption>, initial_content: Option<String>) -> Result<InputResponse>;

    /// Show progress for a long-running task
    async fn show_progress(&self, task_id: String, progress: ProgressInfo);

    /// Submit an async HITL response (for GUI backends)
    async fn submit_response(&self, request_id: String, response: String) -> Result<()>;

    /// Get a clone of the event bus for subscription
    fn event_bus(&self) -> Arc<crate::event_bus::EventBus>;
}

// Re-export implementations
pub use cli::CliBackend;
pub use tauri::TauriBackend;