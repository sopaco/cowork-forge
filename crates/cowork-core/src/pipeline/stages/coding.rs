use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Content, LlmRequest};
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

                // Validate code syntax
                let issues = validate_code_syntax(file_path, &clean_code);
                if !issues.is_empty() {
                    interaction
                        .show_message(
                            crate::interaction::MessageLevel::Warning,
                            format!("Code validation warnings for {}: {}", file_path, issues.join(", ")),
                        )
                        .await;
                }

                // Create parent directory
                if let Some(parent) = full_path.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        interaction
                            .show_message(
                                crate::interaction::MessageLevel::Error,
                                format!("Failed to create directory for {}: {}", file_path, e),
                            )
                            .await;
                        continue;
                    }
                }

                if let Err(e) = std::fs::write(&full_path, &clean_code) {
                    interaction
                        .show_message(
                            crate::interaction::MessageLevel::Error,
                            format!("Failed to write {}: {}", file_path, e),
                        )
                        .await;
                } else {
                    interaction
                        .show_message(
                            crate::interaction::MessageLevel::Success,
                            format!("Generated {} ({} bytes)", file_path, clean_code.len()),
                        )
                        .await;
                }
            }
        }

        // Write a summary file of generated code
        let summary_path = workspace_path.join(".generated_files.txt");
        let summary = generate_code_summary(workspace_path);
        let _ = std::fs::write(&summary_path, summary);

        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                format!("Code generation complete in: {}", workspace_path.display()),
            )
            .await;

        // Return workspace path as the artifact so it can be tracked
        let workspace_str = workspace_path.to_string_lossy().to_string();
        StageResult::Success(Some(workspace_str))
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

/// Parse file list from LLM response (improved to handle multiple formats)
fn parse_file_list(text: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut in_files_section = false;

    for line in text.lines() {
        let trimmed = line.trim();
        
        // Detect file section start
        if trimmed.starts_with("FILES:") 
            || trimmed.starts_with("File Structure:")
            || trimmed.starts_with("## Files")
            || trimmed.starts_with("### Files")
            || trimmed.to_lowercase().starts_with("file list:")
            || trimmed.to_lowercase().starts_with("project structure:") {
            in_files_section = true;
            continue;
        }

        // Detect section end (next header or empty line after files)
        if in_files_section && trimmed.starts_with("#") && !trimmed.starts_with("##") {
            in_files_section = false;
            continue;
        }

        if in_files_section && (trimmed.starts_with("-") || trimmed.starts_with("*")) {
            // Parse line like: - path/to/file.rs: description
            // or: - `path/to/file.rs`: description
            let content = trimmed.trim_start_matches("-").trim_start_matches("*").trim();
            
            // Remove backticks if present
            let content = content.trim_matches('`');
            
            if let Some(pos) = content.find(':') {
                let path = content[..pos].trim().trim_matches('`').to_string();
                let desc = content[pos + 1..].trim().to_string();
                if is_valid_file_path(&path) {
                    files.push((path, desc));
                }
            } else if let Some(pos) = content.find('|') {
                let path = content[..pos].trim().trim_matches('`').to_string();
                let desc = content[pos + 1..].trim().to_string();
                if is_valid_file_path(&path) {
                    files.push((path, desc));
                }
            } else if is_valid_file_path(content) {
                // Just a file path without description
                files.push((content.to_string(), "Generated file".to_string()));
            }
        }

        // Also try to detect file paths in code blocks or plain text
        if !in_files_section && trimmed.contains('.') {
            // Try to extract file paths like src/main.rs or path/to/file.js
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            for word in words {
                let clean = word.trim_matches(|c| c == '`' || c == '"' || c == '\'' || c == '(' || c == ')' || c == ',' || c == '.');
                if is_valid_file_path(clean) && !files.iter().any(|(p, _)| p == clean) {
                    files.push((clean.to_string(), "Generated file".to_string()));
                }
            }
        }
    }

    // If no files found in structured format, try regex-like extraction
    if files.is_empty() {
        files = extract_files_from_text(text);
    }

    files
}

/// Check if a string looks like a valid file path
fn is_valid_file_path(path: &str) -> bool {
    // Must have a file extension
    let has_extension = path.contains('.') && !path.ends_with('.');
    
    // Must not contain invalid characters
    let invalid_chars = ['<', '>', '|', '?', '*', '\0'];
    let no_invalid = !invalid_chars.iter().any(|&c| path.contains(c));
    
    // Should look like a path (contains / or is a simple filename)
    let looks_like_path = path.contains('/') || !path.contains(' ');
    
    has_extension && no_invalid && looks_like_path && path.len() > 2
}

/// Extract file paths from unstructured text
fn extract_files_from_text(text: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let common_patterns = [
        (r"(?:^|\s)([\w\-/]+\.\w+)\s*[:\-]", "code"),  // path.ext: description
        (r"`([^`]+\.\w+)`", "markdown"),                 // `path.ext`
    ];

    for line in text.lines() {
        for (_pattern, _p_type) in &common_patterns {
            // Simple pattern matching without regex crate
            if let Some(start) = line.find("/") {
                if let Some(end) = line[start..].find(' ') {
                    let candidate = &line[start..start+end];
                    if is_valid_file_path(candidate) {
                        let desc = line[end..].trim_start_matches(|c| c == ':' || c == '-').trim();
                        files.push((candidate.to_string(), desc.to_string()));
                    }
                }
            }
        }
    }

    files
}

/// Extract code from markdown code blocks (improved)
fn extract_code(text: &str) -> String {
    let mut result = String::new();
    let mut in_code_block = false;
    let mut code_block_content = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("```") {
            if in_code_block {
                // End of code block
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&code_block_content);
                code_block_content.clear();
                in_code_block = false;
            } else {
                // Start of code block - skip the language identifier
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            code_block_content.push_str(line);
            code_block_content.push('\n');
        }
    }

    // If still in code block at end, include the content
    if in_code_block && !code_block_content.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&code_block_content);
    }

    if result.is_empty() {
        // No code blocks found, return entire text (but clean it up)
        text.trim()
            .trim_start_matches(|c| c == '`')
            .trim_end_matches(|c| c == '`')
            .to_string()
    } else {
        result.trim_end().to_string()
    }
}

/// Validate generated code has basic syntax correctness
fn validate_code_syntax(file_path: &str, code: &str) -> Vec<String> {
    let mut issues = Vec::new();
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    // Check for unclosed braces/parentheses (basic check)
    let open_braces = code.matches('{').count();
    let close_braces = code.matches('}').count();
    if open_braces != close_braces {
        issues.push(format!("Unbalanced braces: {} open, {} close", open_braces, close_braces));
    }

    let open_parens = code.matches('(').count();
    let close_parens = code.matches(')').count();
    if open_parens != close_parens {
        issues.push(format!("Unbalanced parentheses: {} open, {} close", open_parens, close_parens));
    }

    // Language-specific checks
    match ext {
        "rs" => {
            // Rust: check for fn main or lib structure
            if !code.contains("fn ") && !code.contains("struct ") && !code.contains("enum ") {
                issues.push("No function or type definitions found".to_string());
            }
        }
        "js" | "ts" | "jsx" | "tsx" => {
            // JavaScript/TypeScript: check for basic structure
            if !code.contains("function") && !code.contains("const") && !code.contains("export") {
                issues.push("No JavaScript/TypeScript constructs found".to_string());
            }
        }
        "py" => {
            // Python: check for def or class
            if !code.contains("def ") && !code.contains("class ") {
                issues.push("No function or class definitions found".to_string());
            }
        }
        _ => {}
    }

    // Check for placeholder comments that indicate incomplete generation
    let placeholder_patterns = [
        "TODO:", "FIXME:", "// ...", "// ...", "/* ...", "# ...",
        "// implementation", "// your code here", "// add your logic",
    ];
    for pattern in &placeholder_patterns {
        if code.to_lowercase().contains(&pattern.to_lowercase()) {
            issues.push(format!("Contains placeholder: {}", pattern));
        }
    }

    issues
}

/// Generate a summary of generated code files
fn generate_code_summary(workspace_path: &std::path::Path) -> String {
    let mut summary = String::from("# Generated Code Files\n\n");
    summary.push_str(&format!("Generated at: {}\n\n", chrono::Utc::now().to_rfc3339()));

    let mut total_files = 0;
    let mut total_lines = 0;

    if let Ok(entries) = std::fs::read_dir(workspace_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some() {
                let file_name = path.file_name().unwrap().to_string_lossy();
                if file_name.starts_with('.') {
                    continue; // Skip hidden files
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let lines = content.lines().count();
                    total_lines += lines;
                    total_files += 1;
                    summary.push_str(&format!("- {} ({} lines)\n", file_name, lines));
                }
            }
        }
    }

    summary.push_str(&format!("\n## Summary\n"));
    summary.push_str(&format!("- Total files: {}\n", total_files));
    summary.push_str(&format!("- Total lines: {}\n", total_lines));

    summary
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
