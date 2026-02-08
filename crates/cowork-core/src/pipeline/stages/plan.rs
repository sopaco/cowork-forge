use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::plan::PLAN_ACTOR_INSTRUCTION;
use crate::pipeline::stage_executor::execute_stage_with_instruction;

/// Plan Stage - Generate implementation plan using Agent with Instructions + Tools
pub struct PlanStage;

#[async_trait::async_trait]
impl Stage for PlanStage {
    fn name(&self) -> &str {
        "plan"
    }

    fn description(&self) -> &str {
        "Plan - Generate implementation plan using Agent with Memory and Tools"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        execute_stage_with_instruction(ctx, interaction, "plan", PLAN_ACTOR_INSTRUCTION, None).await
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
                "Regenerating plan based on your feedback...".to_string(),
            )
            .await;
        execute_stage_with_instruction(ctx, interaction, "plan", PLAN_ACTOR_INSTRUCTION, Some(feedback)).await
    }
}