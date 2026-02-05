use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};

/// Delivery Stage - Generate delivery report
pub struct DeliveryStage;

#[async_trait::async_trait]
impl Stage for DeliveryStage {
    fn name(&self) -> &str {
        "delivery"
    }

    fn description(&self) -> &str {
        "Delivery - Generate delivery report"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Generating delivery report...".to_string(),
            )
            .await;

        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/delivery.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let content = format!(
            "# Delivery Report\n\nIteration: {}\nTitle: {}\nStatus: Completed",
            ctx.iteration.number, ctx.iteration.title
        );
        let _ = std::fs::write(&artifact_path, content);

        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                "Delivery report generated!".to_string(),
            )
            .await;

        StageResult::Success(Some(artifact_path))
    }
}
