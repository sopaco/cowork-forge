use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::check::CHECK_AGENT_INSTRUCTION;
use crate::pipeline::stage_executor::execute_stage_with_instruction;

/// Check Stage - Quality assurance and validation using Agent with Instructions + Tools
pub struct CheckStage;

#[async_trait::async_trait]
impl Stage for CheckStage {
    fn name(&self) -> &str {
        "check"
    }

    fn description(&self) -> &str {
        "Check - Quality assurance and validation using Agent with Memory and Tools"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        execute_stage_with_instruction(ctx, interaction, "check", CHECK_AGENT_INSTRUCTION, None).await
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
                "Regenerating check report based on your feedback...".to_string(),
            )
            .await;
        execute_stage_with_instruction(ctx, interaction, "check", CHECK_AGENT_INSTRUCTION, Some(feedback)).await
    }
}