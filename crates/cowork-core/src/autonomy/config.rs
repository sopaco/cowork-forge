// AutonomyConfig —— 自主模式配置（设计 §4.1 schema 一次定型）
//
// 无 `serde(default)` 垫片：schema 变更必须提升持久化 schema_version 并显式报错，
// 不做任何旧版本兼容。

use serde::{Deserialize, Serialize};

/// 外部 MCP 调研来源策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalMcpPolicy {
    /// 按需使用外部 MCP 调研。
    AsNeeded,
    /// 关闭外部 MCP 调研。
    Off,
}

/// 自主循环触发条件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutonomyTriggers {
    /// 迭代完成后触发。
    pub on_iteration_completed: bool,
    /// 空闲时触发。
    pub on_idle: bool,
    /// 定时触发。
    pub on_schedule: bool,
    /// 定时触发间隔（分钟）。
    pub schedule_interval_mins: u64,
}

/// 自主循环调研来源配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutonomyResearchConfig {
    /// 本地文件调研。
    pub local: bool,
    /// 记忆库调研。
    pub memory: bool,
    /// 反馈调研。
    pub feedback: bool,
    /// 外部 MCP 策略。
    pub external_mcp: ExternalMcpPolicy,
}

/// 自主循环预算限制。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutonomyBudget {
    /// 单个周期内最多 LLM 调用次数。
    pub max_llm_calls_per_cycle: u32,
    /// 每天最多周期数。
    pub max_cycles_per_day: u32,
    /// 单个周期内最多 MCP 调用次数。
    pub max_mcp_calls_per_cycle: u32,
}

/// 自主模式配置（顶层文档，持久化于 autonomy/config.json）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutonomyConfig {
    /// 是否启用自主模式。
    pub enabled: bool,
    /// 触发条件。
    pub triggers: AutonomyTriggers,
    /// 调研来源。
    pub research: AutonomyResearchConfig,
    /// 预算限制。
    pub budget: AutonomyBudget,
    /// 是否自动创建迭代。
    pub auto_create_iteration: bool,
}

impl Default for ExternalMcpPolicy {
    fn default() -> Self {
        Self::AsNeeded
    }
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            triggers: AutonomyTriggers {
                on_iteration_completed: false,
                on_idle: false,
                on_schedule: false,
                schedule_interval_mins: 60,
            },
            research: AutonomyResearchConfig {
                local: true,
                memory: true,
                feedback: true,
                external_mcp: ExternalMcpPolicy::AsNeeded,
            },
            budget: AutonomyBudget {
                max_llm_calls_per_cycle: 10,
                max_cycles_per_day: 20,
                max_mcp_calls_per_cycle: 3,
            },
            auto_create_iteration: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_design_decisions() {
        let config = AutonomyConfig::default();
        assert!(!config.enabled);
        assert!(!config.triggers.on_iteration_completed);
        assert!(!config.triggers.on_idle);
        assert!(!config.triggers.on_schedule);
        assert_eq!(config.triggers.schedule_interval_mins, 60);
        assert!(config.research.local);
        assert!(config.research.memory);
        assert!(config.research.feedback);
        assert_eq!(config.research.external_mcp, ExternalMcpPolicy::AsNeeded);
        assert_eq!(config.budget.max_llm_calls_per_cycle, 10);
        assert_eq!(config.budget.max_cycles_per_day, 20);
        assert_eq!(config.budget.max_mcp_calls_per_cycle, 3);
        assert!(!config.auto_create_iteration);
    }
}
