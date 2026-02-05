// Persistence module - File system storage
use std::path::PathBuf;

pub mod iteration_store;
pub mod memory_store;
pub mod project_store;

pub use iteration_store::*;
pub use memory_store::*;
pub use project_store::*;

const COWORK_DIR: &str = ".cowork-v2";

/// Get the .cowork directory path
pub fn get_cowork_dir() -> anyhow::Result<PathBuf> {
    let path = PathBuf::from(COWORK_DIR);
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

/// Initialize project structure
pub fn init_project_structure(_project_name: &str) -> anyhow::Result<PathBuf> {
    let cow_dir = get_cowork_dir()?;

    // Create subdirectories
    std::fs::create_dir_all(cow_dir.join("iterations"))?;
    std::fs::create_dir_all(cow_dir.join("memory/project"))?;
    std::fs::create_dir_all(cow_dir.join("memory/iterations"))?;
    std::fs::create_dir_all(cow_dir.join("workspace"))?;

    Ok(cow_dir)
}
