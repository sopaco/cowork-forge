// Persistence module - File system storage
use std::path::PathBuf;

pub mod iteration_store;
pub mod memory_store;
pub mod project_store;

pub use iteration_store::*;
pub use memory_store::*;
pub use project_store::*;

const COWORK_DIR: &str = ".cowork-v2";

/// Get the .cowork-v2 directory path
/// Note: This function does NOT create the directory. 
/// Directory creation is handled by init_project_structure() or individual store save operations.
/// This prevents "Read-only file system" errors when the app starts without a workspace set.
pub fn get_cowork_dir() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(COWORK_DIR))
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
