use std::path::PathBuf;

use crate::domain::{Iteration, IterationSummary};

use super::get_cowork_dir;

/// Iteration store for persistence
pub struct IterationStore;

impl IterationStore {
    pub fn new() -> Self {
        Self
    }

    /// Load iteration by ID
    pub fn load(&self, iteration_id: &str) -> anyhow::Result<Iteration> {
        let path = self.iteration_file_path(iteration_id)?;
        let content = std::fs::read_to_string(&path)?;
        let iteration: Iteration = serde_json::from_str(&content)?;
        Ok(iteration)
    }

    /// Save iteration to disk
    pub fn save(&self, iteration: &Iteration) -> anyhow::Result<()> {
        let path = self.iteration_file_path(&iteration.id)?;
        let content = serde_json::to_string_pretty(iteration)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Check if iteration exists
    pub fn exists(&self, iteration_id: &str) -> bool {
        self.iteration_file_path(iteration_id)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Delete iteration
    pub fn delete(&self, iteration_id: &str) -> anyhow::Result<()> {
        let path = self.iteration_file_path(iteration_id)?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Load all iterations
    pub fn load_all(&self) -> anyhow::Result<Vec<Iteration>> {
        let dir = get_cowork_dir()?.join("iterations");
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut iterations = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(iteration) = serde_json::from_str::<Iteration>(&content) {
                        iterations.push(iteration);
                    }
                }
            }
        }

        // Sort by iteration number
        iterations.sort_by_key(|i| i.number);
        Ok(iterations)
    }

    /// Load iterations as summaries
    pub fn load_summaries(&self) -> anyhow::Result<Vec<IterationSummary>> {
        let iterations = self.load_all()?;
        Ok(iterations.into_iter().map(|i| i.to_summary()).collect())
    }

    /// Get workspace path for iteration (V2 architecture: iteration-specific workspace)
    pub fn workspace_path(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("iterations").join(iteration_id).join("workspace"))
    }

    /// Ensure workspace exists for iteration (V2 architecture: iteration-specific workspace)
    pub fn ensure_workspace(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        let workspace = self.workspace_path(iteration_id)?;
        std::fs::create_dir_all(&workspace)?;
        Ok(workspace)
    }

    /// Get iteration directory path (contains artifacts subdirectory)
    pub fn iteration_path(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("iterations").join(iteration_id))
    }

    fn iteration_file_path(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("iterations").join(format!("{}.json", iteration_id)))
    }
}

impl Default for IterationStore {
    fn default() -> Self {
        Self::new()
    }
}
