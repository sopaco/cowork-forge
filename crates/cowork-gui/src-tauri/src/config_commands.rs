use cowork_core::llm::config::{self, ModelConfig, LlmConfig};
use cowork_core::config_definition::{AgentDefinition, StageDefinition, FlowDefinition};
use cowork_core::config_definition::validator::ConfigValidator;
use cowork_core::skills::SkillManager;
use cowork_core::config_definition::IntegrationDefinition;
use cowork_core::instructions::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Skill info for frontend (from adk-skill SkillDocument)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
}

impl From<&adk_skill::SkillDocument> for SkillInfo {
    fn from(doc: &adk_skill::SkillDocument) -> Self {
        Self {
            id: doc.id.clone(),
            name: doc.name.clone(),
            description: doc.description.clone(),
            tags: doc.tags.clone(),
            body: doc.body.clone(),
        }
    }
}

impl From<&adk_skill::ParsedSkill> for SkillInfo {
    fn from(parsed: &adk_skill::ParsedSkill) -> Self {
        Self {
            id: format!("skill-{}", parsed.name),
            name: parsed.name.clone(),
            description: parsed.description.clone(),
            tags: parsed.tags.clone(),
            body: parsed.body.clone(),
        }
    }
}

/// Config Registry state for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRegistryState {
    pub agents: HashMap<String, AgentDefinition>,
    pub stages: HashMap<String, StageDefinition>,
    pub flows: HashMap<String, FlowDefinition>,
    pub skills: Vec<SkillInfo>,
    pub integrations: HashMap<String, IntegrationDefinition>,
    pub default_flow_id: Option<String>,
}

#[tauri::command]
pub async fn gui_get_config_registry() -> Result<ConfigRegistryState, String> {
    // Get the global registry
    let registry = cowork_core::config_definition::global_registry();

    // Extract all configurations
    let mut agents = HashMap::new();
    for id in registry.list_agents() {
        if let Some(agent) = registry.get_agent(&id) {
            agents.insert(id, agent);
        }
    }

    let mut stages = HashMap::new();
    for id in registry.list_stages() {
        if let Some(stage) = registry.get_stage(&id) {
            stages.insert(id, stage);
        }
    }

    let mut flows = HashMap::new();
    for id in registry.list_flows() {
        if let Some(flow) = registry.get_flow(&id) {
            flows.insert(id, flow);
        }
    }

    // Get skills from SkillManager (using current directory as project root)
    let skills = match SkillManager::for_project(".") {
        Ok(manager) => manager.list_skills().iter().map(SkillInfo::from).collect(),
        Err(e) => {
            tracing::warn!("Failed to load skills: {}", e);
            Vec::new()
        }
    };

    let mut integrations = HashMap::new();
    for id in registry.list_integrations() {
        if let Some(integration) = registry.get_integration(&id) {
            integrations.insert(id, integration);
        }
    }

    // Get default flow ID
    let default_flow_id = registry.get_default_flow_id();

    Ok(ConfigRegistryState {
        agents,
        stages,
        flows,
        skills,
        integrations,
        default_flow_id,
    })
}

/// Reset configuration registry to default built-in configurations
#[tauri::command]
pub async fn gui_reset_config_registry() -> Result<ConfigRegistryState, String> {
    let registry = cowork_core::config_definition::global_registry();

    // Clear existing configurations
    registry.clear()
        .map_err(|e| format!("Failed to clear registry: {}", e))?;

    // Reload built-in configurations
    let report = cowork_core::config_definition::load_builtin_configs(&registry)
        .map_err(|e| format!("Failed to load built-in configs: {}", e))?;

    tracing::info!(
        "Reset config registry: {} agents, {} stages, {} flows",
        report.agents_loaded, report.stages_loaded, report.flows_loaded
    );

    // Return the reset state
    gui_get_config_registry().await
}

/// Builtin instruction info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinInstruction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
}

#[tauri::command]
pub async fn gui_get_builtin_instructions() -> Result<Vec<BuiltinInstruction>, String> {
    let instructions = vec![
        BuiltinInstruction {
            id: "idea_agent".to_string(),
            name: "Idea Agent".to_string(),
            description: "Captures and structures the initial project idea".to_string(),
            content: IDEA_AGENT_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "prd_actor".to_string(),
            name: "PRD Actor".to_string(),
            description: "Generates Product Requirements Document".to_string(),
            content: PRD_ACTOR_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "prd_critic".to_string(),
            name: "PRD Critic".to_string(),
            description: "Reviews and validates the PRD".to_string(),
            content: PRD_CRITIC_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "design_actor".to_string(),
            name: "Design Actor".to_string(),
            description: "Generates system design specification".to_string(),
            content: DESIGN_ACTOR_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "design_critic".to_string(),
            name: "Design Critic".to_string(),
            description: "Reviews and validates the design".to_string(),
            content: DESIGN_CRITIC_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "plan_actor".to_string(),
            name: "Plan Actor".to_string(),
            description: "Generates implementation plan with tasks".to_string(),
            content: PLAN_ACTOR_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "plan_critic".to_string(),
            name: "Plan Critic".to_string(),
            description: "Reviews and validates the plan".to_string(),
            content: PLAN_CRITIC_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "coding_actor".to_string(),
            name: "Coding Actor".to_string(),
            description: "Implements code based on the plan".to_string(),
            content: CODING_ACTOR_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "coding_critic".to_string(),
            name: "Coding Critic".to_string(),
            description: "Reviews and validates the code".to_string(),
            content: CODING_CRITIC_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "check_agent".to_string(),
            name: "Check Agent".to_string(),
            description: "Performs quality validation on code".to_string(),
            content: CHECK_AGENT_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "delivery_agent".to_string(),
            name: "Delivery Agent".to_string(),
            description: "Generates delivery report and deploys".to_string(),
            content: DELIVERY_AGENT_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "summary_agent".to_string(),
            name: "Summary Agent".to_string(),
            description: "Generates document summaries".to_string(),
            content: SUMMARY_AGENT_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "knowledge_gen_agent".to_string(),
            name: "Knowledge Gen Agent".to_string(),
            description: "Extracts knowledge from iterations".to_string(),
            content: KNOWLEDGE_GEN_AGENT_INSTRUCTION.to_string(),
        },
        BuiltinInstruction {
            id: "project_manager".to_string(),
            name: "Project Manager".to_string(),
            description: "Handles user interactions and project management".to_string(),
            content: PROJECT_MANAGER_AGENT_INSTRUCTION.to_string(),
        },
    ];

    Ok(instructions)
}

#[tauri::command]
pub async fn gui_save_agent_config(agent: AgentDefinition) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Register in memory
    registry.register_agent(agent.clone())
        .map_err(|e| format!("Failed to save agent: {}", e))?;

    // Persist to file
    registry.save_agent_to_file(&agent)
        .map_err(|e| format!("Failed to persist agent to file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn gui_delete_agent_config(agent_id: String) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Remove from memory
    let removed = registry.unregister_agent(&agent_id);

    // Delete from file (regardless of memory removal result)
    registry.delete_agent_file(&agent_id)
        .map_err(|e| format!("Failed to delete agent file: {}", e))?;

    if removed {
        Ok(())
    } else {
        // Still return Ok since file was deleted
        tracing::warn!("Agent {} was not in memory but file was deleted", agent_id);
        Ok(())
    }
}

#[tauri::command]
pub async fn gui_save_stage_config(stage: StageDefinition) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Register in memory
    registry.register_stage(stage.clone())
        .map_err(|e| format!("Failed to save stage: {}", e))?;

    // Persist to file
    registry.save_stage_to_file(&stage)
        .map_err(|e| format!("Failed to persist stage to file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn gui_delete_stage_config(stage_id: String) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Delete from file
    registry.delete_stage_file(&stage_id)
        .map_err(|e| format!("Failed to delete stage file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn gui_save_flow_config(flow: FlowDefinition) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Check if trying to modify a built-in flow
    if let Some(existing) = registry.get_flow(&flow.id) {
        if existing.is_builtin {
            return Err("Cannot modify built-in flow. Create a new flow instead.".to_string());
        }
    }

    // Ensure user flows are not marked as builtin
    let mut flow = flow;
    flow.is_builtin = false;

    // Register in memory
    registry.register_flow(flow.clone())
        .map_err(|e| format!("Failed to save flow: {}", e))?;

    // Persist to file
    registry.save_flow_to_file(&flow)
        .map_err(|e| format!("Failed to persist flow to file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn gui_delete_flow_config(flow_id: String) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Check if trying to delete a built-in flow
    if let Some(flow) = registry.get_flow(&flow_id) {
        if flow.is_builtin {
            return Err("Cannot delete built-in flow.".to_string());
        }
    }

    // Remove from memory
    registry.unregister_flow(&flow_id);

    // Delete from file
    registry.delete_flow_file(&flow_id)
        .map_err(|e| format!("Failed to delete flow file: {}", e))?;

    // If this was the default flow, clear the default setting
    if registry.get_default_flow_id().as_deref() == Some(&flow_id) {
        registry.set_default_flow(None)
            .map_err(|e| format!("Failed to clear default flow: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn gui_set_default_flow(flow_id: String) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Verify the flow exists
    if registry.get_flow(&flow_id).is_none() {
        return Err(format!("Flow not found: {}", flow_id));
    }

    // Set as default
    let flow_id_clone = flow_id.clone();
    registry.set_default_flow(Some(flow_id))
        .map_err(|e| format!("Failed to set default flow: {}", e))?;

    tracing::info!("Set default flow to: {}", flow_id_clone);
    Ok(())
}

/// Install a skill from a local directory
#[tauri::command]
pub async fn gui_install_skill(skill_path: String) -> Result<SkillInfo, String> {
    use std::path::Path;

    // Strip @ prefix if present (Tauri dialog may add this)
    let skill_path = skill_path.strip_prefix('@').unwrap_or(&skill_path);
    let path = Path::new(skill_path);

    // Read and parse SKILL.md first to get skill info
    let skill_md_path = path.join("SKILL.md");
    if !skill_md_path.exists() {
        return Err(format!("Source directory does not contain SKILL.md: {:?}", path));
    }

    let content = std::fs::read_to_string(&skill_md_path)
        .map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

    let parsed = adk_skill::parse_skill_markdown(&skill_md_path, &content)
        .map_err(|e| format!("Failed to parse SKILL.md: {}", e))?;

    let skill_name = parsed.name.clone();

    // Create target directory and copy skill files
    let target_dir = Path::new(".").join(".skills").join(&skill_name);
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    // Copy all files from source to target
    copy_dir_all(path, &target_dir)
        .map_err(|e| format!("Failed to copy skill files: {}", e))?;

    tracing::info!("Installed skill '{}' to {:?}", skill_name, target_dir);

    // Return SkillInfo from the parsed skill document
    Ok(SkillInfo::from(&parsed))
}

/// Recursively copy a directory
fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let ty = entry.file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?;

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}

/// Uninstall a skill by name
#[tauri::command]
pub async fn gui_uninstall_skill(skill_name: String) -> Result<(), String> {
    use std::path::Path;

    // Get skills directory
    let skills_dir = Path::new(".").join(".skills").join(&skill_name);

    if skills_dir.exists() {
        std::fs::remove_dir_all(&skills_dir)
            .map_err(|e| format!("Failed to remove skill directory: {}", e))?;
        tracing::info!("Uninstalled skill: {}", skill_name);
    } else {
        tracing::warn!("Skill directory not found: {:?}", skills_dir);
    }

    Ok(())
}

#[tauri::command]
pub async fn gui_save_integration_config(integration: IntegrationDefinition) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Register in memory
    registry.register_integration(integration.clone())
        .map_err(|e| format!("Failed to save integration: {}", e))?;

    // Persist to file
    registry.save_integration_to_file(&integration)
        .map_err(|e| format!("Failed to persist integration to file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn gui_delete_integration_config(integration_id: String) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    // Delete from file
    registry.delete_integration_file(&integration_id)
        .map_err(|e| format!("Failed to delete integration file: {}", e))?;

    Ok(())
}

/// Validation issue for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
    pub severity: String,
}

/// Validation result for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

impl From<cowork_core::config_definition::validator::ValidationResult> for ValidationResult {
    fn from(result: cowork_core::config_definition::validator::ValidationResult) -> Self {
        let issues = result.errors.iter()
            .map(|e| ValidationIssue {
                path: String::new(),
                message: e.clone(),
                severity: "error".to_string(),
            })
            .chain(result.warnings.iter()
                .map(|w| ValidationIssue {
                    path: String::new(),
                    message: w.clone(),
                    severity: "warning".to_string(),
                }))
            .collect();

        Self {
            valid: result.is_valid,
            issues,
        }
    }
}

#[tauri::command]
pub async fn gui_validate_agent_config(agent: AgentDefinition) -> Result<ValidationResult, String> {
    let registry = cowork_core::config_definition::global_registry();
    let validator = ConfigValidator::new(&registry);
    let result = validator.validate_agent(&agent);
    Ok(ValidationResult::from(result))
}

#[tauri::command]
pub async fn gui_validate_flow_config(flow: FlowDefinition) -> Result<ValidationResult, String> {
    let registry = cowork_core::config_definition::global_registry();
    let validator = ConfigValidator::new(&registry);
    let result = validator.validate_flow(&flow);
    Ok(ValidationResult::from(result))
}

#[tauri::command]
pub async fn gui_export_config(config_type: String, config_id: String) -> Result<String, String> {
    let registry = cowork_core::config_definition::global_registry();

    let json = match config_type.as_str() {
        "agent" => {
            let agent = registry.get_agent(&config_id)
                .ok_or_else(|| format!("Agent not found: {}", config_id))?;
            serde_json::to_string_pretty(&agent)
                .map_err(|e| format!("Failed to serialize agent: {}", e))?
        }
        "stage" => {
            let stage = registry.get_stage(&config_id)
                .ok_or_else(|| format!("Stage not found: {}", config_id))?;
            serde_json::to_string_pretty(&stage)
                .map_err(|e| format!("Failed to serialize stage: {}", e))?
        }
        "flow" => {
            let flow = registry.get_flow(&config_id)
                .ok_or_else(|| format!("Flow not found: {}", config_id))?;
            serde_json::to_string_pretty(&flow)
                .map_err(|e| format!("Failed to serialize flow: {}", e))?
        }
        _ => return Err(format!("Unknown config type: {}", config_type)),
    };

    Ok(json)
}

#[tauri::command]
pub async fn gui_import_config(config_type: String, json_data: String) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();

    match config_type.as_str() {
        "agent" => {
            let agent: AgentDefinition = serde_json::from_str(&json_data)
                .map_err(|e| format!("Failed to parse agent JSON: {}", e))?;
            registry.register_agent(agent)
                .map_err(|e| format!("Failed to register agent: {}", e))?;
        }
        "stage" => {
            let stage: StageDefinition = serde_json::from_str(&json_data)
                .map_err(|e| format!("Failed to parse stage JSON: {}", e))?;
            registry.register_stage(stage)
                .map_err(|e| format!("Failed to register stage: {}", e))?;
        }
        "flow" => {
            let flow: FlowDefinition = serde_json::from_str(&json_data)
                .map_err(|e| format!("Failed to parse flow JSON: {}", e))?;
            registry.register_flow(flow)
                .map_err(|e| format!("Failed to register flow: {}", e))?;
        }
        _ => return Err(format!("Unknown config type: {}", config_type)),
    };

    Ok(())
}

// Legacy LLM config commands below

#[tauri::command]
pub async fn get_app_config() -> Result<ModelConfig, String> {
    config::load_config()
        .map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
pub async fn save_app_config(config: ModelConfig) -> Result<String, String> {
    let path = config::save_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_config_path() -> Result<String, String> {
    let path = config::get_config_path()
        .map_err(|e| format!("Failed to get config path: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_default_config() -> ModelConfig {
    ModelConfig::default()
}

#[tauri::command]
pub async fn open_config_folder() -> Result<(), String> {
    let config_path = config::get_config_path()
        .map_err(|e| format!("Failed to get config path: {}", e))?;

    let parent = config_path.parent()
        .ok_or_else(|| "Failed to get config directory".to_string())?;

    open_folder_in_explorer(parent)
}

#[tauri::command]
pub async fn test_llm_connection(llm_config: LlmConfig) -> Result<bool, String> {
    use cowork_core::llm::create_llm_client;
    use adk_core::{Content, Part, LlmRequest};
    use futures::StreamExt;

    if llm_config.api_base_url.is_empty() || llm_config.api_key.is_empty() || llm_config.model_name.is_empty() {
        return Err("Please fill in all LLM settings (API URL, API Key, and Model Name)".to_string());
    }

    let client = create_llm_client(&llm_config)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;

    let contents = vec![Content {
        role: "user".to_string(),
        parts: vec![Part::Text { text: "Hello, this is a connection test. Please respond with 'OK'.".to_string() }],
    }];

    let test_request = LlmRequest {
        model: llm_config.model_name.clone(),
        contents,
        config: None,
        tools: Default::default(),
    };

    let mut stream = client.generate_content(test_request, false).await
        .map_err(|e| format!("Connection test failed: {}", e))?;

    let mut response_text = String::new();
    while let Some(chunk) = stream.next().await {
        if let Ok(r) = chunk {
            if let Some(c) = r.content {
                for p in c.parts.iter() {
                    if let Part::Text { text } = p {
                        response_text.push_str(text);
                    }
                }
            }
        }
    }

    if response_text.is_empty() {
        return Err("Connection test failed: Empty response from API".to_string());
    }

    Ok(true)
}

#[tauri::command]
pub async fn has_valid_config() -> bool {
    config::load_config().is_ok()
}

fn open_folder_in_explorer(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

/// Tool info for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
}

/// Get all available tools that can be assigned to agents
#[tauri::command]
pub async fn gui_get_available_tools() -> Result<Vec<ToolInfo>, String> {
    // This list should match the tools supported in agent_factory.rs
    let tools = vec![
        // Idea tools
        ToolInfo {
            id: "save_idea".to_string(),
            name: "Save Idea".to_string(),
            category: "Idea".to_string(),
            description: "Save the initial project idea document".to_string(),
        },
        
        // Data tools
        ToolInfo {
            id: "create_requirement".to_string(),
            name: "Create Requirement".to_string(),
            category: "Data".to_string(),
            description: "Create a new requirement".to_string(),
        },
        ToolInfo {
            id: "update_requirement".to_string(),
            name: "Update Requirement".to_string(),
            category: "Data".to_string(),
            description: "Update an existing requirement".to_string(),
        },
        ToolInfo {
            id: "delete_requirement".to_string(),
            name: "Delete Requirement".to_string(),
            category: "Data".to_string(),
            description: "Delete a requirement".to_string(),
        },
        ToolInfo {
            id: "get_requirements".to_string(),
            name: "Get Requirements".to_string(),
            category: "Data".to_string(),
            description: "Retrieve all requirements".to_string(),
        },
        ToolInfo {
            id: "add_feature".to_string(),
            name: "Add Feature".to_string(),
            category: "Data".to_string(),
            description: "Add a feature to a requirement".to_string(),
        },
        ToolInfo {
            id: "update_feature".to_string(),
            name: "Update Feature".to_string(),
            category: "Data".to_string(),
            description: "Update an existing feature".to_string(),
        },
        ToolInfo {
            id: "create_task".to_string(),
            name: "Create Task".to_string(),
            category: "Data".to_string(),
            description: "Create a new task in the plan".to_string(),
        },
        ToolInfo {
            id: "update_task_status".to_string(),
            name: "Update Task Status".to_string(),
            category: "Data".to_string(),
            description: "Update the status of a task".to_string(),
        },
        ToolInfo {
            id: "get_design".to_string(),
            name: "Get Design".to_string(),
            category: "Data".to_string(),
            description: "Retrieve the current design specification".to_string(),
        },
        ToolInfo {
            id: "get_implementation_plan".to_string(),
            name: "Get Implementation Plan".to_string(),
            category: "Data".to_string(),
            description: "Retrieve the implementation plan (alias: get_plan)".to_string(),
        },
        
        // File tools
        ToolInfo {
            id: "read_file".to_string(),
            name: "Read File".to_string(),
            category: "File".to_string(),
            description: "Read the contents of a file in the workspace".to_string(),
        },
        ToolInfo {
            id: "write_file".to_string(),
            name: "Write File".to_string(),
            category: "File".to_string(),
            description: "Write content to a file in the workspace".to_string(),
        },
        ToolInfo {
            id: "list_files".to_string(),
            name: "List Files".to_string(),
            category: "File".to_string(),
            description: "List files in a directory within the workspace".to_string(),
        },
        ToolInfo {
            id: "run_command".to_string(),
            name: "Run Command".to_string(),
            category: "File".to_string(),
            description: "Execute a shell command in the workspace".to_string(),
        },
        ToolInfo {
            id: "read_file_truncated".to_string(),
            name: "Read File Truncated".to_string(),
            category: "File".to_string(),
            description: "Read a file with intelligent truncation for large files".to_string(),
        },
        
        // Document tools (Project Iteration Files)
        ToolInfo {
            id: "load_idea".to_string(),
            name: "Load Idea".to_string(),
            category: "Document".to_string(),
            description: "Load the idea document from current iteration".to_string(),
        },
        ToolInfo {
            id: "load_prd_doc".to_string(),
            name: "Load PRD Doc".to_string(),
            category: "Document".to_string(),
            description: "Load the PRD document from current iteration".to_string(),
        },
        ToolInfo {
            id: "load_design_doc".to_string(),
            name: "Load Design Doc".to_string(),
            category: "Document".to_string(),
            description: "Load the design document from current iteration".to_string(),
        },
        ToolInfo {
            id: "load_plan_doc".to_string(),
            name: "Load Plan Doc".to_string(),
            category: "Document".to_string(),
            description: "Load the implementation plan document from current iteration".to_string(),
        },
        ToolInfo {
            id: "save_prd_doc".to_string(),
            name: "Save PRD Doc".to_string(),
            category: "Document".to_string(),
            description: "Save the PRD document to the artifacts directory".to_string(),
        },
        ToolInfo {
            id: "save_design_doc".to_string(),
            name: "Save Design Doc".to_string(),
            category: "Document".to_string(),
            description: "Save the design document to the artifacts directory".to_string(),
        },
        ToolInfo {
            id: "save_plan_doc".to_string(),
            name: "Save Plan Doc".to_string(),
            category: "Document".to_string(),
            description: "Save the implementation plan document to the artifacts directory".to_string(),
        },
        ToolInfo {
            id: "save_delivery_report".to_string(),
            name: "Save Delivery Report".to_string(),
            category: "Document".to_string(),
            description: "Save the delivery report to the artifacts directory".to_string(),
        },
        ToolInfo {
            id: "save_check_report".to_string(),
            name: "Save Check Report".to_string(),
            category: "Document".to_string(),
            description: "Save the check report to the artifacts directory".to_string(),
        },
        
        // Design tools
        ToolInfo {
            id: "create_design_component".to_string(),
            name: "Create Design Component".to_string(),
            category: "Design".to_string(),
            description: "Create a new design component".to_string(),
        },
        
        // Validation tools
        ToolInfo {
            id: "check_feature_coverage".to_string(),
            name: "Check Feature Coverage".to_string(),
            category: "Validation".to_string(),
            description: "Validate that all features are covered in the design".to_string(),
        },
        ToolInfo {
            id: "check_task_dependencies".to_string(),
            name: "Check Task Dependencies".to_string(),
            category: "Validation".to_string(),
            description: "Validate task dependencies in the plan".to_string(),
        },
        ToolInfo {
            id: "check_tests".to_string(),
            name: "Check Tests".to_string(),
            category: "Validation".to_string(),
            description: "Run tests in the workspace".to_string(),
        },
        ToolInfo {
            id: "check_lint".to_string(),
            name: "Check Lint".to_string(),
            category: "Validation".to_string(),
            description: "Run linting in the workspace".to_string(),
        },
        ToolInfo {
            id: "check_data_format".to_string(),
            name: "Check Data Format".to_string(),
            category: "Validation".to_string(),
            description: "Validate data format consistency".to_string(),
        },
        
        // HITL tools
        ToolInfo {
            id: "provide_feedback".to_string(),
            name: "Provide Feedback".to_string(),
            category: "HITL".to_string(),
            description: "Provide feedback to the user for review".to_string(),
        },
        ToolInfo {
            id: "load_feedback_history".to_string(),
            name: "Load Feedback History".to_string(),
            category: "HITL".to_string(),
            description: "Load history of feedback from previous iterations".to_string(),
        },
        
        // Memory tools
        ToolInfo {
            id: "query_memory".to_string(),
            name: "Query Memory".to_string(),
            category: "Memory".to_string(),
            description: "Query the project memory for relevant context".to_string(),
        },
        ToolInfo {
            id: "save_insight".to_string(),
            name: "Save Insight".to_string(),
            category: "Memory".to_string(),
            description: "Save an insight to the iteration memory".to_string(),
        },
        ToolInfo {
            id: "save_issue".to_string(),
            name: "Save Issue".to_string(),
            category: "Memory".to_string(),
            description: "Save an issue to the iteration memory".to_string(),
        },
        ToolInfo {
            id: "save_learning".to_string(),
            name: "Save Learning".to_string(),
            category: "Memory".to_string(),
            description: "Save a learning to the iteration memory".to_string(),
        },
        ToolInfo {
            id: "promote_to_decision".to_string(),
            name: "Promote to Decision".to_string(),
            category: "Memory".to_string(),
            description: "Promote an insight to a project-level decision".to_string(),
        },
        ToolInfo {
            id: "promote_to_pattern".to_string(),
            name: "Promote to Pattern".to_string(),
            category: "Memory".to_string(),
            description: "Promote a learning to a project-level pattern".to_string(),
        },
        
        // Deployment tools
        ToolInfo {
            id: "copy_workspace_to_project".to_string(),
            name: "Copy Workspace to Project".to_string(),
            category: "Deployment".to_string(),
            description: "Copy generated workspace files to the project directory".to_string(),
        },
        
        // Flow control tools
        ToolInfo {
            id: "goto_stage".to_string(),
            name: "Goto Stage".to_string(),
            category: "Flow Control".to_string(),
            description: "Jump to a specific stage in the flow".to_string(),
        },
        
        // PM tools
        ToolInfo {
            id: "pm_goto_stage".to_string(),
            name: "PM Goto Stage".to_string(),
            category: "PM".to_string(),
            description: "PM agent: Jump to a specific stage".to_string(),
        },
        ToolInfo {
            id: "pm_create_iteration".to_string(),
            name: "PM Create Iteration".to_string(),
            category: "PM".to_string(),
            description: "PM agent: Create a new iteration".to_string(),
        },
        ToolInfo {
            id: "pm_respond".to_string(),
            name: "PM Respond".to_string(),
            category: "PM".to_string(),
            description: "PM agent: Respond to user".to_string(),
        },
        ToolInfo {
            id: "pm_save_decision".to_string(),
            name: "PM Save Decision".to_string(),
            category: "PM".to_string(),
            description: "PM agent: Save a decision to memory".to_string(),
        },
    ];
    
    Ok(tools)
}
