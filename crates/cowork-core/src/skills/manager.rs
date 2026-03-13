// Skill Manager - High-level skill management for Cowork Forge
//
// Provides:
// - Skill discovery and loading from project directories
// - Skill selection based on user query
// - Skill context injection for LLM prompts
// - Local skill installation from directories

use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

use adk_skill::{
    SkillIndex, SkillDocument, SkillMatch, SelectionPolicy,
    load_skill_index, select_skills,
};

/// Configuration for the skill manager
#[derive(Debug, Clone)]
pub struct SkillManagerConfig {
    /// Root directory for skill discovery (typically project root)
    pub root_path: PathBuf,
    /// Selection policy for matching skills
    pub selection_policy: SelectionPolicy,
    /// Maximum characters to inject from skill body
    pub max_injected_chars: usize,
}

impl Default for SkillManagerConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            selection_policy: SelectionPolicy::default(),
            max_injected_chars: 2000,
        }
    }
}

impl SkillManagerConfig {
    /// Create a new config with the given root path
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
            ..Default::default()
        }
    }

    /// Set the selection policy
    pub fn with_policy(mut self, policy: SelectionPolicy) -> Self {
        self.selection_policy = policy;
        self
    }

    /// Set the maximum injected characters
    pub fn with_max_injected_chars(mut self, max: usize) -> Self {
        self.max_injected_chars = max;
        self
    }
}

/// High-level skill manager for Cowork Forge
///
/// Manages skill discovery, loading, selection, and injection.
/// Uses the agentskills.io standard for skill definitions.
pub struct SkillManager {
    config: SkillManagerConfig,
    index: SkillIndex,
}

impl SkillManager {
    /// Create a new skill manager with the given configuration
    pub fn new(config: SkillManagerConfig) -> Result<Self> {
        let index = load_skill_index(&config.root_path)
            .map_err(|e| anyhow::anyhow!("Failed to load skill index: {}", e))?;

        tracing::info!(
            "SkillManager initialized with {} skills from {:?}",
            index.len(),
            config.root_path
        );

        Ok(Self { config, index })
    }

    /// Create a skill manager for the current project
    pub fn for_project(project_path: impl AsRef<Path>) -> Result<Self> {
        let config = SkillManagerConfig::new(project_path.as_ref());
        Self::new(config)
    }

    /// Get the skill index
    pub fn index(&self) -> &SkillIndex {
        &self.index
    }

    /// Get all skills
    pub fn list_skills(&self) -> &[SkillDocument] {
        self.index.skills()
    }

    /// Get the number of loaded skills
    pub fn skill_count(&self) -> usize {
        self.index.len()
    }

    /// Check if there are any skills loaded
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Find a skill by name (linear search through skills)
    pub fn find_skill(&self, name: &str) -> Option<&SkillDocument> {
        self.index.skills().iter().find(|s| s.name == name)
    }

    /// Find a skill by ID (linear search through skills)
    pub fn find_skill_by_id(&self, id: &str) -> Option<&SkillDocument> {
        self.index.skills().iter().find(|s| s.id == id)
    }

    /// Select skills matching a query
    pub fn select(&self, query: &str) -> Vec<SkillMatch> {
        select_skills(&self.index, query, &self.config.selection_policy)
    }

    /// Select the best matching skill for a query
    pub fn select_best(&self, query: &str) -> Option<SkillMatch> {
        self.select(query).into_iter().next()
    }

    /// Select skills with a custom policy
    pub fn select_with_policy(&self, query: &str, policy: &SelectionPolicy) -> Vec<SkillMatch> {
        select_skills(&self.index, query, policy)
    }

    /// Get skill summaries for display/UI purposes
    pub fn get_summaries(&self) -> Vec<adk_skill::SkillSummary> {
        self.index.summaries()
    }

    /// Reload the skill index from disk
    pub fn reload(&mut self) -> Result<()> {
        self.index = load_skill_index(&self.config.root_path)
            .map_err(|e| anyhow::anyhow!("Failed to reload skill index: {}", e))?;

        tracing::info!(
            "SkillManager reloaded with {} skills",
            self.index.len()
        );

        Ok(())
    }

    /// Install a skill from a local directory
    ///
    /// Copies the skill directory to the project's .skills directory
    pub fn install_skill_from_dir(&self, source_dir: &Path) -> Result<String> {
        // Read the SKILL.md file to get the skill name
        let skill_md_path = source_dir.join("SKILL.md");
        if !skill_md_path.exists() {
            anyhow::bail!("Source directory does not contain SKILL.md: {:?}", source_dir);
        }

        let content = std::fs::read_to_string(&skill_md_path)
            .with_context(|| format!("Failed to read {:?}", skill_md_path))?;

        // Parse to get skill name
        let parsed = adk_skill::parse_skill_markdown(&skill_md_path, &content)
            .map_err(|e| anyhow::anyhow!("Failed to parse SKILL.md: {}", e))?;

        let skill_name = parsed.name.clone();

        // Create target directory
        let target_dir = self.config.root_path.join(".skills").join(&skill_name);
        std::fs::create_dir_all(&target_dir)
            .with_context(|| format!("Failed to create target directory: {:?}", target_dir))?;

        // Copy all files from source to target
        copy_dir_all(source_dir, &target_dir)?;

        tracing::info!("Installed skill '{}' to {:?}", skill_name, target_dir);

        Ok(skill_name)
    }

    /// Get the path to the project's .skills directory
    pub fn skills_directory(&self) -> PathBuf {
        self.config.root_path.join(".skills")
    }

    /// Check if a skill with the given name exists
    pub fn has_skill(&self, name: &str) -> bool {
        self.index.skills().iter().any(|s| s.name == name)
    }
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_skill_manager_empty() {
        let temp = TempDir::new().unwrap();
        let config = SkillManagerConfig::new(temp.path());
        let manager = SkillManager::new(config).unwrap();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_skill_manager_with_skill() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        fs::create_dir_all(&skills_dir).unwrap();

        fs::write(
            skills_dir.join("test.md"),
            "---\nname: test\ndescription: A test skill\n---\nTest instructions.",
        ).unwrap();

        let config = SkillManagerConfig::new(temp.path());
        let manager = SkillManager::new(config).unwrap();

        assert_eq!(manager.skill_count(), 1);
        assert!(manager.find_skill("test").is_some());
    }

    #[test]
    fn test_skill_selection() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        fs::create_dir_all(&skills_dir).unwrap();

        fs::write(
            skills_dir.join("search.md"),
            "---\nname: search\ndescription: Search code\n---\nUse ripgrep.",
        ).unwrap();

        let config = SkillManagerConfig::new(temp.path())
            .with_policy(SelectionPolicy {
                min_score: 0.1,
                ..Default::default()
            });

        let manager = SkillManager::new(config).unwrap();

        let matches = manager.select("search the code");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].skill.name, "search");
    }
}