use anyhow::Result;
use async_trait::async_trait;

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

/// Agent trait（通用接口）
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent 名称
    fn name(&self) -> &str;

    /// 执行 Agent 逻辑
    async fn execute(&self, context: &AgentContext) -> Result<AgentOutput>;
}

/// Agent 执行上下文
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub session_id: String,
    pub input: String,
    pub prev_artifacts: Vec<String>,
}

/// Agent 输出
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub content: String,
    pub artifact_id: Option<String>,
}
