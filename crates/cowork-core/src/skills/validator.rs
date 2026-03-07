// Skill Validator - Validate skill definitions
//
// Validates:
// - Required fields
// - Version format (semver)
// - Tool definitions
// - Prompt definitions
// - Compatibility requirements

use crate::config_definition::{
    SkillDefinition, SkillCategory, ToolImplementation, SkillPromptType,
};
use semver::Version;

/// Validation result for skills
#[derive(Debug, Clone, Default)]
pub struct SkillValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl SkillValidationResult {
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
    
    pub fn merge(&mut self, other: SkillValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Skill validator
pub struct SkillValidator {
    /// Minimum Cowork Forge version supported
    min_cowork_version: Version,
}

impl SkillValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            // Default to current version
            min_cowork_version: Version::parse("3.0.0").unwrap(),
        }
    }
    
    /// Set minimum supported version
    pub fn with_min_version(mut self, version: &str) -> Self {
        if let Ok(v) = Version::parse(version) {
            self.min_cowork_version = v;
        }
        self
    }
    
    /// Validate a skill definition
    pub fn validate(&self, skill: &SkillDefinition) -> SkillValidationResult {
        let mut result = SkillValidationResult::new();
        
        // Validate required fields
        self.validate_required_fields(skill, &mut result);
        
        // Validate version format
        self.validate_version(skill, &mut result);
        
        // Validate tools
        self.validate_tools(skill, &mut result);
        
        // Validate prompts
        self.validate_prompts(skill, &mut result);
        
        // Validate compatibility
        self.validate_compatibility(skill, &mut result);
        
        // Validate metadata
        self.validate_metadata(skill, &mut result);
        
        result
    }
    
    /// Validate required fields
    fn validate_required_fields(&self, skill: &SkillDefinition, result: &mut SkillValidationResult) {
        if skill.id.is_empty() {
            result.error("Skill ID is required");
        }
        
        // Validate ID format (alphanumeric, hyphens, underscores)
        if !skill.id.is_empty() && !self.is_valid_id(&skill.id) {
            result.error(format!(
                "Skill ID '{}' must contain only alphanumeric characters, hyphens, and underscores",
                skill.id
            ));
        }
        
        if skill.name.is_empty() {
            result.error(format!("Skill '{}' name is required", skill.id));
        }
        
        if skill.version.is_empty() {
            result.error(format!("Skill '{}' version is required", skill.id));
        }
        
        if skill.description.is_none() {
            result.warning(format!("Skill '{}' has no description", skill.id));
        }
        
        if skill.author.is_none() {
            result.warning(format!("Skill '{}' has no author specified", skill.id));
        }
    }
    
    /// Validate version format
    fn validate_version(&self, skill: &SkillDefinition, result: &mut SkillValidationResult) {
        if skill.version.is_empty() {
            return; // Already caught in required fields
        }
        
        match Version::parse(&skill.version) {
            Ok(_) => {
                // Valid semver
            }
            Err(_) => {
                result.warning(format!(
                    "Skill '{}' version '{}' is not in semver format (e.g., 1.0.0)",
                    skill.id, skill.version
                ));
            }
        }
        
        // Validate dependency versions
        for dep in &skill.dependencies {
            if let Some(ref version) = dep.version {
                if Version::parse(version).is_err() {
                    result.warning(format!(
                        "Skill '{}' dependency '{}' has invalid version constraint: {}",
                        skill.id, dep.skill_id, version
                    ));
                }
            }
        }
    }
    
    /// Validate tool definitions
    fn validate_tools(&self, skill: &SkillDefinition, result: &mut SkillValidationResult) {
        for (idx, tool) in skill.tools.iter().enumerate() {
            if tool.tool_id.is_empty() {
                result.error(format!(
                    "Skill '{}' tool at index {} has empty ID",
                    skill.id, idx
                ));
            }
            
            if tool.description.is_empty() {
                result.warning(format!(
                    "Skill '{}' tool '{}' has no description",
                    skill.id, tool.tool_id
                ));
            }
            
            // Validate tool implementation
            match &tool.implementation {
                ToolImplementation::Script { interpreter, script } => {
                    if interpreter.is_empty() {
                        result.error(format!(
                            "Skill '{}' tool '{}' has script implementation but no interpreter",
                            skill.id, tool.tool_id
                        ));
                    }
                    if script.is_empty() {
                        result.error(format!(
                            "Skill '{}' tool '{}' has script implementation but no script path",
                            skill.id, tool.tool_id
                        ));
                    }
                }
                ToolImplementation::Http { method, url_template, .. } => {
                    if method.is_empty() {
                        result.error(format!(
                            "Skill '{}' tool '{}' has HTTP implementation but no method",
                            skill.id, tool.tool_id
                        ));
                    }
                    if url_template.is_empty() {
                        result.error(format!(
                            "Skill '{}' tool '{}' has HTTP implementation but no URL template",
                            skill.id, tool.tool_id
                        ));
                    }
                }
                ToolImplementation::Wasm { module, function } => {
                    if module.is_empty() {
                        result.error(format!(
                            "Skill '{}' tool '{}' has WASM implementation but no module",
                            skill.id, tool.tool_id
                        ));
                    }
                    if function.is_empty() {
                        result.error(format!(
                            "Skill '{}' tool '{}' has WASM implementation but no function",
                            skill.id, tool.tool_id
                        ));
                    }
                }
                ToolImplementation::Builtin => {
                    // Built-in tools are always valid
                }
            }
            
            // Validate parameters schema if present
            if let Some(ref schema) = tool.parameters_schema {
                if !schema.is_object() {
                    result.warning(format!(
                        "Skill '{}' tool '{}' parameters_schema should be an object",
                        skill.id, tool.tool_id
                    ));
                }
            }
        }
    }
    
    /// Validate prompt definitions
    fn validate_prompts(&self, skill: &SkillDefinition, result: &mut SkillValidationResult) {
        for (idx, prompt) in skill.prompts.iter().enumerate() {
            if prompt.content.is_empty() {
                result.error(format!(
                    "Skill '{}' prompt at index {} has empty content",
                    skill.id, idx
                ));
            }
            
            // Validate prompt type matches content
            match prompt.prompt_type {
                SkillPromptType::File => {
                    // Content should be a file path
                    if !prompt.content.ends_with(".txt") 
                        && !prompt.content.ends_with(".md")
                        && !prompt.content.ends_with(".prompt")
                    {
                        result.warning(format!(
                            "Skill '{}' prompt at index {} is file type but content doesn't look like a file path",
                            skill.id, idx
                        ));
                    }
                }
                SkillPromptType::Url => {
                    // Content should be a URL
                    if !prompt.content.starts_with("http://") 
                        && !prompt.content.starts_with("https://")
                    {
                        result.warning(format!(
                            "Skill '{}' prompt at index {} is URL type but content doesn't look like a URL",
                            skill.id, idx
                        ));
                    }
                }
                SkillPromptType::Inline => {
                    // Inline prompts are always valid
                }
            }
        }
    }
    
    /// Validate compatibility requirements
    fn validate_compatibility(&self, skill: &SkillDefinition, result: &mut SkillValidationResult) {
        let compat = &skill.compatibility;
        
        // Validate version constraints
        if let Some(ref min_version) = compat.min_cowork_version {
            match Version::parse(min_version) {
                Ok(v) => {
                    if v > self.min_cowork_version {
                        result.warning(format!(
                            "Skill '{}' requires Cowork Forge {} but current version is {}",
                            skill.id, min_version, self.min_cowork_version
                        ));
                    }
                }
                Err(_) => {
                    result.error(format!(
                        "Skill '{}' has invalid min_cowork_version: {}",
                        skill.id, min_version
                    ));
                }
            }
        }
        
        if let Some(ref max_version) = compat.max_cowork_version {
            if Version::parse(max_version).is_err() {
                result.error(format!(
                    "Skill '{}' has invalid max_cowork_version: {}",
                    skill.id, max_version
                ));
            }
        }
        
        // Validate runtime requirements
        for req in &compat.requires {
            if req.is_empty() {
                result.warning(format!(
                    "Skill '{}' has empty runtime requirement",
                    skill.id
                ));
            }
        }
    }
    
    /// Validate metadata
    fn validate_metadata(&self, skill: &SkillDefinition, result: &mut SkillValidationResult) {
        // Validate tags
        for tag in &skill.tags {
            if tag.is_empty() {
                result.warning(format!("Skill '{}' has empty tag", skill.id));
            } else if !self.is_valid_tag(tag) {
                result.warning(format!(
                    "Skill '{}' tag '{}' contains invalid characters",
                    skill.id, tag
                ));
            }
        }
        
        // Validate category
        if matches!(skill.category, SkillCategory::Custom(ref name) if name.is_empty()) {
            result.error(format!("Skill '{}' has custom category with empty name", skill.id));
        }
    }
    
    /// Check if ID is valid
    fn is_valid_id(&self, id: &str) -> bool {
        id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }
    
    /// Check if tag is valid
    fn is_valid_tag(&self, tag: &str) -> bool {
        tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
    }
}

impl Default for SkillValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_definition::SkillTool;
    
    #[test]
    fn test_validate_valid_skill() {
        let validator = SkillValidator::new();
        
        let skill = SkillDefinition::new("web-frontend", "Web Frontend", "1.0.0")
            .in_category(SkillCategory::WebFrontend);
        
        let result = validator.validate(&skill);
        assert!(result.is_valid);
    }
    
    #[test]
    fn test_validate_invalid_skill() {
        let validator = SkillValidator::new();
        
        let mut skill = SkillDefinition::new("", "", "");
        skill.tools.push(SkillTool {
            tool_id: "".to_string(),
            implementation: ToolImplementation::Builtin,
            description: "".to_string(),
            parameters_schema: None,
            enabled: true,
        });
        
        let result = validator.validate(&skill);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
    
    #[test]
    fn test_validate_version_format() {
        let validator = SkillValidator::new();
        
        let skill = SkillDefinition::new("test", "Test", "invalid-version");
        let result = validator.validate(&skill);
        
        assert!(result.is_valid); // Warning, not error
        assert!(!result.warnings.is_empty());
    }
}
