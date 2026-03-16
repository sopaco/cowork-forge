// Configuration Validator - Validate configuration definitions
//
// Validates:
// - Required fields
// - Reference integrity (e.g., stage references existing agent)
// - Schema compliance
//
// Note: Skills are validated by adk-skill module

use super::agent_definition::AgentDefinition;
use super::stage_definition::{StageDefinition, StageType};
use super::flow_definition::FlowDefinition;
use super::integration_definition::IntegrationDefinition;
use super::registry::ConfigRegistry;
use std::collections::HashSet;

/// Validation result with warnings and errors
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
        self.is_valid = false;
    }
    
    pub fn warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }
    
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Configuration validator
pub struct ConfigValidator<'a> {
    registry: &'a ConfigRegistry,
}

impl<'a> ConfigValidator<'a> {
    /// Create a new validator with registry reference
    pub fn new(registry: &'a ConfigRegistry) -> Self {
        Self { registry }
    }
    
    /// Validate all configurations in the registry
    pub fn validate_all(&self) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Validate all agents
        for id in self.registry.list_agents() {
            if let Some(agent) = self.registry.get_agent(&id) {
                let agent_result = self.validate_agent(&agent);
                result.merge(agent_result);
            }
        }
        
        // Validate all stages
        for id in self.registry.list_stages() {
            if let Some(stage) = self.registry.get_stage(&id) {
                let stage_result = self.validate_stage(&stage);
                result.merge(stage_result);
            }
        }
        
        // Validate all flows
        for id in self.registry.list_flows() {
            if let Some(flow) = self.registry.get_flow(&id) {
                let flow_result = self.validate_flow(&flow);
                result.merge(flow_result);
            }
        }
        
        // Validate all integrations
        for id in self.registry.list_integrations() {
            if let Some(integration) = self.registry.get_integration(&id) {
                let integration_result = self.validate_integration(&integration);
                result.merge(integration_result);
            }
        }
        
        result
    }
    
    /// Validate an agent definition
    pub fn validate_agent(&self, agent: &AgentDefinition) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check required fields
        if agent.id.is_empty() {
            result.error("Agent ID is required");
        }
        
        if agent.name.is_empty() {
            result.error(format!("Agent '{}' name is required", agent.id));
        }
        
        if agent.instruction.is_empty() {
            result.error(format!("Agent '{}' instruction is required", agent.id));
        }
        
        // Validate instruction reference
        if !agent.instruction.starts_with("builtin://") 
            && !agent.instruction.starts_with("file://")
            && !agent.instruction.starts_with("inline://") 
        {
            result.warning(format!(
                "Agent '{}' instruction '{}' does not use a recognized protocol (builtin://, file://, inline://)",
                agent.id, agent.instruction
            ));
        }
        
        // Validate model configuration
        if let Some(temp) = agent.model.temperature {
            if temp < 0.0 || temp > 2.0 {
                result.error(format!(
                    "Agent '{}' temperature {} is out of range [0.0, 2.0]",
                    agent.id, temp
                ));
            }
        }
        
        result
    }
    
    /// Validate a stage definition
    pub fn validate_stage(&self, stage: &StageDefinition) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check required fields
        if stage.id.is_empty() {
            result.error("Stage ID is required");
        }
        
        if stage.name.is_empty() {
            result.error(format!("Stage '{}' name is required", stage.id));
        }
        
        // Validate agent references based on stage type
        match &stage.stage_type {
            StageType::Simple => {
                if stage.agent.is_none() {
                    result.error(format!("Stage '{}' is Simple type but has no agent reference", stage.id));
                } else if let Some(ref agent_id) = stage.agent {
                    if self.registry.get_agent(agent_id).is_none() {
                        result.error(format!(
                            "Stage '{}' references unknown agent: {}",
                            stage.id, agent_id
                        ));
                    }
                }
            }
            StageType::ActorCritic => {
                if let Some(ref ac) = stage.actor_critic {
                    if self.registry.get_agent(&ac.actor).is_none() {
                        result.error(format!(
                            "Stage '{}' references unknown actor agent: {}",
                            stage.id, ac.actor
                        ));
                    }
                    if self.registry.get_agent(&ac.critic).is_none() {
                        result.error(format!(
                            "Stage '{}' references unknown critic agent: {}",
                            stage.id, ac.critic
                        ));
                    }
                } else {
                    result.error(format!("Stage '{}' is ActorCritic type but has no actor_critic config", stage.id));
                }
            }
        }
        
        // Validate hook integration references
        for hook in &stage.hooks {
            if self.registry.get_integration(&hook.integration_id).is_none() {
                result.error(format!(
                    "Stage '{}' hook references unknown integration: {}",
                    stage.id, hook.integration_id
                ));
            }
        }
        
        result
    }
    
    /// Validate a flow definition
    pub fn validate_flow(&self, flow: &FlowDefinition) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check required fields
        if flow.id.is_empty() {
            result.error("Flow ID is required");
        }
        
        if flow.name.is_empty() {
            result.error(format!("Flow '{}' name is required", flow.id));
        }
        
        if flow.stages.is_empty() {
            result.error(format!("Flow '{}' has no stages", flow.id));
        }
        
        // Validate stage references
        let mut stage_ids = HashSet::new();
        let mut stage_aliases = HashSet::new();
        
        for stage_ref in &flow.stages {
            // Check stage exists
            if self.registry.get_stage(&stage_ref.stage_id).is_none() {
                result.error(format!(
                    "Flow '{}' references unknown stage: {}",
                    flow.id, stage_ref.stage_id
                ));
            }
            
            // Check for duplicate stage IDs
            if !stage_ids.insert(&stage_ref.stage_id) {
                result.warning(format!(
                    "Flow '{}' has duplicate stage reference: {}",
                    flow.id, stage_ref.stage_id
                ));
            }
            
            // Track aliases
            if let Some(ref alias) = stage_ref.alias {
                if !stage_aliases.insert(alias) {
                    result.error(format!(
                        "Flow '{}' has duplicate stage alias: {}",
                        flow.id, alias
                    ));
                }
            }
        }
        
        // Validate start stage
        if let Some(ref start) = flow.start_stage {
            let found = flow.stages.iter().any(|s| {
                &s.stage_id == start || s.alias.as_ref() == Some(start)
            });
            if !found {
                result.error(format!(
                    "Flow '{}' start_stage '{}' not found in stages",
                    flow.id, start
                ));
            }
        }
        
        // Validate global hooks
        for hook in &flow.global_hooks {
            if self.registry.get_integration(&hook.integration_id).is_none() {
                result.error(format!(
                    "Flow '{}' global hook references unknown integration: {}",
                    flow.id, hook.integration_id
                ));
            }
        }
        
        result
    }
    
    /// Validate an integration definition
    pub fn validate_integration(&self, integration: &IntegrationDefinition) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check required fields
        if integration.id.is_empty() {
            result.error("Integration ID is required");
        }
        
        if integration.name.is_empty() {
            result.error(format!("Integration '{}' name is required", integration.id));
        }
        
        // Validate REST API integration
        if matches!(integration.integration_type, super::integration_definition::IntegrationType::RestApi) {
            if integration.connection.base_url.is_none() && integration.connection.endpoints.is_empty() {
                result.error(format!(
                    "Integration '{}' is REST API type but has no base_url or endpoints",
                    integration.id
                ));
            }
        }
        
        // Validate authentication
        if let Some(ref auth) = integration.auth {
            match &auth.credentials {
                super::integration_definition::CredentialSource::Static { value: _ } => {
                    result.warning(format!(
                        "Integration '{}' uses static credentials (not recommended for production)",
                        integration.id
                    ));
                }
                super::integration_definition::CredentialSource::EnvVar { name } => {
                    // Could validate env var exists, but might not be set yet
                    tracing::debug!("Integration '{}' expects env var: {}", integration.id, name);
                }
                _ => {}
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_definition::agent_definition::AgentDefinition;
    
    #[test]
    fn test_validate_agent() {
        let registry = ConfigRegistry::new();
        let validator = ConfigValidator::new(&registry);
        
        let valid_agent = AgentDefinition::new("test", "Test", "builtin://test");
        let result = validator.validate_agent(&valid_agent);
        assert!(result.is_valid);
        
        let invalid_agent = AgentDefinition::new("", "", "");
        let result = validator.validate_agent(&invalid_agent);
        assert!(!result.is_valid);
    }
}