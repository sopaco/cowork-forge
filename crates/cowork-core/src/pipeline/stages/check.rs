use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Content, LlmRequest};
use futures::StreamExt;

/// Check Stage - Quality assurance using LLM
pub struct CheckStage;

#[async_trait::async_trait]
impl Stage for CheckStage {
    fn name(&self) -> &str {
        "check"
    }

    fn description(&self) -> &str {
        "Check - Quality assurance"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Running quality checks with AI...".to_string(),
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

        // Load generated code
        let code_content = load_workspace_code(ctx);
        let plan_content = load_plan_document(ctx);

        // Generate code review using LLM
        let prompt = format!(
            r#"You are a code reviewer. Review the implementation against the requirements and provide a quality report.

**Iteration:** #{} - {}

**Requirements:**
{}

**Implementation:**
{}

Please provide a comprehensive code review that includes:

1. **Overall Assessment** - Does the code meet the requirements?
2. **Code Quality** - Readability, maintainability, best practices
3. **Functionality** - Does it implement all required features?
4. **Error Handling** - Are edge cases handled properly?
5. **Security Issues** - Any potential vulnerabilities?
6. **Performance** - Any obvious performance issues?
7. **Recommendations** - Specific improvements with code examples

Provide a PASS/FAIL verdict at the end."#,
            ctx.iteration.number,
            ctx.iteration.title,
            plan_content,
            if code_content.len() > 8000 { &code_content[..8000] } else { &code_content }
        );

        let content = Content::new("user").with_text(prompt);
        let request = LlmRequest::new(&config.llm.model_name, vec![content]);
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Analyzing code quality...".to_string(),
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

        // Determine if passed
        let passed = review_text.to_uppercase().contains("PASS") || 
                     review_text.to_uppercase().contains("APPROVED");

        // Write review to file
        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/check_report.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let verdict = if passed { "✅ PASS" } else { "⚠️ NEEDS IMPROVEMENT" };
        let report = format!(
            "# Code Quality Report\n\n**Iteration:** #{} - {}\n\n**Verdict:** {}\n\n**Generated:** {}\n\n---\n\n{}",
            ctx.iteration.number,
            ctx.iteration.title,
            verdict,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            review_text
        );

        if let Err(e) = std::fs::write(&artifact_path, report) {
            return StageResult::Failed(format!("Failed to write check report: {}", e));
        }

        if passed {
            interaction
                .show_message(
                    crate::interaction::MessageLevel::Success,
                    "Quality check passed!".to_string(),
                )
                .await;
        } else {
            interaction
                .show_message(
                    crate::interaction::MessageLevel::Warning,
                    "Quality check completed with recommendations".to_string(),
                )
                .await;
        }

        // Check stage always succeeds but reports issues
        StageResult::Success(Some(artifact_path))
    }
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
