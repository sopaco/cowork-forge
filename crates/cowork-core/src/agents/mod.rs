// Agents module - Agent builders using adk-rust
//
// IMPORTANT: This file uses StageExecutor instead of SequentialAgent to allow
// LoopAgents to use ExitLoopTool without affecting other stages.
//
// SOLUTION: StageExecutor isolates each stage's escalate flag, so when a
// sub-agent in LoopAgent calls exit_loop(), it only terminates that specific
// LoopAgent, not the entire workflow.

use crate::instructions::*;
use crate::tools::*;
use crate::interaction::InteractiveBackend;
use adk_agent::{LlmAgentBuilder, LoopAgent};
use adk_core::{Llm, IncludeContents};
use adk_tool::ExitLoopTool;
use anyhow::Result;
use std::sync::Arc;

mod hitl;
pub use hitl::ResilientAgent;

pub mod iterative_assistant;
pub use iterative_assistant::*;

// ============================================================================
// V1 Legacy Agents (deprecated - not used in V2)
// ============================================================================
/*
// IdeaAgent - Simple agent to capture initial idea
pub fn create_idea_agent(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {
    create_idea_agent_with_interaction(model, session_id, None)
}

pub fn create_idea_agent_with_interaction(
    model: Arc<dyn Llm>, 
    session_id: &str,
    interaction: Option<Arc<dyn InteractiveBackend>>
) -> Result<Arc<dyn adk_core::Agent>> {
    // Set global interaction backend if provided
    if let Some(backend) = interaction {
        set_interaction_backend(backend);
    }

    let agent = LlmAgentBuilder::new("idea_agent")
        .instruction(IDEA_AGENT_INSTRUCTION)
        .model(model)
        .tool(Arc::new(SaveIdeaTool::new(session_id.to_string())))
        .tool(Arc::new(LoadIdeaTool::new(session_id.to_string())))
        .tool(Arc::new(ReviewAndEditContentTool))
        .tool(Arc::new(QueryMemoryTool::new(session_id.to_string())))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// PRD Loop - Actor + Critic with LoopAgent
// ============================================================================

pub fn create_prd_loop(model: Arc<dyn Llm>, session_id: &str, interaction: Arc<dyn InteractiveBackend>) -> Result<Arc<dyn adk_core::Agent>> {
    // Set global interaction backend
    set_interaction_backend(interaction.clone());

    let session = session_id.to_string();
    
    let prd_actor = LlmAgentBuilder::new("prd_actor")
        .instruction(PRD_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(LoadIdeaTool::new(session.clone())))
        .tool(Arc::new(ReviewWithFeedbackContentTool))
        .tool(Arc::new(CreateRequirementTool::new(session.clone())))
        .tool(Arc::new(AddFeatureTool::new(session.clone())))
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(SavePrdDocTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveInsightTool::new(session.clone())))
        .tool(Arc::new(SaveIssueTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let prd_critic = LlmAgentBuilder::new("prd_critic")
        .instruction(PRD_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(ProvideFeedbackTool::new(session.clone()))) // Write feedback to file when checks fail
.tool(Arc::new(RequestHumanReviewTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveLearningTool::new(session.clone())))
        .tool(Arc::new(PromoteToDecisionTool::new(session.clone())))
        .tool(Arc::new(PromoteToPatternTool::new(session.clone())))
        .include_contents(IncludeContents::Default)
        .build()?;

    let mut loop_agent = LoopAgent::new("prd_loop", vec![Arc::new(prd_actor), Arc::new(prd_critic)]);
    loop_agent = loop_agent.with_max_iterations(3); // Loop will complete naturally after 3 iterations

    Ok(Arc::new(ResilientAgent::new(Arc::new(loop_agent), interaction)))
}

// ============================================================================
// Design Loop - Actor + Critic
// ============================================================================

pub fn create_design_loop(model: Arc<dyn Llm>, session_id: &str, interaction: Arc<dyn InteractiveBackend>) -> Result<Arc<dyn adk_core::Agent>> {
    // Set global interaction backend
    set_interaction_backend(interaction.clone());

    let session = session_id.to_string();
    
    let design_actor = LlmAgentBuilder::new("design_actor")
        .instruction(DESIGN_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(ReviewWithFeedbackContentTool))
        .tool(Arc::new(CreateDesignComponentTool::new(session.clone())))
        .tool(Arc::new(SaveDesignDocTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveInsightTool::new(session.clone())))
        .tool(Arc::new(SaveIssueTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let design_critic = LlmAgentBuilder::new("design_critic")
        .instruction(DESIGN_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(CheckFeatureCoverageTool::new(session.clone())))
        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))
        .tool(Arc::new(ExitLoopTool::new()))
        .tool(Arc::new(RequestHumanReviewTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveLearningTool::new(session.clone())))
        .tool(Arc::new(PromoteToDecisionTool::new(session.clone())))
        .tool(Arc::new(PromoteToPatternTool::new(session.clone())))
        .include_contents(IncludeContents::Default)
        .build()?;

    let mut loop_agent = LoopAgent::new("design_loop", vec![Arc::new(design_actor), Arc::new(design_critic)]);
    loop_agent = loop_agent.with_max_iterations(3);

    Ok(Arc::new(ResilientAgent::new(Arc::new(loop_agent), interaction)))
}

// ============================================================================
// Plan Loop - Actor + Critic
// ============================================================================

pub fn create_plan_loop(model: Arc<dyn Llm>, session_id: &str, interaction: Arc<dyn InteractiveBackend>) -> Result<Arc<dyn adk_core::Agent>> {
    // Set global interaction backend
    set_interaction_backend(interaction.clone());

    let session = session_id.to_string();
    
    let plan_actor = LlmAgentBuilder::new("plan_actor")
        .instruction(PLAN_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(ReviewWithFeedbackContentTool))
        .tool(Arc::new(CreateTaskTool::new(session.clone())))
        .tool(Arc::new(UpdateTaskTool::new(session.clone())))
        .tool(Arc::new(DeleteTaskTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveInsightTool::new(session.clone())))
        .tool(Arc::new(SaveIssueTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let plan_critic = LlmAgentBuilder::new("plan_critic")
        .instruction(PLAN_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(CheckTaskDependenciesTool::new(session.clone())))
        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))
        .tool(Arc::new(ExitLoopTool::new()))
        .tool(Arc::new(RequestHumanReviewTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveLearningTool::new(session.clone())))
        .tool(Arc::new(PromoteToDecisionTool::new(session.clone())))
        .tool(Arc::new(PromoteToPatternTool::new(session.clone())))
        .include_contents(IncludeContents::Default)
        .build()?;

    let mut loop_agent = LoopAgent::new("plan_loop", vec![Arc::new(plan_actor), Arc::new(plan_critic)]);
    loop_agent = loop_agent.with_max_iterations(3);

    Ok(Arc::new(ResilientAgent::new(Arc::new(loop_agent), interaction)))
}

// ============================================================================
// Coding Loop - Actor + Critic
// ============================================================================

pub fn create_coding_loop(model: Arc<dyn Llm>, session_id: &str, interaction: Arc<dyn InteractiveBackend>) -> Result<Arc<dyn adk_core::Agent>> {
    // Set global interaction backend
    set_interaction_backend(interaction.clone());

    let session = session_id.to_string();
    
    let coding_actor = LlmAgentBuilder::new("coding_actor")
        .instruction(CODING_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(ReviewWithFeedbackContentTool))
        .tool(Arc::new(UpdateTaskStatusTool::new(session.clone())))
        .tool(Arc::new(UpdateFeatureStatusTool::new(session.clone())))
        .tool(Arc::new(CreateTaskTool::new(session.clone())))
        .tool(Arc::new(UpdateTaskTool::new(session.clone())))
        .tool(Arc::new(DeleteTaskTool::new(session.clone())))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(WriteFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(CheckTestsTool))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveInsightTool::new(session.clone())))
        .tool(Arc::new(SaveIssueTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let coding_critic = LlmAgentBuilder::new("coding_critic")
        .instruction(CODING_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))
        .tool(Arc::new(ExitLoopTool::new()))
        .tool(Arc::new(RequestReplanningTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveLearningTool::new(session.clone())))
        .tool(Arc::new(PromoteToDecisionTool::new(session.clone())))
        .tool(Arc::new(PromoteToPatternTool::new(session.clone())))
        .include_contents(IncludeContents::Default)
        .build()?;

    let mut loop_agent = LoopAgent::new("coding_loop", vec![Arc::new(coding_actor), Arc::new(coding_critic)]);
    loop_agent = loop_agent.with_max_iterations(5);

    Ok(Arc::new(ResilientAgent::new(Arc::new(loop_agent), interaction)))
}

// ============================================================================
// Check Agent - Quality assurance
// ============================================================================

pub fn create_check_agent(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {
    let session = session_id.to_string();
    
    let agent = LlmAgentBuilder::new("check_agent")
        .instruction(CHECK_AGENT_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(CheckDataFormatTool::new(session.clone())))
        .tool(Arc::new(CheckFeatureCoverageTool::new(session.clone())))
        .tool(Arc::new(CheckTaskDependenciesTool::new(session.clone())))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(CheckTestsTool))
        .tool(Arc::new(CheckLintTool))
        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))
        .tool(Arc::new(GotoStageTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveLearningTool::new(session.clone())))
        .tool(Arc::new(PromoteToDecisionTool::new(session.clone())))
        .tool(Arc::new(PromoteToPatternTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// Delivery Agent - Final report generation
// ============================================================================

pub fn create_delivery_agent(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {
    let session = session_id.to_string();
    
    let agent = LlmAgentBuilder::new("delivery_agent")
        .instruction(DELIVERY_AGENT_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(LoadFeedbackHistoryTool::new(session.clone())))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(SaveDeliveryReportTool::new(session.clone())))
        .tool(Arc::new(SavePrdDocTool::new(session.clone())))
        .tool(Arc::new(SaveDesignDocTool::new(session.clone())))
        .tool(Arc::new(QueryMemoryTool::new(session.clone())))
        .tool(Arc::new(SaveLearningTool::new(session.clone())))
        .tool(Arc::new(PromoteToDecisionTool::new(session.clone())))
        .tool(Arc::new(PromoteToPatternTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}
*/
