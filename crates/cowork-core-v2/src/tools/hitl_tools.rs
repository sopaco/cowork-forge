// HITL (Human-in-the-Loop) tool
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use dialoguer::{Confirm, Editor};
use serde_json::{json, Value};
use std::fs;
use std::sync::Arc;

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
