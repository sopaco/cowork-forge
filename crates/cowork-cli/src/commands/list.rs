//! List iterations command

use anyhow::Result;
use cowork_core::domain::IterationStatus;
use cowork_core::persistence::{IterationStore, ProjectStore};
use crate::utils::truncate;

/// List all iterations
pub async fn execute(all: bool) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    match project_store.load()? {
        Some(project) => {
            println!("📊 Project: {}\n", project.name);

            let iterations = iteration_store.load_all()?;

            if iterations.is_empty() {
                println!("No iterations yet. Run 'cowork iter <title>' to create one.");
                return Ok(());
            }

            let filtered: Vec<_> = if all {
                iterations
            } else {
                iterations
                    .into_iter()
                    .filter(|i| matches!(i.status, IterationStatus::Running | IterationStatus::Paused))
                    .collect()
            };

            if filtered.is_empty() && !all {
                println!("No active iterations. Use --all to see completed iterations.");
                return Ok(());
            }

            println!("{:<12} {:<30} {:<12} {:<15} {}",
                "Number", "Title", "Status", "Current Stage", "ID");
            println!("{:-<100}", "");

            for iter in filtered {
                let status_str = format!("{:?}", iter.status);
                let status_colored = match iter.status {
                    IterationStatus::Completed => format!("\x1b[32m{}\x1b[0m", status_str),
                    IterationStatus::Running => format!("\x1b[33m{}\x1b[0m", status_str),
                    IterationStatus::Paused => format!("\x1b[36m{}\x1b[0m", status_str),
                    IterationStatus::Failed => format!("\x1b[31m{}\x1b[0m", status_str),
                    IterationStatus::Draft => status_str,
                };

                let current_stage = iter.current_stage.unwrap_or_else(|| "-".to_string());
                let short_id = &iter.id[..20.min(iter.id.len())];

                println!("{:<12} {:<30} {:<20} {:<15} {}",
                    iter.number,
                    truncate(&iter.title, 28),
                    status_colored,
                    current_stage,
                    short_id
                );
            }

            if !all {
                println!("\nTip: Use --all to see completed iterations");
            }
        }
        None => {
            println!("❌ No project found. Run 'cowork init' first.");
        }
    }

    Ok(())
}
