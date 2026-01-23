use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Artifact metadata envelope (所有 json 共享)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEnvelope<T> {
    pub meta: ArtifactMeta,
    pub summary: Vec<String>,
    pub links: ArtifactLinks,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMeta {
    pub session_id: String,
    pub artifact_id: String,
    pub stage: Stage,
    pub v: u32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ts: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactLinks {
    pub prev: Vec<String>,
}

/// Stage 枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    IdeaIntake,
    Requirements,
    Design,
    Plan,
    Coding,
    Check,
    Feedback,
    Delivery,
}

impl Stage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stage::IdeaIntake => "idea_intake",
            Stage::Requirements => "requirements",
            Stage::Design => "design",
            Stage::Plan => "plan",
            Stage::Coding => "coding",
            Stage::Check => "check",
            Stage::Feedback => "feedback",
            Stage::Delivery => "delivery",
        }
    }

    pub fn all() -> &'static [Stage] {
        &[
            Stage::IdeaIntake,
            Stage::Requirements,
            Stage::Design,
            Stage::Plan,
            Stage::Coding,
            Stage::Check,
            Stage::Feedback,
            Stage::Delivery,
        ]
    }
}

/// IDEA Intake → IdeaSpec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeaSpec {
    pub bg: String,
    pub g: Vec<String>,
    pub ng: Vec<String>,
    pub c: Vec<String>,
    pub sc: Vec<String>,
    pub r: Vec<String>,
    pub q: Vec<String>,
}

/// Requirements → PRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRD {
    pub scope: Scope,
    pub reqs: Vec<Requirement>,
    pub cons: Vec<Constraint>,
    pub hitl: Vec<HitlQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub g: Vec<String>,
    pub ng: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub pri: Priority,
    #[serde(rename = "type")]
    pub req_type: RequirementType,
    pub desc: String,
    pub deps: Vec<String>,
    pub ac: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    P0,
    P1,
    P2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequirementType {
    Func,
    Nfr,
    Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlQuestion {
    pub id: String,
    pub q: String,
    pub opts: Vec<String>,
    pub def: String,
}

/// Design → DesignDoc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignDoc {
    pub cli: CliDesign,
    pub wf: Workflow,
    pub arch: Architecture,
    pub io: IoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliDesign {
    pub modes: Vec<String>,
    pub hitl_flow: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub stages: Vec<String>,
    pub transitions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architecture {
    pub layers: Vec<String>,
    pub comps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoConfig {
    pub artifact_dir: String,
    pub formats: Vec<String>,
}

/// Plan → Plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub c4: C4Design,
    pub tasks: Vec<Task>,
    pub milestones: Vec<Milestone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub todo_list: Option<TodoList>,  // 新增：任务分解列表
}

/// TodoList（任务分解）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,                      // "TASK-001"
    pub description: String,             // "实现用户登录功能"
    pub status: TodoStatus,
    pub related_requirements: Vec<String>,  // ["REQ-001", "REQ-002"]
    pub related_files: Vec<String>,         // ["src/auth/login.rs"]
    pub verification_method: String,        // "unit_test" | "manual_test" | "code_review"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Blocked { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C4Design {
    pub context: Vec<String>,
    pub containers: Vec<String>,
    pub components: Vec<String>,
    pub code: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub pri: Priority,
    pub desc: String,
    pub deps: Vec<String>,
    pub out: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub desc: String,
    pub done_when: Vec<String>,
}

/// Coding → CodeChange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub target: TargetProject,
    pub project: ProjectStructure,
    pub changes: Vec<Change>,
    pub cmds: Vec<Command>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_mapping: Vec<RequirementMapping>,  // 新增：需求映射
}

/// 需求到文件的映射关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementMapping {
    pub req_id: String,
    pub files: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetProject {
    pub lang: String,
    pub stack: Vec<String>,
    pub build: Vec<String>,
    pub test: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub root: String,
    pub layout: Layout,
    pub modules: Vec<Module>,
    pub tooling: Tooling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Mono,
    Single,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub module_type: ModuleType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleType {
    Service,
    Lib,
    App,
    Pkg,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tooling {
    pub pkg: String,
    pub build: Vec<String>,
    pub test: Vec<String>,
    pub lint: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub path: String,
    pub kind: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub cmd: String,
    pub expect: String,
    pub phase: Phase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Check,
    Build,
    Test,
    Lint,
    Run,
}

/// Check → CheckReport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    pub checks: Vec<CheckResult>,
    pub ac_results: Vec<AcceptanceResult>,
    pub issues: Vec<Issue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub todo_completion: Option<TodoCompletion>,        // 新增：TodoList 完成度
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirement_coverage: Option<RequirementCoverage>,  // 新增：需求覆盖度
}

/// TodoList 完成度统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoCompletion {
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
    pub blocked: usize,
}

/// 需求覆盖度统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementCoverage {
    pub total_requirements: usize,
    pub verified: usize,
    pub partially_verified: usize,
    pub not_verified: usize,
    pub failed: usize,
    pub coverage_percentage: f64,
}

/// 需求检查清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementChecklist {
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub req_id: String,                  // "REQ-001"
    pub description: String,             // "支持诗歌语义化展示"
    pub implemented_in: Vec<String>,     // ["poem.html"]
    pub verification_status: VerificationStatus,
    pub evidence: Vec<String>,           // ["Found <article> tags", "Semantic HTML structure"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    NotVerified,
    Verified,
    PartiallyVerified,
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub cmd: String,
    pub status: String,
    pub out_ref: String,
    pub notes: Vec<String>,
    pub phase: Phase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceResult {
    pub req_id: String,
    pub ac: String,
    pub status: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub sev: String,
    pub desc: String,
    pub fix_hint: String,
}

/// Feedback → Feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub delta: Vec<Delta>,
    pub rerun: Vec<Rerun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub target_stage: Stage,
    pub change: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rerun {
    pub stage: Stage,
    pub reason: String,
}

/// Delivery → DeliveryReport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReport {
    pub cap: Vec<String>,
    pub howto: Vec<String>,
    pub limits: Vec<String>,
    pub acceptance: Vec<String>,
}

/// Type aliases for convenience
pub type IdeaSpecArtifact = ArtifactEnvelope<IdeaSpec>;
pub type PRDArtifact = ArtifactEnvelope<PRD>;
pub type DesignDocArtifact = ArtifactEnvelope<DesignDoc>;
pub type PlanArtifact = ArtifactEnvelope<Plan>;
pub type CodeChangeArtifact = ArtifactEnvelope<CodeChange>;
pub type CheckReportArtifact = ArtifactEnvelope<CheckReport>;
pub type FeedbackArtifact = ArtifactEnvelope<Feedback>;
pub type DeliveryReportArtifact = ArtifactEnvelope<DeliveryReport>;

impl<T> ArtifactEnvelope<T>
where
    T: Serialize,
{
    pub fn new(session_id: String, stage: Stage, data: T) -> Self {
        Self {
            meta: ArtifactMeta {
                session_id: session_id.clone(),
                artifact_id: Uuid::new_v4().to_string(),
                stage,
                v: 1,
                ts: Utc::now(),
            },
            summary: Vec::new(),
            links: ArtifactLinks { prev: Vec::new() },
            data,
        }
    }

    pub fn with_summary(mut self, summary: Vec<String>) -> Self {
        self.summary = summary;
        self
    }

    pub fn with_prev(mut self, prev: Vec<String>) -> Self {
        self.links.prev = prev;
        self
    }
}
