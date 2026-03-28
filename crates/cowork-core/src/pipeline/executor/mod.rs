// Iteration Executor - Single entry point for all development cycles

mod interaction_ext;
mod knowledge;
mod workspace;

use std::sync::Arc;

use crate::domain::{IterationStatus, Project};
use crate::interaction::{InteractiveBackend, MessageContext};
use crate::persistence::{IterationStore, ProjectStore};

use super::{PipelineContext, StageResult, get_stages_from_flow, get_flow_config, is_critical_stage};

pub use interaction_ext::{ConfirmationAction, InteractionExt};

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
    ) -> anyhow::Result<crate::domain::Iteration> {
        let iteration = crate::domain::Iteration::create_genesis(project, title.into(), description.into());

        self.iteration_store.save(&iteration)?;
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
    ) -> anyhow::Result<crate::domain::Iteration> {
        let iteration = crate::domain::Iteration::create_evolution(
            project,
            title.into(),
            description.into(),
            base_iteration_id.into(),
            crate::domain::InheritanceMode::Full,
        );

        self.iteration_store.save(&iteration)?;
        self.project_store
            .add_iteration(project, iteration.to_summary())?;

        Ok(iteration)
    }

    /// Execute an iteration
    pub async fn execute(
        &self,
        project: &mut Project,
        iteration_id: &str,
        resume_stage: Option<String>,
        _model: Option<Arc<dyn adk_core::Llm>>,
    ) -> anyhow::Result<()> {
        let mut iteration = self.iteration_store.load(iteration_id)?;

        // Prepare workspace
        let workspace = workspace::prepare_workspace(
            &self.iteration_store,
            &self.interaction,
            &iteration,
        ).await?;

        // Determine starting stage
        let start_stage = if let Some(stage) = resume_stage {
            stage
        } else if let Some(ref current) = iteration.current_stage {
            current.clone()
        } else {
            iteration.determine_start_stage()
        };

        let stages = get_stages_from_flow(&start_stage);
        let flow_config = get_flow_config();

        println!(
            "[Executor] Using Flow config: stop_on_failure={}, memory_scope={:?}",
            flow_config.stop_on_failure, flow_config.memory_scope
        );

        // Start iteration
        iteration.start();
        self.iteration_store.save(&iteration)?;
        self.project_store
            .set_current_iteration(project, iteration_id.to_string())?;

        // Ensure iteration memory exists
        let memory_store = crate::persistence::MemoryStore::new();
        if let Err(e) = memory_store.ensure_iteration_memory(iteration_id) {
            println!("[Executor] Warning: Failed to create iteration memory: {}", e);
        }

        println!(
            "[Executor] Iteration '{}' started, will execute {} stages starting from '{}'",
            iteration.title,
            stages.len(),
            start_stage
        );

        self.interaction
            .show_message_with_context(
                crate::interaction::MessageLevel::Info,
                format!(
                    "Starting iteration '{}' from stage '{}'",
                    iteration.title, start_stage
                ),
                MessageContext::new("Pipeline Controller"),
            )
            .await;

        // Evolution iteration: Inject project knowledge
        if iteration.base_iteration_id.is_some() {
            if let Err(e) = knowledge::inject_project_knowledge(&self.iteration_store, &iteration).await {
                println!("[Executor] Warning: Failed to inject project knowledge: {}", e);
            }
        }

        println!("[Executor] Starting stage execution loop...");
        self.execute_stages_from(project, &mut iteration, stages, workspace, flow_config).await
    }

    /// Execute stages starting from a given list
    async fn execute_stages_from(
        &self,
        project: &mut Project,
        iteration: &mut crate::domain::Iteration,
        stages: Vec<Box<dyn crate::pipeline::Stage>>,
        workspace: std::path::PathBuf,
        flow_config: crate::config_definition::flow_definition::FlowConfig,
    ) -> anyhow::Result<()> {
        const MAX_STAGE_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 5000;
        const MAX_FEEDBACK_LOOPS: u32 = 5;
        
        let total_stages = stages.len();
        let ctx = PipelineContext::new(project.clone(), iteration.clone(), workspace.clone());
        
        crate::persistence::set_iteration_id(iteration.id.clone());

        for (stage_idx, stage) in stages.into_iter().enumerate() {
            let stage_name = stage.name().to_string();
            let stage_num = stage_idx + 1;

            iteration.set_stage(&stage_name);
            self.iteration_store.save(&iteration)?;

            println!("[Executor] Stage updated: {} (iteration: {})", stage_name, iteration.id);

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

            let mut last_error = None;
            let mut success = false;

            for attempt in 0..MAX_STAGE_RETRIES {
                if attempt > 0 {
                    println!(
                        "[Executor] Retrying stage '{}' (attempt {}/{})",
                        stage_name, attempt + 1, MAX_STAGE_RETRIES
                    );
                    self.interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            format!(
                                "Retrying stage '{}' (attempt {}/{})",
                                stage_name, attempt + 1, MAX_STAGE_RETRIES
                            ),
                            MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                        )
                        .await;
                    tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                }

                // Load stored feedback for this stage
                let mut current_feedback: Option<String> = None;
                let mut feedback_loop_count: u32 = 0;

                if let Ok(feedback_history) = crate::persistence::load_feedback_history() {
                    if let Some(fb) = feedback_history
                        .feedbacks
                        .iter()
                        .filter(|f| f.stage == stage_name)
                        .max_by_key(|f| f.timestamp)
                    {
                        tracing::info!("[Executor] Found stored feedback for stage '{}': {}", 
                            stage_name, fb.details.chars().take(100).collect::<String>());
                        current_feedback = Some(fb.details.clone());
                    }
                }

                loop {
                    let result = if let Some(ref feedback) = current_feedback {
                        stage
                            .execute_with_feedback(&ctx, self.interaction.clone(), feedback)
                            .await
                    } else {
                        stage.execute(&ctx, self.interaction.clone()).await
                    };

                    match result {
                        StageResult::GotoStage(target_stage, reason) => {
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

                            if let Err(e) = crate::persistence::clear_stage_feedback(&stage_name) {
                                eprintln!("[Warning] Failed to clear feedback for stage '{}': {}", stage_name, e);
                            }

                            iteration.set_stage(&target_stage);
                            self.iteration_store.save(&iteration)?;

                            let new_stages = get_stages_from_flow(&target_stage);
                            
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

                            return Box::pin(self.execute_stages_from(
                                project,
                                iteration,
                                new_stages,
                                workspace.clone(),
                                flow_config.clone(),
                            )).await;
                        }
                        StageResult::Success(artifact_path) => {
                            let artifact_exists = if let Some(ref path) = artifact_path {
                                std::path::Path::new(path).exists()
                            } else {
                                workspace::check_artifact_exists(&stage_name, &workspace).await
                            };

                            if !artifact_exists {
                                last_error = Some(format!("Artifacts not generated for stage '{}'", stage_name));

                                self.interaction
                                    .show_message_with_context(
                                        crate::interaction::MessageLevel::Error,
                                        format!("❌ Stage '{}' completed but artifacts not found. Will retry...", stage_name),
                                        MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                    )
                                    .await;
                                break;
                            }

                            if let Err(e) = crate::persistence::clear_stage_feedback(&stage_name) {
                                eprintln!("[Warning] Failed to clear feedback for stage '{}': {}", stage_name, e);
                            }

                            iteration.complete_stage(&stage_name, artifact_path.clone());
                            self.iteration_store.save(&iteration)?;

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
                                format!("✅ [{}/{}] Stage '{}' completed", stage_num, total_stages, stage_name)
                            };

                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Success,
                                    progress_msg,
                                    MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                )
                                .await;

                            if is_critical_stage(&stage_name) {
                                iteration.pause();
                                self.iteration_store.save(&iteration)?;

                                let artifact_type = match stage_name.as_str() {
                                    "idea" => "idea",
                                    "prd" => "requirements",
                                    "design" => "design",
                                    "plan" => "plan",
                                    "coding" => "code",
                                    _ => "artifacts",
                                };

                                let action = self.interaction
                                    .request_confirmation_with_feedback(
                                        &format!(
                                            "Stage '{}' completed. Please review the generated {} document.{}",
                                            stage_name,
                                            stage_name.to_uppercase(),
                                            if feedback_loop_count > 0 {
                                                format!(" (Revision {})", feedback_loop_count)
                                            } else {
                                                String::new()
                                            }
                                        ), 
                                        artifact_type
                                    )
                                    .await;

                                match action {
                                    ConfirmationAction::Continue => {
                                        iteration.resume();
                                        self.iteration_store.save(&iteration)?;
                                        success = true;
                                        break;
                                    }
                                    ConfirmationAction::ViewArtifact => {
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
                                                format!("Revising stage '{}' based on feedback...", stage_name),
                                                MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                            )
                                            .await;
                                        continue;
                                    }
                                    ConfirmationAction::Cancel => {
                                        iteration.pause();
                                        self.iteration_store.save(&iteration)?;
                                        return Err(anyhow::anyhow!("User cancelled at stage '{}'", stage_name));
                                    }
                                }
                            } else {
                                success = true;
                                break;
                            }
                        }
                        StageResult::Failed(e) => {
                            last_error = Some(e.clone());
                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Error,
                                    format!("❌ Stage '{}' failed: {}", stage_name, e),
                                    MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                )
                                .await;
                            break;
                        }
                        StageResult::Paused => {
                            iteration.pause();
                            self.iteration_store.save(&iteration)?;
                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Info,
                                    format!("⏸️ Stage '{}' paused by user", stage_name),
                                    MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                )
                                .await;
                            return Ok(());
                        }
                        StageResult::NeedsRevision(e) => {
                            last_error = Some(e.clone());
                            self.interaction
                                .show_message_with_context(
                                    crate::interaction::MessageLevel::Warning,
                                    format!("🔄 Stage '{}' needs revision: {}", stage_name, e),
                                    MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                                )
                                .await;
                            break;
                        }
                    }
                }

                if success {
                    break;
                }
            }

            if !success {
                if flow_config.stop_on_failure {
                    iteration.fail();
                    self.iteration_store.save(&iteration)?;

                    return Err(anyhow::anyhow!(
                        "Stage '{}' failed after {} retries: {}",
                        stage_name,
                        MAX_STAGE_RETRIES,
                        last_error.unwrap_or_else(|| "Unknown error".to_string())
                    ));
                } else {
                    self.interaction
                        .show_message_with_context(
                            crate::interaction::MessageLevel::Warning,
                            format!("Skipping failed stage '{}' and continuing...", stage_name),
                            MessageContext::new("Pipeline Controller").with_stage(&stage_name),
                        )
                        .await;
                }
            }
        }

        // Complete iteration
        iteration.complete();
        self.iteration_store.save(&iteration)?;

        // Promote iteration insights to project decisions
        if let Err(e) = crate::persistence::MemoryStore::new().promote_insights_to_decisions(&iteration.id) {
            println!("[Executor] Warning: Failed to promote insights: {}", e);
        }

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
            return Err(anyhow::anyhow!("Iteration is not paused"));
        }

        let resume_stage = iteration.current_stage.clone();
        println!("[Executor] Resuming from stage: {:?}", resume_stage);

        iteration.resume();
        self.iteration_store.save(&iteration)?;

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

        self.execute(project, iteration_id, resume_stage, model).await
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
            return Err(anyhow::anyhow!("Iteration is not failed"));
        }

        let retry_stage = if let Some(ref current) = iteration.current_stage {
            current.clone()
        } else {
            println!("[Executor] No current_stage found, defaulting to 'check' for retry");
            "check".to_string()
        };

        iteration.resume();
        self.iteration_store.save(&iteration)?;

        self.interaction
            .show_message_with_context(
                crate::interaction::MessageLevel::Info,
                format!("Retrying iteration '{}' from stage: {}", iteration_id, retry_stage),
                MessageContext::new("Pipeline Controller").with_stage(&retry_stage),
            )
            .await;

        self.execute(project, iteration_id, Some(retry_stage), None).await
    }

    // ========================================================================
    // Knowledge Generation Methods (delegated to knowledge module)
    // ========================================================================

    /// Generate summaries for iteration documents using LLM
    pub async fn generate_document_summaries(
        &self,
        iteration: &crate::domain::Iteration,
        model: Arc<dyn adk_core::Llm>,
    ) -> anyhow::Result<()> {
        knowledge::generate_document_summaries(&self.iteration_store, iteration, model).await
    }

    /// Generate iteration knowledge using LLM
    pub async fn generate_iteration_knowledge(
        &self,
        iteration: &crate::domain::Iteration,
        model: Arc<dyn adk_core::Llm>,
    ) -> anyhow::Result<()> {
        knowledge::generate_iteration_knowledge(&self.iteration_store, iteration, model).await
    }

    /// Inject project knowledge into iteration memory (for evolution iterations)
    pub async fn inject_project_knowledge(&self, iteration: &crate::domain::Iteration) -> anyhow::Result<()> {
        knowledge::inject_project_knowledge(&self.iteration_store, iteration).await
    }

    /// Regenerate knowledge for a specific iteration (for recovery)
    pub async fn regenerate_iteration_knowledge(
        &self,
        iteration_id: &str,
        model: Arc<dyn adk_core::Llm>,
    ) -> anyhow::Result<()> {
        knowledge::regenerate_iteration_knowledge(&self.iteration_store, iteration_id, model).await
    }
}
