use crate::gui_types::*;
use crate::static_server;
use crate::commands::PROJECT_RUNNER;
use cowork_core::persistence::IterationStore;
use cowork_core::RuntimeType;
use std::path::PathBuf;
use std::fs;

fn get_code_directory(iteration_id: &str) -> Result<PathBuf, String> {
    let iteration_store = IterationStore::new();
    let workspace = iteration_store
        .workspace_path(iteration_id)
        .map_err(|e| e.to_string())?;
    
    println!("[Runner] Workspace path from store: {:?}", workspace);
    
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    println!("[Runner] Current directory: {:?}", project_root);
    
    if workspace.exists() {
        println!("[Runner] Using workspace path: {:?}", workspace);
        Ok(workspace)
    } else {
        println!("[Runner] Workspace not found, using current dir: {:?}", project_root);
        Ok(project_root)
    }
}

async fn install_deps_if_needed(workspace: &std::path::Path) -> Result<(), String> {
    let pkg = workspace.join("package.json");
    let mods = workspace.join("node_modules");
    if pkg.exists() && !mods.exists() {
        let (cmd, args) = if which::which("bun").is_ok() { ("bun", vec!["install"]) }
        else if which::which("npm").is_ok() { ("npm", vec!["install"]) }
        else { return Ok(()) };
        
        println!("[Runner] Installing dependencies with {} {:?}", cmd, args);
        let out = std::process::Command::new(cmd).args(&args).current_dir(workspace).output();
        if let Ok(r) = out {
            if !r.status.success() { 
                println!("[Runner] Install warning: {}", String::from_utf8_lossy(&r.stderr));
            } else {
                println!("[Runner] Dependencies installed successfully");
            }
        }
    }
    Ok(())
}

fn try_analyze(code_dir: &std::path::Path) -> Result<cowork_core::ProjectRuntimeConfig, String> {
    let config = cowork_core::llm::config::load_config().ok();
    let analyzer = if let Some(cfg) = config {
        cowork_core::RuntimeAnalyzer::new().with_llm_config(cfg.llm)
    } else {
        cowork_core::RuntimeAnalyzer::new()
    };
    
    println!("[Runner] Attempting LLM analysis...");
    let result = tokio::task::block_in_place(|| {
        tauri::async_runtime::handle().block_on(async { analyzer.analyze(code_dir).await })
    });
    
    match &result {
        Ok(config) => println!("[Runner] LLM analysis succeeded, runtime type: {:?}", config.runtime_type),
        Err(e) => println!("[Runner] LLM analysis failed: {}", e),
    }
    
    result
}

/// Check if directory is a vanilla HTML project (has index.html, no package.json/Cargo.toml)
fn is_vanilla_html_project(dir: &std::path::Path) -> bool {
    let has_index_html = dir.join("index.html").exists();
    let has_package_json = dir.join("package.json").exists();
    let has_cargo_toml = dir.join("Cargo.toml").exists();
    
    has_index_html && !has_package_json && !has_cargo_toml
}

/// Check if directory has HTML files in root
fn has_html_files(dir: &std::path::Path) -> bool {
    if dir.join("index.html").exists() {
        return true;
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
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

/// Detect start command from package.json
fn detect_npm_start_command(dir: &std::path::Path) -> Option<String> {
    let pkg_path = dir.join("package.json");
    if !pkg_path.exists() {
        return None;
    }
    
    if let Ok(content) = fs::read_to_string(&pkg_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
                // Try common start scripts in order
                for script_name in &["dev", "start", "serve"] {
                    if scripts.get(*script_name).and_then(|s| s.as_str()).is_some() {
                        let pkg_manager = if which::which("bun").is_ok() { "bun" } else { "npm" };
                        let command = format!("{} run {}", pkg_manager, script_name);
                        println!("[Runner] Detected start command from package.json: {}", command);
                        return Some(command);
                    }
                }
            }
        }
    }
    
    None
}

#[tauri::command]
pub async fn start_iteration_project(iteration_id: String) -> Result<RunInfo, String> {
    println!("[Runner] ========== Starting project for iteration: {} ==========", iteration_id);
    let code_dir = get_code_directory(&iteration_id)?;
    
    if !code_dir.exists() {
        return Err(format!("Code directory does not exist: {:?}", code_dir));
    }
    
    println!("[Runner] Code directory exists, contents:");
    if let Ok(entries) = fs::read_dir(&code_dir) {
        for entry in entries.flatten() {
            println!("[Runner]   - {:?}", entry.file_name());
        }
    }
    
    install_deps_if_needed(&code_dir).await?;
    
    // Try LLM-based analysis first
    let config_result = try_analyze(&code_dir);
    
    // Check if it's a vanilla HTML project
    let is_vanilla_html = is_vanilla_html_project(&code_dir);
    println!("[Runner] Is vanilla HTML: {}", is_vanilla_html);
    
    // Check for package.json
    let has_package_json = code_dir.join("package.json").exists();
    println!("[Runner] Has package.json: {}", has_package_json);
    
    if let Ok(ref config) = config_result {
        println!("[Runner] Runtime type from LLM: {:?}", config.runtime_type);
        
        if config.runtime_type == RuntimeType::VanillaHtml || is_vanilla_html {
            println!("[Runner] Starting static server for HTML project");
            let srv = static_server::start_static_server(iteration_id.clone(), code_dir.clone(), 8000, None)?;
            return Ok(RunInfo { 
                status: RunStatus::Running, 
                process_id: None, 
                command: Some(format!("Static server on port {}", srv.port)), 
                ..Default::default() 
            });
        }
        
        if is_fullstack(&config.runtime_type) {
            return start_fullstack(iteration_id, code_dir, config).await;
        }
        
        // LLM detected a specific runtime type, try to use its command
        if let Some(cmd) = get_start_command_from_config(config) {
            println!("[Runner] Using command from LLM config: {}", cmd);
            let pid = PROJECT_RUNNER.start(iteration_id.clone(), cmd.clone(), code_dir.to_string_lossy().to_string(), None, None).await?;
            return Ok(RunInfo { status: RunStatus::Running, process_id: Some(pid), command: Some(cmd), ..Default::default() });
        }
    }
    
    // LLM analysis failed or didn't provide a command - try fallback detection
    
    // Fallback 1: HTML project without package.json
    if is_vanilla_html || has_html_files(&code_dir) {
        println!("[Runner] Fallback: starting static server for HTML files");
        let srv = static_server::start_static_server(iteration_id.clone(), code_dir.clone(), 8000, None)?;
        return Ok(RunInfo { 
            status: RunStatus::Running, 
            process_id: None, 
            command: Some(format!("Static server on port {}", srv.port)), 
            ..Default::default() 
        });
    }
    
    // Fallback 2: Node.js project with package.json
    if has_package_json {
        if let Some(cmd) = detect_npm_start_command(&code_dir) {
            println!("[Runner] Fallback: using npm script: {}", cmd);
            let pid = PROJECT_RUNNER.start(iteration_id.clone(), cmd.clone(), code_dir.to_string_lossy().to_string(), None, None).await?;
            return Ok(RunInfo { status: RunStatus::Running, process_id: Some(pid), command: Some(cmd), ..Default::default() });
        }
        
        // No start script but has package.json - try common defaults
        let pkg_manager = if which::which("bun").is_ok() { "bun" } else { "npm" };
        let default_cmd = format!("{} run dev", pkg_manager);
        println!("[Runner] Fallback: trying default command: {}", default_cmd);
        
        // Check if dev script exists, otherwise try start
        if let Ok(content) = fs::read_to_string(code_dir.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if json.get("scripts").and_then(|s| s.as_object()).map(|s| s.contains_key("dev")).unwrap_or(false) {
                    let pid = PROJECT_RUNNER.start(iteration_id.clone(), default_cmd.clone(), code_dir.to_string_lossy().to_string(), None, None).await?;
                    return Ok(RunInfo { status: RunStatus::Running, process_id: Some(pid), command: Some(default_cmd), ..Default::default() });
                }
                
                if json.get("scripts").and_then(|s| s.as_object()).map(|s| s.contains_key("start")).unwrap_or(false) {
                    let cmd = format!("{} run start", pkg_manager);
                    let pid = PROJECT_RUNNER.start(iteration_id.clone(), cmd.clone(), code_dir.to_string_lossy().to_string(), None, None).await?;
                    return Ok(RunInfo { status: RunStatus::Running, process_id: Some(pid), command: Some(cmd), ..Default::default() });
                }
            }
        }
    }
    
    // Fallback 3: Cargo.toml (Rust project)
    if code_dir.join("Cargo.toml").exists() {
        let cmd = "cargo run".to_string();
        println!("[Runner] Fallback: detected Rust project, using: {}", cmd);
        let pid = PROJECT_RUNNER.start(iteration_id.clone(), cmd.clone(), code_dir.to_string_lossy().to_string(), None, None).await?;
        return Ok(RunInfo { status: RunStatus::Running, process_id: Some(pid), command: Some(cmd), ..Default::default() });
    }

    Err(format!(
        "Cannot start project. Please check:\n\
         1. For Node.js projects: ensure package.json has 'dev' or 'start' script\n\
         2. For Rust projects: ensure Cargo.toml exists\n\
         3. For static HTML: use Preview button instead\n\
         \nProject directory: {:?}",
        code_dir
    ))
}

#[tauri::command]
pub async fn stop_iteration_project(iteration_id: String) -> Result<(), String> {
    static_server::stop_static_server(&iteration_id)?;
    if let Some(_inst) = static_server::remove_fullstack_process(&iteration_id) {
        let (fk, bk) = static_server::get_fullstack_process_keys(&iteration_id);
        let _ = PROJECT_RUNNER.stop(fk).await;
        let _ = PROJECT_RUNNER.stop(bk).await;
        return Ok(());
    }
    PROJECT_RUNNER.stop(iteration_id).await
}

#[tauri::command]
pub async fn check_project_status(iteration_id: String) -> Result<bool, String> {
    if static_server::is_server_running(&iteration_id) { return Ok(true); }
    if static_server::is_fullstack_running(&iteration_id) {
        let (fk, bk) = static_server::get_fullstack_process_keys(&iteration_id);
        if PROJECT_RUNNER.is_running(&fk) || PROJECT_RUNNER.is_running(&bk) { return Ok(true); }
        static_server::remove_fullstack_process(&iteration_id);
    }
    Ok(PROJECT_RUNNER.is_running(&iteration_id))
}

#[tauri::command]
pub async fn format_code(_session_id: String, _file_path: Option<String>) -> Result<FormatResult, String> {
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut formatted = Vec::new();
    let mut errors = Vec::new();

    if root.join("package.json").exists() {
        match tokio::process::Command::new("npx").args(["prettier", "--write", "."]).current_dir(&root).output().await {
            Ok(o) if o.status.success() => formatted.push("JS/TS files formatted".to_string()),
            Ok(o) => errors.push(format!("Prettier: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => errors.push(format!("Prettier error: {}", e)),
        }
    }
    if root.join("Cargo.toml").exists() {
        match tokio::process::Command::new("cargo").args(["fmt"]).current_dir(&root).output().await {
            Ok(o) if o.status.success() => formatted.push("Rust files formatted".to_string()),
            Ok(o) => errors.push(format!("rustfmt: {}", String::from_utf8_lossy(&o.stdout))),
            Err(e) => errors.push(format!("rustfmt error: {}", e)),
        }
    }

    let success = errors.is_empty();
    Ok(FormatResult { formatted_files: formatted, errors, success })
}

#[tauri::command]
pub async fn check_formatter_available(_session_id: String) -> Result<FormatterAvailability, String> {
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut prettier = false;
    let mut rustfmt = false;

    if root.join("package.json").exists() {
        prettier = tokio::process::Command::new("npx").args(["prettier", "--version"]).output().await
            .map(|o| o.status.success()).unwrap_or(false);
    }
    if root.join("Cargo.toml").exists() {
        rustfmt = tokio::process::Command::new("cargo").args(["fmt", "--version"]).output().await
            .map(|o| o.status.success()).unwrap_or(false);
    }

    Ok(FormatterAvailability { prettier, rustfmt })
}

fn is_fullstack(rt: &RuntimeType) -> bool {
    matches!(rt, RuntimeType::FullstackReactRust | RuntimeType::FullstackReactNode | 
        RuntimeType::FullstackVanillaRust | RuntimeType::FullstackVanillaNode)
}

fn get_start_command_from_config(config: &cowork_core::ProjectRuntimeConfig) -> Option<String> {
    if let Some(ref f) = config.frontend { 
        return Some(f.dev_command.clone()); 
    }
    if let Some(ref b) = config.backend { 
        if let Some(ref cmd) = b.start_command {
            if !cmd.is_empty() {
                return Some(cmd.clone());
            }
        }
    }
    None
}

async fn start_fullstack(iteration_id: String, code_dir: PathBuf, config: &cowork_core::ProjectRuntimeConfig) -> Result<RunInfo, String> {
    let fs_cfg = config.fullstack.as_ref().ok_or("No fullstack config")?;
    let (fk, bk) = static_server::get_fullstack_process_keys(&iteration_id);

    let b_url = format!("http://localhost:{}", fs_cfg.backend_port);
    let _bpid = PROJECT_RUNNER.start(bk.clone(), fs_cfg.backend_dev_command.clone(), 
        code_dir.to_string_lossy().to_string(), Some(b_url.clone()), Some(fs_cfg.backend_port)).await?;

    for _ in 0..30 {
        if !PROJECT_RUNNER.is_running(&fk) { tokio::time::sleep(std::time::Duration::from_secs(1)).await; continue; }
        if attohttpc::get(&b_url).timeout(std::time::Duration::from_secs(1)).send().is_ok() { break; }
    }

    let f_url = format!("http://localhost:{}", fs_cfg.frontend_port);
    let fpid = PROJECT_RUNNER.start(fk.clone(), fs_cfg.frontend_dev_command.clone(),
        code_dir.to_string_lossy().to_string(), Some(f_url.clone()), Some(fs_cfg.frontend_port)).await?;

    let inst = static_server::FullstackProcessInstance {
        iteration_id: iteration_id.clone(),
        frontend_pid: Some(fpid),
        backend_pid: Some(fpid),
        frontend_port: fs_cfg.frontend_port,
        backend_port: fs_cfg.backend_port,
        frontend_url: f_url.clone(),
        backend_url: b_url.clone(),
    };
    static_server::register_fullstack_process(inst);

    Ok(RunInfo {
        status: RunStatus::Running,
        command: Some(format!("{} | {}", fs_cfg.frontend_dev_command, fs_cfg.backend_dev_command)),
        frontend_pid: Some(fpid),
        backend_pid: Some(fpid),
        frontend_url: Some(f_url),
        backend_url: Some(b_url),
        is_fullstack: true,
        ..Default::default()
    })
}
