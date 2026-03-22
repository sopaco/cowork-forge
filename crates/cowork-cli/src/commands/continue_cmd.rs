//! Continue a paused iteration command

use anyhow::{Context, Result};
use cowork_core::domain::IterationStatus;
use cowork_core::interaction::CliBackend;
use cowork_core::llm::{create_llm_client, load_config};
use cowork_core::persistence::{IterationStore, ProjectStore};
use cowork_core::pipeline::IterationExecutor;
use std::sync::Arc;

/// Continue a paused iteration
pub async fn execute(iteration_id: Option<String>) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    let iteration_id = match iteration_id {
        Some(id) => id,
        None => {
            let iterations = iteration_store.load_all()?;
            let paused: Vec<_> = iterations
                .into_iter()
                .filter(|i| matches!(i.status, IterationStatus::Paused))
                .collect();

            match paused.len() {
                0 => {
                    anyhow::bail!("No paused iterations found.");
                }
                1 => paused[0].id.clone(),
                _ => {
                    println!("Multiple paused iterations found. Please specify one:");
                    for iter in paused {
                        println!("  - {} ({})", iter.id, iter.title);
                    }
                    anyhow::bail!("Multiple paused iterations");
                }
            }
        }
    };

    let mut project = match project_store.load()? {
        Some(p) => p,
        None => {
            anyhow::bail!("No project found. Run 'cowork init' first.");
        }
    };

    let iteration = iteration_store.load(&iteration_id)?;

    if iteration.status != IterationStatus::Paused {
        anyhow::bail!("Iteration '{}' is not paused (status: {:?})", iteration_id, iteration.status);
    }

    println!("🔄 Continuing iteration: {}", iteration.title);
    println!("   Current stage: {:?}", iteration.current_stage);
    println!();

    let interaction = Arc::new(CliBackend::new());
    let executor = IterationExecutor::new(interaction);

    let model_config = load_config()
        .context("Failed to load LLM configuration. Run 'cowork config' to set up.")?;

    let model = create_llm_client(&model_config.llm)
        .context("Failed to create LLM client")?;

    match executor.continue_iteration(&mut project, &iteration_id, Some(model)).await {
        Ok(_) => {
            println!("\n✅ Iteration completed!");
            Ok(())
        }
        Err(e) => {
            println!("\n❌ Iteration failed: {}", e);
            Err(e)
        }
    }
}
