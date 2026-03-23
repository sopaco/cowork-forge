// Import commands - Handles importing existing projects into Cowork Forge

use cowork_core::importer::{
    ImportPreview, ArtifactOptions,
    ProjectAnalysis, analyze_project,
};
use cowork_core::persistence::{init_project_structure, set_workspace_path, IterationStore, ProjectStore, get_cowork_dir};
use cowork_core::domain::{Iteration, IterationStatus, Project};
use cowork_core::agents::create_legacy_project_analyzer_with_context;
use cowork_core::llm::{create_llm_client, load_config};
use cowork_core::pipeline::{PipelineContext, SimpleInvocationContext};
use adk_core::Content;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;
use chrono::Utc;

use crate::AppState;

// Import progress event types
#[derive(Debug, Clone, Serialize)]
pub struct ImportProgressEvent {
    pub step: String,
    pub message: String,
    pub progress: u8,  // 0-100
}

fn emit_progress(app_handle: &tauri::AppHandle, step: &str, message: &str, progress: u8) {
    eprintln!("[IMPORT] Progress: {}% - {} ({})", progress, message, step);
    let _ = app_handle.emit("import_progress", ImportProgressEvent {
        step: step.to_string(),
        message: message.to_string(),
        progress,
    });
}

/// Response for preview operation
#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewResponse {
    pub success: bool,
    pub preview: Option<ImportPreview>,
    pub error: Option<String>,
}

/// Response for import operation
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResponse {
    pub success: bool,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub iteration_id: Option<String>,
    pub artifacts: Vec<String>,
    pub used_llm: bool,
    pub error: Option<String>,
}

/// Preview an existing project without importing
#[tauri::command]
pub async fn preview_import(path: String) -> Result<PreviewResponse, String> {
    let project_path = PathBuf::from(&path);

    if !project_path.exists() {
        return Ok(PreviewResponse {
            success: false,
            preview: None,
            error: Some(format!("Path does not exist: {}", path)),
        });
    }

    if !project_path.is_dir() {
        return Ok(PreviewResponse {
            success: false,
            preview: None,
            error: Some(format!("Path is not a directory: {}", path)),
        });
    }

    // Check if it's already a Cowork Forge project
    if project_path.join(".cowork-v2").exists() {
        return Ok(PreviewResponse {
            success: false,
            preview: None,
            error: Some("This is already a Cowork Forge project. Use 'Open Project' instead.".to_string()),
        });
    }

    let preview = ImportPreview::from_path(&project_path);

    Ok(PreviewResponse {
        success: true,
        preview: Some(preview),
        error: None,
    })
}

/// Analyze an existing project and return detailed analysis
#[tauri::command]
pub async fn analyze_existing_project(path: String) -> Result<ProjectAnalysis, String> {
    let project_path = PathBuf::from(&path);

    analyze_project(&project_path).map_err(|e| e.to_string())
}

/// Run LLM Agent to generate artifacts
/// ADK handles the tool calling loop automatically - we just monitor the output
async fn run_llm_agent(
    project_path: &PathBuf,
    artifacts_dir: &PathBuf,
    artifact_options: &str,
    app_handle: &tauri::AppHandle,
) -> Result<Vec<String>, String> {
    eprintln!("[IMPORT] Loading LLM config...");
    
    // Load LLM configuration
    let llm_config = load_config().map_err(|e| {
        format!("Failed to load LLM config: {}", e)
    })?;

    eprintln!("[IMPORT] Creating LLM client...");
    let model = create_llm_client(&llm_config.llm).map_err(|e| {
        format!("Failed to create LLM client: {}", e)
    })?;

    let project_path_str = project_path.to_string_lossy().to_string();
    
    eprintln!("[IMPORT] Creating Legacy Project Analyzer agent...");
    // ADK LlmAgentBuilder handles tool registration and automatic tool calling
    let agent = create_legacy_project_analyzer_with_context(
        model,
        project_path_str.clone(),
        artifact_options.to_string(),
    ).map_err(|e| {
        format!("Failed to create agent: {}", e)
    })?;

    // Build the prompt for the agent
    let prompt = format!(
        r#"Analyze the project at "{}" and generate the requested artifacts.

Project Path: {}
Artifact Options: {}

Start by scanning the project structure, detecting the tech stack, and reading key files.
Then generate and save each artifact using save_artifact tool.
"#,
        project_path_str, project_path_str, artifact_options
    );

    eprintln!("[IMPORT] Executing LLM agent (ADK handles tool calling automatically)...");

    // Create a minimal pipeline context
    let project = Project::new("imported_project");
    let iteration = Iteration::create_genesis(
        &project,
        "Legacy Project Import".to_string(),
        "Initial import from existing project".to_string(),
    );
    
    // Use the provided artifacts_dir as workspace
    let workspace_path = artifacts_dir.parent().unwrap().to_path_buf();
    
    let ctx = PipelineContext::new(project, iteration, workspace_path);
    let content = Content::new("user").with_text(&prompt);

    // Create invocation context
    let invocation_ctx = Arc::new(SimpleInvocationContext::new(
        &ctx,
        &content,
        agent.clone(),
    ));

    // Execute agent - ADK automatically handles tool calling loop
    let mut stream = agent.run(invocation_ctx).await.map_err(|e| {
        format!("Agent execution failed: {}", e)
    })?;

    use futures::StreamExt;

    eprintln!("[IMPORT] Monitoring agent output...");

    // Simply monitor the stream - ADK handles all tool execution internally
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                // Extract text output from the event
                if let Some(content) = event.content() {
                    for part in &content.parts {
                        if let adk_core::Part::Text { text } = part {
                            if !text.trim().is_empty() {
                                let preview: String = text.chars().take(100).collect();
                                emit_progress(app_handle, "llm", &format!("Processing: {}...", preview), 50);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[IMPORT] Stream error: {}", e);
            }
        }
    }

    eprintln!("[IMPORT] LLM Agent completed. Checking for saved artifacts...");

    // Verify artifacts were created by checking the file system
    let check_files = ["idea.md", "prd.md", "design.md", "plan.md"];
    let mut saved_artifacts = Vec::new();
    
    // Check iteration artifacts directory
    for filename in &check_files {
        let file_path = artifacts_dir.join(filename);
        if file_path.exists() {
            eprintln!("[IMPORT] Found artifact: {}", filename);
            saved_artifacts.push(filename.to_string());
        }
    }
    
    // Also check root artifacts directory (fallback)
    if let Ok(root_cow_dir) = get_cowork_dir() {
        let root_artifacts_dir = root_cow_dir.join("artifacts");
        for filename in &check_files {
            let root_file = root_artifacts_dir.join(filename);
            if root_file.exists() && !saved_artifacts.contains(&filename.to_string()) {
                // Move file to iteration artifacts directory
                let dest_file = artifacts_dir.join(filename);
                if let Err(e) = std::fs::copy(&root_file, &dest_file) {
                    eprintln!("[IMPORT] Failed to copy {} from root artifacts: {}", filename, e);
                } else {
                    eprintln!("[IMPORT] Moved {} from root artifacts to iteration", filename);
                    saved_artifacts.push(filename.to_string());
                }
            }
        }
    }

    // Deduplicate
    saved_artifacts.sort();
    saved_artifacts.dedup();

    eprintln!("[IMPORT] Total artifacts saved: {}", saved_artifacts.len());

    Ok(saved_artifacts)
}

/// Generate artifacts using template fallback
fn generate_template_artifacts(
    analysis: &ProjectAnalysis,
    options: &cowork_core::importer::ArtifactGenerationOptions,
    artifacts_dir: &PathBuf,
    app_handle: &tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let artifacts = cowork_core::importer::generate_artifacts(analysis, options);
    let mut generated_files = Vec::new();
    let total_artifacts = artifacts.len();
    
    for (idx, artifact) in artifacts.iter().enumerate() {
        let progress = 40 + ((idx + 1) as u8 * 45 / total_artifacts.max(1) as u8);
        emit_progress(
            app_handle, 
            "generate", 
            &format!("Generating {} (template)...", artifact.filename),
            progress,
        );

        let artifact_path = artifacts_dir.join(&artifact.filename);
        eprintln!("[IMPORT] Saving artifact: {:?} to {:?}", artifact.filename, artifact_path);

        match std::fs::write(&artifact_path, &artifact.content) {
            Ok(_) => {
                eprintln!("[IMPORT] Successfully saved: {:?}", artifact_path);
                generated_files.push(artifact.filename.clone());
            }
            Err(e) => {
                eprintln!("[IMPORT] Failed to save {:?}: {}", artifact_path, e);
            }
        }
    }

    Ok(generated_files)
}

/// Import an existing project into Cowork Forge
/// Creates an initial iteration and generates artifacts using LLM (with template fallback)
#[tauri::command]
#[allow(non_snake_case)]
pub async fn import_project(
    path: String,
    projectName: Option<String>,
    generateIdea: bool,
    generatePrd: bool,
    generateDesign: bool,
    generatePlan: bool,
    scanReadme: bool,
    scanDocs: bool,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ImportResponse, String> {
    let project_path = PathBuf::from(&path);

    // Step 1: Validate path (5%)
    emit_progress(&app_handle, "validate", "Validating project path...", 5);

    if !project_path.exists() {
        return Ok(ImportResponse {
            success: false,
            project_id: None,
            project_name: None,
            iteration_id: None,
            artifacts: Vec::new(),
            used_llm: false,
            error: Some(format!("Path does not exist: {}", path)),
        });
    }

    if !project_path.is_dir() {
        return Ok(ImportResponse {
            success: false,
            project_id: None,
            project_name: None,
            iteration_id: None,
            artifacts: Vec::new(),
            used_llm: false,
            error: Some(format!("Path is not a directory: {}", path)),
        });
    }

    // Check if already a Cowork Forge project
    if project_path.join(".cowork-v2").exists() {
        return Ok(ImportResponse {
            success: false,
            project_id: None,
            project_name: None,
            iteration_id: None,
            artifacts: Vec::new(),
            used_llm: false,
            error: Some("This is already a Cowork Forge project.".to_string()),
        });
    }

    // Step 2: Get project name (10%)
    emit_progress(&app_handle, "init", "Preparing project...", 10);
    
    let proj_name = projectName.unwrap_or_else(|| {
        project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("imported_project")
            .to_string()
    });

    // Step 3: Set workspace path and initialize structure (15%)
    emit_progress(&app_handle, "init", "Initializing Cowork Forge project structure...", 15);
    
    set_workspace_path(project_path.clone());

    let cow_dir = init_project_structure(&proj_name).map_err(|e| {
        format!("Failed to initialize project: {}", e)
    })?;

    // Step 4: Create project and save (20%)
    emit_progress(&app_handle, "init", "Creating project metadata...", 20);
    
    let project = Project::new(&proj_name);
    
    let project_store = ProjectStore::new();
    project_store.save(&project).map_err(|e| {
        format!("Failed to save project: {}", e)
    })?;

    // Step 5: Create initial iteration (25%)
    emit_progress(&app_handle, "iteration", "Creating initial iteration...", 25);
    
    let now = Utc::now();
    let iteration_id = format!("iter-{}-{}", project.next_iteration_number(), now.timestamp());
    
    let mut iteration = Iteration::create_genesis(
        &project,
        "Project Import".to_string(),
        "Initial artifacts generated from project analysis".to_string(),
    );
    iteration.status = IterationStatus::Completed;
    iteration.completed_at = Some(now);
    
    // Ensure iteration ID matches our generated one
    assert_eq!(iteration.id, iteration_id, "Iteration ID mismatch");
    
    let iteration_store = IterationStore::new();
    iteration_store.save(&iteration).map_err(|e| {
        format!("Failed to save iteration: {}", e)
    })?;

    // Create iteration directory structure
    let iteration_dir = cow_dir.join("iterations").join(&iteration_id);
    let artifacts_dir = iteration_dir.join("artifacts");
    let workspace_dir = iteration_dir.join("workspace");
    
    std::fs::create_dir_all(&artifacts_dir).map_err(|e| {
        format!("Failed to create artifacts directory: {}", e)
    })?;
    std::fs::create_dir_all(&workspace_dir).map_err(|e| {
        format!("Failed to create workspace directory: {}", e)
    })?;
    
    eprintln!("[IMPORT] Iteration directory: {:?}", iteration_dir);
    eprintln!("[IMPORT] Artifacts directory: {:?}", artifacts_dir);
    
    // CRITICAL: Copy project source files to workspace
    // This ensures that partial iterations have access to the existing code
    emit_progress(&app_handle, "copy", "Copying project files to workspace...", 28);
    copy_project_to_workspace(&project_path, &workspace_dir).map_err(|e| {
        format!("Failed to copy project files: {}", e)
    })?;
    eprintln!("[IMPORT] Project files copied to workspace");

    // Step 6: Analyze project (35%)
    emit_progress(&app_handle, "analyze", "Analyzing project structure and technology stack...", 35);

    let analysis = analyze_project(&project_path).map_err(|e| {
        format!("Failed to analyze project: {}", e)
    })?;
    
    eprintln!("[IMPORT] Analysis complete: {} technologies detected", analysis.technologies.len());

    // Build artifact options string
    let artifact_options = format!(
        "generate_idea: {}, generate_prd: {}, generate_design: {}, generate_plan: {}, scan_readme: {}, scan_docs: {}",
        generateIdea, generatePrd, generateDesign, generatePlan, scanReadme, scanDocs
    );

    // Step 7: Try LLM generation first, fallback to template
    let mut used_llm = false;
    let generated_files: Vec<String>;
    
    // Check if at least one artifact is requested
    let any_artifact_requested = generateIdea || generatePrd || generateDesign || generatePlan;
    
    if any_artifact_requested {
        // Try LLM Agent first
        emit_progress(&app_handle, "llm", "Attempting LLM-powered artifact generation...", 40);
        
        match run_llm_agent(&project_path, &artifacts_dir, &artifact_options, &app_handle).await {
            Ok(files) if !files.is_empty() => {
                eprintln!("[IMPORT] LLM generation successful: {:?} files", files);
                generated_files = files;
                used_llm = true;
            }
            Ok(_) => {
                eprintln!("[IMPORT] LLM returned no files, falling back to template...");
                let options = cowork_core::importer::ArtifactGenerationOptions {
                    generate_idea: generateIdea,
                    generate_prd: generatePrd,
                    generate_design: generateDesign,
                    generate_plan: generatePlan,
                    scan_readme: scanReadme,
                    scan_docs: scanDocs,
                };
                generated_files = generate_template_artifacts(&analysis, &options, &artifacts_dir, &app_handle)?;
            }
            Err(e) => {
                eprintln!("[IMPORT] LLM generation failed: {}, falling back to template", e);
                emit_progress(&app_handle, "fallback", &format!("LLM unavailable, using template: {}", e), 45);
                let options = cowork_core::importer::ArtifactGenerationOptions {
                    generate_idea: generateIdea,
                    generate_prd: generatePrd,
                    generate_design: generateDesign,
                    generate_plan: generatePlan,
                    scan_readme: scanReadme,
                    scan_docs: scanDocs,
                };
                generated_files = generate_template_artifacts(&analysis, &options, &artifacts_dir, &app_handle)?;
            }
        }
    } else {
        generated_files = Vec::new();
    }

    // Step 8: Register project to global registry (90%)
    emit_progress(&app_handle, "register", "Registering project...", 90);

    let project_record = {
        let mut registry = state.project_registry_manager.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        registry.register_project(
            path.clone(),
            proj_name.clone(),
            Some(format!(
                "Imported project with {} detected technologies", 
                analysis.technologies.len()
            )),
        )
        .map_err(|e| format!("Failed to register project: {}", e))?
    };

    // Step 9: Complete (100%)
    let method = if used_llm { "LLM-powered" } else { "Template-based" };
    emit_progress(&app_handle, "complete", &format!("Import completed! {} ({})", iteration_id, method), 100);

    Ok(ImportResponse {
        success: true,
        project_id: Some(project_record.project_id),
        project_name: Some(project_record.name),
        iteration_id: Some(iteration_id),
        artifacts: generated_files,
        used_llm,
        error: None,
    })
}

/// Get default artifact options
#[tauri::command]
pub fn get_default_artifact_options() -> ArtifactOptions {
    ArtifactOptions::default()
}

/// Copy project files to workspace, excluding unnecessary directories
fn copy_project_to_workspace(project_path: &PathBuf, workspace_dir: &PathBuf) -> Result<(), String> {
    // Directories to skip (common build/cache/dependency directories)
    const SKIP_DIRS: &[&str] = &[
        "node_modules",
        "target",
        "dist",
        "build",
        ".git",
        ".cowork-v2",
        "__pycache__",
        ".venv",
        "venv",
        ".idea",
        ".vscode",
        "coverage",
        ".next",
        ".nuxt",
        "vendor",
        "Pods",
        ".gradle",
        "out",
    ];
    
    fn should_skip(name: &str) -> bool {
        SKIP_DIRS.iter().any(|&skip| name == skip) || name.starts_with('.')
    }
    
    fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf, skip_hidden: bool) -> Result<(), String> {
        if !dst.exists() {
            std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
        }
        
        for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files and excluded directories
            if skip_hidden && should_skip(&name) {
                continue;
            }
            
            let src_path = entry.path();
            let dst_path = dst.join(&name);
            
            let ty = entry.file_type().map_err(|e| e.to_string())?;
            
            if ty.is_dir() {
                copy_dir_recursive(&src_path, &dst_path, true)?;
            } else {
                std::fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
            }
        }
        
        Ok(())
    }
    
    copy_dir_recursive(project_path, workspace_dir, true)
}