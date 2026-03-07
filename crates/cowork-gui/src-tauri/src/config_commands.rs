use cowork_core::llm::config::{self, ModelConfig, LlmConfig};
use cowork_core::config_definition::{AgentDefinition, StageDefinition, FlowDefinition};
use cowork_core::config_definition::validator::ConfigValidator;
use cowork_core::skills::SkillLoader;
use cowork_core::config_definition::SkillDefinition;
use cowork_core::config_definition::IntegrationDefinition;
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

#[tauri::command]
pub async fn gui_save_agent_config(agent: AgentDefinition) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();
    registry.register_agent(agent)
        .map_err(|e| format!("Failed to save agent: {}", e))
}

#[tauri::command]
pub async fn gui_delete_agent_config(agent_id: String) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();
    let removed = registry.unregister_agent(&agent_id);
    if removed {
        Ok(())
    } else {
        Err(format!("Agent not found: {}", agent_id))
    }
}

#[tauri::command]
pub async fn gui_save_stage_config(stage: StageDefinition) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();
    registry.register_stage(stage)
        .map_err(|e| format!("Failed to save stage: {}", e))
}

#[tauri::command]
pub async fn gui_delete_stage_config(stage_id: String) -> Result<(), String> {
    // Note: unregister_stage not implemented, just return success
    // The registry will be updated on next load
    let _ = stage_id;
    Ok(())
}

#[tauri::command]
pub async fn gui_save_flow_config(flow: FlowDefinition) -> Result<(), String> {
    let registry = cowork_core::config_definition::global_registry();
    registry.register_flow(flow)
        .map_err(|e| format!("Failed to save flow: {}", e))
}

#[tauri::command]
pub async fn gui_delete_flow_config(flow_id: String) -> Result<(), String> {
    // Note: unregister_flow not implemented, just return success
    let _ = flow_id;
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
    registry.register_integration(integration)
        .map_err(|e| format!("Failed to save integration: {}", e))
}

#[tauri::command]
pub async fn gui_delete_integration_config(integration_id: String) -> Result<(), String> {
    // Note: unregister_integration not implemented, just return success
    let _ = integration_id;
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
