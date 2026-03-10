// Skill Loader - Load skills from filesystem
//
// Supports loading from:
// - System skill directory: .cowork-v3/skills/
// - User skill directory: ~/.cowork/skills/
// - Project skill directory: project/.cowork-v3/skills/

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use anyhow::{Result, Context};

use crate::config_definition::{SkillDefinition, SkillManifest, SkillSource, ToolImplementation, SkillPromptType};
use super::validator::SkillValidator;

/// Skill loader for filesystem-based skill loading
pub struct SkillLoader {
    /// Skill directories to search (in priority order)
    skill_dirs: Vec<PathBuf>,
    /// Validator for loaded skills
    validator: SkillValidator,
}

impl SkillLoader {
    /// Create a new loader with default search paths
    pub fn new(project_path: Option<&Path>) -> Self {
        let mut skill_dirs = Vec::new();
        
        // Project-level skills (highest priority)
        if let Some(project) = project_path {
            skill_dirs.push(project.join(".cowork-v3").join("skills"));
        }
        
        // User-level skills
        if let Some(home) = dirs::home_dir() {
            skill_dirs.push(home.join(".cowork").join("skills"));
        }
        
        // System-level skills (built-in)
        if let Some(exe_dir) = std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())) 
        {
            skill_dirs.push(exe_dir.join("skills"));
        }
        
        Self {
            skill_dirs,
            validator: SkillValidator::new(),
        }
    }
    
    /// Create a loader with custom skill directories
    pub fn with_skill_dirs(skill_dirs: Vec<PathBuf>) -> Self {
        Self {
            skill_dirs,
            validator: SkillValidator::new(),
        }
    }
    
    /// Get the skill directories
    pub fn skill_dirs(&self) -> &[PathBuf] {
        &self.skill_dirs
    }
    
    /// Discover all available skills
    pub fn discover_skills(&self) -> Result<Vec<PathBuf>> {
        let mut skill_paths = Vec::new();
        
        for dir in &self.skill_dirs {
            if !dir.exists() {
                continue;
            }
            
            // Each skill is a subdirectory with manifest.json
            for entry in fs::read_dir(dir)
                .with_context(|| format!("Failed to read skill directory: {:?}", dir))?
            {
                let entry = entry?;
                let skill_dir = entry.path();
                
                if skill_dir.is_dir() {
                    let manifest_path = skill_dir.join("manifest.json");
                    if manifest_path.exists() {
                        skill_paths.push(skill_dir);
                    }
                }
            }
        }
        
        Ok(skill_paths)
    }
    
    /// Load a single skill from its directory
    pub fn load_skill(&self, skill_dir: &Path) -> Result<SkillManifest> {
        let manifest_path = skill_dir.join("manifest.json");
        
        // Read manifest
        let content = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read manifest: {:?}", manifest_path))?;
        
        // Parse manifest
        let mut manifest: SkillManifest = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse manifest: {:?}", manifest_path))?;
        
        // Resolve relative paths in skill definition
        self.resolve_skill_paths(&mut manifest.definition, skill_dir)?;
        
        // Validate skill
        let validation = self.validator.validate(&manifest.definition);
        if !validation.is_valid {
            return Err(anyhow::anyhow!(
                "Skill validation failed: {:?}",
                validation.errors
            ));
        }
        
        // Set installation path in source
        if matches!(manifest.source, SkillSource::Local) {
            manifest.installed_at = fs::metadata(&manifest_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.to_rfc3339()
                });
        }
        
        tracing::info!("Loaded skill: {} v{}", manifest.definition.id, manifest.definition.version);
        
        Ok(manifest)
    }
    
    /// Load all skills from configured directories
    pub fn load_all(&self) -> Result<SkillLoadReport> {
        let mut report = SkillLoadReport::default();
        
        for skill_dir in self.discover_skills()? {
            match self.load_skill(&skill_dir) {
                Ok(manifest) => {
                    report.skills.insert(
                        manifest.definition.id.clone(),
                        manifest,
                    );
                    report.loaded_count += 1;
                }
                Err(e) => {
                    report.errors.push(format!(
                        "Failed to load skill from {:?}: {}",
                        skill_dir, e
                    ));
                    tracing::warn!("Failed to load skill from {:?}: {}", skill_dir, e);
                }
            }
        }
        
        // Check skill dependencies
        self.check_dependencies(&mut report);
        
        Ok(report)
    }
    
    /// Resolve relative paths in skill definition
    fn resolve_skill_paths(&self, skill: &mut SkillDefinition, skill_dir: &Path) -> Result<()> {
        // Resolve script paths in tools
        for tool in &mut skill.tools {
            if let ToolImplementation::Script { ref mut script, .. } = tool.implementation {
                let script_path = Path::new(&**script);
                if !script_path.is_absolute() {
                    let resolved = skill_dir
                        .join(&*script)
                        .to_string_lossy()
                        .to_string();
                    *script = resolved;
                }
            }
        }
        
        // Resolve file paths in prompts
        for prompt in &mut skill.prompts {
            if matches!(prompt.prompt_type, SkillPromptType::File) {
                let prompt_path = Path::new(&prompt.content);
                if !prompt_path.is_absolute() {
                    let resolved = skill_dir
                        .join(&prompt.content)
                        .to_string_lossy()
                        .to_string();
                    prompt.content = resolved;
                }
            }
        }
        
        // Resolve context template paths
        for template in &mut skill.context_templates {
            let template_path = Path::new(&**template);
            if !template_path.is_absolute() {
                let resolved = skill_dir
                    .join(&*template)
                    .to_string_lossy()
                    .to_string();
                *template = resolved;
            }
        }
        
        Ok(())
    }
    
    /// Check skill dependencies
    fn check_dependencies(&self, report: &mut SkillLoadReport) {
        let skill_ids: Vec<&str> = report.skills.keys().map(|s| s.as_str()).collect();
        
        for manifest in report.skills.values() {
            for dep in &manifest.definition.dependencies {
                if !skill_ids.contains(&dep.skill_id.as_str()) && !dep.optional {
                    report.errors.push(format!(
                        "Skill '{}' depends on missing skill '{}' (required)",
                        manifest.definition.id, dep.skill_id
                    ));
                }
            }
        }
    }
    
    /// Install a skill from a source directory
    pub fn install_skill(&self, source_dir: &Path, target_dir: &Path) -> Result<String> {
        let manifest_path = source_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(anyhow::anyhow!("Source directory does not contain manifest.json"));
        }
        
        // Load and validate source skill
        let manifest = self.load_skill(source_dir)?;
        let skill_id = manifest.definition.id.clone();
        
        // Create target directory
        let target_skill_dir = target_dir.join(&skill_id);
        fs::create_dir_all(&target_skill_dir)
            .with_context(|| format!("Failed to create target directory: {:?}", target_skill_dir))?;
        
        // Copy all files
        self.copy_dir_all(source_dir, &target_skill_dir)?;
        
        tracing::info!("Installed skill: {} to {:?}", skill_id, target_skill_dir);
        
        Ok(skill_id)
    }
    
    /// Uninstall a skill
    pub fn uninstall_skill(&self, skill_id: &str, skills_dir: &Path) -> Result<()> {
        let skill_dir = skills_dir.join(skill_id);
        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir)
                .with_context(|| format!("Failed to remove skill directory: {:?}", skill_dir))?;
            tracing::info!("Uninstalled skill: {}", skill_id);
        }
        Ok(())
    }
    
    /// Recursively copy a directory
    fn copy_dir_all(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            
            if ty.is_dir() {
                self.copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.join(entry.file_name()))?;
            }
        }
        
        Ok(())
    }
}

/// Report of skill loading results
#[derive(Debug, Clone, Default)]
pub struct SkillLoadReport {
    /// Loaded skills by ID
    pub skills: HashMap<String, SkillManifest>,
    /// Number of successfully loaded skills
    pub loaded_count: usize,
    /// Loading errors
    pub errors: Vec<String>,
}

impl SkillLoadReport {
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Get a loaded skill by ID
    pub fn get_skill(&self, id: &str) -> Option<&SkillManifest> {
        self.skills.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_loader_discovery() {
        let temp = TempDir::new().unwrap();
        let loader = SkillLoader::new(Some(temp.path()));
        
        // Should not fail even with empty directories
        let skills = loader.discover_skills().unwrap();
        assert!(skills.is_empty());
    }
}
