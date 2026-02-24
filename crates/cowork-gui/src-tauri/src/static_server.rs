// Static file server using tiny_http
// Provides built-in HTTP server for previewing static HTML projects
// Also manages Fullstack process instances

use std::collections::HashMap;
use std::fs::File;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

lazy_static::lazy_static! {
    static ref STATIC_SERVERS: Arc<Mutex<HashMap<String, StaticServerInstance>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref FULLSTACK_PROCESSES: Arc<Mutex<HashMap<String, FullstackProcessInstance>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Static server instance info
struct StaticServerInstance {
    port: u16,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

// ============================================================================
// Fullstack Process Manager
// ============================================================================

/// Fullstack process instance - manages frontend and backend processes
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FullstackProcessInstance {
    pub iteration_id: String,
    pub frontend_pid: Option<u32>,
    pub backend_pid: Option<u32>,
    pub frontend_port: u16,
    pub backend_port: u16,
    pub frontend_url: String,
    pub backend_url: String,
}

/// Register a fullstack process instance
pub fn register_fullstack_process(instance: FullstackProcessInstance) {
    let mut processes = FULLSTACK_PROCESSES.lock().unwrap();
    processes.insert(instance.iteration_id.clone(), instance);
}

/// Get fullstack process instance
#[allow(dead_code)]
pub fn get_fullstack_process(iteration_id: &str) -> Option<FullstackProcessInstance> {
    let processes = FULLSTACK_PROCESSES.lock().unwrap();
    processes.get(iteration_id).cloned()
}

/// Remove fullstack process instance
pub fn remove_fullstack_process(iteration_id: &str) -> Option<FullstackProcessInstance> {
    let mut processes = FULLSTACK_PROCESSES.lock().unwrap();
    processes.remove(iteration_id)
}

/// Check if a fullstack process is registered
pub fn is_fullstack_running(iteration_id: &str) -> bool {
    let processes = FULLSTACK_PROCESSES.lock().unwrap();
    processes.contains_key(iteration_id)
}

/// Generate process keys for frontend and backend
/// Uses "::" as separator to avoid conflicts with iteration IDs that might contain "_frontend" or "_backend"
pub fn get_fullstack_process_keys(iteration_id: &str) -> (String, String) {
    (
        format!("{}::frontend", iteration_id),
        format!("{}::backend", iteration_id),
    )
}

// ============================================================================
// Static File Server
// ============================================================================

/// Start a static file server for the given directory
pub fn start_static_server(
    iteration_id: String,
    serve_dir: PathBuf,
    preferred_port: u16,
    app_handle: Option<tauri::AppHandle>,
) -> Result<StaticServerInfo, String> {
    // Check if already running
    {
        let servers = STATIC_SERVERS.lock().unwrap();
        if servers.contains_key(&iteration_id) {
            return Err(format!(
                "Server already running for iteration: {}",
                iteration_id
            ));
        }
    }

    // Find an available port
    let port = find_available_port(preferred_port)?;

    println!(
        "[StaticServer] Starting server on port {} for {}",
        port,
        serve_dir.display()
    );

    // Create shutdown channel
    let (shutdown_tx, _shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Create server
    let server = tiny_http::Server::http(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("Failed to create server: {}", e))?;

    let server = Arc::new(server);
    let server_clone = Arc::clone(&server);

    // Store instance
    {
        let mut servers = STATIC_SERVERS.lock().unwrap();
        servers.insert(
            iteration_id.clone(),
            StaticServerInstance { port, shutdown_tx },
        );
    }

    let serve_dir_clone = serve_dir.clone();
    let iteration_id_clone = iteration_id.clone();
    let servers_ref = Arc::clone(&STATIC_SERVERS);

    // Spawn server thread
    std::thread::spawn(move || {
        loop {
            // Try to accept with timeout
            match server_clone.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(Some(request)) => {
                    handle_request(request, &serve_dir_clone, &app_handle);
                }
                Ok(None) | Err(_) => {
                    // No request or error (including timeout), check if we should shutdown
                    let should_shutdown = {
                        let servers = servers_ref.lock().unwrap();
                        !servers.contains_key(&iteration_id_clone)
                    };
                    if should_shutdown {
                        break;
                    }
                }
            }
        }
        println!(
            "[StaticServer] Server stopped for iteration: {}",
            iteration_id_clone
        );
    });

    Ok(StaticServerInfo {
        url: format!("http://localhost:{}", port),
        port,
    })
}

/// Stop the static server for the given iteration
pub fn stop_static_server(iteration_id: &str) -> Result<(), String> {
    let mut servers = STATIC_SERVERS.lock().unwrap();

    if let Some(instance) = servers.remove(iteration_id) {
        // The shutdown_tx will be dropped, signaling the server to stop
        drop(instance.shutdown_tx);
        println!(
            "[StaticServer] Stopped server for iteration: {}",
            iteration_id
        );
    }

    Ok(())
}

/// Check if a static server is running for the given iteration
pub fn is_server_running(iteration_id: &str) -> bool {
    let servers = STATIC_SERVERS.lock().unwrap();
    servers.contains_key(iteration_id)
}

/// Get server info for the given iteration
pub fn get_server_info(iteration_id: &str) -> Option<StaticServerInfo> {
    let servers = STATIC_SERVERS.lock().unwrap();
    servers.get(iteration_id).map(|instance| StaticServerInfo {
        url: format!("http://localhost:{}", instance.port),
        port: instance.port,
    })
}

/// Static server info
#[derive(Debug, Clone, serde::Serialize)]
pub struct StaticServerInfo {
    pub url: String,
    pub port: u16,
}

/// Handle an incoming HTTP request
fn handle_request(
    request: tiny_http::Request,
    serve_dir: &Path,
    app_handle: &Option<tauri::AppHandle>,
) {
    let url = request.url().to_string();

    // Parse the path (remove query string)
    let path = url.split('?').next().unwrap_or(&url);

    // Security: prevent directory traversal
    if path.contains("..") {
        let _ =
            request.respond(tiny_http::Response::from_string("Forbidden").with_status_code(403));
        return;
    }

    // Determine file path
    let file_path = if path == "/" {
        serve_dir.join("index.html")
    } else {
        // Remove leading slash
        let relative_path = path.trim_start_matches('/');
        serve_dir.join(relative_path)
    };

    // Log request
    println!("[StaticServer] {} {}", request.method(), path);

    // Emit log event
    if let Some(handle) = app_handle {
        let _ = handle.emit(
            "project_log",
            serde_json::json!({
                "stream": "stdout",
                "content": format!("[HTTP] {} {}\n", request.method(), path)
            }),
        );
    }

    // Serve file
    if file_path.exists() && file_path.is_file() {
        // Determine content type
        let content_type = get_content_type(&file_path);

        match File::open(&file_path) {
            Ok(file) => {
                let content_type_header = tiny_http::Header::from_bytes(
                    "Content-Type".as_bytes(),
                    content_type.as_bytes(),
                )
                .unwrap();
                let cors_header = tiny_http::Header::from_bytes(
                    "Access-Control-Allow-Origin".as_bytes(),
                    "*".as_bytes(),
                )
                .unwrap();

                let response = tiny_http::Response::from_file(file)
                    .with_header(content_type_header)
                    .with_header(cors_header);

                let _ = request.respond(response);
            }
            Err(e) => {
                eprintln!("[StaticServer] Error opening file: {}", e);
                let _ = request.respond(
                    tiny_http::Response::from_string("Internal Server Error").with_status_code(500),
                );
            }
        }
    } else {
        // Try to serve index.html for SPA routing
        let index_path = serve_dir.join("index.html");
        if index_path.exists() && !path.contains('.') {
            // Path has no extension, might be SPA route
            match File::open(&index_path) {
                Ok(file) => {
                    let content_type_header = tiny_http::Header::from_bytes(
                        "Content-Type".as_bytes(),
                        "text/html; charset=utf-8".as_bytes(),
                    )
                    .unwrap();

                    let response =
                        tiny_http::Response::from_file(file).with_header(content_type_header);
                    let _ = request.respond(response);
                }
                Err(_) => {
                    let _ = request.respond(
                        tiny_http::Response::from_string("Not Found").with_status_code(404),
                    );
                }
            }
        } else {
            let _ = request
                .respond(tiny_http::Response::from_string("Not Found").with_status_code(404));
        }
    }
}

/// Get MIME content type based on file extension
fn get_content_type(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8".to_string(),
        "css" => "text/css; charset=utf-8".to_string(),
        "js" | "mjs" => "application/javascript; charset=utf-8".to_string(),
        "json" => "application/json; charset=utf-8".to_string(),
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "gif" => "image/gif".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "ico" => "image/x-icon".to_string(),
        "webp" => "image/webp".to_string(),
        "woff" | "woff2" => "font/woff2".to_string(),
        "ttf" => "font/ttf".to_string(),
        "eot" => "application/vnd.ms-fontobject".to_string(),
        "mp4" => "video/mp4".to_string(),
        "webm" => "video/webm".to_string(),
        "mp3" => "audio/mpeg".to_string(),
        "wav" => "audio/wav".to_string(),
        "pdf" => "application/pdf".to_string(),
        "xml" => "application/xml; charset=utf-8".to_string(),
        "md" => "text/markdown; charset=utf-8".to_string(),
        "txt" => "text/plain; charset=utf-8".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

/// Find an available port starting from preferred_port
fn find_available_port(preferred_port: u16) -> Result<u16, String> {
    // Try preferred port first
    if is_port_available(preferred_port) {
        return Ok(preferred_port);
    }

    // Try ports in range [preferred_port, preferred_port + 100]
    for port in (preferred_port + 1)..=(preferred_port + 100) {
        if is_port_available(port) {
            return Ok(port);
        }
    }

    // Try random ports in a larger range
    for port in (8000..=9000).rev() {
        if is_port_available(port) {
            return Ok(port);
        }
    }

    Err("No available port found in range 8000-9000".to_string())
}

/// Check if a port is available
fn is_port_available(port: u16) -> bool {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    TcpListener::bind(addr).is_ok()
}
