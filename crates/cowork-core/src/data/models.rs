// Structured data models for Cowork Forge
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Requirements (requirements.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirements {
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,  // REQ-001, REQ-002, etc.
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub category: RequirementCategory,
    pub acceptance_criteria: Vec<String>,
    pub related_features: Vec<String>,  // Feature IDs
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequirementCategory {
    Functional,
    NonFunctional,
}

// ============================================================================
// Feature List (feature_list.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureList {
    pub schema_version: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,  // FEAT-001, FEAT-002, etc.
    pub name: String,
    pub description: String,
    pub requirement_ids: Vec<String>,
    pub status: FeatureStatus,
    pub assigned_to_tasks: Vec<String>,  // Task IDs
    pub completion_criteria: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub metadata: FeatureMetadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

// ============================================================================
// Design Spec (design_spec.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSpec {
    pub schema_version: String,
    pub architecture: Architecture,
    pub technology_stack: TechnologyStack,
    pub deployment: DeploymentInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architecture {
    pub style: String,  // "microservices", "monolith", etc.
    pub components: Vec<DesignComponent>,
    pub data_models: Vec<DataModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignComponent {
    pub id: String,  // COMP-001, COMP-002, etc.
    pub name: String,
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    pub responsibilities: Vec<String>,
    pub technology: String,
    pub interfaces: Vec<ComponentInterface>,
    pub related_features: Vec<String>,  // Feature IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    BackendService,
    FrontendComponent,
    Database,
    ApiGateway,
    MessageQueue,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInterface {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataModel {
    pub name: String,
    pub fields: Vec<DataField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyStack {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub architecture: String,
}

// ============================================================================
// Implementation Plan (implementation_plan.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub schema_version: String,
    pub milestones: Vec<Milestone>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,  // M1, M2, etc.
    pub name: String,
    pub features: Vec<String>,  // Feature IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,  // TASK-001, TASK-002, etc.
    pub title: String,
    pub description: String,
    pub feature_id: String,
    pub component_id: String,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,  // Task IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<String>,
    pub files_to_create: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

// ============================================================================
// Code Metadata (code_metadata.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    pub schema_version: String,
    pub files: Vec<FileMetadata>,
    pub build_status: BuildStatus,
    pub test_status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub lines_of_code: usize,
    pub test_coverage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatus {
    pub last_build: DateTime<Utc>,
    pub success: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStatus {
    pub last_run: DateTime<Utc>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub details: Vec<TestDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDetail {
    pub test_name: String,
    pub status: String,  // "passed" or "failed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ============================================================================
// Session Meta (iteration session/meta.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub session_id: String,  // Agent execution session ID (same as iteration_id in V2)
    pub created_at: DateTime<Utc>,
    pub current_stage: Option<Stage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Idea,
    Prd,
    Design,
    Plan,
    Coding,
    Check,
    Delivery,
}

// ============================================================================
// Feedback (session/feedback.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackHistory {
    pub feedbacks: Vec<Feedback>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub feedback_type: FeedbackType,
    pub severity: Severity,
    pub details: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    BuildError,
    QualityIssue,
    MissingRequirement,
    Suggestion,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Major,
    Minor,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl Requirements {
    pub fn new() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            requirements: Vec::new(),
        }
    }
}

impl FeatureList {
    pub fn new() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            features: Vec::new(),
        }
    }
}

impl DesignSpec {
    pub fn new() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            architecture: Architecture {
                style: String::new(),
                components: Vec::new(),
                data_models: Vec::new(),
            },
            technology_stack: TechnologyStack {
                backend: None,
                frontend: None,
                database: None,
            },
            deployment: DeploymentInfo {
                architecture: String::new(),
            },
        }
    }
}

impl ImplementationPlan {
    pub fn new() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            milestones: Vec::new(),
            tasks: Vec::new(),
        }
    }
}

impl CodeMetadata {
    pub fn new() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            files: Vec::new(),
            build_status: BuildStatus {
                last_build: Utc::now(),
                success: false,
                errors: Vec::new(),
            },
            test_status: TestStatus {
                last_run: Utc::now(),
                total: 0,
                passed: 0,
                failed: 0,
                details: Vec::new(),
            },
        }
    }
}

impl FeedbackHistory {
    pub fn new() -> Self {
        Self {
            feedbacks: Vec::new(),
        }
    }
}