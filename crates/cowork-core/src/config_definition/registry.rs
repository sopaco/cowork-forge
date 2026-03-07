// Configuration Registry - Central registry for all configuration definitions
//
// The registry manages Agent, Stage, Flow, Skill, and Integration definitions.
// It provides lookup, validation, and lifecycle management.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, Context};

use super::agent_definition::AgentDefinition;
use super::stage_definition::StageDefinition;
use super::flow_definition::FlowDefinition;
use super::skill_definition::SkillDefinition;
use super::integration_definition::IntegrationDefinition;

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
        let mut default_flow = self.default_flow.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        *default_flow = id;
        Ok(())
    }
    
    /// Get the default flow
    pub fn get_default_flow(&self) -> Option<FlowDefinition> {
        let default_flow = self.default_flow.read().ok()?;
        let flow_id = default_flow.as_ref()?;
        self.get_flow(flow_id)
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
