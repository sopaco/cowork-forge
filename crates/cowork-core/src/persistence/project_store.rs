use std::path::PathBuf;

use crate::domain::{IterationSummary, Project};

use super::get_cowork_dir;

const PROJECT_FILE: &str = "project.json";
const ITERATIONS_FILE: &str = "iterations.json";

/// Project store for persistence
pub struct ProjectStore;

impl ProjectStore {
    pub fn new() -> Self {
        Self
    }

    /// Load project from disk
    pub fn load(&self) -> anyhow::Result<Option<Project>> {
        let path = self.project_file_path()?;
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let project: Project = serde_json::from_str(&content)?;
        Ok(Some(project))
    }

    /// Save project to disk
    pub fn save(&self, project: &Project) -> anyhow::Result<()> {
        let path = self.project_file_path()?;
        let content = serde_json::to_string_pretty(project)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Check if project exists
    pub fn exists(&self) -> bool {
        self.project_file_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Create new project
    pub fn create(&self, name: impl Into<String>) -> anyhow::Result<Project> {
        let project = Project::new(name);
        super::init_project_structure(&project.name)?;
        self.save(&project)?;
        Ok(project)
    }

    /// Update project
    pub fn update(&self, project: &Project) -> anyhow::Result<()> {
        self.save(project)
    }

    /// Add iteration summary to project
    pub fn add_iteration(&self, project: &mut Project, summary: IterationSummary) -> anyhow::Result<()> {
        project.add_iteration(summary);
        self.save(project)
    }

    /// Set current iteration
    pub fn set_current_iteration(&self, project: &mut Project, iteration_id: String) -> anyhow::Result<()> {
        project.set_current_iteration(iteration_id);
        self.save(project)
    }

    fn project_file_path(&self) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join(PROJECT_FILE))
    }
}

impl Default for ProjectStore {
    fn default() -> Self {
        Self::new()
    }
}
