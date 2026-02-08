// Preview server management for GUI
use super::gui_types::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread;
use std::fs;

use tiny_http::{Server, Response, Request, Header, StatusCode};

pub struct PreviewServerManager {
    servers: Arc<Mutex<HashMap<String, PreviewServer>>>,
}

struct PreviewServer {
    port: u16,
    base_dir: PathBuf,
    handle: Option<thread::JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl PreviewServerManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self, iteration_id: String, base_dir: PathBuf) -> Result<PreviewInfo, String> {
        // Stop existing server if any
        if let Ok(()) = self.stop(iteration_id.clone()).await {
            println!("[Preview] Stopped existing server for iteration: {}", iteration_id);
        }

        let port = self.find_available_port()?;
        let base_dir_clone = base_dir.clone();
        let iteration_id_clone = iteration_id.clone();

        // Create running flag
        let running = Arc::new(Mutex::new(true));

        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            println!("[Preview] Starting server on port {} for iteration: {}", port, iteration_id_clone);
            
            match Server::http(format!("0.0.0.0:{}", port)) {
                Ok(server) => {
                    println!("[Preview] Server started successfully");
                    
                    for request in server.incoming_requests() {
                        // Check if still running
                        if !*running_clone.lock().unwrap() {
                            break;
                        }

                        let response = Self::handle_request(&request, &base_dir_clone);
                        let _ = request.respond(response);
                    }
                }
                Err(e) => {
                    eprintln!("[Preview] Failed to create server: {}", e);
                }
            }
            
            println!("[Preview] Server stopped for iteration: {}", iteration_id_clone);
        });

        let base_dir_clone2 = base_dir.clone();
        let mut servers = self.servers.lock().unwrap();
        servers.insert(iteration_id.clone(), PreviewServer {
            port,
            base_dir,
            handle: Some(handle),
            running,
        });

        let project_type = Self::detect_project_type(&base_dir_clone2);

        Ok(PreviewInfo {
            url: format!("http://localhost:{}", port),
            port,
            status: PreviewStatus::Running,
            project_type,
        })
    }
    
    /// Check if a server is currently running for an iteration
    pub fn is_running(&self, iteration_id: &str) -> bool {
        let servers = self.servers.lock().unwrap();
        if let Some(server) = servers.get(iteration_id) {
            *server.running.lock().unwrap()
        } else {
            false
        }
    }
    
    /// Get preview info for a running server
    pub fn get_info(&self, iteration_id: &str) -> Option<PreviewInfo> {
        let servers = self.servers.lock().unwrap();
        if let Some(server) = servers.get(iteration_id) {
            let is_running = *server.running.lock().unwrap();
            Some(PreviewInfo {
                url: format!("http://localhost:{}", server.port),
                port: server.port,
                status: if is_running { PreviewStatus::Running } else { PreviewStatus::Stopped },
                project_type: Self::detect_project_type(&server.base_dir),
            })
        } else {
            None
        }
    }

    pub async fn stop(&self, iteration_id: String) -> Result<(), String> {
        let mut servers = self.servers.lock().unwrap();
        
        if let Some(mut server) = servers.remove(&iteration_id) {
            println!("[Preview] Stopping server for iteration: {}", iteration_id);
            
            // Signal the server to stop
            *server.running.lock().unwrap() = false;
            
            // Make a request to unblock the server
            if let Err(e) = attohttpc::get(&format!("http://localhost:{}", server.port)).send() {
                println!("[Preview] Unblock request failed: {}", e);
            }
            
            if let Some(handle) = server.handle.take() {
                // Wait for thread to finish with timeout
                let _ = handle.join();
            }
            
            Ok(())
        } else {
            Err(format!("No running server for iteration: {}", iteration_id))
        }
    }

    fn find_available_port(&self) -> Result<u16, String> {
        for port in 5000..6000 {
            if Self::is_port_available(port) {
                return Ok(port);
            }
        }
        Err("No available port".to_string())
    }

    fn is_port_available(port: u16) -> bool {
        // Try to bind to the port to check if it's available
        std::net::TcpListener::bind(format!("0.0.0.0:{}", port)).is_ok()
    }

    fn detect_project_type(base_dir: &Path) -> ProjectType {
        let index_html = base_dir.join("index.html");
        let package_json = base_dir.join("package.json");
        let cargo_toml = base_dir.join("Cargo.toml");

        if index_html.exists() {
            if package_json.exists() {
                return ProjectType::React;
            } else if cargo_toml.exists() {
                return ProjectType::Html;
            }
            return ProjectType::Static;
        }

        if cargo_toml.exists() {
            return ProjectType::Unknown;
        }

        ProjectType::Unknown
    }

    fn handle_request(request: &Request, base_dir: &Path) -> Response<std::io::Cursor<Vec<u8>>> {
        let url = request.url();
        let path_str = url.trim_start_matches('/');
        let path = if path_str.is_empty() || path_str == "/" {
            base_dir.join("index.html")
        } else {
            // URL decode the path
            let decoded = urlencoding::decode(path_str)
                .unwrap_or_else(|_| std::borrow::Cow::Borrowed(path_str))
                .to_string();
            
            // Security: prevent path traversal
            let safe_path = decoded.replace("..", "");
            base_dir.join(&safe_path)
        };

        println!("[Preview] Request: {} -> {:?}", url, path);

        if !path.exists() || !path.starts_with(base_dir) {
            let response = Response::from_data(Vec::from("404 Not Found"));
            return response.with_status_code(StatusCode(404));
        }

        match fs::read(&path) {
            Ok(content) => {
                let mime_type = Self::get_mime_type(&path);
                let header = Header::from_bytes(&b"Content-Type"[..], mime_type.as_bytes()).unwrap();
                Response::from_data(content).with_header(header)
            }
            Err(e) => {
                eprintln!("[Preview] Error reading file: {}", e);
                Response::from_data(Vec::from(format!("500 Internal Server Error: {}", e)))
                    .with_status_code(StatusCode(500))
            }
        }
    }

    fn get_mime_type(path: &Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("html") => "text/html".to_string(),
            Some("htm") => "text/html".to_string(),
            Some("css") => "text/css".to_string(),
            Some("js") => "application/javascript".to_string(),
            Some("json") => "application/json".to_string(),
            Some("png") => "image/png".to_string(),
            Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
            Some("gif") => "image/gif".to_string(),
            Some("svg") => "image/svg+xml".to_string(),
            Some("ico") => "image/x-icon".to_string(),
            Some("woff") | Some("woff2") => "font/woff2".to_string(),
            Some("ttf") => "font/ttf".to_string(),
            Some("txt") => "text/plain".to_string(),
            Some("md") => "text/markdown".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
}