use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::llm::{ModelConfig, create_llm_client};
use adk_core::{Content, LlmRequest};
use futures::StreamExt;

/// Plan Stage - Create implementation plan using LLM
pub struct PlanStage;

#[async_trait::async_trait]
impl Stage for PlanStage {
    fn name(&self) -> &str {
        "plan"
    }

    fn description(&self) -> &str {
        "Plan - Create implementation tasks"
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
                "Creating implementation plan with AI...".to_string(),
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

        // Load design document
        let design_content = load_design_document(ctx);

        // Detect project type and get tech stack constraints
        let project_type = crate::tech_stack::detect_project_type(&ctx.iteration.description);
        let tech_stack_instructions = crate::tech_stack::get_tech_stack_instructions(project_type.clone());

        // Prepare artifact path
        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/plan.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return StageResult::Failed(format!("Failed to create directory: {}", e));
            }
        }

        // Generate Implementation Plan using LLM with tech stack constraints
        let prompt = format!(
            r#"You are a technical lead. Create a detailed implementation plan based on the design document.

**Iteration:** #{} - {}
**Project Type:** {}

{}

{}

Please create a comprehensive Implementation Plan that includes:

1. **Overview** - Summary of what needs to be built
2. **Task Breakdown** - Detailed list of development tasks
   - Task ID, Description, Estimated effort
   - Dependencies between tasks
3. **File Structure** - Recommended project structure with file paths
   - MUST follow the project structure specified in the tech stack constraints
4. **Implementation Order** - Sequence of tasks with priorities
5. **Key Algorithms/Logic** - Pseudocode or logic description for complex parts
6. **Testing Strategy** - How to verify each component
7. **Risk Mitigation** - Potential issues and solutions
8. **Definition of Done** - Criteria for completing this iteration

**CRITICAL:** 
- You MUST strictly follow the technology stack constraints specified above
- The file structure MUST match the requirements for the project type
- Do not deviate from the required languages, frameworks, or tools

Format the plan as a Markdown document with clear sections and checklists."#,
            ctx.iteration.number,
            ctx.iteration.title,
            project_type,
            design_content,
            tech_stack_instructions
        );

        let content = Content::new("user").with_text(prompt);
        let request = LlmRequest::new(&config.llm.model_name, vec![content]);
        
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Generating implementation plan...".to_string(),
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
            generated_text = "# Implementation Plan\n\nNo content generated.".to_string();
        }

        // Write to file
        let plan_content = format!(
            "# Implementation Plan\n\n**Iteration:** #{} - {}\n\n**Generated:** {}\n\n---\n\n{}",
            ctx.iteration.number,
            ctx.iteration.title,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            generated_text
        );

        if let Err(e) = std::fs::write(&artifact_path, plan_content) {
            return StageResult::Failed(format!("Failed to write plan file: {}", e));
        }

        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                format!("Implementation plan generated: {}", artifact_path),
            )
            .await;

        StageResult::Success(Some(artifact_path))
    }
}

/// Load design document
fn load_design_document(ctx: &PipelineContext) -> String {
    let design_path = format!(
        "{}/.cowork-v2/iterations/{}/artifacts/design.md",
        std::env::current_dir().unwrap().display(),
        ctx.iteration.id
    );

    let prd_path = format!(
        "{}/.cowork-v2/iterations/{}/artifacts/prd.md",
        std::env::current_dir().unwrap().display(),
        ctx.iteration.id
    );

    let mut result = String::new();

    if let Ok(content) = std::fs::read_to_string(&design_path) {
        result.push_str(&format!("**Design Document:**\n{}\n\n", content));
    }

    if let Ok(content) = std::fs::read_to_string(&prd_path) {
        result.push_str(&format!("**PRD Document:**\n{}\n\n", content));
    }

    if result.is_empty() {
        result = format!("**Requirements:**\n{}", ctx.iteration.description);
    }

    result
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
