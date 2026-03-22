//! Cowork Forge CLI - AI-powered software development system
//!
//! A command-line interface for the Cowork Forge development platform.
//! Supports project initialization, iteration management, and project import.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod utils;

#[derive(Parser)]
#[command(name = "cowork")]
#[command(about = "AI-powered software development system - Iteration Architecture", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

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

    /// Regenerate knowledge for a completed iteration
    RegenerateKnowledge {
        /// Iteration ID
        iteration_id: String,
    },

    /// Import an existing project into Cowork Forge
    Import {
        /// Path to the existing project directory
        path: String,

        /// Project name (defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,

        /// Generate idea.md artifact
        #[arg(long, default_value = "true")]
        idea: bool,

        /// Generate prd.md artifact
        #[arg(long, default_value = "true")]
        prd: bool,

        /// Generate design.md artifact
        #[arg(long, default_value = "true")]
        design: bool,

        /// Generate plan.md artifact
        #[arg(long, default_value = "true")]
        plan: bool,

        /// Skip LLM generation, use template only
        #[arg(long)]
        template_only: bool,
    },

    /// Configure LLM settings
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(log_filter)
        .init();

    // Dispatch to command handlers
    match cli.command {
        Commands::Iter { title, description, base, inherit } => {
            commands::iter(title, description, base, inherit).await?
        }
        Commands::List { all } => {
            commands::list(all).await?
        }
        Commands::Show { iteration_id } => {
            commands::show(iteration_id).await?
        }
        Commands::Continue { iteration_id } => {
            commands::continue_iteration(iteration_id).await?
        }
        Commands::Init { name } => {
            commands::init(name).await?
        }
        Commands::Status => {
            commands::status().await?
        }
        Commands::Delete { iteration_id } => {
            commands::delete(iteration_id).await?
        }
        Commands::RegenerateKnowledge { iteration_id } => {
            commands::regenerate_knowledge(iteration_id).await?
        }
        Commands::Import { path, name, idea, prd, design, plan, template_only } => {
            commands::import(path, name, idea, prd, design, plan, template_only).await?
        }
        Commands::Config => {
            commands::config().await?
        }
    }

    Ok(())
}