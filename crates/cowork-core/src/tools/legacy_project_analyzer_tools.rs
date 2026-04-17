// Project scanning tools for Legacy Project Analyzer Agent
// These tools help analyze existing project structures

use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use super::get_required_string_param;

// ============================================================================
// ScanProjectTool - Scans project structure and returns overview
// ============================================================================

pub struct ScanProjectTool;

#[async_trait]
impl Tool for ScanProjectTool {
    fn name(&self) -> &str {
        "scan_project"
    }

    fn description(&self) -> &str {
        "Scan the project directory and return its structure overview. Returns file tree, key directories, and detected configurations."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Absolute path to the project root directory"
                },
                "max_depth": {
                    "type": "number",
                    "description": "Maximum depth to scan (default: 4)",
                    "default": 4
                }
            },
            "required": ["project_path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let project_path = get_required_string_param(&args, "project_path")?;
        let max_depth = args.get("max_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(4) as usize;

        let path = Path::new(project_path);
        if !path.exists() {
            return Err(adk_core::AdkError::tool(format!(
                "Project path does not exist: {}",
                project_path
            )));
        }

        let mut file_tree = Vec::new();
        let walker = walkdir::WalkDir::new(path)
            .max_depth(max_depth)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.')
                    && name != "node_modules"
                    && name != "target"
                    && name != "dist"
                    && name != "build"
                    && name != "__pycache__"
                    && name != ".git"
                    && name != "vendor"
            });

        for entry in walker.filter_map(|e| e.ok()) {
            let rel_path = entry.path().strip_prefix(path).unwrap_or(entry.path());
            let rel_str = rel_path.to_string_lossy().to_string();
            if !rel_str.is_empty() {
                let is_dir = entry.path().is_dir();
                file_tree.push(json!({
                    "path": rel_str,
                    "is_dir": is_dir
                }));
            }
        }

        Ok(json!({
            "status": "success",
            "project_path": project_path,
            "file_count": file_tree.len(),
            "files": file_tree
        }))
    }
}

// ============================================================================
// DetectTechStackTool - Detects technology stack from project files
// ============================================================================

pub struct DetectTechStackTool;

#[async_trait]
impl Tool for DetectTechStackTool {
    fn name(&self) -> &str {
        "detect_tech_stack"
    }

    fn description(&self) -> &str {
        "Detect the technology stack of a project by analyzing configuration files like package.json, Cargo.toml, requirements.txt, etc."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Absolute path to the project root directory"
                }
            },
            "required": ["project_path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let project_path = get_required_string_param(&args, "project_path")?;
        let path = Path::new(project_path);

        let mut tech_stack = json!({
            "frontend": null,
            "backend": null,
            "build_tools": [],
            "database": null,
            "has_docker": false,
            "detected_files": []
        });

        // Check for package.json (Node.js)
        let pkg_json = path.join("package.json");
        if pkg_json.exists() {
            if let Ok(content) = fs::read_to_string(&pkg_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let deps = json.get("dependencies")
                        .or(json.get("devDependencies"))
                        .and_then(|d| d.as_object());
                    
                    let mut detected = Vec::new();
                    if let Some(deps) = deps {
                        for (key, _) in deps {
                            detected.push(key.clone());
                        }
                    }

                    let name = json.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown");
                    
                    tech_stack["frontend"] = json!({
                        "type": "nodejs",
                        "package": name,
                        "dependencies": detected
                    });
                    
                    tech_stack["detected_files"].as_array_mut().unwrap()
                        .push(json!("package.json"));
                }
            }
        }

        // Check for Cargo.toml (Rust)
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                let mut deps = Vec::new();
                let mut is_binary = true;
                
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("[[bin]]") || line.starts_with("[[lib]]") {
                        is_binary = false;
                    }
                    if line.starts_with("axum") || line.starts_with("actix-web") || 
                       line.starts_with("rocket") || line.starts_with("warp") ||
                       line.starts_with("serde") || line.starts_with("tokio") {
                        deps.push(line.split('=').next().unwrap_or(line).trim().to_string());
                    }
                }

                tech_stack["backend"] = json!({
                    "type": if is_binary { "rust-cli" } else { "rust" },
                    "dependencies": deps,
                    "is_binary": is_binary
                });
                
                tech_stack["detected_files"].as_array_mut().unwrap()
                    .push(json!("Cargo.toml"));
            }
        }

        // Check for requirements.txt (Python)
        let requirements = path.join("requirements.txt");
        if requirements.exists() {
            if let Ok(content) = fs::read_to_string(&requirements) {
                let deps: Vec<String> = content.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect();
                
                tech_stack["backend"] = json!({
                    "type": "python",
                    "dependencies": deps
                });
                
                tech_stack["detected_files"].as_array_mut().unwrap()
                    .push(json!("requirements.txt"));
            }
        }

        // Check for go.mod (Go)
        let go_mod = path.join("go.mod");
        if go_mod.exists() {
            tech_stack["backend"] = json!({
                "type": "golang"
            });
            tech_stack["detected_files"].as_array_mut().unwrap()
                .push(json!("go.mod"));
        }

        // Check for build configuration files
        let vite_config = path.join("vite.config.js");
        let webpack_config = path.join("webpack.config.js");
        
        if vite_config.exists() || path.join("vite.config.ts").exists() {
            tech_stack["build_tools"].as_array_mut().unwrap()
                .push(json!("vite"));
            tech_stack["detected_files"].as_array_mut().unwrap()
                .push(json!("vite.config.js"));
        }
        
        if webpack_config.exists() {
            tech_stack["build_tools"].as_array_mut().unwrap()
                .push(json!("webpack"));
            tech_stack["detected_files"].as_array_mut().unwrap()
                .push(json!("webpack.config.js"));
        }

        // Check for Docker
        if path.join("Dockerfile").exists() || path.join("docker-compose.yml").exists() {
            tech_stack["has_docker"] = serde_json::Value::Bool(true);
        }

        Ok(json!({
            "status": "success",
            "tech_stack": tech_stack
        }))
    }
}

// ============================================================================
// ReadProjectFileTool - Reads a specific file from the project
// ============================================================================

pub struct ReadProjectFileTool;

#[async_trait]
impl Tool for ReadProjectFileTool {
    fn name(&self) -> &str {
        "read_project_file"
    }

    fn description(&self) -> &str {
        "Read a specific file from the project. Use this to read README.md, configuration files, or source code."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Absolute path to the project root directory"
                },
                "relative_path": {
                    "type": "string",
                    "description": "Relative path to the file from project root"
                },
                "max_lines": {
                    "type": "number",
                    "description": "Maximum number of lines to read (default: 500)",
                    "default": 500
                }
            },
            "required": ["project_path", "relative_path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let project_path = get_required_string_param(&args, "project_path")?;
        let relative_path = get_required_string_param(&args, "relative_path")?;
        let max_lines = args.get("max_lines")
            .and_then(|v| v.as_u64())
            .unwrap_or(500) as usize;

        let path = Path::new(project_path).join(relative_path);
        
        if !path.exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("File not found: {}", relative_path)
            }));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| adk_core::AdkError::tool(format!("Failed to read file: {}", e)))?;

        let lines: Vec<&str> = content.lines().take(max_lines).collect();
        let truncated = lines.join("\n");
        let is_truncated = content.lines().count() > max_lines;

        Ok(json!({
            "status": "success",
            "file_path": relative_path,
            "content": truncated,
            "is_truncated": is_truncated,
            "total_lines": content.lines().count()
        }))
    }
}

// ============================================================================
// ListProjectDirectoryTool - Lists files in a directory
// ============================================================================

pub struct ListProjectDirectoryTool;

#[async_trait]
impl Tool for ListProjectDirectoryTool {
    fn name(&self) -> &str {
        "list_project_directory"
    }

    fn description(&self) -> &str {
        "List all files and subdirectories in a project directory."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Absolute path to the project root directory"
                },
                "relative_path": {
                    "type": "string",
                    "description": "Relative path to the directory from project root (default: '.')"
                }
            },
            "required": ["project_path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let project_path = get_required_string_param(&args, "project_path")?;
        let relative_path = args.get("relative_path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let base_path = Path::new(project_path);
        let dir_path = base_path.join(relative_path);
        
        if !dir_path.exists() {
            return Ok(json!({
                "status": "error",
                "message": format!("Directory not found: {}", relative_path)
            }));
        }

        let entries = fs::read_dir(&dir_path)
            .map_err(|e| adk_core::AdkError::tool(format!("Failed to read directory: {}", e)))?;

        let mut items = Vec::new();
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.path().is_dir();
            items.push(json!({
                "name": name,
                "is_dir": is_dir
            }));
        }

        Ok(json!({
            "status": "success",
            "directory": relative_path,
            "items": items
        }))
    }
}

// ============================================================================
// SaveArtifactTool - Saves generated artifact to artifacts directory
// ============================================================================

pub struct SaveArtifactTool;

#[async_trait]
impl Tool for SaveArtifactTool {
    fn name(&self) -> &str {
        "save_artifact"
    }

    fn description(&self) -> &str {
        "Save a generated artifact (idea.md, prd.md, design.md, plan.md) to the artifacts directory. This is MANDATORY for completing the artifact generation."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "filename": {
                    "type": "string",
                    "description": "Filename of the artifact (e.g., 'idea.md', 'prd.md', 'design.md', 'plan.md')"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content of the artifact"
                }
            },
            "required": ["filename", "content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let filename = get_required_string_param(&args, "filename")?;
        let content = get_required_string_param(&args, "content")?;

        // Validate filename
        let valid_filenames = ["idea.md", "prd.md", "design.md", "plan.md"];
        if !valid_filenames.contains(&filename) {
            return Err(adk_core::AdkError::tool(format!(
                "Invalid artifact filename: {}. Must be one of: {:?}",
                filename, valid_filenames
            )));
        }

        // Try multiple strategies to find the correct artifacts directory:
        // 1. Check global workspace path (set by GUI)
        // 2. Check current directory for .cowork-v2
        // 3. Check parent directories for .cowork-v2 (in case we're in a subdirectory)
        
        let mut artifacts_dir: Option<std::path::PathBuf> = None;
        
        // Strategy 1: Use global workspace path if set
        if let Ok(cow_dir) = crate::persistence::get_cowork_dir() {
            let iterations_dir = cow_dir.join("iterations");
            if iterations_dir.exists() {
                // Find the most recent iteration directory
                let mut latest_iteration: Option<String> = None;
                let mut latest_time = 0u64;
                
                if let Ok(entries) = std::fs::read_dir(&iterations_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Ok(metadata) = entry.metadata() {
                                if let Ok(modified) = metadata.modified() {
                                    let time = modified.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    if time > latest_time {
                                        latest_time = time;
                                        latest_iteration = path.file_name()
                                            .and_then(|n| n.to_str())
                                            .map(|s| s.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                
                if let Some(iteration_name) = latest_iteration {
                    let iter_artifacts = iterations_dir.join(&iteration_name).join("artifacts");
                    artifacts_dir = Some(iter_artifacts);
                }
            }
            
            // If no iteration found, use root artifacts directory
            if artifacts_dir.is_none() && cow_dir.exists() {
                artifacts_dir = Some(cow_dir.join("artifacts"));
            }
        }
        
        // Strategy 2: Check current directory and parents for .cowork-v2
        if artifacts_dir.is_none() {
            if let Ok(cwd) = std::env::current_dir() {
                let mut current = cwd.as_path();
                while let Some(parent) = current.parent() {
                    let cow_dir = parent.join(".cowork-v2");
                    if cow_dir.exists() {
                        let iter_dir = cow_dir.join("iterations");
                        if iter_dir.exists() {
                            // Find latest iteration
                            if let Ok(entries) = std::fs::read_dir(&iter_dir) {
                                let mut latest: Option<(std::path::PathBuf, u64)> = None;
                                for entry in entries.filter_map(|e| e.ok()) {
                                    let path = entry.path();
                                    if path.is_dir() {
                                        if let Ok(metadata) = entry.metadata() {
                                            if let Ok(modified) = metadata.modified() {
                                                let time = modified.duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs();
                                                if latest.as_ref().map_or(true, |(_, t)| time > *t) {
                                                    latest = Some((path, time));
                                                }
                                            }
                                        }
                                    }
                                }
                                if let Some((iter_path, _)) = latest {
                                    artifacts_dir = Some(iter_path.join("artifacts"));
                                }
                            }
                        }
                        break;
                    }
                    current = parent;
                }
            }
        }
        
        // Fallback: Create in current directory
        let artifacts_dir = artifacts_dir.unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.join("artifacts"))
                .unwrap_or_else(|_| std::path::PathBuf::from("artifacts"))
        });

        std::fs::create_dir_all(&artifacts_dir)
            .map_err(|e| adk_core::AdkError::tool(format!("Failed to create artifacts dir: {}", e)))?;

        let artifact_path = artifacts_dir.join(filename);
        std::fs::write(&artifact_path, content)
            .map_err(|e| adk_core::AdkError::tool(format!("Failed to write artifact: {}", e)))?;

        eprintln!("[SaveArtifactTool] Saved {} to {:?}", filename, artifact_path);

        Ok(json!({
            "status": "success",
            "message": format!("Artifact '{}' saved successfully", filename),
            "file_path": artifact_path.to_string_lossy()
        }))
    }
}
