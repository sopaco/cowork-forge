// Configuration Registry - Central registry for all configuration definitions
//
// The registry manages Agent, Stage, Flow, Skill, and Integration definitions.
// It provides lookup, validation, and lifecycle management.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

use super::agent_definition::AgentDefinition;
use super::stage_definition::StageDefinition;
use super::flow_definition::FlowDefinition;
use super::skill_definition::SkillDefinition;
use super::integration_definition::IntegrationDefinition;

/// Get the user config directory for persistent storage
fn get_user_config_dir() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        Some(home.join(".cowork").join("config"))
    } else {
        None
    }
}

/// Ensure the user config directory exists
fn ensure_user_config_dir() -> Result<PathBuf> {
    let config_dir = get_user_config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine user config directory"))?;
    
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
    
    Ok(config_dir)
}

/// Configuration registry for managing all definitions
pub struct ConfigRegistry {
    /// Agent definitions by ID
    agents: RwLock<HashMap<String, AgentDefinition>>,
    /// Stage definitions by ID
    stages: RwLock<HashMap<String, StageDefinition>>,
    /// Flow definitions by ID
    flows: RwLock<HashMap<String, FlowDefinition>>,
    /// Skill definitions by ID
    skills: RwLock<HashMap<String, SkillDefinition>>,
    /// Integration definitions by ID
    integrations: RwLock<HashMap<String, IntegrationDefinition>>,
    /// Default flow ID
    default_flow: RwLock<Option<String>>,
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            stages: RwLock::new(HashMap::new()),
            flows: RwLock::new(HashMap::new()),
            skills: RwLock::new(HashMap::new()),
            integrations: RwLock::new(HashMap::new()),
            default_flow: RwLock::new(None),
        }
    }
    
    // =========================================================================
    // Agent Management
    // =========================================================================
    
    /// Register an agent definition
    pub fn register_agent(&self, definition: AgentDefinition) -> Result<()> {
        let id = definition.id.clone();
        let mut agents = self.agents.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        agents.insert(id.clone(), definition);
        tracing::debug!("Registered agent: {}", id);
        Ok(())
    }
    
    /// Get an agent definition by ID
    pub fn get_agent(&self, id: &str) -> Option<AgentDefinition> {
        let agents = self.agents.read().ok()?;
        agents.get(id).cloned()
    }
    
    /// List all agent IDs
    pub fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().unwrap_or_else(|e| {
            tracing::error!("Lock error: {}", e);
            panic!("Lock error")
        });
        agents.keys().cloned().collect()
    }
    
    /// Remove an agent definition
    pub fn unregister_agent(&self, id: &str) -> bool {
        let mut agents = self.agents.write().unwrap_or_else(|e| {
            tracing::error!("Lock error: {}", e);
            panic!("Lock error")
        });
        agents.remove(id).is_some()
    }
    
    // =========================================================================
    // Stage Management
    // =========================================================================
    
    /// Register a stage definition
    pub fn register_stage(&self, definition: StageDefinition) -> Result<()> {
        let id = definition.id.clone();
        let mut stages = self.stages.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        stages.insert(id.clone(), definition);
        tracing::debug!("Registered stage: {}", id);
        Ok(())
    }
    
    /// Get a stage definition by ID
    pub fn get_stage(&self, id: &str) -> Option<StageDefinition> {
        let stages = self.stages.read().ok()?;
        stages.get(id).cloned()
    }
    
    /// List all stage IDs
    pub fn list_stages(&self) -> Vec<String> {
        let stages = self.stages.read().unwrap_or_else(|e| {
            tracing::error!("Lock error: {}", e);
            panic!("Lock error")
        });
        stages.keys().cloned().collect()
    }
    
    // =========================================================================
    // Flow Management
    // =========================================================================
    
    /// Register a flow definition
    pub fn register_flow(&self, definition: FlowDefinition) -> Result<()> {
        let id = definition.id.clone();
        let mut flows = self.flows.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        flows.insert(id.clone(), definition);
        tracing::debug!("Registered flow: {}", id);
        Ok(())
    }
    
    /// Get a flow definition by ID
    pub fn get_flow(&self, id: &str) -> Option<FlowDefinition> {
        let flows = self.flows.read().ok()?;
        flows.get(id).cloned()
    }
    
    /// List all flow IDs
    pub fn list_flows(&self) -> Vec<String> {
        let flows = self.flows.read().unwrap_or_else(|e| {
            tracing::error!("Lock error: {}", e);
            panic!("Lock error")
        });
        flows.keys().cloned().collect()
    }
    
    /// Set the default flow
    pub fn set_default_flow(&self, id: Option<String>) -> Result<()> {
        {
            let mut default_flow = self.default_flow.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            *default_flow = id.clone();
        }
        // Persist the setting
        self.save_settings()?;
        Ok(())
    }
    
    /// Get the default flow ID
    pub fn get_default_flow_id(&self) -> Option<String> {
        let default_flow = self.default_flow.read().ok()?;
        default_flow.clone()
    }
    
    /// Get the default flow
    pub fn get_default_flow(&self) -> Option<FlowDefinition> {
        let default_flow = self.default_flow.read().ok()?;
        let flow_id = default_flow.as_ref()?;
        self.get_flow(flow_id)
    }
    
    // =========================================================================
    // Settings Persistence
    // =========================================================================
    
    /// Get the settings file path
    fn get_settings_file_path() -> Option<PathBuf> {
        get_user_config_dir().map(|dir| dir.join("settings.json"))
    }
    
    /// Save settings to persistent storage
    pub fn save_settings(&self) -> Result<()> {
        let settings = Settings {
            default_flow_id: self.get_default_flow_id(),
        };
        
        let config_dir = ensure_user_config_dir()?;
        let file_path = config_dir.join("settings.json");
        let content = serde_json::to_string_pretty(&settings)
            .with_context(|| "Failed to serialize settings")?;
        
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write settings file: {:?}", file_path))?;
        
        tracing::debug!("Saved settings to {:?}", file_path);
        Ok(())
    }
    
    /// Load settings from persistent storage
    pub fn load_settings(&self) -> Result<()> {
        if let Some(file_path) = Self::get_settings_file_path() {
            if file_path.exists() {
                match fs::read_to_string(&file_path)
                    .and_then(|content| serde_json::from_str::<Settings>(&content)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
                {
                    Ok(settings) => {
                        if let Some(flow_id) = settings.default_flow_id {
                            // Only set if the flow exists
                            if self.get_flow(&flow_id).is_some() {
                                let mut default_flow = self.default_flow.write()
                                    .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
                                *default_flow = Some(flow_id.clone());
                                tracing::info!("Loaded default flow setting: {}", flow_id);
                            } else {
                                tracing::warn!("Default flow '{}' not found, ignoring setting", flow_id);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load settings: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
    
    // =========================================================================
    // Skill Management
    // =========================================================================
    
    /// Register a skill definition
    pub fn register_skill(&self, definition: SkillDefinition) -> Result<()> {
        let id = definition.id.clone();
        let mut skills = self.skills.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        skills.insert(id.clone(), definition);
        tracing::debug!("Registered skill: {}", id);
        Ok(())
    }
    
    /// Get a skill definition by ID
    pub fn get_skill(&self, id: &str) -> Option<SkillDefinition> {
        let skills = self.skills.read().ok()?;
        skills.get(id).cloned()
    }
    
    /// List all skill IDs
    pub fn list_skills(&self) -> Vec<String> {
        let skills = self.skills.read().unwrap_or_else(|e| {
            tracing::error!("Lock error: {}", e);
            panic!("Lock error")
        });
        skills.keys().cloned().collect()
    }
    
    /// Check if all skill dependencies are satisfied
    pub fn check_skill_dependencies(&self, skill_id: &str) -> Result<Vec<String>> {
        let skill = self.get_skill(skill_id)
            .with_context(|| format!("Skill not found: {}", skill_id))?;
        
        let mut missing = Vec::new();
        for dep in &skill.dependencies {
            if !dep.optional && self.get_skill(&dep.skill_id).is_none() {
                missing.push(dep.skill_id.clone());
            }
        }
        Ok(missing)
    }
    
    // =========================================================================
    // Integration Management
    // =========================================================================
    
    /// Register an integration definition
    pub fn register_integration(&self, definition: IntegrationDefinition) -> Result<()> {
        let id = definition.id.clone();
        let mut integrations = self.integrations.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        integrations.insert(id.clone(), definition);
        tracing::debug!("Registered integration: {}", id);
        Ok(())
    }
    
    /// Get an integration definition by ID
    pub fn get_integration(&self, id: &str) -> Option<IntegrationDefinition> {
        let integrations = self.integrations.read().ok()?;
        integrations.get(id).cloned()
    }
    
    /// List all integration IDs
    pub fn list_integrations(&self) -> Vec<String> {
        let integrations = self.integrations.read().unwrap_or_else(|e| {
            tracing::error!("Lock error: {}", e);
            panic!("Lock error")
        });
        integrations.keys().cloned().collect()
    }
    
    /// Get all enabled integrations
    pub fn get_enabled_integrations(&self) -> Vec<IntegrationDefinition> {
        let integrations = self.integrations.read().unwrap_or_else(|e| {
            tracing::error!("Lock error: {}", e);
            panic!("Lock error")
        });
        integrations.values().filter(|i| i.enabled).cloned().collect()
    }
    
    // =========================================================================
    // Bulk Operations
    // =========================================================================
    
    /// Clear all definitions
    pub fn clear(&self) -> Result<()> {
        {
            let mut agents = self.agents.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            agents.clear();
        }
        {
            let mut stages = self.stages.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            stages.clear();
        }
        {
            let mut flows = self.flows.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            flows.clear();
        }
        {
            let mut skills = self.skills.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            skills.clear();
        }
        {
            let mut integrations = self.integrations.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            integrations.clear();
        }
        Ok(())
    }
    
    /// Get statistics about the registry
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            agents: self.agents.read().map(|g| g.len()).unwrap_or(0),
            stages: self.stages.read().map(|g| g.len()).unwrap_or(0),
            flows: self.flows.read().map(|g| g.len()).unwrap_or(0),
            skills: self.skills.read().map(|g| g.len()).unwrap_or(0),
            integrations: self.integrations.read().map(|g| g.len()).unwrap_or(0),
        }
    }
    
    // =========================================================================
    // Persistence Operations
    // =========================================================================
    
    /// Save an agent definition to persistent storage
    pub fn save_agent_to_file(&self, agent: &AgentDefinition) -> Result<()> {
        let config_dir = ensure_user_config_dir()?;
        let agents_dir = config_dir.join("agents");
        fs::create_dir_all(&agents_dir)
            .with_context(|| format!("Failed to create agents directory: {:?}", agents_dir))?;
        
        let file_path = agents_dir.join(format!("{}.json", agent.id));
        let content = serde_json::to_string_pretty(agent)
            .with_context(|| format!("Failed to serialize agent: {}", agent.id))?;
        
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write agent file: {:?}", file_path))?;
        
        tracing::info!("Saved agent '{}' to {:?}", agent.id, file_path);
        Ok(())
    }
    
    /// Delete an agent file from persistent storage
    pub fn delete_agent_file(&self, id: &str) -> Result<()> {
        if let Some(config_dir) = get_user_config_dir() {
            let file_path = config_dir.join("agents").join(format!("{}.json", id));
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .with_context(|| format!("Failed to delete agent file: {:?}", file_path))?;
                tracing::info!("Deleted agent file: {:?}", file_path);
            }
        }
        Ok(())
    }
    
    /// Save a stage definition to persistent storage
    pub fn save_stage_to_file(&self, stage: &StageDefinition) -> Result<()> {
        let config_dir = ensure_user_config_dir()?;
        let stages_dir = config_dir.join("stages");
        fs::create_dir_all(&stages_dir)
            .with_context(|| format!("Failed to create stages directory: {:?}", stages_dir))?;
        
        let file_path = stages_dir.join(format!("{}.json", stage.id));
        let content = serde_json::to_string_pretty(stage)
            .with_context(|| format!("Failed to serialize stage: {}", stage.id))?;
        
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write stage file: {:?}", file_path))?;
        
        tracing::info!("Saved stage '{}' to {:?}", stage.id, file_path);
        Ok(())
    }
    
    /// Delete a stage file from persistent storage
    pub fn delete_stage_file(&self, id: &str) -> Result<()> {
        if let Some(config_dir) = get_user_config_dir() {
            let file_path = config_dir.join("stages").join(format!("{}.json", id));
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .with_context(|| format!("Failed to delete stage file: {:?}", file_path))?;
                tracing::info!("Deleted stage file: {:?}", file_path);
            }
        }
        Ok(())
    }
    
    /// Save a flow definition to persistent storage
    pub fn save_flow_to_file(&self, flow: &FlowDefinition) -> Result<()> {
        let config_dir = ensure_user_config_dir()?;
        let flows_dir = config_dir.join("flows");
        fs::create_dir_all(&flows_dir)
            .with_context(|| format!("Failed to create flows directory: {:?}", flows_dir))?;
        
        let file_path = flows_dir.join(format!("{}.json", flow.id));
        let content = serde_json::to_string_pretty(flow)
            .with_context(|| format!("Failed to serialize flow: {}", flow.id))?;
        
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write flow file: {:?}", file_path))?;
        
        tracing::info!("Saved flow '{}' to {:?}", flow.id, file_path);
        Ok(())
    }
    
    /// Delete a flow file from persistent storage
    pub fn delete_flow_file(&self, id: &str) -> Result<()> {
        if let Some(config_dir) = get_user_config_dir() {
            let file_path = config_dir.join("flows").join(format!("{}.json", id));
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .with_context(|| format!("Failed to delete flow file: {:?}", file_path))?;
                tracing::info!("Deleted flow file: {:?}", file_path);
            }
        }
        Ok(())
    }
    
    /// Save an integration definition to persistent storage
    pub fn save_integration_to_file(&self, integration: &IntegrationDefinition) -> Result<()> {
        let config_dir = ensure_user_config_dir()?;
        let integrations_dir = config_dir.join("integrations");
        fs::create_dir_all(&integrations_dir)
            .with_context(|| format!("Failed to create integrations directory: {:?}", integrations_dir))?;
        
        let file_path = integrations_dir.join(format!("{}.json", integration.id));
        let content = serde_json::to_string_pretty(integration)
            .with_context(|| format!("Failed to serialize integration: {}", integration.id))?;
        
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write integration file: {:?}", file_path))?;
        
        tracing::info!("Saved integration '{}' to {:?}", integration.id, file_path);
        Ok(())
    }
    
    /// Delete an integration file from persistent storage
    pub fn delete_integration_file(&self, id: &str) -> Result<()> {
        if let Some(config_dir) = get_user_config_dir() {
            let file_path = config_dir.join("integrations").join(format!("{}.json", id));
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .with_context(|| format!("Failed to delete integration file: {:?}", file_path))?;
                tracing::info!("Deleted integration file: {:?}", file_path);
            }
        }
        Ok(())
    }
    
    /// Load user configurations from persistent storage
    pub fn load_user_configs(&self) -> Result<LoadUserReport> {
        let mut report = LoadUserReport::default();
        
        if let Some(config_dir) = get_user_config_dir() {
            if config_dir.exists() {
                // Load agents
                let agents_dir = config_dir.join("agents");
                if agents_dir.exists() {
                    for entry in fs::read_dir(&agents_dir)
                        .with_context(|| format!("Failed to read agents directory: {:?}", agents_dir))?
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                    {
                        let path = entry.path();
                        match fs::read_to_string(&path)
                            .and_then(|content| serde_json::from_str::<AgentDefinition>(&content)
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
                        {
                            Ok(agent) => {
                                let id = agent.id.clone();
                                self.register_agent(agent)?;
                                report.agents_loaded += 1;
                                tracing::debug!("Loaded user agent: {} from {:?}", id, path);
                            }
                            Err(e) => {
                                report.errors.push(format!("Failed to load agent from {:?}: {}", path, e));
                            }
                        }
                    }
                }
                
                // Load stages
                let stages_dir = config_dir.join("stages");
                if stages_dir.exists() {
                    for entry in fs::read_dir(&stages_dir)
                        .with_context(|| format!("Failed to read stages directory: {:?}", stages_dir))?
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                    {
                        let path = entry.path();
                        match fs::read_to_string(&path)
                            .and_then(|content| serde_json::from_str::<StageDefinition>(&content)
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
                        {
                            Ok(stage) => {
                                let id = stage.id.clone();
                                self.register_stage(stage)?;
                                report.stages_loaded += 1;
                                tracing::debug!("Loaded user stage: {} from {:?}", id, path);
                            }
                            Err(e) => {
                                report.errors.push(format!("Failed to load stage from {:?}: {}", path, e));
                            }
                        }
                    }
                }
                
                // Load flows
                let flows_dir = config_dir.join("flows");
                if flows_dir.exists() {
                    for entry in fs::read_dir(&flows_dir)
                        .with_context(|| format!("Failed to read flows directory: {:?}", flows_dir))?
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                    {
                        let path = entry.path();
                        match fs::read_to_string(&path)
                            .and_then(|content| serde_json::from_str::<FlowDefinition>(&content)
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
                        {
                            Ok(flow) => {
                                let id = flow.id.clone();
                                self.register_flow(flow)?;
                                report.flows_loaded += 1;
                                tracing::debug!("Loaded user flow: {} from {:?}", id, path);
                            }
                            Err(e) => {
                                report.errors.push(format!("Failed to load flow from {:?}: {}", path, e));
                            }
                        }
                    }
                }
                
                // Load integrations
                let integrations_dir = config_dir.join("integrations");
                if integrations_dir.exists() {
                    for entry in fs::read_dir(&integrations_dir)
                        .with_context(|| format!("Failed to read integrations directory: {:?}", integrations_dir))?
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                    {
                        let path = entry.path();
                        match fs::read_to_string(&path)
                            .and_then(|content| serde_json::from_str::<IntegrationDefinition>(&content)
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
                        {
                            Ok(integration) => {
                                let id = integration.id.clone();
                                self.register_integration(integration)?;
                                report.integrations_loaded += 1;
                                tracing::debug!("Loaded user integration: {} from {:?}", id, path);
                            }
                            Err(e) => {
                                report.errors.push(format!("Failed to load integration from {:?}: {}", path, e));
                            }
                        }
                    }
                }
                
                // Load settings (after flows are loaded so we can validate default_flow_id)
                if let Err(e) = self.load_settings() {
                    tracing::warn!("Failed to load settings: {}", e);
                }
            }
        }
        
        Ok(report)
    }
}

/// Statistics about the registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub agents: usize,
    pub stages: usize,
    pub flows: usize,
    pub skills: usize,
    pub integrations: usize,
}

/// Report of loading user configurations
#[derive(Debug, Clone, Default)]
pub struct LoadUserReport {
    pub agents_loaded: usize,
    pub stages_loaded: usize,
    pub flows_loaded: usize,
    pub integrations_loaded: usize,
    pub errors: Vec<String>,
}

/// Settings that persist across sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Default flow ID to use for iterations
    pub default_flow_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_flow_id: None,
        }
    }
}

// Global registry instance
lazy_static::lazy_static! {
    static ref GLOBAL_REGISTRY: Arc<ConfigRegistry> = Arc::new(ConfigRegistry::new());
}

/// Get the global configuration registry
pub fn global_registry() -> Arc<ConfigRegistry> {
    GLOBAL_REGISTRY.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_definition::agent_definition::AgentDefinition;
    
    #[test]
    fn test_registry_operations() {
        let registry = ConfigRegistry::new();
        
        let agent = AgentDefinition::new("test_agent", "Test Agent", "test instruction");
        registry.register_agent(agent).unwrap();
        
        assert!(registry.get_agent("test_agent").is_some());
        assert!(registry.list_agents().contains(&"test_agent".to_string()));
        
        registry.unregister_agent("test_agent");
        assert!(registry.get_agent("test_agent").is_none());
    }
}
