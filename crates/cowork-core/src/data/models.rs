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
// Session Meta (session/meta.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub session_id: String,
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
// Session-scoped Models (for session isolation)
// ============================================================================

/// Project index - tracks all sessions and current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIndex {
    pub schema_version: String,
    pub project_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// The latest successful session (for modify to use as base)
    pub latest_successful_session: Option<String>,
    /// All session records
    pub sessions: Vec<SessionRecord>,
}

/// Record of a single session (new/modify/revert execution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub session_type: SessionType,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    /// For modify sessions: which session is the base
    pub base_session_id: Option<String>,
    /// Input description (idea for new, change request for modify)
    pub input_description: String,
    /// Change request (only for modify sessions)
    pub change_request_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    New,      // Full project creation (new command)
    Modify,   // Incremental change (modify command)
    Revert,   // Revert and rerun (revert command)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    InProgress,
    Completed,
    Failed,
}

/// Change request - describes an incremental modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRequest {
    pub id: String,
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    /// User's idea/description of the change
    pub idea: String,
    /// Which session to use as baseline
    pub base_session_id: String,
    /// Automatically determined scope (which artifacts need update)
    pub scope: ChangeScope,
    /// Acceptance criteria extracted from idea
    pub acceptance_criteria: Vec<String>,
    /// Constraints (e.g., don't break existing features)
    pub constraints: Vec<String>,
    /// Analysis result from triage agent
    pub analysis: Option<ChangeAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeScope {
    pub requires_prd_update: bool,
    pub requires_design_update: bool,
    pub requires_plan_update: bool,
    pub requires_code_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeAnalysis {
    /// Affected components
    pub affected_components: Vec<String>,
    /// Affected features
    pub affected_features: Vec<String>,
    /// Risk assessment
    pub risk_level: RiskLevel,
    /// Estimated effort
    pub estimated_effort: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Patch metadata - tracks what changed in a modify session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchMetadata {
    pub session_id: String,
    pub base_session_id: String,
    pub created_at: DateTime<Utc>,
    /// Files added
    pub added_files: Vec<String>,
    /// Files modified
    pub modified_files: Vec<String>,
    /// Files deleted
    pub deleted_files: Vec<String>,
    /// Artifact updates
    pub artifact_updates: Vec<ArtifactUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactUpdate {
    pub artifact_type: ArtifactType,
    pub change_type: ChangeType,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Requirements,
    Features,
    Design,
    Plan,
    Code,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl ProjectIndex {
    pub fn new(project_name: String) -> Self {
        Self {
            schema_version: "2.0".to_string(),
            project_name,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            latest_successful_session: None,
            sessions: Vec::new(),
        }
    }

    pub fn add_session(&mut self, record: SessionRecord) {
        self.sessions.push(record);
        self.updated_at = Utc::now();
    }

    pub fn update_latest_successful(&mut self, session_id: String) {
        self.latest_successful_session = Some(session_id);
        self.updated_at = Utc::now();
    }
}

impl ChangeRequest {
    pub fn new(
        session_id: String,
        idea: String,
        base_session_id: String,
    ) -> Self {
        Self {
            id: format!("CR-{}", Utc::now().timestamp()),
            session_id,
            created_at: Utc::now(),
            idea,
            base_session_id,
            scope: ChangeScope::default(),
            acceptance_criteria: Vec::new(),
            constraints: Vec::new(),
            analysis: None,
        }
    }
}

impl Default for ChangeScope {
    fn default() -> Self {
        Self {
            requires_prd_update: false,
            requires_design_update: false,
            requires_plan_update: false,
            requires_code_change: true, // Default to code-only change
        }
    }
}

impl PatchMetadata {
    pub fn new(session_id: String, base_session_id: String) -> Self {
        Self {
            session_id,
            base_session_id,
            created_at: Utc::now(),
            added_files: Vec::new(),
            modified_files: Vec::new(),
            deleted_files: Vec::new(),
            artifact_updates: Vec::new(),
        }
    }
}

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
