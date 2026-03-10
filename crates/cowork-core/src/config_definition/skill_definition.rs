// Skill Definition - Data structure for agent skill plugins
//
// Skills are domain-specific capabilities that can be loaded into agents.
// They provide tools, prompts, and context for specialized tasks.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

/// Skill definition for domain-specific agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SkillDefinition {
    /// Unique identifier for this skill
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this skill enables
    pub description: Option<String>,
    /// Version (semver)
    pub version: String,
    
    /// Skill category for organization
    #[serde(default)]
    pub category: SkillCategory,
    
    /// Author/maintainer information
    #[serde(default)]
    pub author: Option<String>,
    
    /// Dependencies on other skills
    #[serde(default)]
    pub dependencies: Vec<SkillDependency>,
    
    /// Tools provided by this skill
    #[serde(default)]
    pub tools: Vec<SkillTool>,
    
    /// Prompt extensions for agents using this skill
    #[serde(default)]
    pub prompts: Vec<SkillPrompt>,
    
    /// Context templates loaded when skill is active
    #[serde(default)]
    pub context_templates: Vec<String>,
    
    /// Configuration schema for this skill
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,
    
    /// Default configuration values
    #[serde(default)]
    pub default_config: HashMap<String, serde_json::Value>,
    
    /// Compatibility information
    #[serde(default)]
    pub compatibility: CompatibilityInfo,
    
    /// Tags for discovery
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Metadata for extensions
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Skill category for organization
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    #[default]
    General,
    WebFrontend,
    WebBackend,
    Mobile,
    Desktop,
    DevOps,
    Testing,
    Documentation,
    DataProcessing,
    MachineLearning,
    Security,
    Custom(String),
}

/// Dependency on another skill
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SkillDependency {
    /// Required skill ID
    pub skill_id: String,
    /// Version constraint (semver range)
    pub version: Option<String>,
    /// Whether this is an optional dependency
    #[serde(default)]
    pub optional: bool,
}

/// Tool provided by a skill
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SkillTool {
    /// Tool identifier
    pub tool_id: String,
    /// Tool implementation type
    #[serde(default)]
    pub implementation: ToolImplementation,
    /// Tool description
    pub description: String,
    /// Parameter schema
    #[serde(default)]
    pub parameters_schema: Option<serde_json::Value>,
    /// Whether this tool is enabled by default
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Tool implementation type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolImplementation {
    /// Built-in tool (registered in core)
    #[default]
    Builtin,
    /// Script-based tool (shell/python/node)
    Script {
        /// Interpreter to use
        interpreter: String,
        /// Script path relative to skill directory
        script: String,
    },
    /// HTTP-based tool (calls external API)
    Http {
        /// HTTP method
        method: String,
        /// URL template with parameter interpolation
        url_template: String,
        /// Request headers
        headers: HashMap<String, String>,
    },
    /// WebAssembly tool
    Wasm {
        /// Path to .wasm file
        module: String,
        /// Function to call
        function: String,
    },
}

fn default_true() -> bool { true }

/// Prompt extension for agents
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SkillPrompt {
    /// Prompt type
    #[serde(default)]
    pub prompt_type: SkillPromptType,
    /// Prompt content or template path
    pub content: String,
    /// When to inject this prompt
    #[serde(default)]
    pub injection_point: PromptInjectionPoint,
    /// Priority for ordering (higher = earlier)
    #[serde(default)]
    pub priority: i32,
}

/// Type of skill prompt
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillPromptType {
    /// Inline prompt text
    #[default]
    Inline,
    /// File path relative to skill directory
    File,
    /// URL to fetch prompt from
    Url,
}

/// When to inject skill prompts
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum PromptInjectionPoint {
    /// Prepend to agent instruction
    #[default]
    Prepend,
    /// Append to agent instruction
    Append,
    /// Replace specific placeholder
    Replace { placeholder: String },
}

/// Compatibility information for skills
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct CompatibilityInfo {
    /// Minimum Cowork Forge version
    pub min_cowork_version: Option<String>,
    /// Maximum Cowork Forge version
    pub max_cowork_version: Option<String>,
    /// Required runtime capabilities
    #[serde(default)]
    pub requires: Vec<String>,
    /// Supported platforms
    #[serde(default)]
    pub platforms: Vec<Platform>,
}

/// Supported platform
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[default]
    All,
    Windows,
    Macos,
    Linux,
}

/// Skill manifest file (manifest.json) structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SkillManifest {
    /// Skill definition
    #[serde(flatten)]
    pub definition: SkillDefinition,
    /// Installation source
    #[serde(default)]
    pub source: SkillSource,
    /// Installation timestamp
    #[serde(default)]
    pub installed_at: Option<String>,
    /// Checksum for verification
    #[serde(default)]
    pub checksum: Option<String>,
}

/// Skill installation source
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    #[default]
    Local,
    Registry { registry_url: String },
    Git { repository: String, commit: Option<String> },
}

impl SkillDefinition {
    /// Create a new skill definition
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: version.into(),
            category: SkillCategory::default(),
            author: None,
            dependencies: Vec::new(),
            tools: Vec::new(),
            prompts: Vec::new(),
            context_templates: Vec::new(),
            config_schema: None,
            default_config: HashMap::new(),
            compatibility: CompatibilityInfo::default(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a tool to the skill
    pub fn with_tool(mut self, tool: SkillTool) -> Self {
        self.tools.push(tool);
        self
    }
    
    /// Add a prompt extension
    pub fn with_prompt(mut self, prompt: SkillPrompt) -> Self {
        self.prompts.push(prompt);
        self
    }
    
    /// Add a dependency
    pub fn with_dependency(mut self, skill_id: impl Into<String>, optional: bool) -> Self {
        self.dependencies.push(SkillDependency {
            skill_id: skill_id.into(),
            version: None,
            optional,
        });
        self
    }
    
    /// Set category
    pub fn in_category(mut self, category: SkillCategory) -> Self {
        self.category = category;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skill_definition() {
        let skill = SkillDefinition::new("web-frontend", "Web Frontend Development", "1.0.0")
            .in_category(SkillCategory::WebFrontend)
            .with_tool(SkillTool {
                tool_id: "generate_react_component".to_string(),
                implementation: ToolImplementation::Builtin,
                description: "Generate a React component with TypeScript".to_string(),
                parameters_schema: None,
                enabled: true,
            })
            .with_prompt(SkillPrompt {
                prompt_type: SkillPromptType::Inline,
                content: "You are an expert in React, TypeScript, and modern web development.".to_string(),
                injection_point: PromptInjectionPoint::Prepend,
                priority: 100,
            });
        
        let json = serde_json::to_string_pretty(&skill).unwrap();
        println!("{}", json);
        
        let parsed: SkillDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "web-frontend");
        assert_eq!(parsed.tools.len(), 1);
    }
}
