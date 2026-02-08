// GUI-specific commands for enhanced functionality
use super::gui_types::*;
use super::gui_types::FileReadResult;
use crate::AppState;
use crate::preview_server::PreviewServerManager;
use crate::project_runner::ProjectRunner;
use cowork_core::persistence::IterationStore;
use tauri::{State, Window};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
// Open Folder Command
// ============================================================================

#[tauri::command]
pub async fn open_in_file_manager(path: String, _window: Window) -> Result<(), String> {
    use std::env;
    
    // Resolve the path
    let resolved_path = if path.starts_with("workspace_") {
        // It's a workspace path
        let iteration_id = path.strip_prefix("workspace_").unwrap_or(&path);
        let iteration_store = IterationStore::new();
        iteration_store.workspace_path(iteration_id)
            .map_err(|e| format!("Failed to get workspace path: {}", e))?
    } else if path.contains("iter-") {
        // It's an iteration artifacts path
        let iteration_store = IterationStore::new();
        iteration_store.iteration_path(&path)
            .map_err(|e| format!("Failed to get iteration path: {}", e))?
    } else {
        // It's a direct path
        PathBuf::from(path)
    };
    
    if !resolved_path.exists() {
        return Err(format!("Path does not exist: {}", resolved_path.display()));
    }
    
    // Open in file manager based on OS
    if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(&resolved_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(&resolved_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    } else {
        // Linux
        Command::new("xdg-open")
            .arg(&resolved_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

// ============================================================================
// Get Iteration Artifacts (New V2 API)
// ============================================================================

#[tauri::command]
pub async fn get_iteration_artifacts(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<IterationArtifacts, String> {
    println!("[GUI] Getting artifacts for iteration: {}", iteration_id);

    let iteration_store = IterationStore::new();
    let _iteration = iteration_store.load(&iteration_id)
        .map_err(|e| format!("Failed to load iteration: {}", e))?;

    // Get iteration artifacts directory
    let iteration_dir = iteration_store.iteration_path(&iteration_id)
        .map_err(|e| format!("Failed to get iteration dir: {}", e))?;
    let artifacts_dir = iteration_dir.join("artifacts");

    println!("[GUI] Current dir: {:?}", std::env::current_dir());
    println!("[GUI] Iteration dir: {:?}", iteration_dir);
    println!("[GUI] Artifacts dir: {:?}", artifacts_dir);
    println!("[GUI] Artifacts dir exists: {}", artifacts_dir.exists());

    // List files in artifacts directory for debugging
    if artifacts_dir.exists() {
        if let Ok(entries) = fs::read_dir(&artifacts_dir) {
            println!("[GUI] Files in artifacts dir:");
            for entry in entries.flatten() {
                println!("[GUI]   - {:?}", entry.file_name());
            }
        }
    }

    // Load artifacts from .cowork-v2 structure
    let idea = fs::read_to_string(artifacts_dir.join("idea.md")).ok();
    let prd = fs::read_to_string(artifacts_dir.join("prd.md")).ok();
    let design_raw = fs::read_to_string(artifacts_dir.join("design.md")).ok();
    let plan_raw = fs::read_to_string(artifacts_dir.join("plan.md")).ok();
    let delivery_report = fs::read_to_string(artifacts_dir.join("delivery_report.md")).ok();
    let check_report = fs::read_to_string(artifacts_dir.join("check_report.md")).ok();

    println!("[GUI] Idea loaded: {}", idea.is_some());
    println!("[GUI] PRD loaded: {}", prd.is_some());

    // Load workspace code files if available
    let workspace = iteration_store.workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;
    let code_files = if workspace.exists() {
        collect_files(&workspace)
    } else {
        vec![]
    };

    Ok(IterationArtifacts {
        iteration_id: iteration_id.clone(),
        idea,
        requirements: prd,
        design: design_raw,
        plan: plan_raw,
        code_files,
        check_report,
        delivery_report,
    })
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

    install_dependencies_if_needed(&workspace).await?;

    PREVIEW_SERVER_MANAGER.start(iteration_id, workspace).await
}

#[tauri::command]
pub async fn check_preview_status(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<Option<PreviewInfo>, String> {
    println!("[GUI] Checking preview status for iteration: {}", iteration_id);
    
    if PREVIEW_SERVER_MANAGER.is_running(&iteration_id) {
        Ok(PREVIEW_SERVER_MANAGER.get_info(&iteration_id))
    } else {
        Ok(None)
    }
}

/// Install dependencies if package.json exists and node_modules is missing
async fn install_dependencies_if_needed(workspace: &std::path::Path) -> Result<(), String> {
    let package_json = workspace.join("package.json");
    let node_modules = workspace.join("node_modules");
    
    if package_json.exists() && !node_modules.exists() {
        println!("[GUI] package.json found but node_modules missing, installing dependencies...");
        
        // Try bun first, then npm
        let use_bun = which::which("bun").is_ok();
        let use_npm = which::which("npm").is_ok();
        
        let install_cmd = if use_bun {
            "bun install"
        } else if use_npm {
            "npm install"
        } else {
            return Err("Neither bun nor npm found. Cannot install dependencies.".to_string());
        };
        
        println!("[GUI] Running: {} in {}", install_cmd, workspace.display());
        
        let output = std::process::Command::new(if use_bun { "bun" } else { "npm" })
            .arg("install")
            .current_dir(workspace)
            .output();
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("[GUI] Dependencies installed successfully");
                    // Print summary
                    if let Ok(stdout) = String::from_utf8(result.stdout) {
                        let lines: Vec<&str> = stdout.lines().collect();
                        if let Some(last_line) = lines.last() {
                            println!("[GUI] {}", last_line);
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    return Err(format!("Failed to install dependencies: {}", stderr));
                }
            }
            Err(e) => {
                return Err(format!("Failed to run install command: {}", e));
            }
        }
    }
    
    Ok(())
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

    // Install dependencies if needed
    install_dependencies_if_needed(&workspace).await?;

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
pub async fn check_project_status(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    Ok(PROJECT_RUNNER.is_running(&iteration_id))
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
    _category: String,
    stage: Option<String>,
    limit: i64,
    iteration_id: Option<String>,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Querying memory index: query_type={}, stage={:?}, limit={}, iteration_id={:?}", 
             query_type, stage, limit, iteration_id);

    let store = cowork_core::persistence::MemoryStore::new();
    
    // Convert parameters to new MemoryQuery format
    let scope = cowork_core::domain::MemoryScope::Smart; // Default to smart query
    let query_type_enum = match query_type.as_str() {
        "decisions" => cowork_core::domain::MemoryQueryType::Decisions,
        "patterns" => cowork_core::domain::MemoryQueryType::Patterns,
        "insights" => cowork_core::domain::MemoryQueryType::Insights,
        _ => cowork_core::domain::MemoryQueryType::All,
    };
    
    let keywords = if let Some(s) = stage {
        vec![s]
    } else {
        vec![]
    };
    
    let query = cowork_core::domain::MemoryQuery {
        scope,
        query_type: query_type_enum,
        keywords,
        limit: Some(limit as usize),
    };
    
    let result = store.query(&query, iteration_id.as_deref())
        .map_err(|e| format!("Failed to query memory: {}", e))?;
    
    Ok(serde_json::json!({
        "decisions": result.decisions,
        "patterns": result.patterns,
        "insights": result.insights,
        "total_decisions": result.decisions.len(),
        "total_patterns": result.patterns.len(),
        "total_insights": result.insights.len()
    }))
}

#[tauri::command]
pub async fn load_memory_detail(
    memory_id: String,
    _file: String,
    iteration_id: Option<String>,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Loading memory detail: memory_id={}, iteration_id={:?}", memory_id, iteration_id);

    let store = cowork_core::persistence::MemoryStore::new();
    
    // Try to find the memory item in project memory first
    let project_memory = store.load_project_memory()
        .map_err(|e| format!("Failed to load project memory: {}", e))?;
    
    // Search in decisions
    for decision in &project_memory.decisions {
        if decision.id == memory_id {
            return Ok(serde_json::json!({
                "memory_id": memory_id,
                "content": format!("**Context:** {}\n\n**Decision:** {}\n\n**Consequences:**\n{}", 
                    decision.context, 
                    decision.decision,
                    decision.consequences.join("\n")),
                "type": "decision"
            }));
        }
    }
    
    // Search in patterns
    for pattern in &project_memory.patterns {
        if pattern.id == memory_id {
            return Ok(serde_json::json!({
                "memory_id": memory_id,
                "content": format!("**Description:** {}\n\n**Usage:**\n{}\n\n**Tags:** {}\n\n**Code Example:**\n{}", 
                    pattern.description,
                    pattern.usage.join("\n"),
                    pattern.tags.join(", "),
                    pattern.code_example.as_deref().unwrap_or("None")),
                "type": "pattern"
            }));
        }
    }
    
    // Try iteration memory - use memory_id format like "insight-123" to identify items
    if let Some(iter_id) = iteration_id {
        if let Ok(iter_memory) = store.load_iteration_memory(&iter_id) {
            // Search in insights by timestamp (memory_id may contain timestamp)
            if memory_id.starts_with("insight-") {
                if let Ok(ts) = memory_id.replace("insight-", "").parse::<i64>() {
                    for insight in &iter_memory.insights {
                        if insight.created_at.timestamp() == ts {
                            return Ok(serde_json::json!({
                                "memory_id": memory_id,
                                "content": format!("**Stage:** {}\n\n**Content:** {}", 
                                    insight.stage,
                                    insight.content),
                                "type": "insight"
                            }));
                        }
                    }
                }
            }
            
            // Search in issues by timestamp
            if memory_id.starts_with("issue-") {
                if let Ok(ts) = memory_id.replace("issue-", "").parse::<i64>() {
                    for issue in &iter_memory.issues {
                        if issue.created_at.timestamp() == ts {
                            return Ok(serde_json::json!({
                                "memory_id": memory_id,
                                "content": format!("**Stage:** {}\n\n**Issue:** {}\n\n**Resolved:** {}", 
                                    issue.stage,
                                    issue.content,
                                    issue.resolved),
                                "type": "issue"
                            }));
                        }
                    }
                }
            }
            
            // Search in learnings by timestamp
            if memory_id.starts_with("learning-") {
                if let Ok(ts) = memory_id.replace("learning-", "").parse::<i64>() {
                    for learning in &iter_memory.learnings {
                        if learning.created_at.timestamp() == ts {
                            return Ok(serde_json::json!({
                                "memory_id": memory_id,
                                "content": format!("**Learning:** {}", learning.content),
                                "type": "learning"
                            }));
                        }
                    }
                }
            }
        }
    }
    
    Err(format!("Memory item not found: {}", memory_id))
}

#[tauri::command]
pub async fn save_session_memory(
    memory_type: String,
    title: String,
    summary: String,
    content: String,
    stage: String,
    _category: Option<String>,
    _impact: Option<String>,
    _tags: Option<Vec<String>>,
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Saving session memory: memory_type={}, title={}, iteration_id={}", memory_type, title, iteration_id);
    
    let store = cowork_core::persistence::MemoryStore::new();
    let mut memory = store.load_iteration_memory(&iteration_id)
        .map_err(|e| format!("Failed to load iteration memory: {}", e))?;
    
    match memory_type.as_str() {
        "decision" => {
            let decision = cowork_core::domain::Decision::new(
                &title,
                &summary,
                &content,
                &iteration_id
            );
            store.add_decision(decision)
                .map_err(|e| format!("Failed to add decision: {}", e))?;
        }
        "pattern" => {
            let mut pattern = cowork_core::domain::Pattern::new(
                &title,
                &content,
                &iteration_id
            );
            pattern.tags = vec![stage.clone()];
            store.add_pattern(pattern)
                .map_err(|e| format!("Failed to add pattern: {}", e))?;
        }
        "insight" => {
            memory.add_insight(&stage, &format!("{}: {}", title, content));
        }
        "issue" => {
            memory.add_issue(&stage, &format!("{}: {}", title, content));
        }
        "learning" => {
            memory.add_learning(&format!("{}: {}", title, content));
        }
        _ => {
            return Err(format!("Unknown memory type: {}", memory_type));
        }
    }
    
    store.save_iteration_memory(&memory)
        .map_err(|e| format!("Failed to save iteration memory: {}", e))?;
    
    Ok(serde_json::json!({
        "message": "Memory saved successfully",
        "iteration_id": iteration_id
    }))
}

#[tauri::command]
pub async fn promote_to_project_memory(
    memory_id: String,
    _reason: String,
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Promoting to project memory: memory_id={}, iteration_id={}", memory_id, iteration_id);

    let store = cowork_core::persistence::MemoryStore::new();
    let iter_memory = store.load_iteration_memory(&iteration_id)
        .map_err(|e| format!("Failed to load iteration memory: {}", e))?;
    
    // Find and promote insight by timestamp
    if memory_id.starts_with("insight-") {
        if let Ok(ts) = memory_id.replace("insight-", "").parse::<i64>() {
            for insight in &iter_memory.insights {
                if insight.created_at.timestamp() == ts {
                    let decision = cowork_core::domain::Decision::new(
                        &format!("Insight from {}", insight.stage),
                        &format!("Discovered during {} stage", insight.stage),
                        &insight.content,
                        &iteration_id
                    );
                    store.add_decision(decision)
                        .map_err(|e| format!("Failed to add decision: {}", e))?;
                    return Ok(serde_json::json!({
                        "message": "Promoted to project decision successfully",
                        "memory_id": memory_id
                    }));
                }
            }
        }
    }
    
    // Find and promote learning by timestamp
    if memory_id.starts_with("learning-") {
        if let Ok(ts) = memory_id.replace("learning-", "").parse::<i64>() {
            for learning in &iter_memory.learnings {
                if learning.created_at.timestamp() == ts {
                    let pattern = cowork_core::domain::Pattern::new(
                        "Learning",
                        &learning.content,
                        &iteration_id
                    );
                    store.add_pattern(pattern)
                        .map_err(|e| format!("Failed to add pattern: {}", e))?;
                    return Ok(serde_json::json!({
                        "message": "Promoted to project pattern successfully",
                        "memory_id": memory_id
                    }));
                }
            }
        }
    }
    
    Err(format!("Memory item not found for promotion: {}", memory_id))
}

#[tauri::command]
pub async fn get_memory_context(
    iteration_id: Option<String>,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Getting memory context: iteration_id={:?}", iteration_id);

    let store = cowork_core::persistence::MemoryStore::new();
    
    // Get project memory
    let project_memory = store.load_project_memory()
        .map_err(|e| format!("Failed to load project memory: {}", e))?;
    
    // Get iteration memory if available
    let iteration_memory = if let Some(iter_id) = iteration_id {
        Some(store.load_iteration_memory(&iter_id)
            .map_err(|e| format!("Failed to load iteration memory: {}", e))?)
    } else {
        None
    };
    
    Ok(serde_json::json!({
        "project_memory": {
            "total_decisions": project_memory.decisions.len(),
            "total_patterns": project_memory.patterns.len(),
            "key_decisions": project_memory.decisions.iter()
                .take(5)
                .map(|d| serde_json::json!({"id": d.id, "title": d.title}))
                .collect::<Vec<_>>()
        },
        "iteration_memory": iteration_memory.map(|mem| serde_json::json!({
            "iteration_id": mem.iteration_id,
            "total_insights": mem.insights.len(),
            "total_issues": mem.issues.len(),
            "total_learnings": mem.learnings.len()
        }))
    }))
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
        "dummy_iteration"
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

    let project_root = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

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

    let project_root = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

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

    let project_root = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

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
