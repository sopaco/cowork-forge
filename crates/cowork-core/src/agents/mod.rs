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
use crate::IterationStore;
use adk_agent::{LlmAgentBuilder, LoopAgent};
use adk_core::{Llm, IncludeContents};
use anyhow::Result;
use std::sync::Arc;

// External Coding Agent (ACP-based)
pub mod external_coding_agent;
pub use external_coding_agent::{ExternalCodingAgent, StreamingTask};

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

    let save_idea_tool = Arc::new(SaveIdeaTool);
    eprintln!("[DEBUG] Created SaveIdeaTool");

    let agent = LlmAgentBuilder::new("idea_agent")
        .instruction(&instruction)
        .model(model)
        .tool(save_idea_tool)
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(SaveInsightTool::new(iteration_id.clone())))
        .include_contents(IncludeContents::None)
        .build()?;

    eprintln!("[DEBUG] Created idea_agent successfully");
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

// ============================================================================
// Summary Agent - Generates summaries of iteration documents
// ============================================================================

pub fn create_summary_agent(model: Arc<dyn Llm>, iteration_id: String, iteration_number: u32) -> Result<Arc<dyn adk_core::Agent>> {
    let instruction = SUMMARY_AGENT_INSTRUCTION
        .replace("{iteration_id}", &iteration_id)
        .replace("{iteration_number}", &iteration_number.to_string());

    let agent = LlmAgentBuilder::new(SUMMARY_AGENT_NAME)
        .instruction(&instruction)
        .model(model)
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// Knowledge Generation Agent - Extracts project-level knowledge from iterations
// ============================================================================

pub fn create_knowledge_generation_agent(
    model: Arc<dyn Llm>,
    iteration_id: String,
    iteration_number: u32,
    base_iteration_id: Option<String>
) -> Result<Arc<dyn adk_core::Agent>> {
    let instruction = KNOWLEDGE_GEN_AGENT_INSTRUCTION
        .replace("{iteration_id}", &iteration_id)
        .replace("{iteration_number}", &iteration_number.to_string());

    let read_file_with_limit = Arc::new(ReadFileWithLimitTool::new(10)); // Limit to 10 calls

    let mut builder = LlmAgentBuilder::new(KNOWLEDGE_GEN_AGENT_NAME)
        .instruction(&instruction)
        .model(model)
        .tool(Arc::new(LoadDocumentSummaryTool::new(iteration_id.clone())))
        .tool(read_file_with_limit.clone())
        .tool(Arc::new(ListFilesWorkspaceTool))
        .tool(Arc::new(SaveKnowledgeSnapshotTool::new(iteration_id.clone(), iteration_number)))
        .include_contents(IncludeContents::None);

    // Add base knowledge tool if this is an evolution iteration
    if let Some(base_id) = base_iteration_id {
        builder = builder.tool(Arc::new(LoadBaseKnowledgeTool::new(base_id)));
    }

    let agent = builder.build()?;

    Ok(Arc::new(agent))
}

// ============================================================================
// Project Manager Agent - Post-delivery chat agent
// ============================================================================

pub fn create_project_manager_agent(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn adk_core::Agent>> {
    let instruction = PROJECT_MANAGER_AGENT_INSTRUCTION.replace("{ITERATION_ID}", &iteration_id);

    let agent = LlmAgentBuilder::new("project_manager_agent")
        .instruction(&instruction)
        .model(model)
        .tool(Arc::new(PMGotoStageTool))
        .tool(Arc::new(PMCreateIterationTool::new(iteration_id.clone())))
        .tool(Arc::new(PMRespondTool))
        .tool(Arc::new(PMSaveDecisionTool::new(iteration_id.clone())))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id.clone())))
        .tool(Arc::new(ListFilesTool))  // Allow PM to see project files
        .tool(Arc::new(ReadFileTool))   // Allow PM to read files
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

/// Load artifacts summary for a given iteration
fn load_artifacts_summary_for_pm(iteration_store: &IterationStore, iteration_id: &str) -> Result<String, String> {
    use std::fs;
    
    let iteration_dir = iteration_store.iteration_path(iteration_id)
        .map_err(|e| format!("Failed to get iteration path: {}", e))?;
    
    let mut summary = String::new();
    
    // Load key artifacts
    let artifacts_to_load = [
        ("idea", "idea.md"),
        ("prd", "prd.md"),
        ("design", "design.md"),
        ("plan", "plan.md"),
    ];
    
    for (name, filename) in artifacts_to_load.iter() {
        let path = iteration_dir.join("artifacts").join(filename);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                // Only include first 500 chars of each artifact
                let truncated = if content.len() > 500 {
                    format!("{}...[truncated]", &content[..500])
                } else {
                    content
                };
                summary.push_str(&format!("\n\n## {} ({}):\n{}", name.to_uppercase(), filename, truncated));
            }
        }
    }
    
    // Add code structure info
    let code_dir = iteration_dir.join("workspace");
    if code_dir.exists() {
        summary.push_str("\n\n## Project Files:\n");
        if let Ok(entries) = fs::read_dir(&code_dir) {
            for entry in entries.flatten().take(20) {
                if let Ok(name) = entry.file_name().into_string() {
                    summary.push_str(&format!("- {}\n", name));
                }
            }
        }
    }
    
    Ok(summary)
}

/// Execute a PM agent message and return the response and function calls
pub async fn execute_pm_agent_message(
    model: Arc<dyn Llm>,
    iteration_id: String,
    message: String,
    history: Vec<serde_json::Value>,
) -> Result<(String, Vec<adk_core::Part>), String> {
    use adk_core::Content;
    use futures::StreamExt;
    use crate::pipeline::stage_executor::{SimpleInvocationContext, extract_text_from_event};
    use crate::pipeline::PipelineContext;
    use crate::persistence::{ProjectStore, IterationStore};
    use std::sync::Arc as StdArc;

    // Load project and iteration
    let project_store = ProjectStore::new();
    let project = project_store.load()
        .map_err(|e| format!("Failed to load project: {}", e))?
        .ok_or_else(|| "No project found".to_string())?;

    let iteration_store = IterationStore::new();
    let iteration = iteration_store.load(&iteration_id)
        .map_err(|e| format!("Failed to load iteration: {}", e))?;

    // Get workspace path
    let workspace_path = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace path: {}", e))?;

    // Load artifacts summary for context
    let artifacts_summary = load_artifacts_summary_for_pm(&iteration_store, &iteration_id)
        .unwrap_or_else(|e| {
            eprintln!("[PM Agent] Warning: Failed to load artifacts: {}", e);
            String::new()
        });

    // Load memory/decisions
    let memory_store = crate::persistence::MemoryStore::new();
    let project_memory = memory_store.load_project_memory()
        .map_err(|e| format!("Failed to load memory: {}", e))
        .unwrap_or_default();
    
    let decisions_summary = if !project_memory.decisions.is_empty() {
        let mut summary = String::from("\n\n## Previous Decisions:\n");
        for decision in project_memory.decisions.iter().take(10) {
            summary.push_str(&format!("- {}: {}\n", decision.title, decision.decision));
        }
        summary
    } else {
        String::new()
    };

    // Create PM Agent
    let pm_agent = create_project_manager_agent(model, iteration_id.clone())
        .map_err(|e| format!("Failed to create PM agent: {}", e))?;

    // Build conversation history string
    let conversation_history = if !history.is_empty() {
        let mut history_str = String::from("\n\n## Conversation History:\n");
        for msg in history.iter() {
            let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let role = if msg_type == "user" { "User" } else { "Assistant" };
            history_str.push_str(&format!("{}: {}\n", role, content));
        }
        history_str
    } else {
        String::new()
    };

    // Build language instruction from global config
    let language_instruction = crate::config::get_language_instruction();

    // Build prompt with context
    let prompt = format!(
        "User message: {}\n\n\
        ## Current Iteration Info:\n\
        - Title: {}\n\
        - Description: {}\n\
        - Status: {}\n\
        - Current Stage: {}\n\
        {}\
        {}\
        {}\
        \n\nPlease analyze the user's request and respond appropriately. \
        If the user wants to fix a bug or make changes, use the appropriate tool (pm_goto_stage or pm_create_iteration). \
        If you need more information, use pm_respond to ask for clarification.\n\n{}",
        message,
        iteration.title,
        iteration.description,
        format!("{:?}", iteration.status),
        iteration.current_stage.clone().unwrap_or_default(),
        artifacts_summary,
        decisions_summary,
        conversation_history,
        language_instruction
    );

    // Create content
    let content = Content::new("user").with_text(prompt);

    // Create context
    let ctx = PipelineContext::new(project, iteration, workspace_path);

    // Create invocation context
    let invocation_ctx = StdArc::new(SimpleInvocationContext::new(
        &ctx,
        &content,
        pm_agent.clone(),
    ));

    // Execute agent
    let mut stream = pm_agent.run(invocation_ctx)
        .await
        .map_err(|e| format!("Agent execution failed: {}", e))?;

    // Collect response and detect actions
    let mut agent_message = String::new();
    let mut all_parts: Vec<adk_core::Part> = Vec::new();
    let mut suggested_actions: Vec<serde_json::Value> = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                // Extract text content
                if let Some(text) = extract_text_from_event(&event) {
                    if !text.trim().is_empty() {
                        agent_message.push_str(&text);
                    }
                }
                
                // Collect all parts (includes function calls)
                if let Some(content) = event.content() {
                    for part in &content.parts {
                        all_parts.push(part.clone());
                    }
                }
            }
            Err(e) => {
                eprintln!("[PM Agent] Event error: {}", e);
            }
        }
    }

    // After execution, check if any tools were called by examining the response
    // The agent should have called tools if user requested actions
    let msg_lower = agent_message.to_lowercase();
    
    // Detect actions based on response content
    if msg_lower.contains("pm_goto_stage") || msg_lower.contains("goto_stage") {
        let mut action = serde_json::json!({
            "type": "pm_goto_stage",
            "description": "Tool called: pm_goto_stage"
        });
        
        // Try to extract stage from message
        for stage in &["idea", "prd", "design", "plan", "coding"] {
            if msg_lower.contains(stage) {
                action["target_stage"] = serde_json::json!(stage);
                break;
            }
        }
        suggested_actions.push(action);
    }
    
    if msg_lower.contains("pm_create_iteration") || msg_lower.contains("create_iteration") {
        suggested_actions.push(serde_json::json!({
            "type": "pm_create_iteration",
            "description": "Tool called: pm_create_iteration"
        }));
    }

    if agent_message.is_empty() {
        agent_message = "处理完成".to_string();
    }

    // Append action suggestions to message if detected
    if !suggested_actions.is_empty() {
        let actions_json = serde_json::to_string(&suggested_actions)
            .unwrap_or_else(|_| "[]".to_string());
        agent_message = format!(
            "{}\n\n<!--ACTIONS:{}-->",
            agent_message,
            actions_json
        );
    }

    Ok((agent_message, all_parts))
}
