// Project Manager - Top-level project management system
// Manages all Cowork projects across different directories

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: &str = "1.0";
const REGISTRY_FILENAME: &str = "project_registry.json";

// ============================================================================
// Data Structures
// ============================================================================

/// Project Registry - stores all Cowork projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRegistry {
    pub schema_version: String,
    pub projects: Vec<ProjectRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            projects: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Project Record - represents a single Cowork project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub workspace_path: String,
    pub created_at: DateTime<Utc>,
    pub last_opened_at: Option<DateTime<Utc>>,
    pub status: ProjectStatus,
    pub metadata: ProjectMetadata,
}

/// Project Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Archived,
    Deleted,
}

/// Project Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub session_count: usize,
    pub last_session_id: Option<String>,
    pub technology_stack: Vec<String>,
    pub project_type: String,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            session_count: 0,
            last_session_id: None,
            technology_stack: Vec::new(),
            project_type: "unknown".to_string(),
        }
    }
}

/// Project Query Options
#[derive(Debug, Clone, Default)]
pub struct ProjectQueryOptions {
    pub status: Option<ProjectStatus>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

// ============================================================================
// Project Registry Manager
// ============================================================================

pub struct ProjectRegistryManager {
    registry_path: PathBuf,
    registry: ProjectRegistry,
}

impl ProjectRegistryManager {
    /// Create a new ProjectRegistryManager
    pub fn new() -> Result<Self> {
        let registry_path = get_registry_path()?;
        
        // Ensure the directory exists
        if let Some(parent) = registry_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create registry directory")?;
        }
        
        // Load or create registry
        let registry = if registry_path.exists() {
            Self::load_registry_from_file(&registry_path)?
        } else {
            ProjectRegistry::default()
        };
        
        Ok(Self {
            registry_path,
            registry,
        })
    }
    
    /// Load registry from file
    fn load_registry_from_file(path: &Path) -> Result<ProjectRegistry> {
        let content = fs::read_to_string(path)
            .context("Failed to read registry file")?;
        
        let registry: ProjectRegistry = serde_json::from_str(&content)
            .context("Failed to parse registry file")?;
        
        // Validate schema version
        if registry.schema_version != SCHEMA_VERSION {
            tracing::warn!(
                "Registry schema version mismatch: expected {}, got {}",
                SCHEMA_VERSION,
                registry.schema_version
            );
        }
        
        Ok(registry)
    }
    
    /// Save registry to file
    fn save_registry_to_file(&self) -> Result<()> {
        let mut registry = self.registry.clone();
        registry.updated_at = Utc::now();
        
        let content = serde_json::to_string_pretty(&registry)
            .context("Failed to serialize registry")?;
        
        fs::write(&self.registry_path, content)
            .context("Failed to write registry file")?;
        
        Ok(())
    }
    
    /// Register a new project
    pub fn register_project(
        &mut self,
        workspace_path: String,
        name: String,
        description: Option<String>,
    ) -> Result<ProjectRecord> {
        // Check if project already exists
        if self.registry.projects.iter().any(|p| p.workspace_path == workspace_path) {
            anyhow::bail!("Project already registered: {}", workspace_path);
        }
        
        // Validate and create workspace path if it doesn't exist
        let workspace = Path::new(&workspace_path);
        if !workspace.exists() {
            // Create the directory if it doesn't exist
            fs::create_dir_all(workspace)
                .context(format!("Failed to create workspace directory: {}", workspace_path))?;
        }
        
        if !workspace.is_dir() {
            anyhow::bail!("Workspace path is not a directory: {}", workspace_path);
        }
        
        // Detect project metadata
        let metadata = detect_project_metadata(workspace)?;
        
        // Create project record
        let project = ProjectRecord {
            project_id: format!("proj-{}", Utc::now().timestamp_millis()),
            name,
            description,
            workspace_path,
            created_at: Utc::now(),
            last_opened_at: None,
            status: ProjectStatus::Active,
            metadata,
        };
        
        // Add to registry
        self.registry.projects.push(project.clone());
        self.save_registry_to_file()?;
        
        Ok(project)
    }
    
    /// Get all projects
    pub fn get_all_projects(&self, options: Option<ProjectQueryOptions>) -> Vec<ProjectRecord> {
        let mut projects = self.registry.projects.clone();
        
        // Apply filters
        if let Some(opts) = options {
            if let Some(status) = opts.status {
                projects.retain(|p| p.status == status);
            }
            
            if let Some(search) = opts.search {
                let search_lower = search.to_lowercase();
                projects.retain(|p| {
                    p.name.to_lowercase().contains(&search_lower)
                        || p.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&search_lower))
                            .unwrap_or(false)
                        || p.workspace_path.to_lowercase().contains(&search_lower)
                });
            }
            
            if let Some(limit) = opts.limit {
                projects.truncate(limit);
            }
        }
        
        // Sort by last opened (most recent first)
        projects.sort_by(|a, b| {
            match (&a.last_opened_at, &b.last_opened_at) {
                (Some(at_a), Some(at_b)) => at_b.cmp(at_a),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => b.created_at.cmp(&a.created_at),
            }
        });
        
        projects
    }
    
    /// Get project by ID
    pub fn get_project(&self, project_id: &str) -> Option<ProjectRecord> {
        self.registry.projects
            .iter()
            .find(|p| p.project_id == project_id)
            .cloned()
    }
    
    /// Delete a project
    pub fn delete_project(&mut self, project_id: &str, delete_files: bool) -> Result<()> {
        let index = self.registry.projects
            .iter()
            .position(|p| p.project_id == project_id)
            .ok_or_else(|| anyhow::anyhow!("Project not found: {}", project_id))?;
        
        let project = &self.registry.projects[index];
        
        // Delete files if requested
        if delete_files {
            let workspace = Path::new(&project.workspace_path);
            if workspace.exists() {
                fs::remove_dir_all(workspace)
                    .context("Failed to delete project files")?;
            }
        }
        
        // Remove from registry
        self.registry.projects.remove(index);
        self.save_registry_to_file()?;
        
        Ok(())
    }
    
    /// Update project information
    pub fn update_project(
        &mut self,
        project_id: &str,
        name: Option<String>,
        description: Option<String>,
        status: Option<ProjectStatus>,
    ) -> Result<ProjectRecord> {
        let index = self.registry.projects
            .iter()
            .position(|p| p.project_id == project_id)
            .ok_or_else(|| anyhow::anyhow!("Project not found: {}", project_id))?;
        
        let project = &mut self.registry.projects[index];
        
        if let Some(name) = name {
            project.name = name;
        }
        
        if let Some(description) = description {
            project.description = Some(description);
        }
        
        if let Some(status) = status {
            project.status = status;
        }
        
        // Clone before saving to avoid borrow issues
        let result = project.clone();
        
        self.save_registry_to_file()?;
        
        Ok(result)
    }
    
    /// Update last opened time
    pub fn update_last_opened(&mut self, project_id: &str) -> Result<()> {
        if let Some(project) = self.registry.projects
            .iter_mut()
            .find(|p| p.project_id == project_id)
        {
            project.last_opened_at = Some(Utc::now());
            self.save_registry_to_file()?;
        }
        
        Ok(())
    }
    
    /// Auto-register current project if it's a Cowork project
    pub fn auto_register_current_project(&mut self) -> Result<Option<ProjectRecord>> {
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory")?;
        
        // Check if it's a Cowork project
        if !is_cowork_project(&current_dir)? {
            return Ok(None);
        }
        
        let workspace_path = current_dir.to_string_lossy().to_string();
        
        // Check if already registered
        if self.registry.projects.iter().any(|p| p.workspace_path == workspace_path) {
            return Ok(None);
        }
        
        // Extract project name from directory
        let project_name = current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled Project")
            .to_string();
        
        // Register the project
        let project = self.register_project(
            workspace_path,
            project_name,
            None,
        )?;
        
        Ok(Some(project))
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the registry file path (cross-platform)
fn get_registry_path() -> Result<PathBuf> {
    let base_dir = if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\CoworkCreative\
        let appdata = std::env::var("APPDATA")
            .context("Failed to get APPDATA environment variable")?;
        PathBuf::from(appdata).join("CoworkCreative")
    } else if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/CoworkCreative/
        let home = std::env::var("HOME")
            .context("Failed to get HOME environment variable")?;
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("CoworkCreative")
    } else {
        // Linux: ~/.config/cowork-creative/
        let home = std::env::var("HOME")
            .context("Failed to get HOME environment variable")?;
        PathBuf::from(home).join(".config").join("cowork-creative")
    };
    
    Ok(base_dir.join(REGISTRY_FILENAME))
}

/// Check if a directory is a Cowork project
fn is_cowork_project(path: &Path) -> Result<bool> {
    let cowork_dir = path.join(".cowork");
    Ok(cowork_dir.exists() && cowork_dir.is_dir())
}

/// Detect project metadata from workspace
fn detect_project_metadata(workspace: &Path) -> Result<ProjectMetadata> {
    let mut metadata = ProjectMetadata::default();
    
    // Detect project type
    metadata.project_type = detect_project_type(workspace);
    
    // Detect technology stack
    metadata.technology_stack = detect_technology_stack(workspace);
    
    // Count sessions
    let sessions_dir = workspace.join(".cowork").join("sessions");
    if sessions_dir.exists() {
        metadata.session_count = fs::read_dir(sessions_dir)
            .map(|entries| entries.filter_map(Result::ok).count())
            .unwrap_or(0);
        
        // Find last session ID
        if let Ok(mut entries) = fs::read_dir(&workspace.join(".cowork").join("sessions")) {
            if let Some(Ok(last_entry)) = entries.last() {
                if let Some(name) = last_entry.file_name().to_str() {
                    metadata.last_session_id = Some(name.to_string());
                }
            }
        }
    }
    
    Ok(metadata)
}

/// Detect project type
fn detect_project_type(workspace: &Path) -> String {
    // Check for common project indicators
    if workspace.join("package.json").exists() {
        return "nodejs".to_string();
    }
    
    if workspace.join("Cargo.toml").exists() {
        return "rust".to_string();
    }
    
    if workspace.join("requirements.txt").exists()
        || workspace.join("pyproject.toml").exists()
        || workspace.join("setup.py").exists()
    {
        return "python".to_string();
    }
    
    if workspace.join("pom.xml").exists() {
        return "java".to_string();
    }
    
    if workspace.join("go.mod").exists() {
        return "go".to_string();
    }
    
    "unknown".to_string()
}

/// Detect technology stack
fn detect_technology_stack(workspace: &Path) -> Vec<String> {
    let mut stack = Vec::new();
    
    // Frontend frameworks
    if workspace.join("package.json").exists() {
        stack.push("JavaScript".to_string());
        
        if let Ok(content) = fs::read_to_string(workspace.join("package.json")) {
            if content.contains("\"react\"") {
                stack.push("React".to_string());
            }
            if content.contains("\"vue\"") {
                stack.push("Vue".to_string());
            }
            if content.contains("\"@angular\"") {
                stack.push("Angular".to_string());
            }
            if content.contains("\"svelte\"") {
                stack.push("Svelte".to_string());
            }
        }
    }
    
    // Backend frameworks
    if workspace.join("Cargo.toml").exists() {
        stack.push("Rust".to_string());
    }
    
    if workspace.join("requirements.txt").exists()
        || workspace.join("pyproject.toml").exists()
    {
        stack.push("Python".to_string());
        
        if let Ok(content) = fs::read_to_string(workspace.join("requirements.txt")) {
            if content.contains("django") {
                stack.push("Django".to_string());
            }
            if content.contains("flask") {
                stack.push("Flask".to_string());
            }
            if content.contains("fastapi") {
                stack.push("FastAPI".to_string());
            }
        }
    }
    
    stack
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_default() {
        let registry = ProjectRegistry::default();
        assert_eq!(registry.schema_version, SCHEMA_VERSION);
        assert!(registry.projects.is_empty());
    }
    
    #[test]
    fn test_project_metadata_default() {
        let metadata = ProjectMetadata::default();
        assert_eq!(metadata.session_count, 0);
        assert!(metadata.last_session_id.is_none());
        assert!(metadata.technology_stack.is_empty());
    }
}