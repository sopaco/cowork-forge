// AutonomyState —— 自主模式生命周期状态（设计 §4.2 全集）

use serde::{Deserialize, Serialize};

/// 自主模式状态全集。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyStatus {
    /// 未启用。
    Disabled,
    /// 已启用，空闲待触发。
    Idle,
    /// 调研中。
    Researching,
    /// 提案中。
    Proposing,
    /// 等待用户决策。
    AwaitingDecision,
    /// 执行中。
    Executing,
    /// 已暂停。
    Paused,
    /// 熔断。
    Tripped,
}

impl AutonomyStatus {
    /// 状态名（与序列化值一致，用于错误信息）。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Idle => "idle",
            Self::Researching => "researching",
            Self::Proposing => "proposing",
            Self::AwaitingDecision => "awaiting_decision",
            Self::Executing => "executing",
            Self::Paused => "paused",
            Self::Tripped => "tripped",
        }
    }
}

/// 自主模式运行状态（顶层文档，持久化于 autonomy/state.json）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutonomyState {
    /// 当前状态。
    pub status: AutonomyStatus,
    /// 已完成的自主周期数。
    pub cycle_count: u64,
    /// 最近一次周期时间。
    pub last_cycle_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 熔断原因（status=Tripped 时给出）。
    pub tripped_reason: Option<String>,
}

impl Default for AutonomyState {
    fn default() -> Self {
        Self {
            status: AutonomyStatus::Disabled,
            cycle_count: 0,
            last_cycle_at: None,
            tripped_reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_as_snake_case() {
        let values = [
            (AutonomyStatus::Disabled, "\"disabled\""),
            (AutonomyStatus::Idle, "\"idle\""),
            (AutonomyStatus::Researching, "\"researching\""),
            (AutonomyStatus::Proposing, "\"proposing\""),
            (AutonomyStatus::AwaitingDecision, "\"awaiting_decision\""),
            (AutonomyStatus::Executing, "\"executing\""),
            (AutonomyStatus::Paused, "\"paused\""),
            (AutonomyStatus::Tripped, "\"tripped\""),
        ];
        for (status, expected) in values {
            let json = serde_json::to_string(&status).expect("serialize status");
            assert_eq!(json, expected);
            assert_eq!(status.as_str(), expected.trim_matches('"'));
        }
    }

    #[test]
    fn default_state_is_disabled_with_zero_cycles() {
        let state = AutonomyState::default();
        assert_eq!(state.status, AutonomyStatus::Disabled);
        assert_eq!(state.cycle_count, 0);
        assert!(state.last_cycle_at.is_none());
        assert!(state.tripped_reason.is_none());
    }
}
