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

mod hitl;
use hitl::ResilientAgent;

// ============================================================================
// IdeaAgent - Simple agent to capture initial idea
// ============================================================================

pub fn create_idea_agent(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    let agent = LlmAgentBuilder::new("idea_agent")
        .instruction(IDEA_AGENT_INSTRUCTION)
        .model(model)
        .tool(Arc::new(SaveIdeaTool))
        .tool(Arc::new(ReviewAndEditContentTool))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

pub fn create_idea_agent_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace {ITERATION_ID} placeholder in instruction
    let instruction = IDEA_AGENT_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);

    let agent = LlmAgentBuilder::new("idea_agent")
        .instruction(&instruction)
        .model(model)
        .tool(Arc::new(SaveIdeaTool))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveInsightTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// PRD Loop - Actor + Critic with LoopAgent
// ============================================================================

pub fn create_prd_loop(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    let prd_actor = LlmAgentBuilder::new("prd_actor")
        .instruction(PRD_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(LoadIdeaTool))  // Load idea document
        .tool(Arc::new(ReviewWithFeedbackContentTool))  // HITL tool (content-based)
        .tool(Arc::new(CreateRequirementTool))
        .tool(Arc::new(AddFeatureTool))
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(SavePrdDocTool))  // Save final PRD document
        .include_contents(IncludeContents::None)
        .build()?;

    let prd_critic = LlmAgentBuilder::new("prd_critic")
        .instruction(PRD_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(LoadIdeaTool))  // Load idea for context
        .tool(Arc::new(ProvideFeedbackTool))
        .include_contents(IncludeContents::None)
        .build()?;

    // Create LoopAgent with agents vector
    let mut loop_agent = LoopAgent::new(
        "prd_loop",
        vec![Arc::new(prd_actor), Arc::new(prd_critic)],
    );
    // Use max_iterations=1 to avoid SequentialAgent termination bug
    loop_agent = loop_agent.with_max_iterations(1);

    Ok(Arc::new(loop_agent))
}

pub fn create_prd_loop_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace {ITERATION_ID} placeholder in instructions
    let actor_instruction = PRD_ACTOR_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    let critic_instruction = PRD_CRITIC_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    
    let prd_actor = LlmAgentBuilder::new("prd_actor")
        .instruction(&actor_instruction)
        .model(model.clone())
        .tool(Arc::new(LoadFeedbackHistoryTool))  // For incremental update support
        .tool(Arc::new(LoadIdeaTool))  // Load idea document
        .tool(Arc::new(CreateRequirementTool))
        .tool(Arc::new(AddFeatureTool))
        .tool(Arc::new(UpdateRequirementTool))  // For incremental updates
        .tool(Arc::new(UpdateFeatureTool))  // For incremental updates
        .tool(Arc::new(DeleteRequirementTool))  // For incremental updates
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(SavePrdDocTool))  // Save final PRD document
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveInsightTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let prd_critic = LlmAgentBuilder::new("prd_critic")
        .instruction(&critic_instruction)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(LoadIdeaTool))  // Load idea for context
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveIssueTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    // Create LoopAgent with agents vector
    let mut loop_agent = LoopAgent::new(
        "prd_loop",
        vec![Arc::new(prd_actor), Arc::new(prd_critic)],
    );
    // Use max_iterations=1 to avoid SequentialAgent termination bug
    loop_agent = loop_agent.with_max_iterations(1);

    Ok(Arc::new(loop_agent))
}

// ============================================================================
// Design Loop - Actor + Critic
// ============================================================================

pub fn create_design_loop(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    let design_actor = LlmAgentBuilder::new("design_actor")
        .instruction(DESIGN_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(LoadPrdDocTool))  // Load PRD document
        .tool(Arc::new(ReviewWithFeedbackContentTool))  // HITL tool (content-based)
        .tool(Arc::new(CreateDesignComponentTool))
        .tool(Arc::new(SaveDesignDocTool))  // Save final design document
        .include_contents(IncludeContents::None)
        .build()?;

    let design_critic = LlmAgentBuilder::new("design_critic")
        .instruction(DESIGN_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(LoadDesignDocTool))  // Verify design markdown
        .tool(Arc::new(CheckFeatureCoverageTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("design_loop", vec![Arc::new(design_actor), Arc::new(design_critic)]);
    loop_agent = loop_agent.with_max_iterations(1);

    Ok(Arc::new(loop_agent))
}

pub fn create_design_loop_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace {ITERATION_ID} placeholder in instructions
    let actor_instruction = DESIGN_ACTOR_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    let critic_instruction = DESIGN_CRITIC_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    
    let design_actor = LlmAgentBuilder::new("design_actor")
        .instruction(&actor_instruction)
        .model(model.clone())
        .tool(Arc::new(LoadFeedbackHistoryTool))  // For incremental update support
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(LoadPrdDocTool))  // Load PRD document
        .tool(Arc::new(CreateDesignComponentTool))
        .tool(Arc::new(SaveDesignDocTool))  // Save final design document
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveInsightTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveIssueTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveLearningTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let design_critic = LlmAgentBuilder::new("design_critic")
        .instruction(&critic_instruction)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(LoadDesignDocTool))  // Verify design markdown
        .tool(Arc::new(CheckFeatureCoverageTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveIssueTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("design_loop", vec![Arc::new(design_actor), Arc::new(design_critic)]);
    loop_agent = loop_agent.with_max_iterations(1);

    Ok(Arc::new(loop_agent))
}

// ============================================================================
// Plan Loop - Actor + Critic
// ============================================================================

pub fn create_plan_loop(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    let plan_actor = LlmAgentBuilder::new("plan_actor")
        .instruction(PLAN_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(LoadPrdDocTool))  // Load PRD document
        .tool(Arc::new(LoadDesignDocTool))  // Load design document
        .tool(Arc::new(ReviewWithFeedbackContentTool))  // HITL tool (content-based)
        .tool(Arc::new(CreateTaskTool))
        .tool(Arc::new(SavePlanDocTool))  // Save final plan document
        .include_contents(IncludeContents::None)
        .build()?;

    let plan_critic = LlmAgentBuilder::new("plan_critic")
        .instruction(PLAN_CRITIC_INSTRUCTION)
        .model(model)
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(LoadPlanDocTool))  // Verify plan markdown
        .tool(Arc::new(CheckTaskDependenciesTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("plan_loop", vec![Arc::new(plan_actor), Arc::new(plan_critic)]);
    loop_agent = loop_agent.with_max_iterations(1);

    Ok(Arc::new(loop_agent))
}

pub fn create_plan_loop_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace {ITERATION_ID} placeholder in instructions
    let actor_instruction = PLAN_ACTOR_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    let critic_instruction = PLAN_CRITIC_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    
    let plan_actor = LlmAgentBuilder::new("plan_actor")
        .instruction(&actor_instruction)
        .model(model.clone())
        .tool(Arc::new(LoadFeedbackHistoryTool))  // For incremental update support
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(LoadPrdDocTool))  // Load PRD document
        .tool(Arc::new(LoadDesignDocTool))  // Load design document
        .tool(Arc::new(CreateTaskTool))
        .tool(Arc::new(SavePlanDocTool))  // Save final plan document
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveInsightTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveIssueTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveLearningTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let plan_critic = LlmAgentBuilder::new("plan_critic")
        .instruction(&critic_instruction)
        .model(model)
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(LoadPlanDocTool))  // Verify plan markdown
        .tool(Arc::new(CheckTaskDependenciesTool))
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveIssueTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let mut loop_agent = LoopAgent::new("plan_loop", vec![Arc::new(plan_actor), Arc::new(plan_critic)]);
    loop_agent = loop_agent.with_max_iterations(1);

    Ok(Arc::new(loop_agent))
}

// ============================================================================
// Coding Loop - Actor + Critic
// ============================================================================

pub fn create_coding_loop(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
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

    // Coding needs more iterations to implement and review tasks
    let mut loop_agent = LoopAgent::new("coding_loop", vec![Arc::new(coding_actor), Arc::new(coding_critic)]);
    loop_agent = loop_agent.with_max_iterations(5);

    Ok(Arc::new(loop_agent))
}

pub fn create_coding_loop_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace {ITERATION_ID} placeholder in instructions
    let actor_instruction = CODING_ACTOR_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    let critic_instruction = CODING_CRITIC_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    
    let coding_actor = LlmAgentBuilder::new("coding_actor")
        .instruction(&actor_instruction)
        .model(model.clone())
        .tool(Arc::new(LoadFeedbackHistoryTool))  // For incremental update support
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(UpdateTaskStatusTool))
        .tool(Arc::new(UpdateFeatureStatusTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(WriteFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(CheckTestsTool))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveInsightTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveIssueTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveLearningTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    let coding_critic = LlmAgentBuilder::new("coding_critic")
        .instruction(&critic_instruction)
        .model(model)
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(RunCommandTool))
        // Removed check_tests and check_lint - not applicable for pure frontend projects
        .tool(Arc::new(ProvideFeedbackTool))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveIssueTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    // Coding needs more iterations to implement and review tasks
    let mut loop_agent = LoopAgent::new("coding_loop", vec![Arc::new(coding_actor), Arc::new(coding_critic)]);
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

pub fn create_check_agent_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace {ITERATION_ID} placeholder in instruction
    let instruction = CHECK_AGENT_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    
    let agent = LlmAgentBuilder::new("check_agent")
        .instruction(&instruction)
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
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveInsightTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveIssueTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveLearningTool::new(iteration_id.clone())))
        .tool(Arc::new(PromoteToDecisionTool::new(iteration_id.clone())))
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
        .tool(Arc::new(ListFilesTool))  // To verify project files exist
        .tool(Arc::new(LoadIdeaTool))  // Load idea document
        .tool(Arc::new(LoadPrdDocTool))  // Load PRD document
        .tool(Arc::new(LoadDesignDocTool))  // Load design document
        .tool(Arc::new(SaveDeliveryReportTool))
        .tool(Arc::new(CopyWorkspaceToProjectTool))  // Copy files to project root
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

pub fn create_delivery_agent_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace {ITERATION_ID} placeholder in instruction
    let instruction = DELIVERY_AGENT_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);
    
    let agent = LlmAgentBuilder::new("delivery_agent")
        .instruction(&instruction)
        .model(model)
        .tool(Arc::new(GetRequirementsTool))
        .tool(Arc::new(GetDesignTool))
        .tool(Arc::new(GetPlanTool))
        .tool(Arc::new(LoadFeedbackHistoryTool))
        .tool(Arc::new(ListFilesTool))  // To verify project files exist
        .tool(Arc::new(LoadIdeaTool))  // Load idea document
        .tool(Arc::new(LoadPrdDocTool))  // Load PRD document
        .tool(Arc::new(LoadDesignDocTool))  // Load design document
        .tool(Arc::new(SaveDeliveryReportTool))
        .tool(Arc::new(CopyWorkspaceToProjectTool))  // Copy files to project root
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveInsightTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveLearningTool::new(iteration_id.clone())))
        .tool(Arc::new(PromoteToPatternTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}