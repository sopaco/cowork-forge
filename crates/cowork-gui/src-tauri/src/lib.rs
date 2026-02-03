// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::fs;
use tauri::{Emitter, Manager, State, Window};
use cowork_core::interaction::{InteractiveBackend, InputOption, InputResponse, MessageLevel, ProgressInfo};
use cowork_core::event_bus::EventBus;
use std::collections::HashMap;
use tokio::sync::oneshot;
use tracing::info;
use std::path::{Path, PathBuf};
use anyhow::Context;
use serde::{Serialize, Deserialize};

// GUI-specific modules
mod gui_types;
mod gui_commands;
mod preview_server;
mod project_runner;
mod project_manager;
use gui_types::*;
use project_manager::*;

// Import iterative assistant types
use cowork_core::agents::iterative_assistant::*;

// ============================================================================
// Iterative Assistant Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatActionResult {
    pub action_type: String,
    pub data: Option<serde_json::Value>,
}

/// Build project context from project index
fn build_project_context(index: &cowork_core::ProjectIndex) -> anyhow::Result<ProjectContext> {
    let sessions: Vec<cowork_core::agents::SessionInfo> = index.sessions.iter()
        .map(|s| {
            cowork_core::agents::SessionInfo {
                session_id: s.session_id.clone(),
                status: format!("{:?}", s.status),
                description: s.input_description.clone(),
                created_at: s.created_at.to_rfc3339(),
            }
        })
        .collect();
    
    Ok(ProjectContext {
        project_name: index.project_name.clone(),
        sessions,
        technology_stack: vec![],
    })
}

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
    pub project_registry_manager: Arc<Mutex<ProjectRegistryManager>>,
    pub workspace_path: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new(event_bus: Arc<EventBus>) -> Result<Self, anyhow::Error> {
        let project_registry_manager = ProjectRegistryManager::new()
            .context("Failed to initialize project registry manager")?;
        
        Ok(Self {
            event_bus,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            project_registry_manager: Arc::new(Mutex::new(project_registry_manager)),
            workspace_path: Arc::new(Mutex::new(None)),
        })
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

// ============================================================================
// Project Manager Commands
// ============================================================================

#[tauri::command]
async fn register_project(
    workspace_path: String,
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<ProjectRecord, String> {
    let mut registry = state.project_registry_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    registry.register_project(workspace_path, name, description)
        .map_err(|e| format!("Failed to register project: {}", e))
}

#[tauri::command]
async fn get_all_projects(
    status: Option<String>,
    search: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ProjectRecord>, String> {
    let registry = state.project_registry_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    let options = ProjectQueryOptions {
        status: status.and_then(|s| serde_json::from_str(&s).ok()),
        search,
        limit,
    };
    
    Ok(registry.get_all_projects(Some(options)))
}

#[tauri::command]
async fn delete_project(
    project_id: String,
    delete_files: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut registry = state.project_registry_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    registry.delete_project(&project_id, delete_files)
        .map_err(|e| format!("Failed to delete project: {}", e))
}

#[tauri::command]
async fn update_project(
    project_id: String,
    name: Option<String>,
    description: Option<String>,
    status: Option<String>,
    state: State<'_, AppState>,
) -> Result<ProjectRecord, String> {
    let mut registry = state.project_registry_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    let status = status.and_then(|s| serde_json::from_str(&s).ok());
    
    registry.update_project(&project_id, name, description, status)
        .map_err(|e| format!("Failed to update project: {}", e))
}

#[tauri::command]
async fn open_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut registry = state.project_registry_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    // Get project
    let project = registry.get_project(&project_id)
        .ok_or_else(|| "Project not found".to_string())?;
    
    // Update last opened time
    registry.update_last_opened(&project_id)
        .map_err(|e| format!("Failed to update last opened time: {}", e))?;
    
    // Drop the lock before spawning new process
    drop(registry);
    
    // Get current executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    // Spawn new process with --workspace parameter
    std::process::Command::new(&exe_path)
        .arg("--workspace")
        .arg(&project.workspace_path)
        .spawn()
        .map_err(|e| format!("Failed to open project: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn auto_register_current_project(
    state: State<'_, AppState>,
) -> Result<Option<ProjectRecord>, String> {
    let mut registry = state.project_registry_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    registry.auto_register_current_project()
        .map_err(|e| format!("Failed to auto-register project: {}", e))
}

#[tauri::command]
async fn set_workspace(
    workspace_path: String,
    state: State<'_, AppState>,
    window: Window,
) -> Result<(), String> {
    use std::path::Path;
    use cowork_core::storage::*;
    
    println!("[GUI] Setting workspace to: {}", workspace_path);
    
    let path = Path::new(&workspace_path);
    if !path.exists() {
        // Create the directory if it doesn't exist
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create workspace directory: {}", e))?;
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", workspace_path));
    }
    
    // Check if workspace is already set
    let workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    if workspace.is_some() {
        drop(workspace);
        return Err("This window already has a project opened. Please open a new window to work on another project.".to_string());
    }
    drop(workspace);
    
    // Change current directory
    std::env::set_current_dir(path)
        .map_err(|e| format!("Failed to set current directory: {}", e))?;
    
    // Initialize project if not already initialized
    if !is_project_initialized() {
        println!("[GUI] Project not initialized, initializing...");
        let project_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("cowork_project")
            .to_string();
        
        let index = init_project_index(project_name)
            .map_err(|e| {
                eprintln!("[GUI] Failed to init project: {}", e);
                format!("Failed to init project: {}", e)
            })?;
        
        // Verify that index.json was created
        let index_path = path.join(".cowork/index.json");
        if !index_path.exists() {
            return Err(format!("Failed to create index.json at {:?}", index_path));
        }
        
        println!("[GUI] Project initialized successfully with {} sessions", index.sessions.len());
    } else {
        println!("[GUI] Project already initialized");
    }
    
    // Store in app state
    let mut workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    *workspace = Some(workspace_path.clone());
    
    // Emit event to trigger reload
    let _ = window.emit("project_loaded", ());
    
    println!("[GUI] Workspace set successfully");
    Ok(())
}

#[tauri::command]
async fn has_open_project(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(workspace.is_some())
}

#[tauri::command]
async fn open_project_in_current_window(
    project_id: String,
    state: State<'_, AppState>,
    window: Window,
) -> Result<(), String> {
    use std::path::Path;
    
    // Check if window already has a project
    let workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    if workspace.is_some() {
        drop(workspace);
        return Err("This window already has a project opened. Please open a new window to work on another project.".to_string());
    }
    drop(workspace);
    
    // Get project from registry
    let mut registry = state.project_registry_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    let project = registry.get_project(&project_id)
        .ok_or_else(|| "Project not found".to_string())?;
    
    // Update last opened time
    registry.update_last_opened(&project_id)
        .map_err(|e| format!("Failed to update last opened time: {}", e))?;
    
    let workspace_path = project.workspace_path.clone();
    drop(registry);
    
    // Log for debugging
    println!("[GUI] Project opened in current window: {}", workspace_path);
    
    // Set workspace in current window
    let path = Path::new(&workspace_path);
    if !path.exists() {
        return Err(format!("Project path does not exist: {}", workspace_path));
    }
    
    std::env::set_current_dir(path)
        .map_err(|e| format!("Failed to set current directory: {}", e))?;
    
    let mut workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    *workspace = Some(workspace_path);
    
    // Emit event to trigger reload
    let _ = window.emit("project_loaded", ());
    
    Ok(())
}

#[tauri::command]
async fn get_workspace(
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(workspace.clone())
}

// ============================================================================
// Iterative Assistant Commands
// ============================================================================

#[tauri::command]
async fn send_chat_message(
    session_id: String,
    message: String,
    window: Window,
) -> Result<ChatActionResult, String> {
    use cowork_core::storage::*;
    use cowork_core::agents::iterative_assistant::*;
    use cowork_core::llm::ModelConfig;
    
    println!("[Iterative] Processing chat message for session: {}", session_id);
    
    // Check if project is initialized
    if !is_project_initialized() {
        return Ok(ChatActionResult {
            action_type: "direct_processing".to_string(),
            data: Some(serde_json::json!({ "new_session_id": session_id })),
        });
    }
    
    // Load project index
    let index = load_project_index()
        .map_err(|e| format!("Failed to load project index: {}", e))?;
    
    // Find session by id
    let session_record = index.sessions.iter()
        .find(|s| s.session_id == session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))?;
    
    match session_record.status {
        cowork_core::data::SessionStatus::InProgress => {
            // Session in progress, direct processing
            println!("[Iterative] Session in progress, direct processing");
            Ok(ChatActionResult {
                action_type: "direct_processing".to_string(),
                data: Some(serde_json::json!({ "new_session_id": session_id })),
            })
        }
        cowork_core::data::SessionStatus::Completed => {
            // Session completed, use IterativeAssistant
            println!("[Iterative] Session completed, analyzing intent");
            
            // For now, just return a simple response suggesting modify
            // Full implementation would use LLM
            Ok(ChatActionResult {
                action_type: "await_confirmation".to_string(),
                data: Some(serde_json::json!({
                    "intent": {
                        "intent_type": "continue_development",
                        "confidence": 0.8,
                        "reasoning": "Session completed, suggesting to continue development",
                        "suggested_action": "modify"
                    },
                    "suggestion": {
                        "title": message,
                        "modification_type": "feature_modification",
                        "affected_modules": ["unknown"],
                        "implementation_plan": ["Will be generated by AI"],
                        "risk_assessment": {
                            "risk_level": "low",
                            "risks": [],
                            "mitigation_strategies": []
                        },
                        "estimated_effort": "unknown",
                        "confidence": 0.8
                    },
                    "session_id": session_id
                })),
            })
        }
        _ => {
            // Other statuses
            Ok(ChatActionResult {
                action_type: "direct_processing".to_string(),
                data: Some(serde_json::json!({ "new_session_id": session_id })),
            })
        }
    }
}

#[tauri::command]
async fn confirm_modify(
    session_id: String,
    suggestion_str: String,
    window: Window,
) -> Result<String, String> {
    use cowork_core::storage::*;
    use cowork_core::pipeline::create_cowork_pipeline;
    use cowork_core::llm::ModelConfig;
    
    println!("[Iterative] Confirming modify for session: {}", session_id);
    
    // Parse suggestion string
    let suggestion: ModifySuggestion = serde_json::from_str(&suggestion_str)
        .map_err(|e| format!("Failed to parse suggestion: {}", e))?;
    
    // Load LLM config
    let config = ModelConfig::from_env()
        .map_err(|e| format!("Failed to load config: {}", e))?;
    
    // Create interaction backend
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        window.app_handle().state::<AppState>().event_bus.clone(),
        window.app_handle().state::<AppState>().pending_requests.clone(),
    ));
    
    // Create modify session
    let index = load_project_index()
        .map_err(|e| format!("Failed to load project index: {}", e))?;
    
    let new_session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    // Create session record
    let session_record = cowork_core::data::SessionRecord {
        session_id: new_session_id.clone(),
        session_type: cowork_core::data::SessionType::Modify,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: cowork_core::data::SessionStatus::InProgress,
        base_session_id: Some(session_id),
        input_description: suggestion.title.clone(),
        change_request_id: None,
    };
    
    let mut index = index;
    index.add_session(session_record);
    save_project_index(&index)
        .map_err(|e| format!("Failed to save index: {}", e))?;
    
    // Create pipeline
    let pipeline = create_cowork_pipeline(
        &config,
        &new_session_id,
        interaction,
    ).map_err(|e| format!("Failed to create pipeline: {}", e))?;
    
    // Execute pipeline in background
    let pipeline_clone = pipeline.clone();
    let session_id_clone = new_session_id.clone();
    let window_clone = window.app_handle().clone();
    let idea_clone = suggestion.title.clone();

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
                        
                        // Send event to frontend
                        let _ = window_clone.emit("agent_event", serde_json::json!({
                            "content": content,
                            "is_thinking": content.is_empty(),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }));
                    }
                    Err(e) => {
                        eprintln!("[Pipeline] Event error: {:?}", e);
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        };

        if let Err(e) = result.await {
            eprintln!("[Pipeline] Execution error: {:?}", e);
            let _ = window_clone.emit("pipeline_error", format!("Pipeline execution failed: {}", e));
        }
    });

    Ok(new_session_id)
}

// ============================================================================
// Core Commands
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

    // Get workspace path from app state
    let workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    let workspace_path = if let Some(ref ws) = *workspace {
        ws.clone()
    } else {
        // No workspace set, use current directory
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .to_str()
            .ok_or("Invalid current directory")?
            .to_string()
    };
    drop(workspace);

    // Ensure we're in the correct workspace directory
    let workspace_path_buf = std::path::Path::new(&workspace_path);
    if !workspace_path_buf.exists() {
        return Err(format!("Workspace directory does not exist: {}", workspace_path));
    }
    
    std::env::set_current_dir(workspace_path_buf)
        .map_err(|e| format!("Failed to set working directory to {}: {}", workspace_path, e))?;

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
        println!("[GUI] Loading existing project index");
        load_project_index()
            .map_err(|e| format!("Failed to load existing index: {}", e))?
    } else {
        println!("[GUI] Initializing new project");
        let index = init_project_index(project_name)
            .map_err(|e| format!("Failed to init project: {}", e))?;
        
        // Verify that index.json was created
        let index_path = PathBuf::from(".cowork/index.json");
        if !index_path.exists() {
            return Err(format!("Failed to create index.json at {:?}", index_path));
        }
        
        println!("[GUI] New project initialized successfully");
        index
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
fn get_sessions(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    use cowork_core::storage::*;
    
    // Get the current workspace from app state
    let workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    // If workspace is set, ensure we're in that directory
    if let Some(ref workspace_path) = *workspace {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        
        let workspace_path = std::path::Path::new(workspace_path);
        if current_dir != workspace_path {
            println!("[GUI] Current directory {:?} doesn't match workspace {:?}, switching...", 
                     current_dir, workspace_path);
            std::env::set_current_dir(workspace_path)
                .map_err(|e| format!("Failed to set workspace directory: {}", e))?;
        }
    } else {
        // Only try to find .cowork directory if no workspace is set
        try_find_cowork_dir();
    }
    drop(workspace);

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
    
    let app_state = AppState::new(event_bus)
        .expect("Failed to initialize application state");
    
    // Check for --workspace argument
    let args: Vec<String> = std::env::args().collect();
    let workspace_path = args.iter()
        .position(|arg| arg == "--workspace")
        .and_then(|pos| args.get(pos + 1))
        .cloned();
    
    let workspace_path_clone = workspace_path.clone();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
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
            // Memory commands
            gui_commands::query_memory_index,
            gui_commands::load_memory_detail,
            gui_commands::save_session_memory,
            gui_commands::promote_to_project_memory,
            gui_commands::get_memory_context,
            // Code formatting commands
            gui_commands::format_code,
            gui_commands::check_formatter_available,
            // Template commands
            gui_commands::get_templates,
            gui_commands::export_template,
            gui_commands::import_template,
            gui_commands::delete_template,
            gui_commands::apply_template,
            // Project manager commands
            register_project,
            get_all_projects,
            delete_project,
            update_project,
            open_project,
            open_project_in_current_window,
            auto_register_current_project,
            set_workspace,
            get_workspace,
            has_open_project,
            // Iterative assistant commands
            send_chat_message,
            confirm_modify,
        ])
        .setup(move |app| {
            // Initialize app handle for project runner
            gui_commands::init_app_handle(app.handle().clone());
            
            // Set workspace if provided via command line
            if let Some(workspace) = workspace_path_clone {
                println!("[GUI] Workspace path from command line: {}", workspace);
                use std::path::Path;
                let path = Path::new(&workspace);
                if path.exists() && path.is_dir() {
                    if let Err(e) = std::env::set_current_dir(path) {
                        eprintln!("[GUI] Failed to set workspace directory: {}", e);
                    } else {
                        println!("[GUI] Working directory set to: {}", workspace);
                        // Store in app state
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Ok(mut ws) = state.workspace_path.lock() {
                                *ws = Some(workspace);
                            }
                        }
                    }
                } else {
                    eprintln!("[GUI] Invalid workspace path: {}", workspace);
                }
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}




