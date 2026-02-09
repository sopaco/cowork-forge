// File operation tools with workspace support

use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::get_required_string_param;
use crate::persistence::IterationStore;
use crate::storage::get_iteration_id;

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper function to strip UNC path prefix on Windows
/// Windows uses \\?\ prefix for paths > 260 characters, but this can cause
/// issues with path comparison. This function removes the prefix if present.
pub fn strip_unc_prefix(path: &Path) -> PathBuf {
    let path_str = path.display().to_string();
    if path_str.starts_with(r"\\?\") {
        PathBuf::from(&path_str[4..])
    } else {
        path.to_path_buf()
    }
}

// ============================================================================
// Security Helper - Path Validation
// ============================================================================

/// Validate that a path is safe to access within a workspace
/// Rules:
/// 1. Must be relative path (no absolute paths like /tmp, C:\)
/// 2. Must not escape workspace directory (no ..)
/// 3. Must be within the provided workspace directory
fn validate_path_security_within_workspace(path: &str, workspace_dir: &Path) -> Result<PathBuf, String> {
    let path_obj = Path::new(path);
    
    // Rule 1: Reject absolute paths
    if path_obj.is_absolute() {
        return Err(format!(
            "Security: Absolute paths are not allowed. Path '{}' must be relative to workspace. Use relative paths like 'src/index.html' or 'components/Button.jsx'.",
            path
        ));
    }
    
    // Rule 2: Reject parent directory access (..)
    if path.contains("..") {
        return Err(format!(
            "Security: Parent directory access (..) is not allowed. Path: '{}'. Files must be within the iteration workspace directory.",
            path
        ));
    }
    
    // Rule 3: Construct full path and verify it's within workspace
    let full_path = workspace_dir.join(path);
    
    // Canonicalize both paths for reliable comparison
    // On Windows, canonicalize() returns \\?\ prefix paths
    let normalized_workspace_dir = workspace_dir.canonicalize()
        .map_err(|e| format!("Failed to canonicalize workspace directory: {}", e))?;
    
    let canonical_path = if full_path.exists() {
        full_path.canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?
    } else {
        // For non-existent paths (e.g., files to be created), we need to:
        // 1. Try to canonicalize the parent directory
        // 2. Append the filename to get the full path
        // 3. Use starts_with() check on the path components, not the full string
        if let Some(parent) = full_path.parent() {
            let canonical_parent = match parent.canonicalize() {
                Ok(p) => p,
                Err(_) => parent.to_path_buf(), // Fallback if parent doesn't exist
            };
            if let Some(filename) = full_path.file_name() {
                canonical_parent.join(filename)
            } else {
                full_path
            }
        } else {
            full_path
        }
    };
    
    // Verify the path is within workspace directory
    // For Windows, we need to handle UNC path prefixes (\\?\)
    // Strip the UNC prefix if present for comparison
    let canonical_path_stripped = strip_unc_prefix(&canonical_path);
    let workspace_dir_stripped = strip_unc_prefix(&normalized_workspace_dir);
    
    if !canonical_path_stripped.starts_with(&workspace_dir_stripped) {
        return Err(format!(
            "Security: Path escapes workspace directory. Path '{}' resolves to '{}', expected to be within workspace: '{}'",
            path,
            canonical_path_stripped.display(),
            workspace_dir_stripped.display()
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

        // Get iteration workspace path
        let iteration_id = get_iteration_id()
            .ok_or_else(|| adk_core::AdkError::Tool("Iteration ID not set. Cannot list files without an active iteration.".to_string()))?;

        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get workspace path: {}", e)))?;

        // Ensure workspace exists
        fs::create_dir_all(&workspace_dir)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to create workspace: {}", e)))?;

        // Handle the case where Agent passes the workspace path itself
        // If the path equals the workspace directory, treat it as "."
        let path_str = path.trim();
        let safe_path = if path_str == workspace_dir.display().to_string() ||
                        path_str == workspace_dir.to_string_lossy().as_ref() {
            "."
        } else if path_str.contains(".cowork-v2/iterations") && path_str.contains("workspace") {
            // Agent might have passed the full workspace path, extract just "."
            "."
        } else {
            path_str
        };

        // Security check - ensure path doesn't escape workspace
        let validated_path = match validate_path_security_within_workspace(&safe_path, &workspace_dir) {
            Ok(p) => p,
            Err(e) => {
                return Ok(json!({
                    "status": "security_error",
                    "message": e
                }));
            }
        };

        // Construct full path in workspace
        let full_path = workspace_dir.join(&validated_path);
        
        let recursive = args.get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let max_depth = args.get("max_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize;

        if !full_path.exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("Path not found: {} (in workspace: {})", path, iteration_id)
            }));
        }

        let mut files = Vec::new();
        let mut directories = Vec::new();

        if recursive {
            // Recursive listing with max depth
            for entry in WalkDir::new(&full_path)
                .max_depth(max_depth)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    // Prune hidden directories early (except the root itself)
                    if let Some(name) = e.file_name().to_str() {
                        if name.starts_with('.') && name != "." {
                            return false;
                        }
                    }
                    true
                })
                .filter_map(|e| e.ok())
            {
                // Convert to relative path (relative to workspace)
                // Strip UNC prefixes for reliable comparison
                let entry_path = entry.path();
                let entry_path_stripped = strip_unc_prefix(entry_path);
                let workspace_dir_stripped = strip_unc_prefix(&workspace_dir);

                let rel = entry_path_stripped.strip_prefix(&workspace_dir_stripped)
                    .unwrap_or(&entry_path_stripped);
                let rel_str = rel.to_string_lossy();
                let path_str = format!("./{}", rel_str.trim_start_matches("./"));

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
            let entries = fs::read_dir(&full_path)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read directory: {}", e)))?;

            for entry in entries {
                let entry = entry.map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

                // Skip hidden at top-level
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with('.') {
                        continue;
                    }
                }

                let full = entry.path().to_path_buf();
                // Strip UNC prefixes for reliable comparison
                let full_stripped = strip_unc_prefix(&full);
                let workspace_dir_stripped = strip_unc_prefix(&workspace_dir);

                let rel = full_stripped.strip_prefix(&workspace_dir_stripped)
                    .unwrap_or(&full_stripped);
                let rel_str = rel.to_string_lossy();
                let path_str = format!("./{}", rel_str.trim_start_matches("./"));

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
            "total_directories": directories.len(),
            "workspace": workspace_dir.to_string_lossy().to_string()
        }))
    }
}

fn should_ignore(path: &str) -> bool {
    // Normalize: we mostly work with "./..." relative paths now

    // 1) Hide dotfiles / dot-directories broadly
    // (We still keep root path "." out of this function; callers handle it)
    if let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            return true;
        }
    }

    // 2) Common ignore patterns
    let ignore_patterns = [
        "./.git", "./target", "./node_modules", "./.litho",
        "./.idea", "./.vscode", "./dist", "./build", "./docs", "./tests",
        "__tests__", "./.archived",
        ".DS_Store", "Thumbs.db",
    ];
    // Note: .cowork-v2 is NOT ignored - it's the V2 architecture directory structure

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
        let path = get_required_string_param(&args, "path")?;

        // Get iteration workspace path
        let iteration_id = get_iteration_id()
            .ok_or_else(|| adk_core::AdkError::Tool("Iteration ID not set. Cannot read files without an active iteration.".to_string()))?;
        
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get workspace path: {}", e)))?;

        // Security check - ensure path is within workspace
        let safe_path = match validate_path_security_within_workspace(path, &workspace_dir) {
            Ok(p) => p,
            Err(e) => {
                return Ok(json!({
                    "status": "security_error",
                    "message": e
                }));
            }
        };

        // Construct full path in workspace
        let full_path = workspace_dir.join(&safe_path);

        if !full_path.exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("File not found: {} (in workspace: {})", path, iteration_id)
            }));
        }
        
        match fs::read_to_string(&full_path) {
            Ok(content) => Ok(json!({
                "status": "success",
                "path": path,
                "workspace_path": full_path.to_string_lossy().to_string(),
                "content": content,
                "workspace": workspace_dir.to_string_lossy().to_string()
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
        let path = get_required_string_param(&args, "path")?;
        let content = get_required_string_param(&args, "content")?;

        // Get iteration workspace path
        let iteration_id = get_iteration_id()
            .ok_or_else(|| adk_core::AdkError::Tool("Iteration ID not set. Cannot write files without an active iteration.".to_string()))?;
        
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get workspace path: {}", e)))?;
        
        // Ensure workspace exists
        fs::create_dir_all(&workspace_dir)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to create workspace: {}", e)))?;

        // Security check - ensure path is within workspace
        let safe_path = match validate_path_security_within_workspace(path, &workspace_dir) {
            Ok(p) => p,
            Err(e) => {
                return Ok(json!({
                    "status": "security_error",
                    "message": e
                }));
            }
        };

        // Construct full path in workspace
        let full_path = workspace_dir.join(&safe_path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;
        }

        match fs::write(&full_path, content) {
            Ok(_) => {
                // Log file creation for user visibility
                println!("📝 Writing file: {} ({} lines) [iteration: {}]", path, content.lines().count(), iteration_id);
                Ok(json!({
                    "status": "success",
                    "path": path,
                    "workspace_path": full_path.to_string_lossy().to_string(),
                    "lines_written": content.lines().count(),
                    "workspace": workspace_dir.to_string_lossy().to_string()
                }))
            },
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
        let command = get_required_string_param(&args, "command")?;

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

        // Get iteration workspace path
        let iteration_id = get_iteration_id()
            .ok_or_else(|| adk_core::AdkError::Tool("Iteration ID not set. Cannot run command without an active iteration.".to_string()))?;
        
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get workspace path: {}", e)))?;

        // Execute command with timeout in workspace directory
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&workspace_dir) // Run in workspace
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
                    "stderr": stderr,
                    "workspace": workspace_dir.to_string_lossy().to_string()
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

// ============================================================================
// ReadFileTruncatedTool - Smart truncated file reading for knowledge generation
// ============================================================================

pub struct ReadFileTruncatedTool;

#[async_trait]
impl Tool for ReadFileTruncatedTool {
    fn name(&self) -> &str {
        "read_file_truncated"
    }

    fn description(&self) -> &str {
        "Read a file with intelligent truncation to prevent context overflow. \
         This tool automatically summarizes large files and provides a structured output. \
         Use this when you need to read files for knowledge generation but want to avoid \
         excessive token consumption."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file within workspace"
                },
                "max_chars": {
                    "type": "number",
                    "description": "Maximum characters to return (default: 2000)",
                    "default": 2000
                },
                "prefer_structure": {
                    "type": "boolean",
                    "description": "If true, prefer structure over content for code files (default: true)",
                    "default": true
                }
            },
            "required": ["path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let path = get_required_string_param(&args, "path")?;
        let max_chars = args.get("max_chars")
            .and_then(|v| v.as_i64())
            .unwrap_or(2000) as usize;
        let prefer_structure = args.get("prefer_structure")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Get iteration workspace path
        let iteration_id = get_iteration_id()
            .ok_or_else(|| adk_core::AdkError::Tool("Iteration ID not set".to_string()))?;
        
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get workspace: {}", e)))?;

        // Validate path security
        let full_path = validate_path_security_within_workspace(&path, &workspace_dir)
            .map_err(|e| adk_core::AdkError::Tool(e))?;

        // Read file content
        let content = fs::read_to_string(&full_path)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read file: {}", e)))?;

        // Check file extension for code files
        let file_ext = full_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let is_code_file = matches!(file_ext, 
            "rs" | "js" | "jsx" | "ts" | "tsx" | "py" | "java" | "go" | "cpp" | "c" | "h"
        );

        // Smart truncation based on file type and preferences
        let truncated_content = if content.len() <= max_chars {
            content.clone()
        } else if is_code_file && prefer_structure {
            // For code files, extract structure (functions, classes, etc.)
            extract_code_structure(&content, max_chars, file_ext)
        } else {
            // For other files, simple truncation with context
            truncate_with_context(&content, max_chars)
        };

        let total_chars = content.len();
        let truncated = total_chars > max_chars;

        Ok(json!({
            "path": path,
            "file_size": total_chars,
            "returned_size": truncated_content.len(),
            "truncated": truncated,
            "is_code_file": is_code_file,
            "content": truncated_content,
            "original_lines": content.lines().count(),
            "returned_lines": truncated_content.lines().count()
        }))
    }
}

// ============================================================================
// ReadFileWithLimitTool - File reading with call count tracking
// ============================================================================

pub struct ReadFileWithLimitTool {
    max_calls: usize,
    call_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl ReadFileWithLimitTool {
    pub fn new(max_calls: usize) -> Self {
        Self {
            max_calls,
            call_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub fn reset(&self) {
        self.call_count.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn calls_remaining(&self) -> usize {
        let current = self.call_count.load(std::sync::atomic::Ordering::SeqCst);
        if current >= self.max_calls {
            0
        } else {
            self.max_calls - current
        }
    }
}

#[async_trait]
impl Tool for ReadFileWithLimitTool {
    fn name(&self) -> &str {
        "read_file_with_limit"
    }

    fn description(&self) -> &str {
        "Read a file with a call limit to prevent excessive file reading. \
         This tool tracks the number of calls and enforces a maximum limit. \
         Use this when you need to read multiple files but want to control token usage."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file within workspace"
                },
                "max_chars": {
                    "type": "number",
                    "description": "Maximum characters to return (default: 3000)",
                    "default": 3000
                }
            },
            "required": ["path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // Check call limit
        let current_calls = self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        if current_calls >= self.max_calls {
            return Ok(json!({
                "status": "limit_exceeded",
                "message": format!(
                    "File reading limit exceeded (max {} calls). Please use the information you've already gathered.",
                    self.max_calls
                ),
                "calls_made": current_calls + 1,
                "max_calls": self.max_calls
            }));
        }

        let path = get_required_string_param(&args, "path")?;
        let max_chars = args.get("max_chars")
            .and_then(|v| v.as_i64())
            .unwrap_or(3000) as usize;

        // Get iteration workspace path
        let iteration_id = get_iteration_id()
            .ok_or_else(|| adk_core::AdkError::Tool("Iteration ID not set".to_string()))?;
        
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get workspace: {}", e)))?;

        // Validate path security
        let full_path = validate_path_security_within_workspace(&path, &workspace_dir)
            .map_err(|e| adk_core::AdkError::Tool(e))?;

        // Read file content
        let content = fs::read_to_string(&full_path)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read file: {}", e)))?;

        // Smart truncation
        let truncated_content = if content.len() <= max_chars {
            content.clone()
        } else {
            truncate_with_context(&content, max_chars)
        };

        Ok(json!({
            "status": "success",
            "path": path,
            "file_size": content.len(),
            "returned_size": truncated_content.len(),
            "truncated": content.len() > max_chars,
            "content": truncated_content,
            "calls_remaining": self.max_calls - (current_calls + 1)
        }))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract code structure (functions, classes, etc.) instead of full content
fn extract_code_structure(content: &str, max_chars: usize, file_ext: &str) -> String {
    let mut structure = String::new();
    
    // Define patterns for different languages
    let patterns = match file_ext {
        "rs" => vec![
            (r"(?m)^(pub )?(async )?fn \w+.*?\{", "Function"),
            (r"(?m)^pub (struct|enum|trait) \w+", "Type"),
            (r"(?m)^impl \w+", "Implementation"),
        ],
        "js" | "jsx" | "ts" | "tsx" => vec![
            (r"(?m)^(export )?(async )?function \w+.*?\{", "Function"),
            (r"(?m)^(export )?(const|let|var) \w+.*?=.*=>", "Arrow Function"),
            (r"(?m)^class \w+", "Class"),
            (r"(?m)^interface \w+", "Interface"),
        ],
        "py" => vec![
            (r"(?m)^def \w+.*?\:", "Function"),
            (r"(?m)^class \w+.*?\:", "Class"),
        ],
        _ => vec![],
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut char_count = 0;
    
    for line in &lines {
        // Check if line matches any pattern
        let mut is_structure = false;
        for (pattern, _) in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(line) {
                    is_structure = true;
                    break;
                }
            }
        }
        
        // Include structure lines and important comments
        if is_structure || line.trim().starts_with("//") || line.trim().starts_with("#") {
            let line_with_newline = format!("{}\n", line);
            if char_count + line_with_newline.len() > max_chars {
                structure.push_str("...\n");
                break;
            }
            structure.push_str(&line_with_newline);
            char_count += line_with_newline.len();
        }
    }
    
    // If no structure found, return first N lines
    if structure.is_empty() {
        let mut result = String::new();
        for line in lines.iter().take(20) {
            if result.len() + line.len() > max_chars {
                break;
            }
            result.push_str(line);
            result.push('\n');
        }
        result
    } else {
        structure
    }
}

/// Truncate content with context preservation
fn truncate_with_context(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        return content.to_string();
    }
    
    // For longer content, return beginning and end with ellipsis
    let keep_start = max_chars / 2;
    let keep_end = max_chars - keep_start - 10; // 10 for ellipsis
    
    let start = &content[..keep_start];
    let end = &content[content.len().saturating_sub(keep_end)..];
    
    format!("{}\n\n[... content truncated ...]\n\n{}", start, end)
}


