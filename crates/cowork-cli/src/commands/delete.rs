//! Delete an iteration command

use anyhow::Result;
use cowork_core::persistence::{IterationStore, ProjectStore};

/// Delete an iteration
pub async fn execute(iteration_id: String) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    if !iteration_store.exists(&iteration_id) {
        anyhow::bail!("Iteration '{}' not found", iteration_id);
    }

    let iteration = iteration_store.load(&iteration_id)?;

    println!("⚠️  You are about to delete iteration:");
    println!("   #{} - {}", iteration.number, iteration.title);
    println!("   ID: {}", iteration_id);
    println!();

    print!("Are you sure? [y/N]: ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        println!("Deletion cancelled.");
        return Ok(());
    }

    iteration_store.delete(&iteration_id)?;

    if let Ok(Some(mut project)) = project_store.load() {
        project.iterations.retain(|i| i.id != iteration_id);
        project_store.save(&project)?;
    }

    println!("✅ Iteration '{}' deleted.", iteration_id);

    Ok(())
}
