// Agents module - Agent builders using adk-rust

use crate::instructions::*;
use crate::tools::*;
use adk_agent::{LlmAgentBuilder, LoopAgent};
use adk_core::{Llm, IncludeContents};
use adk_tool::ExitLoopTool;
use anyhow::Result;
use std::sync::Arc;

// ============================================================================
// IdeaAgent - Simple agent to capture initial idea
// ============================================================================

pub fn create_idea_agent(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    let agent = LlmAgentBuilder::new("idea_agent")
        .instruction(IDEA_AGENT_INSTRUCTION)
        .model(model)
        .tool(Arc::new(WriteFileTool))
        .tool(Arc::new(ReviewAndEditFileTool))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// PRD Loop - Actor + Critic with LoopAgent
// ============================================================================

pub fn create_prd_loop(model: Arc<dyn Llm>) -> Result<Arc<LoopAgent>> {
    let prd_actor = LlmAgentBuilder::new("prd_actor")
        .instruction(PRD_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(CreateRequirementTool))
        .tool(Arc::new(AddFeatureTool))
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(ExitLoopTool::new()))
        .include_contents(IncludeContents::None)
        .build()?;

    let prd_critic = LlmAgentBuilder::new("prd_critic")
        .instruction(PRD_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(ExitLoopTool::new()))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new(
        "prd_loop",
        vec![Arc::new(prd_actor), Arc::new(prd_critic)],
    );
    loop_agent = loop_agent.with_max_iterations(10);

    Ok(Arc::new(loop_agent))
}

// ============================================================================
// Design Loop - Actor + Critic
// ============================================================================

pub fn create_design_loop(model: Arc<dyn Llm>) -> Result<Arc<LoopAgent>> {
    let design_actor = LlmAgentBuilder::new("design_actor")
        .instruction(DESIGN_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(CreateDesignComponentTool))
        .tool(Arc::new(ExitLoopTool::new()))
        .include_contents(IncludeContents::None)
        .build()?;

    let design_critic = LlmAgentBuilder::new("design_critic")
        .instruction(DESIGN_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(CheckFeatureCoverageTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(ExitLoopTool::new()))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("design_loop", vec![Arc::new(design_actor), Arc::new(design_critic)]);
    loop_agent = loop_agent.with_max_iterations(10);

    Ok(Arc::new(loop_agent))
}

// ============================================================================
// Plan Loop - Actor + Critic
// ============================================================================

pub fn create_plan_loop(model: Arc<dyn Llm>) -> Result<Arc<LoopAgent>> {
    let plan_actor = LlmAgentBuilder::new("plan_actor")
        .instruction(PLAN_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(CreateTaskTool))
        .tool(Arc::new(ExitLoopTool::new()))
        .include_contents(IncludeContents::None)
        .build()?;

    let plan_critic = LlmAgentBuilder::new("plan_critic")
        .instruction(PLAN_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(CheckTaskDependenciesTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(ExitLoopTool::new()))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("plan_loop", vec![Arc::new(plan_actor), Arc::new(plan_critic)]);
    loop_agent = loop_agent.with_max_iterations(10);

    Ok(Arc::new(loop_agent))
}

// ============================================================================
// Coding Loop - Actor + Critic
// ============================================================================

pub fn create_coding_loop(model: Arc<dyn Llm>) -> Result<Arc<LoopAgent>> {
    let coding_actor = LlmAgentBuilder::new("coding_actor")
        .instruction(CODING_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(UpdateTaskStatusTool))
        .tool(Arc::new(UpdateFeatureStatusTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(WriteFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(CheckTestsTool))
        .tool(Arc::new(ExitLoopTool::new()))
        .include_contents(IncludeContents::None)
        .build()?;

    let coding_critic = LlmAgentBuilder::new("coding_critic")
        .instruction(CODING_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(CheckTestsTool))
        .tool(Arc::new(CheckLintTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(ExitLoopTool::new()))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("coding_loop", vec![Arc::new(coding_actor), Arc::new(coding_critic)]);
    loop_agent = loop_agent.with_max_iterations(20);

    Ok(Arc::new(loop_agent))
}

// ============================================================================
// Check Agent - Quality assurance
// ============================================================================

pub fn create_check_agent(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    let agent = LlmAgentBuilder::new("check_agent")
        .instruction(CHECK_AGENT_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(CheckDataFormatTool))
        .tool(Arc::new(CheckFeatureCoverageTool))
        .tool(Arc::new(CheckTaskDependenciesTool))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(CheckTestsTool))
        .tool(Arc::new(CheckLintTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(GotoStageTool))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// Delivery Agent - Final report generation
// ============================================================================

pub fn create_delivery_agent(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    let agent = LlmAgentBuilder::new("delivery_agent")
        .instruction(DELIVERY_AGENT_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(LoadFeedbackHistoryTool))
        .tool(Arc::new(SaveDeliveryReportTool))
        .tool(Arc::new(SavePrdDocTool))
        .tool(Arc::new(SaveDesignDocTool))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}
