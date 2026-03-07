// Skill Context - Manage skill runtime context and configuration
//
// Provides:
// - Skill configuration management
// - Prompt content resolution
// - Template interpolation

use std::collections::HashMap;
use anyhow::{Result, Context};

use crate::config_definition::{SkillPrompt, SkillPromptType};

/// Prompt injection for agent instructions
#[derive(Debug, Clone)]
pub struct PromptInjection {
    /// Prepended prompts
    pub prepend: Vec<String>,
    /// Appended prompts
    pub append: Vec<String>,
    /// Replacement placeholders
    pub replacements: HashMap<String, String>,
}

impl PromptInjection {
    pub fn new() -> Self {
        Self {
            prepend: Vec::new(),
            append: Vec::new(),
            replacements: HashMap::new(),
        }
    }
    
    /// Apply prompt injections to agent instruction
    pub fn apply(&self, instruction: &str) -> String {
        let mut result = String::new();
        
        // Prepend prompts
        for prompt in &self.prepend {
            result.push_str(prompt);
            result.push_str("\n\n");
        }
        
        // Add original instruction
        result.push_str(instruction);
        
        // Apply replacements
        for (placeholder, content) in &self.replacements {
            result = result.replace(placeholder, content);
        }
        
        // Append prompts
        if !self.append.is_empty() {
            result.push_str("\n\n");
            for prompt in &self.append {
                result.push_str(prompt);
                result.push_str("\n\n");
            }
        }
        
        result
    }
}

/// Skill context for runtime state
#[derive(Debug, Clone)]
pub struct SkillContext {
    /// Skill configuration values
    config: HashMap<String, serde_json::Value>,
    /// Cached prompt contents
    prompt_cache: HashMap<String, String>,
}

impl SkillContext {
    /// Create a new skill context
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
            prompt_cache: HashMap::new(),
        }
    }
    
    /// Get a configuration value
    pub fn get_config<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.config.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    /// Get a configuration value with default
    pub fn get_config_or<T: serde::de::DeserializeOwned + Default>(&self, key: &str) -> T {
        self.get_config(key).unwrap_or_default()
    }
    
    /// Set a configuration value
    pub fn set_config(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.config.insert(key.into(), value);
    }
    
    /// Update configuration from a map
    pub fn update_config(&mut self, config: HashMap<String, serde_json::Value>) {
        self.config.extend(config);
    }
    
    /// Get all configuration values
    pub fn get_all_config(&self) -> &HashMap<String, serde_json::Value> {
        &self.config
    }
    
    /// Resolve prompt content based on prompt type
    pub fn resolve_prompt_content(&mut self, prompt: &SkillPrompt) -> Result<String> {
        // Check cache first
        let cache_key = format!("{:?}:{}", prompt.prompt_type, prompt.content);
        if let Some(cached) = self.prompt_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Resolve based on type
        let content = match prompt.prompt_type {
            SkillPromptType::Inline => {
                prompt.content.clone()
            }
            SkillPromptType::File => {
                // Load from file
                let path = std::path::Path::new(&prompt.content);
                std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read prompt file: {}", prompt.content))?
            }
            SkillPromptType::Url => {
                // Fetch from URL
                // Note: This would require async runtime, so we'll skip for now
                // and return a placeholder
                tracing::warn!("URL prompt type not yet implemented: {}", prompt.content);
                prompt.content.clone()
            }
        };
        
        // Interpolate configuration values
        let interpolated = self.interpolate_config(&content);
        
        // Cache the result
        self.prompt_cache.insert(cache_key.clone(), interpolated.clone());
        
        Ok(interpolated)
    }
    
    /// Interpolate configuration values in content
    fn interpolate_config(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Replace {{config.key}} with config values
        for (key, value) in &self.config {
            let placeholder = format!("{{{{config.{}}}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }
        
        result
    }
    
    /// Clear prompt cache
    pub fn clear_cache(&mut self) {
        self.prompt_cache.clear();
    }
}

impl Default for SkillContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for skill context
pub struct SkillContextBuilder {
    config: HashMap<String, serde_json::Value>,
}

impl SkillContextBuilder {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
    
    /// Add a configuration value
    pub fn with_config_value(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }
    
    /// Add multiple configuration values
    pub fn with_config(mut self, config: HashMap<String, serde_json::Value>) -> Self {
        self.config.extend(config);
        self
    }
    
    /// Build the context
    pub fn build(self) -> SkillContext {
        SkillContext {
            config: self.config,
            prompt_cache: HashMap::new(),
        }
    }
}

impl Default for SkillContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skill_context_config() {
        let mut ctx = SkillContext::new();
        ctx.set_config("model", serde_json::json!("gpt-4"));
        
        let model: String = ctx.get_config("model").unwrap();
        assert_eq!(model, "gpt-4");
    }
    
    #[test]
    fn test_prompt_injection_apply() {
        let mut injection = PromptInjection::new();
        injection.prepend.push("You are a helpful assistant.".to_string());
        injection.append.push("Be concise.".to_string());
        
        let result = injection.apply("Help the user.");
        assert!(result.contains("You are a helpful assistant."));
        assert!(result.contains("Help the user."));
        assert!(result.contains("Be concise."));
    }
    
    #[test]
    fn test_interpolate_config() {
        let mut ctx = SkillContext::new();
        ctx.set_config("language", serde_json::json!("TypeScript"));
        ctx.set_config("framework", serde_json::json!("React"));
        
        let content = "You are an expert in {{config.language}} and {{config.framework}}.";
        let result = ctx.interpolate_config(content);
        
        assert_eq!(result, "You are an expert in TypeScript and React.");
    }
}
