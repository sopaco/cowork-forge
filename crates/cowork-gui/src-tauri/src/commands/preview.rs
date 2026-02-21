use crate::gui_types::*;
use crate::static_server;
use crate::commands::PROJECT_RUNNER;
use cowork_core::persistence::IterationStore;
use std::path::PathBuf;

fn get_code_directory(iteration_id: &str) -> Result<PathBuf, String> {
    let iteration_store = IterationStore::new();
    let workspace = iteration_store
        .workspace_path(iteration_id)
        .map_err(|e| e.to_string())?;

    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;

    if workspace.exists() { Ok(workspace) } else { Ok(project_root) }
}

async fn install_dependencies_if_needed(workspace: &std::path::Path) -> Result<(), String> {
    let package_json = workspace.join("package.json");
    let node_modules = workspace.join("node_modules");

    if package_json.exists() && !node_modules.exists() {
        let use_bun = which::which("bun").is_ok();
        let use_npm = which::which("npm").is_ok();

        let (cmd, args) = if use_bun { ("bun", vec!["install"]) }
        else if use_npm { ("npm", vec!["install"]) }
        else { return Ok(()) }; // Skip if no package manager

        let output = std::process::Command::new(cmd)
            .args(&args)
            .current_dir(workspace)
            .output();

        if let Ok(result) = output {
            if !result.status.success() {
                println!("[Preview] Install warning: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
    }
    Ok(())
}

fn try_analyze_runtime(code_dir: &std::path::Path) -> Result<cowork_core::ProjectRuntimeConfig, String> {
    let config = cowork_core::llm::config::load_config().ok();
    let analyzer = if let Some(cfg) = config {
        cowork_core::RuntimeAnalyzer::new().with_llm_config(cfg.llm)
    } else {
        cowork_core::RuntimeAnalyzer::new()
    };
    
    tokio::task::block_in_place(|| {
        tauri::async_runtime::handle().block_on(async {
            analyzer.analyze(code_dir).await
        })
    })
}

/// Check if directory is a vanilla HTML project
fn is_vanilla_html_project(dir: &std::path::Path) -> bool {
    let has_index_html = dir.join("index.html").exists();
    let has_package_json = dir.join("package.json").exists();
    let has_cargo_toml = dir.join("Cargo.toml").exists();
    
    has_index_html && !has_package_json && !has_cargo_toml
}

/// Check if directory has any HTML files
fn has_html_files(dir: &std::path::Path) -> bool {
    if dir.join("index.html").exists() {
        return true;
    }
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "html" {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[tauri::command]
pub async fn start_iteration_preview(iteration_id: String) -> Result<PreviewInfo, String> {
    println!("[Preview] Starting preview for iteration: {}", iteration_id);
    let code_dir = get_code_directory(&iteration_id)?;
    println!("[Preview] Code directory: {:?}", code_dir);
    
    if !code_dir.exists() {
        return Err(format!("Code directory does not exist: {:?}", code_dir));
    }
    
    install_dependencies_if_needed(&code_dir).await?;
    
    let config_result = try_analyze_runtime(&code_dir);
    let is_vanilla_html = is_vanilla_html_project(&code_dir);
    println!("[Preview] Is vanilla HTML: {}", is_vanilla_html);

    // Check via LLM analysis
    if let Ok(ref config) = config_result {
        println!("[Preview] Runtime type detected: {:?}", config.runtime_type);
        if config.runtime_type == cowork_core::RuntimeType::VanillaHtml || is_vanilla_html {
            println!("[Preview] Starting static server for HTML project");
            let server_info = static_server::start_static_server(
                iteration_id.clone(),
                code_dir.clone(),
                8000,
                None,
            )?;

            return Ok(PreviewInfo {
                url: server_info.url,
                port: server_info.port,
                status: PreviewStatus::Running,
                project_type: ProjectType::Html,
            });
        }
    } else if is_vanilla_html || has_html_files(&code_dir) {
        // Fallback: analysis failed but has HTML files
        println!("[Preview] Analysis failed but found HTML files, starting static server");
        let server_info = static_server::start_static_server(
            iteration_id.clone(),
            code_dir.clone(),
            8000,
            None,
        )?;

        return Ok(PreviewInfo {
            url: server_info.url,
            port: server_info.port,
            status: PreviewStatus::Running,
            project_type: ProjectType::Html,
        });
    }

    let (command, port, url) = detect_start_command_with_info(&code_dir, &config_result)?;

    PROJECT_RUNNER
        .start(iteration_id.clone(), command, code_dir.to_string_lossy().to_string(), Some(url.clone()), Some(port))
        .await?;

    Ok(PreviewInfo {
        url,
        port,
        status: PreviewStatus::Running,
        project_type: ProjectType::Unknown,
    })
}

#[tauri::command]
pub async fn stop_iteration_preview(iteration_id: String) -> Result<(), String> {
    static_server::stop_static_server(&iteration_id)?;
    PROJECT_RUNNER.stop(iteration_id).await
}

#[tauri::command]
pub async fn check_preview_status(iteration_id: String) -> Result<Option<PreviewInfo>, String> {
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

    if PROJECT_RUNNER.is_running(&iteration_id) {
        if let Some(info) = PROJECT_RUNNER.get_info(&iteration_id) {
            return Ok(Some(info));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn get_project_runtime_info(iteration_id: String) -> Result<ProjectRuntimeInfo, String> {
    let code_dir = get_code_directory(&iteration_id)?;
    let config_result = try_analyze_runtime(&code_dir);
    let is_vanilla_html = is_vanilla_html_project(&code_dir);

    if let Ok(config) = config_result {
        if config.runtime_type == cowork_core::RuntimeType::VanillaHtml || is_vanilla_html {
            let (url, port) = static_server::get_server_info(&iteration_id)
                .map(|s| (Some(s.url), Some(s.port)))
                .unwrap_or((Some("http://localhost:8000".to_string()), Some(8000)));

            return Ok(ProjectRuntimeInfo {
                has_frontend: true,
                has_backend: false,
                preview_url: url,
                frontend_port: port,
                backend_port: None,
                start_command: Some("(Built-in static server)".to_string()),
            });
        }

        let has_frontend = config.frontend.is_some();
        let has_backend = config.backend.is_some() || config.fullstack.is_some();

        let (preview_url, frontend_port) = if let Some(ref frontend) = config.frontend {
            (Some(format!("http://{}:{}", frontend.dev_host, frontend.dev_port)), Some(frontend.dev_port))
        } else if let Some(ref fullstack) = config.fullstack {
            (Some(format!("http://localhost:{}", fullstack.frontend_port)), Some(fullstack.frontend_port))
        } else {
            (None, None)
        };

        let backend_port = config.backend.as_ref().map(|b| b.port)
            .or_else(|| config.fullstack.as_ref().map(|f| f.backend_port));

        let start_command = generate_start_command(&config);

        Ok(ProjectRuntimeInfo { has_frontend, has_backend, preview_url, frontend_port, backend_port, start_command })
    } else {
        // Fallback: check for HTML files
        if is_vanilla_html || code_dir.join("index.html").exists() {
            return Ok(ProjectRuntimeInfo {
                has_frontend: true,
                has_backend: false,
                preview_url: Some("http://localhost:8000".to_string()),
                frontend_port: Some(8000),
                backend_port: None,
                start_command: Some("(Built-in static server)".to_string()),
            });
        }
        
        let has_frontend = code_dir.join("package.json").exists();
        let has_backend = code_dir.join("Cargo.toml").exists() || code_dir.join("main.py").exists();
        Ok(ProjectRuntimeInfo { has_frontend, has_backend, preview_url: None, frontend_port: None, backend_port: None, start_command: None })
    }
}

fn detect_start_command_with_info(
    _code_dir: &std::path::Path,
    config_result: &Result<cowork_core::ProjectRuntimeConfig, String>,
) -> Result<(String, u16, String), String> {
    if let Ok(config) = config_result {
        if config.runtime_type == cowork_core::RuntimeType::VanillaHtml {
            return Err("VanillaHtml should use built-in static server".to_string());
        }

        let (port, url) = if let Some(ref frontend) = config.frontend {
            (frontend.dev_port, format!("http://{}:{}", frontend.dev_host, frontend.dev_port))
        } else if let Some(ref backend) = config.backend {
            (backend.port, format!("http://{}:{}", backend.host, backend.port))
        } else if let Some(ref fullstack) = config.fullstack {
            (fullstack.frontend_port, format!("http://localhost:{}", fullstack.frontend_port))
        } else {
            (3000, "http://localhost:3000".to_string())
        };

        if let Some(cmd) = generate_start_command(&config) {
            return Ok((cmd, port, url));
        }
    }

    Err("Cannot detect start command. Please configure LLM in settings.".to_string())
}

fn generate_start_command(config: &cowork_core::ProjectRuntimeConfig) -> Option<String> {
    if let Some(ref frontend) = config.frontend {
        Some(frontend.dev_command.clone())
    } else if let Some(ref backend) = config.backend {
        backend.start_command.clone()
    } else if let Some(ref fullstack) = config.fullstack {
        Some(format!("{} & {}", fullstack.backend_dev_command, fullstack.frontend_dev_command))
    } else {
        None
    }
}
