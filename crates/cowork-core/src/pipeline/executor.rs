// Iteration Executor - Single entry point for all development cycles

use std::sync::Arc;

use crate::domain::{Iteration, IterationStatus, Project};
use crate::interaction::InteractiveBackend;
use crate::persistence::{IterationStore, ProjectStore};

use super::{get_stages_from, is_critical_stage, PipelineContext, StageResult};

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
        self.project_store.add_iteration(project, iteration.to_summary())?;

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
        self.project_store.add_iteration(project, iteration.to_summary())?;

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
                // Copy all workspace files from base iteration
                println!("[Executor] Full inheritance - copying all files from base iteration: {}", base_iteration_id);
                let base_workspace = self.iteration_store.workspace_path(base_iteration_id)?;
                if base_workspace.exists() {
                    self.copy_dir_all(&base_workspace, workspace).await?;
                    println!("[Executor] Copied workspace from {}", base_workspace.display());
                } else {
                    println!("[Executor] Warning: Base workspace does not exist: {}", base_workspace.display());
                }
                Ok(())
            }
            InheritanceMode::Partial => {
                // Copy only specific artifacts and configuration files
                // Don't copy generated code - will regenerate based on artifacts
                println!("[Executor] Partial inheritance - copying artifacts only from base: {}", base_iteration_id);

                let base_workspace = self.iteration_store.workspace_path(base_iteration_id)?;

                // Copy configuration files and non-code assets
                if base_workspace.exists() {
                    self.copy_non_code_files(&base_workspace, workspace).await?;
                    println!("[Executor] Copied non-code files from base workspace");
                }

                // Copy artifacts from base iteration's artifact directory
                let base_iteration_dir = self.iteration_store.iteration_path(base_iteration_id)?;
                let base_artifacts_dir = base_iteration_dir.join("artifacts");
                let current_artifacts_dir = workspace.parent()
                    .map(|p| p.join("artifacts"))
                    .unwrap_or_else(|| workspace.join("artifacts"));

                if base_artifacts_dir.exists() {
                    std::fs::create_dir_all(&current_artifacts_dir)?;
                    self.copy_dir_all(&base_artifacts_dir, &current_artifacts_dir).await?;
                    println!("[Executor] Copied artifacts from base iteration");
                }

                Ok(())
            }
        }
    }

    /// Copy only non-code files (config, assets, docs)
    async fn copy_non_code_files(
        &self,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> anyhow::Result<()> {
        let code_extensions = [
            "rs", "js", "jsx", "ts", "tsx", "py", "java", "go", "cpp", "c", "h", "hpp",
            "cs", "php", "rb", "swift", "kt", "scala", "r", "m", "mm",
        ];

        if !dst.exists() {
            tokio::fs::create_dir_all(dst).await?;
        }

        let mut entries = tokio::fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dest_path = dst.join(file_name);

            if path.is_dir() {
                // Skip node_modules, target, .git, etc.
                let dir_name = file_name.to_string_lossy();
                if matches!(dir_name.as_ref(), "node_modules" | "target" | ".git" | "dist" | "build" | ".cowork" | ".cowork-v2") {
                    continue;
                }
                Box::pin(self.copy_non_code_files(&path, &dest_path)).await?;
            } else {
                // Check if it's a code file
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                if !code_extensions.contains(&ext) {
                    // Copy non-code files (config, readme, assets, etc.)
                    tokio::fs::copy(&path, &dest_path).await?;
                }
            }
        }

        Ok(())
    }

    /// Execute an iteration
    pub async fn execute(&self, project: &mut Project, iteration_id: &str) -> anyhow::Result<()> {
        // Load iteration
        let mut iteration = self.iteration_store.load(iteration_id)?;

        // Prepare workspace
        let workspace = self.prepare_workspace(&iteration).await?;

        // Create pipeline context
        let ctx = PipelineContext::new(project.clone(), iteration.clone(), workspace);

        // Determine starting stage
        let start_stage = iteration.determine_start_stage();

        // Get stages to execute
        let stages = get_stages_from(&start_stage);

        // Start iteration
        iteration.start();
        self.iteration_store.save(&iteration)?;
        self.project_store.set_current_iteration(project, iteration_id.to_string())?;

        // Emit event
        self.interaction
            .show_message(
                crate::interaction::MessageLevel::Info,
                format!("Starting iteration '{}' from stage '{}'", iteration.title, start_stage),
            )
            .await;

        // Execute stages with retry logic
        const MAX_STAGE_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 2000;

        for stage in stages {
            let stage_name = stage.name().to_string();

            // Update current stage
            iteration.set_stage(&stage_name);
            self.iteration_store.save(&iteration)?;

            // Execute stage with retry
            let mut last_error = None;
            let mut success = false;

            for attempt in 0..MAX_STAGE_RETRIES {
                // Emit stage started event
                if attempt == 0 {
                    self.interaction
                        .show_message(
                            crate::interaction::MessageLevel::Info,
                            format!("Stage: {}", stage.description()),
                        )
                        .await;
                } else {
                    self.interaction
                        .show_message(
                            crate::interaction::MessageLevel::Warning,
                            format!("Stage '{}' retry {}/{}...", stage_name, attempt, MAX_STAGE_RETRIES - 1),
                        )
                        .await;
                    // Wait before retry
                    tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                }

                // Execute stage
                let result = stage.execute(&ctx, self.interaction.clone()).await;

                match result {
                    StageResult::Success(artifact_path) => {
                        // Complete stage
                        iteration.complete_stage(&stage_name, artifact_path);
                        self.iteration_store.save(&iteration)?;
                        success = true;

                        // Show success message on retry
                        if attempt > 0 {
                            self.interaction
                                .show_message(
                                    crate::interaction::MessageLevel::Success,
                                    format!("Stage '{}' succeeded after {} retries", stage_name, attempt),
                                )
                                .await;
                        }

                        // Check if needs human confirmation
                        if is_critical_stage(&stage_name) {
                            iteration.pause();
                            self.iteration_store.save(&iteration)?;

                            // Request confirmation
                            let confirmed = self
                                .interaction
                                .request_confirmation(&format!(
                                    "Stage '{}' completed. Review the output and confirm to continue.",
                                    stage_name
                                ))
                                .await;

                            if !confirmed {
                                return Ok(()); // User cancelled
                            }

                            iteration.resume();
                            self.iteration_store.save(&iteration)?;
                        }
                        break; // Success, exit retry loop
                    }
                    StageResult::Failed(error) => {
                        last_error = Some(error);
                        // Continue to next retry
                    }
                    StageResult::Paused => {
                        iteration.pause();
                        self.iteration_store.save(&iteration)?;

                        self.interaction
                            .show_message(
                                crate::interaction::MessageLevel::Info,
                                format!("Stage '{}' paused. Use 'continue' to resume.", stage_name),
                            )
                        .await;

                        return Ok(());
                    }
                }
            }

            // If all retries failed
            if !success {
                let error = last_error.unwrap_or_else(|| "Unknown error after retries".to_string());
                iteration.fail();
                self.iteration_store.save(&iteration)?;

                self.interaction
                    .show_message(
                        crate::interaction::MessageLevel::Error,
                        format!("Stage '{}' failed after {} attempts: {}", stage_name, MAX_STAGE_RETRIES, error),
                    )
                    .await;

                return Err(anyhow::anyhow!("Iteration failed at stage '{}' after {} retries", stage_name, MAX_STAGE_RETRIES));
            }
        }

        // Complete iteration
        iteration.complete();
        self.iteration_store.save(&iteration)?;

        // Update project
        project.current_iteration_id = Some(iteration_id.to_string());
        self.project_store.save(project)?;

        self.interaction
            .show_message(
                crate::interaction::MessageLevel::Success,
                format!("Iteration '{}' completed successfully!", iteration.title),
            )
            .await;

        Ok(())
    }

    /// Continue a paused iteration
    pub async fn continue_iteration(&self, project: &mut Project, iteration_id: &str) -> anyhow::Result<()> {
        let mut iteration = self.iteration_store.load(iteration_id)?;

        if iteration.status != IterationStatus::Paused {
            return Err(anyhow::anyhow!("Iteration is not paused"));
        }

        iteration.resume();
        self.iteration_store.save(&iteration)?;

        // Resume execution from current stage
        self.execute(project, iteration_id).await
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

#[async_trait::async_trait]
pub trait InteractionExt {
    async fn request_confirmation(&self, prompt: &str) -> bool;
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
}
