// File operation tools
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

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
         Useful for understanding project structure."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list (default: current directory)"
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
        
        let recursive = args.get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let max_depth = args.get("max_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize;

        if !Path::new(path).exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("Path not found: {}", path)
            }));
        }

        let mut files = Vec::new();
        let mut directories = Vec::new();

        if recursive {
            // Recursive listing with max depth
            for entry in WalkDir::new(path)
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
            let entries = fs::read_dir(path)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read directory: {}", e)))?;

            for entry in entries {
                let entry = entry.map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;
                let path_str = entry.path().display().to_string();

                if should_ignore(&path_str) {
                    continue;
                }

                if entry.path().is_dir() {
                    directories.push(path_str);
                } else {
                    files.push(path_str);
                }
            }
        }

        // Sort for consistent output
        files.sort();
        directories.sort();

        Ok(json!({
            "status": "success",
            "path": path,
            "recursive": recursive,
            "directories": directories,
            "files": files,
            "total_directories": directories.len(),
            "total_files": files.len()
        }))
    }
}

/// Check if a path should be ignored
fn should_ignore(path: &str) -> bool {
    let ignore_patterns = [
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
        "Read the contents of a file."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to read"
                }
            },
            "required": ["path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let path = args["path"].as_str().unwrap();

        if !Path::new(path).exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("File not found: {}", path)
            }));
        }        match fs::read_to_string(path) {
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
        "Write content to a file. Creates parent directories if needed."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to write"
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

        // Create parent directories if needed
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;
        }

        match fs::write(path, content) {
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
// RunCommandTool
// ============================================================================

pub struct RunCommandTool;

#[async_trait]
impl Tool for RunCommandTool {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command and return the output."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                }
            },
            "required": ["command"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let command = args["command"].as_str().unwrap();

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(json!({
            "status": if output.status.success() { "success" } else { "failed" },
            "exit_code": output.status.code(),
            "stdout": stdout,
            "stderr": stderr
        }))
    }
}
