// Agents module - Agent builders using adk-rust
// 
// IMPORTANT: This file solves a CRITICAL bug where SequentialAgent stops after
// the first LoopAgent completes. 
//
// PROBLEM: When a sub-agent in LoopAgent calls exit_loop(), it terminates the
// ENTIRE SequentialAgent, not just the LoopAgent. This is adk-rust's design.
//
// SOLUTION: Remove exit_loop tools and use max_iterations=1 to let LoopAgent
// complete naturally, allowing SequentialAgent to continue to next agent.

use crate::instructions::*;
use crate::tools::*;
use adk_agent::{LlmAgentBuilder, LoopAgent};
use adk_core::{Llm, IncludeContents};
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
        .tool(Arc::new(WriteFileTool))  // For creating draft files
        .tool(Arc::new(ReviewWithFeedbackTool))  // HITL tool
        .tool(Arc::new(CreateRequirementTool))
        .tool(Arc::new(AddFeatureTool))
        .tool(Arc::new(GetRequirementsTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let prd_critic = LlmAgentBuilder::new("prd_critic")
        .instruction(PRD_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new(
        "prd_loop",
        vec![Arc::new(prd_actor), Arc::new(prd_critic)],
    );
    loop_agent = loop_agent.with_max_iterations(1);

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
        .tool(Arc::new(WriteFileTool))  // For creating draft files
        .tool(Arc::new(ReviewWithFeedbackTool))  // HITL tool
        .tool(Arc::new(CreateDesignComponentTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let design_critic = LlmAgentBuilder::new("design_critic")
        .instruction(DESIGN_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(CheckFeatureCoverageTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("design_loop", vec![Arc::new(design_actor), Arc::new(design_critic)]);
    loop_agent = loop_agent.with_max_iterations(1);

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
        .tool(Arc::new(WriteFileTool))  // For creating draft files
        .tool(Arc::new(ReviewWithFeedbackTool))  // HITL tool
        .tool(Arc::new(CreateTaskTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let plan_critic = LlmAgentBuilder::new("plan_critic")
        .instruction(PLAN_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(CheckTaskDependenciesTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("plan_loop", vec![Arc::new(plan_actor), Arc::new(plan_critic)]);
    loop_agent = loop_agent.with_max_iterations(1);

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
        .include_contents(IncludeContents::None)
        .build()?;

    let coding_critic = LlmAgentBuilder::new("coding_critic")
        .instruction(CODING_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(RunCommandTool))
        // Removed check_tests and check_lint - not applicable for pure frontend projects
        .tool(Arc::new(ProvideFeedbackTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("coding_loop", vec![Arc::new(coding_actor), Arc::new(coding_critic)]);
    // Coding needs a few iterations to implement and review tasks
    // Reduced from 20 to 5 to avoid excessive loops
    loop_agent = loop_agent.with_max_iterations(5);

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
        .tool(Arc::new(ListFilesTool))  // Added to verify project files exist
        .tool(Arc::new(ReadFileTool))   // For checking file content
        .tool(Arc::new(SaveDeliveryReportTool))
        .tool(Arc::new(SavePrdDocTool))
        .tool(Arc::new(SaveDesignDocTool))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}
