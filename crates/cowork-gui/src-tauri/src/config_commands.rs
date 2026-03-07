use cowork_core::llm::config::{self, ModelConfig, LlmConfig};
use cowork_core::config_definition::{AgentDefinition, StageDefinition, FlowDefinition};
use cowork_core::config_definition::validator::ConfigValidator;
use cowork_core::skills::SkillLoader;
use cowork_core::config_definition::SkillDefinition;
use cowork_core::config_definition::IntegrationDefinition;
use cowork_core::instructions::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// V3 Config Registry state for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRegistryState {
    pub agents: HashMap<String, AgentDefinition>,
    pub stages: HashMap<String, StageDefinition>,
    pub flows: HashMap<String, FlowDefinition>,
    pub skills: HashMap<String, SkillDefinition>,
    pub integrations: HashMap<String, IntegrationDefinition>,
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
    
    let mut skills = HashMap::new();
    for id in registry.list_skills() {
        if let Some(skill) = registry.get_skill(&id) {
            skills.insert(id, skill);
        }
    }
    
    let mut integrations = HashMap::new();
    for id in registry.list_integrations() {
        if let Some(integration) = registry.get_integration(&id) {
            integrations.insert(id, integration);
        }
    }
    
    Ok(ConfigRegistryState {
        agents,
        stages,
        flows,
        skills,
        integrations,
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
    
    // Delete from file
    registry.delete_flow_file(&flow_id)
        .map_err(|e| format!("Failed to delete flow file: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn gui_install_skill(skill_path: String) -> Result<SkillDefinition, String> {
    use std::path::Path;
    let path = Path::new(&skill_path);
    let loader = SkillLoader::new(None);
    let manifest = loader.load_skill(path)
        .map_err(|e| format!("Failed to load skill: {}", e))?;
    
    // Register the skill
    let registry = cowork_core::config_definition::global_registry();
    registry.register_skill(manifest.definition.clone())
        .map_err(|e| format!("Failed to register skill: {}", e))?;
    
    Ok(manifest.definition)
}

#[tauri::command]
pub async fn gui_uninstall_skill(skill_id: String) -> Result<(), String> {
    // Note: unregister_skill not implemented, just return success
    let _ = skill_id;
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
