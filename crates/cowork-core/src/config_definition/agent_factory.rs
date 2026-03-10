// Config-Driven Agent Factory
//
// Creates agents based on configuration definitions instead of hardcoded functions.

use std::sync::Arc;
use anyhow::{Result, Context};

use crate::config_definition::{
    AgentDefinition, StageDefinition, StageType,
    global_registry,
};
use crate::instructions::*;
use crate::tools::*;
use crate::storage::set_iteration_id;
use adk_agent::{LlmAgentBuilder, LoopAgent};
use adk_core::{Llm, Agent, IncludeContents};

/// Create an agent from configuration definition
pub fn create_agent_from_config(
    definition: &AgentDefinition,
    model: Arc<dyn Llm>,
    iteration_id: String,
) -> Result<Arc<dyn Agent>> {
    // Resolve instruction from reference
    let instruction = resolve_instruction(&definition.instruction, &iteration_id)?;
    
    // Create agent builder
    let mut builder = LlmAgentBuilder::new(&definition.id)
        .instruction(&instruction)
        .model(model);
    
    // Add tools based on tool references
    for tool_ref in &definition.tools {
        let tool = create_tool_from_reference(&tool_ref.tool_id, &iteration_id)?;
        builder = builder.tool(tool);
    }
    
    // Set content inclusion mode
    // Note: Current adk_core only supports None, LastN, All variants may not be available
    let include_contents = IncludeContents::None; // All agents use None for now
    builder = builder.include_contents(include_contents);
    
    // Build the agent
    let agent = builder.build()
        .with_context(|| format!("Failed to build agent: {}", definition.id))?;
    
    Ok(Arc::new(agent))
}

/// Create a loop agent (Actor-Critic) from stage definition
pub fn create_loop_agent_from_config(
    stage_definition: &StageDefinition,
    model: Arc<dyn Llm>,
    iteration_id: String,
) -> Result<Arc<dyn Agent>> {
    let actor_critic = stage_definition.actor_critic.as_ref()
        .with_context(|| format!("Stage {} is ActorCritic type but has no actor_critic config", stage_definition.id))?;
    
    // Get actor definition from registry
    let registry = global_registry();
    let actor_def = registry.get_agent(&actor_critic.actor)
        .with_context(|| format!("Actor agent not found: {}", actor_critic.actor))?;
    let critic_def = registry.get_agent(&actor_critic.critic)
        .with_context(|| format!("Critic agent not found: {}", actor_critic.critic))?;
    
    // Create actor agent
    let actor = create_simple_agent_from_config(&actor_def, model.clone(), iteration_id.clone())?;
    
    // Create critic agent
    let critic = create_simple_agent_from_config(&critic_def, model, iteration_id)?;
    
    // Create loop agent
    let max_iterations = actor_critic.max_iterations;
    let mut loop_agent = LoopAgent::new(
        &stage_definition.id,
        vec![actor, critic],
    );
    loop_agent = loop_agent.with_max_iterations(max_iterations);
    
    Ok(Arc::new(loop_agent))
}

/// Create a simple (non-loop) agent from config
fn create_simple_agent_from_config(
    definition: &AgentDefinition,
    model: Arc<dyn Llm>,
    iteration_id: String,
) -> Result<Arc<dyn Agent>> {
    // Resolve instruction
    let instruction = resolve_instruction(&definition.instruction, &iteration_id)?;
    
    // Create agent builder
    let mut builder = LlmAgentBuilder::new(&definition.id)
        .instruction(&instruction)
        .model(model);
    
    // Add tools
    for tool_ref in &definition.tools {
        let tool = create_tool_from_reference(&tool_ref.tool_id, &iteration_id)?;
        builder = builder.tool(tool);
    }
    
    // Set content inclusion mode
    // Note: Current adk_core only supports None, LastN, All variants may not be available
    let include_contents = IncludeContents::None; // All agents use None for now
    builder = builder.include_contents(include_contents);
    
    let agent = builder.build()
        .with_context(|| format!("Failed to build agent: {}", definition.id))?;
    
    Ok(Arc::new(agent))
}

/// Resolve instruction from reference string
fn resolve_instruction(reference: &str, iteration_id: &str) -> Result<String> {
    if let Some(content) = reference.strip_prefix("inline://") {
        Ok(content.replace("{ITERATION_ID}", iteration_id))
    } else if let Some(path) = reference.strip_prefix("file://") {
        // Load from file
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to load instruction file: {}", path))?;
        Ok(content.replace("{ITERATION_ID}", iteration_id))
    } else if let Some(name) = reference.strip_prefix("builtin://") {
        // Use built-in instruction
        let instruction = get_builtin_instruction(name)
            .with_context(|| format!("Unknown built-in instruction: {}", name))?;
        Ok(instruction.replace("{ITERATION_ID}", iteration_id))
    } else {
        // Treat as built-in by default (backward compatibility)
        if let Some(instruction) = get_builtin_instruction(reference) {
            Ok(instruction.replace("{ITERATION_ID}", iteration_id))
        } else {
            Ok(reference.replace("{ITERATION_ID}", iteration_id))
        }
    }
}

/// Get built-in instruction by name
fn get_builtin_instruction(name: &str) -> Option<&'static str> {
    match name {
        // Idea agent
        "idea_agent" => Some(IDEA_AGENT_INSTRUCTION),
        
        // PRD agents
        "prd_actor" => Some(PRD_ACTOR_INSTRUCTION),
        "prd_critic" => Some(PRD_CRITIC_INSTRUCTION),
        
        // Design agents
        "design_actor" => Some(DESIGN_ACTOR_INSTRUCTION),
        "design_critic" => Some(DESIGN_CRITIC_INSTRUCTION),
        
        // Plan agents
        "plan_actor" => Some(PLAN_ACTOR_INSTRUCTION),
        "plan_critic" => Some(PLAN_CRITIC_INSTRUCTION),
        
        // Coding agents
        "coding_actor" => Some(CODING_ACTOR_INSTRUCTION),
        "coding_critic" => Some(CODING_CRITIC_INSTRUCTION),
        
        // Other agents
        "check_agent" => Some(CHECK_AGENT_INSTRUCTION),
        "delivery_agent" => Some(DELIVERY_AGENT_INSTRUCTION),
        "summary_agent" => Some(SUMMARY_AGENT_INSTRUCTION),
        "knowledge_gen_agent" => Some(KNOWLEDGE_GEN_AGENT_INSTRUCTION),
        "project_manager" => Some(PROJECT_MANAGER_AGENT_INSTRUCTION),
        
        _ => None,
    }
}

/// Create a tool instance from a tool reference
fn create_tool_from_reference(tool_id: &str, iteration_id: &str) -> Result<Arc<dyn adk_tool::Tool>> {
    // Map tool IDs to tool instances
    let tool: Arc<dyn adk_tool::Tool> = match tool_id {
        // Idea tools
        "save_idea" => Arc::new(SaveIdeaTool),
        
        // Data tools
        "create_requirement" => Arc::new(CreateRequirementTool),
        "update_requirement" => Arc::new(UpdateRequirementTool),
        "delete_requirement" => Arc::new(DeleteRequirementTool),
        "get_requirements" => Arc::new(GetRequirementsTool),
        "add_feature" => Arc::new(AddFeatureTool),
        "update_feature" => Arc::new(UpdateFeatureTool),
        "create_task" => Arc::new(CreateTaskTool),
        "update_task_status" => Arc::new(UpdateTaskStatusTool),
        "get_design" => Arc::new(GetDesignTool),
        
        // File tools
        "read_file" => Arc::new(ReadFileTool),
        "write_file" => Arc::new(WriteFileTool),
        "list_files" => Arc::new(ListFilesTool),
        "run_command" => Arc::new(RunCommandTool),
        "read_file_truncated" => Arc::new(ReadFileTruncatedTool),
        
        // Document tools
        "load_idea" => Arc::new(LoadIdeaTool),
        "load_prd_doc" => Arc::new(LoadPrdDocTool),
        "load_design_doc" => Arc::new(LoadDesignDocTool),
        "load_plan_doc" => Arc::new(LoadPlanDocTool),
        "save_prd_doc" => Arc::new(SavePrdDocTool),
        "save_design_doc" => Arc::new(SaveDesignDocTool),
        "save_plan_doc" => Arc::new(SavePlanDocTool),
        "save_delivery_report" => Arc::new(SaveDeliveryReportTool),
        
        // Design tools
        "create_design_component" => Arc::new(CreateDesignComponentTool),
        "add_component" => Arc::new(CreateDesignComponentTool), // Alias for backward compatibility
        
        // Plan tools
        "get_plan" => Arc::new(GetPlanTool),
        "get_implementation_plan" => Arc::new(GetPlanTool), // Alias for backward compatibility
        
        // Validation tools
        "check_feature_coverage" => Arc::new(CheckFeatureCoverageTool),
        "check_task_dependencies" => Arc::new(CheckTaskDependenciesTool),
        "check_tests" => Arc::new(CheckTestsTool),
        "check_lint" => Arc::new(CheckLintTool),
        "check_data_format" => Arc::new(CheckDataFormatTool),
        
        // HITL tools
        "provide_feedback" => Arc::new(ProvideFeedbackTool),
        "load_feedback_history" => Arc::new(LoadFeedbackHistoryTool),
        
        // Memory tools
        "query_memory" => Arc::new(QueryMemoryTool::new(iteration_id.to_string())),
        "save_insight" => Arc::new(SaveInsightTool::new(iteration_id.to_string())),
        "save_issue" => Arc::new(SaveIssueTool::new(iteration_id.to_string())),
        "save_learning" => Arc::new(SaveLearningTool::new(iteration_id.to_string())),
        "promote_to_decision" => Arc::new(PromoteToDecisionTool::new(iteration_id.to_string())),
        "promote_to_pattern" => Arc::new(PromoteToPatternTool::new(iteration_id.to_string())),
        
        // Deployment tools
        "copy_workspace_to_project" => Arc::new(CopyWorkspaceToProjectTool),
        
        // Flow control tools
        "goto_stage" => Arc::new(GotoStageTool),
        
        // PM tools (require iteration_id)
        "pm_goto_stage" => Arc::new(PMGotoStageTool::new(iteration_id.to_string())),
        "pm_create_iteration" => Arc::new(PMCreateIterationTool::new(iteration_id.to_string())),
        "pm_respond" => Arc::new(PMRespondTool),
        "pm_save_decision" => Arc::new(PMSaveDecisionTool::new(iteration_id.to_string())),
        
        _ => return Err(anyhow::anyhow!("Unknown tool: {}", tool_id)),
    };
    
    Ok(tool)
}

/// Create an agent for a stage using configuration
pub fn create_agent_for_stage(
    stage_id: &str,
    model: Arc<dyn Llm>,
    iteration_id: String,
) -> Result<Arc<dyn Agent>> {
    // Set iteration ID for data tools
    set_iteration_id(iteration_id.clone());
    
    // Get stage definition from registry
    let registry = global_registry();
    let stage = registry.get_stage(stage_id)
        .with_context(|| format!("Stage not found: {}", stage_id))?;
    
    // Create agent based on stage type
    match &stage.stage_type {
        StageType::Simple => {
            let agent_id = stage.agent.as_ref()
                .with_context(|| format!("Simple stage {} has no agent reference", stage_id))?;
            let agent_def = registry.get_agent(agent_id)
                .with_context(|| format!("Agent not found: {}", agent_id))?;
            create_agent_from_config(&agent_def, model, iteration_id)
        }
        StageType::ActorCritic => {
            create_loop_agent_from_config(&stage, model, iteration_id)
        }
    }
}

/// Initialize the configuration registry with built-in configs and user configs
pub fn initialize_config_registry() -> Result<()> {
    let registry = global_registry();
    
    // Load built-in configs first
    let builtin_report = crate::config_definition::load_builtin_configs(&registry)?;
    
    tracing::info!(
        "Loaded built-in configs: {} agents, {} stages, {} flows",
        builtin_report.agents_loaded, builtin_report.stages_loaded, builtin_report.flows_loaded
    );
    
    if builtin_report.has_errors() {
        for error in &builtin_report.errors {
            tracing::warn!("Built-in config load error: {}", error);
        }
    }
    
    // Load user configs (will override built-in with same ID)
    let user_report = registry.load_user_configs()?;
    
    if user_report.agents_loaded > 0 || user_report.stages_loaded > 0 || user_report.flows_loaded > 0 {
        tracing::info!(
            "Loaded user configs: {} agents, {} stages, {} flows, {} integrations",
            user_report.agents_loaded, user_report.stages_loaded, 
            user_report.flows_loaded, user_report.integrations_loaded
        );
    }
    
    if !user_report.errors.is_empty() {
        for error in &user_report.errors {
            tracing::warn!("User config load error: {}", error);
        }
    }
    
    let stats = registry.stats();
    tracing::info!(
        "Config registry initialized: {} agents, {} stages, {} flows total",
        stats.agents, stats.stages, stats.flows
    );
    
    Ok(())
}
