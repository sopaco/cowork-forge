use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::delivery::DELIVERY_AGENT_INSTRUCTION;
use crate::pipeline::stage_executor::execute_stage_with_instruction;

/// Delivery Stage - Generate delivery report using Agent with Instructions + Tools
pub struct DeliveryStage;

#[async_trait::async_trait]
impl Stage for DeliveryStage {
    fn name(&self) -> &str {
        "delivery"
    }

    fn description(&self) -> &str {
        "Delivery - Generate delivery report using Agent with Memory and Tools"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        execute_stage_with_instruction(ctx, interaction, "delivery", DELIVERY_AGENT_INSTRUCTION, None).await
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
                "Regenerating delivery report based on your feedback...".to_string(),
            )
            .await;
        execute_stage_with_instruction(ctx, interaction, "delivery", DELIVERY_AGENT_INSTRUCTION, Some(feedback)).await
    }
}