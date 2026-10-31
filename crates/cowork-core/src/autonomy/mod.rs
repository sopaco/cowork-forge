// Autonomy domain (Phase 0 skeleton)
//
// 自主模式的领域类型（config/state）与生命周期状态机（controller）。
// 本模块只放接口 + 领域类型，不含任何业务流程；Idea 聚合、提案、调研、
// run_cycle 等业务逻辑在 Phase 1 接入，不进本模块。

pub mod config;
pub mod controller;
pub mod state;

pub use config::{
    AutonomyBudget, AutonomyConfig, AutonomyResearchConfig, AutonomyTriggers, ExternalMcpPolicy,
};
pub use controller::AutonomyController;
pub use state::{AutonomyState, AutonomyStatus};
