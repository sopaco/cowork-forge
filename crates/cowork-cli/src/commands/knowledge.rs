//! Regenerate knowledge for a completed iteration command

use anyhow::{Context, Result};
use cowork_core::domain::IterationStatus;
use cowork_core::interaction::CliBackend;
use cowork_core::llm::{create_llm_client, load_config};
use cowork_core::persistence::IterationStore;
use cowork_core::pipeline::IterationExecutor;
use std::sync::Arc;

/// Regenerate knowledge for a completed iteration
pub async fn execute(iteration_id: String) -> Result<()> {
    println!("🔄 Regenerating knowledge for iteration: {}", iteration_id);
    println!();

    let iteration_store = IterationStore::new();

    let iteration = iteration_store.load(&iteration_id)
        .context("Failed to load iteration")?;

    if iteration.status != IterationStatus::Completed {
        println!("❌ Iteration is not completed (status: {:?})", iteration.status);
        println!("   Knowledge can only be regenerated for completed iterations.");
        return Ok(());
    }

    let interaction = Arc::new(CliBackend::new());
    let executor = IterationExecutor::new(interaction);

    let model_config = load_config()
        .context("Failed to load LLM configuration. Run 'cowork config' to set up.")?;

    let model = create_llm_client(&model_config.llm)
        .context("Failed to create LLM client")?;

    match executor.regenerate_iteration_knowledge(&iteration_id, model).await {
        Ok(_) => {
            println!("\n✅ Knowledge regenerated successfully for iteration {}", iteration_id);
            Ok(())
        }
        Err(e) => {
            println!("\n❌ Failed to regenerate knowledge: {}", e);
            Err(e)
        }
    }
}
