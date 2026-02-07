use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Content, LlmRequest};
use futures::StreamExt;

/// Check Stage - Quality assurance and project completeness validation using LLM
pub struct CheckStage;

#[async_trait::async_trait]
impl Stage for CheckStage {
    fn name(&self) -> &str {
        "check"
    }

    fn description(&self) -> &str {
        "Check - Quality assurance and completeness validation"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Running quality and completeness checks...".to_string(),
            )
            .await;

        // First, check project completeness
        let completeness_issues = check_project_completeness(ctx, &interaction).await;
        
        // Then, load LLM config for code review
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

        // Load generated code
        let code_content = load_workspace_code(ctx);
        let plan_content = load_plan_document(ctx);

        // Generate code review using LLM
        let prompt = format!(
            r#"You are a code reviewer and QA specialist. Review the implementation against the requirements and provide a comprehensive quality report.

**Iteration:** #{} - {}

**Requirements:**
{}

**Generated Code:**
{}

**Project Completeness Issues Found:**
{}

Please provide a comprehensive review that includes:

1. **Project Completeness Assessment**
   - Are all required files present?
   - Are configuration files valid?
   - Is the project structure correct?
   - Are dependencies properly defined?

2. **Code Quality**
   - Readability, maintainability, best practices
   - Proper error handling
   - Security vulnerabilities
   - Performance issues

3. **Functionality**
   - Does it implement all required features?
   - Are there missing implementations?

4. **Actionable Recommendations**
   - Specific issues that MUST be fixed
   - Improvements with code examples

**CRITICAL:** If the project has ANY of these issues, you MUST FAIL the review:
- Missing package.json (for Node.js projects)
- Missing Cargo.toml (for Rust projects)
- Missing index.html (for web projects)
- Empty or placeholder files
- Code that cannot run/compile

Provide a PASS/FAIL verdict at the end. Be STRICT - FAIL if the project is not actually runnable."#,
            ctx.iteration.number,
            ctx.iteration.title,
            plan_content,
            if code_content.len() > 8000 { &code_content[..8000] } else { &code_content },
            if completeness_issues.is_empty() {
                "No completeness issues found".to_string()
            } else {
                completeness_issues.join("\n- ")
            }
        );

        let content = Content::new("user").with_text(prompt);
        let request = LlmRequest::new(&config.llm.model_name, vec![content]);
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Analyzing code quality and project completeness...".to_string(),
            )
            .await;

        // Call LLM
        let mut stream = match llm.generate_content(request, false).await {
            Ok(resp) => resp,
            Err(e) => {
                return StageResult::Failed(format!("LLM review failed: {}", e));
            }
        };

        // Collect response from stream
        let mut review_text = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Some(content) = response.content {
                        for part in content.parts {
                            if let Some(text) = part.text() {
                                review_text.push_str(text);
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        if review_text.is_empty() {
            review_text = "# Code Review\n\nNo review generated.".to_string();
        }

        // Determine if passed - be more strict
        let passed = (review_text.to_uppercase().contains("PASS") || 
                     review_text.to_uppercase().contains("APPROVED"))
                    && !review_text.to_uppercase().contains("FAIL")
                    && completeness_issues.is_empty();

        // Write review to file
        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/check_report.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let verdict = if passed { "✅ PASS" } else { "❌ FAIL" };
        let report = format!(
            "# Code Quality and Completeness Report\n\n**Iteration:** #{} - {}\n\n**Verdict:** {}\n\n**Generated:** {}\n\n---\n\n## Project Completeness Issues\n{}\n\n## Code Review\n{}",
            ctx.iteration.number,
            ctx.iteration.title,
            verdict,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            if completeness_issues.is_empty() {
                "None found".to_string()
            } else {
                format!("\n- {}", completeness_issues.join("\n- "))
            },
            review_text
        );

        if let Err(e) = std::fs::write(&artifact_path, report) {
            return StageResult::Failed(format!("Failed to write check report: {}", e));
        }

        if passed {
            interaction
                .show_message(
                    crate::interaction::MessageLevel::Success,
                    "All quality and completeness checks passed!".to_string(),
                )
                .await;
        } else {
            let issue_count = completeness_issues.len();
            interaction
                .show_message(
                    crate::interaction::MessageLevel::Error,
                    format!("Quality check FAILED! {} critical issue(s) found. See report for details.", issue_count),
                )
                .await;
        }

        // Return failure if not passed to allow iteration to be fixed
        if passed {
            StageResult::Success(Some(artifact_path))
        } else {
            StageResult::Failed("Quality checks failed. Project is incomplete or has critical issues.".to_string())
        }
    }
}

/// Check project completeness
async fn check_project_completeness(ctx: &PipelineContext, interaction: &Arc<dyn InteractiveBackend>) -> Vec<String> {
    let mut issues = Vec::new();
    let workspace = &ctx.workspace_path;
    
    interaction
        .show_message(
            crate::interaction::MessageLevel::Info,
            "Checking project completeness...".to_string(),
        )
        .await;
    
    // Check if workspace exists
    if !workspace.exists() {
        issues.push("Workspace directory does not exist".to_string());
        return issues;
    }
    
    // Detect project type and validate against tech stack constraints
    let project_type = crate::tech_stack::detect_project_type(&ctx.iteration.description);
    
    // Check for files
    let entries = match std::fs::read_dir(workspace) {
        Ok(e) => e,
        Err(e) => {
            issues.push(format!("Cannot read workspace: {}", e));
            return issues;
        }
    };
    
    let files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    
    if files.is_empty() {
        issues.push("No files generated in workspace".to_string());
        return issues;
    }
    
    // Validate project structure against tech stack requirements
    let file_paths: Vec<String> = files.iter()
        .map(|f| f.path().file_name().unwrap_or_default().to_string_lossy().to_string())
        .collect();
    
    let structure_issues = crate::tech_stack::validate_project_structure(&project_type, &file_paths);
    issues.extend(structure_issues);
    
    // Check for Node.js/React projects
    let has_package_json = files.iter().any(|f| {
        f.path().file_name() == Some(std::ffi::OsStr::new("package.json"))
    });
    
    if has_package_json {
        let package_json_path = workspace.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&package_json_path) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check for dependencies
                let has_deps = pkg.get("dependencies").is_some();
                let has_dev_deps = pkg.get("devDependencies").is_some();
                
                if !has_deps && !has_dev_deps {
                    issues.push("package.json has no dependencies defined".to_string());
                }
                
                // Check for scripts
                if pkg.get("scripts").and_then(|s| s.as_object()).map_or(true, |s| s.is_empty()) {
                    issues.push("package.json has no scripts defined (need at least 'dev' or 'start')".to_string());
                }
                
                // Check for forbidden frameworks (Vue, Angular, Svelte, Next.js, Nuxt.js)
                if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_object()) {
                    for dep_name in deps.keys() {
                        let dep_lower = dep_name.to_lowercase();
                        if dep_lower.contains("vue") || dep_lower.contains("angular") 
                            || dep_lower.contains("svelte") || dep_lower.contains("next") 
                            || dep_lower.contains("nuxt") {
                            issues.push(format!(
                                "Forbidden framework detected in package.json: {}. Web projects must use Vanilla HTML/JS/CSS or React only.",
                                dep_name
                            ));
                        }
                    }
                }
                
                if let Some(dev_deps) = pkg.get("devDependencies").and_then(|d| d.as_object()) {
                    for dep_name in dev_deps.keys() {
                        let dep_lower = dep_name.to_lowercase();
                        if dep_lower.contains("vue") || dep_lower.contains("angular") 
                            || dep_lower.contains("svelte") || dep_lower.contains("next") 
                            || dep_lower.contains("nuxt") {
                            issues.push(format!(
                                "Forbidden framework detected in devDependencies: {}. Web projects must use Vanilla HTML/JS/CSS or React only.",
                                dep_name
                            ));
                        }
                    }
                }
            } else {
                issues.push("package.json is invalid JSON".to_string());
            }
        }
    }
    
    // Check for web projects
    let has_index_html = files.iter().any(|f| {
        f.path().file_name() == Some(std::ffi::OsStr::new("index.html"))
    });
    
    if project_type == crate::tech_stack::ProjectType::Web {
        if !has_index_html && !has_package_json {
            // Check for any entry point
            let has_entry = files.iter().any(|f| {
                let file_path = f.path();
                let name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with("main") || name.starts_with("index") || name == "app.rs"
            });
            
            if !has_entry {
                issues.push("Web project must have index.html or package.json".to_string());
            }
        }
    }
    
    // Check for placeholder content
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file.path()) {
            let content_lower = content.to_lowercase();
            let placeholder_patterns = [
                "your code here",
                "implementation here",
                "todo:",
                "fixme:",
                "// ...",
                "// add your",
                "placeholder",
                "not implemented",
            ];
            
            for pattern in &placeholder_patterns {
                if content_lower.contains(pattern) {
                    issues.push(format!(
                        "File {} contains placeholder text: '{}'",
                        file.path().file_name().unwrap_or_default().to_string_lossy(),
                        pattern
                    ));
                    break;
                }
            }
            
            // Check for empty or near-empty files
            if content.trim().len() < 10 {
                issues.push(format!(
                    "File {} is empty or nearly empty",
                    file.path().file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }
    }
    
    issues
}

/// Load code from workspace
fn load_workspace_code(ctx: &PipelineContext) -> String {
    let mut all_code = String::new();
    
    if let Ok(entries) = std::fs::read_dir(&ctx.workspace_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    all_code.push_str(&format!("\n\n// File: {}\n{}", 
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        content));
                }
            }
        }
    }

    if all_code.is_empty() {
        all_code = "// No code files found in workspace".to_string();
    }

    all_code
}

/// Load plan document
fn load_plan_document(ctx: &PipelineContext) -> String {
    let plan_path = format!(
        "{}/.cowork-v2/iterations/{}/artifacts/plan.md",
        std::env::current_dir().unwrap().display(),
        ctx.iteration.id
    );

    match std::fs::read_to_string(&plan_path) {
        Ok(content) => content,
        Err(_) => ctx.iteration.description.clone(),
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
