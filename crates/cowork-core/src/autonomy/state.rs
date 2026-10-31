// AutonomyState —— 自主模式生命周期状态（设计 §4.2 全集）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::cycle::{CycleStep, CycleTrigger};

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

/// 进行中的调研周期。
///
/// **只用于展示**（"现在卡在哪一步"），不作为重入保护依据——
/// 重入由进程内 `CycleRunGuard` 判定。进程崩溃后这里会残留，
/// 启动时由 `recover_stale_state` 清掉。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveCycle {
    pub cycle_id: String,
    pub trigger: CycleTrigger,
    pub started_at: DateTime<Utc>,
    /// 最后一次进度心跳（每推进一步刷新）。
    pub heartbeat_at: DateTime<Utc>,
    /// 已发生的步骤（含正在跑的那一步）。
    pub steps: Vec<CycleStep>,
}

impl ActiveCycle {
    pub fn new(cycle_id: impl Into<String>, trigger: CycleTrigger) -> Self {
        let now = Utc::now();
        Self {
            cycle_id: cycle_id.into(),
            trigger,
            started_at: now,
            heartbeat_at: now,
            steps: Vec::new(),
        }
    }

    /// 按名字覆盖同一步骤（同一步骤会经历 running → done 两次推送）。
    pub fn upsert_step(&mut self, step: CycleStep) {
        if let Some(existing) = self.steps.iter_mut().find(|s| s.name == step.name) {
            *existing = step;
        } else {
            self.steps.push(step);
        }
        self.heartbeat_at = Utc::now();
    }

    /// 正在跑的步骤名（面板找不到本地进度时用来兜底显示）。
    pub fn running_step(&self) -> Option<&CycleStep> {
        self.steps
            .iter()
            .find(|s| s.status == super::cycle::StepStatus::Running)
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
    /// 连续无新有效 idea 的轮数（熔断条件①：连续 3 轮 → 降频/暂停）。
    pub consecutive_empty_cycles: u32,
    /// 连续创建迭代失败次数（熔断条件④：≥3 次 → 熔断待人工）。
    pub iteration_create_failures: u32,
    /// 进行中的调研（无调研时为 None）。
    ///
    /// `serde(default)`：这是**新增可选字段**，旧 state.json 没有它也能读，
    /// 属于兼容变更，不需要提升 schema_version。
    #[serde(default)]
    pub active_cycle: Option<ActiveCycle>,
}

impl Default for AutonomyState {
    fn default() -> Self {
        Self {
            status: AutonomyStatus::Disabled,
            cycle_count: 0,
            last_cycle_at: None,
            tripped_reason: None,
            consecutive_empty_cycles: 0,
            iteration_create_failures: 0,
            active_cycle: None,
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
        assert_eq!(state.consecutive_empty_cycles, 0);
        assert_eq!(state.iteration_create_failures, 0);
    }
}
