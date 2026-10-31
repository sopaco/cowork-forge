// Autonomy domain —— PM 自主规划环
//
// 领域类型（config/state）、生命周期状态机与预算（controller/budget）、
// 周期快照（cycle）。调研证据收集在 `research`，提示词在
// `instructions/autonomous_planner.rs`，提案工具在 `tools/idea_tools.rs`。

pub mod budget;
pub mod config;
pub mod controller;
pub mod cycle;
pub mod research;
pub mod state;

pub use budget::{BudgetGuard, BudgetStatus, DailyBudget};
pub use config::{
    AutonomyBudget, AutonomyConfig, AutonomyResearchConfig, AutonomyTriggers, ExternalMcpPolicy,
};
pub use controller::{
    AutonomyController, CycleReport, DecisionOutcome, SkipReason, TriggerDecision,
};
pub use cycle::{
    CycleOutcome, CycleProgressSink, CycleSnapshot, CycleStep, CycleTrigger, NoopProgressSink,
    StepStatus,
};
pub use state::{AutonomyState, AutonomyStatus};
