use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Content, LlmRequest};
use futures::StreamExt;

/// PRD Stage - Generate Product Requirements Document using LLM
pub struct PrdStage;

#[async_trait::async_trait]
impl Stage for PrdStage {
    fn name(&self) -> &str {
        "prd"
    }

    fn description(&self) -> &str {
        "PRD - Generate product requirements"
    }

    fn needs_confirmation(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        self.generate_prd(ctx, interaction, None).await
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
                "Regenerating PRD based on your feedback...".to_string(),
            )
            .await;
        self.generate_prd(ctx, interaction, Some(feedback)).await
    }
}

impl PrdStage {
    async fn generate_prd(
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

        // Load idea document if exists
        let idea_content = load_idea_document(ctx);

        // Prepare artifact path
        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/prd.md",
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
                r#"You are a product manager. Please REVISE the Product Requirements Document (PRD) based on the following user feedback.

**Iteration:** #{} - {}

**Original Request:**
{}

{}

**User Feedback for Revision:**
{}

Please create an IMPROVED PRD addressing the feedback. Include:

1. **Overview** - Brief summary of the product/feature
2. **Goals** - What we want to achieve
3. **User Stories** - As a [user], I want [feature] so that [benefit]
4. **Functional Requirements** - Detailed list of features and behaviors
5. **Non-Functional Requirements** - Performance, security, usability requirements
6. **UI/UX Requirements** - Interface guidelines and user experience expectations
7. **Data Requirements** - Data models, storage needs
8. **API Requirements** - External/internal API specifications
9. **Open Questions** - Issues that need clarification

Write the response in professional Markdown format suitable for a technical team."#,
                ctx.iteration.number,
                ctx.iteration.title,
                ctx.iteration.description,
                idea_content,
                feedback_text
            )
        } else {
            format!(
                r#"You are a product manager. Create a comprehensive Product Requirements Document (PRD) based on the following information.

**Iteration:** #{} - {}

**Original Request:**
{}

{}

Please create a detailed PRD that includes:

1. **Overview** - Brief summary of the product/feature
2. **Goals** - What we want to achieve
3. **User Stories** - As a [user], I want [feature] so that [benefit]
4. **Functional Requirements** - Detailed list of features and behaviors
5. **Non-Functional Requirements** - Performance, security, usability requirements
6. **UI/UX Requirements** - Interface guidelines and user experience expectations
7. **Data Requirements** - Data models, storage needs
8. **API Requirements** - External/internal API specifications
9. **Open Questions** - Issues that need clarification

Write the response in professional Markdown format suitable for a technical team."#,
                ctx.iteration.number,
                ctx.iteration.title,
                ctx.iteration.description,
                idea_content
            )
        };

        let content = Content::new("user").with_text(prompt);
        let request = LlmRequest::new(&config.llm.model_name, vec![content]);
        
        let status_msg = if feedback.is_some() {
            "Regenerating PRD with your feedback..."
        } else {
            "Generating PRD..."
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
            generated_text = "# PRD\n\nNo content generated.".to_string();
        }

        // Write to file
        let header = if feedback.is_some() {
            format!("# Product Requirements Document (PRD) - Revised\n\n**Iteration:** #{} - {}\n\n**Generated:** {}\n\n**Applied Feedback:**\n{}\n\n---\n\n",
                ctx.iteration.number,
                ctx.iteration.title,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                feedback.unwrap()
            )
        } else {
            format!("# Product Requirements Document (PRD)\n\n**Iteration:** #{} - {}\n\n**Generated:** {}\n\n---\n\n",
                ctx.iteration.number,
                ctx.iteration.title,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            )
        };
        
        let prd_content = format!("{}{}", header, generated_text);

        if let Err(e) = std::fs::write(&artifact_path, prd_content) {
            return StageResult::Failed(format!("Failed to write PRD file: {}", e));
        }

        let success_msg = if feedback.is_some() {
            format!("PRD revised: {}", artifact_path)
        } else {
            format!("PRD generated: {}", artifact_path)
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

/// Load idea document if exists
fn load_idea_document(ctx: &PipelineContext) -> String {
    let idea_path = format!(
        "{}/.cowork-v2/iterations/{}/artifacts/idea.md",
        std::env::current_dir().unwrap().display(),
        ctx.iteration.id
    );

    if let Ok(content) = std::fs::read_to_string(&idea_path) {
        format!("\n**Idea Document:**\n{}", content)
    } else {
        String::new()
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
