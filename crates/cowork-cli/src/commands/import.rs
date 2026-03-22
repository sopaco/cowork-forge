//! Import an existing project into Cowork Forge command

use anyhow::{Context, Result};
use cowork_core::domain::{Iteration, IterationStatus};
use cowork_core::agents::create_legacy_project_analyzer_with_context;
use cowork_core::llm::{create_llm_client, load_config};
use cowork_core::persistence::{get_cowork_dir, init_project_structure, set_workspace_path, IterationStore, ProjectStore};
use cowork_core::pipeline::{PipelineContext, SimpleInvocationContext};
use cowork_core::importer::{analyze_project, ArtifactGenerationOptions, generate_artifacts};
use adk_core::Content;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::Utc;

/// Import an existing project into Cowork Forge
pub async fn execute(
    path: String,
    name: Option<String>,
    generate_idea: bool,
    generate_prd: bool,
    generate_design: bool,
    generate_plan: bool,
    template_only: bool,
) -> Result<()> {
    println!("📦 Importing existing project into Cowork Forge...");
    println!();

    let project_path = PathBuf::from(&path);

    // Step 1: Validate path
    println!("🔍 Step 1/6: Validating project path...");

    if !project_path.exists() {
        anyhow::bail!("Path does not exist: {}", path);
    }

    if !project_path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", path);
    }

    // Check if already a Cowork Forge project
    if project_path.join(".cowork-v2").exists() {
        anyhow::bail!("This is already a Cowork Forge project. Use 'cowork init' to work with it.");
    }

    // Step 2: Get project name
    let proj_name = name.unwrap_or_else(|| {
        project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("imported_project")
            .to_string()
    });
    println!("   Project name: {}", proj_name);

    // Step 3: Initialize Cowork Forge structure
    println!("📁 Step 2/6: Initializing Cowork Forge project structure...");

    set_workspace_path(project_path.clone());

    let cow_dir = init_project_structure(&proj_name)
        .context("Failed to initialize project structure")?;

    // Step 4: Create project and save
    println!("📝 Step 3/6: Creating project metadata...");

    let project = cowork_core::domain::Project::new(&proj_name);
    let project_store = ProjectStore::new();
    project_store.save(&project)
        .context("Failed to save project")?;

    // Step 5: Create initial iteration
    println!("🔄 Step 4/6: Creating initial iteration...");

    let now = Utc::now();
    let mut iteration = Iteration::create_genesis(
        &project,
        "Project Import".to_string(),
        "Initial artifacts generated from project analysis".to_string(),
    );
    iteration.status = IterationStatus::Completed;
    iteration.completed_at = Some(now);

    let iteration_store = IterationStore::new();
    iteration_store.save(&iteration)
        .context("Failed to save iteration")?;

    // Create iteration directory structure
    let iteration_dir = cow_dir.join("iterations").join(&iteration.id);
    let artifacts_dir = iteration_dir.join("artifacts");
    let workspace_dir = iteration_dir.join("workspace");

    std::fs::create_dir_all(&artifacts_dir)
        .context("Failed to create artifacts directory")?;
    std::fs::create_dir_all(&workspace_dir)
        .context("Failed to create workspace directory")?;

    // Step 6: Analyze project
    println!("🔬 Step 5/6: Analyzing project structure and technology stack...");

    let analysis = analyze_project(&project_path)
        .map_err(|e| anyhow::anyhow!("Failed to analyze project: {}", e))?;

    println!("   Detected technologies: {}",
        if analysis.technologies.is_empty() {
            "None".to_string()
        } else {
            analysis.technologies.iter()
                .take(5)
                .map(|t| format!("{:?}", t))
                .collect::<Vec<_>>()
                .join(", ")
        }
    );

    // Step 7: Generate artifacts
    println!("📄 Step 6/6: Generating artifacts...");

    let any_artifact_requested = generate_idea || generate_prd || generate_design || generate_plan;
    let mut generated_files = Vec::new();
    let mut used_llm = false;

    if any_artifact_requested {
        if !template_only {
            // Try LLM Agent first
            println!("   Attempting LLM-powered artifact generation...");

            match run_llm_agent_import(&project_path, &artifacts_dir,
                generate_idea, generate_prd, generate_design, generate_plan).await {
                Ok(files) if !files.is_empty() => {
                    println!("   ✅ LLM generation successful: {} files", files.len());
                    generated_files = files;
                    used_llm = true;
                }
                Ok(_) => {
                    println!("   ⚠️  LLM returned no files, falling back to template...");
                    generated_files = generate_template_artifacts(
                        &analysis, &artifacts_dir,
                        generate_idea, generate_prd, generate_design, generate_plan
                    )?;
                }
                Err(e) => {
                    println!("   ⚠️  LLM generation failed: {}", e);
                    println!("   Falling back to template generation...");
                    generated_files = generate_template_artifacts(
                        &analysis, &artifacts_dir,
                        generate_idea, generate_prd, generate_design, generate_plan
                    )?;
                }
            }
        } else {
            println!("   Using template-only generation (--template-only specified)...");
            generated_files = generate_template_artifacts(
                &analysis, &artifacts_dir,
                generate_idea, generate_prd, generate_design, generate_plan
            )?;
        }
    } else {
        println!("   No artifacts requested, skipping generation.");
    }

    // Summary
    println!();
    println!("✅ Import completed successfully!");
    println!("   Project ID: {}", project.id);
    println!("   Iteration ID: {}", iteration.id);
    println!("   Artifacts generated: {:?}", generated_files);
    println!("   Method: {}", if used_llm { "LLM-powered" } else { "Template-based" });
    println!();
    println!("Next steps:");
    println!("  cd {}", path);
    println!("  cowork status");

    Ok(())
}

/// Run LLM Agent to generate artifacts for import
async fn run_llm_agent_import(
    project_path: &PathBuf,
    artifacts_dir: &PathBuf,
    generate_idea: bool,
    generate_prd: bool,
    generate_design: bool,
    generate_plan: bool,
) -> Result<Vec<String>> {
    let llm_config = load_config()
        .context("Failed to load LLM config. Run 'cowork config' to set up.")?;

    let model = create_llm_client(&llm_config.llm)
        .context("Failed to create LLM client")?;

    let project_path_str = project_path.to_string_lossy().to_string();

    let artifact_options = format!(
        "generate_idea: {}, generate_prd: {}, generate_design: {}, generate_plan: {}",
        generate_idea, generate_prd, generate_design, generate_plan
    );

    let agent = create_legacy_project_analyzer_with_context(
        model,
        project_path_str.clone(),
        artifact_options.clone(),
    ).context("Failed to create agent")?;

    let prompt = format!(
        r#"Analyze the project at "{}" and generate the requested artifacts.

Project Path: {}
Artifact Options: {}

Start by scanning the project structure, detecting the tech stack, and reading key files.
Then generate and save each artifact using save_artifact tool.
"#,
        project_path_str, project_path_str, artifact_options
    );

    // Create a minimal pipeline context
    let project = cowork_core::domain::Project::new("imported_project");
    let iteration = Iteration::create_genesis(
        &project,
        "Legacy Project Import".to_string(),
        "Initial import from existing project".to_string(),
    );

    let workspace_path = artifacts_dir.parent().unwrap().to_path_buf();

    let ctx = PipelineContext::new(project, iteration, workspace_path);
    let content = Content::new("user").with_text(&prompt);

    let invocation_ctx = Arc::new(SimpleInvocationContext::new(
        &ctx,
        &content,
        agent.clone(),
    ));

    // Execute agent - ADK automatically handles tool calling loop
    let mut stream = agent.run(invocation_ctx).await
        .context("Agent execution failed")?;

    use futures::StreamExt;

    // Monitor the stream - ADK handles all tool execution internally
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = event.content() {
                    for part in &content.parts {
                        if let adk_core::Part::Text { text } = part {
                            if !text.trim().is_empty() {
                                let preview: String = text.chars().take(80).collect();
                                println!("      Processing: {}...", preview);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("      Stream error: {}", e);
            }
        }
    }

    // Check for saved artifacts
    let check_files = ["idea.md", "prd.md", "design.md", "plan.md"];
    let mut saved_artifacts = Vec::new();

    for filename in &check_files {
        let file_path = artifacts_dir.join(filename);
        if file_path.exists() {
            saved_artifacts.push(filename.to_string());
        }
    }

    // Also check root artifacts directory (fallback)
    if let Ok(root_cow_dir) = get_cowork_dir() {
        let root_artifacts_dir = root_cow_dir.join("artifacts");
        for filename in &check_files {
            let root_file = root_artifacts_dir.join(filename);
            if root_file.exists() && !saved_artifacts.contains(&filename.to_string()) {
                let dest_file = artifacts_dir.join(filename);
                if let Err(e) = std::fs::copy(&root_file, &dest_file) {
                    println!("      Warning: Failed to copy {}: {}", filename, e);
                } else {
                    saved_artifacts.push(filename.to_string());
                }
            }
        }
    }

    saved_artifacts.sort();
    saved_artifacts.dedup();

    Ok(saved_artifacts)
}

/// Generate artifacts using template fallback
fn generate_template_artifacts(
    analysis: &cowork_core::importer::ProjectAnalysis,
    artifacts_dir: &PathBuf,
    generate_idea: bool,
    generate_prd: bool,
    generate_design: bool,
    generate_plan: bool,
) -> Result<Vec<String>> {
    let options = ArtifactGenerationOptions {
        generate_idea,
        generate_prd,
        generate_design,
        generate_plan,
        scan_readme: true,
        scan_docs: true,
    };

    let artifacts = generate_artifacts(analysis, &options);
    let mut generated_files = Vec::new();

    for artifact in &artifacts {
        println!("      Generating {} (template)...", artifact.filename);
        let artifact_path = artifacts_dir.join(&artifact.filename);
        std::fs::write(&artifact_path, &artifact.content)
            .with_context(|| format!("Failed to save {}", artifact.filename))?;
        generated_files.push(artifact.filename.clone());
    }

    Ok(generated_files)
}
