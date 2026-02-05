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
    }

    pub fn fail(&mut self) {
        self.status = IterationStatus::Failed;
        self.current_stage = None;
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
            "delivery" => self.artifacts.delivery = Some(path),
            _ => {}
        }
    }

    /// Determine the starting stage based on inheritance mode
    pub fn determine_start_stage(&self) -> String {
        match self.inheritance {
            InheritanceMode::None => "idea".to_string(),
            InheritanceMode::Full | InheritanceMode::Partial => {
                // Analyze description to determine scope
                analyze_change_scope(&self.description)
            }
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
    None,     // Genesis only - fresh start
    Full,     // Inherit all code from base iteration
    Partial,  // Inherit only artifact definitions
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
    pub delivery: Option<String>,
}

impl Artifacts {
    pub fn get(&self, stage: &str) -> Option<&String> {
        match stage {
            "idea" => self.idea.as_ref(),
            "prd" => self.prd.as_ref(),
            "design" => self.design.as_ref(),
            "plan" => self.plan.as_ref(),
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
            "delivery" => self.delivery = Some(path),
            _ => {}
        }
    }
}
