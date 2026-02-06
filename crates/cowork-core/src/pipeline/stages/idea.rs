use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Content, LlmRequest};

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
        self.generate_idea(ctx, interaction, None).await
    }

    async fn execute_with_feedback(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
        feedback: &str,
    ) -> StageResult {
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Regenerating idea document based on your feedback...".to_string(),
            )
            .await;
        self.generate_idea(ctx, interaction, Some(feedback)).await
    }
}

impl IdeaStage {
    async fn generate_idea(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
        feedback: Option<&str>,
    ) -> StageResult {
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

        // Build prompt
        let prompt = if let Some(feedback_text) = feedback {
            format!(
                r#"You are a product analyst. Please REVISE the idea document based on the following user feedback.

**Original Request:**
{}

**User Feedback for Revision:**
{}

Please create an IMPROVED idea document addressing the feedback. Include:
1. Problem Statement - What problem are we solving?
2. Target Users - Who will use this?
3. Core Features - What are the main features?
4. Success Criteria - How do we measure success?
5. Constraints - Any limitations or requirements?

Write the response in Markdown format."#,
                ctx.iteration.description,
                feedback_text
            )
        } else {
            format!(
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
            )
        };

        let content = Content::new("user").with_text(prompt);
        let request = LlmRequest::new(&config.llm.model_name, vec![content]);
        
        let status_msg = if feedback.is_some() {
            "Regenerating idea document with your feedback..."
        } else {
            "Generating idea document..."
        };
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                status_msg.to_string(),
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
        let header = if feedback.is_some() {
            format!("# Idea Document (Revised)\n\n**Iteration:** #{} - {}\n\n**Original Request:**\n{}\n\n**Applied Feedback:**\n{}\n\n---\n\n", 
                ctx.iteration.number,
                ctx.iteration.title,
                ctx.iteration.description,
                feedback.unwrap()
            )
        } else {
            format!("# Idea Document\n\n**Iteration:** #{} - {}\n\n**Original Request:**\n{}\n\n---\n\n", 
                ctx.iteration.number,
                ctx.iteration.title,
                ctx.iteration.description
            )
        };
        
        let idea_content = format!("{}{}", header, generated_text);

        if let Err(e) = std::fs::write(&artifact_path, idea_content) {
            return StageResult::Failed(format!("Failed to write idea file: {}", e));
        }

        let success_msg = if feedback.is_some() {
            format!("Idea document revised: {}", artifact_path)
        } else {
            format!("Idea document generated: {}", artifact_path)
        };
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                success_msg,
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
