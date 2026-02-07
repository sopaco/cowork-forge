// CLI implementation of InteractiveBackend
// Uses console and stdin for terminal-based interaction with UTF-8 support

use super::{InteractiveBackend, InputOption, InputResponse, MessageLevel, ProgressInfo};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::io::{self, Write};

pub struct CliBackend {
    // event_bus removed in V2
}

impl CliBackend {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl InteractiveBackend for CliBackend {
    async fn show_message(&self, level: MessageLevel, content: String) {
        println!("{} {}", level.emoji(), content);
    }

    async fn request_input(&self, prompt: &str, options: Vec<InputOption>, initial_content: Option<String>) -> Result<InputResponse> {
        use dialoguer::Editor;

        // Helper function to read a line with UTF-8 support
        fn read_line(prompt: &str) -> Result<String> {
            print!("{} ", prompt);
            io::stdout().flush()
                .map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| anyhow::anyhow!("Failed to read input: {}", e))?;

            Ok(input.trim().to_string())
        }

        if options.is_empty() {
            // Text input with edit support
            let input = read_line(prompt)?;

            // Check if user wants to open editor
            if input.to_lowercase() == "edit" {
                println!("📝 Opening editor... (Save and close to submit changes)");
                let initial = initial_content.as_ref().map(|s| s.as_str()).unwrap_or("");
                let edited = Editor::new()
                    .require_save(true)
                    .edit(initial)
                    .map_err(|e| anyhow::anyhow!("Editor error: {}", e))?;

                match edited {
                    Some(content) => {
                        Ok(InputResponse::Text(content.to_string()))
                    }
                    None => {
                        Ok(InputResponse::Cancel)
                    }
                }
            } else if input.is_empty() {
                Ok(InputResponse::Cancel)
            } else if input.to_lowercase() == "pass" {
                Ok(InputResponse::Selection("pass".to_string()))
            } else {
                Ok(InputResponse::Text(input.to_string()))
            }
        } else {
            // Selection input with text input fallback
            println!("\n{}", prompt);
            if !options.is_empty() {
                for (i, option) in options.iter().enumerate() {
                    println!("  {}. {}{}", i + 1, option.label,
                        option.description.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default());
                }
            }
            println!("  Type 'edit' to open editor, 'pass' to continue, or provide feedback:");
            println!();

            let input = read_line("Your choice:")?;

            // Check if user entered a number
            if let Ok(num) = input.parse::<usize>() {
                if num > 0 && num <= options.len() {
                    Ok(InputResponse::Selection(options[num - 1].id.clone()))
                } else {
                    Ok(InputResponse::Cancel)
                }
            } else if input.to_lowercase() == "edit" {
                println!("📝 Opening editor... (Save and close to submit changes)");
                let initial = initial_content.as_ref().map(|s| s.as_str()).unwrap_or("");
                let edited = Editor::new()
                    .require_save(true)
                    .edit(initial)
                    .map_err(|e| anyhow::anyhow!("Editor error: {}", e))?;

                match edited {
                    Some(content) => {
                        Ok(InputResponse::Text(content.to_string()))
                    }
                    None => {
                        Ok(InputResponse::Cancel)
                    }
                }
            } else if input.to_lowercase() == "pass" || input.is_empty() {
                Ok(InputResponse::Selection("pass".to_string()))
            } else {
                // User provided feedback text
                Ok(InputResponse::Text(input.to_string()))
            }
        }
    }

    async fn show_progress(&self, task_id: String, progress: ProgressInfo) {
        let percentage = if progress.total > 0 {
            (progress.current as f64 / progress.total as f64 * 100.0) as u32
        } else {
            0
        };
        println!("[{}%] {}: {}/{} - {}", percentage, task_id, progress.current, progress.total, progress.message);
    }

    async fn submit_response(&self, _request_id: String, _response: String) -> Result<()> {
        // CLI doesn't use async HITL, responses are handled synchronously
        Ok(())
    }
}