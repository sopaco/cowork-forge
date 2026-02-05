use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};

/// PRD Stage - Generate Product Requirements Document
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
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Generating PRD...".to_string(),
            )
            .await;

        // TODO: Implement actual PRD agent execution

        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/prd.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let content = format!("# PRD\n\nBased on: {}", ctx.iteration.description);
        let _ = std::fs::write(&artifact_path, content);

        StageResult::Success(Some(artifact_path))
    }
}
