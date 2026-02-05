// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::fs;
use tauri::{Emitter, Manager, State, Window};
use cowork_core::interaction::{InteractiveBackend, InputResponse};
use cowork_core::event_bus::EventBus;
use std::collections::HashMap;
use tokio::sync::oneshot;
use anyhow::Context;
use serde::{Serialize, Deserialize};

// GUI-specific modules
mod gui_types;
mod gui_commands;
mod preview_server;
mod project_runner;
mod project_manager;
mod iteration_commands;
use project_manager::*;

// ============================================================================
// Iterative Assistant Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatActionResult {
    pub action_type: String,
    pub data: Option<serde_json::Value>,
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
    async fn show_message(&self, level: cowork_core::interaction::MessageLevel, content: String) {
        let _ = self.app_handle.emit("message", (level, content));
    }

    async fn request_input(&self, prompt: &str, options: Vec<cowork_core::interaction::InputOption>, _initial_content: Option<String>) -> anyhow::Result<InputResponse> {
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

    async fn show_progress(&self, task_id: String, progress: cowork_core::interaction::ProgressInfo) {
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
    
    // Initialize project using new V2 API
    let project_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("cowork_project")
        .to_string();
    
    // Use new persistence API to check if initialized
    let cowork_v2_path = path.join(".cowork-v2");
    if !cowork_v2_path.exists() {
        println!("[GUI] Project not initialized, initializing with V2 API...");
        
        // Create .cowork-v2 directory structure
        std::fs::create_dir_all(&cowork_v2_path)
            .map_err(|e| format!("Failed to create .cowork-v2 directory: {}", e))?;
        
        let project_file = cowork_v2_path.join("project.json");
        let iterations_dir = cowork_v2_path.join("iterations");
        std::fs::create_dir_all(&iterations_dir)
            .map_err(|e| format!("Failed to create iterations directory: {}", e))?;
        
        // Create initial project file
        let project = cowork_core::domain::Project::new(&project_name);
        let project_json = serde_json::to_string_pretty(&project)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;
        std::fs::write(&project_file, project_json)
            .map_err(|e| format!("Failed to write project.json: {}", e))?;
        
        println!("[GUI] Project initialized successfully with V2 API");
    } else {
        println!("[GUI] Project already initialized with V2");
    }
    
    // Auto-register project to registry
    let mut registry = state.project_registry_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    // Check if already registered
    let workspace_path_str = workspace_path.clone();
    if !registry.get_all_projects(None).iter().any(|p| p.workspace_path == workspace_path_str) {
        println!("[GUI] Auto-registering project to registry");
        if let Err(e) = registry.register_project(
            workspace_path.clone(),
            project_name,
            Some(format!("Cowork project at {}", workspace_path))
        ) {
            eprintln!("[GUI] Warning: Failed to auto-register project: {}", e);
            // Don't fail the whole operation if registration fails
        } else {
            println!("[GUI] Project registered successfully");
        }
    } else {
        println!("[GUI] Project already registered");
    }
    drop(registry);
    
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
// Legacy Commands (Deprecated - use iteration-based API instead)
// ============================================================================

#[tauri::command]
async fn send_chat_message(
    _session_id: String,
    _message: String,
) -> Result<ChatActionResult, String> {
    // Legacy command - functionality moved to iteration-based architecture
    Err("This command is deprecated. Please use the new Iteration-based API".to_string())
}

#[tauri::command]
async fn confirm_modify(
    _session_id: String,
    _suggestion_str: String,
) -> Result<String, String> {
    // Legacy command - functionality moved to iteration-based architecture
    Err("This command is deprecated. Please use the new Iteration-based API (gui_create_iteration, gui_execute_iteration)".to_string())
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
    _idea: String,
) -> Result<String, String> {
    // Legacy command - functionality moved to iteration-based architecture
    Err("This command is deprecated. Please use the new Iteration-based API (gui_init_project, gui_create_iteration)".to_string())
}

#[tauri::command]
async fn revert_project(
    _base_session_id: String,
    _from_stage: String,
) -> Result<String, String> {
    // Legacy command - functionality moved to iteration-based architecture
    Err("This command is deprecated. Please use the new Iteration-based API".to_string())
}

#[tauri::command]
async fn modify_project(
    _base_session_id: String,
    _idea: String,
) -> Result<String, String> {
    // Legacy command - functionality moved to iteration-based architecture
    Err("This command is deprecated. Please use the new Iteration-based API (gui_create_iteration)".to_string())
}

#[tauri::command]
async fn resume_project(
    _base_session_id: String,
) -> Result<String, String> {
    // Legacy command - functionality moved to iteration-based architecture
    Err("This command is deprecated. Please use the new Iteration-based API (gui_continue_iteration)".to_string())
}

// ============================================================================
// Legacy Session Commands (use iteration-based API instead)
// ============================================================================

#[tauri::command]
fn get_sessions(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    use std::path::Path;
    
    // Get the current workspace from app state
    let workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    // Check for V2 project structure
    if let Some(ref workspace_path) = *workspace {
        let project_file = Path::new(workspace_path).join(".cowork-v2").join("project.json");
        if project_file.exists() {
            // V2 project - return empty list (iterations are separate)
            return Ok(vec![]);
        }
        
        // Check for old V1 structure
        let v1_index = Path::new(workspace_path).join(".cowork").join("index.json");
        if v1_index.exists() {
            // Try to load old sessions
            match std::fs::read_to_string(&v1_index) {
                Ok(content) => {
                    if let Ok(index) = serde_json::from_str::<cowork_core::data::ProjectIndex>(&content) {
                        return Ok(index.sessions.into_iter().map(|s| SessionInfo {
                            id: s.session_id,
                            status: format!("{:?}", s.status),
                            created_at: s.created_at.to_rfc3339(),
                            description: s.input_description,
                        }).collect());
                    }
                }
                Err(_) => {}
            }
        }
    }
    
    Ok(vec![])
}

#[tauri::command]
async fn submit_input_response(
    request_id: String,
    response: String,
    response_type: String,
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

#[tauri::command]
async fn delete_session(
    _session_id: String,
    _window: tauri::Window,
) -> Result<(), String> {
    // Legacy command - use gui_delete_iteration for V2 API
    Err("This command is deprecated for V2 projects. Use gui_delete_iteration instead.".to_string())
}

#[tauri::command]
async fn get_session_logs(
    _session_id: String,
) -> Result<Vec<SessionLogEntry>, String> {
    // Legacy command - logs are now per-iteration in V2
    Ok(vec![])
}

// ============================================================================
// Session Info Types
// ============================================================================

#[derive(serde::Serialize, serde::Deserialize)]
struct SessionInfo {
    id: String,
    status: String,
    created_at: String,
    description: String,
}

#[derive(serde::Serialize)]
struct SessionLogEntry {
    file: String,
    content: String,
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
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Legacy commands (for backward compatibility during transition)
            create_project,
            revert_project,
            modify_project,
            resume_project,
            get_sessions,
            submit_input_response,
            delete_session,
            get_session_logs,
            // New Iteration-based commands
            iteration_commands::gui_init_project,
            iteration_commands::gui_get_project,
            iteration_commands::gui_delete_project,
            iteration_commands::gui_create_iteration,
            iteration_commands::gui_get_iterations,
            iteration_commands::gui_get_iteration,
            iteration_commands::gui_execute_iteration,
            iteration_commands::gui_continue_iteration,
            iteration_commands::gui_delete_iteration,
            // GUI-specific commands (Legacy)
            gui_commands::get_session_artifacts,
            gui_commands::read_file_content,
            gui_commands::save_file_content,
            gui_commands::get_file_tree,
            gui_commands::start_preview,
            gui_commands::stop_preview,
            gui_commands::start_project,
            gui_commands::stop_project,
            gui_commands::execute_project_command,
            // GUI-specific commands (V2 API - Iteration based)
            gui_commands::get_iteration_artifacts,
            gui_commands::read_iteration_file,
            gui_commands::save_iteration_file,
            gui_commands::get_iteration_file_tree,
            gui_commands::start_iteration_preview,
            gui_commands::stop_iteration_preview,
            gui_commands::start_iteration_project,
            gui_commands::stop_iteration_project,
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
