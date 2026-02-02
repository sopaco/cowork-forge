// Cowork Forge - CLI Entry Point

use anyhow::Result;
use clap::{Parser, Subcommand};
use cowork_core::llm::ModelConfig;
use cowork_core::pipeline::{create_cowork_pipeline, create_partial_pipeline, create_resume_pipeline, create_modify_pipeline};
use cowork_core::interaction::CliBackend;
use cowork_core::event_bus::EventBus;
use std::path::Path;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use walkdir::WalkDir;
use tracing::{info, error};
use adk_runner::{Runner, RunnerConfig};
use adk_session::InMemorySessionService;
use adk_core::Content;
use futures::StreamExt;

#[derive(Parser)]
#[command(name = "cowork")]
#[command(about = "AI-powered software development system", long_about = None)]
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
    Resume {
        /// Resume from a specific session ID (optional).
        /// If omitted, defaults to latest successful session; if none, tries latest in-progress session.
        #[arg(short, long)]
        base: Option<String>,
    },

    /// Revert project and restart from a specific stage
    Revert {
        /// Stage to restart from (prd, design, plan, coding, check, delivery)
        #[arg(short, long)]
        from: String,
    },

    /// Modify existing project with incremental changes
    Modify {
        /// Change idea/description
        idea: String,
        /// Base session ID (defaults to latest successful session)
        #[arg(short, long)]
        base: Option<String>,
    },

    /// Show project status
    Status {
        /// Show all sessions
        #[arg(short, long)]
        sessions: bool,
    },

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
        Commands::Resume { base } => cmd_resume(base, &config, enable_stream).await?,
        Commands::Revert { from } => cmd_revert(&from, &config, enable_stream).await?,
        Commands::Modify { idea, base } => cmd_modify(&idea, base, &config, enable_stream).await?,
        Commands::Status { sessions } => cmd_status(sessions).await?,
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
    use cowork_core::storage::*;
    use cowork_core::data::*;
    
    info!("Starting new project with idea: {}", idea);

    if is_project_initialized() {
        error!(".cowork directory already initialized. Use 'resume' or 'modify' instead.");
        anyhow::bail!("Project already initialized");
    }

    // Initialize project index
    let project_name = idea.split_whitespace().take(3).collect::<Vec<_>>().join("_");
    let mut index = init_project_index(project_name)?;
    
    // Generate session ID
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    // Create session record
    let session_record = SessionRecord {
        session_id: session_id.clone(),
        session_type: SessionType::New,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: SessionStatus::InProgress,
        base_session_id: None,
        input_description: idea.clone(),
        change_request_id: None,
    };
    
    index.add_session(session_record);
    save_project_index(&index)?;
    
    // Save session input
    let session_input = SessionInput {
        session_id: session_id.clone(),
        session_type: SessionType::New,
        description: idea.clone(),
        base_session_id: None,
        created_at: chrono::Utc::now(),
    };
    save_session_input(&session_id, &session_input)?;

    // Create interaction backend
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));

    // Create pipeline
    let pipeline = create_cowork_pipeline(config, &session_id, interaction)?;

    // Execute pipeline with idea as input
    println!("✨ Creating new project...");
    println!("Session ID: {}", session_id);
    println!("Idea: {}", idea);
    println!();

    let result = execute_pipeline(pipeline, &idea, enable_stream).await;

    // Mark session status based on result
    match result {
        Ok(_) => {
            mark_session_completed(&session_id)?;
            println!("\n✅ Project creation complete!");
            println!("Session ID: {}", session_id);
            println!("Check .cowork/sessions/{}/artifacts/ for outputs", session_id);
        }
        Err(e) => {
            mark_session_failed(&session_id)?;
            return Err(e);
        }
    }

    Ok(())
}

/// Resume an existing project
async fn cmd_resume(base: Option<String>, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    use cowork_core::storage::*;
    
    info!("Resuming project");

    if !is_project_initialized() {
        error!(".cowork directory not found. Use 'new' to create a project.");
        anyhow::bail!("No project found");
    }

    // Determine base session
    let base_session_id = if let Some(base_id) = base {
        base_id
    } else if let Some(latest_ok) = get_latest_successful_session()? {
        latest_ok
    } else {
        // Fallback: try latest in-progress session (useful when previous run was interrupted)
        let index = load_project_index()?;
        let last_in_progress = index
            .sessions
            .iter()
            .rev()
            .find(|s| s.status == cowork_core::data::SessionStatus::InProgress)
            .map(|s| s.session_id.clone());

        if let Some(sid) = last_in_progress {
            sid
        } else {
            error!("No successful session found. Cannot resume.");
            anyhow::bail!("No session to resume from");
        }
    };

    info!("Resuming from session: {}", base_session_id);

    // Create new session for resume
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    let mut index = load_project_index()?;
    let session_record = cowork_core::data::SessionRecord {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::New, // Resume is treated as continuation
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: cowork_core::data::SessionStatus::InProgress,
        base_session_id: Some(base_session_id.clone()),
        input_description: "Resume from last checkpoint".to_string(),
        change_request_id: None,
    };
    index.add_session(session_record);
    save_project_index(&index)?;
    
    // Save session input
    let session_input = SessionInput {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::New, // Resume is treated as continuation
        description: "Resume from last checkpoint".to_string(),
        base_session_id: Some(base_session_id.clone()),
        created_at: chrono::Utc::now(),
    };
    save_session_input(&session_id, &session_input)?;

    // Bootstrap session state from base session
    init_session_from_base(&session_id, &base_session_id)?;

    // Create interaction backend
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));

    // Create resume pipeline
    let pipeline = create_resume_pipeline(config, &session_id, &base_session_id, interaction)?;

    // Execute pipeline
    println!("🔄 Resuming project...");
    println!("Base session: {}", base_session_id);
    println!("New session: {}", session_id);
    println!();

    let result = execute_pipeline(pipeline, "Resume from last checkpoint", enable_stream).await;

    match result {
        Ok(_) => {
            mark_session_completed(&session_id)?;
            println!("\n✅ Project resume complete!");
        }
        Err(e) => {
            mark_session_failed(&session_id)?;
            return Err(e);
        }
    }

    Ok(())
}

/// Revert project and restart from a specific stage
async fn cmd_revert(from_stage: &str, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    use cowork_core::storage::*;    info!("Reverting project from stage: {}", from_stage);

    if !is_project_initialized() {
        error!(".cowork directory not found. Use 'new' to create a project.");
        anyhow::bail!("No project found");
    }

    let latest_session = get_latest_successful_session()?;
    if latest_session.is_none() {
        error!("No successful session found. Cannot revert.");
        anyhow::bail!("No session to revert from");
    }
    
    let base_session_id = latest_session.unwrap();

    // Support `--from auto`: use the latest session meta's requested restart stage (if any)
    let resolved_stage = if from_stage == "auto" {
        let index = load_project_index()?;
        let last_session_id = index
            .sessions
            .last()
            .map(|s| s.session_id.clone())
            .ok_or_else(|| anyhow::anyhow!("No session records found"))?;

        if let Some(meta) = load_session_meta(&last_session_id)? {
            if let Some(stage) = meta.current_stage {
                match stage {
                    cowork_core::data::Stage::Prd => "prd",
                    cowork_core::data::Stage::Design => "design",
                    cowork_core::data::Stage::Plan => "plan",
                    cowork_core::data::Stage::Coding => "coding",
                    cowork_core::data::Stage::Check => "check",
                    cowork_core::data::Stage::Delivery => "delivery",
                    cowork_core::data::Stage::Idea => "prd",
                }
            } else {
                "prd"
            }
        } else {
            "prd"
        }
    } else {
        from_stage
    };

    // Create new session for revert
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    let mut index = load_project_index()?;
    let session_record = cowork_core::data::SessionRecord {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::Revert,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: cowork_core::data::SessionStatus::InProgress,
        base_session_id: Some(base_session_id.clone()),
        input_description: format!("Revert from {} stage", resolved_stage),
        change_request_id: None,
    };
    index.add_session(session_record);
    save_project_index(&index)?;
    
    // Save session input
    let session_input = SessionInput {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::Revert,
        description: format!("Revert from {} stage", resolved_stage),
        base_session_id: Some(base_session_id.clone()),
        created_at: chrono::Utc::now(),
    };
    save_session_input(&session_id, &session_input)?;

    // Bootstrap session state from base session
    init_session_from_base(&session_id, &base_session_id)?;

    // Create interaction backend
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));

    // Create partial pipeline
    let pipeline = create_partial_pipeline(config, &session_id, &base_session_id, resolved_stage, interaction)?;

    // Execute pipeline
    println!("🔧 Reverting project from {} stage...", resolved_stage);
    println!("Base session: {}", base_session_id);
    println!("New session: {}", session_id);
    println!();

    let result = execute_pipeline(pipeline, &format!("Revert from {} stage", resolved_stage), enable_stream).await;

    match result {
        Ok(_) => {
            mark_session_completed(&session_id)?;
            println!("\n✅ Revert complete!");
        }
        Err(e) => {
            mark_session_failed(&session_id)?;
            return Err(e);
        }
    }

    Ok(())
}

fn should_ignore_project_path(path: &str) -> bool {
    // Ignore cowork internal state and common build artifacts
    let ignore_patterns = [
        "./.cowork/",
        "./target/",
        "./node_modules/",
        "./.git/",
        "./dist/",
        "./build/",
        "./.vscode/",
        "./.idea/",
    ];
    ignore_patterns.iter().any(|p| path.contains(p))
}

fn collect_project_file_fingerprints() -> Result<HashMap<String, (u64, u64)>> {
    // path -> (len, mtime_secs)
    let mut map = HashMap::new();

    for entry in WalkDir::new(".").follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let rel = p.strip_prefix(".").unwrap_or(p).to_string_lossy();
        let rel = format!("./{}", rel.trim_start_matches("/"));

        if should_ignore_project_path(&rel) {
            continue;
        }

        let md = entry.metadata()?;
        let len = md.len();
        let mtime = md
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        map.insert(rel, (len, mtime));
    }

    Ok(map)
}

fn diff_project_files(
    before: &HashMap<String, (u64, u64)>,
    after: &HashMap<String, (u64, u64)>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let before_keys: HashSet<&String> = before.keys().collect();
    let after_keys: HashSet<&String> = after.keys().collect();

    let mut added = Vec::new();
    let mut deleted = Vec::new();
    let mut modified = Vec::new();

    for k in after_keys.difference(&before_keys) {
        added.push((**k).clone());
    }

    for k in before_keys.difference(&after_keys) {
        deleted.push((**k).clone());
    }

    for k in before_keys.intersection(&after_keys) {
        let b = before.get(*k);
        let a = after.get(*k);
        if b != a {
            modified.push((**k).clone());
        }
    }

    added.sort();
    deleted.sort();
    modified.sort();

    (added, modified, deleted)
}

/// Modify existing project with incremental changes
async fn cmd_modify(idea: &str, base: Option<String>, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    use cowork_core::storage::*;
    use cowork_core::data::*;
    
    info!("Modifying project with idea: {}", idea);

    if !is_project_initialized() {
        error!(".cowork directory not found. Use 'new' to create a project.");
        anyhow::bail!("No project found");
    }

    // Determine base session
    let base_session_id = if let Some(base_id) = base {
        base_id
    } else {
        get_latest_successful_session()?
            .ok_or_else(|| anyhow::anyhow!("No successful session found. Cannot modify without a base."))?
    };
    
    info!("Using base session: {}", base_session_id);

    // Create new session for modify
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    // Create change request
    let change_request = ChangeRequest::new(
        session_id.clone(),
        idea.to_string(),
        base_session_id.clone(),
    );
    let change_request_id = change_request.id.clone();
    save_change_request(&session_id, &change_request)?;
    
    // Create session record
    let mut index = load_project_index()?;
    let session_record = SessionRecord {
        session_id: session_id.clone(),
        session_type: SessionType::Modify,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: SessionStatus::InProgress,
        base_session_id: Some(base_session_id.clone()),
        input_description: idea.to_string(),
        change_request_id: Some(change_request_id.clone()),
    };
    index.add_session(session_record);
    save_project_index(&index)?;
    
    // Save session input
    let session_input = SessionInput {
        session_id: session_id.clone(),
        session_type: SessionType::Modify,
        description: idea.to_string(),
        base_session_id: Some(base_session_id.clone()),
        created_at: chrono::Utc::now(),
    };
    save_session_input(&session_id, &session_input)?;

    // Bootstrap session state from base session
    init_session_from_base(&session_id, &base_session_id)?;

    // Create interaction backend
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));

    // Create modify pipeline (incremental change pipeline)
    let pipeline = create_modify_pipeline(config, &session_id, &base_session_id, interaction)?;

    // Snapshot project files before modification (for patch metadata)
    let before_files = collect_project_file_fingerprints()?;

    // Execute pipeline
    println!("🔄 Applying incremental changes...");
    println!("Change: {}", idea);
    println!("Base session: {}", base_session_id);
    println!("New session: {}", session_id);
    println!();

    let result = execute_pipeline(pipeline, idea, enable_stream).await;

    match result {
        Ok(_) => {
            // Snapshot after modification and persist patch metadata
            let after_files = collect_project_file_fingerprints()?;
            let (added_files, modified_files, deleted_files) = diff_project_files(&before_files, &after_files);

            let mut patch = PatchMetadata::new(session_id.clone(), base_session_id.clone());
            patch.added_files = added_files;
            patch.modified_files = modified_files;
            patch.deleted_files = deleted_files;
            save_patch_metadata(&session_id, &patch)?;

            mark_session_completed(&session_id)?;
            println!("\n✅ Modification complete!");
            println!("Session ID: {}", session_id);
            println!("Patch metadata: .cowork/sessions/{}/patch/metadata.json", session_id);
        }
        Err(e) => {
            mark_session_failed(&session_id)?;
            return Err(e);
        }
    }

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
    let app_name = "cowork-forge".to_string();
    
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
async fn cmd_status(show_sessions: bool) -> Result<()> {
    use cowork_core::storage::*;
    use cowork_core::data::*;

    if !is_project_initialized() {
        println!("❌ No project found in current directory");
        return Ok(());
    }

    let index = load_project_index()?;
    
    println!("📊 Project Status\n");
    println!("Project: {}", index.project_name);
    println!("Created: {}", index.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!();

    if show_sessions {
        // Show all sessions
        println!("Sessions ({} total):", index.sessions.len());
        println!("{:<20} {:<10} {:<15} {:<25}", "Session ID", "Type", "Status", "Created At");
        println!("{:-<70}", "");
        
        for session in &index.sessions {
            let session_type = match session.session_type {
                SessionType::New => "New",
                SessionType::Modify => "Modify",
                SessionType::Revert => "Revert",
            };
            let status = match session.status {
                SessionStatus::InProgress => "In Progress",
                SessionStatus::Completed => "Completed",
                SessionStatus::Failed => "Failed",
            };
            println!(
                "{:<20} {:<10} {:<15} {}",
                session.session_id,
                session_type,
                status,
                session.created_at.format("%Y-%m-%d %H:%M:%S")
            );
        }
        println!();
        
        if let Some(latest) = &index.latest_successful_session {
            println!("Latest successful: {}", latest);
        }
    } else {
        // Show summary of latest session
        if let Some(latest_id) = &index.latest_successful_session {
            println!("Latest successful session: {}", latest_id);
            
            // Try to load artifacts from latest session
            match load_requirements(latest_id) {
                Ok(reqs) => {
                    println!("Requirements: {} total", reqs.requirements.len());
                }
                Err(_) => println!("Requirements: Not yet created"),
            }

            match load_feature_list(latest_id) {
                Ok(features) => {
                    let completed = features.features.iter().filter(|f| matches!(f.status, FeatureStatus::Completed)).count();
                    println!("Features: {}/{} completed", completed, features.features.len());
                }
                Err(_) => println!("Features: Not yet created"),
            }

            match load_design_spec(latest_id) {
                Ok(design) => {
                    println!("Components: {} defined", design.architecture.components.len());
                }
                Err(_) => println!("Design: Not yet created"),
            }

            match load_implementation_plan(latest_id) {
                Ok(plan) => {
                    let completed = plan.tasks.iter().filter(|t| matches!(t.status, TaskStatus::Completed)).count();
                    println!("Tasks: {}/{} completed", completed, plan.tasks.len());
                }
                Err(_) => println!("Implementation Plan: Not yet created"),
            }
        } else {
            println!("No successful sessions yet");
        }
        
        println!();
        println!("Tip: Use 'cowork status --sessions' to see all sessions");
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
