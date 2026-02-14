// Unified Iteration Pipeline
// Single entry point for all development cycles

use std::sync::Arc;

use crate::domain::{Iteration, Project};
use crate::interaction::InteractiveBackend;

pub mod executor;
pub mod stages;
pub mod stage_executor;

pub use executor::*;
pub use stages::*;
pub use stage_executor::*;

/// Stage execution result
#[derive(Debug)]
pub enum StageResult {
    Success(Option<String>), // Artifact path
    Failed(String),          // Error message
    Paused,                  // Waiting for human confirmation
    NeedsRevision(String),   // Needs revision with feedback
    GotoStage(String, String), // (target_stage, reason) - Jump to another stage and continue from there
}

/// Pipeline context for stage execution
#[derive(Debug, Clone)]
pub struct PipelineContext {
    pub project: Project,
    pub iteration: Iteration,
    pub workspace_path: std::path::PathBuf,
}

impl PipelineContext {
    pub fn new(project: Project, iteration: Iteration, workspace_path: std::path::PathBuf) -> Self {
        Self {
            project,
            iteration,
            workspace_path,
        }
    }
}

/// Stage trait - all stages implement this
#[async_trait::async_trait]
pub trait Stage: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    /// Check if this stage needs human confirmation after completion
    fn needs_confirmation(&self) -> bool {
        false
    }

    /// Execute the stage
    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult;

    /// Execute the stage with feedback (for revision)
    async fn execute_with_feedback(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
        _feedback: &str,
    ) -> StageResult {
        // Default implementation just calls execute
        // Stages that support revision should override this
        self.execute(ctx, interaction).await
    }
}

/// Get all available stages in order
pub fn get_all_stages() -> Vec<Box<dyn Stage>> {
    vec![
        Box::new(stages::IdeaStage),
        Box::new(stages::PrdStage),
        Box::new(stages::DesignStage),
        Box::new(stages::PlanStage),
        Box::new(stages::CodingStage),
        Box::new(stages::CheckStage),
        Box::new(stages::DeliveryStage),
    ]
}

/// Get stages starting from a specific stage
pub fn get_stages_from(start_stage: &str) -> Vec<Box<dyn Stage>> {
    let all = get_all_stages();
    let start_idx = all.iter().position(|s| s.name() == start_stage).unwrap_or(0);
    all.into_iter().skip(start_idx).collect()
}

/// Determine if a stage needs human confirmation
pub fn is_critical_stage(stage_name: &str) -> bool {
    matches!(stage_name, "idea" | "prd" | "design" | "plan" | "coding")
}
