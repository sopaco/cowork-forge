// Idea artifact tools (Session-scoped)
use crate::storage::*;
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct SaveIdeaTool {
    session_id: String,
}

impl SaveIdeaTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for SaveIdeaTool {
    fn name(&self) -> &str {
        "save_idea"
    }

    fn description(&self) -> &str {
        "Save idea.md as a session-scoped artifact (.cowork/sessions/<id>/artifacts/idea.md)."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Markdown content of idea.md"
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let content = args["content"].as_str().unwrap();
        save_idea(&self.session_id, content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "Idea saved successfully"
        }))
    }
}

pub struct LoadIdeaTool {
    session_id: String,
}

impl LoadIdeaTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for LoadIdeaTool {
    fn name(&self) -> &str {
        "load_idea"
    }

    fn description(&self) -> &str {
        "Load idea.md from current session artifacts."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({"type": "object", "properties": {}}))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let content = load_idea(&self.session_id)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "content": content
        }))
    }
}
