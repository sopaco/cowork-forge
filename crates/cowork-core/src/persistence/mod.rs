// Persistence module - File system storage
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub mod iteration_store;
pub mod memory_store;
pub mod project_store;
pub mod iteration_data;

pub use iteration_store::*;
pub use memory_store::*;
pub use project_store::*;
pub use iteration_data::*;

const COWORK_DIR: &str = ".cowork-v2";

// Global workspace path set by GUI when a project is opened
static GLOBAL_WORKSPACE_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

fn get_global_workspace_lock() -> &'static Mutex<Option<PathBuf>> {
    GLOBAL_WORKSPACE_PATH.get_or_init(|| Mutex::new(None))
}

/// Set the global workspace path (called by GUI when a project is opened)
/// This is critical for macOS app bundle launches where current_dir() returns unexpected values
pub fn set_workspace_path(path: PathBuf) {
    if let Ok(mut workspace) = get_global_workspace_lock().lock() {
        *workspace = Some(path);
    }
}

/// Get the global workspace path
pub fn get_workspace_path() -> Option<PathBuf> {
    get_global_workspace_lock().lock().ok().and_then(|w| w.clone())
}

/// Get the .cowork-v2 directory path
/// Returns an absolute path if:
/// 1. A global workspace path has been set via set_workspace_path(), OR
/// 2. The relative path can be canonicalized (exists in current directory)
/// 
/// Note: This function does NOT create the directory. 
/// Directory creation is handled by init_project_structure() or individual store save operations.
pub fn get_cowork_dir() -> anyhow::Result<PathBuf> {
    // First, check if a global workspace path is set (GUI mode)
    if let Some(workspace) = get_workspace_path() {
        let cowork_dir = workspace.join(COWORK_DIR);
        return Ok(cowork_dir);
    }
    
    // Try to canonicalize the relative path (works if .cowork-v2 exists)
    let relative = PathBuf::from(COWORK_DIR);
    if let Ok(absolute) = relative.canonicalize() {
        return Ok(absolute);
    }
    
    // Fallback: try current directory
    let cwd = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
    Ok(cwd.join(COWORK_DIR))
}

/// Check if the .cowork-v2 directory exists (i.e., if we're in an initialized project)
pub fn is_project_initialized() -> bool {
    get_cowork_dir().map(|p| p.exists()).unwrap_or(false)
}

/// Initialize project structure
pub fn init_project_structure(_project_name: &str) -> anyhow::Result<PathBuf> {
    let cow_dir = get_cowork_dir()?;

    // Create main .cowork-v2 directory
    std::fs::create_dir_all(&cow_dir)?;

    // Create subdirectories
    std::fs::create_dir_all(cow_dir.join("iterations"))?;
    std::fs::create_dir_all(cow_dir.join("memory/project"))?;
    std::fs::create_dir_all(cow_dir.join("memory/iterations"))?;
    std::fs::create_dir_all(cow_dir.join("workspace"))?;

    Ok(cow_dir)
}
