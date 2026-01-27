// File operation tools with SECURITY constraints
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ============================================================================
// Security Helper - Path Validation
// ============================================================================

/// Validate that a path is safe to access
/// Rules:
/// 1. Must be relative path (no absolute paths like /tmp, C:\)
/// 2. Must not escape current directory (no ..)
/// 3. Must be within current working directory or .cowork
fn validate_path_security(path: &str) -> Result<PathBuf, String> {
    let path_obj = Path::new(path);
    
    // Rule 1: Reject absolute paths
    if path_obj.is_absolute() {
        return Err(format!(
            "Security: Absolute paths are not allowed. Path '{}' must be relative to current directory.",
            path
        ));
    }
    
    // Rule 2: Reject parent directory access (..)
    if path.contains("..") {
        return Err(format!(
            "Security: Parent directory access (..) is not allowed. Path: '{}'",
            path
        ));
    }
    
    // Rule 3: Canonicalize and verify it's within current directory
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    let full_path = current_dir.join(path);
    
    // Canonicalize if path exists, otherwise just check the constructed path
    let canonical_path = if full_path.exists() {
        full_path.canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?
    } else {
        // For non-existent paths (e.g., files to be created), just verify parent
        full_path
    };
    
    // Verify the path is within current directory
    if !canonical_path.starts_with(&current_dir) {
        return Err(format!(
            "Security: Path escapes current directory. Path '{}' resolves to '{}'",
            path,
            canonical_path.display()
        ));
    }
    
    Ok(canonical_path)
}

// ============================================================================
// ListFilesTool
// ============================================================================

pub struct ListFilesTool;

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files in a directory (recursively or non-recursively). \
         SECURITY: Only works within current directory. \
         Useful for understanding project structure."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list (default: current directory). Must be relative path."
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to list files recursively (default: false)"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum depth for recursive listing (default: 3)"
                }
            }
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        
        // Security check
        let safe_path = match validate_path_security(path) {
            Ok(p) => p,
            Err(e) => {
                return Ok(json!({
                    "status": "security_error",
                    "message": e
                }));
            }
        };
        
        let recursive = args.get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let max_depth = args.get("max_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize;

        if !safe_path.exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("Path not found: {}", path)
            }));
        }

        let mut files = Vec::new();
        let mut directories = Vec::new();

        if recursive {
            // Recursive listing with max depth
            for entry in WalkDir::new(&safe_path)
                .max_depth(max_depth)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path_str = entry.path().display().to_string();
                
                // Skip hidden files and common ignore patterns
                if should_ignore(&path_str) {
                    continue;
                }

                if entry.file_type().is_dir() {
                    directories.push(path_str);
                } else {
                    files.push(path_str);
                }
            }
        } else {
            // Non-recursive listing
            let entries = fs::read_dir(&safe_path)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read directory: {}", e)))?;

            for entry in entries {
                let entry = entry.map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;
                let path_str = entry.path().display().to_string();

                if should_ignore(&path_str) {
                    continue;
                }

                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    directories.push(path_str);
                } else {
                    files.push(path_str);
                }
            }
        }

        Ok(json!({
            "status": "success",
            "path": path,
            "files": files,
            "directories": directories,
            "total_files": files.len(),
            "total_directories": directories.len()
        }))
    }
}

fn should_ignore(path: &str) -> bool {
    let ignore_patterns = vec![
        "/.git/", "/target/", "/node_modules/", "/.cowork/",
        "/.idea/", "/.vscode/", "/dist/", "/build/",
        ".DS_Store", "Thumbs.db"
    ];

    ignore_patterns.iter().any(|pattern| path.contains(pattern))
}

// ============================================================================
// ReadFileTool
// ============================================================================

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. \
         SECURITY: Only works within current directory."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to read (must be relative path within current directory)"
                }
            },
            "required": ["path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let path = args["path"].as_str().unwrap();

        // Security check
        let safe_path = match validate_path_security(path) {
            Ok(p) => p,
            Err(e) => {
                return Ok(json!({
                    "status": "security_error",
                    "message": e
                }));
            }
        };

        if !safe_path.exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("File not found: {}", path)
            }));
        }
        
        match fs::read_to_string(&safe_path) {
            Ok(content) => Ok(json!({
                "status": "success",
                "path": path,
                "content": content
            })),
            Err(e) => Ok(json!({
                "status": "error",
                "message": format!("Failed to read file: {}", e)
            })),
        }
    }
}

// ============================================================================
// WriteFileTool
// ============================================================================

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates parent directories if needed. \
         SECURITY: Only works within current directory. Absolute paths and .. are forbidden."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to write (must be relative path within current directory)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write"
                }
            },
            "required": ["path", "content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let path = args["path"].as_str().unwrap();
        let content = args["content"].as_str().unwrap();

        // Security check
        let safe_path = match validate_path_security(path) {
            Ok(p) => p,
            Err(e) => {
                return Ok(json!({
                    "status": "security_error",
                    "message": e
                }));
            }
        };

        // Create parent directories if needed
        if let Some(parent) = safe_path.parent() {
            fs::create_dir_all(parent).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;
        }

        match fs::write(&safe_path, content) {
            Ok(_) => Ok(json!({
                "status": "success",
                "path": path,
                "lines_written": content.lines().count()
            })),
            Err(e) => Ok(json!({
                "status": "error",
                "message": format!("Failed to write file: {}", e)
            })),
        }
    }
}

// ============================================================================
// RunCommandTool with blocking detection
// ============================================================================

pub struct RunCommandTool;

/// Detect if a command is a long-running service that would block execution
fn is_blocking_service_command(command: &str) -> bool {
    let blocking_patterns = vec![
        "http.server",      // python -m http.server
        "npm run dev",      // npm dev server
        "npm start",        // npm start
        "yarn dev",
        "yarn start",
        "pnpm dev",
        "pnpm start",
        "uvicorn",          // Python ASGI server
        "gunicorn",         // Python WSGI server
        "flask run",
        "django runserver",
        "rails server",
        "cargo run",        // Might be a server
        "serve",            // serve package
        "webpack-dev-server",
        "vite",
        "next dev",
    ];

    blocking_patterns.iter().any(|pattern| command.contains(pattern))
}

#[async_trait]
impl Tool for RunCommandTool {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command and return the output. \
         WARNING: This tool will REJECT commands that start long-running services \
         (like http.server, npm dev, etc.) as they would block execution. \
         Use this for: building, testing, linting - NOT for starting servers."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute (must not be a blocking service command)"
                }
            },
            "required": ["command"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let command = args["command"].as_str().unwrap();

        // Check if command would block
        if is_blocking_service_command(command) {
            return Ok(json!({
                "status": "rejected",
                "message": format!(
                    "BLOCKED: This command appears to start a long-running service: '{}'. \
                     Starting services would block the agent. \
                     If you need to verify the code works, just create the files - don't start servers.",
                    command
                )
            }));
        }

        // Execute command with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(std::env::current_dir().unwrap()) // Run in current dir
                .output()
        )
        .await;

        match output {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                Ok(json!({
                    "status": if output.status.success() { "success" } else { "failed" },
                    "exit_code": output.status.code(),
                    "stdout": stdout,
                    "stderr": stderr
                }))
            }
            Ok(Err(e)) => {
                Ok(json!({
                    "status": "error",
                    "message": format!("Failed to execute command: {}", e)
                }))
            }
            Err(_) => {
                Ok(json!({
                    "status": "timeout",
                    "message": "Command execution timeout (30s limit)"
                }))
            }
        }
    }
}
