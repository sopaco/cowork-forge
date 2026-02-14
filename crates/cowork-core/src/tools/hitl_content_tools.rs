// HITL tools (content-based) using InteractiveBackend
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use crate::interaction::{InteractiveBackend, InputOption, InputResponse, MessageLevel};

// Global InteractiveBackend for HITL tools
static INTERACTION_BACKEND: Lazy<Mutex<Option<Arc<dyn InteractiveBackend + Send + Sync>>>> = 
    Lazy::new(|| Mutex::new(None));

/// Set the global InteractiveBackend
pub fn set_interaction_backend(backend: Arc<dyn InteractiveBackend + Send + Sync>) {
    *INTERACTION_BACKEND.lock().unwrap() = Some(backend);
}

/// Get the global InteractiveBackend
fn get_interaction_backend() -> Option<Arc<dyn InteractiveBackend + Send + Sync>> {
    INTERACTION_BACKEND.lock().unwrap().clone()
}

/// review_and_edit_content
/// - Shows content to user
/// - Lets user choose: edit, pass, or provide feedback
/// - Returns action and content/feedback
pub struct ReviewAndEditContentTool;

#[async_trait]
impl Tool for ReviewAndEditContentTool {
    fn name(&self) -> &str {
        "review_and_edit_content"
    }

    fn description(&self) -> &str {
        "Let the user review content and choose: edit, pass, or provide feedback."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Title shown to user"},
                "content": {"type": "string", "description": "Content to review"}
            },
            "required": ["title", "content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let title = args["title"].as_str()
            .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter: title".to_string()))?;
        let content = args["content"].as_str()
            .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter: content".to_string()))?;

        // Get InteractiveBackend
        let interaction = get_interaction_backend()
            .ok_or_else(|| adk_core::AdkError::Tool("InteractiveBackend not set".to_string()))?;

        // Show content to user
        interaction.show_message(
            MessageLevel::Info,
            format!("\n📝 {}\n{}\n---\n{}",
                title,
                "─".repeat(40),
                content.lines().take(15).collect::<Vec<_>>().join("\n")
            )
        ).await;

        // Request user input
        let options = vec![
            InputOption {
                id: "pass".to_string(),
                label: "✓ Pass".to_string(),
                description: Some("Continue without changes".to_string()),
            },
        ];

        let response = interaction.request_input(
            "Type 'edit' to open editor, 'pass' to continue, or provide feedback:",
            options,
            Some(content.to_string())
        ).await.map_err(|e| adk_core::AdkError::Tool(format!("Input error: {}", e)))?;

        match response {
            InputResponse::Selection(id) => match id.as_str() {
                "pass" => Ok(json!({
                    "action": "pass",
                    "content": content,
                    "message": "User passed"
                })),
                _ => Ok(json!({
                    "action": "pass",
                    "content": content,
                    "message": "Unknown action"
                }))
            },
            InputResponse::Text(text) => {
                let text = text.trim();
                // Check if text looks like edited content (multiline) or feedback (short)
                if text.contains('\n') || text.len() > 100 {
                    // Assume this is edited content
                    Ok(json!({
                        "action": "edit",
                        "content": text,
                        "message": "User provided edited content"
                    }))
                } else {
                    // Assume this is feedback
                    Ok(json!({
                        "action": "feedback",
                        "feedback": text,
                        "content": content,
                        "message": "User provided feedback"
                    }))
                }
            },
            InputResponse::Cancel => Ok(json!({
                "action": "pass",
                "content": content,
                "message": "User cancelled"
            })),
        }
    }
}

/// review_with_feedback_content
/// - Shows content to user
/// - Allows edit/pass/feedback
/// - Returns edited content OR feedback text
pub struct ReviewWithFeedbackContentTool;

#[async_trait]
impl Tool for ReviewWithFeedbackContentTool {
    fn name(&self) -> &str {
        "review_with_feedback_content"
    }

    fn description(&self) -> &str {
        "Review content and allow user to: edit, pass, or provide feedback text."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "content": {"type": "string"},
                "prompt": {"type": "string"}
            },
            "required": ["title", "content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let title = args["title"].as_str()
            .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter: title".to_string()))?;
        let content = args["content"].as_str()
            .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter: content".to_string()))?;
        let default_prompt = "Type 'edit' to open editor, 'pass' to continue, or provide feedback:";
        let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or(default_prompt);

        // Get InteractiveBackend
        let interaction = get_interaction_backend()
            .ok_or_else(|| adk_core::AdkError::Tool("InteractiveBackend not set".to_string()))?;

        // Show content to user
        interaction.show_message(
            MessageLevel::Info,
            format!("\n📝 {}\n{}\n---\n{}",
                title,
                "─".repeat(40),
                content.lines().take(15).collect::<Vec<_>>().join("\n")
            )
        ).await;

        // Request user input
        let options = vec![
            InputOption {
                id: "pass".to_string(),
                label: "✓ Pass".to_string(),
                description: Some("Continue without changes".to_string()),
            },
        ];

        let response = interaction.request_input(prompt, options, Some(content.to_string()))
            .await.map_err(|e| adk_core::AdkError::Tool(format!("Input error: {}", e)))?;

        match response {
            InputResponse::Selection(id) => match id.as_str() {
                "pass" => Ok(json!({
                    "action": "pass",
                    "content": content,
                    "message": "User passed"
                })),
                _ => Ok(json!({
                    "action": "pass",
                    "content": content,
                    "message": "Unknown action"
                }))
            },
            InputResponse::Text(text) => {
                let text = text.trim();
                // Check if text looks like edited content (multiline) or feedback (short)
                if text.contains('\n') || text.len() > 100 {
                    // Assume this is edited content
                    Ok(json!({
                        "action": "edit",
                        "content": text,
                        "message": "User provided edited content"
                    }))
                } else {
                    // Assume this is feedback
                    Ok(json!({
                        "action": "feedback",
                        "feedback": text,
                        "content": content,
                        "message": "User provided feedback"
                    }))
                }
            },
            InputResponse::Cancel => Ok(json!({
                "action": "pass",
                "content": content,
                "message": "User cancelled"
            })),
        }
    }
}