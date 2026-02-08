use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::coding::CODING_ACTOR_INSTRUCTION;
use crate::pipeline::stage_executor::execute_stage_with_instruction;

/// Coding Stage - Generate code implementation using Agent with Instructions + Tools
pub struct CodingStage;

#[async_trait::async_trait]
impl Stage for CodingStage {
    fn name(&self) -> &str {
        "coding"
    }

    fn description(&self) -> &str {
        "Coding - Generate code implementation using Agent with Memory and Tools"
    }

    fn needs_confirmation(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, None).await
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
                "Regenerating code based on your feedback...".to_string(),
            )
            .await;
        execute_stage_with_instruction(ctx, interaction, "coding", CODING_ACTOR_INSTRUCTION, Some(feedback)).await
    }
}