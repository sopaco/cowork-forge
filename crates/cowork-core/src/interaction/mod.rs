// Interaction abstraction layer - decouples core engine from UI implementations
// This module defines the trait that different UI backends (CLI, GUI) must implement

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

/// Message type for categorizing different kinds of agent outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Normal message
    Normal,
    /// Thinking/reasoning process
    Thinking,
    /// Tool call notification
    ToolCall { tool_name: String, arguments: Value },
    /// Tool execution result
    ToolResult { tool_name: String, success: bool },
    /// Streaming content (incremental)
    Streaming { is_first: bool, is_last: bool },
}

/// Context for a message, providing metadata about the source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContext {
    /// Agent name (from agent.name())
    pub agent_name: String,
    /// Message type
    pub message_type: MessageType,
    /// Stage name (if applicable)
    pub stage_name: Option<String>,
}

impl MessageContext {
    /// Create a new message context
    pub fn new(agent_name: impl Into<String>) -> Self {
        Self {
            agent_name: agent_name.into(),
            message_type: MessageType::Normal,
            stage_name: None,
        }
    }

    /// Set the message type
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Set the stage name
    pub fn with_stage(mut self, stage_name: impl Into<String>) -> Self {
        self.stage_name = Some(stage_name.into());
        self
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
    /// Show a message to the user with optional agent context
    async fn show_message(&self, level: MessageLevel, content: String);

    /// Show a message with full context (agent name, message type, etc.)
    async fn show_message_with_context(
        &self,
        level: MessageLevel,
        content: String,
        _context: MessageContext,
    ) {
        // Default implementation: just call show_message
        // Backends can override for richer display
        self.show_message(level, content).await;
    }

    /// Send streaming content (incremental output during agent execution)
    async fn send_streaming(&self, _content: String, _agent_name: &str, _is_thinking: bool) {
        // Default: no-op, backends can override
    }

    /// Send tool call notification
    async fn send_tool_call(&self, _tool_name: &str, _arguments: &Value, _agent_name: &str) {
        // Default: no-op, backends can override
    }

    /// Send tool result notification
    async fn send_tool_result(
        &self,
        _tool_name: &str,
        _result: &str,
        _success: bool,
        _agent_name: &str,
    ) {
        // Default: no-op, backends can override
    }

    /// Request user input - either text or selection
    /// initial_content: Optional initial content to display in editor (for CLI edit mode)
    async fn request_input(
        &self,
        prompt: &str,
        options: Vec<InputOption>,
        initial_content: Option<String>,
    ) -> Result<InputResponse>;

    /// Show progress for a long-running task
    async fn show_progress(&self, task_id: String, progress: ProgressInfo);

    /// Submit an async HITL response (for GUI backends)
    async fn submit_response(&self, request_id: String, response: String) -> Result<()>;
}

// Re-export implementations
pub use cli::CliBackend;
pub use tauri::TauriBackend;
