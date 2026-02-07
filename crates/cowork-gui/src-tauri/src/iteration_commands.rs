// Iteration Commands - New iteration-based architecture for GUI

use crate::AppState;
use crate::TauriBackend;
use cowork_core::domain::{Iteration, InheritanceMode, Project};
use cowork_core::persistence::{IterationStore, ProjectStore};
use cowork_core::pipeline::IterationExecutor;
use tauri::{Emitter, Manager, State, Window};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationInfo {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub description: String,
    pub status: String,
    pub current_stage: Option<String>,
    pub completed_stages: Vec<String>,
    pub base_iteration_id: Option<String>,
    pub inheritance: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub current_iteration_id: Option<String>,
    pub iteration_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIterationRequest {
    pub title: String,
    pub description: String,
    pub base_iteration_id: Option<String>,
    pub inheritance: String,
}

// ============================================================================
// Project Commands
// ============================================================================

#[tauri::command]
pub async fn gui_init_project(
    name: String,
    window: Window,
) -> Result<ProjectInfo, String> {
    let store = ProjectStore::new();

    if store.exists() {
        let project = store.load().map_err(|e| e.to_string())?.unwrap();
        return Ok(project_to_info(&project));
    }

    let project = store.create(&name).map_err(|e| e.to_string())?;

    // Emit event
    let _ = window.emit("project_initialized", ());

    Ok(project_to_info(&project))
}

#[tauri::command]
pub async fn gui_get_project() -> Result<Option<ProjectInfo>, String> {
    let store = ProjectStore::new();

    match store.load().map_err(|e| e.to_string())? {
        Some(project) => Ok(Some(project_to_info(&project))),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn gui_delete_project(
    confirm: bool,
) -> Result<(), String> {
    if !confirm {
        return Err("Deletion not confirmed".to_string());
    }

    use std::fs;
    use cowork_core::persistence::get_cowork_dir;

    let cow_dir = get_cowork_dir().map_err(|e| e.to_string())?;
    if cow_dir.exists() {
        fs::remove_dir_all(&cow_dir).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Iteration Commands
// ============================================================================

#[tauri::command]
pub async fn gui_create_iteration(
    request: CreateIterationRequest,
    window: Window,
    _state: State<'_, AppState>,
) -> Result<IterationInfo, String> {
    let project_store = ProjectStore::new();
    let iteration_store = IterationStore::new();

    let mut project = project_store.load().map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not initialized".to_string())?;

    // Determine inheritance mode
    let inheritance = match request.inheritance.as_str() {
        "none" => InheritanceMode::None,
        "partial" => InheritanceMode::Partial,
        _ => InheritanceMode::Full,
    };

    // Create iteration
    let iteration = if let Some(base_id) = request.base_iteration_id {
        Iteration::create_evolution(
            &project,
            request.title,
            request.description,
            base_id,
            inheritance,
        )
    } else {
        Iteration::create_genesis(
            &project,
            request.title,
            request.description,
        )
    };

    // Save iteration
    iteration_store.save(&iteration).map_err(|e| e.to_string())?;
    project_store.add_iteration(&mut project, iteration.to_summary()).map_err(|e| e.to_string())?;

    // Emit event
    let _ = window.emit("iteration_created", iteration.id.clone());

    Ok(iteration_to_info(&iteration))
}

#[tauri::command]
pub async fn gui_get_iterations() -> Result<Vec<IterationInfo>, String> {
    let store = IterationStore::new();

    let iterations = store.load_all().map_err(|e| e.to_string())?;
    Ok(iterations.iter().map(iteration_to_info).collect())
}

#[tauri::command]
pub async fn gui_get_iteration(
    iteration_id: String,
) -> Result<IterationInfo, String> {
    let store = IterationStore::new();

    let iteration = store.load(&iteration_id).map_err(|e| e.to_string())?;
    Ok(iteration_to_info(&iteration))
}

#[tauri::command]
pub async fn gui_execute_iteration(
    iteration_id: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_store = ProjectStore::new();

    let mut project = project_store.load().map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not initialized".to_string())?;

    // Create interaction backend
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.event_bus.clone(),
        state.pending_requests.clone(),
    ));

    let executor = IterationExecutor::new(interaction);

    // Emit started event
    let _ = window.emit("iteration_started", iteration_id.clone());

    // Execute in background
    let window_clone = window.app_handle().clone();
    let iteration_id_clone = iteration_id.clone();

    tokio::spawn(async move {
        match executor.execute(&mut project, &iteration_id_clone, None).await {
            Ok(_) => {
                let _ = window_clone.emit("iteration_completed", iteration_id_clone);
            }
            Err(e) => {
                let _ = window_clone.emit("iteration_failed", (iteration_id_clone, e.to_string()));
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn gui_continue_iteration(
    iteration_id: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_store = ProjectStore::new();

    let mut project = project_store.load().map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not initialized".to_string())?;

    // Create interaction backend
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.event_bus.clone(),
        state.pending_requests.clone(),
    ));

    let executor = IterationExecutor::new(interaction);

    // Emit started event
    let _ = window.emit("iteration_continued", iteration_id.clone());

    // Execute in background
    let window_clone = window.app_handle().clone();
    let iteration_id_clone = iteration_id.clone();

    tokio::spawn(async move {
        println!("[GUI] Starting continue_iteration for iteration: {}", iteration_id_clone);
        match executor.continue_iteration(&mut project, &iteration_id_clone).await {
            Ok(_) => {
                println!("[GUI] continue_iteration completed successfully");
                let _ = window_clone.emit("iteration_completed", iteration_id_clone);
            }
            Err(e) => {
                println!("[GUI] continue_iteration failed: {}", e);
                let _ = window_clone.emit("iteration_failed", (iteration_id_clone, e.to_string()));
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn gui_retry_iteration(
    iteration_id: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_store = ProjectStore::new();

    let mut project = project_store.load().map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not initialized".to_string())?;

    // Create interaction backend
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.event_bus.clone(),
        state.pending_requests.clone(),
    ));

    let executor = IterationExecutor::new(interaction);

    // Emit started event
    let _ = window.emit("iteration_retrying", iteration_id.clone());

    // Execute in background
    let window_clone = window.app_handle().clone();
    let iteration_id_clone = iteration_id.clone();

    tokio::spawn(async move {
        println!("[GUI] Starting retry_iteration for iteration: {}", iteration_id_clone);
        match executor.retry_iteration(&mut project, &iteration_id_clone).await {
            Ok(_) => {
                println!("[GUI] retry_iteration completed successfully");
                let _ = window_clone.emit("iteration_completed", iteration_id_clone);
            }
            Err(e) => {
                println!("[GUI] retry_iteration failed: {}", e);
                let _ = window_clone.emit("iteration_failed", (iteration_id_clone, e.to_string()));
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn gui_delete_iteration(
    iteration_id: String,
    window: Window,
) -> Result<(), String> {
    let iteration_store = IterationStore::new();
    let project_store = ProjectStore::new();

    // Delete iteration
    iteration_store.delete(&iteration_id).map_err(|e| e.to_string())?;

    // Update project
    if let Ok(Some(mut project)) = project_store.load() {
        project.iterations.retain(|i| i.id != iteration_id);
        let _ = project_store.save(&project);
    }

    // Emit event
    let _ = window.emit("iteration_deleted", iteration_id);

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn project_to_info(project: &Project) -> ProjectInfo {
    ProjectInfo {
        id: project.id.clone(),
        name: project.name.clone(),
        created_at: project.created_at.to_rfc3339(),
        updated_at: project.updated_at.to_rfc3339(),
        current_iteration_id: project.current_iteration_id.clone(),
        iteration_count: project.iterations.len(),
    }
}

fn iteration_to_info(iteration: &Iteration) -> IterationInfo {
    IterationInfo {
        id: iteration.id.clone(),
        number: iteration.number,
        title: iteration.title.clone(),
        description: iteration.description.clone(),
        status: format!("{:?}", iteration.status),
        current_stage: iteration.current_stage.clone(),
        completed_stages: iteration.completed_stages.clone(),
        base_iteration_id: iteration.base_iteration_id.clone(),
        inheritance: format!("{:?}", iteration.inheritance),
        created_at: iteration.started_at.to_rfc3339(),
        completed_at: iteration.completed_at.map(|t| t.to_rfc3339()),
    }
}
