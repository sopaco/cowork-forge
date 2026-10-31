// 自主循环的周期快照与步骤进度（设计 §6.2 / §6.3 / §9）
//
// 每轮 cycle 落一份证据快照到 `autonomy/cycles/{cycle_id}.json`：
// - 增量调研的依据（下轮只看 diff 与未决 idea）
// - 降级（degraded）、预算消耗、丢弃原因的可追溯记录

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 触发来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleTrigger {
    /// 人工「立即调研」。
    Manual,
    /// 迭代完成/失败后的事件触发。
    IterationCompleted,
    /// 定时触发。
    Schedule,
    /// 空闲触发。
    Idle,
}

impl CycleTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::IterationCompleted => "iteration_completed",
            Self::Schedule => "schedule",
            Self::Idle => "idle",
        }
    }

    /// 该触发源是否在配置中开启。
    pub fn enabled_in(&self, triggers: &super::config::AutonomyTriggers) -> bool {
        match self {
            Self::Manual => true,
            Self::IterationCompleted => triggers.on_iteration_completed,
            Self::Schedule => triggers.on_schedule,
            Self::Idle => triggers.on_idle,
        }
    }
}

/// 本轮结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleOutcome {
    /// 产出了新提案。
    Completed,
    /// 跑完但没有新提案（空转）。
    Empty,
    /// 前置检查未通过，本轮跳过（不报错）。
    Skipped,
    /// 本轮失败（LLM/网络等）。
    Failed,
    /// 触发熔断。
    Tripped,
}

impl CycleOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Empty => "empty",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
            Self::Tripped => "tripped",
        }
    }
}

/// 步骤状态（设计 §11.2：完成打勾 / 进行中 / 跳过置灰，不做假百分比进度条）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Done,
    Skipped,
    Failed,
}

impl StepStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

/// 单个步骤的可读进度。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleStep {
    pub name: String,
    pub status: StepStatus,
    /// 当前动作一行文本（如"正在读取最近迭代的已知问题"）或跳过原因。
    pub detail: Option<String>,
    pub elapsed_ms: Option<u64>,
}

impl CycleStep {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: StepStatus::Pending,
            detail: None,
            elapsed_ms: None,
        }
    }
}

/// 周期快照（落盘 + 推送前端的同源数据）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleSnapshot {
    pub cycle_id: String,
    pub trigger: CycleTrigger,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub outcome: CycleOutcome,
    pub llm_calls: u32,
    pub mcp_calls: u32,
    /// 外部检索降级标记：连接失败/超时/未配置/被拒时置 true，本轮用本地证据继续。
    pub degraded: bool,
    pub degraded_reason: Option<String>,
    /// 实际发布（Proposed）的 idea。
    pub proposed_idea_ids: Vec<String>,
    /// 通过质量门槛但超出每轮发布上限、留在 Draft 池的 idea。
    pub drafted_idea_ids: Vec<String>,
    /// 与既有 idea 合并（同指纹/同证据）的条数。
    pub merged_count: u32,
    /// 被过滤掉的提案及原因（质量门槛/去重/状态抑制）。
    pub discarded: Vec<String>,
    pub steps: Vec<CycleStep>,
    /// 人类可读的一行结论（如"这轮找到 3 条值得考虑的方向，2 条已合并进已有建议"）。
    pub note: Option<String>,
    /// 本轮实际使用的外部 MCP server。
    pub mcp_servers: Vec<String>,
    /// 调研输入的摘要指纹（增量调研依据）。
    pub evidence_digest: String,
    /// 模型本轮的自由文本（截断保存）。
    ///
    /// "没有新发现"必须能复核：没有这段，"模型判断项目无缺口"和
    /// "模型根本没跑完"在磁盘上长得一模一样。
    #[serde(default)]
    pub planner_note: Option<String>,
    /// 本轮真实调用过的工具序列（按调用顺序）。
    ///
    /// 用于解释"为什么是空轮"：没读过文件 vs 读了但没提案 vs 查了外部。
    #[serde(default)]
    pub called_tools: Vec<String>,
    /// 模型是否真的调用了 `propose_ideas`（false ⇒ 本轮不算结论）。
    #[serde(default)]
    pub propose_called: bool,
}

impl CycleSnapshot {
    pub fn new(cycle_id: String, trigger: CycleTrigger, evidence_digest: String) -> Self {
        Self {
            cycle_id,
            trigger,
            started_at: Utc::now(),
            finished_at: None,
            outcome: CycleOutcome::Empty,
            llm_calls: 0,
            mcp_calls: 0,
            degraded: false,
            degraded_reason: None,
            proposed_idea_ids: Vec::new(),
            drafted_idea_ids: Vec::new(),
            merged_count: 0,
            discarded: Vec::new(),
            steps: Vec::new(),
            note: None,
            mcp_servers: Vec::new(),
            evidence_digest,
            planner_note: None,
            called_tools: Vec::new(),
            propose_called: false,
        }
    }

    /// 生成人话结论（设计 §11.7 文案风格，不用 AI 黑话）。
    pub fn summarize(&mut self) {
        let published = self.proposed_idea_ids.len();
        let merged = self.merged_count;
        let discarded = self.discarded.len();
        self.note = Some(match self.outcome {
            CycleOutcome::Completed => {
                if merged > 0 {
                    format!("这轮找到 {} 条值得考虑的方向，{} 条已合并进已有建议。", published, merged)
                } else {
                    format!("这轮找到 {} 条值得考虑的方向。", published)
                }
            }
            CycleOutcome::Empty => {
                if discarded > 0 {
                    format!("这轮没有新发现：{} 条候选与已有建议重复或未达标。", discarded)
                } else if let Some(reason) = self.planner_note.as_deref().and_then(last_line) {
                    // 把模型自己的判断摆出来：否则"没有新发现"是句无从复核的空话
                    format!("这轮没有新发现。模型的判断：{}", reason)
                } else {
                    "这轮没有新发现，项目目前没有明显的缺口。".to_string()
                }
            }
            CycleOutcome::Skipped => "这轮调研被跳过。".to_string(),
            CycleOutcome::Failed => "这轮调研没跑完，已有建议不受影响。".to_string(),
            CycleOutcome::Tripped => "自主循环已熔断，需要人工确认后恢复。".to_string(),
        });
    }
}

/// 取模型输出中最后一条非空行（结论通常在末尾），去掉 markdown 记号并截断。
fn last_line(text: &str) -> Option<String> {
    let raw = text
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.trim().is_empty())?;
    let cleaned = raw
        .trim_start_matches(['#', '-', '*', '>', ' '])
        .trim_end_matches(['*', ' ', '\t'])
        .trim();
    let cleaned = if cleaned.is_empty() { raw } else { cleaned };
    Some(cleaned.chars().take(160).collect())
}

/// 步骤进度推送端口（core 不感知 UI；GUI 实现为 Tauri 事件）。
pub trait CycleProgressSink: Send + Sync {
    fn on_step(&self, cycle_id: &str, step: &CycleStep);
    fn on_snapshot(&self, snapshot: &CycleSnapshot);
}

/// 默认的空实现（测试与无 UI 场景）。
pub struct NoopProgressSink;

impl CycleProgressSink for NoopProgressSink {
    fn on_step(&self, _cycle_id: &str, _step: &CycleStep) {}
    fn on_snapshot(&self, _snapshot: &CycleSnapshot) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_enabled_respects_config() {
        let triggers = super::super::config::AutonomyTriggers {
            on_iteration_completed: true,
            on_idle: false,
            on_schedule: false,
            schedule_interval_mins: 60,
        };
        assert!(CycleTrigger::Manual.enabled_in(&triggers));
        assert!(CycleTrigger::IterationCompleted.enabled_in(&triggers));
        assert!(!CycleTrigger::Schedule.enabled_in(&triggers));
        assert!(!CycleTrigger::Idle.enabled_in(&triggers));
    }

    #[test]
    fn summary_mentions_merge_and_discard_counts() {
        let mut snapshot = CycleSnapshot::new(
            "cycle-1".to_string(),
            CycleTrigger::Manual,
            "digest".to_string(),
        );
        snapshot.outcome = CycleOutcome::Completed;
        snapshot.proposed_idea_ids = vec!["a".into(), "b".into(), "c".into()];
        snapshot.merged_count = 2;
        snapshot.summarize();
        assert_eq!(
            snapshot.note.as_deref(),
            Some("这轮找到 3 条值得考虑的方向，2 条已合并进已有建议。")
        );

        let mut empty = CycleSnapshot::new(
            "cycle-2".to_string(),
            CycleTrigger::Manual,
            "digest".to_string(),
        );
        empty.discarded = vec!["重复".to_string()];
        empty.summarize();
        assert!(empty.note.as_deref().unwrap().contains("没有新发现"));
    }

    #[test]
    fn empty_summary_quotes_the_model_when_it_explained_itself() {
        let mut snapshot = CycleSnapshot::new(
            "cycle-empty".to_string(),
            CycleTrigger::Manual,
            "digest".to_string(),
        );
        snapshot.outcome = CycleOutcome::Empty;
        snapshot.propose_called = true;
        snapshot.planner_note = Some(
            "我看了 src 下的实现。\n\n**结论：8 个培训模块都已经实现，本轮没有值得提的方向。**"
                .to_string(),
        );
        snapshot.summarize();

        let note = snapshot.note.as_deref().expect("note");
        assert!(note.contains("模型的判断"), "{note}");
        // markdown 记号要清掉，最后一行的内容要出来
        assert!(note.contains("8 个培训模块都已经实现"), "{note}");
        assert!(!note.contains("**"), "{note}");
        // 不能退化成那句没有依据的空话
        assert!(!note.contains("项目目前没有明显的缺口"), "{note}");
    }

    #[test]
    fn snapshot_serde_round_trip() {
        let mut snapshot = CycleSnapshot::new(
            "cycle-3".to_string(),
            CycleTrigger::IterationCompleted,
            "digest".to_string(),
        );
        snapshot.degraded = true;
        snapshot.degraded_reason = Some("tavily 连接超时".to_string());
        snapshot.steps.push(CycleStep {
            name: "收集证据".to_string(),
            status: StepStatus::Done,
            detail: Some("读取 memory/project/memory.json".to_string()),
            elapsed_ms: Some(120),
        });
        let json = serde_json::to_string(&snapshot).expect("serialize");
        let back: CycleSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, snapshot);
        assert!(json.contains("\"trigger\":\"iteration_completed\""));
        assert!(json.contains("\"status\":\"done\""));
    }
}
