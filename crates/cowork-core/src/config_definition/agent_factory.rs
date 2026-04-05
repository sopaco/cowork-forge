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
use crate::persistence::set_iteration_id;
use crate::skills::{SkillManager, SelectionPolicy};
use adk_agent::{LlmAgentBuilder, LoopAgent};
use adk_core::{Llm, Agent, IncludeContents};
use adk_skill::select_skill_prompt_block;
use crate::llm::config::McpConfig;
use crate::tools::{create_mcp_toolsets_from_config, ConnectedMcpToolset};

/// Global MCP toolsets (initialized once at startup)
static GLOBAL_MCP_TOOLSETS: once_cell::sync::Lazy<std::sync::Mutex<Vec<ConnectedMcpToolset>>> = 
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(Vec::new()));

/// Global flag to track if MCP has been initialized
static GLOBAL_MCP_INITIALIZED: once_cell::sync::Lazy<std::sync::atomic::AtomicBool> = 
    once_cell::sync::Lazy::new(|| std::sync::atomic::AtomicBool::new(false));

/// Initialize MCP toolsets from configuration file
/// This should be called once at application startup after loading config
pub async fn initialize_mcp_toolsets() -> Result<usize> {
    tracing::info!("[MCP] Starting MCP initialization...");
    
    // Try to load config from file
    let mcp_config = match crate::llm::config::load_config() {
        Ok(config) => {
            tracing::info!("[MCP] Config loaded successfully");
            config.mcp
        }
        Err(e) => {
            tracing::warn!("[MCP] Failed to load config: {}", e);
            McpConfig::default()
        }
    };
    
    tracing::info!("[MCP] Config: tavily_api_key={}, deepwiki_enabled={}", 
        if mcp_config.tavily_api_key.is_empty() { "empty" } else { "configured" },
        mcp_config.deepwiki_enabled);
    
    if !mcp_config.is_any_enabled() {
        tracing::info!("[MCP] No MCP servers enabled in config (tavily_api_key is empty and deepwiki is disabled)");
        return Ok(0);
    }
    
    let toolsets = create_mcp_toolsets_from_config(&mcp_config).await?;
    let count = toolsets.len();
    
    tracing::info!("[MCP] Created {} toolset(s) from config", count);
    
    if count > 0 {
        // Store in global static
        {
            let mut guard = GLOBAL_MCP_TOOLSETS.lock().unwrap();
            *guard = toolsets;
            tracing::info!("[MCP] Stored {} toolset(s) in GLOBAL_MCP_TOOLSETS", guard.len());
        }
        tracing::info!("[MCP] MCP initialization completed with {} toolset(s)", count);
        GLOBAL_MCP_INITIALIZED.store(true, std::sync::atomic::Ordering::SeqCst);
    } else {
        tracing::warn!("[MCP] MCP config enabled but no servers connected");
    }
    
    Ok(count)
}

/// Check if MCP has been initialized
pub fn is_mcp_initialized() -> bool {
    GLOBAL_MCP_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst)
}

/// Add MCP toolsets to an agent builder if they are available
pub fn add_mcp_toolsets_to_builder(builder: LlmAgentBuilder) -> LlmAgentBuilder {
    tracing::info!("[MCP] add_mcp_toolsets_to_builder called, attempting to acquire lock...");
    
    // Wait for lock (blocks if MCP is initializing)
    let guard = match GLOBAL_MCP_TOOLSETS.lock() {
        Ok(g) => {
            tracing::info!("[MCP] Lock acquired, toolsets count: {}", g.len());
            g
        }
        Err(e) => {
            tracing::error!("[MCP] Failed to acquire MCP lock: {}", e);
            return builder;
        }
    };
    
    if guard.is_empty() {
        tracing::warn!("[MCP] GLOBAL_MCP_TOOLSETS is empty! This is unexpected if MCP was configured.");
        tracing::warn!("[MCP] MCP initialized flag: {}", GLOBAL_MCP_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst));
        return builder;
    }
    
    // Add all MCP toolsets to the agent using .toolset() method
    let mut current_builder = builder;
    for connected in guard.iter() {
        tracing::info!("[MCP] Adding MCP toolset '{}' to agent", connected.name);
        current_builder = current_builder.toolset(connected.toolset.clone());
    }
    
    tracing::info!("[MCP] Successfully added {} MCP toolset(s) to agent", guard.len());
    current_builder
}

/// Get a list of configured MCP server names (for injecting into agent instructions)
pub fn get_mcp_server_names() -> Vec<String> {
    match GLOBAL_MCP_TOOLSETS.lock() {
        Ok(guard) => guard.iter().map(|t| t.name.clone()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Default max characters to inject from skill body
const DEFAULT_MAX_SKILL_CHARS: usize = 4000;

/// Build a skill context string for injection into agent instructions
///
/// Selects relevant skills based on agent metadata (tags, description, stage type)
/// and returns a formatted context block to prepend to instructions.
fn build_skill_context(definition: &AgentDefinition, stage_type: Option<&StageType>) -> Option<String> {
    // Try to load skills from current directory
    let manager = match SkillManager::for_project(".") {
        Ok(m) => m,
        Err(_) => return None, // No skills available
    };
    
    if manager.is_empty() {
        return None;
    }
    
    // Build query from agent metadata
    let query = build_skill_query(definition, stage_type);
    
    // Select matching skill
    let policy = SelectionPolicy::default();
    let (skill_match, prompt_block) = select_skill_prompt_block(
        manager.index(),
        &query,
        &policy,
        DEFAULT_MAX_SKILL_CHARS,
    )?;
    
    tracing::info!(
        "Injected skill '{}' into agent '{}' (score: {:.2})",
        skill_match.skill.name,
        definition.id,
        skill_match.score
    );
    
    Some(prompt_block)
}

/// Build a query string for skill matching based on agent metadata
fn build_skill_query(definition: &AgentDefinition, stage_type: Option<&StageType>) -> String {
    let mut parts = Vec::new();
    
    // Add agent description
    if let Some(desc) = &definition.description {
        parts.push(desc.clone());
    }
    
    // Add tags
    parts.extend(definition.tags.clone());
    
    // Add stage type context
    if let Some(st) = stage_type {
        let stage_str = match st {
            StageType::Simple => "simple execution",
            StageType::ActorCritic => "iterative refinement actor critic",
        };
        parts.push(stage_str.to_string());
    }
    
    // Join all parts
    parts.join(" ")
}

/// Create an agent from configuration definition
pub fn create_agent_from_config(
    definition: &AgentDefinition,
    model: Arc<dyn Llm>,
    iteration_id: String,
) -> Result<Arc<dyn Agent>> {
    create_agent_from_config_with_stage(definition, model, iteration_id, None)
}

/// Create an agent from configuration definition with optional stage context for skill matching
pub fn create_agent_from_config_with_stage(
    definition: &AgentDefinition,
    model: Arc<dyn Llm>,
    iteration_id: String,
    stage_type: Option<&StageType>,
) -> Result<Arc<dyn Agent>> {
    // Resolve instruction from reference
    let base_instruction = resolve_instruction(&definition.instruction, &iteration_id)?;
    
    // Build skill context and combine with base instruction
    let instruction = if let Some(skill_context) = build_skill_context(definition, stage_type) {
        format!(
            "## Relevant Skills\n\n{}\n\n---\n\n## Agent Instructions\n\n{}",
            skill_context,
            base_instruction
        )
    } else {
        base_instruction
    };
    
    // Create agent builder
    let mut builder = LlmAgentBuilder::new(&definition.id)
        .instruction(&instruction)
        .model(model);
    
    // Add tools based on tool references
    for tool_ref in &definition.tools {
        let tool = create_tool_from_reference(&tool_ref.tool_id, &iteration_id)?;
        builder = builder.tool(tool);
    }

    // Add MCP toolsets if available
    builder = add_mcp_toolsets_to_builder(builder);

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
    
    // Create actor agent with skill injection
    let actor = create_simple_agent_from_config_with_stage(
        &actor_def, 
        model.clone(), 
        iteration_id.clone(),
        Some(&StageType::ActorCritic)
    )?;
    
    // Create critic agent with skill injection
    let critic = create_simple_agent_from_config_with_stage(
        &critic_def, 
        model, 
        iteration_id,
        Some(&StageType::ActorCritic)
    )?;
    
    // Create loop agent
    let max_iterations = actor_critic.max_iterations;
    let mut loop_agent = LoopAgent::new(
        &stage_definition.id,
        vec![actor, critic],
    );
    loop_agent = loop_agent.with_max_iterations(max_iterations);
    
    Ok(Arc::new(loop_agent))
}

/// Create a simple (non-loop) agent from config with optional stage context for skill matching
fn create_simple_agent_from_config_with_stage(
    definition: &AgentDefinition,
    model: Arc<dyn Llm>,
    iteration_id: String,
    stage_type: Option<&StageType>,
) -> Result<Arc<dyn Agent>> {
    // Resolve instruction
    let base_instruction = resolve_instruction(&definition.instruction, &iteration_id)?;
    
    // Build skill context and combine with base instruction
    let instruction = if let Some(skill_context) = build_skill_context(definition, stage_type) {
        format!(
            "## Relevant Skills\n\n{}\n\n---\n\n## Agent Instructions\n\n{}",
            skill_context,
            base_instruction
        )
    } else {
        base_instruction
    };
    
    // Create agent builder
    let mut builder = LlmAgentBuilder::new(&definition.id)
        .instruction(&instruction)
        .model(model);
    
    // Add tools
    for tool_ref in &definition.tools {
        let tool = create_tool_from_reference(&tool_ref.tool_id, &iteration_id)?;
        builder = builder.tool(tool);
    }

    // Add MCP toolsets if available
    builder = add_mcp_toolsets_to_builder(builder);

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
        "save_check_report" => Arc::new(SaveCheckReportTool),
        
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
            create_agent_from_config_with_stage(&agent_def, model, iteration_id, Some(&stage.stage_type))
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
