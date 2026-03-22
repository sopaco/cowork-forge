//! Show iteration details command

use anyhow::Result;
use cowork_core::persistence::{IterationStore, ProjectStore};

/// Show iteration details
pub async fn execute(iteration_id: Option<String>) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    let iteration_id = match iteration_id {
        Some(id) => id,
        None => {
            match project_store.load()? {
                Some(project) => {
                    match project.current_iteration_id {
                        Some(id) => id,
                        None => {
                            anyhow::bail!("No current iteration. Specify an iteration ID or run an iteration first.");
                        }
                    }
                }
                None => {
                    anyhow::bail!("No project found. Run 'cowork init' first.");
                }
            }
        }
    };

    let iteration = iteration_store.load(&iteration_id)?;

    println!("📋 Iteration Details\n");
    println!("  ID:          {}", iteration.id);
    println!("  Number:      {}", iteration.number);
    println!("  Title:       {}", iteration.title);
    println!("  Description: {}", iteration.description);
    println!("  Status:      {:?}", iteration.status);
    println!("  Started:     {}", iteration.started_at.format("%Y-%m-%d %H:%M:%S"));

    if let Some(completed_at) = iteration.completed_at {
        println!("  Completed:   {}", completed_at.format("%Y-%m-%d %H:%M:%S"));
    }

    if let Some(ref base_id) = iteration.base_iteration_id {
        println!("  Base:        {}", base_id);
        println!("  Inheritance: {:?}", iteration.inheritance);
    }

    if let Some(ref stage) = iteration.current_stage {
        println!("  Current:     {}", stage);
    }

    if !iteration.completed_stages.is_empty() {
        println!("  Completed:   {}", iteration.completed_stages.join(", "));
    }

    println!("\n  Artifacts:");
    if iteration.artifacts.idea.is_some() {
        println!("    ✓ Idea");
    }
    if iteration.artifacts.prd.is_some() {
        println!("    ✓ PRD");
    }
    if iteration.artifacts.design.is_some() {
        println!("    ✓ Design");
    }
    if iteration.artifacts.plan.is_some() {
        println!("    ✓ Plan");
    }
    if iteration.artifacts.delivery.is_some() {
        println!("    ✓ Delivery");
    }

    Ok(())
}
