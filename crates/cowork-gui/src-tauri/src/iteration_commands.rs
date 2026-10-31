// Iteration Commands - New iteration-based architecture for GUI

use crate::AppState;
use crate::TauriBackend;
use cowork_core::domain::{Iteration, InheritanceMode, Project, IterationStatus};
use cowork_core::llm::create_llm_client;
use cowork_core::llm::config::load_config;
use cowork_core::persistence::{IterationStore, ProjectStore, MemoryStore};
use cowork_core::pipeline::IterationExecutor;
use tauri::{Emitter, Manager, State, Window};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tracing;

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

    // Update window title to reflect the newly initialized project name.
    // This handles the edge case where the workspace was opened before
    // `.cowork-v2/project.json` existed (title was "Cowork Forge" until now).
    if let Err(e) = window.set_title(&crate::format_window_title(Some(&project.name))) {
        tracing::warn!("[GUI] Failed to update window title after project init: {}", e);
    }

    // Update tray menu with the project name
    crate::tray::set_project_name(Some(project.name.clone()));

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

    // 继承模式解析收敛为 core 单实现（设计 Phase 0.2），与 PM 工具共享同一语义
    let inheritance = InheritanceMode::parse(&request.inheritance);

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

    // Create interaction backend (V2 - no event_bus parameter)
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.pending_requests.clone(),
    ));

    let executor = IterationExecutor::new(interaction);

    let model_config = load_config()
        .map_err(|e| format!("Failed to load LLM configuration: {}", e))?;

    let model = create_llm_client(&model_config.llm)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;

    // Emit started event
    let _ = window.emit("iteration_started", iteration_id.clone());
    crate::tray::set_working(true);

    // Execute in background
    let window_clone = window.app_handle().clone();
    let iteration_id_clone = iteration_id.clone();

    tokio::spawn(async move {
        match executor.execute(&mut project, &iteration_id_clone, None, Some(model)).await {
            Ok(_) => {
                crate::tray::set_working(false);
                let _ = window_clone.emit("iteration_completed", iteration_id_clone);
                crate::commands::autonomy::notify_iteration_finished();
            }
            Err(e) => {
                crate::tray::set_working(false);
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

    // Create interaction backend (V2 - no event_bus parameter)
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.pending_requests.clone(),
    ));

    let executor = IterationExecutor::new(interaction);

    let model_config = load_config()
        .map_err(|e| format!("Failed to load LLM configuration: {}", e))?;

    let model = create_llm_client(&model_config.llm)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;

    // Emit started event
    let _ = window.emit("iteration_continued", iteration_id.clone());
    crate::tray::set_working(true);

    // Execute in background
    let window_clone = window.app_handle().clone();
    let iteration_id_clone = iteration_id.clone();

    tokio::spawn(async move {
        tracing::info!("[GUI] Starting continue_iteration for iteration: {}", iteration_id_clone);
        match executor.continue_iteration(&mut project, &iteration_id_clone, Some(model)).await {
            Ok(_) => {
                crate::tray::set_working(false);
                tracing::info!("[GUI] continue_iteration completed successfully");
                let _ = window_clone.emit("iteration_completed", iteration_id_clone);
                crate::commands::autonomy::notify_iteration_finished();
            }
            Err(e) => {
                crate::tray::set_working(false);
                tracing::error!("[GUI] continue_iteration failed: {}", e);
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

    // Create interaction backend (V2 - no event_bus parameter)
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.pending_requests.clone(),
    ));

    let executor = IterationExecutor::new(interaction);

    let model_config = load_config()
        .map_err(|e| format!("Failed to load LLM configuration: {}", e))?;

    let model = create_llm_client(&model_config.llm)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;

    // Emit started event
    let _ = window.emit("iteration_retrying", iteration_id.clone());
    crate::tray::set_working(true);

    // Execute in background
    let window_clone = window.app_handle().clone();
    let iteration_id_clone = iteration_id.clone();

    tokio::spawn(async move {
        tracing::info!("[GUI] Starting retry_iteration for iteration: {}", iteration_id_clone);
        match executor.retry_iteration(&mut project, &iteration_id_clone, Some(model)).await {
            Ok(_) => {
                crate::tray::set_working(false);
                tracing::info!("[GUI] retry_iteration completed successfully");
                let _ = window_clone.emit("iteration_completed", iteration_id_clone);
                crate::commands::autonomy::notify_iteration_finished();
            }
            Err(e) => {
                crate::tray::set_working(false);
                tracing::error!("[GUI] retry_iteration failed: {}", e);
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

#[tauri::command]
pub async fn gui_get_project_knowledge(
    _project_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let memory_store = MemoryStore::new();
    let project_memory = memory_store.load_project_memory()
        .map_err(|e| format!("Failed to load project memory: {}", e))?;

    // Load project to get project name
    let project_store = ProjectStore::new();
    let project = project_store.load()
        .map_err(|e| format!("Failed to load project: {}", e))?
        .ok_or_else(|| "No project found".to_string())?;

    // Convert to frontend-friendly format
    let knowledge_list: Vec<serde_json::Value> = project_memory
        .iteration_knowledge
        .into_iter()
        .map(|(iteration_id, knowledge)| {
            serde_json::json!({
                "iteration_id": iteration_id,
                "title": format!("Iteration #{}", knowledge.iteration_number),
                "idea_summary": knowledge.idea_summary,
                "design_summary": knowledge.design_summary,
                "plan_summary": knowledge.plan_summary,
                "tech_stack": knowledge.tech_stack,
                "key_decisions": knowledge.key_decisions,
                "key_patterns": knowledge.key_patterns,
                "code_structure": knowledge.code_structure,
                "known_issues": knowledge.known_issues,
                "created_at": knowledge.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "project_id": project.id,
        "project_name": project.name,
        "knowledge_list": knowledge_list,
    }))
}

/// Returns the iteration IDs whose knowledge generation is currently running.
///
/// The frontend uses this on mount (and on manual refresh) to restore the
/// "generating" UI state after the Knowledge panel has been unmounted by a tab
/// switch, so a stale Generate button is never shown while work is in flight.
#[tauri::command]
pub async fn gui_get_knowledge_generation_status(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let guard = state.knowledge_generating.lock().map_err(|e| e.to_string())?;
    Ok(guard.iter().cloned().collect())
}

/// RAII guard that removes an iteration from the in-flight set on drop.
///
/// Guarantees the slot is released on *every* exit path — early `return`s in the
/// command, task completion, and task panics — so the UI can never get stuck
/// showing "generating" forever.
struct KnowledgeInflightGuard {
    generating: Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
    iteration_id: String,
}

impl KnowledgeInflightGuard {
    /// Claims the slot; returns `None` when generation is already running.
    fn claim(
        generating: Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
        iteration_id: String,
    ) -> Option<Self> {
        let inserted = {
            let mut guard = generating.lock().ok()?;
            guard.insert(iteration_id.clone())
        };
        if inserted {
            Some(Self { generating, iteration_id })
        } else {
            None
        }
    }
}

impl Drop for KnowledgeInflightGuard {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.generating.lock() {
            guard.remove(&self.iteration_id);
        }
    }
}

#[tauri::command]
pub async fn gui_regenerate_knowledge(
    iteration_id: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let iteration_store = IterationStore::new();

    // Load iteration
    let iteration = iteration_store.load(&iteration_id)
        .map_err(|e| e.to_string())?;

    // Check if iteration is completed
    if iteration.status != IterationStatus::Completed {
        return Err(format!(
            "Iteration is not completed (status: {:?})",
            iteration.status
        ));
    }

    // Claim the in-flight slot: refuse duplicate runs for the same iteration.
    // This also protects against a remounted panel firing Generate twice.
    let inflight = match KnowledgeInflightGuard::claim(
        state.knowledge_generating.clone(),
        iteration_id.clone(),
    ) {
        Some(guard) => guard,
        None => {
            tracing::warn!(
                "[GUI] Knowledge generation already running for iteration: {}",
                iteration_id
            );
            return Ok(());
        }
    };

    let model_config = load_config()
        .map_err(|e| format!("Failed to load LLM configuration: {}", e))?;

    let model = create_llm_client(&model_config.llm)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;

    // Create executor
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.pending_requests.clone(),
    ));

    let executor = IterationExecutor::new(interaction);

    // Emit started event
    let _ = window.emit("knowledge_regeneration_started", iteration_id.clone());

    // Execute in background
    let window_clone = window.app_handle().clone();
    let iteration_id_clone = iteration_id.clone();

    tokio::spawn(async move {
        // Owned by the task: released when the task finishes or unwinds.
        let _inflight = inflight;
        tracing::info!("[GUI] Starting knowledge regeneration for iteration: {}", iteration_id_clone);
        match executor.regenerate_iteration_knowledge(&iteration_id_clone, model).await {
            Ok(_) => {
                tracing::info!("[GUI] Knowledge regeneration completed successfully");
                let _ = window_clone.emit("knowledge_regeneration_completed", iteration_id_clone);
            }
            Err(e) => {
                tracing::error!("[GUI] Knowledge regeneration failed: {}", e);
                let _ = window_clone.emit("knowledge_regeneration_failed", (iteration_id_clone, e.to_string()));
            }
        }
        // `_inflight` drops here, *after* the event is emitted, so a panel that
        // remounts upon receiving the event still sees the running state and
        // never flashes a stale Generate button.
    });

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
