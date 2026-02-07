// GUI-specific commands for enhanced functionality
use super::gui_types::*;
use super::gui_types::FileReadResult;
use crate::AppState;
use crate::preview_server::PreviewServerManager;
use crate::project_runner::ProjectRunner;
use cowork_core::storage::*;
use cowork_core::persistence::IterationStore;
use cowork_core::data::{Requirements, DesignSpec, ImplementationPlan};
use tauri::{State, Window};
use std::fs;
use std::path::{Path, PathBuf};

// Global instances
lazy_static::lazy_static! {
    static ref PREVIEW_SERVER_MANAGER: PreviewServerManager = PreviewServerManager::new();
    static ref PROJECT_RUNNER: ProjectRunner = ProjectRunner::new();
}

// ============================================================================
// Initialization
// ============================================================================

pub fn init_app_handle(handle: tauri::AppHandle) {
    PROJECT_RUNNER.set_app_handle(handle);
}

// ============================================================================
// Get Iteration Artifacts (New V2 API)
// ============================================================================

#[tauri::command]
pub async fn get_iteration_artifacts(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<SessionArtifacts, String> {
    println!("[GUI] Getting artifacts for iteration: {}", iteration_id);

    let iteration_store = IterationStore::new();
    let _iteration = iteration_store.load(&iteration_id)
        .map_err(|e| format!("Failed to load iteration: {}", e))?;

    // Get iteration artifacts directory
    let iteration_dir = iteration_store.iteration_path(&iteration_id)
        .map_err(|e| format!("Failed to get iteration dir: {}", e))?;
    let artifacts_dir = iteration_dir.join("artifacts");

    // Load artifacts from .cowork-v2 structure
    let idea = fs::read_to_string(artifacts_dir.join("idea.md")).ok();
    let prd = fs::read_to_string(artifacts_dir.join("prd.md")).ok();
    let design_raw = fs::read_to_string(artifacts_dir.join("design.md")).ok();
    let plan_raw = fs::read_to_string(artifacts_dir.join("plan.md")).ok();
    let delivery_report = fs::read_to_string(artifacts_dir.join("delivery_report.md")).ok();
    let _check_report = fs::read_to_string(artifacts_dir.join("check_report.md")).ok();

    // Load workspace code files if available
    let workspace = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;
    let code_files = if workspace.exists() {
        collect_files(&workspace)
    } else {
        vec![]
    };

    // Parse PRD content into structured requirements
    let requirements = prd.as_ref().and_then(|content| {
        use chrono::Utc;
        
        let parsed_reqs = parse_requirements_from_prd(content);
        
        Some(Requirements {
            schema_version: "1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            requirements: parsed_reqs,
        })
    });

    // Parse design content - return None for now as DesignSpec has complex structure
    let design: Option<DesignSpec> = None; // TODO: Parse from markdown

    // Parse plan content - return None for now as ImplementationPlan has complex structure
    let plan: Option<ImplementationPlan> = None; // TODO: Parse from markdown

    Ok(SessionArtifacts {
        session_id: iteration_id.clone(),
        idea,
        requirements,
        features: None,
        design,
        plan,
        design_raw,
        plan_raw,
        code_files,
        delivery_report,
    })
}

// ============================================================================
// Get Session Artifacts (Legacy - for backward compatibility)
// ============================================================================

#[tauri::command]
pub async fn get_session_artifacts(
    session_id: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<SessionArtifacts, String> {
    // Try to load as iteration first (V2)
    let iteration_store = IterationStore::new();
    if iteration_store.exists(&session_id) {
        return get_iteration_artifacts(session_id, window, state).await;
    }

    // Fall back to legacy session-based loading (V1)
    println!("[GUI] Getting artifacts for session (legacy): {}", session_id);

    let session_dir = get_session_dir(&session_id)
        .map_err(|e| format!("Failed to get session dir: {}", e))?;

    let code_dir = session_dir.join("code");

    let idea = load_idea(&session_id).ok();
    let requirements = load_requirements(&session_id).ok();
    let features = load_feature_list(&session_id).ok();
    let design = load_design_spec(&session_id).ok();
    let plan = load_implementation_plan(&session_id).ok();

    let code_files = if code_dir.exists() {
        collect_files(&code_dir)
    } else {
        vec![]
    };

    let delivery_report = fs::read_to_string(session_dir.join("delivery_report.md")).ok();

    Ok(SessionArtifacts {
        session_id,
        idea,
        requirements,
        features,
        design,
        plan,
        design_raw: None,
        plan_raw: None,
        code_files,
        delivery_report,
    })
}

// ============================================================================
// File Operations (Iteration-based V2 API)
// ============================================================================

// Legacy alias for read_iteration_file
#[tauri::command]
pub async fn read_file_content(
    session_id: String,
    file_path: String,
    offset: Option<usize>,
    limit: Option<usize>,
    window: Window,
    state: State<'_, AppState>,
) -> Result<FileReadResult, String> {
    // Delegate to the V2 implementation
    read_iteration_file(session_id, file_path, offset, limit, window, state).await
}

#[tauri::command]
pub async fn read_iteration_file(
    iteration_id: String,
    file_path: String,
    offset: Option<usize>,
    limit: Option<usize>,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<FileReadResult, String> {
    println!("[GUI] Reading file for iteration {}: {}", iteration_id, file_path);

    let iteration_store = IterationStore::new();
    let workspace = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;

    let full_path = workspace.join(&file_path);

    // Get file metadata
    let metadata = fs::metadata(&full_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    let file_size = metadata.len() as usize;
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB limit for full read

    // If file is too large or offset/limit specified, read in chunks
    if file_size > MAX_FILE_SIZE || offset.is_some() || limit.is_some() {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(1024 * 1024); // Default 1MB chunks
        
        let mut file = fs::File::open(&full_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        use std::io::{Seek, Read};
        
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|e| format!("Failed to seek in file: {}", e))?;
        
        let mut buffer = vec![0; limit.min(file_size - offset)];
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        buffer.truncate(bytes_read);
        
        let content = String::from_utf8_lossy(&buffer).to_string();
        
        Ok(FileReadResult {
            content,
            offset: offset as u64,
            total_size: file_size as u64,
            is_partial: true,
        })
    } else {
        // Read full file for small files
        let content = fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        Ok(FileReadResult {
            content,
            offset: 0,
            total_size: file_size as u64,
            is_partial: false,
        })
    }
}

#[tauri::command]
pub async fn save_file_content(
    _session_id: String,
    file_path: String,
    content: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] Saving file: {}", file_path);

    let project_root = cowork_core::storage::get_project_root()
        .map_err(|e| format!("Failed to get project root: {}", e))?;

    let full_path = project_root.join(&file_path);

    // Create parent directories if needed
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    fs::write(&full_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    println!("[GUI] File saved successfully");
    Ok(())
}

#[tauri::command]
pub async fn save_iteration_file(
    iteration_id: String,
    file_path: String,
    content: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] Saving file for iteration {}: {}", iteration_id, file_path);

    let iteration_store = IterationStore::new();
    let workspace = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;

    let full_path = workspace.join(&file_path);

    // Create parent directories if needed
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    fs::write(&full_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    println!("[GUI] File saved successfully");
    Ok(())
}

// ============================================================================
// File Tree
// ============================================================================

#[tauri::command]
pub async fn get_file_tree(
    session_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<FileTreeNode, String> {
    println!("[GUI] Getting file tree for session: {}", session_id);

    // Use project root directory instead of session code directory
    let project_root = cowork_core::storage::get_project_root()
        .map_err(|e| format!("Failed to get project root: {}", e))?;

    if !project_root.exists() {
        return Ok(FileTreeNode {
            name: project_root.file_name().unwrap_or(project_root.as_os_str()).to_string_lossy().to_string(),
            path: ".".to_string(),
            is_dir: true,
            children: Some(vec![]),
            is_expanded: true,
            language: None,
        });
    }

    build_file_tree(&project_root, &project_root, 0)
        .map_err(|e| format!("Failed to build file tree: {}", e))
}

#[tauri::command]
pub async fn get_iteration_file_tree(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<FileTreeNode, String> {
    println!("[GUI] Getting file tree for iteration: {}", iteration_id);

    let iteration_store = IterationStore::new();
    let workspace = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;

    if !workspace.exists() {
        return Ok(FileTreeNode {
            name: workspace.file_name().unwrap_or(workspace.as_os_str()).to_string_lossy().to_string(),
            path: ".".to_string(),
            is_dir: true,
            children: Some(vec![]),
            is_expanded: true,
            language: None,
        });
    }

    build_file_tree(&workspace, &workspace, 0)
        .map_err(|e| format!("Failed to build file tree: {}", e))
}

// ============================================================================
// Preview and Run Commands
// ============================================================================

#[tauri::command]
pub async fn start_preview(
    session_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<PreviewInfo, String> {
    println!("[GUI] Starting preview for session: {}", session_id);

    let project_root = cowork_core::storage::get_project_root()
        .map_err(|e| format!("Failed to get project root: {}", e))?;

    if !project_root.exists() {
        return Err(format!("Project directory not found: {}", project_root.display()));
    }

    PREVIEW_SERVER_MANAGER.start(session_id, project_root).await
}

#[tauri::command]
pub async fn stop_preview(
    session_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] Stopping preview for session: {}", session_id);
    PREVIEW_SERVER_MANAGER.stop(session_id).await
}

#[tauri::command]
pub async fn start_project(
    session_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<RunInfo, String> {
    println!("[GUI] Starting project for session: {}", session_id);

    // Use project root directory
    let project_root = cowork_core::storage::get_project_root()
        .map_err(|e| format!("Failed to get project root: {}", e))?;

    let command = detect_start_command(&project_root)?;

    println!("[GUI] Detected start command: {}", command);

    let command_clone = command.clone();
    let pid = PROJECT_RUNNER.start(session_id, command).await?;

    Ok(RunInfo {
        status: RunStatus::Running,
        process_id: Some(pid),
        command: Some(command_clone),
    })
}

#[tauri::command]
pub async fn stop_project(
    session_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] Stopping project for session: {}", session_id);
    PROJECT_RUNNER.stop(session_id).await
}

// ============================================================================
// Iteration-based Preview and Run Commands (V2 API)
// ============================================================================

#[tauri::command]
pub async fn start_iteration_preview(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<PreviewInfo, String> {
    println!("[GUI] Starting preview for iteration: {}", iteration_id);

    // Use workspace directory for preview (working code location)
    // This ensures consistency with Code editor
    let iteration_store = IterationStore::new();
    let workspace = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;

    if !workspace.exists() {
        return Err(format!("Workspace directory not found: {}", workspace.display()));
    }

    PREVIEW_SERVER_MANAGER.start(iteration_id, workspace).await
}

#[tauri::command]
pub async fn stop_iteration_preview(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] Stopping preview for iteration: {}", iteration_id);
    PREVIEW_SERVER_MANAGER.stop(iteration_id).await
}

#[tauri::command]
pub async fn start_iteration_project(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<RunInfo, String> {
    println!("[GUI] Starting project for iteration: {}", iteration_id);

    // Use workspace directory for running (working code location)
    // This ensures consistency with Code editor and Preview
    let iteration_store = IterationStore::new();
    let workspace = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;

    let command = detect_start_command(&workspace)?;

    println!("[GUI] Detected start command: {}", command);

    let command_clone = command.clone();
    let pid = PROJECT_RUNNER.start(iteration_id.clone(), command).await?;

    Ok(RunInfo {
        status: RunStatus::Running,
        process_id: Some(pid),
        command: Some(command_clone),
    })
}

#[tauri::command]
pub async fn stop_iteration_project(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] Stopping project for iteration: {}", iteration_id);
    PROJECT_RUNNER.stop(iteration_id).await
}

#[tauri::command]
pub async fn execute_project_command(
    session_id: String,
    command: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<CommandResult, String> {
    println!("[GUI] Executing command for session {}: {}", session_id, command);

    let result = PROJECT_RUNNER.execute_command(session_id, command).await?;

    Ok(CommandResult {
        status: "completed".to_string(),
        exit_code: Some(0),
        stdout: result,
        stderr: String::new(),
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

fn collect_files(dir: &Path) -> Vec<FileInfo> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let metadata = entry.metadata().ok();

            if let Some(meta) = metadata {
                let name = path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                let relative_path = path.strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                let language = detect_language(&name);

                files.push(FileInfo {
                    path: relative_path,
                    name,
                    size: meta.len(),
                    is_dir: meta.is_dir(),
                    language,
                    modified_at: meta.modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs().to_string()),
                });
            }
        }
    }

    files
}

fn build_file_tree(dir: &Path, root: &Path, depth: usize) -> Result<FileTreeNode, String> {
    let name = dir.file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let path = dir.strip_prefix(root)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let is_dir = dir.is_dir();

    let children = if is_dir && depth < 10 {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            // Directories first
            b_is_dir.cmp(&a_is_dir)
                .then_with(|| a.file_name().cmp(&b.file_name()))
        });

        Some(entries.into_iter()
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                // Skip hidden files
                if name.starts_with('.') {
                    return None;
                }
                build_file_tree(&path, root, depth + 1).ok()
            })
            .collect())
    } else {
        None
    };

    let language = if !is_dir {
        detect_language(&name)
    } else {
        None
    };

    Ok(FileTreeNode {
        name,
        path,
        is_dir,
        children,
        is_expanded: depth < 3,
        language,
    })
}

fn detect_language(filename: &str) -> Option<String> {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("rs") => Some("rust".to_string()),
        Some("js") | Some("jsx") => Some("javascript".to_string()),
        Some("ts") | Some("tsx") => Some("typescript".to_string()),
        Some("py") => Some("python".to_string()),
        Some("html") => Some("html".to_string()),
        Some("css") | Some("scss") | Some("sass") => Some("css".to_string()),
        Some("json") => Some("json".to_string()),
        Some("md") => Some("markdown".to_string()),
        Some("xml") => Some("xml".to_string()),
        Some("toml") => Some("toml".to_string()),
        Some("yaml") | Some("yml") => Some("yaml".to_string()),
        _ => None,
    }
}

// ============================================================================
// Project Detection
// ============================================================================

fn detect_start_command(code_dir: &Path) -> Result<String, String> {
    let package_json = code_dir.join("package.json");
    let cargo_toml = code_dir.join("Cargo.toml");
    let index_html = code_dir.join("index.html");

    // Check for Node.js/React/Vue projects
    if package_json.exists() {
        let pkg = fs::read_to_string(&package_json)
            .map_err(|e| format!("Failed to read package.json: {}", e))?;

        let pkg_json: serde_json::Value = serde_json::from_str(&pkg)
            .map_err(|e| format!("Failed to parse package.json: {}", e))?;

        // Check for scripts
        if let Some(scripts) = pkg_json.get("scripts").and_then(|s| s.as_object()) {
            // Priority: dev -> start -> serve -> build
            for script_name in &["dev", "start", "serve", "build"] {
                if scripts.contains_key(*script_name) {
                    return Ok(format!("npm run {}", script_name));
                }
            }
        }
    }

    // Check for Rust projects
    if cargo_toml.exists() {
        // Check if it's a binary project
        let cargo_content = fs::read_to_string(&cargo_toml)
            .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

        if cargo_content.contains("[[bin]]") || cargo_content.contains("[[lib]]") {
            return Ok("cargo run".to_string());
        }
    }

    // Check for static HTML
    if index_html.exists() {
        return Ok("python -m http.server 8000".to_string());
    }

    Err("Unable to detect project type".to_string())
}

// ============================================================================
// Memory Commands
// ============================================================================

#[tauri::command]
pub async fn query_memory_index(
    query_type: String,
    category: String,
    stage: Option<String>,
    limit: i64,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Querying memory index: query_type={}, category={}, stage={:?}, limit={}", 
             query_type, category, stage, limit);

    use cowork_core::tools::QueryMemoryIndexTool;
    use adk_core::Tool;

    let tool = QueryMemoryIndexTool;
    
    let args = serde_json::json!({
        "query_type": query_type,
        "category": category,
        "stage": stage,
        "limit": limit
    });

    // Create a simple tool context - we don't need full context for memory tools
    // Since our memory tools use _ctx parameter, they don't actually use it
    let ctx = std::sync::Arc::new(DummyToolContext);
    
    tool.execute(ctx, args)
        .await
        .map_err(|e| format!("Failed to query memory index: {}", e))
}

#[tauri::command]
pub async fn load_memory_detail(
    memory_id: String,
    file: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Loading memory detail: memory_id={}, file={}", memory_id, file);

    use cowork_core::tools::LoadMemoryDetailTool;
    use adk_core::Tool;

    let tool = LoadMemoryDetailTool;
    
    let args = serde_json::json!({
        "memory_id": memory_id,
        "file": file
    });

    let ctx = std::sync::Arc::new(DummyToolContext);
    
    tool.execute(ctx, args)
        .await
        .map_err(|e| format!("Failed to load memory detail: {}", e))
}

#[tauri::command]
pub async fn save_session_memory(
    memory_type: String,
    title: String,
    summary: String,
    content: String,
    stage: String,
    category: Option<String>,
    impact: Option<String>,
    tags: Option<Vec<String>>,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Saving session memory: memory_type={}, title={}", memory_type, title);

    use cowork_core::tools::SaveSessionMemoryTool;
    use adk_core::Tool;

    let tool = SaveSessionMemoryTool;
    
    let args = serde_json::json!({
        "memory_type": memory_type,
        "title": title,
        "summary": summary,
        "content": content,
        "stage": stage,
        "category": category.unwrap_or("general".to_string()),
        "impact": impact.unwrap_or("medium".to_string()),
        "tags": tags.unwrap_or_default()
    });

    let ctx = std::sync::Arc::new(DummyToolContext);
    
    tool.execute(ctx, args)
        .await
        .map_err(|e| format!("Failed to save session memory: {}", e))
}

#[tauri::command]
pub async fn promote_to_project_memory(
    memory_id: String,
    reason: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Promoting to project memory: memory_id={}, reason={}", memory_id, reason);

    use cowork_core::tools::PromoteToProjectMemoryTool;
    use adk_core::Tool;

    let tool = PromoteToProjectMemoryTool;
    
    let args = serde_json::json!({
        "memory_id": memory_id,
        "reason": reason
    });

    let ctx = std::sync::Arc::new(DummyToolContext);
    
    tool.execute(ctx, args)
        .await
        .map_err(|e| format!("Failed to promote to project memory: {}", e))
}

#[tauri::command]
pub async fn get_memory_context(
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Getting memory context");

    use cowork_core::tools::GetMemoryContextTool;
    use adk_core::Tool;

    let tool = GetMemoryContextTool;
    
    let args = serde_json::json!({});

    let ctx = std::sync::Arc::new(DummyToolContext);
    
    tool.execute(ctx, args)
        .await
        .map_err(|e| format!("Failed to get memory context: {}", e))
}

// ============================================================================
// Dummy Tool Context for GUI Commands
// ============================================================================

use adk_core::{ToolContext, CallbackContext, ReadonlyContext, EventActions, AdkError};

/// Dummy tool context used for GUI commands that don't need full tool context
pub struct DummyToolContext;

impl CallbackContext for DummyToolContext {
    fn artifacts(&self) -> Option<std::sync::Arc<dyn adk_core::Artifacts>> {
        None
    }
}

impl ReadonlyContext for DummyToolContext {
    fn invocation_id(&self) -> &str {
        "dummy_invocation"
    }
    
    fn session_id(&self) -> &str {
        "dummy_session"
    }
    
    fn agent_name(&self) -> &str {
        "dummy_agent"
    }
    
    fn user_id(&self) -> &str {
        "dummy_user"
    }
    
    fn app_name(&self) -> &str {
        "cowork_gui"
    }
    
    fn branch(&self) -> &str {
        "main"
    }
    
    fn user_content(&self) -> &adk_core::Content {
        use std::sync::OnceLock;
        static CONTENT: OnceLock<adk_core::Content> = OnceLock::new();
        CONTENT.get_or_init(|| adk_core::Content::new("user"))
    }
}

#[async_trait::async_trait]
impl ToolContext for DummyToolContext {
    fn function_call_id(&self) -> &str {
        "dummy"
    }

    fn actions(&self) -> EventActions {
        EventActions::default()
    }

    fn set_actions(&self, _actions: EventActions) {
        // No-op
    }

    async fn search_memory<'life0: 'async_trait, 'life1: 'async_trait>(
        &'life0 self,
        _query: &'life1 str,
    ) -> Result<Vec<adk_core::MemoryEntry>, AdkError> {
        Ok(vec![])
    }
}

// ============================================================================
// Code Formatting Commands
// ============================================================================

#[tauri::command]
pub async fn format_code(
    _session_id: String,
    _file_path: Option<String>,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<FormatResult, String> {
    println!("[GUI] Formatting code in project root");

    let project_root = cowork_core::storage::get_project_root()
        .map_err(|e| format!("Failed to get project root: {}", e))?;

    let code_dir = &project_root;
    
    if !code_dir.exists() {
        return Err("Project directory not found".to_string());
    }

    // Detect project type and run appropriate formatter
    let package_json = code_dir.join("package.json");
    let cargo_toml = code_dir.join("Cargo.toml");

    let mut formatted_files = Vec::new();
    let mut errors = Vec::new();

    // Check for Prettier (JavaScript/TypeScript)
    if package_json.exists() {
        // Try to run prettier
        let output = tokio::process::Command::new("npx")
            .args(["prettier", "--write", "."])
            .current_dir(&code_dir)
            .output()
            .await;

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                if output.status.success() {
                    // Parse formatted files from output
                    for line in stdout.lines() {
                        if line.trim().len() > 0 {
                            formatted_files.push(line.to_string());
                        }
                    }
                } else {
                    errors.push(format!("Prettier failed: {}", stderr));
                }
            }
            Err(e) => {
                errors.push(format!("Failed to run prettier: {}", e));
            }
        }
    }

    // Check for rustfmt (Rust)
    if cargo_toml.exists() {
        let output = tokio::process::Command::new("cargo")
            .args(["fmt"])
            .current_dir(&code_dir)
            .output()
            .await;

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                
                if output.status.success() {
                    // rustfmt doesn't output formatted files by default
                    formatted_files.push("All Rust files formatted".to_string());
                } else {
                    errors.push(format!("rustfmt failed: {}", stdout));
                }
            }
            Err(e) => {
                errors.push(format!("Failed to run rustfmt: {}", e));
            }
        }
    }

    // If no formatter found
    if formatted_files.is_empty() && errors.is_empty() {
        return Err("No supported formatter found for this project type".to_string());
    }

    let success = errors.is_empty();
    Ok(FormatResult {
        formatted_files,
        errors,
        success,
    })
}

#[tauri::command]
pub async fn check_formatter_available(
    _session_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<FormatterAvailability, String> {
    println!("[GUI] Checking formatter availability in project root");

    let project_root = cowork_core::storage::get_project_root()
        .map_err(|e| format!("Failed to get project root: {}", e))?;

    let code_dir = &project_root;
    
    let mut prettier_available = false;
    let mut rustfmt_available = false;

    // Check for Prettier
    let package_json = code_dir.join("package.json");
    if package_json.exists() {
        let output = tokio::process::Command::new("npx")
            .args(["prettier", "--version"])
            .output()
            .await;

        prettier_available = output.is_ok() && output.unwrap().status.success();
    }

    // Check for rustfmt
    let cargo_toml = code_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let output = tokio::process::Command::new("cargo")
            .args(["fmt", "--version"])
            .output()
            .await;

        rustfmt_available = output.is_ok() && output.unwrap().status.success();
    }

    Ok(FormatterAvailability {
        prettier: prettier_available,
        rustfmt: rustfmt_available,
    })
}

// ============================================================================
// Project Template Commands
// ============================================================================

#[tauri::command]
pub async fn get_templates(
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<Vec<ProjectTemplate>, String> {
    println!("[GUI] Getting templates");

    // Get templates directory
    let templates_dir = get_templates_dir()?;
    
    if !templates_dir.exists() {
        return Ok(vec![]);
    }

    let mut templates = Vec::new();

    // Read built-in templates
    let built_in_templates = get_built_in_templates();
    templates.extend(built_in_templates);

    // Read custom templates
    if let Ok(entries) = fs::read_dir(&templates_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(template) = serde_json::from_str::<ProjectTemplate>(&content) {
                                templates.push(template);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(templates)
}

#[tauri::command]
pub async fn export_template(
    _session_id: String,
    name: String,
    description: String,
    category: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<ProjectTemplate, String> {
    println!("[GUI] Exporting template from project root");

    let project_root = cowork_core::storage::get_project_root()
        .map_err(|e| format!("Failed to get project root: {}", e))?;

    let code_dir = &project_root;
    
    if !code_dir.exists() {
        return Err("Project directory not found".to_string());
    }

    // Collect all files
    let mut files = Vec::new();
    collect_template_files(&code_dir, &mut files)?;

    // Create template
    let template_id = format!("template-{}", chrono::Utc::now().timestamp_millis());
    let template = ProjectTemplate {
        id: template_id.clone(),
        name,
        description,
        category,
        technology_stack: vec![], // TODO: detect from files
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        is_built_in: false,
        files,
        config: TemplateConfig {
            variables: vec![],
            post_creation_commands: vec![],
        },
    };

    // Save template
    let templates_dir = get_templates_dir()?;
    fs::create_dir_all(&templates_dir)
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;

    let template_file = templates_dir.join(format!("{}.json", template_id));
    fs::write(&template_file, serde_json::to_string_pretty(&template).unwrap())
        .map_err(|e| format!("Failed to save template: {}", e))?;

    Ok(template)
}

#[tauri::command]
pub async fn import_template(
    template_data: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<ProjectTemplate, String> {
    println!("[GUI] Importing template");

    let template: ProjectTemplate = serde_json::from_str(&template_data)
        .map_err(|e| format!("Failed to parse template data: {}", e))?;

    // Validate template
    if template.id.is_empty() || template.name.is_empty() {
        return Err("Invalid template: missing id or name".to_string());
    }

    // Save template
    let templates_dir = get_templates_dir()?;
    fs::create_dir_all(&templates_dir)
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;

    let template_file = templates_dir.join(format!("{}.json", template.id));
    fs::write(&template_file, serde_json::to_string_pretty(&template).unwrap())
        .map_err(|e| format!("Failed to save template: {}", e))?;

    Ok(template)
}

#[tauri::command]
pub async fn delete_template(
    template_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] Deleting template: {}", template_id);

    let templates_dir = get_templates_dir()?;
    let template_file = templates_dir.join(format!("{}.json", template_id));

    if !template_file.exists() {
        return Err("Template not found".to_string());
    }

    fs::remove_file(&template_file)
        .map_err(|e| format!("Failed to delete template: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn apply_template(
    template_id: String,
    variables: serde_json::Value,
    target_dir: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    println!("[GUI] Applying template: {} to {}", template_id, target_dir);

    // Get template
    let templates_dir = get_templates_dir()?;
    let template_file = templates_dir.join(format!("{}.json", template_id));

    if !template_file.exists() {
        return Err("Template not found".to_string());
    }

    let template_content = fs::read_to_string(&template_file)
        .map_err(|e| format!("Failed to read template: {}", e))?;

    let template: ProjectTemplate = serde_json::from_str(&template_content)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    // Create target directory
    let target_path = Path::new(&target_dir);
    fs::create_dir_all(target_path)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    // Apply template files
    let mut created_files = Vec::new();
    for file in &template.files {
        let file_path = target_path.join(&file.path);
        
        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Replace variables in content
        let content = replace_template_variables(&file.content, &variables, &template.config.variables);

        // Write file
        fs::write(&file_path, content)
            .map_err(|e| format!("Failed to write file {}: {}", file.path, e))?;

        created_files.push(file.path.clone());
    }

    Ok(created_files)
}

// ============================================================================
// Template Helper Functions
// ============================================================================

fn get_templates_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?;
    
    let templates_dir = config_dir.join("CoworkCreative").join("templates");
    Ok(templates_dir)
}

fn get_built_in_templates() -> Vec<ProjectTemplate> {
    // Return built-in templates
    vec![
        ProjectTemplate {
            id: "react-basic".to_string(),
            name: "React Basic".to_string(),
            description: "Basic React project structure".to_string(),
            category: "Frontend".to_string(),
            technology_stack: vec!["React".to_string(), "JavaScript".to_string()],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            is_built_in: true,
            files: vec![],
            config: TemplateConfig {
                variables: vec![],
                post_creation_commands: vec!["npm install".to_string()],
            },
        },
        ProjectTemplate {
            id: "rust-cli".to_string(),
            name: "Rust CLI".to_string(),
            description: "Basic Rust CLI project structure".to_string(),
            category: "Backend".to_string(),
            technology_stack: vec!["Rust".to_string()],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            is_built_in: true,
            files: vec![],
            config: TemplateConfig {
                variables: vec![],
                post_creation_commands: vec!["cargo build".to_string()],
            },
        },
    ]
}

fn collect_template_files(dir: &Path, files: &mut Vec<TemplateFile>) -> Result<(), String> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let relative_path = path.strip_prefix(dir)
                            .map_err(|e| format!("Failed to get relative path: {}", e))?
                            .to_string_lossy()
                            .to_string();
                        
                        files.push(TemplateFile {
                            path: relative_path,
                            content,
                            is_template: true,
                        });
                    }
                } else if meta.is_dir() {
                    collect_template_files(&path, files)?;
                }
            }
        }
    }
    Ok(())
}

fn replace_template_variables(content: &str, variables: &serde_json::Value, config_vars: &[TemplateVariable]) -> String {
    let mut result = content.to_string();
    
    // Replace variables
    for var in config_vars {
        let placeholder = format!("{{{{{}}}}}", var.name);
        let value = variables.get(&var.name)
            .and_then(|v| v.as_str())
            .unwrap_or(&var.default_value);
        result = result.replace(&placeholder, value);
    }
    
    result
}

// ============================================================================
// PRD Parsing Helper Functions
// ============================================================================

use cowork_core::data::{Requirement, Priority, RequirementCategory};

/// Parse requirements from PRD markdown content
fn parse_requirements_from_prd(content: &str) -> Vec<Requirement> {
    let mut requirements = Vec::new();
    let mut req_id_counter = 1;
    
    // Parse functional requirements section
    if let Some(func_section) = extract_section(content, "Functional Requirements", "Non-Functional") {
        for line in func_section.lines() {
            let line = line.trim();
            if line.starts_with('-') || line.starts_with("*") || 
               (line.len() > 3 && line.chars().nth(0).map(|c| c.is_numeric()).unwrap_or(false)) {
                // Extract requirement text
                let text = line.trim_start_matches(|c: char| !c.is_alphabetic())
                    .trim_start_matches("-")
                    .trim_start_matches("*")
                    .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ')')
                    .trim();
                
                if !text.is_empty() {
                    let (title, description) = if text.contains(':') {
                        let parts: Vec<&str> = text.splitn(2, ':').collect();
                        (parts[0].trim().to_string(), parts[1].trim().to_string())
                    } else if text.len() > 100 {
                        // Long text - use first sentence as title
                        let first_sentence = text.split('.').next().unwrap_or(&text);
                        (first_sentence.to_string() + ".", text.to_string())
                    } else {
                        (text.to_string(), String::new())
                    };
                    
                    requirements.push(Requirement {
                        id: format!("REQ-{:03}", req_id_counter),
                        title,
                        description,
                        priority: Priority::High,
                        category: RequirementCategory::Functional,
                        acceptance_criteria: vec![],
                        related_features: vec![],
                    });
                    req_id_counter += 1;
                }
            }
        }
    }
    
    // Parse non-functional requirements section
    if let Some(non_func_section) = extract_section(content, "Non-Functional Requirements", "UI/UX") {
        for line in non_func_section.lines() {
            let line = line.trim();
            if line.starts_with('-') || line.starts_with("*") || 
               (line.len() > 3 && line.chars().nth(0).map(|c| c.is_numeric()).unwrap_or(false)) {
                let text = line.trim_start_matches(|c: char| !c.is_alphabetic())
                    .trim_start_matches("-")
                    .trim_start_matches("*")
                    .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ')')
                    .trim();
                
                if !text.is_empty() {
                    let (title, description) = if text.contains(':') {
                        let parts: Vec<&str> = text.splitn(2, ':').collect();
                        (parts[0].trim().to_string(), parts[1].trim().to_string())
                    } else if text.len() > 100 {
                        let first_sentence = text.split('.').next().unwrap_or(&text);
                        (first_sentence.to_string() + ".", text.to_string())
                    } else {
                        (text.to_string(), String::new())
                    };
                    
                    requirements.push(Requirement {
                        id: format!("REQ-{:03}", req_id_counter),
                        title,
                        description,
                        priority: Priority::Medium,
                        category: RequirementCategory::NonFunctional,
                        acceptance_criteria: vec![],
                        related_features: vec![],
                    });
                    req_id_counter += 1;
                }
            }
        }
    }
    
    // If no structured requirements found, extract from user stories section
    if requirements.is_empty() {
        if let Some(stories_section) = extract_section(content, "User Stories", "Functional") {
            for line in stories_section.lines() {
                let line = line.trim();
                if line.to_lowercase().starts_with("as a") || 
                   line.to_lowercase().starts_with("- as a") ||
                   line.to_lowercase().starts_with("* as a") {
                    let text = line.trim_start_matches("-").trim_start_matches("*").trim();
                    if !text.is_empty() {
                        requirements.push(Requirement {
                            id: format!("REQ-{:03}", req_id_counter),
                            title: text.chars().take(80).collect::<String>() + "...",
                            description: text.to_string(),
                            priority: Priority::High,
                            category: RequirementCategory::Functional,
                            acceptance_criteria: vec![],
                            related_features: vec![],
                        });
                        req_id_counter += 1;
                    }
                }
            }
        }
    }
    
    // If still empty, create a single requirement from PRD summary
    if requirements.is_empty() {
        // Extract overview/goals section as a requirement
        let summary = content.lines()
            .take(10)
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        
        if !summary.is_empty() {
            requirements.push(Requirement {
                id: "REQ-001".to_string(),
                title: "PRD Requirements".to_string(),
                description: summary,
                priority: Priority::High,
                category: RequirementCategory::Functional,
                acceptance_criteria: vec![],
                related_features: vec![],
            });
        }
    }
    
    requirements
}

/// Extract a section from markdown content
fn extract_section(content: &str, section_start: &str, section_end: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut section_lines = Vec::new();
    
    for line in &lines {
        let line_lower = line.to_lowercase();
        
        // Check for section start
        if line_lower.contains(&section_start.to_lowercase()) && 
           (line.starts_with("#") || line.starts_with("**")) {
            in_section = true;
            continue;
        }
        
        // Check for section end
        if in_section && 
           (line_lower.contains(&section_end.to_lowercase()) ||
            line.starts_with("# ") || 
            line.starts_with("## ")) &&
           !line_lower.contains(&section_start.to_lowercase()) {
            break;
        }
        
        if in_section {
            section_lines.push(*line);
        }
    }
    
    if section_lines.is_empty() {
        None
    } else {
        Some(section_lines.join("\n"))
    }
}
