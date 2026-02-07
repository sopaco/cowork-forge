// Cowork Forge - CLI Entry Point (Iteration Architecture)

use anyhow::Result;
use clap::{Parser, Subcommand};
use cowork_core::domain::IterationStatus;
use cowork_core::interaction::CliBackend;
use cowork_core::persistence::{IterationStore, ProjectStore};
use cowork_core::pipeline::IterationExecutor;
use cowork_core::event_bus::EventBus;
use std::sync::Arc;
use tracing::{info, error, warn};

#[derive(Parser)]
#[command(name = "cowork")]
#[command(about = "AI-powered software development system - Iteration Architecture", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file (default: config.toml)
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and execute a new iteration
    Iter {
        /// Iteration title
        title: String,

        /// Detailed description of the iteration
        #[arg(short, long)]
        description: Option<String>,

        /// Base iteration ID to inherit from (for evolution iterations)
        #[arg(short, long)]
        base: Option<String>,

        /// Inheritance mode: none, full, or partial
        #[arg(short, long, default_value = "full")]
        inherit: String,
    },

    /// List all iterations
    List {
        /// Show all iterations including completed ones
        #[arg(short, long)]
        all: bool,
    },

    /// Show iteration details
    Show {
        /// Iteration ID (defaults to current iteration)
        iteration_id: Option<String>,
    },

    /// Continue a paused iteration
    Continue {
        /// Iteration ID (defaults to current iteration)
        iteration_id: Option<String>,
    },

    /// Initialize a new project
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Show project status
    Status,

    /// Delete an iteration
    Delete {
        /// Iteration ID to delete
        iteration_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_filter = if cli.verbose {
        "debug".to_string()
    } else {
        "info".to_string()
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(log_filter)
        .init();

    // Execute command
    match cli.command {
        Commands::Iter { title, description, base, inherit } => {
            cmd_iter(title, description, base, inherit).await?
        }
        Commands::List { all } => cmd_list(all).await?,
        Commands::Show { iteration_id } => cmd_show(iteration_id).await?,
        Commands::Continue { iteration_id } => cmd_continue(iteration_id).await?,
        Commands::Init { name } => cmd_init(name).await?,
        Commands::Status => cmd_status().await?,
        Commands::Delete { iteration_id } => cmd_delete(iteration_id).await?,
    }

    Ok(())
}

/// Create and execute a new iteration
async fn cmd_iter(
    title: String,
    description: Option<String>,
    base: Option<String>,
    inherit: String,
) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    // Check if project exists
    let mut project = match project_store.load()? {
        Some(p) => p,
        None => {
            error!("No project found. Run 'cowork init' first.");
            anyhow::bail!("Project not initialized");
        }
    };

    let description = description.unwrap_or_else(|| title.clone());

    // Create interaction backend
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));

    // Create executor
    let executor = IterationExecutor::new(interaction);

    // Create iteration based on whether it's genesis or evolution
    let iteration = if let Some(base_id) = base {
        // Evolution iteration
        info!("Creating evolution iteration based on: {}", base_id);

        // Verify base iteration exists
        if !iteration_store.exists(&base_id) {
            error!("Base iteration '{}' not found", base_id);
            anyhow::bail!("Base iteration not found");
        }

        // Parse inheritance mode
        let inheritance = match inherit.as_str() {
            "none" => cowork_core::domain::InheritanceMode::None,
            "partial" => cowork_core::domain::InheritanceMode::Partial,
            _ => cowork_core::domain::InheritanceMode::Full,
        };

        // Create evolution iteration
        cowork_core::domain::Iteration::create_evolution(
            &project,
            title.clone(),
            description.clone(),
            base_id,
            inheritance,
        )
    } else {
        // Genesis iteration
        info!("Creating genesis iteration");
        cowork_core::domain::Iteration::create_genesis(
            &project,
            title.clone(),
            description.clone(),
        )
    };

    // Save iteration
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

    // Execute iteration
    println!("🚀 Starting iteration execution...");
    println!();

    match executor.execute(&mut project, &iteration.id, None).await {
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

/// List all iterations
async fn cmd_list(all: bool) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    // Check if project exists
    match project_store.load()? {
        Some(project) => {
            println!("📊 Project: {}\n", project.name);

            let iterations = iteration_store.load_all()?;

            if iterations.is_empty() {
                println!("No iterations yet. Run 'cowork iter <title>' to create one.");
                return Ok(());
            }

            // Filter iterations if not showing all
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

            // Print header
            println!("{:<12} {:<30} {:<12} {:<15} {}",
                "Number", "Title", "Status", "Current Stage", "ID");
            println!("{:-<100}", "");

            // Print iterations
            for iter in filtered {
                let status_str = format!("{:?}", iter.status);
                let status_colored = match iter.status {
                    IterationStatus::Completed => format!("\x1b[32m{}\x1b[0m", status_str), // Green
                    IterationStatus::Running => format!("\x1b[33m{}\x1b[0m", status_str),   // Yellow
                    IterationStatus::Paused => format!("\x1b[36m{}\x1b[0m", status_str),    // Cyan
                    IterationStatus::Failed => format!("\x1b[31m{}\x1b[0m", status_str),    // Red
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

/// Show iteration details
async fn cmd_show(iteration_id: Option<String>) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    // Determine which iteration to show
    let iteration_id = match iteration_id {
        Some(id) => id,
        None => {
            // Use current iteration from project
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

    // Load iteration
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

    // Show artifacts
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

/// Continue a paused iteration
async fn cmd_continue(iteration_id: Option<String>) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    // Determine which iteration to continue
    let iteration_id = match iteration_id {
        Some(id) => id,
        None => {
            // Find a paused iteration
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

    // Load project and iteration
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

    // Create executor
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));
    let executor = IterationExecutor::new(interaction);

    match executor.continue_iteration(&mut project, &iteration_id).await {
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

/// Initialize a new project
async fn cmd_init(name: Option<String>) -> Result<()> {
    let project_store = ProjectStore::new();

    if project_store.exists() {
        let existing = project_store.load()?.unwrap();
        warn!("Project '{}' already exists", existing.name);
        println!("⚠️  Project '{}' already exists", existing.name);
        println!("   Use 'cowork iter' to create iterations.");
        return Ok(());
    }

    // Get project name
    let name = match name {
        Some(n) => n,
        None => {
            // Try to infer from current directory
            std::env::current_dir()?
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "my-project".to_string())
        }
    };

    // Create project
    let project = project_store.create(&name)?;

    println!("✅ Created project: {}", project.name);
    println!("   Project ID: {}", project.id);
    println!("   Working directory: .cowork-v2/");
    println!();
    println!("Next steps:");
    println!("  1. Run 'cowork iter \"<title>\"' to create your first iteration");
    println!("  2. Or configure LLM settings in config.toml");

    Ok(())
}

/// Show project status
async fn cmd_status() -> Result<()> {
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

            // Load iterations
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

            // Show latest completed iteration
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

/// Delete an iteration
async fn cmd_delete(iteration_id: String) -> Result<()> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    // Verify iteration exists
    if !iteration_store.exists(&iteration_id) {
        anyhow::bail!("Iteration '{}' not found", iteration_id);
    }

    // Load iteration to show details
    let iteration = iteration_store.load(&iteration_id)?;

    println!("⚠️  You are about to delete iteration:");
    println!("   #{} - {}", iteration.number, iteration.title);
    println!("   ID: {}", iteration_id);
    println!();

    // Confirm deletion
    print!("Are you sure? [y/N]: ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        println!("Deletion cancelled.");
        return Ok(());
    }

    // Delete iteration
    iteration_store.delete(&iteration_id)?;

    // Update project (remove from iterations list)
    if let Ok(Some(mut project)) = project_store.load() {
        project.iterations.retain(|i| i.id != iteration_id);
        project_store.save(&project)?;
    }

    println!("✅ Iteration '{}' deleted.", iteration_id);

    Ok(())
}

/// Helper function to truncate string
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
