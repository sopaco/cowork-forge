use std::sync::Arc;

use crate::interaction::InteractiveBackend;
use crate::pipeline::{PipelineContext, Stage, StageResult};
use crate::instructions::idea::IDEA_AGENT_INSTRUCTION;
use crate::pipeline::stage_executor::execute_stage_with_instruction;

/// Idea Stage - Capture and structure user requirements using Agent with Instructions + Tools
pub struct IdeaStage;

#[async_trait::async_trait]
impl Stage for IdeaStage {
    fn name(&self) -> &str {
        "idea"
    }

    fn description(&self) -> &str {
        "Idea - Capture and structure requirements using Agent with Memory and Tools"
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult {
        execute_stage_with_instruction(ctx, interaction, "idea", IDEA_AGENT_INSTRUCTION, None).await
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
                "Regenerating idea document based on your feedback...".to_string(),
            )
            .await;
        execute_stage_with_instruction(ctx, interaction, "idea", IDEA_AGENT_INSTRUCTION, Some(feedback)).await
    }
}