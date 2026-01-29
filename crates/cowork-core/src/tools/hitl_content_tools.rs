// HITL tools (content-based) to avoid hardcoding artifact file paths
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use dialoguer::{Editor, Input};
use serde_json::{json, Value};
use std::sync::Arc;

/// review_and_edit_content
/// - Takes content as input
/// - Optionally lets user edit in editor
/// - Returns edited content
pub struct ReviewAndEditContentTool;

#[async_trait]
impl Tool for ReviewAndEditContentTool {
    fn name(&self) -> &str {
        "review_and_edit_content"
    }

    fn description(&self) -> &str {
        "Let the user review and optionally edit markdown content using their default editor. Returns edited content." 
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Title shown to user"},
                "content": {"type": "string", "description": "Content to review/edit"}
            },
            "required": ["title", "content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let title = args["title"].as_str().unwrap();
        let content = args["content"].as_str().unwrap();

        println!("\n📝 {}", title);
        println!("  ────────────────────────────────────────");
        for (i, line) in content.lines().take(12).enumerate() {
            println!("  {}: {}", i + 1, line);
        }
        let line_count = content.lines().count();
        if line_count > 12 {
            println!("  ... ({} more lines)", line_count - 12);
        }
        println!("  ────────────────────────────────────────\n");

        let input: String = Input::new()
            .with_prompt("输入 'edit' 打开编辑器，或直接回车跳过")
            .allow_empty(true)
            .interact_text()
            .map_err(|e| adk_core::AdkError::Tool(format!("Interaction error: {}", e)))?;

        if input.trim().to_lowercase() != "edit" {
            return Ok(json!({
                "action": "pass",
                "content": content,
                "message": "User skipped editing"
            }));
        }

        println!("📝 Opening editor... (Save and close to submit changes)");
        let edited = Editor::new()
            .require_save(true)
            .edit(content)
            .map_err(|e| adk_core::AdkError::Tool(format!("Editor error: {}", e)))?;

        let new_content = edited.unwrap_or_else(|| content.to_string());

        Ok(json!({
            "action": "edit",
            "content": new_content,
            "message": "Content edited"
        }))
    }
}

/// review_with_feedback_content
/// - Takes content as input
/// - Allows edit/pass/feedback
/// - Returns edited content OR feedback text
pub struct ReviewWithFeedbackContentTool;

#[async_trait]
impl Tool for ReviewWithFeedbackContentTool {
    fn name(&self) -> &str {
        "review_with_feedback_content"
    }

    fn description(&self) -> &str {
        "Review content and allow user to: edit in editor, pass, or provide feedback text."
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
        let title = args["title"].as_str().unwrap();
        let content = args["content"].as_str().unwrap();
        let default_prompt = "输入 'edit' 编辑，'pass' 继续，或直接输入修改建议";
        let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or(default_prompt);

        println!("\n📝 {}", title);
        println!("  ────────────────────────────────────────");
        for (i, line) in content.lines().take(15).enumerate() {
            println!("  {}: {}", i + 1, line);
        }
        let line_count = content.lines().count();
        if line_count > 15 {
            println!("  ... ({} more lines)", line_count - 15);
        }
        println!("  ────────────────────────────────────────\n");

        let user_input: String = Input::new()
            .with_prompt(prompt)
            .allow_empty(true)
            .interact_text()
            .map_err(|e| adk_core::AdkError::Tool(format!("Interaction error: {}", e)))?;

        let trimmed = user_input.trim();

        match trimmed.to_lowercase().as_str() {
            "edit" => {
                println!("📝 Opening editor... (Save and close to submit changes)");
                let edited = Editor::new()
                    .require_save(true)
                    .edit(content)
                    .map_err(|e| adk_core::AdkError::Tool(format!("Editor error: {}", e)))?;

                let new_content = edited.unwrap_or_else(|| content.to_string());
                Ok(json!({
                    "action": "edit",
                    "content": new_content,
                    "message": "User edited content"
                }))
            }
            "pass" | "" => Ok(json!({
                "action": "pass",
                "content": content,
                "message": "User passed"
            })),
            _ => Ok(json!({
                "action": "feedback",
                "feedback": trimmed,
                "content": content,
                "message": "User provided feedback"
            })),
        }
    }
}
