// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State, Window};
use cowork_core::interaction::{InteractiveBackend, InputOption, InputResponse, MessageLevel, ProgressInfo};
use cowork_core::event_bus::EventBus;
use std::collections::HashMap;
use tokio::sync::oneshot;
use tracing::info;
use std::path::Path;
use cowork_core::llm::ModelConfig;

// GUI-specific modules
mod gui_types;
mod gui_commands;
mod preview_server;
mod project_runner;
use gui_types::*;

// ============================================================================
// TauriBackend - GUI implementation of InteractiveBackend
// ============================================================================

pub struct TauriBackend {
    app_handle: tauri::AppHandle,
    event_bus: Arc<EventBus>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<InputResponse>>>>,
}

impl TauriBackend {
    pub fn new(
        app_handle: tauri::AppHandle,
        event_bus: Arc<EventBus>,
        pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<InputResponse>>>>,
    ) -> Self {
        Self {
            app_handle,
            event_bus,
            pending_requests,
        }
    }
}

#[async_trait::async_trait]
impl InteractiveBackend for TauriBackend {
    async fn show_message(&self, level: MessageLevel, content: String) {
        let _ = self.app_handle.emit("message", (level, content));
    }

    async fn request_input(&self, prompt: &str, options: Vec<InputOption>, _initial_content: Option<String>) -> anyhow::Result<InputResponse> {
        use std::time::Duration;

        // Generate a unique request ID
        let request_id = format!("req-{}", chrono::Utc::now().timestamp_millis());

        println!("[HITL] Requesting input: {} (ID: {})", prompt, request_id);
        println!("[HITL] Options: {:?}", options.iter().map(|o| &o.id).collect::<Vec<_>>());

        // Create a channel for waiting for response
        let (tx, rx) = oneshot::channel();

        // Store the sender so it can be used later when user responds
        {
            let mut pending = self.pending_requests.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            pending.insert(request_id.clone(), tx);
        }

        // Emit request to frontend
        let _ = self.app_handle.emit("input_request", (request_id.clone(), prompt, options));
        println!("[HITL] Request sent to frontend");

        // Wait for response with timeout
        tokio::select! {
            result = rx => {
                match result {
                    Ok(response) => {
                        println!("[HITL] Received response: {:?}", response);
                        Ok(response)
                    },
                    Err(_) => {
                        println!("[HITL] Request canceled");
                        anyhow::bail!("Request canceled")
                    },
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(300)) => { // 5 minute timeout
                println!("[HITL] Request timeout after 300 seconds");
                anyhow::bail!("Request timeout")
            }
        }
    }

    async fn show_progress(&self, task_id: String, progress: ProgressInfo) {
        let _ = self.app_handle.emit("progress", (task_id, progress));
    }

    async fn submit_response(&self, _request_id: String, _response: String) -> anyhow::Result<()> {
        // CLI doesn't use async HITL, responses are handled synchronously
        Ok(())
    }

    fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }
}

// ============================================================================
// AppState
// ============================================================================

pub struct AppState {
    pub event_bus: Arc<EventBus>,
    pub pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<InputResponse>>>>,
}

impl AppState {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Cowork Forge GUI!", name)
}

#[tauri::command]
async fn create_project(
    idea: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use cowork_core::llm::ModelConfig;
    use cowork_core::pipeline::create_cowork_pipeline;
    use cowork_core::storage::*;

    println!("[GUI] Creating project with idea: {}", idea);

    // Try to find .cowork directory first
    try_find_cowork_dir();

    let src_tauri_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf();

    let project_root = src_tauri_dir.parent()
        .ok_or("Failed to get project root")?;

    // Change to project root directory
    std::env::set_current_dir(project_root)
        .map_err(|e| format!("Failed to set working directory to {}: {}", project_root.display(), e))?;

    println!("[GUI] Working directory set to: {}", std::env::current_dir().unwrap().display());

    // Load config - try working directory first, then exe directory
    let config = if Path::new("config.toml").exists() {
        ModelConfig::from_file("config.toml")
            .map_err(|e| format!("Failed to load config from working directory: {}", e))?
    } else if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(&exe_path);
        let config_path = exe_dir.join("config.toml");
        if config_path.exists() {
            ModelConfig::from_file(config_path.to_str().unwrap())
                .map_err(|e| format!("Failed to load config from exe directory: {}", e))?
        } else {
            ModelConfig::from_env()
                .map_err(|e| format!("Failed to load config from env: {}", e))?
        }
    } else {
        ModelConfig::from_env()
            .map_err(|e| format!("Failed to load config: {}", e))?
    };
    
    // Initialize project (allow even if already initialized for GUI)
    let project_name = idea.split_whitespace().take(3).collect::<Vec<_>>().join("_");
    let mut index = if is_project_initialized() {
        load_project_index()
            .map_err(|e| format!("Failed to load existing index: {}", e))?
    } else {
        init_project_index(project_name)
            .map_err(|e| format!("Failed to init project: {}", e))?
    };
    
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    // Create session record
    let session_record = cowork_core::data::SessionRecord {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::New,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: cowork_core::data::SessionStatus::InProgress,
        base_session_id: None,
        input_description: idea.clone(),
        change_request_id: None,
    };
    index.add_session(session_record);
    save_project_index(&index)
        .map_err(|e| format!("Failed to save index: {}", e))?;
    
    // Create interaction backend with shared pending_requests
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.event_bus.clone(),
        state.pending_requests.clone(),
    ));

    // Create pipeline
    let pipeline = create_cowork_pipeline(&config, &session_id, interaction)
        .map_err(|e| format!("Failed to create pipeline: {}", e))?;

    // Save session input
    let session_input = SessionInput {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::New,
        description: idea.clone(),
        base_session_id: None,
        created_at: chrono::Utc::now(),
    };
    save_session_input(&session_id, &session_input)
        .map_err(|e| format!("Failed to save session input: {}", e))?;

    // Ensure session directory exists
    let _ = get_session_dir(&session_id)
        .map_err(|e| format!("Failed to create session directory: {}", e))?;

    // Execute pipeline in background
    let pipeline_clone = pipeline.clone();
    let session_id_clone = session_id.clone();
    let window_clone = window.app_handle().clone();
    let idea_clone = idea.clone();

    tokio::spawn(async move {
        use adk_core::{RunConfig, Content};
        use adk_session::{CreateRequest, SessionService, InMemorySessionService};
        use adk_runner::{Runner, RunnerConfig};
        use futures::StreamExt;
        use std::collections::HashMap;

        let result = async {
            // Create session service
            let session_service = Arc::new(InMemorySessionService::new());

            // Create session
            let user_id = "cowork-gui-user".to_string();
            let app_name = "cowork-forge".to_string();

            let session = session_service
                .create(CreateRequest {
                    app_name: app_name.clone(),
                    user_id: user_id.clone(),
                    session_id: None,
                    state: HashMap::<String, serde_json::Value>::new(),
                })
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;

            let session_id = session.id().to_string();

            // Create runner
            let runner = Runner::new(RunnerConfig {
                app_name,
                agent: pipeline_clone,
                session_service,
                artifact_service: None,
                memory_service: None,
                run_config: Some(RunConfig::default()),
            })?;

            // Execute
            let content = Content::new("user").with_text(&idea_clone);
            let mut event_stream = runner.run(user_id, session_id, content).await?;

            // Process events
            while let Some(event_result) = event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        // Extract content
                        let content = if let Some(ref response_content) = event.llm_response.content {
                            response_content.parts.iter()
                                .filter_map(|part| part.text())
                                .collect::<Vec<_>>()
                                .join("")
                        } else {
                            "".to_string()
                        };
                        
                        // Log to console for debugging
                        if content.len() > 0 {
                            println!("[Agent Event] Content length: {}", content.len());
                        }
                        
                        // Send structured event to frontend
                        let _ = window_clone.emit("agent_event", serde_json::json!({
                            "content": content,
                            "is_thinking": content.is_empty(),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }));

                        if content.is_empty() {
                            let _ = window_clone.emit("session_completed", session_id_clone.clone());
                        }
                    }
                    Err(e) => {
                        eprintln!("Agent event error: {}", e);
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        }.await;

        match result {
            Ok(_) => {
                println!("[Session] Marking as completed: {}", session_id_clone);
                let _ = cowork_core::storage::mark_session_completed(&session_id_clone);
                let _ = window_clone.emit("session_completed", session_id_clone);
            }
            Err(e) => {
                println!("[Session] Marking as failed: {} - Error: {}", session_id_clone, e);
                let _ = cowork_core::storage::mark_session_failed(&session_id_clone);
                let _ = window_clone.emit("session_failed", (&session_id_clone, e.to_string()));
            }
        }
    });

    // Notify UI
    let _ = window.emit("project_created", &session_id);

    Ok(session_id)
}

#[tauri::command]
async fn resume_project(
    base_session_id: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use cowork_core::storage::*;
    use cowork_core::interaction::CliBackend;
    use cowork_core::pipeline::create_resume_pipeline;
    use cowork_core::llm::ModelConfig;
    use std::path::Path;
    
    info!("Resuming project from session: {}", base_session_id);

    // Try to find .cowork directory
    try_find_cowork_dir();

    if !is_project_initialized() {
        return Err("No project found".to_string());
    }

    // Verify base session exists
    let index = load_project_index()
        .map_err(|e| format!("Failed to load index: {}", e))?;
    
    let base_session = index.sessions.iter()
        .find(|s| s.session_id == base_session_id)
        .ok_or_else(|| format!("Base session {} not found", base_session_id))?;

    if base_session.status != cowork_core::data::SessionStatus::Completed {
        return Err("Can only resume from completed sessions".to_string());
    }

    // Create new session for resume
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    let mut index = load_project_index()
        .map_err(|e| format!("Failed to load index: {}", e))?;
    
    let session_record = cowork_core::data::SessionRecord {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::New,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: cowork_core::data::SessionStatus::InProgress,
        base_session_id: Some(base_session_id.clone()),
        input_description: format!("Resume from: {}", base_session.input_description),
        change_request_id: None,
    };

    index.add_session(session_record);
    save_project_index(&index)
        .map_err(|e| format!("Failed to save index: {}", e))?;
    
    // Save session input
    let session_input = SessionInput {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::New,
        description: format!("Resume from: {}", base_session.input_description),
        base_session_id: Some(base_session_id.clone()),
        created_at: chrono::Utc::now(),
    };
    save_session_input(&session_id, &session_input)
        .map_err(|e| format!("Failed to save session input: {}", e))?;

    // Bootstrap session state from base session
    init_session_from_base(&session_id, &base_session_id).map_err(|e| e.to_string())?;

    // Load config - try working directory first, then exe directory
    let config = if Path::new("config.toml").exists() {
        ModelConfig::from_file("config.toml")
            .map_err(|e| format!("Failed to load config from working directory: {}", e))?
    } else if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(&exe_path);
        let config_path = exe_dir.join("config.toml");
        if config_path.exists() {
            ModelConfig::from_file(config_path.to_str().unwrap())
                .map_err(|e| format!("Failed to load config from exe directory: {}", e))?
        } else {
            ModelConfig::from_env()
                .map_err(|e| format!("Failed to load config from env: {}", e))?
        }
    } else {
        ModelConfig::from_env()
            .map_err(|e| format!("Failed to load config: {}", e))?
    };

    // Create interaction backend
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));

    // Create resume pipeline
    let pipeline = create_resume_pipeline(&config, &session_id, &base_session_id, interaction).map_err(|e| e.to_string())?;

    // Execute pipeline in background
    let pipeline_clone = pipeline.clone();
    let session_id_clone = session_id.clone();
    let window_clone = window.app_handle().clone();

    tokio::spawn(async move {
        use adk_core::{RunConfig, Content};
        use adk_session::{CreateRequest, SessionService, InMemorySessionService};
        use adk_runner::{Runner, RunnerConfig};
        use futures::StreamExt;
        use std::collections::HashMap;

        let result = async {
            let session_service = Arc::new(InMemorySessionService::new());

            let user_id = "cowork-gui-user".to_string();
            let app_name = "cowork-forge".to_string();

            let session = session_service
                .create(CreateRequest {
                    app_name: app_name.clone(),
                    user_id: user_id.clone(),
                    session_id: None,
                    state: HashMap::new(),
                })
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;

            let session_id = session.id().to_string();

            let runner = Runner::new(RunnerConfig {
                app_name,
                agent: pipeline_clone,
                session_service,
                artifact_service: None,
                memory_service: None,
                run_config: Some(RunConfig::default()),
            })?;

            let content = Content::new("user").with_text("Resume session");
            let mut event_stream = runner.run(user_id, session_id, content).await?;

            while let Some(event_result) = event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        let content = if let Some(ref response_content) = event.llm_response.content {
                            response_content.parts.iter()
                                .filter_map(|part| part.text())
                                .collect::<Vec<_>>()
                                .join("")
                        } else {
                            "".to_string()
                        };
                        
                        if content.len() > 0 {
                            println!("[Agent Event] Content length: {}", content.len());
                        }
                        
                        let _ = window_clone.emit("agent_event", serde_json::json!({
                            "content": content,
                            "is_thinking": content.is_empty(),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }));

                        if content.is_empty() {
                            let _ = window_clone.emit("session_completed", session_id_clone.clone());
                        }
                    }
                    Err(e) => {
                        eprintln!("Agent event error: {}", e);
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        }.await;

        match result {
            Ok(_) => {
                println!("[Session] Marking as completed: {}", session_id_clone);
                let _ = cowork_core::storage::mark_session_completed(&session_id_clone);
                let _ = window_clone.emit("session_completed", session_id_clone);
            }
            Err(e) => {
                println!("[Session] Marking as failed: {} - Error: {}", session_id_clone, e);
                let _ = cowork_core::storage::mark_session_failed(&session_id_clone);
                let _ = window_clone.emit("session_failed", (&session_id_clone, e.to_string()));
            }
        }
    });

    // Notify UI
    let _ = window.emit("project_created", &session_id);

    Ok(session_id)
}

#[tauri::command]
async fn revert_project(
    base_session_id: String,
    from_stage: String,
    window: tauri::Window,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    use cowork_core::storage::*;
    use cowork_core::interaction::CliBackend;
    use cowork_core::llm::ModelConfig;
    use cowork_core::pipeline::create_resume_pipeline;
    use std::path::Path;
    
    info!("Reverting project from session: {} at stage: {}", base_session_id, from_stage);

    // Try to find .cowork directory
    try_find_cowork_dir();

    if !is_project_initialized() {
        return Err("No project found".to_string());
    }

    // Verify base session exists and is completed
    let index = load_project_index()
        .map_err(|e| format!("Failed to load index: {}", e))?;
    
    let base_session = index.sessions.iter()
        .find(|s| s.session_id == base_session_id)
        .ok_or_else(|| format!("Base session {} not found", base_session_id))?;

    if base_session.status != cowork_core::data::SessionStatus::Completed {
        return Err("Can only revert from completed sessions".to_string());
    }

    // Create new session for revert
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    let mut index = load_project_index()
        .map_err(|e| format!("Failed to load index: {}", e))?;
    
    let session_record = cowork_core::data::SessionRecord {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::Revert,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: cowork_core::data::SessionStatus::InProgress,
        base_session_id: Some(base_session_id.clone()),
        input_description: format!("Revert from {} stage", from_stage),
        change_request_id: None,
    };

    index.add_session(session_record);
    save_project_index(&index)
        .map_err(|e| format!("Failed to save index: {}", e))?;
    
    // Save session input
    let session_input = SessionInput {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::Revert,
        description: format!("Revert from {} stage", from_stage),
        base_session_id: Some(base_session_id.clone()),
        created_at: chrono::Utc::now(),
    };
    save_session_input(&session_id, &session_input)
        .map_err(|e| format!("Failed to save session input: {}", e))?;

    // Bootstrap session state from base session
    init_session_from_base(&session_id, &base_session_id).map_err(|e| e.to_string())?;

    // Load config - try working directory first, then exe directory
    let config = if Path::new("config.toml").exists() {
        ModelConfig::from_file("config.toml")
            .map_err(|e| format!("Failed to load config from working directory: {}", e))?
    } else if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(&exe_path);
        let config_path = exe_dir.join("config.toml");
        if config_path.exists() {
            ModelConfig::from_file(config_path.to_str().unwrap())
                .map_err(|e| format!("Failed to load config from exe directory: {}", e))?
        } else {
            ModelConfig::from_env()
                .map_err(|e| format!("Failed to load config from env: {}", e))?
        }
    } else {
        ModelConfig::from_env()
            .map_err(|e| format!("Failed to load config: {}", e))?
    };

    // Create interaction backend
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));

    // Create resume pipeline
    let pipeline = create_resume_pipeline(&config, &session_id, &base_session_id, interaction).map_err(|e| e.to_string())?;

    // Execute pipeline in background
    let pipeline_clone = pipeline.clone();
    let session_id_clone = session_id.clone();
    let window_clone = window.app_handle().clone();

    tokio::spawn(async move {
        use adk_core::{RunConfig, Content};
        use adk_session::{CreateRequest, SessionService, InMemorySessionService};
        use adk_runner::{Runner, RunnerConfig};
        use futures::StreamExt;
        use std::collections::HashMap;

        let result = async {
            let session_service = Arc::new(InMemorySessionService::new());

            let user_id = "cowork-gui-user".to_string();
            let app_name = "cowork-forge".to_string();

            let session = session_service
                .create(CreateRequest {
                    app_name: app_name.clone(),
                    user_id: user_id.clone(),
                    session_id: None,
                    state: HashMap::new(),
                })
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;

            let session_id = session.id().to_string();

            let runner = Runner::new(RunnerConfig {
                app_name,
                agent: pipeline_clone,
                session_service,
                artifact_service: None,
                memory_service: None,
                run_config: Some(RunConfig::default()),
            })?;

            let content = Content::new("user").with_text("Revert session");
            let mut event_stream = runner.run(user_id, session_id, content).await?;

            while let Some(event_result) = event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        let content = if let Some(ref response_content) = event.llm_response.content {
                            response_content.parts.iter()
                                .filter_map(|part| part.text())
                                .collect::<Vec<_>>()
                                .join("")
                        } else {
                            "".to_string()
                        };
                        
                        if content.len() > 0 {
                            println!("[Agent Event] Content length: {}", content.len());
                        }
                        
                        let _ = window_clone.emit("agent_event", serde_json::json!({
                            "content": content,
                            "is_thinking": content.is_empty(),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }));

                        if content.is_empty() {
                            let _ = window_clone.emit("session_completed", session_id_clone.clone());
                        }
                    }
                    Err(e) => {
                        eprintln!("Agent event error: {}", e);
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        }.await;

        match result {
            Ok(_) => {
                println!("[Session] Marking as completed: {}", session_id_clone);
                let _ = cowork_core::storage::mark_session_completed(&session_id_clone);
                let _ = window_clone.emit("session_completed", session_id_clone);
            }
            Err(e) => {
                println!("[Session] Marking as failed: {} - Error: {}", session_id_clone, e);
                let _ = cowork_core::storage::mark_session_failed(&session_id_clone);
                let _ = window_clone.emit("session_failed", (&session_id_clone, e.to_string()));
            }
        }
    });

    // Notify UI
    let _ = window.emit("project_created", &session_id);

    Ok(session_id)
}

#[tauri::command]
async fn modify_project(
    base_session_id: String,
    idea: String,
    window: tauri::Window,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    use cowork_core::storage::*;
    use cowork_core::interaction::CliBackend;
    use cowork_core::llm::ModelConfig;
    use cowork_core::pipeline::create_modify_pipeline;
    use std::path::Path;
    
    info!("Modifying project from session: {} with idea: {}", base_session_id, idea);

    // Try to find .cowork directory
    try_find_cowork_dir();

    if !is_project_initialized() {
        return Err("No project found".to_string());
    }

    // Verify base session exists and is completed
    let index = load_project_index()
        .map_err(|e| format!("Failed to load index: {}", e))?;
    
    let base_session = index.sessions.iter()
        .find(|s| s.session_id == base_session_id)
        .ok_or_else(|| format!("Base session {} not found", base_session_id))?;

    if base_session.status != cowork_core::data::SessionStatus::Completed {
        return Err("Can only modify completed sessions".to_string());
    }

    // Create new session for modify
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    let mut index = load_project_index()
        .map_err(|e| format!("Failed to load index: {}", e))?;
    
    let session_record = cowork_core::data::SessionRecord {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::Modify,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: cowork_core::data::SessionStatus::InProgress,
        base_session_id: Some(base_session_id.clone()),
        input_description: idea.clone(),
        change_request_id: None,
    };

    index.add_session(session_record);
    save_project_index(&index)
        .map_err(|e| format!("Failed to save index: {}", e))?;
    
    // Save session input
    let session_input = SessionInput {
        session_id: session_id.clone(),
        session_type: cowork_core::data::SessionType::Modify,
        description: idea.clone(),
        base_session_id: Some(base_session_id.clone()),
        created_at: chrono::Utc::now(),
    };
    save_session_input(&session_id, &session_input)
        .map_err(|e| format!("Failed to save session input: {}", e))?;

    // Bootstrap session state from base session
    init_session_from_base(&session_id, &base_session_id).map_err(|e| e.to_string())?;

    // Load config - try working directory first, then exe directory
    let config = if Path::new("config.toml").exists() {
        ModelConfig::from_file("config.toml")
            .map_err(|e| format!("Failed to load config from working directory: {}", e))?
    } else if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(&exe_path);
        let config_path = exe_dir.join("config.toml");
        if config_path.exists() {
            ModelConfig::from_file(config_path.to_str().unwrap())
                .map_err(|e| format!("Failed to load config from exe directory: {}", e))?
        } else {
            ModelConfig::from_env()
                .map_err(|e| format!("Failed to load config from env: {}", e))?
        }
    } else {
        ModelConfig::from_env()
            .map_err(|e| format!("Failed to load config: {}", e))?
    };

    // Create interaction backend
    let event_bus = Arc::new(EventBus::new());
    let interaction = Arc::new(CliBackend::new(event_bus));

    // Create modify pipeline
    let pipeline = create_modify_pipeline(&config, &session_id, &base_session_id, interaction).map_err(|e| e.to_string())?;

    // Execute pipeline in background
    let pipeline_clone = pipeline.clone();
    let session_id_clone = session_id.clone();
    let window_clone = window.app_handle().clone();

    tokio::spawn(async move {
        use adk_core::{RunConfig, Content};
        use adk_session::{CreateRequest, SessionService, InMemorySessionService};
        use adk_runner::{Runner, RunnerConfig};
        use futures::StreamExt;
        use std::collections::HashMap;

        let result = async {
            let session_service = Arc::new(InMemorySessionService::new());

            let user_id = "cowork-gui-user".to_string();
            let app_name = "cowork-forge".to_string();

            let session = session_service
                .create(CreateRequest {
                    app_name: app_name.clone(),
                    user_id: user_id.clone(),
                    session_id: None,
                    state: HashMap::new(),
                })
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;

            let session_id = session.id().to_string();

            let runner = Runner::new(RunnerConfig {
                app_name,
                agent: pipeline_clone,
                session_service,
                artifact_service: None,
                memory_service: None,
                run_config: Some(RunConfig::default()),
            })?;

            let content = Content::new("user").with_text("Modify session");
            let mut event_stream = runner.run(user_id, session_id, content).await?;

            while let Some(event_result) = event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        let content = if let Some(ref response_content) = event.llm_response.content {
                            response_content.parts.iter()
                                .filter_map(|part| part.text())
                                .collect::<Vec<_>>()
                                .join("")
                        } else {
                            "".to_string()
                        };
                        
                        if content.len() > 0 {
                            println!("[Agent Event] Content length: {}", content.len());
                        }
                        
                        let _ = window_clone.emit("agent_event", serde_json::json!({
                            "content": content,
                            "is_thinking": content.is_empty(),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }));

                        if content.is_empty() {
                            let _ = window_clone.emit("session_completed", session_id_clone.clone());
                        }
                    }
                    Err(e) => {
                        eprintln!("Agent event error: {}", e);
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        }.await;

        match result {
            Ok(_) => {
                println!("[Session] Marking as completed: {}", session_id_clone);
                let _ = cowork_core::storage::mark_session_completed(&session_id_clone);
                let _ = window_clone.emit("session_completed", session_id_clone);
            }
            Err(e) => {
                println!("[Session] Marking as failed: {} - Error: {}", session_id_clone, e);
                let _ = cowork_core::storage::mark_session_failed(&session_id_clone);
                let _ = window_clone.emit("session_failed", (&session_id_clone, e.to_string()));
            }
        }
    });

    // Notify UI
    let _ = window.emit("project_created", &session_id);

    Ok(session_id)
}

#[tauri::command]
fn get_sessions() -> Result<Vec<SessionInfo>, String> {
    use cowork_core::storage::*;

    // Try to find .cowork directory
    try_find_cowork_dir();

    // Check if project is initialized
    if !is_project_initialized() {
        return Ok(vec![]); // Return empty list if not initialized
    }

    let index = load_project_index()
        .map_err(|e| format!("Failed to load index: {}", e))?;

    Ok(index.sessions.into_iter().map(|s| SessionInfo {
        id: s.session_id,
        status: format!("{:?}", s.status),
        created_at: s.created_at.to_rfc3339(),
        description: s.input_description,
    }).collect())
}

fn try_find_cowork_dir() -> bool {
    use std::path::Path;
use cowork_core::llm::ModelConfig;
    let current_dir = std::env::current_dir().unwrap();
    let mut search_dir = current_dir.clone();

    for _ in 0..5 {
        let cowork_path = search_dir.join(".cowork");
        if cowork_path.exists() && cowork_path.join("index.json").exists() {
            std::env::set_current_dir(&search_dir).unwrap();
            println!("[GUI] Working directory set to: {:?}", search_dir);
            return true;
        }

        if let Ok(entries) = std::fs::read_dir(&search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let cowork_path = path.join(".cowork");
                    if cowork_path.exists() && cowork_path.join("index.json").exists() {
                        std::env::set_current_dir(&path).unwrap();
                        println!("[GUI] Working directory set to: {:?}", path);
                        return true;
                    }
                }
            }
        }

        search_dir = search_dir.parent().unwrap_or(&search_dir).to_path_buf();
        if search_dir == current_dir {
            break;
        }
    }
    false
}

// ============================================================================
// Session Info
// ============================================================================

#[derive(serde::Serialize, serde::Deserialize)]
struct SessionInfo {
    id: String,
    status: String,
    created_at: String,
    description: String,
}

// ============================================================================
// HITL Commands
// ============================================================================

#[tauri::command]
async fn submit_input_response(
    request_id: String,
    response: String,
    response_type: String, // "text" or "selection"
    state: State<'_, AppState>,
) -> Result<(), String> {
    let input_response = match response_type.as_str() {
        "text" => InputResponse::Text(response),
        "selection" => InputResponse::Selection(response),
        _ => InputResponse::Cancel,
    };

    let mut pending = state.pending_requests.lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    if let Some(tx) = pending.remove(&request_id) {
        let _ = tx.send(input_response);
        Ok(())
    } else {
        Err(format!("Request {} not found", request_id))
    }
}

// ============================================================================
// Run Application
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let event_bus = Arc::new(EventBus::new());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(event_bus))
        .invoke_handler(tauri::generate_handler![
            greet,
            create_project,
            revert_project,
            modify_project,
            resume_project,
            get_sessions,
            submit_input_response,
            // GUI-specific commands
            gui_commands::get_session_artifacts,
            gui_commands::read_file_content,
            gui_commands::save_file_content,
            gui_commands::get_file_tree,
            gui_commands::start_preview,
            gui_commands::stop_preview,
            gui_commands::start_project,
            gui_commands::stop_project,
            gui_commands::execute_project_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}




