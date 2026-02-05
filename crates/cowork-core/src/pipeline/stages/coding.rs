use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Llm, Content, LlmRequest};
use futures::StreamExt;

/// Coding Stage - Generate and write actual code using LLM
pub struct CodingStage;

#[async_trait::async_trait]
impl Stage for CodingStage {
    fn name(&self) -> &str {
        "coding"
    }

    fn description(&self) -> &str {
        "Coding - Generate and write code"
    }

    fn needs_confirmation(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Generating code with AI...".to_string(),
            )
            .await;

        // Load LLM config
        let config = match load_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                return StageResult::Failed(format!("Failed to load config: {}", e));
            }
        };

        // Create LLM client
        let llm = match create_llm_client(&config.llm) {
            Ok(client) => client,
            Err(e) => {
                return StageResult::Failed(format!("Failed to create LLM client: {}", e));
            }
        };

        // Load implementation plan
        let plan_content = load_plan_document(ctx);

        // Ensure workspace directory exists
        let workspace_path = &ctx.workspace_path;
        if let Err(e) = std::fs::create_dir_all(workspace_path) {
            return StageResult::Failed(format!("Failed to create workspace directory: {}", e));
        }

        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                format!("Generating code in: {}", workspace_path.display()),
            )
            .await;

        // Generate code structure first
        let structure_prompt = format!(
            r#"You are a senior software engineer. Based on the implementation plan, generate a complete file structure for the project.

**Iteration:** #{} - {}

{}

Please provide:
1. List of all files to create with their full paths
2. Brief description of what each file contains

Format your response as:
```
FILES:
- path/to/file1.ext: description
- path/to/file2.ext: description
...
```"#,
            ctx.iteration.number,
            ctx.iteration.title,
            plan_content
        );

        let structure_content = Content::new("user").with_text(structure_prompt);
        let structure_request = LlmRequest::new(&config.llm.model_name, vec![structure_content]);
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Analyzing requirements and generating file structure...".to_string(),
            )
            .await;

        let mut structure_stream = match llm.generate_content(structure_request, false).await {
            Ok(resp) => resp,
            Err(e) => {
                return StageResult::Failed(format!("LLM structure generation failed: {}", e));
            }
        };

        let mut structure_text = String::new();
        while let Some(result) = structure_stream.next().await {
            match result {
                Ok(response) => {
                    if let Some(content) = response.content {
                        for part in content.parts {
                            if let Some(text) = part.text() {
                                structure_text.push_str(text);
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        // Parse file list from response
        let files_to_generate = parse_file_list(&structure_text);

        if files_to_generate.is_empty() {
            // Fallback: generate a single main file
            let main_file_path = workspace_path.join("main.rs");
            
            interaction
                .show_message(
                    crate::interaction::MessageLevel::Info,
                    "Generating main implementation file...".to_string(),
                )
                .await;

            let code_prompt = format!(
                r#"You are a senior software engineer. Generate a complete, working implementation based on the plan.

**Iteration:** #{} - {}

{}

Generate a complete, working code implementation. The code should be:
- Fully functional and compilable
- Well-commented
- Following best practices
- Include error handling where appropriate

Provide the complete code for the main implementation."#,
                ctx.iteration.number,
                ctx.iteration.title,
                plan_content
            );

            let code_content = Content::new("user").with_text(code_prompt);
            let code_request = LlmRequest::new(&config.llm.model_name, vec![code_content]);
            
            let mut code_stream = match llm.generate_content(code_request, false).await {
                Ok(resp) => resp,
                Err(e) => {
                    return StageResult::Failed(format!("LLM code generation failed: {}", e));
                }
            };

            let mut generated_code = String::new();
            while let Some(result) = code_stream.next().await {
                match result {
                    Ok(response) => {
                        if let Some(content) = response.content {
                            for part in content.parts {
                                if let Some(text) = part.text() {
                                    generated_code.push_str(text);
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            if generated_code.is_empty() {
                generated_code = "// Code generation failed".to_string();
            }

            // Extract code from markdown code blocks if present
            let clean_code = extract_code(&generated_code);

            if let Err(e) = std::fs::write(&main_file_path, clean_code) {
                return StageResult::Failed(format!("Failed to write main file: {}", e));
            }

            interaction
                .show_message(
                    crate::interaction::MessageLevel::Success,
                    format!("Generated: {}", main_file_path.display()),
                )
                .await;
        } else {
            // Generate each file
            for (file_path, description) in &files_to_generate {
                let full_path = workspace_path.join(file_path);
                
                // Ensure parent directory exists
                if let Some(parent) = full_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                interaction
                    .show_message(
                        crate::interaction::MessageLevel::Info,
                        format!("Generating: {}", file_path),
                    )
                    .await;

                let file_prompt = format!(
                    r#"You are a senior software engineer. Generate the complete code for this specific file.

**File:** {}
**Description:** {}

**Project Context:**
{}

Generate complete, working code for this file only. The code should be:
- Fully functional
- Well-commented
- Following best practices
- Properly integrated with the rest of the project

Provide ONLY the code for this file, no explanations."#,
                    file_path,
                    description,
                    plan_content
                );

                let file_content = Content::new("user").with_text(file_prompt);
                let file_request = LlmRequest::new(&config.llm.model_name, vec![file_content]);
                
                let mut file_stream = match llm.generate_content(file_request, false).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        interaction
                            .show_message(
                                crate::interaction::MessageLevel::Error,
                                format!("Failed to generate {}: {}", file_path, e),
                            )
                            .await;
                        continue;
                    }
                };

                let mut generated_code = String::new();
                while let Some(result) = file_stream.next().await {
                    match result {
                        Ok(response) => {
                            if let Some(content) = response.content {
                                for part in content.parts {
                                    if let Some(text) = part.text() {
                                        generated_code.push_str(text);
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }

                if generated_code.is_empty() {
                    generated_code = format!("// Failed to generate {}", file_path);
                }

                let clean_code = extract_code(&generated_code);

                if let Err(e) = std::fs::write(&full_path, clean_code) {
                    interaction
                        .show_message(
                            crate::interaction::MessageLevel::Error,
                            format!("Failed to write {}: {}", file_path, e),
                        )
                        .await;
                }
            }
        }

        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                format!("Code generation complete in: {}", workspace_path.display()),
            )
            .await;

        // Coding stage doesn't produce an artifact file, it writes to workspace
        StageResult::Success(None)
    }
}

/// Load plan document
fn load_plan_document(ctx: &PipelineContext) -> String {
    let plan_path = format!(
        "{}/.cowork-v2/iterations/{}/artifacts/plan.md",
        std::env::current_dir().unwrap().display(),
        ctx.iteration.id
    );

    let design_path = format!(
        "{}/.cowork-v2/iterations/{}/artifacts/design.md",
        std::env::current_dir().unwrap().display(),
        ctx.iteration.id
    );

    let mut result = String::new();

    if let Ok(content) = std::fs::read_to_string(&plan_path) {
        result.push_str(&format!("**Implementation Plan:**\n{}\n\n", content));
    }

    if let Ok(content) = std::fs::read_to_string(&design_path) {
        result.push_str(&format!("**Design Document:**\n{}\n\n", content));
    }

    if result.is_empty() {
        result = format!(
            "**Requirements:**\nIteration #{} - {}\nDescription: {}",
            ctx.iteration.number,
            ctx.iteration.title,
            ctx.iteration.description
        );
    }

    result
}

/// Parse file list from LLM response
fn parse_file_list(text: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut in_files_section = false;

    for line in text.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("FILES:") || trimmed.starts_with("File Structure:") {
            in_files_section = true;
            continue;
        }

        if in_files_section && trimmed.starts_with("-") {
            // Parse line like: - path/to/file.rs: description
            let content = trimmed.trim_start_matches("-").trim();
            if let Some(pos) = content.find(':') {
                let path = content[..pos].trim().to_string();
                let desc = content[pos + 1..].trim().to_string();
                files.push((path, desc));
            } else if let Some(pos) = content.find('|') {
                let path = content[..pos].trim().to_string();
                let desc = content[pos + 1..].trim().to_string();
                files.push((path, desc));
            } else {
                files.push((content.to_string(), "Generated file".to_string()));
            }
        }
    }

    files
}

/// Extract code from markdown code blocks
fn extract_code(text: &str) -> String {
    // Look for code blocks
    let mut result = String::new();
    let mut in_code_block = false;

    for line in text.lines() {
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            result.push_str(line);
            result.push('\n');
        }
    }

    if result.is_empty() {
        // No code blocks found, return entire text
        text.to_string()
    } else {
        result
    }
}

/// Load config from file or environment
fn load_config() -> anyhow::Result<ModelConfig> {
    use std::path::Path;
    
    if Path::new("config.toml").exists() {
        ModelConfig::from_file("config.toml")
    } else if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(&exe_path);
        let config_path = exe_dir.join("config.toml");
        if config_path.exists() {
            ModelConfig::from_file(config_path.to_str().unwrap())
        } else {
            ModelConfig::from_env()
        }
    } else {
        ModelConfig::from_env()
    }
}
