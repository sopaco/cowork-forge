use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{IterationStatus, Project};

/// Iteration - Core entity representing a single development cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iteration {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub description: String,

    // Inheritance
    pub base_iteration_id: Option<String>,
    pub inheritance: InheritanceMode,

    // Execution state
    pub status: IterationStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub current_stage: Option<String>,
    pub completed_stages: Vec<String>,

    // Artifacts
    pub artifacts: Artifacts,
}

impl Iteration {
    pub fn create_genesis(project: &Project, title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: format!("iter-{}-{}", project.next_iteration_number(), now.timestamp()),
            number: project.next_iteration_number(),
            title,
            description,
            base_iteration_id: None,
            inheritance: InheritanceMode::None,
            status: IterationStatus::Draft,
            started_at: now,
            completed_at: None,
            current_stage: None,
            completed_stages: Vec::new(),
            artifacts: Artifacts::default(),
        }
    }

    pub fn create_evolution(
        project: &Project,
        title: String,
        description: String,
        base_iteration_id: String,
        inheritance: InheritanceMode,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("iter-{}-{}", project.next_iteration_number(), now.timestamp()),
            number: project.next_iteration_number(),
            title,
            description,
            base_iteration_id: Some(base_iteration_id),
            inheritance,
            status: IterationStatus::Draft,
            started_at: now,
            completed_at: None,
            current_stage: None,
            completed_stages: Vec::new(),
            artifacts: Artifacts::default(),
        }
    }

    pub fn start(&mut self) {
        self.status = IterationStatus::Running;
        self.started_at = Utc::now();
    }

    pub fn pause(&mut self) {
        self.status = IterationStatus::Paused;
    }

    pub fn resume(&mut self) {
        self.status = IterationStatus::Running;
    }

    pub fn complete(&mut self) {
        self.status = IterationStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.current_stage = None;
        
        // Ensure we capture all stages that were executed
        // This handles cases where the flow configuration changed during execution
        // or stages were skipped due to inheritance mode
        self.finalize_completed_stages();
    }
    
    /// Finalize completed_stages based on the flow configuration
    /// This ensures progress displays correctly even when flow config changes
    fn finalize_completed_stages(&mut self) {
        // Get stages from the default flow configuration
        let flow_stages = self.get_flow_stages();
        
        // For stages that have artifacts, ensure they're in completed_stages
        let artifact_stages = [
            ("idea", &self.artifacts.idea),
            ("prd", &self.artifacts.prd),
            ("design", &self.artifacts.design),
            ("plan", &self.artifacts.plan),
            ("coding", &self.artifacts.coding),
            ("delivery", &self.artifacts.delivery),
        ];
        
        for (stage_name, artifact) in artifact_stages {
            if artifact.is_some() && !self.completed_stages.contains(&stage_name.to_string()) {
                self.completed_stages.push(stage_name.to_string());
            }
        }
        
        // If no stages were recorded but we have a flow, mark all flow stages as complete
        // This handles the case where the iteration was completed but stages weren't tracked
        if self.completed_stages.is_empty() && !flow_stages.is_empty() {
            self.completed_stages = flow_stages;
        }
    }
    
    /// Get the list of stages from the default flow configuration
    fn get_flow_stages(&self) -> Vec<String> {
        use crate::config_definition::registry::global_registry;
        
        if let Some(flow) = global_registry().get_default_flow() {
            flow.stages.iter().map(|s| s.stage_id.clone()).collect()
        } else {
            // Default stages
            vec![
                "idea".to_string(),
                "prd".to_string(),
                "design".to_string(),
                "plan".to_string(),
                "coding".to_string(),
                "check".to_string(),
                "delivery".to_string(),
            ]
        }
    }

    pub fn fail(&mut self) {
        self.status = IterationStatus::Failed;
        // Keep current_stage to know which stage failed for retry
        // Only clear it if explicitly needed
    }

    pub fn set_stage(&mut self, stage: impl Into<String>) {
        self.current_stage = Some(stage.into());
    }

    pub fn complete_stage(&mut self, stage: impl Into<String>, artifact_path: Option<String>) {
        let stage_name = stage.into();
        self.completed_stages.push(stage_name.clone());

        // Update artifact
        let path = artifact_path.unwrap_or_default();
        match stage_name.as_str() {
            "idea" => self.artifacts.idea = Some(path),
            "prd" => self.artifacts.prd = Some(path),
            "design" => self.artifacts.design = Some(path),
            "plan" => self.artifacts.plan = Some(path),
            "coding" => self.artifacts.coding = Some(path), // Track coding workspace path
            "delivery" => self.artifacts.delivery = Some(path),
            _ => {}
        }
    }

    /// Determine the starting stage based on inheritance mode
    /// Uses Flow configuration's stage_mapping if provided, otherwise falls back to defaults
    pub fn determine_start_stage(&self) -> String {
        let stage_mapping = self.get_stage_mapping_from_flow();
        
        let mode_key = match self.inheritance {
            InheritanceMode::None => "none",
            InheritanceMode::Full => "full",
            InheritanceMode::Partial => "partial",
        };
        
        stage_mapping
            .get(mode_key)
            .cloned()
            .unwrap_or_else(|| "idea".to_string())
    }
    
    /// Get stage mapping from default flow configuration
    fn get_stage_mapping_from_flow(&self) -> std::collections::HashMap<String, String> {
        use crate::config_definition::registry::global_registry;
        
        if let Some(flow) = global_registry().get_default_flow() {
            flow.config.inheritance.stage_mapping
        } else {
            // Default mapping when no flow configuration exists
            let mut mapping = std::collections::HashMap::new();
            mapping.insert("none".to_string(), "idea".to_string());
            mapping.insert("partial".to_string(), "idea".to_string());
            mapping.insert("full".to_string(), "idea".to_string());
            mapping
        }
    }

    pub fn to_summary(&self) -> super::IterationSummary {
        super::IterationSummary {
            id: self.id.clone(),
            number: self.number,
            title: self.title.clone(),
            status: self.status,
            completed_stages: self.completed_stages.clone(),
            created_at: self.started_at,
        }
    }
}

/// Inheritance mode for evolution iterations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InheritanceMode {
    None,     // Genesis iteration - fresh start, no inheritance
    Full,     // Full inheritance - copy all code and artifacts from base iteration (for major refactoring)
    Partial,  // Partial inheritance - copy code files only, regenerate artifacts (for incremental development)
}

impl Default for InheritanceMode {
    fn default() -> Self {
        InheritanceMode::Full
    }
}

/// Artifacts produced by an iteration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Artifacts {
    pub idea: Option<String>,
    pub prd: Option<String>,
    pub design: Option<String>,
    pub plan: Option<String>,
    pub coding: Option<String>,  // Workspace path where code is generated
    pub delivery: Option<String>,
}

impl Artifacts {
    pub fn get(&self, stage: &str) -> Option<&String> {
        match stage {
            "idea" => self.idea.as_ref(),
            "prd" => self.prd.as_ref(),
            "design" => self.design.as_ref(),
            "plan" => self.plan.as_ref(),
            "coding" => self.coding.as_ref(),
            "delivery" => self.delivery.as_ref(),
            _ => None,
        }
    }

    pub fn set(&mut self, stage: &str, path: String) {
        match stage {
            "idea" => self.idea = Some(path),
            "prd" => self.prd = Some(path),
            "design" => self.design = Some(path),
            "plan" => self.plan = Some(path),
            "coding" => self.coding = Some(path),
            "delivery" => self.delivery = Some(path),
            _ => {}
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project() -> Project {
        Project::new("test-project")
    }

    #[test]
    fn test_create_genesis_iteration() {
        let project = create_test_project();
        let iteration = Iteration::create_genesis(
            &project,
            "Test Iteration".to_string(),
            "Test description".to_string(),
        );

        assert!(iteration.id.starts_with("iter-1-"));
        assert_eq!(iteration.number, 1);
        assert_eq!(iteration.title, "Test Iteration");
        assert_eq!(iteration.description, "Test description");
        assert!(iteration.base_iteration_id.is_none());
        assert_eq!(iteration.inheritance, InheritanceMode::None);
        assert_eq!(iteration.status, IterationStatus::Draft);
        assert!(iteration.current_stage.is_none());
        assert!(iteration.completed_stages.is_empty());
    }

    #[test]
    fn test_create_evolution_iteration() {
        let project = create_test_project();
        let iteration = Iteration::create_evolution(
            &project,
            "Evolution Iteration".to_string(),
            "Evolution description".to_string(),
            "iter-0-123".to_string(),
            InheritanceMode::Partial,
        );

        assert_eq!(iteration.base_iteration_id, Some("iter-0-123".to_string()));
        assert_eq!(iteration.inheritance, InheritanceMode::Partial);
    }

    #[test]
    fn test_iteration_status_transitions() {
        let project = create_test_project();
        let mut iteration = Iteration::create_genesis(
            &project,
            "Test".to_string(),
            "Test".to_string(),
        );

        // Draft -> Running
        iteration.start();
        assert_eq!(iteration.status, IterationStatus::Running);

        // Running -> Paused
        iteration.pause();
        assert_eq!(iteration.status, IterationStatus::Paused);

        // Paused -> Running
        iteration.resume();
        assert_eq!(iteration.status, IterationStatus::Running);

        // Running -> Completed
        iteration.complete();
        assert_eq!(iteration.status, IterationStatus::Completed);
        assert!(iteration.completed_at.is_some());
        assert!(iteration.current_stage.is_none());
    }

    #[test]
    fn test_iteration_fail() {
        let project = create_test_project();
        let mut iteration = Iteration::create_genesis(
            &project,
            "Test".to_string(),
            "Test".to_string(),
        );

        iteration.start();
        iteration.set_stage("coding");
        iteration.fail();

        assert_eq!(iteration.status, IterationStatus::Failed);
        // current_stage should be preserved for retry
        assert_eq!(iteration.current_stage, Some("coding".to_string()));
    }

    #[test]
    fn test_set_and_complete_stage() {
        let project = create_test_project();
        let mut iteration = Iteration::create_genesis(
            &project,
            "Test".to_string(),
            "Test".to_string(),
        );

        iteration.set_stage("idea");
        assert_eq!(iteration.current_stage, Some("idea".to_string()));

        iteration.complete_stage("idea", Some("/path/to/idea.md".to_string()));
        assert!(iteration.completed_stages.contains(&"idea".to_string()));
        assert_eq!(iteration.artifacts.idea, Some("/path/to/idea.md".to_string()));
    }

    #[test]
    fn test_determine_start_stage_none_mode() {
        let project = create_test_project();
        let iteration = Iteration::create_genesis(
            &project,
            "Test".to_string(),
            "Test".to_string(),
        );

        // None mode should map to "idea" (default)
        let stage = iteration.determine_start_stage();
        assert_eq!(stage, "idea");
    }

    #[test]
    fn test_determine_start_stage_partial_mode() {
        let project = create_test_project();
        let iteration = Iteration::create_evolution(
            &project,
            "Test".to_string(),
            "Test".to_string(),
            "iter-0-123".to_string(),
            InheritanceMode::Partial,
        );

        // Partial mode should map to "idea" (default)
        let stage = iteration.determine_start_stage();
        assert_eq!(stage, "idea");
    }

    #[test]
    fn test_determine_start_stage_full_mode() {
        let project = create_test_project();
        let iteration = Iteration::create_evolution(
            &project,
            "Test".to_string(),
            "Test".to_string(),
            "iter-0-123".to_string(),
            InheritanceMode::Full,
        );

        // Full mode should map to "idea" (default)
        let stage = iteration.determine_start_stage();
        assert_eq!(stage, "idea");
    }

    #[test]
    fn test_artifacts_get_set() {
        let mut artifacts = Artifacts::default();

        // Test set
        artifacts.set("idea", "/path/to/idea.md".to_string());
        artifacts.set("prd", "/path/to/prd.md".to_string());

        // Test get
        assert_eq!(artifacts.get("idea"), Some(&"/path/to/idea.md".to_string()));
        assert_eq!(artifacts.get("prd"), Some(&"/path/to/prd.md".to_string()));
        assert_eq!(artifacts.get("unknown"), None);
    }

    #[test]
    fn test_to_summary() {
        let project = create_test_project();
        let mut iteration = Iteration::create_genesis(
            &project,
            "Test Iteration".to_string(),
            "Test".to_string(),
        );
        iteration.start();
        iteration.complete_stage("idea", None);

        let summary = iteration.to_summary();

        assert_eq!(summary.title, "Test Iteration");
        assert_eq!(summary.status, IterationStatus::Running);
        assert!(summary.completed_stages.contains(&"idea".to_string()));
    }

    #[test]
    fn test_inheritance_mode_default() {
        assert_eq!(InheritanceMode::default(), InheritanceMode::Full);
    }

    #[test]
    fn test_inheritance_mode_serde() {
        // Test serialization
        assert_eq!(serde_json::to_string(&InheritanceMode::None).unwrap(), "\"none\"");
        assert_eq!(serde_json::to_string(&InheritanceMode::Full).unwrap(), "\"full\"");
        assert_eq!(serde_json::to_string(&InheritanceMode::Partial).unwrap(), "\"partial\"");

        // Test deserialization
        let mode: InheritanceMode = serde_json::from_str("\"none\"").unwrap();
        assert_eq!(mode, InheritanceMode::None);
    }
}
