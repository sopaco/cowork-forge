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
        // Try to get stage mapping from default flow configuration
        let stage_mapping = self.get_stage_mapping_from_flow();
        
        let mode_key = match self.inheritance {
            InheritanceMode::None => "none",
            InheritanceMode::Full => {
                // Full inheritance: use smart analysis to determine scope
                // But respect stage_mapping as the final decision
                let analyzed = analyze_change_scope(&self.description);
                // If stage_mapping has an entry for "full", use it; otherwise use analyzed result
                if stage_mapping.contains_key("full") {
                    "full"
                } else {
                    return analyzed;
                }
            }
            InheritanceMode::Partial => "partial",
        };
        
        // Get stage from mapping, or use smart defaults
        stage_mapping
            .get(mode_key)
            .cloned()
            .unwrap_or_else(|| match self.inheritance {
                InheritanceMode::None => "idea".to_string(),
                InheritanceMode::Full => analyze_change_scope(&self.description),
                InheritanceMode::Partial => "plan".to_string(),
            })
    }
    
    /// Get stage mapping from default flow configuration
    fn get_stage_mapping_from_flow(&self) -> std::collections::HashMap<String, String> {
        use crate::config_definition::registry::global_registry;
        
        if let Some(flow) = global_registry().get_default_flow() {
            flow.config.inheritance.stage_mapping
        } else {
            // Default mapping
            let mut mapping = std::collections::HashMap::new();
            mapping.insert("none".to_string(), "idea".to_string());
            mapping.insert("partial".to_string(), "plan".to_string());
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

/// Change scope for determining start stage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeScope {
    Code,         // Start from plan
    Design,       // Start from design
    Requirement,  // Start from prd
    Architecture, // Start from idea
}

/// Analyze change description to determine scope
fn analyze_change_scope(description: &str) -> String {
    let desc_lower = description.to_lowercase();

    // Keywords indicating architecture changes
    let arch_keywords = ["架构", "architecture", "重构", "rewrite", "重新设计", "redesign"];
    for kw in &arch_keywords {
        if desc_lower.contains(kw) {
            return "idea".to_string();
        }
    }

    // Keywords indicating requirement changes
    let req_keywords = ["需求", "requirement", "功能", "feature", "添加", "add"];
    for kw in &req_keywords {
        if desc_lower.contains(kw) {
            return "prd".to_string();
        }
    }

    // Keywords indicating design changes
    let design_keywords = ["设计", "design", "数据库", "database", "接口", "api"];
    for kw in &design_keywords {
        if desc_lower.contains(kw) {
            return "design".to_string();
        }
    }

    // Default: code changes only
    "plan".to_string()
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
