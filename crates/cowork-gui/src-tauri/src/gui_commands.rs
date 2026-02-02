// GUI-specific commands for enhanced functionality
use super::gui_types::*;
use crate::AppState;
use crate::preview_server::PreviewServerManager;
use crate::project_runner::ProjectRunner;
use cowork_core::storage::*;
use tauri::{State, Window};
use std::fs;
use std::path::Path;
use std::sync::Arc;

// Global instances
lazy_static::lazy_static! {
    static ref PREVIEW_SERVER_MANAGER: PreviewServerManager = PreviewServerManager::new();
    static ref PROJECT_RUNNER: ProjectRunner = ProjectRunner::new();
}

// ============================================================================
// Get Session Artifacts
// ============================================================================

#[tauri::command]
pub async fn get_session_artifacts(
    session_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<SessionArtifacts, String> {
    println!("[GUI] Getting artifacts for session: {}", session_id);

    // Get session directory
    let session_dir = get_session_dir(&session_id)
        .map_err(|e| format!("Failed to get session dir: {}", e))?;

    let code_dir = session_dir.join("code");

    // Load idea
    let idea = load_idea(&session_id).ok();

    // Load requirements
    let requirements = load_requirements(&session_id).ok();

    // Load features
    let features = load_feature_list(&session_id).ok();

    // Load design
    let design = load_design_spec(&session_id).ok();

    // Load plan
    let plan = load_implementation_plan(&session_id).ok();

    // Load code files
    let code_files = if code_dir.exists() {
        collect_files(&code_dir)
    } else {
        vec![]
    };

    // Load delivery report
    let delivery_report = fs::read_to_string(session_dir.join("delivery_report.md")).ok();

    Ok(SessionArtifacts {
        session_id,
        idea,
        requirements,
        features,
        design,
        plan,
        code_files,
        delivery_report,
    })
}

// ============================================================================
// File Operations
// ============================================================================

#[tauri::command]
pub async fn read_file_content(
    session_id: String,
    file_path: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    println!("[GUI] Reading file: {}", file_path);

    let project_root = cowork_core::storage::get_project_root()
        .map_err(|e| format!("Failed to get project root: {}", e))?;

    let full_path = project_root.join(&file_path);

    fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub async fn save_file_content(
    session_id: String,
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
        Some("xml") | Some("html") => Some("xml".to_string()),
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