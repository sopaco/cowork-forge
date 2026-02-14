// Deployment tools for copying code from workspace to project path - Fixed with UNC path handling

use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::fs;

use crate::persistence::IterationStore;
use crate::storage::get_iteration_id;

/// Helper function to strip UNC path prefix on Windows
fn strip_unc_prefix(path: &std::path::Path) -> std::path::PathBuf {
    let path_str = path.display().to_string();
    if path_str.starts_with(r"\\?\\") {
        std::path::PathBuf::from(&path_str[4..])
    } else {
        path.to_path_buf()
    }
}

/// CopyWorkspaceToProjectTool - Copy code from iteration workspace to project path
/// This should be used in Delivery stage to finalize the project
pub struct CopyWorkspaceToProjectTool;

#[async_trait]
impl Tool for CopyWorkspaceToProjectTool {
    fn name(&self) -> &str {
        "copy_workspace_to_project"
    }

    fn description(&self) -> &str {
        "Copy all code files from iteration workspace to the project root directory. \
         This is used in the Delivery stage to finalize the project. \
         Only copies source code files (html, css, js, etc.), not configuration or hidden files."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "confirm": {
                    "type": "boolean",
                    "description": "Must be true to confirm deployment"
                }
            },
            "required": ["confirm"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let confirm = args.get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Ok(json!({
                "status": "cancelled",
                "message": "Deployment cancelled. Set confirm=true to proceed."
            }));
        }

        // Get iteration workspace path
        let iteration_id = get_iteration_id()
            .ok_or_else(|| adk_core::AdkError::Tool("Iteration ID not set. Cannot deploy without an active iteration.".to_string()))?;

        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get workspace path: {}", e)))?;

        // Get project root directory
        let project_root = std::env::current_dir()
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get project root: {}", e)))?;

        // Check if workspace exists
        if !workspace_dir.exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("Workspace not found: {}", workspace_dir.display())
            }));
        }

        // Check if workspace is empty (safety check)
        let workspace_has_files = workspace_dir.read_dir()
            .ok()
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false);

        if !workspace_has_files {
            return Ok(json!({
                "status": "warning",
                "message": "Workspace is empty. No files to deploy. To prevent accidental file deletion, deployment was skipped.",
                "workspace": workspace_dir.to_string_lossy().to_string(),
                "note": "This usually indicates that the Coding stage did not generate any code files. Please check the Plan stage for tasks."
            }));
        }

        println!("[Delivery] Copying files from workspace to project root...");
        println!("[Delivery] Workspace: {}", workspace_dir.display());
        println!("[Delivery] Workspace absolute: {}", workspace_dir.canonicalize().unwrap_or_else(|_| workspace_dir.clone()).display());
        println!("[Delivery] Project root: {}", project_root.display());
        println!("[Delivery] Project root absolute: {}", project_root.canonicalize().unwrap_or_else(|_| project_root.clone()).display());
        println!("[Delivery] Iteration ID: {}", iteration_id);

        // File extensions to copy (source code files)
        let extensions_to_copy = vec![
            ".html", ".htm", ".css", ".js", ".jsx", ".ts", ".tsx",
            ".json", ".md", ".txt", ".svg", ".png", ".jpg", ".jpeg"
        ];

        // Protected files and directories that should NEVER be deleted
        let protected_paths = vec![
            ".cowork-v2",
            ".git",
            ".litho",
            "litho.docs",
            ".gitignore",
            ".gitattributes",
            ".zed",
            "AGENT.md",
            ".vscode",
            ".idea",
            "config.toml",
            "Cargo.toml",
            "Cargo.lock",
            "README.md",
            "LICENSE",
            ".gitignore",
            ".DS_Store",
        ];

        // Step 1: Clean up obsolete files in project root
        println!("[Delivery] Step 1: Cleaning up obsolete files in project root...");
        let mut deleted_files = Vec::new();
        let mut protected_skipped = Vec::new();

        for entry in walkdir::WalkDir::new(&project_root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let dest_path = entry.path();

            // Skip directories (we'll delete them when they become empty)
            if dest_path.is_dir() {
                continue;
            }

            // Get relative path from project root
            // Handle UNC path prefixes on Windows
            let dest_path_stripped = strip_unc_prefix(&dest_path);
            let project_root_stripped = strip_unc_prefix(&project_root);
            
            let rel_path = dest_path_stripped.strip_prefix(&project_root_stripped)
                .unwrap_or(&dest_path_stripped);

            // Check if this is a protected file/directory
            let path_str = rel_path.to_string_lossy().to_string();
            let is_protected = protected_paths.iter().any(|protected| {
                // Exact match for files
                if path_str == *protected {
                    return true;
                }
                // Check if it's inside a protected directory
                path_str.starts_with(&format!("{}/", protected))
            });

            if is_protected {
                protected_skipped.push(path_str.clone());
                println!("[Delivery] Skipped protected: {}", path_str);
                continue;
            }

            // IMPORTANT: Skip files that are inside .cowork-v2 directory
            // These are workspace files and should NOT be deleted
            // Check for both forward and backward slashes
            if path_str.starts_with(".cowork-v2/") || path_str.starts_with(".cowork-v2\\") {
                protected_skipped.push(path_str.clone());
                println!("[Delivery] Skipped workspace file: {}", path_str);
                continue;
            }

            // Check if file exists in workspace
            let src_path = workspace_dir.join(&rel_path);
            let src_path_exists = src_path.exists();

            println!("[Delivery] Checking file: {} -> Workspace path: {} (exists: {})",
                     path_str, src_path.display(), src_path_exists);

            if !src_path_exists {
                // File doesn't exist in workspace, delete it
                println!("[Delivery] File {} not found in workspace, marking for deletion", path_str);
                match fs::remove_file(&dest_path) {
                    Ok(_) => {
                        deleted_files.push(path_str.clone());
                        println!("[Delivery] Deleted obsolete file: {}", path_str);
                    }
                    Err(e) => {
                        println!("[Delivery] Warning: Failed to delete {}: {}", path_str, e);
                    }
                }
            }
        }

        // Clean up empty directories
        for entry in walkdir::WalkDir::new(&project_root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let dir_path = entry.path();

            // Skip protected directories
            let dir_path_stripped = strip_unc_prefix(&dir_path);
            let project_root_stripped = strip_unc_prefix(&project_root);
            
            let rel_path = dir_path_stripped.strip_prefix(&project_root_stripped)
                .unwrap_or(&dir_path_stripped);

            let path_str = rel_path.to_string_lossy().to_string();
            let is_protected = protected_paths.iter().any(|protected| {
                path_str == *protected || path_str.starts_with(&format!("{}/", protected))
            });

            if is_protected {
                continue;
            }

            // IMPORTANT: Skip directories inside .cowork-v2
            if path_str.starts_with(".cowork-v2/") {
                continue;
            }

            // Check if directory is empty
            if dir_path.read_dir().ok().map(|mut it| it.next().is_none()).unwrap_or(false) {
                match fs::remove_dir(&dir_path) {
                    Ok(_) => {
                        println!("[Delivery] Deleted empty directory: {}", path_str);
                    }
                    Err(e) => {
                        println!("[Delivery] Warning: Failed to delete directory {}: {}", path_str, e);
                    }
                }
            }
        }

        // Step 2: Copy files from workspace to project root
        println!("[Delivery] Step 2: Copying files from workspace...");
        let mut copied_files = Vec::new();
        let mut skipped_files = Vec::new();

        for entry in walkdir::WalkDir::new(&workspace_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let src_path = entry.path();

            // Skip directories
            if src_path.is_dir() {
                continue;
            }

            // Get relative path from workspace
            // Handle UNC path prefixes on Windows
            let src_path_stripped = strip_unc_prefix(&src_path);
            let workspace_dir_stripped = strip_unc_prefix(&workspace_dir);
            
            let rel_path = src_path_stripped.strip_prefix(&workspace_dir_stripped)
                .unwrap_or(&src_path_stripped);

            // Check if file should be copied
            let should_copy = rel_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| extensions_to_copy.iter().any(|e| *e == format!(".{}", ext)))
                .unwrap_or(false);

            if !should_copy {
                skipped_files.push(rel_path.to_string_lossy().to_string());
                continue;
            }

            // Determine destination path
            let dest_path = project_root.join(rel_path);

            // Create parent directories if needed
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| adk_core::AdkError::Tool(format!("Failed to create directory: {}", e)))?;
            }

            // Copy file
            fs::copy(&src_path, &dest_path)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to copy {}: {}", rel_path.display(), e)))?;

            copied_files.push(rel_path.to_string_lossy().to_string());
            println!("[Delivery] Copied: {}", rel_path.display());
        }

        println!("[Delivery] Deployment complete: {} files deleted, {} files copied, {} files skipped, {} protected files skipped",
                 deleted_files.len(), copied_files.len(), skipped_files.len(), protected_skipped.len());

        Ok(json!({
            "status": "success",
            "message": format!(
                "Deployed {} files from workspace to project root ({} deleted, {} protected)",
                copied_files.len(),
                deleted_files.len(),
                protected_skipped.len()
            ),
            "deleted_files": deleted_files,
            "copied_files": copied_files,
            "skipped_files": skipped_files,
            "protected_files": protected_skipped,
            "workspace": workspace_dir.to_string_lossy().to_string(),
            "project_root": project_root.to_string_lossy().to_string()
        }))
    }
}
