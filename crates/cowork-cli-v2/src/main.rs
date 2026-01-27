// Cowork Forge V2 - CLI Entry Point

use anyhow::Result;
use clap::{Parser, Subcommand};
use cowork_core_v2::llm::ModelConfig;
use cowork_core_v2::pipeline::{create_cowork_pipeline, create_partial_pipeline, create_resume_pipeline};
use cowork_core_v2::storage::cowork_dir_exists;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, error};
use adk_runner::{Runner, RunnerConfig};
use adk_session::InMemorySessionService;
use adk_core::Content;
use futures::StreamExt;

#[derive(Parser)]
#[command(name = "cowork-v2")]
#[command(about = "AI-powered software development system V2", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file (default: config.toml)
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable LLM streaming output (shows AI thinking process in real-time)
    #[arg(short, long, global = true)]
    stream: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new project
    New {
        /// Project idea/description
        idea: String,
    },

    /// Resume an existing project
    Resume,

    /// Modify existing project starting from a stage
    Modify {
        /// Stage to restart from (prd, design, plan, coding, check, delivery)
        #[arg(short, long)]
        from: String,
    },

    /// Show project status
    Status,

    /// Initialize config file
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging - output to stderr, not stdout
    let log_filter = if cli.verbose {
        // Verbose mode: show all logs including adk internals
        "debug".to_string()
    } else {
        // Normal mode: filter out adk verbose logs to avoid clutter
        "info,adk_agent=warn,adk_core=warn,adk_runner=warn".to_string()
    };
    
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // Force logs to stderr
        .with_env_filter(log_filter)
        .init();

    // Load configuration
    let config_path = cli.config.unwrap_or_else(|| "config.toml".to_string());
    let config = load_config(&config_path)?;

    // Execute command
    let enable_stream = cli.stream;
    match cli.command {
        Commands::New { idea } => cmd_new(idea, &config, enable_stream).await?,
        Commands::Resume => cmd_resume(&config, enable_stream).await?,
        Commands::Modify { from } => cmd_modify(&from, &config, enable_stream).await?,
        Commands::Status => cmd_status().await?,
        Commands::Init => cmd_init()?,
    }

    Ok(())
}

/// Load configuration from file or environment
fn load_config(path: &str) -> Result<ModelConfig> {
    if Path::new(path).exists() {
        info!("Loading configuration from {}", path);
        ModelConfig::from_file(path)
    } else {
        info!("Config file not found, attempting to load from environment variables");
        ModelConfig::from_env()
    }
}

/// Start a new project
async fn cmd_new(idea: String, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    info!("Starting new project with idea: {}", idea);

    if cowork_dir_exists() {
        error!(".cowork directory already exists. Use 'resume' or 'modify' instead.");
        anyhow::bail!("Project already initialized");
    }

    // Create pipeline
    let pipeline = create_cowork_pipeline(config)?;

    // Execute pipeline with idea as input
    println!("✨ Creating new project...");
    println!("Idea: {}", idea);
    println!();

    execute_pipeline(pipeline, &idea, enable_stream).await?;

    println!("\n✅ Project creation complete!");
    println!("Check .cowork/ directory for artifacts");

    Ok(())
}

/// Resume an existing project
async fn cmd_resume(config: &ModelConfig, enable_stream: bool) -> Result<()> {
    info!("Resuming project");

    if !cowork_dir_exists() {
        error!(".cowork directory not found. Use 'new' to create a project.");
        anyhow::bail!("No project found");
    }

    // Create resume pipeline (skips idea stage)
    let pipeline = create_resume_pipeline(config)?;

    // Execute pipeline
    println!("🔄 Resuming project...");
    println!();

    execute_pipeline(pipeline, "Resume from last checkpoint", enable_stream).await?;

    println!("\n✅ Project resume complete!");

    Ok(())
}

/// Modify project from a specific stage
async fn cmd_modify(from_stage: &str, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    info!("Modifying project from stage: {}", from_stage);

    if !cowork_dir_exists() {
        error!(".cowork directory not found. Use 'new' to create a project.");
        anyhow::bail!("No project found");
    }

    // Create partial pipeline
    let pipeline = create_partial_pipeline(config, from_stage)?;

    // Execute pipeline
    println!("🔧 Modifying project from {} stage...", from_stage);
    println!();

    execute_pipeline(pipeline, &format!("Modify from {} stage", from_stage), enable_stream).await?;

    println!("\n✅ Modification complete!");

    Ok(())
}

/// Execute a pipeline with given input
async fn execute_pipeline(pipeline: Arc<dyn adk_core::Agent>, input: &str, enable_stream: bool) -> Result<()> {
    use adk_core::RunConfig;
    use adk_session::{CreateRequest, SessionService};
    use std::collections::HashMap;

    // Create session service
    let session_service = Arc::new(InMemorySessionService::new());

    // Create session FIRST
    let user_id = "cowork-user".to_string();
    let app_name = "cowork-forge-v2".to_string();
    
    let session = session_service
        .create(CreateRequest {
            app_name: app_name.clone(),
            user_id: user_id.clone(),
            session_id: None, // Auto-generate session ID
            state: HashMap::new(),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;
    
    let session_id = session.id().to_string();

    // Create runner with run config
    let runner = Runner::new(RunnerConfig {
        app_name,
        agent: pipeline,
        session_service,
        artifact_service: None,
        memory_service: None,
        run_config: Some(RunConfig::default()),
    })?;

    // Execute
    let content = Content::new("user").with_text(input);

    let mut event_stream = runner.run(user_id, session_id, content).await?;

    // Simple phase indicator - show when we start processing
    println!("🚀 Starting execution...\n");
    
    // Optional: Show streaming mode status
    if enable_stream {
        println!("💬 Streaming mode enabled - showing LLM output in real-time\n");
    }
    
    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => {
                // If streaming is enabled, show LLM output
                if enable_stream {
                    if let Some(llm_content) = &event.llm_response.content {
                        use std::io::Write;
                        let mut stdout = std::io::stdout();
                        
                        for part in &llm_content.parts {
                            if let Some(text) = part.text() {
                                // Filter out standalone newlines to reduce erratic line breaks
                                if text != "\n" {
                                    print!("{}", text);
                                    stdout.flush().ok();
                                }
                            }
                        }
                    }
                }
                // Tools will always print their own progress (e.g., "📝 Writing file: ...")
            }
            Err(e) => {
                error!("Error during pipeline execution: {}", e);
                anyhow::bail!("Pipeline execution failed: {}", e);
            }
        }
    }

    println!("\n✅ Pipeline complete!");

    Ok(())
}

/// Show project status
async fn cmd_status() -> Result<()> {
    use cowork_core_v2::storage::*;

    if !cowork_dir_exists() {
        println!("❌ No project found in current directory");
        return Ok(());
    }

    println!("📊 Project Status\n");

    // Load and display requirements
    match load_requirements() {
        Ok(reqs) => {
            println!("Requirements: {} total", reqs.requirements.len());
        }
        Err(_) => println!("Requirements: Not yet created"),
    }

    // Load and display features
    match load_feature_list() {
        Ok(features) => {
            let completed = features.features.iter().filter(|f| matches!(f.status, cowork_core_v2::data::FeatureStatus::Completed)).count();
            println!("Features: {}/{} completed", completed, features.features.len());
        }
        Err(_) => println!("Features: Not yet created"),
    }

    // Load and display design
    match load_design_spec() {
        Ok(design) => {
            println!("Components: {} defined", design.architecture.components.len());
        }
        Err(_) => println!("Design: Not yet created"),
    }

    // Load and display plan
    match load_implementation_plan() {
        Ok(plan) => {
            let completed = plan.tasks.iter().filter(|t| matches!(t.status, cowork_core_v2::data::TaskStatus::Completed)).count();
            println!("Tasks: {}/{} completed", completed, plan.tasks.len());
        }
        Err(_) => println!("Implementation Plan: Not yet created"),
    }

    Ok(())
}

/// Initialize configuration file
fn cmd_init() -> Result<()> {
    let config_path = "config.toml";

    if Path::new(config_path).exists() {
        error!("config.toml already exists");
        anyhow::bail!("Configuration file already exists");
    }

    let default_config = r#"[llm]
api_base_url = "http://localhost:8000/v1"
api_key = "your-api-key-here"
model_name = "gpt-4"
"#;

    std::fs::write(config_path, default_config)?;
    println!("✅ Created config.toml");
    println!("\nPlease edit config.toml and set your API credentials:");
    println!("  - api_base_url: Your OpenAI-compatible API endpoint");
    println!("  - api_key: Your API key");
    println!("  - model_name: Model to use (e.g., gpt-4, gpt-3.5-turbo)");

    Ok(())
}
