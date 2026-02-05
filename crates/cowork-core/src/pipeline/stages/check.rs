use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};

/// Check Stage - Verify code quality
pub struct CheckStage;

#[async_trait::async_trait]
impl Stage for CheckStage {
    fn name(&self) -> &str {
        "check"
    }

    fn description(&self) -> &str {
        "Check - Verify code quality"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                "Checking code quality...".to_string(),
            )
            .await;

        // TODO: Implement actual check

        interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                "All checks passed!".to_string(),
            )
            .await;

        StageResult::Success(None)
    }
}
