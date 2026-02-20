// GUI-specific commands for enhanced functionality
use super::gui_types::FileReadResult;
use super::gui_types::*;
use super::static_server;
use super::iteration_commands::gui_execute_iteration;
use crate::AppState;
use crate::project_runner::ProjectRunner;
use cowork_core::llm::config::{LlmConfig, load_config};
use cowork_core::persistence::IterationStore;
use cowork_core::{ProjectRuntimeConfig, RuntimeAnalyzer, RuntimeType};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Emitter, State, Window};

// Global instances
lazy_static::lazy_static! {
    static ref PROJECT_RUNNER: ProjectRunner = ProjectRunner::new();
    static ref RUNTIME_ANALYZER: std::sync::Mutex<Option<RuntimeAnalyzer>> = std::sync::Mutex::new(None);
}

// ============================================================================
// Initialization
// ============================================================================

pub fn init_app_handle(handle: tauri::AppHandle) {
    PROJECT_RUNNER.set_app_handle(handle);
}

// ============================================================================
// System Locale Command
// ============================================================================

#[tauri::command]
pub fn get_system_locale() -> String {
    // 获取系统语言/区域设置
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // 使用 PowerShell 获取 Windows 系统语言
        let output = Command::new("powershell")
            .args(["-Command", "(Get-Culture).Name"])
            .output();
        
        if let Ok(output) = output {
            let locale = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !locale.is_empty() {
                // 存储到 cowork-core 全局配置
                cowork_core::set_system_locale(locale.clone());
                return locale;
            }
        }
    }
    
    // 默认返回英语
    let default_locale = "en-US".to_string();
    cowork_core::set_system_locale(default_locale.clone());
    default_locale
}

// ============================================================================
// Open Folder Command
// ============================================================================

#[tauri::command]
pub async fn open_in_file_manager(path: String, _window: Window) -> Result<(), String> {
    // Resolve the path
    let resolved_path = if path.starts_with("workspace_") {
        // It's a workspace path
        let iteration_id = path.strip_prefix("workspace_").unwrap_or(&path);
        let iteration_store = IterationStore::new();
        iteration_store
            .workspace_path(iteration_id)
            .map_err(|e| format!("Failed to get workspace path: {}", e))?
    } else if path.contains("iter-") {
        // It's an iteration artifacts path
        let iteration_store = IterationStore::new();
        iteration_store
            .iteration_path(&path)
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
    let _iteration = iteration_store
        .load(&iteration_id)
        .map_err(|e| format!("Failed to load iteration: {}", e))?;

    // Get iteration artifacts directory
    let iteration_dir = iteration_store
        .iteration_path(&iteration_id)
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
    let workspace = iteration_store
        .workspace_path(&iteration_id)
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
    println!(
        "[GUI] Reading file for iteration {}: {}",
        iteration_id, file_path
    );

    let iteration_store = IterationStore::new();
    let workspace = iteration_store
        .workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;

    let full_path = workspace.join(&file_path);

    // Get file metadata
    let metadata =
        fs::metadata(&full_path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let file_size = metadata.len() as usize;
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB limit for full read

    // If file is too large or offset/limit specified, read in chunks
    if file_size > MAX_FILE_SIZE || offset.is_some() || limit.is_some() {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(1024 * 1024); // Default 1MB chunks

        let mut file =
            fs::File::open(&full_path).map_err(|e| format!("Failed to open file: {}", e))?;

        use std::io::{Read, Seek};

        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|e| format!("Failed to seek in file: {}", e))?;

        let mut buffer = vec![0; limit.min(file_size - offset)];
        let bytes_read = file
            .read(&mut buffer)
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
        let content =
            fs::read_to_string(&full_path).map_err(|e| format!("Failed to read file: {}", e))?;

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
    println!(
        "[GUI] Saving file for iteration {}: {}",
        iteration_id, file_path
    );

    let iteration_store = IterationStore::new();
    let workspace = iteration_store
        .workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;

    let full_path = workspace.join(&file_path);

    // Create parent directories if needed
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    fs::write(&full_path, content).map_err(|e| format!("Failed to write file: {}", e))?;

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
    let workspace = iteration_store
        .workspace_path(&iteration_id)
        .map_err(|e| format!("Failed to get workspace: {}", e))?;

    if !workspace.exists() {
        return Ok(FileTreeNode {
            name: workspace
                .file_name()
                .unwrap_or(workspace.as_os_str())
                .to_string_lossy()
                .to_string(),
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

    // Use project root (after delivery) or workspace (before delivery)
    let code_dir = get_code_directory(&iteration_id)?;

    // Install dependencies if needed
    install_dependencies_if_needed(&code_dir).await?;

    // Get runtime config to determine project type
    let config_result = try_analyze_with_runtime(&code_dir);

    // Check if this is a VanillaHtml project - use built-in static server
    if let Ok(ref config) = config_result {
        if config.runtime_type == RuntimeType::VanillaHtml {
            println!("[GUI] Detected VanillaHtml, using built-in static server");
            
            let server_info = static_server::start_static_server(
                iteration_id.clone(),
                code_dir.clone(),
                8000, // preferred port
                None, // app_handle not needed here
            )?;

            return Ok(PreviewInfo {
                url: server_info.url,
                port: server_info.port,
                status: PreviewStatus::Running,
                project_type: ProjectType::Html,
            });
        }
    }

    // For other project types, use external commands via ProjectRunner
    let (command, port, url) = detect_start_command_with_info_from_config(&config_result)?;

    println!("[GUI] Preview using command: {}, port: {}", command, port);

    // Start the development server using ProjectRunner
    let _pid = PROJECT_RUNNER
        .start(
            iteration_id.clone(),
            command,
            code_dir.to_string_lossy().to_string(),
            Some(url.clone()),
            Some(port),
        )
        .await?;

    Ok(PreviewInfo {
        url,
        port,
        status: PreviewStatus::Running,
        project_type: ProjectType::Unknown,
    })
}

/// Detect start command with additional info (port, url) from already-analyzed config
fn detect_start_command_with_info_from_config(
    config_result: &Result<ProjectRuntimeConfig, String>,
) -> Result<(String, u16, String), String> {
    if let Ok(config) = config_result {
        // VanillaHtml should be handled separately with built-in server
        if config.runtime_type == RuntimeType::VanillaHtml {
            return Err("VanillaHtml should use built-in static server".to_string());
        }

        // Determine port and URL based on project type
        let (port, url) = if let Some(frontend) = &config.frontend {
            (
                frontend.dev_port,
                format!("http://{}:{}", frontend.dev_host, frontend.dev_port),
            )
        } else if let Some(backend) = &config.backend {
            (
                backend.port,
                format!("http://{}:{}", backend.host, backend.port),
            )
        } else if let Some(fullstack) = &config.fullstack {
            (
                fullstack.frontend_port,
                format!("http://localhost:{}", fullstack.frontend_port),
            )
        } else {
            (3000, "http://localhost:3000".to_string())
        };

        // Generate start command
        if let Some(cmd) = generate_start_command_from_config(&config) {
            println!(
                "[GUI] LLM detected runtime type: {:?}, command: {}, port: {}",
                config.runtime_type, cmd, port
            );
            return Ok((cmd, port, url));
        }
    }

    // LLM 分析失败，返回错误
    Err("无法通过大模型分析获取运行命令，请确保在设置中配置了有效的 LLM".to_string())
}

#[tauri::command]
pub async fn check_preview_status(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<Option<PreviewInfo>, String> {
    println!(
        "[GUI] Checking preview status for iteration: {}",
        iteration_id
    );

    // First check built-in static server
    if static_server::is_server_running(&iteration_id) {
        if let Some(server_info) = static_server::get_server_info(&iteration_id) {
            return Ok(Some(PreviewInfo {
                url: server_info.url,
                port: server_info.port,
                status: PreviewStatus::Running,
                project_type: ProjectType::Html,
            }));
        }
    }

    // Then check external process via ProjectRunner
    if PROJECT_RUNNER.is_running(&iteration_id) {
        if let Some(info) = PROJECT_RUNNER.get_info(&iteration_id) {
            return Ok(Some(info));
        }
    }
    Ok(None)
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
    
    // Stop built-in static server if running
    static_server::stop_static_server(&iteration_id)?;
    
    // Stop external process if running
    PROJECT_RUNNER.stop(iteration_id).await
}

#[tauri::command]
pub async fn get_project_runtime_info(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<ProjectRuntimeInfo, String> {
    println!(
        "[GUI] Getting project runtime info for iteration: {}",
        iteration_id
    );

    // Use project root (after delivery) or workspace (before delivery)
    let code_dir = get_code_directory(&iteration_id)?;

    // Try to get runtime config from LLM analysis
    let config_result = try_analyze_with_runtime(&code_dir);

    let (has_frontend, has_backend, preview_url, frontend_port, backend_port, start_command) =
        if let Ok(config) = config_result {
            // Special handling for VanillaHtml - use built-in static server
            if config.runtime_type == RuntimeType::VanillaHtml {
                // Check if static server is already running
                let (url, port) = if let Some(server_info) = static_server::get_server_info(&iteration_id) {
                    (Some(server_info.url), Some(server_info.port))
                } else {
                    // Return default port info (will be updated when server starts)
                    (Some("http://localhost:8000".to_string()), Some(8000))
                };
                
                return Ok(ProjectRuntimeInfo {
                    has_frontend: true,
                    has_backend: false,
                    preview_url: url,
                    frontend_port: port,
                    backend_port: None,
                    start_command: Some("(Built-in static server)".to_string()),
                });
            }

            // Special handling for Fullstack projects
            if is_fullstack_type(&config.runtime_type) {
                let fullstack = config.fullstack.as_ref();
                let fullstack_instance = static_server::get_fullstack_process(&iteration_id);
                
                let (frontend_url, f_port, b_port) = if let Some(ref instance) = fullstack_instance {
                    (Some(instance.frontend_url.clone()), Some(instance.frontend_port), Some(instance.backend_port))
                } else if let Some(ref fs) = fullstack {
                    let url = format!("http://localhost:{}", fs.frontend_port);
                    (Some(url), Some(fs.frontend_port), Some(fs.backend_port))
                } else {
                    (None, None, None)
                };
                
                let cmd = if let Some(ref fs) = fullstack {
                    Some(format!("Frontend: {} | Backend: {}", fs.frontend_dev_command, fs.backend_dev_command))
                } else {
                    None
                };
                
                return Ok(ProjectRuntimeInfo {
                    has_frontend: true,
                    has_backend: true,
                    preview_url: frontend_url,
                    frontend_port: f_port,
                    backend_port: b_port,
                    start_command: cmd,
                });
            }

            let has_frontend = config.frontend.is_some();
            let has_backend = config.backend.is_some() || config.fullstack.is_some();

            // Get initial port from config
            let (mut preview_url, mut frontend_port) = if let Some(ref frontend) = config.frontend {
                let url = format!("http://{}:{}", frontend.dev_host, frontend.dev_port);
                (Some(url), Some(frontend.dev_port))
            } else if let Some(ref fullstack) = config.fullstack {
                let url = format!("http://localhost:{}", fullstack.frontend_port);
                (Some(url), Some(fullstack.frontend_port))
            } else {
                (None, None)
            };

            let backend_port = if let Some(ref backend) = config.backend {
                Some(backend.port)
            } else if let Some(ref fullstack) = config.fullstack {
                Some(fullstack.backend_port)
            } else {
                None
            };

            let start_command = generate_start_command_from_config(&config);

            // NEW: Try to extract actual port from README if project is running and has LLM config
            // This fixes the issue where the default port (5173) doesn't match the actual running port
            if has_frontend && frontend_port.is_some() {
                if let Ok(model_config) = load_config() {
                    // Only analyze README if project is running to avoid unnecessary LLM calls
                    let is_project_running = PROJECT_RUNNER.is_running(&iteration_id) 
                        || static_server::is_server_running(&iteration_id);
                    
                    if is_project_running {
                        println!("[GUI] Project is running, trying to extract actual port from README...");
                        
                        match cowork_core::runtime_analyzer::analyze_runtime_from_readme(
                            &code_dir,
                            &config,
                            &model_config.llm,
                        ).await {
                            Ok(Some((actual_port, actual_url))) => {
                                println!("[GUI] Extracted actual port {} from README, updating runtime info", actual_port);
                                preview_url = Some(actual_url);
                                frontend_port = Some(actual_port);
                            }
                            Ok(None) => {
                                println!("[GUI] No port information found in README, using default port from config");
                            }
                            Err(e) => {
                                println!("[GUI] Failed to analyze README for port: {}, using default port", e);
                            }
                        }
                    }
                }
            }

            (
                has_frontend,
                has_backend,
                preview_url,
                frontend_port,
                backend_port,
                start_command,
            )
        } else {
            // Fallback: detect based on file existence
            let has_frontend = code_dir.join("index.html").exists()
                || code_dir.join("src").exists()
                || code_dir.join("package.json").exists();
            let has_backend = code_dir.join("Cargo.toml").exists()
                || code_dir.join("main.py").exists()
                || code_dir.join("server.js").exists();

            (has_frontend, has_backend, None, None, None, None)
        };

    Ok(ProjectRuntimeInfo {
        has_frontend,
        has_backend,
        preview_url,
        frontend_port,
        backend_port,
        start_command,
    })
}

#[tauri::command]
pub async fn start_iteration_project(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<RunInfo, String> {
    println!("[GUI] Starting project for iteration: {}", iteration_id);

    // Use project root (after delivery) or workspace (before delivery)
    let code_dir = get_code_directory(&iteration_id)?;

    // Install dependencies if needed
    install_dependencies_if_needed(&code_dir).await?;

    // Analyze project type
    let config_result = try_analyze_with_runtime(&code_dir);
    
    if let Ok(ref config) = config_result {
        // Handle VanillaHtml - use built-in static server
        if config.runtime_type == RuntimeType::VanillaHtml {
            println!("[GUI] Detected VanillaHtml, using built-in static server");
            
            let server_info = static_server::start_static_server(
                iteration_id.clone(),
                code_dir.clone(),
                8000,
                None,
            )?;

            return Ok(RunInfo {
                status: RunStatus::Running,
                process_id: None,
                command: Some(format!("Built-in static server on port {}", server_info.port)),
                ..Default::default()
            });
        }
        
        // Handle Fullstack projects - start both frontend and backend
        if is_fullstack_type(&config.runtime_type) {
            println!("[GUI] Detected Fullstack project: {:?}", config.runtime_type);
            return start_fullstack_project(iteration_id, code_dir, config).await;
        }
    }

    // Single process projects
    let command = detect_start_command(&code_dir)?;

    println!("[GUI] Detected start command: {}", command);

    // Try to extract actual port from README before starting
    let (url, port) = if let Ok(ref config) = config_result {
        if let Ok(model_config) = load_config() {
            match cowork_core::runtime_analyzer::analyze_runtime_from_readme(
                &code_dir,
                config,
                &model_config.llm,
            ).await {
                Ok(Some((actual_port, actual_url))) => {
                    println!("[GUI] Extracted actual port {} from README for project start", actual_port);
                    (Some(actual_url), Some(actual_port))
                }
                _ => {
                    // Fallback to config default
                    let default_port = config.frontend.as_ref()
                        .map(|f| f.dev_port)
                        .or_else(|| config.backend.as_ref().map(|b| b.port))
                        .unwrap_or(3000);
                    let default_url = format!("http://localhost:{}", default_port);
                    (Some(default_url), Some(default_port))
                }
            }
        } else {
            let default_port = config.frontend.as_ref()
                .map(|f| f.dev_port)
                .or_else(|| config.backend.as_ref().map(|b| b.port))
                .unwrap_or(3000);
            let default_url = format!("http://localhost:{}", default_port);
            (Some(default_url), Some(default_port))
        }
    } else {
        // Fallback to default port 3000
        (Some("http://localhost:3000".to_string()), Some(3000))
    };

    let command_clone = command.clone();
    let pid = PROJECT_RUNNER
        .start(
            iteration_id.clone(),
            command,
            code_dir.to_string_lossy().to_string(),
            url.clone(),
            port,
        )
        .await?;

    Ok(RunInfo {
        status: RunStatus::Running,
        process_id: Some(pid),
        command: Some(command_clone),
        ..Default::default()
    })
}

/// Check if runtime type is a fullstack type
fn is_fullstack_type(runtime_type: &RuntimeType) -> bool {
    matches!(runtime_type, 
        RuntimeType::FullstackReactRust | 
        RuntimeType::FullstackReactNode |
        RuntimeType::FullstackVanillaRust |
        RuntimeType::FullstackVanillaNode
    )
}

/// Start a fullstack project (frontend + backend)
async fn start_fullstack_project(
    iteration_id: String,
    code_dir: PathBuf,
    config: &ProjectRuntimeConfig,
) -> Result<RunInfo, String> {
    let fullstack_config = config.fullstack.as_ref()
        .ok_or("Fullstack config not found in project runtime config")?;
    
    let (frontend_key, backend_key) = static_server::get_fullstack_process_keys(&iteration_id);
    let backend_port = fullstack_config.backend_port;
    
    // 1. Start backend first (frontend may depend on backend API)
    let backend_command = &fullstack_config.backend_dev_command;
    println!("[GUI] Starting fullstack backend: {}", backend_command);
    
    let backend_url = format!("http://localhost:{}", backend_port);
    let backend_pid = match PROJECT_RUNNER
        .start(
            backend_key.clone(),
            backend_command.clone(),
            code_dir.to_string_lossy().to_string(),
            Some(backend_url.clone()),
            Some(backend_port),
        )
        .await
    {
        Ok(pid) => pid,
        Err(e) => {
            return Err(format!("Failed to start backend: {}", e));
        }
    };
    
    // 2. Wait for backend to be ready (health check with timeout)
    println!("[GUI] Waiting for backend to be ready on port {}...", backend_port);
    let mut backend_ready = false;
    
    // Try health check for up to 30 seconds
    for attempt in 1..=30 {
        // Check if process is still running
        if !PROJECT_RUNNER.is_running(&backend_key) {
            // Process already stopped, just return error
            return Err("Backend process exited unexpectedly during startup. Check the logs for errors.".to_string());
        }
        
        // Try to connect to backend
        match attohttpc::get(&backend_url)
            .timeout(std::time::Duration::from_secs(1))
            .send()
        {
            Ok(_) => {
                println!("[GUI] Backend is ready after {} attempts", attempt);
                backend_ready = true;
                break;
            }
            Err(_) => {
                // Backend not ready yet, wait and retry
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
    
    if !backend_ready {
        println!("[GUI] Warning: Backend health check timed out, proceeding anyway");
    }
    
    // 3. Start frontend
    let frontend_command = &fullstack_config.frontend_dev_command;
    println!("[GUI] Starting fullstack frontend: {}", frontend_command);
    
    let frontend_url = format!("http://localhost:{}", fullstack_config.frontend_port);
    let frontend_pid = match PROJECT_RUNNER
        .start(
            frontend_key.clone(),
            frontend_command.clone(),
            code_dir.to_string_lossy().to_string(),
            Some(frontend_url.clone()),
            Some(fullstack_config.frontend_port),
        )
        .await
    {
        Ok(pid) => pid,
        Err(e) => {
            // Frontend failed, cleanup backend
            println!("[GUI] Frontend failed, cleaning up backend: {}", e);
            let _ = PROJECT_RUNNER.stop(backend_key).await;
            return Err(format!("Failed to start frontend: {}", e));
        }
    };
    
    // 4. Register fullstack process instance
    let instance = static_server::FullstackProcessInstance {
        iteration_id: iteration_id.clone(),
        frontend_pid: Some(frontend_pid),
        backend_pid: Some(backend_pid),
        frontend_port: fullstack_config.frontend_port,
        backend_port: fullstack_config.backend_port,
        frontend_url: format!("http://localhost:{}", fullstack_config.frontend_port),
        backend_url: format!("http://localhost:{}", fullstack_config.backend_port),
    };
    
    static_server::register_fullstack_process(instance.clone());
    
    println!("[GUI] Fullstack project started: frontend={}, backend={}", 
        instance.frontend_url, instance.backend_url);
    
    Ok(RunInfo {
        status: RunStatus::Running,
        process_id: None,
        command: Some(format!("Frontend: {} | Backend: {}", 
            fullstack_config.frontend_dev_command, 
            fullstack_config.backend_dev_command)),
        frontend_pid: Some(frontend_pid),
        backend_pid: Some(backend_pid),
        frontend_url: Some(instance.frontend_url.clone()),
        backend_url: Some(instance.backend_url.clone()),
        is_fullstack: true,
    })
}

#[tauri::command]
pub async fn stop_iteration_project(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] Stopping project for iteration: {}", iteration_id);
    
    // Stop built-in static server if running
    static_server::stop_static_server(&iteration_id)?;
    
    // Check if this is a fullstack process
    if let Some(instance) = static_server::remove_fullstack_process(&iteration_id) {
        println!("[GUI] Stopping fullstack project: frontend_pid={:?}, backend_pid={:?}", 
            instance.frontend_pid, instance.backend_pid);
        
        let (frontend_key, backend_key) = static_server::get_fullstack_process_keys(&iteration_id);
        
        // Stop frontend
        if let Err(e) = PROJECT_RUNNER.stop(frontend_key).await {
            eprintln!("[GUI] Warning: Failed to stop frontend: {}", e);
        }
        
        // Stop backend
        if let Err(e) = PROJECT_RUNNER.stop(backend_key).await {
            eprintln!("[GUI] Warning: Failed to stop backend: {}", e);
        }
        
        println!("[GUI] Fullstack project stopped");
        return Ok(());
    }
    
    // Stop single external process
    PROJECT_RUNNER.stop(iteration_id).await
}

#[tauri::command]
pub async fn check_project_status(
    iteration_id: String,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    // Check built-in static server first
    if static_server::is_server_running(&iteration_id) {
        return Ok(true);
    }
    
    // Check fullstack process
    if static_server::is_fullstack_running(&iteration_id) {
        let (frontend_key, backend_key) = static_server::get_fullstack_process_keys(&iteration_id);
        let frontend_running = PROJECT_RUNNER.is_running(&frontend_key);
        let backend_running = PROJECT_RUNNER.is_running(&backend_key);
        
        // Return true if either frontend or backend is running
        if frontend_running || backend_running {
            return Ok(true);
        } else {
            // Both stopped, clean up the instance
            static_server::remove_fullstack_process(&iteration_id);
            return Ok(false);
        }
    }
    
    // Check single external process
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
                let name = path.file_name().unwrap().to_string_lossy().to_string();

                let relative_path = path
                    .strip_prefix(dir)
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
                    modified_at: meta
                        .modified()
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
    let name = dir.file_name().unwrap().to_string_lossy().to_string();

    let path = dir
        .strip_prefix(root)
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
            b_is_dir
                .cmp(&a_is_dir)
                .then_with(|| a.file_name().cmp(&b.file_name()))
        });

        Some(
            entries
                .into_iter()
                .filter_map(|entry| {
                    let path = entry.path();
                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                    // Skip hidden files
                    if name.starts_with('.') {
                        return None;
                    }
                    build_file_tree(&path, root, depth + 1).ok()
                })
                .collect(),
        )
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

/// Determine the correct code directory for preview/run.
/// Priority: workspace (before delivery) > project root (after delivery)
fn get_code_directory(iteration_id: &str) -> Result<PathBuf, String> {
    let iteration_store = IterationStore::new();
    let workspace = iteration_store
        .workspace_path(iteration_id)
        .map_err(|e| format!("Failed to get workspace path: {}", e))?;

    // Get project root (current working directory)
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get project root: {}", e))?;

    // Check if a directory has project files
    let has_frontend_files = |dir: &Path| -> bool {
        dir.join("index.html").exists() || dir.join("src").exists() || dir.join("public").exists()
    };

    let has_backend_files = |dir: &Path| -> bool {
        dir.join("Cargo.toml").exists()
            || dir.join("main.rs").exists()
            || dir.join("main.py").exists()
            || dir.join("server.js").exists()
            || dir.join("app.py").exists()
    };

    let has_node_project = |dir: &Path| -> bool { dir.join("package.json").exists() };

    // Check if workspace has actual project files (development phase)
    let workspace_has_files = workspace.exists()
        && (has_frontend_files(&workspace)
            || has_backend_files(&workspace)
            || has_node_project(&workspace));

    // Check if project root has actual project files (after delivery)
    let project_root_has_files = has_frontend_files(&project_root)
        || has_backend_files(&project_root)
        || has_node_project(&project_root);

    // Priority: workspace > project root
    // Use workspace if it exists and has files (development phase)
    // Otherwise use project root (after delivery)
    if workspace_has_files {
        println!(
            "[GUI] Using workspace for preview/run: {}",
            workspace.display()
        );
        Ok(workspace)
    } else if project_root_has_files {
        println!(
            "[GUI] Using project root for preview/run: {}",
            project_root.display()
        );
        Ok(project_root)
    } else if workspace.exists() {
        // Fallback: workspace exists but no project files
        println!("[GUI] Using workspace (fallback): {}", workspace.display());
        Ok(workspace)
    } else {
        Err(format!(
            "No valid code directory found. Workspace: {}, Project root: {}",
            workspace.display(),
            project_root.display()
        ))
    }
}

/// 使用大模型分析项目结构并生成运行命令
fn detect_start_command(code_dir: &Path) -> Result<String, String> {
    // 使用 RuntimeAnalyzer (LLM分析) 获取运行配置
    let analyzer_result = try_analyze_with_runtime(code_dir);

    if let Ok(config) = analyzer_result {
        // 从运行时配置生成启动命令
        if let Some(cmd) = generate_start_command_from_config(&config) {
            println!(
                "[GUI] LLM detected runtime type: {:?}, command: {}",
                config.runtime_type, cmd
            );
            return Ok(cmd);
        }
    }

    // LLM 分析失败，返回错误
    Err("无法通过大模型分析获取运行命令，请确保在设置中配置了有效的 LLM".to_string())
}

/// Try to analyze project using RuntimeAnalyzer
fn try_analyze_with_runtime(code_dir: &Path) -> Result<ProjectRuntimeConfig, String> {
    // Initialize analyzer with config if not already done
    init_runtime_analyzer();

    let guard = RUNTIME_ANALYZER.lock().map_err(|e| e.to_string())?;

    if let Some(analyzer) = guard.as_ref() {
        // Clone the necessary data to move into blocking task
        let code_dir = code_dir.to_path_buf();

        // Use spawn_blocking to run the async analysis in a blocking context
        let result = tokio::task::block_in_place(move || {
            let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            rt.block_on(async { analyzer.analyze(&code_dir).await })
        });

        return result;
    }

    Err("RuntimeAnalyzer not initialized".to_string())
}

/// Initialize RuntimeAnalyzer with LLM config
fn init_runtime_analyzer() {
    let mut guard = match RUNTIME_ANALYZER.lock() {
        Ok(g) => g,
        Err(e) => {
            println!("[GUI] Failed to lock RUNTIME_ANALYZER: {}", e);
            return;
        }
    };

    if guard.is_some() {
        return; // Already initialized
    }

    if let Ok(config) = load_config() {
        let llm_config = LlmConfig {
            api_base_url: config.llm.api_base_url,
            api_key: config.llm.api_key,
            model_name: config.llm.model_name,
        };
        let analyzer = RuntimeAnalyzer::new().with_llm_config(llm_config);
        *guard = Some(analyzer);
        println!("[GUI] RuntimeAnalyzer initialized with LLM config");
        return;
    }

    // No config found, use heuristic-only analyzer
    *guard = Some(RuntimeAnalyzer::new());
    println!("[GUI] RuntimeAnalyzer initialized (heuristic mode)");
}

/// Generate start command from runtime config
fn generate_start_command_from_config(config: &ProjectRuntimeConfig) -> Option<String> {
    // Use dev command if available
    if let Some(ref frontend) = config.frontend {
        if !frontend.dev_command.is_empty() {
            return Some(frontend.dev_command.clone());
        }
    }

    if let Some(ref backend) = config.backend {
        if !backend.dev_command.is_empty() {
            return Some(backend.dev_command.clone());
        }
        if let Some(ref start_cmd) = backend.start_command {
            if !start_cmd.is_empty() {
                return Some(start_cmd.clone());
            }
        }
    }

    // Check runtime type for common patterns
    match config.runtime_type {
        cowork_core::RuntimeType::ReactVite => Some("npm run dev".to_string()),
        cowork_core::RuntimeType::VueVite => Some("npm run dev".to_string()),
        cowork_core::RuntimeType::RustBackend => Some("cargo run".to_string()),
        cowork_core::RuntimeType::NodeExpress => Some("node index.js".to_string()),
        cowork_core::RuntimeType::NodeFastify => Some("node index.js".to_string()),
        cowork_core::RuntimeType::PythonFastapi => Some("uvicorn main:app --reload".to_string()),
        cowork_core::RuntimeType::PythonFlask => Some("flask run".to_string()),
        // VanillaHtml uses built-in static server, handled separately
        cowork_core::RuntimeType::VanillaHtml => None,
        cowork_core::RuntimeType::NodeTool => Some("node index.js".to_string()),
        cowork_core::RuntimeType::RustCli => Some("cargo run".to_string()),
        _ => None,
    }
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
    println!(
        "[GUI] Querying memory index: query_type={}, stage={:?}, limit={}, iteration_id={:?}",
        query_type, stage, limit, iteration_id
    );

    let store = cowork_core::persistence::MemoryStore::new();

    // Convert parameters to new MemoryQuery format
    let scope = cowork_core::domain::MemoryScope::Smart; // Default to smart query
    let query_type_enum = match query_type.as_str() {
        "decisions" => cowork_core::domain::MemoryQueryType::Decisions,
        "patterns" => cowork_core::domain::MemoryQueryType::Patterns,
        "insights" => cowork_core::domain::MemoryQueryType::Insights,
        _ => cowork_core::domain::MemoryQueryType::All,
    };

    let keywords = if let Some(s) = stage { vec![s] } else { vec![] };

    let query = cowork_core::domain::MemoryQuery {
        scope,
        query_type: query_type_enum,
        keywords,
        limit: Some(limit as usize),
    };

    let result = store
        .query(&query, iteration_id.as_deref())
        .map_err(|e| format!("Failed to query memory: {}", e))?;

    // Convert to format expected by frontend: { results: [], total: N }
    let mut results: Vec<serde_json::Value> = Vec::new();

    // Add decisions
    for decision in &result.decisions {
        let decision_data = serde_json::json!({
            "id": decision.id,
            "title": decision.title.chars().take(50).collect::<String>(),
            "summary": decision.context.clone(),
            "category": "decision",
            "stage": "",
            "created_at": decision.created_at.to_rfc3339(),
            "impact": "medium",
            "tags": vec![] as Vec<String>,
            "file": format!("memory/decisions.json")
        });
        results.push(decision_data);
    }

    // Add patterns
    for pattern in &result.patterns {
        let pattern_data = serde_json::json!({
            "id": pattern.id,
            "title": pattern.name.chars().take(50).collect::<String>(),
            "summary": pattern.usage.join("; "),
            "category": "pattern",
            "stage": "",
            "created_at": pattern.created_at.to_rfc3339(),
            "impact": "high",
            "tags": pattern.tags.clone(),
            "file": format!("memory/patterns.json")
        });
        results.push(pattern_data);
    }

    // Add insights
    for insight in &result.insights {
        let impact = match insight.importance {
            cowork_core::domain::Importance::Critical => "high",
            cowork_core::domain::Importance::Important => "medium",
            cowork_core::domain::Importance::Normal => "low",
        };

        let insight_data = serde_json::json!({
            "id": format!("insight-{}", insight.created_at.timestamp()),
            "title": format!("Insight: {}", insight.content.chars().take(40).collect::<String>()),
            "summary": insight.content.clone(),
            "category": "experience",
            "stage": insight.stage.clone(),
            "created_at": insight.created_at.to_rfc3339(),
            "impact": impact,
            "tags": vec![] as Vec<String>,
            "file": format!("memory/insights.json")
        });
        results.push(insight_data);
    }

    // Sort by created_at (newest first)
    results.sort_by(|a, b| {
        let a_time = a.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
        let b_time = b.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
        b_time.cmp(a_time)
    });

    // Apply limit
    results.truncate(limit as usize);

    let total = results.len();

    Ok(serde_json::json!({
        "results": results,
        "total": total
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
    println!(
        "[GUI] Loading memory detail: memory_id={}, iteration_id={:?}",
        memory_id, iteration_id
    );

    let store = cowork_core::persistence::MemoryStore::new();

    // Try to find the memory item in project memory first
    let project_memory = store
        .load_project_memory()
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
    println!(
        "[GUI] Saving session memory: memory_type={}, title={}, iteration_id={}",
        memory_type, title, iteration_id
    );

    let store = cowork_core::persistence::MemoryStore::new();
    let mut memory = store
        .load_iteration_memory(&iteration_id)
        .map_err(|e| format!("Failed to load iteration memory: {}", e))?;

    match memory_type.as_str() {
        "decision" => {
            let decision =
                cowork_core::domain::Decision::new(&title, &summary, &content, &iteration_id);
            store
                .add_decision(decision)
                .map_err(|e| format!("Failed to add decision: {}", e))?;
        }
        "pattern" => {
            let mut pattern = cowork_core::domain::Pattern::new(&title, &content, &iteration_id);
            pattern.tags = vec![stage.clone()];
            store
                .add_pattern(pattern)
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

    store
        .save_iteration_memory(&memory)
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
    println!(
        "[GUI] Promoting to project memory: memory_id={}, iteration_id={}",
        memory_id, iteration_id
    );

    let store = cowork_core::persistence::MemoryStore::new();
    let iter_memory = store
        .load_iteration_memory(&iteration_id)
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
                        &iteration_id,
                    );
                    store
                        .add_decision(decision)
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
                        &iteration_id,
                    );
                    store
                        .add_pattern(pattern)
                        .map_err(|e| format!("Failed to add pattern: {}", e))?;
                    return Ok(serde_json::json!({
                        "message": "Promoted to project pattern successfully",
                        "memory_id": memory_id
                    }));
                }
            }
        }
    }

    Err(format!(
        "Memory item not found for promotion: {}",
        memory_id
    ))
}

#[tauri::command]
pub async fn get_memory_context(
    iteration_id: Option<String>,
    _window: Window,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!(
        "[GUI] Getting memory context: iteration_id={:?}",
        iteration_id
    );

    let store = cowork_core::persistence::MemoryStore::new();

    // Get project memory
    let project_memory = store
        .load_project_memory()
        .map_err(|e| format!("Failed to load project memory: {}", e))?;

    // Get iteration memory if available
    let iteration_memory = if let Some(iter_id) = iteration_id {
        Some(
            store
                .load_iteration_memory(&iter_id)
                .map_err(|e| format!("Failed to load iteration memory: {}", e))?,
        )
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

    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

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

    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

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
                            if let Ok(template) = serde_json::from_str::<ProjectTemplate>(&content)
                            {
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

    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

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
    fs::write(
        &template_file,
        serde_json::to_string_pretty(&template).unwrap(),
    )
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
    fs::write(
        &template_file,
        serde_json::to_string_pretty(&template).unwrap(),
    )
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

    fs::remove_file(&template_file).map_err(|e| format!("Failed to delete template: {}", e))?;

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
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Replace variables in content
        let content =
            replace_template_variables(&file.content, &variables, &template.config.variables);

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
    let config_dir = dirs::config_dir().ok_or("Failed to get config directory")?;

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
                        let relative_path = path
                            .strip_prefix(dir)
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

fn replace_template_variables(
    content: &str,
    variables: &serde_json::Value,
    config_vars: &[TemplateVariable],
) -> String {
    let mut result = content.to_string();

    // Replace variables
    for var in config_vars {
        let placeholder = format!("{{{{{}}}}}", var.name);
        let value = variables
            .get(&var.name)
            .and_then(|v| v.as_str())
            .unwrap_or(&var.default_value);
        result = result.replace(&placeholder, value);
    }

    result
}

// ============================================================================
// Project Manager Agent Commands
// ============================================================================

/// PM Agent message response
#[derive(Debug, Clone, serde::Serialize)]
pub struct PMMessageResponse {
    pub agent_message: String,
    pub actions: Vec<PMActionInfo>,
    pub needs_restart: bool,
    pub target_stage: Option<String>,
    pub new_iteration_id: Option<String>,
}

/// PM Action information for frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct PMActionInfo {
    pub action_type: String,
    pub description: String,
}

#[tauri::command]
pub async fn pm_send_message(
    iteration_id: String,
    message: String,
    history: Vec<serde_json::Value>,
    window: Window,
) -> Result<PMMessageResponse, String> {
    println!("[GUI] PM Agent processing message for iteration: {} (history: {} messages)", iteration_id, history.len());

    // Load LLM config
    let model_config = load_model_config()?;
    let llm_client = cowork_core::llm::config::create_llm_client(&model_config.llm)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;

    // Use the cowork-core helper to run PM agent
    let result = cowork_core::agents::execute_pm_agent_message(
        llm_client,
        iteration_id.clone(),
        message.clone(),
        history,
    ).await
    .map_err(|e| format!("PM agent execution failed: {}", e))?;

    // Extract response
    let mut agent_message = result.0;
    let _parts = result.1;

    // Parse actions from embedded <!--ACTIONS:...--> marker
    let mut actions = Vec::new();
    let mut needs_restart = false;
    let mut target_stage = None;
    let new_iteration_id = None;

    // Try to extract actions from the message marker
    if let Some(start_idx) = agent_message.find("<!--ACTIONS:") {
        if let Some(end_idx) = agent_message[start_idx..].find("-->") {
            let actions_json = &agent_message[start_idx + 12..start_idx + end_idx];
            
            if let Ok(parsed_actions) = serde_json::from_str::<Vec<serde_json::Value>>(actions_json) {
                for action in parsed_actions {
                    let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let pm_action = PMActionInfo {
                        action_type: action_type.to_string(),
                        description: action.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    };
                    
                    // Check for specific actions
                    if action_type == "pm_goto_stage" {
                        needs_restart = true;
                        target_stage = action.get("target_stage").and_then(|v| v.as_str()).map(|s| s.to_string());
                    }
                    
                    actions.push(pm_action);
                }
            }
            
            // Remove the marker from displayed message
            agent_message = format!("{}{}", &agent_message[..start_idx], &agent_message[start_idx + end_idx + 3..]);
        }
    }

    // Emit response event
    let _ = window.emit("pm_message", serde_json::json!({
        "iteration_id": iteration_id,
        "agent_message": &agent_message,
        "actions": &actions
    }));

    Ok(PMMessageResponse {
        agent_message,
        actions,
        needs_restart,
        target_stage,
        new_iteration_id,
    })
}

#[tauri::command]
pub async fn pm_restart_iteration(
    iteration_id: String,
    target_stage: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!("[GUI] PM Agent restarting iteration {} from stage {}", iteration_id, target_stage);

    // Load iteration
    let iteration_store = IterationStore::new();
    let mut iteration = iteration_store.load(&iteration_id)
        .map_err(|e| format!("Failed to load iteration: {}", e))?;

    // Update iteration status
    iteration.status = cowork_core::domain::IterationStatus::Draft;
    iteration.current_stage = Some(target_stage.clone());
    iteration_store.save(&iteration)
        .map_err(|e| format!("Failed to save iteration: {}", e))?;

    // Start execution
    gui_execute_iteration(iteration_id, window, state).await
}

#[tauri::command]
pub async fn pm_get_iteration_context(
    iteration_id: String,
) -> Result<serde_json::Value, String> {
    println!("[GUI] Getting iteration context for PM Agent: {}", iteration_id);

    let iteration_store = IterationStore::new();
    let iteration = iteration_store.load(&iteration_id)
        .map_err(|e| format!("Failed to load iteration: {}", e))?;

    // Load artifacts summary
    let artifacts_summary = load_artifacts_summary(&iteration_id)?;

    Ok(serde_json::json!({
        "iteration_id": iteration_id,
        "title": iteration.title,
        "description": iteration.description,
        "status": format!("{:?}", iteration.status),
        "artifacts": artifacts_summary
    }))
}

fn load_artifacts_summary(iteration_id: &str) -> Result<serde_json::Value, String> {
    let iteration_store = IterationStore::new();
    let iteration_dir = iteration_store.iteration_path(iteration_id)
        .map_err(|e| format!("Failed to get iteration path: {}", e))?;

    let artifacts_dir = iteration_dir.join("artifacts");

    let mut summary = serde_json::Map::new();

    // Check each artifact file
    let artifact_files = [
        ("idea", "idea.md"),
        ("prd", "prd.md"),
        ("design", "design.md"),
        ("plan", "plan.md"),
        ("delivery_report", "delivery_report.md"),
    ];

    for (key, filename) in artifact_files {
        let path = artifacts_dir.join(filename);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                summary.insert(key.to_string(), serde_json::json!({
                    "exists": true,
                    "size": content.len(),
                    "preview": content.chars().take(200).collect::<String>()
                }));
            }
        }
    }

    Ok(serde_json::Value::Object(summary))
}

fn load_model_config() -> Result<cowork_core::llm::config::ModelConfig, String> {
    load_config().map_err(|e| format!("Failed to load config: {}", e))
}
