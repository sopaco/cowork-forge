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

/// Create a stage instance by its ID
/// This is used by Flow configuration to dynamically create stages
pub fn create_stage_by_id(stage_id: &str) -> Option<Box<dyn Stage>> {
    match stage_id {
        "idea" => Some(Box::new(stages::IdeaStage)),
        "prd" => Some(Box::new(stages::PrdStage)),
        "design" => Some(Box::new(stages::DesignStage)),
        "plan" => Some(Box::new(stages::PlanStage)),
        "coding" => Some(Box::new(stages::CodingStage)),
        "check" => Some(Box::new(stages::CheckStage)),
        "delivery" => Some(Box::new(stages::DeliveryStage)),
        _ => {
            tracing::warn!("Unknown stage ID: {}, using default stages", stage_id);
            None
        }
    }
}

/// Get stages from Flow configuration
/// Returns stages defined in the flow, starting from the specified stage
pub fn get_stages_from_flow(start_stage: &str) -> Vec<Box<dyn Stage>> {
    use crate::config_definition::registry::global_registry;
    
    // Try to get stages from default flow
    if let Some(flow) = global_registry().get_default_flow() {
        let stage_ids: Vec<String> = flow.stages.iter()
            .map(|s| s.stage_id.clone())
            .collect();
        
        // Find start index
        let start_idx = stage_ids.iter().position(|s| s == start_stage).unwrap_or(0);
        
        // Create stages dynamically
        let mut stages = Vec::new();
        for stage_id in stage_ids.into_iter().skip(start_idx) {
            if let Some(stage) = create_stage_by_id(&stage_id) {
                stages.push(stage);
            }
        }
        
        if !stages.is_empty() {
            return stages;
        }
    }
    
    // Fallback to hardcoded stages
    get_stages_from(start_stage)
}

/// Get Flow configuration for execution
/// Returns the default flow's configuration or a default configuration
pub fn get_flow_config() -> crate::config_definition::flow_definition::FlowConfig {
    use crate::config_definition::registry::global_registry;
    
    global_registry()
        .get_default_flow()
        .map(|f| f.config)
        .unwrap_or_default()
}

/// Determine if a stage needs human confirmation
pub fn is_critical_stage(stage_name: &str) -> bool {
    matches!(stage_name, "idea" | "prd" | "design" | "plan" | "coding")
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_stages_order() {
        let stages = get_all_stages();
        
        assert_eq!(stages.len(), 7);
        assert_eq!(stages[0].name(), "idea");
        assert_eq!(stages[1].name(), "prd");
        assert_eq!(stages[2].name(), "design");
        assert_eq!(stages[3].name(), "plan");
        assert_eq!(stages[4].name(), "coding");
        assert_eq!(stages[5].name(), "check");
        assert_eq!(stages[6].name(), "delivery");
    }

    #[test]
    fn test_get_stages_from_beginning() {
        let stages = get_stages_from("idea");
        assert_eq!(stages.len(), 7);
        assert_eq!(stages[0].name(), "idea");
    }

    #[test]
    fn test_get_stages_from_middle() {
        let stages = get_stages_from("plan");
        assert_eq!(stages.len(), 4);
        assert_eq!(stages[0].name(), "plan");
        assert_eq!(stages[1].name(), "coding");
        assert_eq!(stages[2].name(), "check");
        assert_eq!(stages[3].name(), "delivery");
    }

    #[test]
    fn test_get_stages_from_end() {
        let stages = get_stages_from("delivery");
        assert_eq!(stages.len(), 1);
        assert_eq!(stages[0].name(), "delivery");
    }

    #[test]
    fn test_get_stages_from_unknown() {
        // Unknown stage should default to index 0 (idea)
        let stages = get_stages_from("unknown");
        assert_eq!(stages.len(), 7);
        assert_eq!(stages[0].name(), "idea");
    }

    #[test]
    fn test_create_stage_by_id_valid() {
        assert!(create_stage_by_id("idea").is_some());
        assert!(create_stage_by_id("prd").is_some());
        assert!(create_stage_by_id("design").is_some());
        assert!(create_stage_by_id("plan").is_some());
        assert!(create_stage_by_id("coding").is_some());
        assert!(create_stage_by_id("check").is_some());
        assert!(create_stage_by_id("delivery").is_some());
    }

    #[test]
    fn test_create_stage_by_id_invalid() {
        assert!(create_stage_by_id("unknown").is_none());
        assert!(create_stage_by_id("").is_none());
    }

    #[test]
    fn test_is_critical_stage() {
        // Critical stages
        assert!(is_critical_stage("idea"));
        assert!(is_critical_stage("prd"));
        assert!(is_critical_stage("design"));
        assert!(is_critical_stage("plan"));
        assert!(is_critical_stage("coding"));

        // Non-critical stages
        assert!(!is_critical_stage("check"));
        assert!(!is_critical_stage("delivery"));
        assert!(!is_critical_stage("unknown"));
    }

    #[test]
    fn test_pipeline_context_new() {
        let project = Project::new("test-project");
        let iteration = crate::domain::Iteration::create_genesis(
            &project,
            "Test".to_string(),
            "Test description".to_string(),
        );
        let workspace = std::path::PathBuf::from("/tmp/workspace");

        let ctx = PipelineContext::new(project.clone(), iteration.clone(), workspace.clone());

        assert_eq!(ctx.project.name, "test-project");
        assert_eq!(ctx.iteration.title, "Test");
        assert_eq!(ctx.workspace_path, workspace);
    }

    #[test]
    fn test_stage_names_and_descriptions() {
        let stages = get_all_stages();
        
        for stage in &stages {
            // Every stage should have a non-empty name
            assert!(!stage.name().is_empty());
            // Every stage should have a non-empty description
            assert!(!stage.description().is_empty());
        }
    }
}
