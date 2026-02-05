use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Content, LlmRequest};
use futures::StreamExt;

/// Delivery Stage - Generate final delivery report using LLM
pub struct DeliveryStage;

#[async_trait::async_trait]
impl Stage for DeliveryStage {
    fn name(&self) -> &str {
        "delivery"
    }

    fn description(&self) -> &str {
        "Delivery - Generate completion report"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Generating delivery report with AI...".to_string(),
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

        // Gather all artifacts
        let idea_content = load_artifact(ctx, "idea.md");
        let prd_content = load_artifact(ctx, "prd.md");
        let design_content = load_artifact(ctx, "design.md");
        let plan_content = load_artifact(ctx, "plan.md");
        let check_content = load_artifact(ctx, "check_report.md");
        let code_summary = summarize_code(ctx);

        // Generate delivery report using LLM
        let prompt = format!(
            r#"You are a project manager. Create a comprehensive delivery report summarizing the completed iteration.

**Iteration:** #{} - {}

**Original Request:**
{}

**Idea:**
{}

**Requirements:**
{}

**Design:**
{}

**Implementation:**
{}

**Quality Check:**
{}

**Generated Files:**
{}

Please create a comprehensive Delivery Report that includes:

1. **Executive Summary** - What was accomplished in this iteration
2. **Requirements Fulfilled** - List of completed requirements
3. **Technical Implementation** - Summary of the solution
4. **Key Features** - What functionality was implemented
5. **Known Limitations** - Any known issues or incomplete features
6. **Next Steps** - Recommendations for future iterations
7. **Files Delivered** - List of all generated files with descriptions

Format as a professional project delivery document."#,
            ctx.iteration.number,
            ctx.iteration.title,
            ctx.iteration.description,
            idea_content,
            prd_content,
            design_content,
            plan_content,
            check_content,
            code_summary
        );

        let content = Content::new("user").with_text(prompt);
        let request = LlmRequest::new(&config.llm.model_name, vec![content]);
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Generating delivery report...".to_string(),
            )
            .await;

        // Call LLM
        let mut stream = match llm.generate_content(request, false).await {
            Ok(resp) => resp,
            Err(e) => {
                return StageResult::Failed(format!("LLM report generation failed: {}", e));
            }
        };

        // Collect response from stream
        let mut report_text = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Some(content) = response.content {
                        for part in content.parts {
                            if let Some(text) = part.text() {
                                report_text.push_str(text);
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        if report_text.is_empty() {
            report_text = "# Delivery Report\n\nNo content generated.".to_string();
        }

        // Write report to file
        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/delivery_report.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return StageResult::Failed(format!("Failed to create directory: {}", e));
            }
        }

        let report = format!(
            "# Delivery Report\n\n**Iteration:** #{} - {}\n\n**Status:** ✅ Completed\n\n**Generated:** {}\n\n---\n\n{}",
            ctx.iteration.number,
            ctx.iteration.title,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            report_text
        );

        if let Err(e) = std::fs::write(&artifact_path, report) {
            return StageResult::Failed(format!("Failed to write delivery report: {}", e));
        }

        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                format!("Iteration #{} '{}' completed successfully!", ctx.iteration.number, ctx.iteration.title),
            )
            .await;

        // Deliver code to project directory
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Delivering code to project directory...".to_string(),
            )
            .await;

        match deliver_code_to_project(&ctx.workspace_path).await {
            Ok(delivered_files) => {
                interaction
                    .show_message(
                        crate::interaction::MessageLevel::Success,
                        format!("Delivered {} files to project directory", delivered_files),
                    )
                    .await;
            }
            Err(e) => {
                interaction
                    .show_message(
                        crate::interaction::MessageLevel::Warning,
                        format!("Failed to deliver code to project directory: {}", e),
                    )
                    .await;
            }
        }

        StageResult::Success(Some(artifact_path))
    }
}

/// Load artifact content
fn load_artifact(ctx: &PipelineContext, filename: &str) -> String {
    let path = format!(
        "{}/.cowork-v2/iterations/{}/artifacts/{}",
        std::env::current_dir().unwrap().display(),
        ctx.iteration.id,
        filename
    );

    match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => format!("_{} not found_", filename),
    }
}

/// Summarize generated code
fn summarize_code(ctx: &PipelineContext) -> String {
    let mut summary = String::new();
    
    if let Ok(entries) = std::fs::read_dir(&ctx.workspace_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                summary.push_str(&format!("- {} ({} bytes)\n", filename, size));
            }
        }
    }

    if summary.is_empty() {
        summary = "_No code files generated_".to_string();
    }

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

/// Deliver code from workspace to project root directory
async fn deliver_code_to_project(workspace_path: &std::path::Path) -> anyhow::Result<usize> {
    let project_root = std::env::current_dir()?;
    let mut delivered_count = 0;
    
    // Directories and files to exclude from delivery
    let exclude_dirs: &[&str] = &[
        ".cowork-v2",
        ".cowork",
        ".git",
        "node_modules",
        "target",
        "dist",
        "build",
        ".idea",
        ".vscode",
    ];
    
    let exclude_files: &[&str] = &[
        ".DS_Store",
        "Thumbs.db",
        ".gitignore",
    ];
    
    if !workspace_path.exists() {
        return Ok(0);
    }
    
    deliver_recursive(
        workspace_path,
        &project_root,
        exclude_dirs,
        exclude_files,
        &mut delivered_count,
    ).await?;
    
    Ok(delivered_count)
}

async fn deliver_recursive(
    src: &std::path::Path,
    dst: &std::path::Path,
    exclude_dirs: &[&str],
    exclude_files: &[&str],
    count: &mut usize,
) -> anyhow::Result<()> {
    use tokio::fs;
    
    let mut entries = fs::read_dir(src).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        
        // Skip excluded directories and files
        if src_path.is_dir() && exclude_dirs.contains(&file_name_str.as_ref()) {
            continue;
        }
        
        if src_path.is_file() && exclude_files.contains(&file_name_str.as_ref()) {
            continue;
        }
        
        let dst_path = dst.join(&file_name);
        
        if src_path.is_dir() {
            // Create directory if it doesn't exist
            if !dst_path.exists() {
                fs::create_dir_all(&dst_path).await?;
            }
            // Recursively process subdirectory
            Box::pin(deliver_recursive(
                &src_path,
                &dst_path,
                exclude_dirs,
                exclude_files,
                count,
            )).await?;
        } else {
            // Copy file
            fs::copy(&src_path, &dst_path).await?;
            *count += 1;
        }
    }
    
    Ok(())
}
