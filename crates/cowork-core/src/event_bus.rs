// Event Bus for real-time communication between engine and UI
// Uses tokio broadcast channel for multi-subscriber support

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Engine events that can be published to subscribers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineEvent {
    // Stage events
    StageStarted {
        stage: String,
        session_id: String,
        timestamp: i64,
    },
    StageCompleted {
        stage: String,
        session_id: String,
        duration_ms: u64,
    },

    // Agent events
    AgentThinking {
        agent: String,
        content: String,
        session_id: String,
    },
    AgentOutput {
        agent: String,
        content: String,
        session_id: String,
    },

    // Tool events
    ToolStarted {
        tool: String,
        session_id: String,
    },
    ToolCompleted {
        tool: String,
        result: String,
        session_id: String,
    },

    // File events
    FileCreated {
        path: String,
        size: u64,
        session_id: String,
    },
    FileModified {
        path: String,
        session_id: String,
    },
    FileDeleted {
        path: String,
        session_id: String,
    },

    // Error events
    Error {
        error: String,
        context: String,
        session_id: String,
        severity: ErrorSeverity,
    },

    // HITL events
    HITLRequest {
        tool: String,
        content: String,
        options: Vec<String>,
        session_id: String,
    },
    HITLResponse {
        tool: String,
        response: String,
        session_id: String,
    },

    // Progress events
    Progress {
        task_id: String,
        current: u32,
        total: u32,
        message: String,
    },
}

/// Error severity level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Event bus for publishing and subscribing to engine events
pub struct EventBus {
    sender: broadcast::Sender<EngineEvent>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<EngineEvent> {
        self.sender.subscribe()
    }

    /// Publish an event
    pub async fn publish(&self, event: EngineEvent) {
        self.sender.send(event).ok();
    }

    /// Get sender for direct use (needed for Arc wrapping)
    pub fn sender(&self) -> broadcast::Sender<EngineEvent> {
        self.sender.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for creating specific events
impl EngineEvent {
    pub fn stage_started(stage: &str, session_id: &str) -> Self {
        Self::StageStarted {
            stage: stage.to_string(),
            session_id: session_id.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn stage_completed(stage: &str, session_id: &str, duration_ms: u64) -> Self {
        Self::StageCompleted {
            stage: stage.to_string(),
            session_id: session_id.to_string(),
            duration_ms,
        }
    }

    pub fn agent_thinking(agent: &str, content: &str, session_id: &str) -> Self {
        Self::AgentThinking {
            agent: agent.to_string(),
            content: content.to_string(),
            session_id: session_id.to_string(),
        }
    }

    pub fn agent_output(agent: &str, content: &str, session_id: &str) -> Self {
        Self::AgentOutput {
            agent: agent.to_string(),
            content: content.to_string(),
            session_id: session_id.to_string(),
        }
    }

    pub fn tool_started(tool: &str, session_id: &str) -> Self {
        Self::ToolStarted {
            tool: tool.to_string(),
            session_id: session_id.to_string(),
        }
    }

    pub fn tool_completed(tool: &str, result: &str, session_id: &str) -> Self {
        Self::ToolCompleted {
            tool: tool.to_string(),
            result: result.to_string(),
            session_id: session_id.to_string(),
        }
    }

    pub fn file_created(path: &str, session_id: &str) -> Self {
        Self::FileCreated {
            path: path.to_string(),
            size: 0,
            session_id: session_id.to_string(),
        }
    }

    pub fn file_deleted(path: &str, session_id: &str) -> Self {
        Self::FileDeleted {
            path: path.to_string(),
            session_id: session_id.to_string(),
        }
    }

    pub fn error(error: &str, context: &str, session_id: &str, severity: ErrorSeverity) -> Self {
        Self::Error {
            error: error.to_string(),
            context: context.to_string(),
            session_id: session_id.to_string(),
            severity,
        }
    }
}