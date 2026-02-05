use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};

/// Coding Stage - Generate and write code
pub struct CodingStage;

#[async_trait::async_trait]
impl Stage for CodingStage {
    fn name(&self) -> &str {
        "coding"
    }

    fn description(&self) -> &str {
        "Coding - Generate and write code"
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
                "Generating code...".to_string(),
            )
            .await;

        // TODO: Implement actual coding agent
        // This is where the actual code generation happens

        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                format!("Code written to {:?}", ctx.workspace_path),
            )
            .await;

        // Coding stage doesn't produce an artifact file, it writes to workspace
        StageResult::Success(None)
    }
}
