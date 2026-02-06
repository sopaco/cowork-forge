// GUI-specific data types for enhanced functionality
use serde::{Deserialize, Serialize};
use cowork_core::data::{Requirements, FeatureList, DesignSpec, ImplementationPlan};

// ============================================================================
// Session Artifacts
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifacts {
    pub session_id: String,
    pub idea: Option<String>,
    pub requirements: Option<Requirements>,
    pub features: Option<FeatureList>,
    pub design: Option<DesignSpec>,
    pub plan: Option<ImplementationPlan>,
    // Raw markdown content for display
    pub design_raw: Option<String>,
    pub plan_raw: Option<String>,
    pub code_files: Vec<FileInfo>,
    pub delivery_report: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub language: Option<String>,
    pub modified_at: Option<String>,
}

// ============================================================================
// Preview
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewInfo {
    pub url: String,
    pub port: u16,
    pub status: PreviewStatus,
    pub project_type: ProjectType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviewStatus {
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Html,
    React,
    Vue,
    Angular,
    Static,
    Unknown,
}

// ============================================================================
// Project Runner
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunInfo {
    pub status: RunStatus,
    pub process_id: Option<u32>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Running,
    Stopped,
    Failed(String),
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub status: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

// ============================================================================
// File Tree
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileTreeNode>>,
    pub is_expanded: bool,
    pub language: Option<String>,
}

// ============================================================================
// File Operations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadResult {
    pub content: String,
    pub offset: u64,
    pub total_size: u64,
    pub is_partial: bool,
}

// ============================================================================
// Code Formatting
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResult {
    pub formatted_files: Vec<String>,
    pub errors: Vec<String>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterAvailability {
    pub prettier: bool,
    pub rustfmt: bool,
}

// ============================================================================
// Project Templates
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub technology_stack: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_built_in: bool,
    pub files: Vec<TemplateFile>,
    pub config: TemplateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub is_template: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub variables: Vec<TemplateVariable>,
    pub post_creation_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub default_value: String,
    pub required: bool,
}

// ============================================================================
// Project Detection
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub project_type: ProjectType,
    pub start_command: Option<String>,
    pub build_command: Option<String>,
    pub preview_command: Option<String>,
    pub has_index_html: bool,
    pub has_package_json: bool,
    pub has_cargo_toml: bool,
}