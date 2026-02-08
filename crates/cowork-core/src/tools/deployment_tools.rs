// Deployment tools for copying code from workspace to project path

use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::fs;

use crate::persistence::IterationStore;
use crate::storage::get_iteration_id;

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

        println!("[Delivery] Copying files from workspace to project root...");
        println!("[Delivery] Workspace: {}", workspace_dir.display());
        println!("[Delivery] Project root: {}", project_root.display());

        // File extensions to copy (source code files)
        let extensions_to_copy = vec![
            ".html", ".htm", ".css", ".js", ".jsx", ".ts", ".tsx",
            ".json", ".md", ".txt", ".svg", ".png", ".jpg", ".jpeg"
        ];

        let mut copied_files = Vec::new();
        let mut skipped_files = Vec::new();

        // Walk through workspace directory
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
            let rel_path = src_path.strip_prefix(&workspace_dir)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get relative path: {}", e)))?;

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

        println!("[Delivery] Deployment complete: {} files copied, {} files skipped", 
                 copied_files.len(), skipped_files.len());

        Ok(json!({
            "status": "success",
            "message": format!("Deployed {} files from workspace to project root", copied_files.len()),
            "copied_files": copied_files,
            "skipped_files": skipped_files,
            "workspace": workspace_dir.to_string_lossy().to_string(),
            "project_root": project_root.to_string_lossy().to_string()
        }))
    }
}
