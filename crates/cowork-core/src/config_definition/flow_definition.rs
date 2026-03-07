// Flow Definition - Data structure for configurable pipeline flows
//
// This replaces hardcoded 7-stage pipeline with configurable flows.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

/// Flow definition for configuration-driven pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlowDefinition {
    /// Unique identifier for this flow
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this flow does
    pub description: Option<String>,
    /// Version of this definition (semver)
    pub version: Option<String>,

    /// Ordered list of stage references
    pub stages: Vec<StageReference>,

    /// Default starting stage (by index or ID)
    #[serde(default)]
    pub start_stage: Option<String>,

    /// Global hooks applied to all stages in this flow
    #[serde(default)]
    pub global_hooks: Vec<GlobalHookConfig>,

    /// Flow-level configuration
    #[serde(default)]
    pub config: FlowConfig,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Metadata for extensions
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Reference to a stage within a flow
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StageReference {
    /// Stage definition ID
    pub stage_id: String,
    /// Optional alias for this stage in this flow
    pub alias: Option<String>,
    /// Stage-specific overrides
    #[serde(default)]
    pub overrides: StageOverrides,
    /// Condition for stage execution
    #[serde(default)]
    pub condition: Option<String>,
    /// Next stage on success (by ID or alias)
    pub on_success: Option<String>,
    /// Next stage on failure (by ID or alias)
    pub on_failure: Option<String>,
}

/// Stage-specific overrides within a flow
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct StageOverrides {
    /// Override confirmation requirement
    pub needs_confirmation: Option<bool>,
    /// Additional hooks for this stage
    pub hooks: Vec<super::stage_definition::HookConfig>,
    /// Override timeout
    pub timeout_secs: Option<u32>,
    /// Skip this stage
    #[serde(default)]
    pub skip: bool,
}

/// Global hook configuration for flow
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GlobalHookConfig {
    /// Integration reference ID
    pub integration_id: String,
    /// Which hook points to apply to
    pub points: Vec<super::stage_definition::HookPoint>,
    /// Default blocking behavior
    #[serde(default)]
    pub blocking: bool,
    /// Default timeout
    #[serde(default = "default_global_timeout")]
    pub timeout_secs: u32,
}

fn default_global_timeout() -> u32 { 30 }

/// Flow-level configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlowConfig {
    /// Whether to stop on first failure
    #[serde(default = "default_stop_on_failure")]
    pub stop_on_failure: bool,

    /// Maximum total execution time in seconds
    pub max_total_time_secs: Option<u32>,

    /// Whether to save state on interruption
    #[serde(default = "default_save_state")]
    pub save_state_on_interrupt: bool,

    /// Memory scope for this flow
    #[serde(default)]
    pub memory_scope: MemoryScope,

    /// Inheritance behavior for evolution iterations
    #[serde(default)]
    pub inheritance: InheritanceConfig,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            stop_on_failure: true,
            max_total_time_secs: None,
            save_state_on_interrupt: true,
            memory_scope: MemoryScope::default(),
            inheritance: InheritanceConfig::default(),
        }
    }
}

fn default_stop_on_failure() -> bool { true }
fn default_save_state() -> bool { true }

/// Memory scope configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    /// Use project-level memory only
    Project,
    /// Use iteration-level memory only
    Iteration,
    /// Merge project and iteration memory
    #[default]
    Merged,
}

/// Inheritance configuration for evolution iterations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InheritanceConfig {
    /// Default inheritance mode
    #[serde(default)]
    pub default_mode: InheritanceMode,
    /// Stage to start from for each inheritance mode
    #[serde(default)]
    pub stage_mapping: HashMap<String, String>,
}

impl Default for InheritanceConfig {
    fn default() -> Self {
        let mut stage_mapping = HashMap::new();
        stage_mapping.insert("none".to_string(), "idea".to_string());
        stage_mapping.insert("partial".to_string(), "plan".to_string());
        stage_mapping.insert("full".to_string(), "idea".to_string());

        Self {
            default_mode: InheritanceMode::Partial,
            stage_mapping,
        }
    }
}

/// Inheritance mode for evolution iterations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum InheritanceMode {
    /// No inheritance, start fresh
    None,
    /// Inherit code only
    #[default]
    Partial,
    /// Inherit code and artifacts
    Full,
}

impl FlowDefinition {
    /// Create a new flow definition
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: None,
            stages: Vec::new(),
            start_stage: None,
            global_hooks: Vec::new(),
            config: FlowConfig::default(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a stage to the flow
    pub fn with_stage(mut self, stage_id: impl Into<String>) -> Self {
        self.stages.push(StageReference {
            stage_id: stage_id.into(),
            alias: None,
            overrides: StageOverrides::default(),
            condition: None,
            on_success: None,
            on_failure: None,
        });
        self
    }

    /// Add a stage with alias
    pub fn with_stage_alias(mut self, stage_id: impl Into<String>, alias: impl Into<String>) -> Self {
        self.stages.push(StageReference {
            stage_id: stage_id.into(),
            alias: Some(alias.into()),
            overrides: StageOverrides::default(),
            condition: None,
            on_success: None,
            on_failure: None,
        });
        self
    }

    /// Set the starting stage
    pub fn start_at(mut self, stage: impl Into<String>) -> Self {
        self.start_stage = Some(stage.into());
        self
    }

    /// Get the default flow
    pub fn default_v3() -> Self {
        Self::new("default", "Default Development Flow")
            .with_stage("idea")
            .with_stage("prd")
            .with_stage("design")
            .with_stage("plan")
            .with_stage("coding")
            .with_stage("check")
            .with_stage("delivery")
            .start_at("idea")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_definition() {
        let flow = FlowDefinition::default_v3();

        let json = serde_json::to_string_pretty(&flow).unwrap();
        println!("{}", json);

        let parsed: FlowDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "default");
        assert_eq!(parsed.stages.len(), 7);
        assert_eq!(parsed.start_stage, Some("idea".to_string()));
    }
}
