use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Content, LlmRequest};
use futures::StreamExt;

/// Design Stage - Create technical design using LLM
pub struct DesignStage;

#[async_trait::async_trait]
impl Stage for DesignStage {
    fn name(&self) -> &str {
        "design"
    }

    fn description(&self) -> &str {
        "Design - Create technical architecture"
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
                "Creating technical design with AI...".to_string(),
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

        // Load PRD document
        let prd_content = load_prd_document(ctx);

        // Prepare artifact path
        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/design.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return StageResult::Failed(format!("Failed to create directory: {}", e));
            }
        }

        // Generate Design using LLM
        let prompt = format!(
            r#"You are a software architect. Create a comprehensive technical design document based on the PRD.

**Iteration:** #{} - {}

{}

Please create a detailed Technical Design Document that includes:

1. **Architecture Overview** - High-level system architecture
2. **Component Design** - Key components and their responsibilities
3. **Data Model** - Database schemas, entities, relationships
4. **API Design** - REST/GraphQL endpoints, request/response formats
5. **Technology Stack** - Recommended technologies and frameworks
6. **Security Design** - Authentication, authorization, data protection
7. **Performance Considerations** - Caching, optimization strategies
8. **Integration Points** - External services and APIs
9. **Deployment Architecture** - Infrastructure requirements

Write the response in professional Markdown format with code examples where appropriate."#,
            ctx.iteration.number,
            ctx.iteration.title,
            prd_content
        );

        let content = Content::new("user").with_text(prompt);
        let request = LlmRequest::new(&config.llm.model_name, vec![content]);
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Generating design document...".to_string(),
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
            generated_text = "# Design\n\nNo content generated.".to_string();
        }

        // Write to file
        let design_content = format!(
            "# Technical Design Document\n\n**Iteration:** #{} - {}\n\n**Generated:** {}\n\n---\n\n{}",
            ctx.iteration.number,
            ctx.iteration.title,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            generated_text
        );

        if let Err(e) = std::fs::write(&artifact_path, design_content) {
            return StageResult::Failed(format!("Failed to write design file: {}", e));
        }

        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                format!("Design document generated: {}", artifact_path),
            )
            .await;

        StageResult::Success(Some(artifact_path))
    }
}

/// Load PRD document
fn load_prd_document(ctx: &PipelineContext) -> String {
    let prd_path = format!(
        "{}/.cowork-v2/iterations/{}/artifacts/prd.md",
        std::env::current_dir().unwrap().display(),
        ctx.iteration.id
    );

    match std::fs::read_to_string(&prd_path) {
        Ok(content) => format!("**PRD Document:**\n{}", content),
        Err(_) => {
            // Fallback to description if PRD not found
            format!(
                "**Requirements:**\nBased on: {}",
                ctx.iteration.description
            )
        }
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
