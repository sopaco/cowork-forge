use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};

/// Plan Stage - Create implementation plan
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
                "Creating implementation plan...".to_string(),
            )
            .await;

        let artifact_path = format!(
            "{}/.cowork-v2/iterations/{}/artifacts/plan.md",
            std::env::current_dir().unwrap().display(),
            ctx.iteration.id
        );

        if let Some(parent) = std::path::Path::new(&artifact_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let content = format!("# Plan\n\nImplementation plan for iteration {}", ctx.iteration.number);
        let _ = std::fs::write(&artifact_path, content);

        StageResult::Success(Some(artifact_path))
    }
}
