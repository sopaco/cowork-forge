//! Iteration creation and execution command

use anyhow::{Context, Result};
use cowork_core::interaction::CliBackend;
use cowork_core::llm::{create_llm_client, load_config};
use cowork_core::persistence::{IterationStore, ProjectStore};
use cowork_core::pipeline::IterationExecutor;
use std::sync::Arc;
use tracing::info;

/// Create and execute a new iteration
pub async fn execute(
    title: String,
    description: Option<String>,
    base: Option<String>,
    inherit: String,
) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    let mut project = match project_store.load()? {
        Some(p) => p,
        None => {
            anyhow::bail!("No project found. Run 'cowork init' first.");
        }
    };

    let description = description.unwrap_or_else(|| title.clone());

    let interaction = Arc::new(CliBackend::new());
    let executor = IterationExecutor::new(interaction);

    let iteration = if let Some(base_id) = base {
        info!("Creating evolution iteration based on: {}", base_id);

        if !iteration_store.exists(&base_id) {
            anyhow::bail!("Base iteration '{}' not found", base_id);
        }

        let inheritance = match inherit.as_str() {
            "none" => cowork_core::domain::InheritanceMode::None,
            "full" => cowork_core::domain::InheritanceMode::Full,
            _ => cowork_core::domain::InheritanceMode::Partial,
        };

        cowork_core::domain::Iteration::create_evolution(
            &project,
            title.clone(),
            description.clone(),
            base_id,
            inheritance,
        )
    } else {
        info!("Creating genesis iteration");
        cowork_core::domain::Iteration::create_genesis(
            &project,
            title.clone(),
            description.clone(),
        )
    };

    iteration_store.save(&iteration)?;
    project_store.add_iteration(&mut project, iteration.to_summary())?;

    println!("✨ Created iteration: {}", iteration.id);
    println!("   Title: {}", iteration.title);
    println!("   Number: {}", iteration.number);
    if iteration.base_iteration_id.is_some() {
        println!("   Base: {:?}", iteration.base_iteration_id);
        println!("   Inheritance: {:?}", iteration.inheritance);
    }
    println!("   Start Stage: {}", iteration.determine_start_stage());
    println!();

    println!("🚀 Starting iteration execution...");
    println!();

    let model_config = load_config()
        .context("Failed to load LLM configuration. Run 'cowork config' to set up.")?;

    let model = create_llm_client(&model_config.llm)
        .context("Failed to create LLM client")?;

    match executor.execute(&mut project, &iteration.id, None, Some(model)).await {
        Ok(_) => {
            println!("\n✅ Iteration '{}' completed successfully!", iteration.title);
            println!("   Iteration ID: {}", iteration.id);
            Ok(())
        }
        Err(e) => {
            println!("\n❌ Iteration failed: {}", e);
            Err(e)
        }
    }
}
