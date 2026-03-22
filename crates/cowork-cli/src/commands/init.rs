//! Initialize a new project command

use anyhow::Result;
use cowork_core::persistence::ProjectStore;
use tracing::warn;

/// Initialize a new project
pub async fn execute(name: Option<String>) -> Result<()> {
    let project_store = ProjectStore::new();

    if project_store.exists() {
        let existing = project_store.load()?.unwrap();
        warn!("Project '{}' already exists", existing.name);
        println!("⚠️  Project '{}' already exists", existing.name);
        println!("   Use 'cowork iter' to create iterations.");
        return Ok(());
    }

    let name = match name {
        Some(n) => n,
        None => {
            std::env::current_dir()?
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "my-project".to_string())
        }
    };

    let project = project_store.create(&name)?;

    println!("✅ Created project: {}", project.name);
    println!("   Project ID: {}", project.id);
    println!("   Working directory: .cowork-v2/");
    println!();
    println!("Next steps:");
    println!("  1. Run 'cowork config' to configure your LLM settings");
    println!("  2. Run 'cowork iter \"<title>\"' to create your first iteration");

    Ok(())
}
