use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};

/// Idea Stage - Capture and structure user requirements
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
                "Analyzing your requirements...".to_string(),
            )
            .await;

        // TODO: Implement actual idea agent execution
        // For now, return success with mock artifact

        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/idea.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Write mock idea file
        let content = format!("# Idea\n\n{}", ctx.iteration.description);
        let _ = std::fs::write(&artifact_path, content);

        StageResult::Success(Some(artifact_path))
    }
}
