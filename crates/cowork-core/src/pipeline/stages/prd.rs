use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::prd::PRD_ACTOR_INSTRUCTION;
use crate::pipeline::stage_executor::execute_stage_with_instruction;

/// PRD Stage - Generate Product Requirements Document using Agent with Instructions + Tools
pub struct PrdStage;

#[async_trait::async_trait]
impl Stage for PrdStage {
    fn name(&self) -> &str {
        "prd"
    }

    fn description(&self) -> &str {
        "PRD - Generate product requirements using Agent with Memory and Tools"
    }

    fn needs_confirmation(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        execute_stage_with_instruction(ctx, interaction, "prd", PRD_ACTOR_INSTRUCTION, None).await
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
                "Regenerating PRD based on your feedback...".to_string(),
            )
            .await;
        execute_stage_with_instruction(ctx, interaction, "prd", PRD_ACTOR_INSTRUCTION, Some(feedback)).await
    }
}