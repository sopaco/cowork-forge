// Iteration Executor - Single entry point for all development cycles

use futures::StreamExt;
use std::sync::Arc;

use crate::domain::{Iteration, IterationStatus, Project};
use crate::interaction::{InteractiveBackend, MessageContext};
use crate::persistence::{IterationStore, ProjectStore};
use adk_core::Content;

use super::{PipelineContext, Stage, StageResult, get_stages_from, is_critical_stage};

// Re-export from stage_executor
use super::stage_executor::{SimpleInvocationContext, extract_text_from_event};

/// Iteration Executor - Manages the complete iteration lifecycle
pub struct IterationExecutor {
    project_store: ProjectStore,
    iteration_store: IterationStore,
    interaction: Arc<dyn InteractiveBackend>,
}

impl IterationExecutor {
    pub fn new(interaction: Arc<dyn InteractiveBackend>) -> Self {
        Self {
            project_store: ProjectStore::new(),
            iteration_store: IterationStore::new(),
            interaction,
        }
    }

    /// Create a new Genesis iteration (first iteration)
    pub fn create_genesis_iteration(
        &self,
        project: &mut Project,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> anyhow::Result<Iteration> {
        let iteration = Iteration::create_genesis(project, title.into(), description.into());

        // Save iteration
        self.iteration_store.save(&iteration)?;

        // Update project
        self.project_store
            .add_iteration(project, iteration.to_summary())?;

        Ok(iteration)
    }

    /// Create a new Evolution iteration (based on previous iteration)
    pub fn create_evolution_iteration(
        &self,
        project: &mut Project,
        title: impl Into<String>,
        description: impl Into<String>,
        base_iteration_id: impl Into<String>,
    ) -> anyhow::Result<Iteration> {
        let iteration = Iteration::create_evolution(
            project,
            title.into(),
            description.into(),
            base_iteration_id.into(),
            crate::domain::InheritanceMode::Full,
        );

        // Save iteration
        self.iteration_store.save(&iteration)?;

        // Update project
        self.project_store
            .add_iteration(project, iteration.to_summary())?;

        Ok(iteration)
    }

    /// Prepare workspace for iteration execution
    async fn prepare_workspace(&self, iteration: &Iteration) -> anyhow::Result<std::path::PathBuf> {
        let workspace = self.iteration_store.ensure_workspace(&iteration.id)?;

        // If evolution, copy base iteration workspace
        if let Some(base_id) = &iteration.base_iteration_id {
            self.inherit_from_base(&workspace, base_id).await?;
        }

        Ok(workspace)
    }

    /// Inherit workspace from base iteration
    async fn inherit_from_base(
        &self,
        workspace: &std::path::PathBuf,
        base_iteration_id: &str,
    ) -> anyhow::Result<()> {
        use crate::domain::InheritanceMode;

        // Load base iteration
        let base = self.iteration_store.load(base_iteration_id)?;

        match base.inheritance {
            InheritanceMode::None => {
                // No inheritance - start fresh
                println!("[Executor] No inheritance (Genesis iteration)");
                Ok(())
            }
            InheritanceMode::Full => {
                // Copy all workspace files and artifacts from base iteration
                println!(
                    "[Executor] Full inheritance - copying all files from base iteration: {}",
                    base_iteration_id
                );

                let base_workspace = self.iteration_store.workspace_path(base_iteration_id)?;
                if base_workspace.exists() {
                    self.copy_dir_all(&base_workspace, workspace).await?;
                    println!(
                        "[Executor] Copied workspace from {}",
                        base_workspace.display()
                    );
                } else {
                    println!(
                        "[Executor] Warning: Base workspace does not exist: {}",
                        base_workspace.display()
                    );
                }

                // Also copy artifacts directory for Full inheritance
                let base_iteration_dir = self.iteration_store.iteration_path(base_iteration_id)?;
                let base_artifacts_dir = base_iteration_dir.join("artifacts");
                let current_artifacts_dir = workspace
                    .parent()
                    .map(|p| p.join("artifacts"))
                    .unwrap_or_else(|| workspace.join("artifacts"));

                if base_artifacts_dir.exists() {
                    std::fs::create_dir_all(&current_artifacts_dir)?;
                    self.copy_dir_all(&base_artifacts_dir, &current_artifacts_dir)
                        .await?;
                    println!("[Executor] Copied artifacts from base iteration");
                }

                Ok(())
            }
            InheritanceMode::Partial => {
                // Copy code files but NOT artifacts (to avoid agent confusion)
                // Agents will regenerate artifacts based on current iteration's description
                println!(
                    "[Executor] Partial inheritance - copying code files only from base: {}",
                    base_iteration_id
                );

                let base_workspace = self.iteration_store.workspace_path(base_iteration_id)?;

                // Copy all code files (excluding artifacts directory)
                if base_workspace.exists() {
                    self.copy_code_files(&base_workspace, workspace).await?;
                    println!("[Executor] Copied code files from base workspace");
                } else {
                    println!(
                        "[Executor] Warning: Base workspace does not exist: {}",
                        base_workspace.display()
                    );
                }

                // Do NOT copy artifacts - let agents regenerate fresh artifacts
                println!(
                    "[Executor] Skipping artifacts copy - agents will regenerate for current iteration"
                );

                Ok(())
            }
        }
    }

    /// Check if expected artifacts exist for a stage
    async fn check_artifact_exists(&self, stage_name: &str, workspace: &std::path::Path) -> bool {
        let iteration_dir = workspace.parent().unwrap_or(workspace);
        let artifacts_dir = iteration_dir.join("artifacts");

        match stage_name {
            "idea" => {
                // Check idea.md
                artifacts_dir.join("idea.md").exists()
            }
            "prd" => {
                // Check prd.md or requirements.json
                artifacts_dir.join("prd.md").exists()
                    || iteration_dir.join("data/requirements.json").exists()
            }
            "design" => {
                // Check design.md or design_spec.json
                artifacts_dir.join("design.md").exists()
                    || iteration_dir.join("data/design_spec.json").exists()
            }
            "plan" => {
                // Check plan.md or implementation_plan.json
                artifacts_dir.join("plan.md").exists()
                    || iteration_dir.join("data/implementation_plan.json").exists()
            }
            "coding" => {
                // Check workspace has code files and perform quality checks
                if !workspace.exists() {
                    return false;
                }

                // Check for any source code files (including web files)
                let code_extensions = [
                    "rs", "js", "jsx", "ts", "tsx", "py", "java", "go", "cpp", "c", "h", "html",
                    "htm", "css", "scss", "json",
                ];
                let mut has_code_files = false;
                let mut has_issues = false;
                let mut issue_messages = Vec::new();

                // Recursively walk through workspace directory to find code files
                if let Ok(entries) = std::fs::read_dir(workspace) {
                    for entry in entries.flatten() {
                        let path = entry.path();

                        // Check if it's a directory, recursively process
                        if path.is_dir() {
                            // Skip hidden directories (except .well-known)
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if name.starts_with('.') && name != ".well-known" {
                                    continue;
                                }
                            }

                            // Recursively check subdirectory
                            if let Ok(sub_entries) = std::fs::read_dir(&path) {
                                for sub_entry in sub_entries.flatten() {
                                    let sub_path = sub_entry.path();

                                    // Check if it's a code file
                                    if let Some(ext) = sub_path.extension().and_then(|e| e.to_str())
                                    {
                                        if code_extensions.contains(&ext) {
                                            has_code_files = true;

                                            // Read file content and check for issues
                                            if let Ok(content) = std::fs::read_to_string(&sub_path)
                                            {
                                                // Check for empty files
                                                if content.trim().is_empty() {
                                                    has_issues = true;
                                                    issue_messages.push(format!(
                                                        "Empty file: {}",
                                                        sub_path.display()
                                                    ));
                                                }

                                                // Check for TODO/FIXME comments
                                                let content_lower = content.to_lowercase();
                                                if content_lower.contains("todo:")
                                                    || content_lower.contains("fixme:")
                                                {
                                                    has_issues = true;
                                                    issue_messages.push(format!(
                                                        "TODO/FIXME found in: {}",
                                                        sub_path.display()
                                                    ));
                                                }

                                                // Check for placeholder code patterns
                                                let placeholder_patterns = [
                                                    "todo: implement",
                                                    "fixme: implement",
                                                    "// implement",
                                                    "/* implement */",
                                                    "// placeholder",
                                                    "<!-- implement -->",
                                                    "// todo",
                                                    "// fixme",
                                                ];
                                                for pattern in &placeholder_patterns {
                                                    if content_lower
                                                        .contains(&pattern.to_lowercase())
                                                    {
                                                        has_issues = true;
                                                        issue_messages.push(format!(
                                                            "Placeholder pattern '{}' found in: {}",
                                                            pattern,
                                                            sub_path.display()
                                                        ));
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Check if it's a code file in the root directory
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                if code_extensions.contains(&ext) {
                                    has_code_files = true;

                                    // Read file content and check for issues
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        // Check for empty files
                                        if content.trim().is_empty() {
                                            has_issues = true;
                                            issue_messages
                                                .push(format!("Empty file: {}", path.display()));
                                        }

                                        // Check for TODO/FIXME comments
                                        let content_lower = content.to_lowercase();
                                        if content_lower.contains("todo:")
                                            || content_lower.contains("fixme:")
                                        {
                                            has_issues = true;
                                            issue_messages.push(format!(
                                                "TODO/FIXME found in: {}",
                                                path.display()
                                            ));
                                        }

                                        // Check for placeholder code patterns
                                        let placeholder_patterns = [
                                            "todo: implement",
                                            "fixme: implement",
                                            "// implement",
                                            "/* implement */",
                                            "// placeholder",
                                            "<!-- implement -->",
                                            "// todo",
                                            "// fixme",
                                        ];
                                        for pattern in &placeholder_patterns {
                                            if content_lower.contains(&pattern.to_lowercase()) {
                                                has_issues = true;
                                                issue_messages.push(format!(
                                                    "Placeholder pattern '{}' found in: {}",
                                                    pattern,
                                                    path.display()
                                                ));
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !has_code_files {
                    eprintln!(
                        "[Coding Quality Check] No code files found in workspace: {}",
                        workspace.display()
                    );
                    return false;
                }

                // Log issues if any found (but don't fail the stage for now)
                if has_issues {
                    eprintln!(
                        "[Coding Quality Check] Found {} code quality issues:",
                        issue_messages.len()
                    );
                    for msg in &issue_messages {
                        eprintln!("  - {}", msg);
                    }
                    // Note: We still return true to allow the pipeline to continue
                    // The Check stage will perform more detailed validation
                }

                true
            }
            "delivery" => {
                // Check delivery_report.md
                artifacts_dir.join("delivery_report.md").exists()
            }
            _ => {
                // Default: assume artifacts exist
                true
            }
        }
    }

    /// Copy only code files from src to dst (for Partial inheritance)
    /// This excludes artifacts directories but includes all code and project files
    async fn copy_code_files(
        &self,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> anyhow::Result<()> {
        if !dst.exists() {
            tokio::fs::create_dir_all(dst).await?;
        }

        let mut entries = tokio::fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dest_path = dst.join(file_name);

            if path.is_dir() {
                // Skip cowork directories and common build directories
                let dir_name = file_name.to_string_lossy();
                if matches!(
                    dir_name.as_ref(),
                    "node_modules"
                        | "target"
                        | ".git"
                        | "dist"
                        | "build"
                        | ".cowork"
                        | ".cowork-v2"
                        | "artifacts"
                ) {
                    continue;
                }
                Box::pin(self.copy_code_files(&path, &dest_path)).await?;
            } else {
                // Copy all files (including code, config, assets, etc.)
                // For Partial inheritance, we copy everything except artifacts directory
                tokio::fs::copy(&path, &dest_path).await?;
            }
        }

        Ok(())
    }

    /// Execute an iteration
    /// If resume_stage is provided, it means we're resuming from a paused state
    pub async fn execute(
        &self,
        project: &mut Project,
        iteration_id: &str,
        resume_stage: Option<String>,
        _model: Option<Arc<dyn adk_core::Llm>>,
    ) -> anyhow::Result<()> {
        // Load iteration
        let mut iteration = self.iteration_store.load(iteration_id)?;

        // Prepare workspace
        let workspace = self.prepare_workspace(&iteration).await?;

        // Create pipeline context (clone workspace for later use in self-healing)
        let _ctx = PipelineContext::new(project.clone(), iteration.clone(), workspace.clone());

        // Determine starting stage
        // If resume_stage is provided (resuming from pause), use it
        // Otherwise if iteration has a current_stage, use it
        // Otherwise determine based on inheritance mode
        let start_stage = if let Some(stage) = resume_stage {
            // Resuming from a paused state - use the provided stage
            stage
        } else if let Some(ref current) = iteration.current_stage {
            // Continue from current stage (for non-resume scenarios)
            current.clone()
        } else {
            // Fresh start - determine based on inheritance
            iteration.determine_start_stage()
        };

        // Get stages to execute
        let stages = get_stages_from(&start_stage);

        // Start iteration
        iteration.start();
        self.iteration_store.save(&iteration)?;
        self.project_store
            .set_current_iteration(project, iteration_id.to_string())?;

        // Ensure iteration memory exists (V2 architecture)
        let memory_store = crate::persistence::MemoryStore::new();
        if let Err(e) = memory_store.ensure_iteration_memory(iteration_id) {
            println!(
                "[Executor] Warning: Failed to create iteration memory: {}",
                e
            );
        }

        println!(
            "[Executor] Iteration '{}' started, will execute {} stages starting from '{}'",
            iteration.title,
            stages.len(),
            start_stage
        );

        // Emit event with Pipeline context
        self.interaction
            .show_message_with_context(
                crate::interaction::MessageLevel::Info,
                format!(
                    "Starting iteration '{}' from stage '{}'",
                    iteration.title, start_stage
                ),
                MessageContext::new("Pipeline Controller"),
            )
            .await; // Evolution iteration: Inject project knowledge from base iteration
        if iteration.base_iteration_id.is_some() {
            if let Err(e) = self.inject_project_knowledge(&iteration).await {
                println!(
                    "[Executor] Warning: Failed to inject project knowledge: {}",
                    e
                );
                // Continue anyway, just warn
            }
        }

        println!("[Executor] Starting stage execution loop...");

        // Execute stages
        self.execute_stages_from(project, &mut iteration, stages, workspace).await
    }

    /// Execute stages starting from a given list
    /// This is used both for initial execution and for goto_stage jumps
    async fn execute_stages_from(
        &self,
        project: &mut Project,
        iteration: &mut Iteration,
        stages: Vec<Box<dyn crate::pipeline::Stage>>,
        workspace: std::path::PathBuf,
    ) -> anyhow::Result<()> {
        // Execute stages with retry logic
        const MAX_STAGE_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 2000;
        let total_stages = stages.len();
        
        // Create pipeline context
        let ctx = PipelineContext::new(project.clone(), iteration.clone(), workspace.clone());

        for (stage_idx, stage) in stages.into_iter().enumerate() {
            let stage_name = stage.name().to_string();
            let stage_num = stage_idx + 1;

            // Update current stage
            iteration.set_stage(&stage_name);
            self.iteration_store.save(&iteration)?;

            println!(
                "[Executor] Stage updated: {} (iteration: {})",
                stage_name, iteration.id
            );

            // Emit stage started event with progress info and agent context
            self.interaction
                .show_message_with_context(
                    crate::interaction::MessageLevel::Info,
                    format!(
                        "🚀 [{}/{}] Starting stage: {}",
                        stage_num,
                        total_stages,
                        stage.description()
                    ),
                    MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                )
                .await;

            // Execute stage with retry
            let mut last_error = None;
            let mut success = false;

            for attempt in 0..MAX_STAGE_RETRIES {
                // Emit retry message if needed
                if attempt > 0 {
                    self.interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            format!(
                                "🔄 Stage '{}' retry {}/{}...",
                                stage_name,
                                attempt,
                                MAX_STAGE_RETRIES - 1
                            ),
                            MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                        )
                        .await;
                    // Wait before retry
                    tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                }

                // Execute stage (with feedback loop for revisions)
                let mut feedback_loop_count = 0;
                const MAX_FEEDBACK_LOOPS: u32 = 5;
                let mut current_feedback: Option<String> = None;

                loop {
                    // Execute or re-execute stage
                    let result = if let Some(ref feedback) = current_feedback {
                        stage
                            .execute_with_feedback(&ctx, self.interaction.clone(), feedback)
                            .await
                    } else {
                        stage.execute(&ctx, self.interaction.clone()).await
                    };

                    match result {
                        StageResult::GotoStage(target_stage, reason) => {
                            // Agent requested to jump to another stage
                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Warning,
                                    format!(
                                        "🔄 Stage jump requested: {} → {}\nReason: {}",
                                        stage_name, target_stage, reason
                                    ),
                                    MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                )
                                .await;

                            // Clear current stage feedback
                            if let Err(e) = crate::storage::clear_stage_feedback(&stage_name) {
                                eprintln!(
                                    "[Warning] Failed to clear feedback for stage '{}': {}",
                                    stage_name, e
                                );
                            }

                            // Update iteration to jump to target stage
                            iteration.set_stage(&target_stage);
                            self.iteration_store.save(&iteration)?;

                            // Get new stages starting from target stage
                            let new_stages = get_stages_from(&target_stage);
                            
                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Info,
                                    format!(
                                        "Restarting pipeline from '{}' stage with {} stages to execute",
                                        target_stage,
                                        new_stages.len()
                                    ),
                                    MessageContext::new("Pipeline Controller"),
                                )
                                .await;

                            // Execute new stages starting from target
                            return Box::pin(self.execute_stages_from(
                                project,
                                iteration,
                                new_stages,
                                workspace.clone(),
                            )).await;
                        }
                        StageResult::Success(artifact_path) => {
                            // Artifacts validation check
                            let artifact_exists = if let Some(ref path) = artifact_path {
                                // Check if artifact file exists
                                std::path::Path::new(path).exists()
                            } else {
                                // No artifact path provided, check expected artifacts
                                self.check_artifact_exists(&stage_name, &workspace).await
                            };

                            if !artifact_exists {
                                last_error = Some(format!(
                                    "Artifacts not generated for stage '{}'",
                                    stage_name
                                ));

                                // Show error message with context
                                self.interaction
                                    .show_message_with_context(
                                        crate::interaction::MessageLevel::Error,
                                        format!("❌ Stage '{}' completed but artifacts not found. Will retry...", stage_name),
                                        MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                    )
                                    .await;

                                // Continue to next retry
                                break; // Exit feedback loop to retry
                            }

                            // Clear feedback for this stage since it completed successfully
                            if let Err(e) = crate::storage::clear_stage_feedback(&stage_name) {
                                eprintln!(
                                    "[Warning] Failed to clear feedback for stage '{}': {}",
                                    stage_name, e
                                );
                            }

                            // Complete stage
                            iteration.complete_stage(&stage_name, artifact_path.clone());
                            self.iteration_store.save(&iteration)?;

                            // Show success message with progress info and agent context
                            let progress_msg = if feedback_loop_count > 0 {
                                format!(
                                    "✅ [{}/{}] Stage '{}' completed (revision {})",
                                    stage_num, total_stages, stage_name, feedback_loop_count
                                )
                            } else if attempt > 0 {
                                format!(
                                    "✅ [{}/{}] Stage '{}' completed (after {} retries)",
                                    stage_num, total_stages, stage_name, attempt
                                )
                            } else {
                                format!(
                                    "✅ [{}/{}] Stage '{}' completed",
                                    stage_num, total_stages, stage_name
                                )
                            };

                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Success,
                                    progress_msg,
                                    MessageContext::new("Pipeline Controller")
                                        .with_stage(&stage_name),
                                )
                                .await;

                            // Check if needs human confirmation and feedback loop
                            if is_critical_stage(&stage_name) {
                                iteration.pause();
                                self.iteration_store.save(&iteration)?;

                                // Determine artifact type for viewing
                                let artifact_type = match stage_name.as_str() {
                                    "idea" => "idea",
                                    "prd" => "requirements",
                                    "design" => "design",
                                    "plan" => "plan",
                                    "coding" => "code",
                                    _ => "artifacts",
                                };

                                // Request confirmation with feedback support
                                let action = self
                                    .interaction
                                    .request_confirmation_with_feedback(&format!(
                                        "Stage '{}' completed. Please review the generated {} document.{}",
                                        stage_name,
                                        stage_name.to_uppercase(),
                                        if feedback_loop_count > 0 {
                                            format!(" (Revision {})", feedback_loop_count)
                                        } else {
                                            String::new()
                                        }
                                    ), artifact_type)
                                    .await;

                                match action {
                                    ConfirmationAction::Continue => {
                                        iteration.resume();
                                        self.iteration_store.save(&iteration)?;
                                        success = true;
                                        break; // Exit feedback loop
                                    }
                                    ConfirmationAction::ViewArtifact => {
                                        // User wants to view artifact, stay in loop
                                        current_feedback = None;
                                        continue;
                                    }
                                    ConfirmationAction::ProvideFeedback(feedback) => {
                                        if feedback_loop_count >= MAX_FEEDBACK_LOOPS {
                                            self.interaction
                                                .show_message_with_context(
                                                    crate::interaction::MessageLevel::Warning,
                                                    format!("Maximum revision attempts ({}) reached. Proceeding...", MAX_FEEDBACK_LOOPS),
                                                    MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                                )
                                                .await;
                                            iteration.resume();
                                            self.iteration_store.save(&iteration)?;
                                            success = true;
                                            break;
                                        }

                                        feedback_loop_count += 1;
                                        current_feedback = Some(feedback);
                                        self.interaction
                                            .show_message_with_context(
                                                crate::interaction::MessageLevel::Info,
                                                format!("Regenerating {} based on your feedback (revision {} of {})...",
                                                    stage_name, feedback_loop_count, MAX_FEEDBACK_LOOPS),
                                                MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                            )
                                            .await;
                                        continue; // Re-execute with feedback
                                    }
                                    ConfirmationAction::Cancel => {
                                        return Ok(()); // User cancelled
                                    }
                                }
                            } else {
                                // Non-critical stage completed, no confirmation needed
                                self.interaction
                                    .show_message_with_context(
                                        crate::interaction::MessageLevel::Success,
                                        format!(
                                            "✅ [{}/{}] Stage '{}' completed (auto-continuing)",
                                            stage_num, total_stages, stage_name
                                        ),
                                        MessageContext::new("Pipeline Controller")
                                            .with_stage(&stage_name),
                                    )
                                    .await;
                                success = true;
                                break; // Success, exit retry loop
                            }
                        }
                        StageResult::Failed(error) => {
                            last_error = Some(error);
                            // Continue to next retry
                            break; // Exit feedback loop to retry
                        }
                        StageResult::Paused => {
                            iteration.pause();
                            self.iteration_store.save(&iteration)?;

                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Info,
                                    format!(
                                        "Stage '{}' paused. Use 'continue' to resume.",
                                        stage_name
                                    ),
                                    MessageContext::new("Pipeline Controller")
                                        .with_stage(&stage_name),
                                )
                                .await;

                            return Ok(());
                        }
                        StageResult::NeedsRevision(feedback) => {
                            // Stage itself is requesting revision
                            if feedback_loop_count >= MAX_FEEDBACK_LOOPS {
                                self.interaction
                                    .show_message_with_context(
                                        crate::interaction::MessageLevel::Warning,
                                        format!("Maximum revision attempts reached. Proceeding..."),
                                        MessageContext::new("Pipeline Controller")
                                            .with_stage(&stage_name),
                                    )
                                    .await;
                                iteration.resume();
                                self.iteration_store.save(&iteration)?;
                                success = true;
                                break;
                            }

                            feedback_loop_count += 1;
                            current_feedback = Some(feedback);
                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Info,
                                    format!(
                                        "Stage '{}' requesting revision ({} of {})...",
                                        stage_name, feedback_loop_count, MAX_FEEDBACK_LOOPS
                                    ),
                                    MessageContext::new("Pipeline Controller")
                                        .with_stage(&stage_name),
                                )
                                .await;
                            continue; // Re-execute with feedback
                        }
                    }
                }

                if success {
                    break; // Exit retry loop
                }
            } // End of for attempt in 0..MAX_STAGE_RETRIES

            // If all retries failed
            if !success {
                let error = last_error.unwrap_or_else(|| "Unknown error after retries".to_string());

                // Special handling for Check stage failure - attempt self-healing
                if stage_name == "check" {
                    self.interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            "Check stage failed. Attempting self-healing by returning to previous stage...".to_string(),
                            MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                        )
                        .await;

                    // Try to go back to coding stage with the error as feedback
                    let feedback = format!(
                        "Validation failed with the following issues:\n\n{}\n\nPlease fix these issues in the code.",
                        error
                    );

                    // Re-execute coding stage with feedback (clone workspace for reuse)
                    let ctx_for_healing =
                        PipelineContext::new(project.clone(), iteration.clone(), workspace.clone());

                    let coding_stage = crate::pipeline::stages::CodingStage;

                    match coding_stage
                        .execute_with_feedback(
                            &ctx_for_healing,
                            self.interaction.clone(),
                            &feedback,
                        )
                        .await
                    {
                        StageResult::Success(artifact_path) => {
                            // Clear feedback for coding stage since it completed successfully
                            if let Err(e) = crate::storage::clear_stage_feedback("coding") {
                                eprintln!(
                                    "[Warning] Failed to clear feedback for stage 'coding': {}",
                                    e
                                );
                            }

                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Success,
                                    "Self-healing successful! Code has been fixed based on validation feedback.".to_string(),
                                    MessageContext::new("Pipeline Controller").with_stage("coding"),
                                )
                                .await;

                            // Save the updated artifact
                            iteration.complete_stage("coding", artifact_path.clone());
                            self.iteration_store.save(&iteration)?;

                            // Retry check stage again
                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Info,
                                    "Re-running validation after self-healing...".to_string(),
                                    MessageContext::new("Pipeline Controller")
                                        .with_stage(&stage_name),
                                )
                                .await;

                            // Retry check (single attempt)
                            match stage
                                .execute(&ctx_for_healing, self.interaction.clone())
                                .await
                            {
                                StageResult::Success(_) => {
                                    // Clear feedback for check stage since it completed successfully
                                    if let Err(e) = crate::storage::clear_stage_feedback("check") {
                                        eprintln!(
                                            "[Warning] Failed to clear feedback for stage 'check': {}",
                                            e
                                        );
                                    }

                                    self.interaction
                                        .show_message_with_context(
                                            crate::interaction::MessageLevel::Success,
                                            "✅ Validation passed after self-healing!".to_string(),
                                            MessageContext::new("Pipeline Controller")
                                                .with_stage(&stage_name),
                                        )
                                        .await;
                                    iteration.complete_stage(&stage_name, None);
                                    self.iteration_store.save(&iteration)?;
                                    success = true;
                                }
                                StageResult::Failed(e) => {
                                    self.interaction
                                        .show_message_with_context(
                                            crate::interaction::MessageLevel::Error,
                                            format!("Self-healing failed: Validation still fails after fix: {}", e),
                                            MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                        )
                                        .await;
                                    // Fall through to failure handling
                                }
                                _ => {
                                    self.interaction
                                        .show_message_with_context(
                                            crate::interaction::MessageLevel::Error,
                                            "Self-healing failed: Unexpected result from validation".to_string(),
                                            MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                        )
                                        .await;
                                    // Fall through to failure handling
                                }
                            }
                        }
                        StageResult::Failed(e) => {
                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Error,
                                    format!("Self-healing failed: Unable to fix code: {}", e),
                                    MessageContext::new("Pipeline Controller").with_stage("coding"),
                                )
                                .await;
                        }
                        _ => {}
                    }
                }

                // If still failed after self-healing attempt
                if !success {
                    iteration.fail();
                    self.iteration_store.save(&iteration)?;

                    self.interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Error,
                            format!(
                                "Stage '{}' failed after {} attempts: {}",
                                stage_name, MAX_STAGE_RETRIES, error
                            ),
                            MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                        )
                        .await;

                    return Err(anyhow::anyhow!(
                        "Iteration failed at stage '{}' after {} retries",
                        stage_name,
                        MAX_STAGE_RETRIES
                    ));
                }
            }
        } // End of for stage in stages

        // Complete iteration
        iteration.complete();
        self.iteration_store.save(&iteration)?;

        // Promote insights to decisions (V2 architecture - memory elevation)
        let memory_store = crate::persistence::MemoryStore::new();
        match memory_store.promote_insights_to_decisions(&iteration.id) {
            Ok(count) => {
                if count > 0 {
                    println!(
                        "[Executor] Promoted {} insights to project decisions",
                        count
                    );
                    self.interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Info,
                            format!("Promoted {} insights to project memory", count),
                            MessageContext::new("Memory System"),
                        )
                        .await;
                }
            }
            Err(e) => {
                println!("[Executor] Warning: Failed to promote insights: {}", e);
            }
        }

        // Update project
        project.current_iteration_id = Some(iteration.id.clone());
        self.project_store.save(project)?;

        self.interaction
            .show_message_with_context(
                crate::interaction::MessageLevel::Success,
                format!("Iteration '{}' completed successfully!", iteration.title),
                MessageContext::new("Pipeline Controller"),
            )
            .await;

        Ok(())
    }

    /// Continue a paused iteration
    pub async fn continue_iteration(
        &self,
        project: &mut Project,
        iteration_id: &str,
        model: Option<Arc<dyn adk_core::Llm>>,
    ) -> anyhow::Result<()> {
        let mut iteration = self.iteration_store.load(iteration_id)?;

        println!(
            "[Executor] Continuing iteration '{}' (status: {:?}, current_stage: {:?})",
            iteration_id, iteration.status, iteration.current_stage
        );

        if iteration.status != IterationStatus::Paused {
            println!(
                "[Executor] Error: Iteration is not paused (current status: {:?})",
                iteration.status
            );
            return Err(anyhow::anyhow!("Iteration is not paused"));
        }

        // Save the stage we're resuming from BEFORE calling resume()
        let resume_stage = iteration.current_stage.clone();
        println!("[Executor] Resuming from stage: {:?}", resume_stage);

        iteration.resume();
        self.iteration_store.save(&iteration)?;

        // Notify that iteration has been resumed and is ready to continue
        self.interaction
            .show_message_with_context(
                crate::interaction::MessageLevel::Info,
                format!(
                    "Iteration '{}' resumed from stage: {}",
                    iteration_id,
                    resume_stage.as_ref().unwrap_or(&"unknown".to_string())
                ),
                MessageContext::new("Pipeline Controller")
                    .with_stage(resume_stage.as_deref().unwrap_or("unknown")),
            )
            .await;

        // Resume execution from the saved stage
        println!("[Executor] Calling execute with resume_stage...");
        self.execute(project, iteration_id, resume_stage, model)
            .await
    }

    /// Retry a failed iteration
    pub async fn retry_iteration(
        &self,
        project: &mut Project,
        iteration_id: &str,
    ) -> anyhow::Result<()> {
        let mut iteration = self.iteration_store.load(iteration_id)?;

        println!(
            "[Executor] Retrying failed iteration '{}' (status: {:?}, current_stage: {:?})",
            iteration_id, iteration.status, iteration.current_stage
        );

        if iteration.status != IterationStatus::Failed {
            println!(
                "[Executor] Error: Iteration is not failed (current status: {:?})",
                iteration.status
            );
            return Err(anyhow::anyhow!("Iteration is not failed"));
        }

        // Determine which stage to retry from
        // For failed iterations, we want to retry from the stage that failed
        // If current_stage is set, use it; otherwise default to "check" for validation failures
        let retry_stage = if let Some(ref current) = iteration.current_stage {
            current.clone()
        } else {
            // If no current_stage, default to "check" since that's where most failures happen
            // This is safer than determine_start_stage() which would go back to "idea"
            println!("[Executor] No current_stage found, defaulting to 'check' for retry");
            "check".to_string()
        };

        println!("[Executor] Retrying from stage: {:?}", retry_stage);

        // Reset iteration status to running
        iteration.start();
        self.iteration_store.save(&iteration)?;
        self.project_store
            .set_current_iteration(project, iteration_id.to_string())?;

        // Execute from the retry stage
        println!("[Executor] Calling execute for retry...");
        self.execute(project, iteration_id, Some(retry_stage), None)
            .await
    }

    /// Copy directory recursively
    async fn copy_dir_all(
        &self,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(dst).await?;

        let mut entries = tokio::fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dest_path = dst.join(file_name);

            if path.is_dir() {
                Box::pin(self.copy_dir_all(&path, &dest_path)).await?;
            } else {
                tokio::fs::copy(&path, &dest_path).await?;
            }
        }

        Ok(())
    }
}

/// Confirmation action for user interaction
#[derive(Debug, Clone)]
pub enum ConfirmationAction {
    Continue,                // User confirmed to continue
    ViewArtifact,            // User wants to view the artifact
    ProvideFeedback(String), // User provided feedback for revision
    Cancel,                  // User cancelled
}

#[async_trait::async_trait]
pub trait InteractionExt {
    async fn request_confirmation(&self, prompt: &str) -> bool;
    async fn request_confirmation_with_artifact(&self, prompt: &str, artifact_type: &str) -> bool;
    async fn request_confirmation_with_feedback(
        &self,
        prompt: &str,
        artifact_type: &str,
    ) -> ConfirmationAction;
}

#[async_trait::async_trait]
impl InteractionExt for dyn InteractiveBackend {
    async fn request_confirmation(&self, prompt: &str) -> bool {
        use crate::interaction::{InputOption, InputResponse};

        let options = vec![
            InputOption {
                id: "yes".to_string(),
                label: "Continue".to_string(),
                description: Some("Proceed to next stage".to_string()),
            },
            InputOption {
                id: "no".to_string(),
                label: "Cancel".to_string(),
                description: Some("Stop the iteration".to_string()),
            },
        ];

        match self.request_input(prompt, options, None).await {
            Ok(InputResponse::Selection(id)) => id == "yes",
            _ => false,
        }
    }

    async fn request_confirmation_with_artifact(&self, prompt: &str, artifact_type: &str) -> bool {
        use crate::interaction::{InputOption, InputResponse};

        let options = vec![
            InputOption {
                id: "yes".to_string(),
                label: "Continue".to_string(),
                description: Some("Confirm and proceed to next stage".to_string()),
            },
            InputOption {
                id: "view_artifact".to_string(),
                label: "View Artifact".to_string(),
                description: Some(format!("Open {} tab to review", artifact_type)),
            },
            InputOption {
                id: "no".to_string(),
                label: "Cancel".to_string(),
                description: Some("Stop the iteration".to_string()),
            },
        ];

        let full_prompt = format!("{}\n[ARTIFACT_TYPE:{}]", prompt, artifact_type);

        match self.request_input(&full_prompt, options, None).await {
            Ok(InputResponse::Selection(id)) => match id.as_str() {
                "yes" => true,
                "view_artifact" => {
                    // Emit event to frontend to view artifact
                    let _ = self
                        .show_message(
                            crate::interaction::MessageLevel::Info,
                            format!("[VIEW_ARTIFACT:{}]", artifact_type),
                        )
                        .await;
                    // Return false to pause, user will need to continue after viewing
                    false
                }
                _ => false,
            },
            _ => false,
        }
    }

    async fn request_confirmation_with_feedback(
        &self,
        prompt: &str,
        artifact_type: &str,
    ) -> ConfirmationAction {
        use crate::interaction::{InputOption, InputResponse};

        let options = vec![
            InputOption {
                id: "yes".to_string(),
                label: "Continue".to_string(),
                description: Some("Confirm and proceed to next stage".to_string()),
            },
            InputOption {
                id: "view_artifact".to_string(),
                label: "View Artifact".to_string(),
                description: Some(format!("Open {} tab to review", artifact_type)),
            },
            InputOption {
                id: "feedback".to_string(),
                label: "Provide Feedback".to_string(),
                description: Some("Enter feedback to regenerate".to_string()),
            },
            InputOption {
                id: "no".to_string(),
                label: "Cancel".to_string(),
                description: Some("Stop the iteration".to_string()),
            },
        ];

        let full_prompt = format!("{}\n[ARTIFACT_TYPE:{}]", prompt, artifact_type);

        match self.request_input(&full_prompt, options, None).await {
            Ok(InputResponse::Selection(id)) => match id.as_str() {
                "yes" => ConfirmationAction::Continue,
                "view_artifact" => ConfirmationAction::ViewArtifact,
                "feedback" => {
                    // Request feedback text
                    let feedback_options = vec![InputOption {
                        id: "submit".to_string(),
                        label: "Submit Feedback".to_string(),
                        description: Some("Submit your feedback".to_string()),
                    }];

                    let feedback_prompt =
                        "Please enter your feedback or suggestions for improvement:";

                    match self
                        .request_input(feedback_prompt, feedback_options, Some(String::new()))
                        .await
                    {
                        Ok(InputResponse::Text(feedback)) => {
                            ConfirmationAction::ProvideFeedback(feedback)
                        }
                        Ok(InputResponse::Selection(_)) => ConfirmationAction::ViewArtifact, // If user selects option, go back
                        _ => ConfirmationAction::Cancel,
                    }
                }
                _ => ConfirmationAction::Cancel,
            },
            Ok(InputResponse::Text(feedback)) => ConfirmationAction::ProvideFeedback(feedback),
            _ => ConfirmationAction::Cancel,
        }
    }
}

// ============================================================================
// Knowledge Generation Methods
// ============================================================================

impl IterationExecutor {
    /// Generate summaries for iteration documents using LLM
    pub async fn generate_document_summaries(
        &self,
        iteration: &Iteration,
        model: Arc<dyn adk_core::Llm>,
    ) -> anyhow::Result<()> {
        println!(
            "[Executor] Generating document summaries for iteration {}...",
            iteration.id
        );

        let iteration_dir = self.iteration_store.iteration_path(&iteration.id)?;
        let artifacts_dir = iteration_dir.join("artifacts");
        let summaries_dir = iteration_dir.join("summaries");

        // Create summaries directory
        std::fs::create_dir_all(&summaries_dir)?;

        // Document types to summarize
        let doc_types = vec!["idea", "prd", "design", "plan"];

        for doc_type in doc_types {
            let doc_path = artifacts_dir.join(format!("{}.md", doc_type));

            if !doc_path.exists() {
                println!("[Executor] Warning: {} not found, skipping", doc_type);
                continue;
            }

            // Read document content
            let content = std::fs::read_to_string(&doc_path)?;

            // Create summary agent
            let summary_agent = crate::agents::create_summary_agent(
                model.clone(),
                iteration.id.clone(),
                iteration.number,
            )?;

            // Create prompt
            let prompt = format!(
                "Document Type: {}\n\nDocument Content:\n\n{}\n\nPlease generate a summary following the format specified in your instructions.",
                doc_type, content
            );

            // Execute agent using adk-rust pattern
            let ctx_content = Content::new("user").with_text(&prompt);
            let dummy_project = crate::domain::Project::new("temp");
            let invocation_ctx = Arc::new(SimpleInvocationContext::new(
                &PipelineContext::new(dummy_project, iteration.clone(), iteration_dir.clone()),
                &ctx_content,
                summary_agent.clone(),
            ));

            let stream = match summary_agent.run(invocation_ctx).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[Executor] Error creating stream for {}: {}", doc_type, e);
                    continue;
                }
            };

            // Process stream and extract text
            let mut generated_text = String::new();
            let mut stream = std::pin::pin!(stream);
            while let Some(result) = stream.next().await {
                if let Ok(event) = result {
                    if let Some(text) = extract_text_from_event(&event) {
                        if !text.trim().is_empty() {
                            generated_text = text;
                        }
                    }
                }
            }

            if generated_text.is_empty() {
                eprintln!("[Executor] No output generated for {}", doc_type);
                continue;
            }

            // Save summary
            let summary = self.extract_summary_from_response(&generated_text);
            let summary_path = summaries_dir.join(format!("{}.md", doc_type));
            std::fs::write(&summary_path, summary)?;

            println!("[Executor] Generated summary for {}", doc_type);
        }

        println!("[Executor] Document summaries generation completed");
        Ok(())
    }

    /// Generate iteration knowledge using LLM
    pub async fn generate_iteration_knowledge(
        &self,
        iteration: &Iteration,
        model: Arc<dyn adk_core::Llm>,
    ) -> anyhow::Result<()> {
        println!(
            "[Executor] Generating iteration knowledge for {}...",
            iteration.id
        );

        // Check if knowledge already exists
        let memory_store = crate::persistence::MemoryStore::new();
        let project_memory = memory_store.load_project_memory()?;

        if project_memory
            .get_iteration_knowledge(&iteration.id)
            .is_some()
        {
            println!(
                "[Executor] Knowledge already exists for iteration {}, skipping",
                iteration.id
            );
            return Ok(());
        }

        println!("[Executor] Creating knowledge generation agent...");

        // Create knowledge generation agent
        let knowledge_agent = crate::agents::create_knowledge_generation_agent(
            model.clone(),
            iteration.id.clone(),
            iteration.number,
            iteration.base_iteration_id.clone(),
        )?;

        println!("[Executor] Setting iteration ID for tool context...");

        // Set iteration ID for tool context
        crate::storage::set_iteration_id(iteration.id.clone());

        // Create prompt
        let prompt = "Please analyze this iteration and generate a comprehensive knowledge snapshot. Use the available tools to load document summaries, examine the codebase structure, and extract meaningful knowledge.";

        println!("[Executor] Creating invocation context...");

        // Execute agent using adk-rust pattern
        let iteration_dir = self.iteration_store.iteration_path(&iteration.id)?;
        let ctx_content = Content::new("user").with_text(prompt);
        let dummy_project = crate::domain::Project::new("temp");
        let invocation_ctx = Arc::new(SimpleInvocationContext::new(
            &PipelineContext::new(dummy_project, iteration.clone(), iteration_dir.clone()),
            &ctx_content,
            knowledge_agent.clone(),
        ));

        println!("[Executor] Running knowledge generation agent...");

        let stream = match knowledge_agent.run(invocation_ctx).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Executor] Failed to create stream: {}", e);
                return Err(anyhow::anyhow!("Failed to create stream: {}", e));
            }
        };

        println!("[Executor] Processing agent stream...");

        // Process stream
        let mut stream = std::pin::pin!(stream);
        let mut step_count = 0;
        while let Some(result) = stream.next().await {
            step_count += 1;
            if step_count % 10 == 0 {
                println!("[Executor] Stream processing step {}...", step_count);
            }
            if let Err(e) = result {
                eprintln!("[Executor] Stream error at step {}: {}", step_count, e);
            }
        }

        println!(
            "[Executor] Stream processing completed after {} steps",
            step_count
        );

        // Verify knowledge was saved
        let project_memory = memory_store.load_project_memory()?;
        if project_memory
            .get_iteration_knowledge(&iteration.id)
            .is_some()
        {
            println!("[Executor] Iteration knowledge generated and saved successfully");
        } else {
            eprintln!(
                "[Executor] Warning: Knowledge generation completed but knowledge not found in project memory"
            );
        }

        Ok(())
    }

    /// Inject project knowledge into iteration memory (for evolution iterations)
    pub async fn inject_project_knowledge(&self, iteration: &Iteration) -> anyhow::Result<()> {
        let base_iteration_id = iteration
            .base_iteration_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Evolution iteration must have base_iteration_id"))?;

        println!(
            "[Executor] Injecting project knowledge from base iteration {}...",
            base_iteration_id
        );

        let memory_store = crate::persistence::MemoryStore::new();

        // Load base iteration knowledge
        let project_memory = memory_store.load_project_memory()?;
        let base_knowledge = project_memory
            .get_iteration_knowledge(base_iteration_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No knowledge found for base iteration {}",
                    base_iteration_id
                )
            })?;

        // Inject into iteration memory
        let mut iter_memory = memory_store.load_iteration_memory(&iteration.id)?;

        iter_memory.add_insight(
            "project_context",
            format!(
                "## Base Iteration Knowledge (#{})\n\n\
                **Iteration ID**: {}\n\n\
                **Tech Stack**: {}\n\n\
                **Project Vision**: {}\n\n\
                **Key Requirements**: {}\n\n\
                **System Design**: {}\n\n\
                **Implementation**: {}\n\n\
                **Key Decisions**: {}",
                base_knowledge.iteration_number,
                base_knowledge.iteration_id,
                base_knowledge.tech_stack.join(", "),
                base_knowledge.idea_summary,
                base_knowledge.prd_summary,
                base_knowledge.design_summary,
                base_knowledge.plan_summary,
                base_knowledge.key_decisions.join("; ")
            ),
        );

        // Mark as critical
        if let Some(last_insight) = iter_memory.insights.last_mut() {
            last_insight.importance = crate::domain::Importance::Critical;
        }

        memory_store.save_iteration_memory(&iter_memory)?;

        println!(
            "[Executor] Project knowledge injected to iteration {}",
            iteration.id
        );
        Ok(())
    }

    /// Regenerate knowledge for a specific iteration (for recovery)
    pub async fn regenerate_iteration_knowledge(
        &self,
        iteration_id: &str,
        model: Arc<dyn adk_core::Llm>,
    ) -> anyhow::Result<()> {
        println!(
            "[Executor] Regenerating knowledge for iteration {}...",
            iteration_id
        );

        let iteration = self.iteration_store.load(iteration_id)?;

        // Check if iteration is completed
        if iteration.status != IterationStatus::Completed {
            return Err(anyhow::anyhow!(
                "Cannot regenerate knowledge for incomplete iteration"
            ));
        }

        // Remove existing knowledge if any
        let memory_store = crate::persistence::MemoryStore::new();
        let mut project_memory = memory_store.load_project_memory()?;
        project_memory.remove_iteration_knowledge(iteration_id);
        memory_store.save_project_memory(&project_memory)?;

        // Generate summaries first
        self.generate_document_summaries(&iteration, model.clone())
            .await?;

        // Then generate knowledge
        self.generate_iteration_knowledge(&iteration, model).await?;

        println!("[Executor] Knowledge regeneration completed");
        Ok(())
    }

    /// Extract summary from agent response
    fn extract_summary_from_response(&self, response: &str) -> String {
        // Simple extraction - return the response as is
        // The Summary Agent should already format it correctly
        response.trim().to_string()
    }
}
