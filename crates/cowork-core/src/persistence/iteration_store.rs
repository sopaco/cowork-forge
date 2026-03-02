use std::path::PathBuf;

use crate::domain::{Iteration, IterationSummary};

use super::{get_cowork_dir, COWORK_DIR};

/// Iteration store for persistence
pub struct IterationStore;

impl IterationStore {
    pub fn new() -> Self {
        Self
    }

    /// Load iteration by ID
    pub fn load(&self, iteration_id: &str) -> anyhow::Result<Iteration> {
        let path = self.iteration_file_path(iteration_id)?;
        if !path.exists() {
            anyhow::bail!("Iteration not found: {}", iteration_id);
        }
        let content = std::fs::read_to_string(&path)?;
        let iteration: Iteration = serde_json::from_str(&content)?;
        Ok(iteration)
    }

    /// Save iteration to disk
    pub fn save(&self, iteration: &Iteration) -> anyhow::Result<()> {
        let path = self.iteration_file_path(&iteration.id)?;
        
        // Ensure parent directory exists before writing
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
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
            if entry
                .path()
                .extension()
                .map(|e| e == "json")
                .unwrap_or(false)
            {
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
    /// Returns ABSOLUTE path to avoid path confusion with external agents
    pub fn workspace_path(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        let relative = PathBuf::from(COWORK_DIR)
            .join("iterations")
            .join(iteration_id)
            .join("workspace");

        // Convert to absolute path
        // First try canonicalize (works if path exists)
        if let Ok(absolute) = relative.canonicalize() {
            return Ok(absolute);
        }

        // If path doesn't exist, canonicalize .cowork-v2 parent if it exists
        let cowork_dir = PathBuf::from(COWORK_DIR);
        if let Ok(cowork_absolute) = cowork_dir.canonicalize() {
            return Ok(cowork_absolute
                .join("iterations")
                .join(iteration_id)
                .join("workspace"));
        }

        // Fallback: use current directory
        let cwd = std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
        Ok(cwd.join(&relative))
    }

    /// Ensure workspace exists for iteration (V2 architecture: iteration-specific workspace)
    pub fn ensure_workspace(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        let workspace = self.workspace_path(iteration_id)?;
        std::fs::create_dir_all(&workspace)?;

        // Also ensure memory directory exists for this iteration
        let memory_dir = get_cowork_dir()?.join("memory/iterations");
        std::fs::create_dir_all(&memory_dir)?;

        Ok(workspace)
    }

    /// Get iteration directory path (contains artifacts subdirectory)
    /// Returns ABSOLUTE path for consistency
    pub fn iteration_path(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        let relative = PathBuf::from(COWORK_DIR)
            .join("iterations")
            .join(iteration_id);

        // Convert to absolute path
        if let Ok(absolute) = relative.canonicalize() {
            return Ok(absolute);
        }

        let cowork_dir = PathBuf::from(COWORK_DIR);
        if let Ok(cowork_absolute) = cowork_dir.canonicalize() {
            return Ok(cowork_absolute.join("iterations").join(iteration_id));
        }

        let cwd = std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
        Ok(cwd.join(&relative))
    }

    fn iteration_file_path(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        let relative = PathBuf::from(COWORK_DIR)
            .join("iterations")
            .join(format!("{}.json", iteration_id));

        // For file path, we need the parent to exist, so just return the path
        // The parent directory should already exist from iteration creation
        Ok(relative)
    }
}

impl Default for IterationStore {
    fn default() -> Self {
        Self::new()
    }
}
