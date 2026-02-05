use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Project - Root entity representing a software project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: ProjectMetadata,
    pub current_iteration_id: Option<String>,
    pub iterations: Vec<IterationSummary>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: format!("proj-{}", now.timestamp()),
            name: name.into(),
            created_at: now,
            updated_at: now,
            metadata: ProjectMetadata::default(),
            current_iteration_id: None,
            iterations: Vec::new(),
        }
    }

    pub fn add_iteration(&mut self, summary: IterationSummary) {
        self.iterations.push(summary);
        self.updated_at = Utc::now();
    }

    pub fn set_current_iteration(&mut self, iteration_id: String) {
        self.current_iteration_id = Some(iteration_id);
        self.updated_at = Utc::now();
    }

    pub fn get_latest_completed_iteration(&self) -> Option<&IterationSummary> {
        self.iterations
            .iter()
            .filter(|i| i.status == IterationStatus::Completed)
            .max_by_key(|i| i.number)
    }

    pub fn next_iteration_number(&self) -> u32 {
        self.iterations.len() as u32 + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMetadata {
    pub tech_stack: Vec<String>,
    pub project_type: String,
    pub language: String,
}

/// Summary of an iteration for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationSummary {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub status: IterationStatus,
    pub completed_stages: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Iteration status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IterationStatus {
    Draft,
    Running,
    Paused,
    Completed,
    Failed,
}

impl std::fmt::Display for IterationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IterationStatus::Draft => write!(f, "draft"),
            IterationStatus::Running => write!(f, "running"),
            IterationStatus::Paused => write!(f, "paused"),
            IterationStatus::Completed => write!(f, "completed"),
            IterationStatus::Failed => write!(f, "failed"),
        }
    }
}
