// HITL (Human-in-the-Loop) tools
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use dialoguer::{Confirm, Editor, Input};
use serde_json::{json, Value};
use std::fs;
use std::sync::Arc;

/// ReviewAndEditFileTool - Original HITL tool (used in Idea stage)
pub struct ReviewAndEditFileTool;

#[async_trait]
impl Tool for ReviewAndEditFileTool {
    fn name(&self) -> &str {
        "review_and_edit_file"
    }

    fn description(&self) -> &str {
        "Let the user review and optionally edit a file using their default editor. \
         User will be prompted: 'Do you want to edit this file? (y/n)'. \
         If 'y', opens the file in an editor. If 'n', continues without changes."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to review and edit"
                },
                "title": {
                    "type": "string",
                    "description": "Title/description for the review prompt"
                }
            },
            "required": ["file_path", "title"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let file_path = args["file_path"].as_str().unwrap();
        let title = args["title"].as_str().unwrap();

        // Read current file content
        let content = fs::read_to_string(file_path)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read file {}: {}", file_path, e)))?;

        // Show preview
        println!("\n📝 {} - {}", title, file_path);
        println!("  ────────────────────────────────────────");
        let line_count = content.lines().count();
        for (i, line) in content.lines().take(10).enumerate() {
            println!("  {}: {}", i + 1, line);
        }
        if line_count > 10 {
            println!("  ... ({} more lines)", line_count - 10);
        }
        println!("  ────────────────────────────────────────\n");

        // Ask user if they want to edit
        let should_edit = Confirm::new()
            .with_prompt("Do you want to edit this file? (y/n)")
            .default(false)
            .interact()
            .map_err(|e| adk_core::AdkError::Tool(format!("Interaction error: {}", e)))?;

        if !should_edit {
            return Ok(json!({
                "status": "no_changes",
                "message": "User chose not to edit the file"
            }));
        }

        // Open editor
        println!("📝 Opening editor... (Save and close to submit changes)");
        let edited = Editor::new()
            .require_save(true)
            .edit(&content)
            .map_err(|e| adk_core::AdkError::Tool(format!("Editor error: {}", e)))?;

        match edited {
            Some(new_content) if new_content.trim() != content.trim() => {
                // Save changes
                fs::write(file_path, &new_content)
                    .map_err(|e| adk_core::AdkError::Tool(format!("Failed to write file: {}", e)))?;

                println!("✅ File updated successfully");
                Ok(json!({
                    "status": "edited",
                    "message": "File was edited and saved",
                    "changes_made": true
                }))
            }
            _ => {
                println!("ℹ️  No changes made");
                Ok(json!({
                    "status": "no_changes",
                    "message": "File was not modified"
                }))
            }
        }
    }
}

/// ReviewWithFeedbackTool - Enhanced HITL tool with three modes:
/// 1. User types "edit" → Opens editor
/// 2. User types "pass" → Continues without changes
/// 3. User types other text → Returns as feedback for agent to process
pub struct ReviewWithFeedbackTool;

#[async_trait]
impl Tool for ReviewWithFeedbackTool {
    fn name(&self) -> &str {
        "review_with_feedback"
    }

    fn description(&self) -> &str {
        "Show user a file preview and ask for feedback. User can:\n\
         - Type 'edit' to open the file in an editor\n\
         - Type 'pass' to continue without changes\n\
         - Type any other text to provide feedback/suggestions (agent will revise based on feedback)"
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to review"
                },
                "title": {
                    "type": "string",
                    "description": "Title/description for the review prompt"
                },
                "prompt": {
                    "type": "string",
                    "description": "Custom prompt to show the user (e.g., '请审查需求大纲')"
                }
            },
            "required": ["path", "title"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let file_path = args["path"].as_str().unwrap();
        let title = args["title"].as_str().unwrap();
        let default_prompt = "输入 'edit' 编辑，'pass' 继续，或直接输入修改建议";
        let prompt = args["prompt"].as_str().unwrap_or(default_prompt);

        // Read current file content
        let content = fs::read_to_string(file_path)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read file {}: {}", file_path, e)))?;

        // Show preview
        println!("\n📝 {} - {}", title, file_path);
        println!("  ────────────────────────────────────────");
        let line_count = content.lines().count();
        for (i, line) in content.lines().take(15).enumerate() {
            println!("  {}: {}", i + 1, line);
        }
        if line_count > 15 {
            println!("  ... ({} more lines)", line_count - 15);
        }
        println!("  ────────────────────────────────────────\n");

        // Ask user for input
        let user_input: String = Input::new()
            .with_prompt(prompt)
            .allow_empty(true)
            .interact_text()
            .map_err(|e| adk_core::AdkError::Tool(format!("Interaction error: {}", e)))?;

        let user_input = user_input.trim();

        // Handle different input modes
        match user_input.to_lowercase().as_str() {
            "edit" => {
                // Mode 1: Open editor
                println!("📝 Opening editor... (Save and close to submit changes)");
                let edited = Editor::new()
                    .require_save(true)
                    .edit(&content)
                    .map_err(|e| adk_core::AdkError::Tool(format!("Editor error: {}", e)))?;

                match edited {
                    Some(new_content) if new_content.trim() != content.trim() => {
                        fs::write(file_path, &new_content)
                            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to write file: {}", e)))?;

                        println!("✅ File updated successfully");
                        Ok(json!({
                            "action": "edit",
                            "status": "edited",
                            "message": "User edited the file in editor",
                            "changes_made": true
                        }))
                    }
                    _ => {
                        println!("ℹ️  No changes made in editor");
                        Ok(json!({
                            "action": "edit",
                            "status": "no_changes",
                            "message": "User opened editor but made no changes"
                        }))
                    }
                }
            }
            "pass" | "" => {
                // Mode 2: Pass/Continue
                println!("➡️  Continuing without changes...");
                Ok(json!({
                    "action": "pass",
                    "status": "passed",
                    "message": "User chose to continue without changes"
                }))
            }
            _ => {
                // Mode 3: Feedback text
                println!("💬 Feedback received: {}", user_input);
                println!("🔄 Agent will revise based on your feedback...");
                Ok(json!({
                    "action": "feedback",
                    "status": "feedback_provided",
                    "feedback": user_input,
                    "message": format!("User provided feedback: {}", user_input)
                }))
            }
        }
    }
}
