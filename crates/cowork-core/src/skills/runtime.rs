// Skill Runtime - Manage loaded skills and execute skill tools
//
// Provides:
// - Skill loading and lifecycle management
// - Tool execution for skill-provided tools
// - Prompt aggregation from loaded skills

use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

use crate::config_definition::{
    SkillDefinition, SkillTool, SkillPrompt, PromptInjectionPoint,
};
use super::loader::{SkillLoader, SkillLoadReport};
use super::context::{SkillContext, SkillContextBuilder};

/// Loaded skill with runtime state
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    /// Skill definition
    pub definition: SkillDefinition,
    /// Skill directory path
    pub skill_dir: PathBuf,
    /// Skill context
    pub context: SkillContext,
    /// Whether this skill is active
    pub is_active: bool,
}

/// Skill runtime error types
#[derive(Debug, thiserror::Error)]
pub enum SkillRuntimeError {
    #[error("Skill not found: {0}")]
    SkillNotFound(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),
    
    #[error("Skill dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Skill runtime manager
pub struct SkillRuntime {
    /// Loaded skills by ID
    skills: HashMap<String, LoadedSkill>,
    /// Active skill IDs
    active_skills: Vec<String>,
    /// Skill loader
    loader: SkillLoader,
}

impl SkillRuntime {
    /// Create a new skill runtime
    pub fn new(project_path: Option<&std::path::Path>) -> Self {
        let loader = SkillLoader::new(project_path);
        Self {
            skills: HashMap::new(),
            active_skills: Vec::new(),
            loader,
        }
    }
    
    /// Load skills from configured directories
    pub fn load_skills(&mut self) -> Result<SkillLoadReport> {
        let report = self.loader.load_all()?;
        
        // Convert loaded manifests to LoadedSkill
        for (id, manifest) in &report.skills {
            let loaded_skill = LoadedSkill {
                definition: manifest.definition.clone(),
                skill_dir: PathBuf::new(), // TODO: track directory
                context: SkillContextBuilder::new()
                    .with_config(manifest.definition.default_config.clone())
                    .build(),
                is_active: false,
            };
            
            self.skills.insert(id.clone(), loaded_skill);
        }
        
        Ok(report)
    }
    
    /// Activate a skill
    pub fn activate_skill(&mut self, skill_id: &str) -> Result<(), SkillRuntimeError> {
        // Check if skill exists and collect dependencies
        let dependencies = if let Some(skill) = self.skills.get(skill_id) {
            skill.definition.dependencies.clone()
        } else {
            return Err(SkillRuntimeError::SkillNotFound(skill_id.to_string()));
        };
        
        // Activate dependencies first
        for dep in &dependencies {
            if dep.optional {
                continue;
            }
            
            if !self.active_skills.contains(&dep.skill_id) {
                // Try to activate dependency
                self.activate_skill(&dep.skill_id)?;
            }
        }
        
        // Now activate this skill
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.is_active = true;
            if !self.active_skills.contains(&skill_id.to_string()) {
                self.active_skills.push(skill_id.to_string());
            }
            
            tracing::info!("Activated skill: {}", skill_id);
        }
        
        Ok(())
    }
    
    /// Deactivate a skill
    pub fn deactivate_skill(&mut self, skill_id: &str) -> Result<(), SkillRuntimeError> {
        // Check if other skills depend on this one
        let dependent_skills: Vec<String> = self.skills.iter()
            .filter(|(id, _)| *id != skill_id)
            .filter_map(|(id, other_skill)| {
                let depends = other_skill.definition.dependencies.iter()
                    .any(|dep| &dep.skill_id == skill_id && !dep.optional && other_skill.is_active);
                
                if depends {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        
        if !dependent_skills.is_empty() {
            return Err(SkillRuntimeError::DependencyNotSatisfied(format!(
                "Cannot deactivate '{}' because active skill(s) depend on it: {:?}",
                skill_id, dependent_skills
            )));
        }
        
        // Deactivate the skill
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.is_active = false;
            self.active_skills.retain(|id| id != skill_id);
            
            tracing::info!("Deactivated skill: {}", skill_id);
            Ok(())
        } else {
            Err(SkillRuntimeError::SkillNotFound(skill_id.to_string()))
        }
    }
    
    /// Get a loaded skill by ID
    pub fn get_skill(&self, skill_id: &str) -> Option<&LoadedSkill> {
        self.skills.get(skill_id)
    }
    
    /// List all loaded skills
    pub fn list_skills(&self) -> Vec<&LoadedSkill> {
        self.skills.values().collect()
    }
    
    /// List active skills
    pub fn list_active_skills(&self) -> Vec<&LoadedSkill> {
        self.active_skills.iter()
            .filter_map(|id| self.skills.get(id))
            .collect()
    }
    
    /// Get all tools from active skills
    pub fn get_active_tools(&self) -> Vec<(&SkillTool, &LoadedSkill)> {
        self.active_skills.iter()
            .filter_map(|id| self.skills.get(id))
            .flat_map(|skill| {
                skill.definition.tools.iter()
                    .filter(|t| t.enabled)
                    .map(move |t| (t, skill))
            })
            .collect()
    }
    
    /// Get tool by ID from active skills
    pub fn get_tool(&self, tool_id: &str) -> Option<(&SkillTool, &LoadedSkill)> {
        self.active_skills.iter()
            .filter_map(|id| self.skills.get(id))
            .find_map(|skill| {
                skill.definition.tools.iter()
                    .find(|t| t.tool_id == tool_id && t.enabled)
                    .map(|t| (t, skill))
            })
    }
    
    /// Get all prompts from active skills for injection
    pub fn get_prompts_for_injection(&self) -> Vec<(&SkillPrompt, &LoadedSkill)> {
        self.active_skills.iter()
            .filter_map(|id| self.skills.get(id))
            .flat_map(|skill| {
                skill.definition.prompts.iter()
                    .map(move |p| (p, skill))
            })
            .collect()
    }
    
    /// Aggregate prompts for agent instruction
    pub fn aggregate_prompts(&self) -> (String, String) {
        let mut prepend_prompts = Vec::new();
        let mut append_prompts = Vec::new();
        
        // Collect prompts and their content without mutably borrowing
        for (prompt, skill) in self.get_prompts_for_injection() {
            // Clone context to resolve prompts (this is inefficient but safe)
            let mut context = skill.context.clone();
            let content = match context.resolve_prompt_content(prompt) {
                Ok(content) => content.clone(),
                Err(e) => {
                    tracing::warn!("Failed to resolve prompt content: {}", e);
                    prompt.content.clone()
                }
            };
            
            match &prompt.injection_point {
                PromptInjectionPoint::Prepend => {
                    prepend_prompts.push((prompt.priority, content));
                }
                PromptInjectionPoint::Append => {
                    append_prompts.push((prompt.priority, content));
                }
                PromptInjectionPoint::Replace { .. } => {
                    // Handle replacement separately
                }
            }
        }
        
        // Sort by priority (higher = earlier)
        prepend_prompts.sort_by(|a, b| b.0.cmp(&a.0));
        append_prompts.sort_by(|a, b| b.0.cmp(&a.0));
        
        let prepend = prepend_prompts.iter()
            .map(|(_, content)| content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let append = append_prompts.iter()
            .map(|(_, content)| content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        
        (prepend, append)
    }
    
    /// Update skill context configuration
    pub fn update_skill_config(
        &mut self,
        skill_id: &str,
        config: HashMap<String, serde_json::Value>,
    ) -> Result<(), SkillRuntimeError> {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.context.update_config(config);
            Ok(())
        } else {
            Err(SkillRuntimeError::SkillNotFound(skill_id.to_string()))
        }
    }
    
    /// Install a new skill
    pub fn install_skill(&mut self, source_dir: &std::path::Path, skills_dir: &std::path::Path) -> Result<String> {
        let skill_id = self.loader.install_skill(source_dir, skills_dir)?;
        
        // Reload skills
        self.load_skills()?;
        
        Ok(skill_id)
    }
    
    /// Uninstall a skill
    pub fn uninstall_skill(&mut self, skill_id: &str, skills_dir: &std::path::Path) -> Result<()> {
        // Deactivate if active
        if self.active_skills.contains(&skill_id.to_string()) {
            self.deactivate_skill(skill_id)?;
        }
        
        // Remove from runtime
        self.skills.remove(skill_id);
        
        // Uninstall from filesystem
        self.loader.uninstall_skill(skill_id, skills_dir)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_definition::SkillCategory;
    
    #[test]
    fn test_skill_runtime_creation() {
        let runtime = SkillRuntime::new(None);
        assert!(runtime.list_skills().is_empty());
    }
    
    #[test]
    fn test_activate_nonexistent_skill() {
        let mut runtime = SkillRuntime::new(None);
        let result = runtime.activate_skill("nonexistent");
        assert!(result.is_err());
        match result {
            Err(SkillRuntimeError::SkillNotFound(id)) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected SkillNotFound error"),
        }
    }
    
    #[test]
    fn test_deactivate_nonexistent_skill() {
        let mut runtime = SkillRuntime::new(None);
        let result = runtime.deactivate_skill("nonexistent");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_loaded_skill_creation() {
        let definition = SkillDefinition::new("test-skill", "Test Skill", "1.0.0")
            .in_category(SkillCategory::WebFrontend);
        
        let loaded_skill = LoadedSkill {
            definition,
            skill_dir: PathBuf::from("/test"),
            context: SkillContextBuilder::new().build(),
            is_active: false,
        };
        
        assert_eq!(loaded_skill.definition.id, "test-skill");
        assert!(!loaded_skill.is_active);
    }
    
    #[test]
    fn test_skill_runtime_error_display() {
        let err = SkillRuntimeError::SkillNotFound("test".to_string());
        assert!(err.to_string().contains("test"));
        
        let err = SkillRuntimeError::ToolNotFound("tool-1".to_string());
        assert!(err.to_string().contains("tool-1"));
        
        let err = SkillRuntimeError::DependencyNotSatisfied("dep-1".to_string());
        assert!(err.to_string().contains("dep-1"));
    }
    
    #[test]
    fn test_get_tool_from_empty_runtime() {
        let runtime = SkillRuntime::new(None);
        let result = runtime.get_tool("any-tool");
        assert!(result.is_none());
    }
    
    #[test]
    fn test_aggregate_prompts_empty() {
        let runtime = SkillRuntime::new(None);
        let (prepend, append) = runtime.aggregate_prompts();
        assert!(prepend.is_empty());
        assert!(append.is_empty());
    }
    
    #[test]
    fn test_list_active_skills_empty() {
        let runtime = SkillRuntime::new(None);
        let active = runtime.list_active_skills();
        assert!(active.is_empty());
    }
}
