// Agent Definition - Data structure for configurable agent creation
//
// This replaces hardcoded agent builders with configuration-driven instantiation.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

/// Agent type determines the execution pattern
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    /// Single agent that executes once
    #[default]
    Simple,
    /// Loop agent with Actor-Critic pattern
    Loop {
        /// Maximum iterations for the loop (default: 1)
        max_iterations: Option<u32>,
    },
}

/// Model configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelConfig {
    /// Model identifier (e.g., "gpt-4", "claude-3-opus")
    pub model_id: Option<String>,
    /// Temperature for sampling (0.0 - 2.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_id: None,
            temperature: Some(0.7),
            max_tokens: None,
            top_p: None,
        }
    }
}

/// Tool reference in agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolReference {
    /// Tool identifier (e.g., "read_file", "write_file")
    pub tool_id: String,
    /// Optional configuration for the tool
    pub config: Option<HashMap<String, serde_json::Value>>,
}

/// Agent definition for configuration-driven agent creation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentDefinition {
    /// Unique identifier for this agent
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this agent does
    pub description: Option<String>,
    /// Version of this definition (semver)
    pub version: Option<String>,
    
    /// Agent type (Simple or Loop)
    #[serde(default)]
    pub agent_type: AgentType,
    
    /// Prompt template path or inline content
    /// Can reference: 
    /// - Built-in: "builtin://idea_actor"
    /// - File: "file://./prompts/idea_actor.md"
    /// - Inline: "inline://..."
    pub instruction: String,
    
    /// List of tools this agent can use
    #[serde(default)]
    pub tools: Vec<ToolReference>,
    
    /// Model configuration (overrides global default)
    #[serde(default)]
    pub model: ModelConfig,
    
    /// Content inclusion mode for context
    #[serde(default)]
    pub include_contents: IncludeContentsMode,
    
    /// Tags for categorization and skill matching
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Metadata for extensions
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Content inclusion mode for agent context
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum IncludeContentsMode {
    /// No content included
    #[default]
    None,
    /// Include all content
    All,
    /// Include only specified content types
    Selected(Vec<String>),
}

/// Actor-Critic pair definition for Loop agents
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ActorCriticDefinition {
    /// Actor agent definition
    pub actor: AgentDefinition,
    /// Critic agent definition
    pub critic: AgentDefinition,
    /// Maximum loop iterations
    pub max_iterations: Option<u32>,
}

impl AgentDefinition {
    /// Create a new agent definition with minimal required fields
    pub fn new(id: impl Into<String>, name: impl Into<String>, instruction: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: None,
            agent_type: AgentType::Simple,
            instruction: instruction.into(),
            tools: Vec::new(),
            model: ModelConfig::default(),
            include_contents: IncludeContentsMode::None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a tool reference
    pub fn with_tool(mut self, tool_id: impl Into<String>) -> Self {
        self.tools.push(ToolReference {
            tool_id: tool_id.into(),
            config: None,
        });
        self
    }
    
    /// Add a tool with configuration
    pub fn with_tool_config(mut self, tool_id: impl Into<String>, config: HashMap<String, serde_json::Value>) -> Self {
        self.tools.push(ToolReference {
            tool_id: tool_id.into(),
            config: Some(config),
        });
        self
    }
    
    /// Add a tag for skill matching
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Set the agent type to Loop
    pub fn as_loop(mut self, max_iterations: Option<u32>) -> Self {
        self.agent_type = AgentType::Loop { max_iterations };
        self
    }
    
    /// Set model configuration
    pub fn with_model(mut self, model: ModelConfig) -> Self {
        self.model = model;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_definition_serialization() {
        let agent = AgentDefinition::new("idea_agent", "Idea Agent", "builtin://idea_actor")
            .with_tool("save_idea")
            .with_tool("query_memory");
        
        let json = serde_json::to_string_pretty(&agent).unwrap();
        println!("{}", json);
        
        let parsed: AgentDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "idea_agent");
        assert_eq!(parsed.tools.len(), 2);
    }
}
