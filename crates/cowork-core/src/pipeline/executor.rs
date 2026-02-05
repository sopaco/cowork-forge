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
        // Load base iteration
        let base = self.iteration_store.load(base_iteration_id)?;

        // Copy workspace files if Full inheritance
        if base.inheritance == crate::domain::InheritanceMode::Full {
            let base_workspace = self.iteration_store.workspace_path(base_iteration_id)?;
            if base_workspace.exists() {
                self.copy_dir_all(&base_workspace, workspace).await?;
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

        // Execute stages
        for stage in stages {
            let stage_name = stage.name().to_string();

            // Update current stage
            iteration.set_stage(&stage_name);
            self.iteration_store.save(&iteration)?;

            // Emit stage started event
            self.interaction
                .show_message(
                    crate::interaction::MessageLevel::Info,
                    format!("Stage: {}", stage.description()),
                )
                .await;

            // Execute stage
            let result = stage.execute(&ctx, self.interaction.clone()).await;

            match result {
                StageResult::Success(artifact_path) => {
                    // Complete stage
                    iteration.complete_stage(&stage_name, artifact_path);
                    self.iteration_store.save(&iteration)?;

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
                }
                StageResult::Failed(error) => {
                    iteration.fail();
                    self.iteration_store.save(&iteration)?;

                    self.interaction
                        .show_message(
                            crate::interaction::MessageLevel::Error,
                            format!("Stage '{}' failed: {}", stage_name, error),
                        )
                        .await;

                    return Err(anyhow::anyhow!("Iteration failed at stage '{}'", stage_name));
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
