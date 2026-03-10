// Stage Definition - Data structure for configurable pipeline stages
//
// This replaces hardcoded stage implementations with configuration-driven execution.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

/// Stage type determines execution behavior
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageType {
    /// Single agent execution
    #[default]
    Simple,
    /// Actor-Critic loop pattern
    ActorCritic,
}

/// Hook point for integration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookPoint {
    /// Before stage execution starts
    PreExecute,
    /// After stage execution completes
    PostExecute,
    /// Before HITL confirmation
    PreConfirmation,
    /// After HITL confirmation
    PostConfirmation,
    /// On stage failure
    OnFailure,
}

/// Hook configuration for external integrations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HookConfig {
    /// Integration reference ID
    pub integration_id: String,
    /// Hook point when to execute
    pub point: HookPoint,
    /// Whether to wait for hook completion (blocking)
    #[serde(default)]
    pub blocking: bool,
    /// Timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
    /// Whether to continue on hook failure
    #[serde(default)]
    pub continue_on_failure: bool,
    /// Additional parameters passed to the integration
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

fn default_timeout() -> u32 { 30 }

/// Artifact configuration for stage output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactConfig {
    /// Output file path relative to iteration workspace
    pub path: String,
    /// Whether to require artifact existence after stage
    #[serde(default)]
    pub required: bool,
    /// Human-readable description of the artifact
    pub description: Option<String>,
}

/// Stage definition for configuration-driven pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StageDefinition {
    /// Unique identifier for this stage
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this stage does
    pub description: Option<String>,
    /// Stage type (Simple or ActorCritic)
    #[serde(default)]
    pub stage_type: StageType,
    
    /// Agent reference for simple stages
    /// Format: "agent_id" or "builtin://agent_id"
    pub agent: Option<String>,
    
    /// Actor-Critic configuration for loop stages
    pub actor_critic: Option<ActorCriticStageConfig>,
    
    /// Whether this stage requires HITL confirmation
    #[serde(default)]
    pub needs_confirmation: bool,
    
    /// Output artifacts configuration
    #[serde(default)]
    pub artifacts: Vec<ArtifactConfig>,
    
    /// Integration hooks for this stage
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    
    /// Timeout for stage execution in seconds
    pub timeout_secs: Option<u32>,
    
    /// Retry configuration
    #[serde(default)]
    pub retry: StageRetryConfig,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Metadata for extensions
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Actor-Critic configuration for stage
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ActorCriticStageConfig {
    /// Actor agent reference
    pub actor: String,
    /// Critic agent reference
    pub critic: String,
    /// Maximum loop iterations
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
}

fn default_max_iterations() -> u32 { 1 }

/// Retry configuration for stage execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StageRetryConfig {
    /// Maximum retry attempts
    #[serde(default)]
    pub max_attempts: u32,
    /// Delay between retries in seconds
    #[serde(default)]
    pub delay_secs: u32,
    /// Whether to retry on validation failure
    #[serde(default)]
    pub retry_on_validation_failure: bool,
}

impl Default for StageRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 0,
            delay_secs: 0,
            retry_on_validation_failure: false,
        }
    }
}

impl StageDefinition {
    /// Create a new simple stage definition
    pub fn simple(id: impl Into<String>, name: impl Into<String>, agent: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            stage_type: StageType::Simple,
            agent: Some(agent.into()),
            actor_critic: None,
            needs_confirmation: false,
            artifacts: Vec::new(),
            hooks: Vec::new(),
            timeout_secs: None,
            retry: StageRetryConfig::default(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new Actor-Critic stage definition
    pub fn actor_critic(
        id: impl Into<String>, 
        name: impl Into<String>, 
        actor: impl Into<String>, 
        critic: impl Into<String>
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            stage_type: StageType::ActorCritic,
            agent: None,
            actor_critic: Some(ActorCriticStageConfig {
                actor: actor.into(),
                critic: critic.into(),
                max_iterations: 1,
            }),
            needs_confirmation: false,
            artifacts: Vec::new(),
            hooks: Vec::new(),
            timeout_secs: None,
            retry: StageRetryConfig::default(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Enable HITL confirmation for this stage
    pub fn with_confirmation(mut self) -> Self {
        self.needs_confirmation = true;
        self
    }
    
    /// Add an artifact configuration
    pub fn with_artifact(mut self, path: impl Into<String>, required: bool) -> Self {
        self.artifacts.push(ArtifactConfig {
            path: path.into(),
            required,
            description: None,
        });
        self
    }
    
    /// Add a hook configuration
    pub fn with_hook(mut self, hook: HookConfig) -> Self {
        self.hooks.push(hook);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stage_definition_simple() {
        let stage = StageDefinition::simple("idea", "Idea Stage", "idea_agent")
            .with_confirmation()
            .with_artifact("artifacts/idea.md", true);
        
        let json = serde_json::to_string_pretty(&stage).unwrap();
        println!("{}", json);
        
        let parsed: StageDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "idea");
        assert!(parsed.needs_confirmation);
        assert_eq!(parsed.artifacts.len(), 1);
    }
    
    #[test]
    fn test_stage_definition_actor_critic() {
        let stage = StageDefinition::actor_critic("prd", "PRD Stage", "prd_actor", "prd_critic")
            .with_confirmation();
        
        let json = serde_json::to_string_pretty(&stage).unwrap();
        println!("{}", json);
        
        let parsed: StageDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stage_type, StageType::ActorCritic);
        assert!(parsed.actor_critic.is_some());
    }
}
