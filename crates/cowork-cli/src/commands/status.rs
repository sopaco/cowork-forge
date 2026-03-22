//! Show project status command

use anyhow::Result;
use cowork_core::domain::IterationStatus;
use cowork_core::persistence::{IterationStore, ProjectStore};

/// Show project status
pub async fn execute() -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    match project_store.load()? {
        Some(project) => {
            println!("📊 Project Status\n");
            println!("  Name:        {}", project.name);
            println!("  ID:          {}", project.id);
            println!("  Created:     {}", project.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("  Updated:     {}", project.updated_at.format("%Y-%m-%d %H:%M:%S"));

            if let Some(ref current_id) = project.current_iteration_id {
                println!("  Current:     {}", current_id);
            }

            let iterations = iteration_store.load_all()?;

            let completed = iterations.iter()
                .filter(|i| matches!(i.status, IterationStatus::Completed))
                .count();
            let running = iterations.iter()
                .filter(|i| matches!(i.status, IterationStatus::Running))
                .count();
            let paused = iterations.iter()
                .filter(|i| matches!(i.status, IterationStatus::Paused))
                .count();
            let failed = iterations.iter()
                .filter(|i| matches!(i.status, IterationStatus::Failed))
                .count();

            println!("\n  Iterations:");
            println!("    Total:      {}", iterations.len());
            println!("    Completed:  {}", completed);
            println!("    Running:    {}", running);
            println!("    Paused:     {}", paused);
            println!("    Failed:     {}", failed);

            if let Some(latest) = project.get_latest_completed_iteration() {
                println!("\n  Latest Completed:");
                println!("    #{} - {}", latest.number, latest.title);
                println!("    Stages: {}", latest.completed_stages.join(", "));
            }
        }
        None => {
            println!("❌ No project found in current directory.");
            println!("   Run 'cowork init' to create a new project.");
        }
    }

    Ok(())
}
