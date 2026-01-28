
pub mod idea_intake;
mod prd_agent;
mod design_agent;
mod plan_agent;
mod code_planner;
mod code_executor;
mod check_agent;
mod feedback_agent;
mod delivery_agent;
pub mod watchdog;
pub mod code_updater;
pub mod error_analyzer;
pub mod batch_context;
pub mod todo_manager;
pub mod command_validator;

// 新增：统一的 Agent 接口和执行器
mod stage_agent;
mod stage_executor;
mod coding_stage_agent;

pub use idea_intake::IdeaIntakeAgent;
pub use prd_agent::PrdAgent;
pub use design_agent::DesignAgent;
pub use plan_agent::PlanAgent;
pub use code_planner::CodePlanner;
pub use code_executor::{CodeExecutor, ExecutionReport, ChangeResult, ChangeStatus};
pub use check_agent::CheckAgent;
pub use feedback_agent::FeedbackAgent;
pub use delivery_agent::DeliveryAgent;
pub use watchdog::WatchDogAgent;
pub use code_updater::CodeUpdater;
pub use error_analyzer::{ErrorAnalyzer, ErrorAnalysis};
pub use batch_context::{BatchContext, FileContext, FileSummaryGenerator};
pub use todo_manager::{TodoListManager, TodoStatusReport};

// 导出新的统一接口
pub use stage_agent::{StageAgent, StageAgentContext, StageAgentResult};
pub use stage_executor::StageExecutor;
pub use coding_stage_agent::CodingStageAgent;
