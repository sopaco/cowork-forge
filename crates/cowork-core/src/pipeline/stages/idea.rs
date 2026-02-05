use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Llm, Content, LlmRequest};

/// Idea Stage - Capture and structure user requirements using LLM
pub struct IdeaStage;

#[async_trait::async_trait]
impl Stage for IdeaStage {
    fn name(&self) -> &str {
        "idea"
    }

    fn description(&self) -> &str {
        "Idea - Capture and structure requirements"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Analyzing your requirements with AI...".to_string(),
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

        // Prepare artifact path
        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/idea.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return StageResult::Failed(format!("Failed to create directory: {}", e));
            }
        }

        // Generate idea using LLM
        let prompt = format!(
            r#"You are a product analyst. Based on the following user request, create a structured idea document.

User Request: {}

Please analyze this request and create a comprehensive idea document that includes:
1. Problem Statement - What problem are we solving?
2. Target Users - Who will use this?
3. Core Features - What are the main features?
4. Success Criteria - How do we measure success?
5. Constraints - Any limitations or requirements?

Write the response in Markdown format."#,
            ctx.iteration.description
        );

        let content = Content::new("user").with_text(prompt);
        let request = LlmRequest::new(&config.llm.model_name, vec![content]);
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Generating idea document...".to_string(),
            )
            .await;

        // Call LLM
        let mut stream = match llm.generate_content(request, false).await {
            Ok(resp) => resp,
            Err(e) => {
                return StageResult::Failed(format!("LLM generation failed: {}", e));
            }
        };

        // Collect response from stream
        use futures::StreamExt;
        let mut generated_text = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Some(content) = response.content {
                        for part in content.parts {
                            if let Some(text) = part.text() {
                                generated_text.push_str(text);
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        if generated_text.is_empty() {
            generated_text = "# Idea\n\nNo content generated.".to_string();
        }

        // Write to file
        let idea_content = format!(
            "# Idea Document\n\n**Iteration:** #{} - {}\n\n**Original Request:**\n{}\n\n---\n\n{}",
            ctx.iteration.number,
            ctx.iteration.title,
            ctx.iteration.description,
            generated_text
        );

        if let Err(e) = std::fs::write(&artifact_path, idea_content) {
            return StageResult::Failed(format!("Failed to write idea file: {}", e));
        }

        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                format!("Idea document generated: {}", artifact_path),
            )
            .await;

        StageResult::Success(Some(artifact_path))
    }
}

/// Load config from file or environment
fn load_config() -> anyhow::Result<ModelConfig> {
    use std::path::Path;
    
    // Try loading from config.toml
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
