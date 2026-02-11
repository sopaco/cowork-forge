// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::fs;
use tauri::{Emitter, Manager, State, Window};
use cowork_core::interaction::{InteractiveBackend, InputResponse};
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
// TauriBackend - GUI implementation of InteractiveBackend
// ============================================================================

pub struct TauriBackend {
    app_handle: tauri::AppHandle,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<InputResponse>>>>,
}

impl TauriBackend {
    pub fn new(
        app_handle: tauri::AppHandle,
        pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<InputResponse>>>>,
    ) -> Self {
        Self {
            app_handle,
            pending_requests,
        }
    }
}

#[async_trait::async_trait]
impl InteractiveBackend for TauriBackend {
    async fn show_message(&self, level: cowork_core::interaction::MessageLevel, content: String) {
        // Determine agent name from message content
        let agent_name = determine_agent_name(&content);

        // Emit agent_event for frontend processing display
        let _ = self.app_handle.emit("agent_event", serde_json::json!({
            "content": content,
            "agent_name": agent_name,
            "is_thinking": false,
            "level": format!("{:?}", level)
        }));

        // Also emit legacy message event for backward compatibility
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
            _ = tokio::time::sleep(Duration::from_secs(3000)) => { // 5 minute timeout
                println!("[HITL] Request timeout after 3000 seconds");
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


}

/// Determine agent name from message content based on stage keywords
fn determine_agent_name(content: &str) -> String {
    let content_lower = content.to_lowercase();

    // Check for stage-specific keywords - order matters!
    // Put more specific keywords first
    if content_lower.contains("delivery") || content_lower.contains("delivering") {
        return "Delivery Agent".to_string();
    } else if content_lower.contains("check") || content_lower.contains("validat") || content_lower.contains("test") {
        return "Validation Agent".to_string();
    } else if content_lower.contains("coding") || content_lower.contains("generating code") || (content_lower.contains("file") && content_lower.contains("generat")) {
        return "Coding Agent".to_string();
    } else if content_lower.contains("plan") || content_lower.contains("implementation plan") {
        return "Planning Agent".to_string();
    } else if content_lower.contains("design") || content_lower.contains("architecture") {
        return "Design Agent".to_string();
    } else if content_lower.contains("prd") || content_lower.contains("requirement") {
        return "Requirements Agent".to_string();
    } else if content_lower.contains("idea") || content_lower.contains("concept") {
        return "Ideation Agent".to_string();
    } else if content_lower.contains("stage") && content_lower.contains("complet") {
        return "Pipeline Controller".to_string();
    }

    // Default agent name
    "Cowork Agent".to_string()
}

// ============================================================================
// AppState
// ============================================================================

pub struct AppState {
    pub pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<InputResponse>>>>,
    pub project_registry_manager: Arc<Mutex<ProjectRegistryManager>>,
    pub workspace_path: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new() -> Result<Self, anyhow::Error> {
        let project_registry_manager = ProjectRegistryManager::new()
            .context("Failed to initialize project registry manager")?;

        Ok(Self {
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

    // Reset any "running" iterations to "paused" since there's no actual execution after reopening
    reset_running_iterations();

    // Store in app state
    let mut workspace = state.workspace_path.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    *workspace = Some(workspace_path.clone());

    // Emit event to trigger reload
    let _ = window.emit("project_loaded", ());

    println!("[GUI] Workspace set successfully");
    Ok(())
}

/// Reset all running iterations to paused state
/// This should be called when opening a project to ensure no "orphaned" running states
fn reset_running_iterations() {
    use cowork_core::persistence::IterationStore;
    use cowork_core::domain::IterationStatus;

    let iteration_store = IterationStore::new();

    match iteration_store.load_all() {
        Ok(iterations) => {
            let mut reset_count = 0;
            for mut iteration in iterations {
                // Check if iteration is in Running state
                if iteration.status == IterationStatus::Running {
                    println!("[GUI] Resetting iteration '{}' from Running to Paused", iteration.id);
                    iteration.status = IterationStatus::Paused;
                    if let Err(e) = iteration_store.save(&iteration) {
                        eprintln!("[GUI] Warning: Failed to reset iteration {}: {}", iteration.id, e);
                    } else {
                        reset_count += 1;
                    }
                }
            }
            if reset_count > 0 {
                println!("[GUI] Reset {} running iteration(s) to paused", reset_count);
            }
        }
        Err(e) => {
            eprintln!("[GUI] Warning: Failed to load iterations for reset: {}", e);
        }
    }
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

    // Reset any "running" iterations to "paused" since there's no actual execution after reopening
    reset_running_iterations();

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
// Legacy Session Commands (use iteration-based API instead)
// ============================================================================

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

// ============================================================================
// Run Application
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new()
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
            // Input/Interaction commands
            submit_input_response,
            // New Iteration-based commands
            iteration_commands::gui_init_project,
            iteration_commands::gui_get_project,
            iteration_commands::gui_delete_project,
            iteration_commands::gui_create_iteration,
            iteration_commands::gui_get_iterations,
            iteration_commands::gui_get_iteration,
            iteration_commands::gui_execute_iteration,
            iteration_commands::gui_continue_iteration,
            iteration_commands::gui_retry_iteration,
            iteration_commands::gui_delete_iteration,
            iteration_commands::gui_get_project_knowledge,
            iteration_commands::gui_regenerate_knowledge,
            // GUI-specific commands (V2 API - Iteration based)
            gui_commands::open_in_file_manager,
            gui_commands::get_iteration_artifacts,
            gui_commands::read_iteration_file,
            gui_commands::save_iteration_file,
            gui_commands::get_iteration_file_tree,
            gui_commands::start_iteration_preview,
            gui_commands::stop_iteration_preview,
            gui_commands::check_preview_status,
            gui_commands::start_iteration_project,
            gui_commands::stop_iteration_project,
            gui_commands::check_project_status,            // Memory commands
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
