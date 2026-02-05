use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};

/// Design Stage - Create technical design
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
                "Creating design...".to_string(),
            )
            .await;

        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/design.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let content = format!("# Design\n\nArchitecture for iteration {}", ctx.iteration.number);
        let _ = std::fs::write(&artifact_path, content);

        StageResult::Success(Some(artifact_path))
    }
}
