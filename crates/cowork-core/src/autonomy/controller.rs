// AutonomyController —— PM 自主规划环的业务控制器
//
// 职责（设计 §6.2）：
//   1. 前置检查（开关/熔断/执行互斥/每日预算）→ 决定本轮是否开跑
//   2. 收集调研证据（Rust 侧确定性）→ 组装 prompt
//   3. 调用规划 Agent 产出结构化提案
//   4. 质量门槛 → 去重 → 状态抑制 → 限量 → 落库发布
//   5. 更新状态、熔断计数，落一份 cycle 快照
//
// 决策落库只有这一份实现：GUI 命令层与 Agent 工具都调用本模块的方法，
// 禁止各自另写一份状态迁移。

use std::sync::Arc;
use std::time::Duration;

use adk_core::Llm;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::idea::{
    compute_fingerprint, DecisionAction, Idea, IdeaDecision, IdeaDraft, IdeaRewrite, IdeaSource,
    IdeaStatus, IdeaSummary,
};
use crate::domain::{InheritanceMode, Iteration, IterationStatus};
use crate::persistence::idea_store::DecisionLogEntry;
use crate::persistence::{AutonomyStore, IdeaStore, IterationStore, ProjectStore};

use super::budget::{BudgetGuard, BudgetStatus, DailyBudget};
use super::config::{AutonomyConfig, ExternalMcpPolicy};
use super::cycle::{
    CycleOutcome, CycleProgressSink, CycleSnapshot, CycleStep, CycleTrigger, StepStatus,
};
use super::guard::{CycleRunGuard, is_cycle_running};
use super::research::collect_research;
use super::state::{ActiveCycle, AutonomyState, AutonomyStatus};

/// 连续空转多少轮后熔断（设计 §8-6 熔断条件①）。
pub const MAX_CONSECUTIVE_EMPTY_CYCLES: u32 = 3;
/// 单轮内重复提案达到多少条熔断（设计 §8-6 熔断条件②）。
pub const MAX_DUPLICATES_PER_CYCLE: usize = 3;
/// 连续创建迭代失败多少次熔断（设计 §8-6 熔断条件④）。
pub const MAX_ITERATION_CREATE_FAILURES: u32 = 3;

/// 单轮调研的总时长上限。
///
/// 单个 LLM 请求已由 `TimeoutLlm` 兜底（600s / 600s），这一层是**整轮**的
/// 最后一道闸：无论卡在哪一步（多个慢请求叠加、MCP 工具串行、上游反复重试），
/// 时间到就收敛成一条 failed 快照并回 Idle，绝不留下永不返回的将来时。
///
/// 取 30 分钟：单请求上限已是 10 分钟，而规划 Agent 一轮里可能发起多次调用
/// （`max_iterations = max_llm_calls_per_cycle`）。若整轮闸门设得和单请求同一量级，
/// 慢网关下会出现"明明没挂死却被整轮超时砍断"，反而掩盖真实原因。
pub const CYCLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// 单个工具调用的超时（adk 默认 5 分钟且会重试，对自主调研太长）。
const CYCLE_TOOL_TIMEOUT: Duration = Duration::from_secs(90);

/// 跳过一轮的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// 自主模式未启用。
    Disabled,
    /// 已暂停。
    Paused,
    /// 已熔断，需人工确认后恢复。
    Tripped,
    /// 该触发源在配置中关闭。
    TriggerDisabled,
    /// 迭代执行中，自主循环让路。
    ExecutionInProgress,
    /// 上一轮调研还在跑，不重入。
    AlreadyRunning,
    /// 当日周期预算用尽。
    BudgetExhausted,
    /// 距上次运行未到间隔。
    NotDue,
}

impl SkipReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Paused => "paused",
            Self::Tripped => "tripped",
            Self::TriggerDisabled => "trigger_disabled",
            Self::ExecutionInProgress => "execution_in_progress",
            Self::AlreadyRunning => "already_running",
            Self::BudgetExhausted => "budget_exhausted",
            Self::NotDue => "not_due",
        }
    }

    /// 人话说明（界面直接展示）。
    pub fn message(&self) -> &'static str {
        match self {
            Self::Disabled => "自主模式没有开启。",
            Self::Paused => "自主循环已暂停。",
            Self::Tripped => "自主循环已熔断，需要你确认后恢复。",
            Self::TriggerDisabled => "这个触发方式没有开启。",
            Self::ExecutionInProgress => "迭代正在执行，自主调研先让路。",
            Self::AlreadyRunning => "上一轮调研还在跑。",
            Self::BudgetExhausted => "今天的自主调研次数用完了，明天自动恢复。",
            Self::NotDue => "还没到下次调研时间。",
        }
    }
}

/// 是否该跑一轮的判定结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDecision {
    pub should_run: bool,
    pub skip_reason: Option<SkipReason>,
    pub detail: Option<String>,
}

impl TriggerDecision {
    fn run() -> Self {
        Self {
            should_run: true,
            skip_reason: None,
            detail: None,
        }
    }

    fn skip(reason: SkipReason) -> Self {
        Self {
            should_run: false,
            skip_reason: Some(reason),
            detail: Some(reason.message().to_string()),
        }
    }
}

/// 一轮自主循环的产出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReport {
    pub snapshot: CycleSnapshot,
    /// 本轮发布的建议。
    pub published: Vec<IdeaSummary>,
    /// 被跳过时的原因。
    pub skipped: Option<SkipReason>,
}

/// 决策结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    pub idea: IdeaSummary,
    /// 采纳时创建（并回填 `source_idea_id`）的迭代。
    pub iteration: Option<Iteration>,
    /// 审计条目 id（可被撤销）。
    pub audit_entry_id: String,
    /// 是否建议自动开始执行（仅低风险范围 + 配置开启时为 true）。
    pub auto_start_recommended: bool,
}

/// 自主模式生命周期控制器。
pub struct AutonomyController {
    store: AutonomyStore,
    idea_store: IdeaStore,
    config: AutonomyConfig,
    state: AutonomyState,
}

impl AutonomyController {
    /// 读取现有 config/state 构造控制器。
    pub fn new() -> anyhow::Result<Self> {
        let store = AutonomyStore::new();
        let config = store.load_config()?;
        let state = store.load_state()?;
        Ok(Self {
            store,
            idea_store: IdeaStore::new(),
            config,
            state,
        })
    }

    // ------------------------------------------------------------------
    // 生命周期
    // ------------------------------------------------------------------

    /// 启用自主模式：任意状态 → Idle，config.enabled=true 并落盘。
    pub fn enable(&mut self) -> anyhow::Result<()> {
        self.state.status = AutonomyStatus::Idle;
        self.config.enabled = true;
        self.persist()?;
        Ok(())
    }

    /// 暂停自主循环：仅 Idle → Paused。
    pub fn pause(&mut self) -> anyhow::Result<()> {
        if self.state.status != AutonomyStatus::Idle {
            anyhow::bail!(
                "cannot pause: autonomy is not idle (current: {})",
                self.state.status.as_str()
            );
        }
        self.state.status = AutonomyStatus::Paused;
        self.persist()?;
        Ok(())
    }

    /// 恢复自主循环：仅 Paused → Idle。
    pub fn resume(&mut self) -> anyhow::Result<()> {
        if self.state.status != AutonomyStatus::Paused {
            anyhow::bail!(
                "cannot resume: autonomy is not paused (current: {})",
                self.state.status.as_str()
            );
        }
        self.state.status = AutonomyStatus::Idle;
        self.persist()?;
        Ok(())
    }

    /// 停止自主模式：任意状态 → Disabled，config.enabled=false 并落盘。
    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.state.status = AutonomyStatus::Disabled;
        self.config.enabled = false;
        self.persist()?;
        Ok(())
    }

    /// 熔断后的人工恢复：Tripped → Idle，并清零熔断计数。
    pub fn reset_trip(&mut self) -> anyhow::Result<()> {
        if self.state.status != AutonomyStatus::Tripped {
            anyhow::bail!(
                "cannot reset: autonomy is not tripped (current: {})",
                self.state.status.as_str()
            );
        }
        self.state.status = AutonomyStatus::Idle;
        self.state.tripped_reason = None;
        self.state.consecutive_empty_cycles = 0;
        self.state.iteration_create_failures = 0;
        self.persist()?;
        Ok(())
    }

    /// 异常退出后残留的 Researching / Proposing 状态复位（进程启动时调用一次）。
    ///
    /// 调研中途崩溃或被强杀时，`state.json` 会停在"正在调研"，
    /// 这里把这类"僵尸状态"复位成 Idle。启动时刻进程内必然没有在跑的周期，
    /// 所以不需要额外的存活判定。
    pub fn recover_stale_state(&mut self) -> anyhow::Result<bool> {
        if matches!(
            self.state.status,
            AutonomyStatus::Researching | AutonomyStatus::Proposing
        ) {
            tracing::warn!(
                "[autonomy] 检测到上次异常退出遗留的 {:?} 状态，已复位为 Idle",
                self.state.status.as_str()
            );
            self.state.status = AutonomyStatus::Idle;
            self.state.active_cycle = None;
            self.persist()?;
            return Ok(true);
        }
        Ok(false)
    }

    /// 运行期自愈：状态说"在跑"，但进程里没有任何一轮在跑 → 孤立状态，复位。
    ///
    /// 判据是进程内的 [`is_cycle_running`] 而不是磁盘上的状态：
    /// 磁盘状态只能证明"曾经开始过"，不能证明"现在还活着"。
    /// 只在**没有**持有者时才复位，因此不会误伤真正在跑的周期。
    pub fn heal_if_orphaned(&mut self) -> anyhow::Result<bool> {
        if !matches!(
            self.state.status,
            AutonomyStatus::Researching | AutonomyStatus::Proposing
        ) {
            return Ok(false);
        }
        if is_cycle_running() {
            return Ok(false);
        }
        tracing::warn!(
            "[autonomy] 状态为 {:?} 但进程内没有在跑的调研，判定为孤立状态并复位为 Idle",
            self.state.status.as_str()
        );
        self.state.status = AutonomyStatus::Idle;
        self.state.active_cycle = None;
        self.persist()?;
        Ok(true)
    }

    /// 当前状态快照。
    pub fn status(&self) -> anyhow::Result<AutonomyState> {
        Ok(self.state.clone())
    }

    /// 当前配置快照。
    pub fn config(&self) -> anyhow::Result<AutonomyConfig> {
        Ok(self.config.clone())
    }

    /// 更新配置（自主模式的唯一配置源：`autonomy/config.json`）。
    pub fn update_config(&mut self, config: AutonomyConfig) -> anyhow::Result<()> {
        self.config = config;
        self.persist()?;
        Ok(())
    }

    /// 预算总览（面板展示）。
    pub fn budget_status(&self) -> anyhow::Result<BudgetStatus> {
        let daily = self.store.load_daily_budget()?;
        Ok(BudgetStatus {
            llm_used: 0,
            llm_limit: self.config.budget.max_llm_calls_per_cycle,
            mcp_used: 0,
            mcp_limit: self.config.budget.max_mcp_calls_per_cycle,
            daily_cycles_used: daily.cycles_used,
            daily_cycle_limit: self.config.budget.max_cycles_per_day,
            day: daily.day,
        })
    }

    // ------------------------------------------------------------------
    // 建议（Idea）读取
    // ------------------------------------------------------------------

    pub fn list_ideas(&self) -> anyhow::Result<Vec<Idea>> {
        self.idea_store.load_all()
    }

    /// 列表视图：按更新时间倒序。
    pub fn list_summaries(&self) -> anyhow::Result<Vec<IdeaSummary>> {
        let mut ideas = self.list_ideas()?;
        ideas.sort_by_key(|idea| std::cmp::Reverse(idea.updated_at));
        Ok(ideas.iter().map(|i| i.to_summary()).collect())
    }

    pub fn get_idea(&self, idea_id: &str) -> anyhow::Result<Idea> {
        self.idea_store.load(idea_id)
    }

    /// 某条建议的决策时间线（含撤销记录）。
    pub fn decision_timeline(&self, idea_id: &str) -> anyhow::Result<Vec<DecisionLogEntry>> {
        self.idea_store.decisions_for(idea_id)
    }

    /// 最近的周期快照列表。
    pub fn list_cycles(&self) -> anyhow::Result<Vec<CycleSnapshot>> {
        self.store.load_cycles()
    }

    // ------------------------------------------------------------------
    // 建议创建与决策
    // ------------------------------------------------------------------

    /// 用户主动输入的建议（source=UserInput）。
    ///
    /// 与 Agent 提案走**同一套**质量门槛：不达标的存为 Draft（不发布），
    /// 达标的直接发布为 Proposed。
    pub fn propose_user_idea(&mut self, draft: IdeaDraft) -> anyhow::Result<Idea> {
        let id = self.idea_store.next_idea_id("idea")?;
        let mut idea = draft.into_idea(id, IdeaSource::UserInput);
        idea.status = if idea.quality_issues().is_empty() {
            IdeaStatus::Proposed
        } else {
            IdeaStatus::Draft
        };
        self.idea_store.save(&idea)?;
        // 用户主动输入的建议也算"待决策"
        self.sync_decision_status()?;
        Ok(idea)
    }

    /// 决策门：四类决策（采纳/驳回/改写/置后）。
    ///
    /// 采纳会**同时**创建迭代并回填 `source_idea_id`（设计 §7.4）。
    pub fn decide_idea(
        &mut self,
        idea_id: &str,
        action: DecisionAction,
        comment: &str,
        rewrite: Option<IdeaRewrite>,
        deferred_until: Option<DateTime<Utc>>,
    ) -> anyhow::Result<DecisionOutcome> {
        let mut idea = self.idea_store.load(idea_id)?;
        if !idea.status.is_open() {
            anyhow::bail!(
                "该建议已经决策过了（当前状态：{}），要改主意请用撤销。",
                idea.status.as_str()
            );
        }

        let mut iteration: Option<Iteration> = None;
        let mut auto_start_recommended = false;

        match action {
            DecisionAction::Accept => {
                if let Some(rewrite) = rewrite.clone() {
                    idea.apply_rewrite(rewrite.clone());
                }
                let created = self.create_iteration_from_idea(&idea)?;
                idea.derived_iteration_id = Some(created.id.clone());
                idea.status = IdeaStatus::Converted;
                idea.decision = Some(IdeaDecision {
                    action,
                    by: "user".to_string(),
                    comment: comment.to_string(),
                    rewritten: rewrite,
                    decided_at: Utc::now(),
                    deferred_until: None,
                });
                auto_start_recommended = self.config.auto_create_iteration && idea.scope.is_low_risk();
                iteration = Some(created);
            }
            DecisionAction::Reject => {
                idea.status = IdeaStatus::Rejected;
                idea.decision = Some(IdeaDecision {
                    action,
                    by: "user".to_string(),
                    comment: comment.to_string(),
                    rewritten: rewrite,
                    decided_at: Utc::now(),
                    deferred_until: None,
                });
            }
            DecisionAction::Rewrite => {
                let rewrite =
                    rewrite.ok_or_else(|| anyhow::anyhow!("改写需要提供新的标题/问题描述/假设"))?;
                idea.apply_rewrite(rewrite.clone());
                idea.status = IdeaStatus::Rewritten;
                idea.decision = Some(IdeaDecision {
                    action,
                    by: "user".to_string(),
                    comment: comment.to_string(),
                    rewritten: Some(rewrite),
                    decided_at: Utc::now(),
                    deferred_until: None,
                });
            }
            DecisionAction::Defer => {
                idea.status = IdeaStatus::Deferred;
                idea.decision = Some(IdeaDecision {
                    action,
                    by: "user".to_string(),
                    comment: comment.to_string(),
                    rewritten: rewrite,
                    decided_at: Utc::now(),
                    deferred_until: deferred_until.or_else(|| Some(Utc::now())),
                });
            }
            DecisionAction::Supersede => {
                anyhow::bail!("取代不是人工决策动作，请用 supersede_idea");
            }
        }

        idea.updated_at = Utc::now();
        self.idea_store.save(&idea)?;

        // 决策后同步状态机：还有待决策建议就保持 AwaitingDecision，否则回 Idle
        self.sync_decision_status()?;

        let entry = DecisionLogEntry::new(
            idea_id,
            action,
            "user",
            comment,
            iteration.as_ref().map(|i| i.id.clone()),
        );
        let audit_entry_id = entry.entry_id.clone();
        self.idea_store.append_decision(&entry)?;

        Ok(DecisionOutcome {
            idea: idea.to_summary(),
            iteration,
            audit_entry_id,
            auto_start_recommended,
        })
    }

    /// 决策后同步状态机：还有待决策建议就保持 AwaitingDecision，否则回 Idle。
    fn sync_decision_status(&mut self) -> anyhow::Result<()> {
        if !matches!(
            self.state.status,
            AutonomyStatus::AwaitingDecision | AutonomyStatus::Idle
        ) {
            // 调研中/暂停/熔断等状态不参与这一步
            return Ok(());
        }
        let has_open = self
            .idea_store
            .load_all()?
            .iter()
            .any(|idea| idea.status.is_open());
        self.state.status = if has_open {
            AutonomyStatus::AwaitingDecision
        } else {
            AutonomyStatus::Idle
        };
        self.persist()
    }

    /// 撤销最近一次决策：追加一条反转记录（不改写历史），建议回到未决。
    pub fn undo_decision(&mut self, idea_id: &str) -> anyhow::Result<Idea> {
        let timeline = self.idea_store.decisions_for(idea_id)?;
        let original = timeline
            .iter()
            .rev()
            .find(|e| !e.is_reversal())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("这条建议还没有决策记录"))?;

        let mut idea = self.idea_store.load(idea_id)?;
        idea.status = IdeaStatus::Proposed;
        idea.decision = None;
        idea.derived_iteration_id = None;
        idea.updated_at = Utc::now();
        self.idea_store.save(&idea)?;

        let reversal = DecisionLogEntry::reversal_of(&original, "user", "撤销上一次决策");
        self.idea_store.append_decision(&reversal)?;
        // 建议回到未决 → 状态机也回到 AwaitingDecision
        self.sync_decision_status()?;
        Ok(idea)
    }

    /// 标记某条建议被另一条取代。
    pub fn supersede_idea(
        &mut self,
        idea_id: &str,
        superseded_by: &str,
        comment: &str,
    ) -> anyhow::Result<()> {
        let mut idea = self.idea_store.load(idea_id)?;
        if !self.idea_store.exists(superseded_by) {
            anyhow::bail!("取代者不存在: {}", superseded_by);
        }
        idea.status = IdeaStatus::Superseded;
        idea.superseded_by = Some(superseded_by.to_string());
        idea.updated_at = Utc::now();
        self.idea_store.save(&idea)?;

        let entry = DecisionLogEntry::new(
            idea_id,
            DecisionAction::Supersede,
            "agent",
            comment,
            None,
        );
        self.idea_store.append_decision(&entry)?;
        Ok(())
    }

    /// 由建议创建迭代：复用现有 7 阶段流水线，不新增 stage。
    ///
    /// 继承模式**显式**指定为 `Partial`（只复制代码、重新生成文档），
    /// 不依赖 `Default`（domain 与 config_definition 两份枚举默认值不同）。
    pub fn create_iteration_from_idea(&mut self, idea: &Idea) -> anyhow::Result<Iteration> {
        let project_store = ProjectStore::new();
        let mut project = project_store
            .load()?
            .ok_or_else(|| anyhow::anyhow!("项目未初始化"))?;

        // 执行互斥：迭代执行期间不允许创建新迭代（会与执行路径竞争 project.json）
        let _guard = crate::execution_guard::ExecutionGuard::try_acquire("autonomy", &idea.id)?;

        let base = project
            .iterations
            .iter()
            .filter(|i| i.status == IterationStatus::Completed)
            .max_by_key(|i| i.number)
            .cloned();

        let description = build_iteration_description(idea);

        let mut iteration = match base {
            Some(base_iteration) => Iteration::create_evolution(
                &project,
                idea.title.clone(),
                description,
                base_iteration.id,
                InheritanceMode::Partial,
            ),
            None => Iteration::create_genesis(&project, idea.title.clone(), description),
        };
        iteration.source_idea_id = Some(idea.id.clone());

        let iteration_store = IterationStore::new();
        iteration_store.save(&iteration)?;
        project_store.add_iteration(&mut project, iteration.to_summary())?;
        Ok(iteration)
    }

    // ------------------------------------------------------------------
    // 自主循环
    // ------------------------------------------------------------------

    /// 前置检查：这一轮该不该跑。
    pub fn should_trigger(&self, trigger: CycleTrigger) -> TriggerDecision {
        if self.state.status == AutonomyStatus::Tripped {
            return TriggerDecision::skip(SkipReason::Tripped);
        }
        if self.state.status == AutonomyStatus::Paused {
            return TriggerDecision::skip(SkipReason::Paused);
        }
        if is_cycle_running() {
            // 上一轮还没跑完，不重入（宿主 tick 与人工触发都走这里）。
            // 判据是**进程内**的持有者，不是磁盘状态：磁盘上的 researching
            // 可能只是上次崩溃的残留，用它拦会让循环永久死锁。
            return TriggerDecision::skip(SkipReason::AlreadyRunning);
        }
        if trigger != CycleTrigger::Manual && !self.config.enabled {
            return TriggerDecision::skip(SkipReason::Disabled);
        }
        if !trigger.enabled_in(&self.config.triggers) {
            return TriggerDecision::skip(SkipReason::TriggerDisabled);
        }
        if crate::execution_guard::is_execution_in_progress() {
            return TriggerDecision::skip(SkipReason::ExecutionInProgress);
        }
        if let Ok(daily) = self.store.load_daily_budget()
            && daily.is_exhausted(self.config.budget.max_cycles_per_day)
        {
            return TriggerDecision::skip(SkipReason::BudgetExhausted);
        }
        if matches!(trigger, CycleTrigger::Schedule | CycleTrigger::Idle)
            && let Some(last) = self.state.last_cycle_at
            && elapsed_since(last) < schedule_interval(&self.config)
        {
            return TriggerDecision::skip(SkipReason::NotDue);
        }
        TriggerDecision::run()
    }

    /// 跑一轮自主调研。
    ///
    /// 三重保护，缺一不可：
    /// 1. **进程内互斥**（[`CycleRunGuard`]）：宿主 tick 与人工触发不会同时开跑；
    /// 2. **单请求超时**（`TimeoutLlm`）：上游不回包也不会永久 pending；
    /// 3. **整轮超时**（[`CYCLE_TIMEOUT`]）：无论卡在哪一步，一定有结局。
    ///
    /// 任何失败路径都统一收敛成一条 failed 快照 + 状态回 Idle，
    /// 不留"永不返回的将来时"，也不留"永远转圈的状态"。
    pub async fn run_cycle(
        &mut self,
        trigger: CycleTrigger,
        model: Arc<dyn Llm>,
        progress: &dyn CycleProgressSink,
    ) -> anyhow::Result<CycleReport> {
        // 孤立状态自愈：上次崩溃留下的 researching/proposing 不该继续拦路
        self.heal_if_orphaned()?;

        let decision = self.should_trigger(trigger);
        if !decision.should_run {
            let reason = decision.skip_reason.unwrap_or(SkipReason::Disabled);
            return Ok(self.skipped_report(trigger, reason));
        }

        let budget = BudgetGuard::new(&self.config.budget);
        if !budget.can_call_llm() {
            return Ok(self.skipped_report(trigger, SkipReason::BudgetExhausted));
        }

        let cycle_id = format!("cycle-{}", Utc::now().timestamp_millis());

        // 进程内互斥（真闸门）：拿不到槽位说明有别的周期刚抢进来
        let Some(_run_guard) = CycleRunGuard::try_acquire(&cycle_id, trigger.as_str()) else {
            return Ok(self.skipped_report(trigger, SkipReason::AlreadyRunning));
        };

        // 状态机：Idle → Researching；同时把"进行中的周期"落盘，
        // 让前端切走再回来（甚至重启）也知道卡在哪一步。
        self.state.status = AutonomyStatus::Researching;
        self.state.active_cycle = Some(ActiveCycle::new(cycle_id.clone(), trigger));
        self.persist()?;

        // 落到局部变量再 match：临时 future 在语句结束时被丢弃，
        // 超时分支才能安全地重新借用 self。
        let attempt = tokio::time::timeout(
            CYCLE_TIMEOUT,
            self.run_cycle_body(&cycle_id, trigger, model, budget, progress),
        )
        .await;

        match attempt {
            Ok(Ok(report)) => Ok(report),
            Ok(Err(e)) => self.fail_cycle(&cycle_id, trigger, format!("{:#}", e), progress),
            Err(_) => self.fail_cycle(
                &cycle_id,
                trigger,
                format!(
                    "整轮调研超过 {} 分钟仍未完成，已中止（多半是模型或网关没有响应）",
                    CYCLE_TIMEOUT.as_secs() / 60
                ),
                progress,
            ),
        }
    }

    /// 一轮调研的主体（不含互斥与超时，由 [`Self::run_cycle`] 包裹）。
    async fn run_cycle_body(
        &mut self,
        cycle_id: &str,
        trigger: CycleTrigger,
        model: Arc<dyn Llm>,
        mut budget: BudgetGuard,
        progress: &dyn CycleProgressSink,
    ) -> anyhow::Result<CycleReport> {
        let existing_ideas = self.idea_store.load_all()?;
        let previous_cycle = self.store.load_latest_cycle().ok().flatten();

        let mut snapshot = CycleSnapshot::new(cycle_id.to_string(), trigger, String::new());

        // --- 步骤 1：收集证据 -------------------------------------------------
        let mut step = CycleStep::new("收集证据");
        step.status = StepStatus::Running;
        self.record_step(cycle_id, &step, progress);

        let mut ctx = match collect_research(
            &self.config.research,
            &existing_ideas,
            previous_cycle.as_ref(),
        ) {
            Ok(ctx) => ctx,
            Err(e) => {
                return self.fail_cycle(
                    cycle_id,
                    trigger,
                    format!("收集证据失败：{}", e),
                    progress,
                );
            }
        };
        snapshot.evidence_digest = ctx.evidence_digest();

        // 外部检索策略（设计 §5.4）：本地三处都查不到才允许外查
        let external_allowed = self.config.research.external_mcp == ExternalMcpPolicy::AsNeeded
            && !ctx.has_local_evidence();
        if self.config.research.external_mcp == ExternalMcpPolicy::Off {
            ctx.degraded = true;
            ctx.degraded_reason = Some("外部检索已关闭".to_string());
        } else if external_allowed {
            let servers = crate::config_definition::agent_factory::get_mcp_server_names();
            if servers.is_empty() {
                ctx.degraded = true;
                ctx.degraded_reason = Some("没有可用的外部检索服务".to_string());
            } else {
                snapshot.mcp_servers = servers;
            }
        }

        step.status = StepStatus::Done;
        step.detail = Some(format!(
            "已知问题 {} 条、未完成特性 {} 条、反馈 {} 条、TODO {} 处",
            ctx.known_issues.len(),
            ctx.pending_features.len(),
            ctx.feedback_items.len(),
            ctx.todo_markers.len()
        ));
        self.record_step(cycle_id, &step, progress);
        snapshot.steps.push(step);

        // --- 步骤 2：生成提案 -------------------------------------------------
        let mut step = CycleStep::new("生成提案");
        step.status = StepStatus::Running;
        self.record_step(cycle_id, &step, progress);

        // 状态机：Researching → Proposing
        self.state.status = AutonomyStatus::Proposing;
        self.persist()?;

        let prompt = build_planner_prompt(&ctx, external_allowed);
        // 预算在这里变成**真闸门**：迭代上限 = LLM 调用上限，
        // 外部检索次数超限直接掐断本轮，工具超时防止某个 MCP 卡死拖垮整轮
        // （adk 默认单工具 5 分钟且带重试）。
        let outcome = match crate::agents::run_autonomous_planner_bounded(
            model,
            prompt,
            "autonomy-research".to_string(),
            budget.llm_limit(),
            CYCLE_TOOL_TIMEOUT,
            budget.mcp_limit(),
            external_allowed,
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(e) => {
                return self.fail_cycle(
                    cycle_id,
                    trigger,
                    format!("生成提案失败：{}", e),
                    progress,
                );
            }
        };

        // 留下复核依据：模型说了什么、调了哪些工具、有没有真的提交。
        // 没有这三样，"没有新发现"就是一句无法证伪的空话。
        if !outcome.text.trim().is_empty() {
            snapshot.planner_note = Some(truncate(&outcome.text, 4000));
        }
        snapshot.called_tools = outcome.called_tools.clone();
        snapshot.propose_called = outcome.propose_called;

        // 「0 条提案」有三种来源，只有第一种是结论：
        // ① 模型提交了空数组 ② 预算掐断 ③ 压根没调用 propose_ideas
        if outcome.truncated && outcome.drafts.is_empty() {
            return self.fail_cycle(
                cycle_id,
                trigger,
                format!(
                    "外部检索用满 {} 次预算，本轮在提交提案前被中止（这不算「没有缺口」）",
                    budget.mcp_limit()
                ),
                progress,
            );
        }
        if !outcome.propose_called {
            return self.fail_cycle(
                cycle_id,
                trigger,
                "模型没有提交提案（未调用 propose_ideas），本轮不作数".to_string(),
                progress,
            );
        }
        if outcome.drafts.is_empty() && outcome.parse_errors > 0 {
            return self.fail_cycle(
                cycle_id,
                trigger,
                format!(
                    "提案参数不合法（{} 次解析失败），本轮不作数",
                    outcome.parse_errors
                ),
                progress,
            );
        }

        // 外部检索次数由 Agent 侧真实计数（不是跑完再数工具名）
        for _ in 0..outcome.external_calls {
            budget.consume_mcp();
        }
        // 真实模型回合数：一次收尾回合 + 每个发起工具调用的回合。
        // 以前这里写的是 budget.llm_used()，恒等于 1，等于没记。
        snapshot.llm_calls = 1 + outcome.tool_rounds;
        snapshot.mcp_calls = budget.mcp_used();
        if outcome.used_external_search() && snapshot.mcp_servers.is_empty() {
            snapshot.mcp_servers = vec!["外部检索".to_string()];
        }

        step.status = StepStatus::Done;
        step.detail = Some(format!(
            "模型提交了 {} 条提案（{} 次工具调用）",
            outcome.drafts.len(),
            outcome.called_tools.len()
        ));
        self.record_step(cycle_id, &step, progress);
        snapshot.steps.push(step);

        // --- 步骤 3：质量门槛 + 去重 + 发布 ---------------------------------
        let mut step = CycleStep::new("质量门槛与去重");
        step.status = StepStatus::Running;
        self.record_step(cycle_id, &step, progress);

        let publish = match self.publish_proposals(&outcome.drafts, &mut snapshot, &existing_ideas) {
            Ok(publish) => publish,
            Err(e) => {
                return self.fail_cycle(
                    cycle_id,
                    trigger,
                    format!("发布建议失败：{}", e),
                    progress,
                );
            }
        };

        step.status = StepStatus::Done;
        step.detail = Some(format!(
            "发布 {} 条，合并 {} 条，过滤 {} 条",
            publish.published.len(),
            snapshot.merged_count,
            snapshot.discarded.len()
        ));
        self.record_step(cycle_id, &step, progress);
        snapshot.steps.push(step);

        // --- 状态更新与熔断 -------------------------------------------------
        let published_count = publish.published.len();
        if published_count == 0 {
            self.state.consecutive_empty_cycles = self.state.consecutive_empty_cycles.saturating_add(1);
        } else {
            self.state.consecutive_empty_cycles = 0;
        }
        self.state.cycle_count = self.state.cycle_count.saturating_add(1);
        self.state.last_cycle_at = Some(Utc::now());
        // 状态机：有新建议 → AwaitingDecision（等人决策）；没有 → 回到 Idle
        self.state.status = if published_count > 0 {
            AutonomyStatus::AwaitingDecision
        } else {
            AutonomyStatus::Idle
        };

        snapshot.outcome = if published_count > 0 {
            CycleOutcome::Completed
        } else {
            CycleOutcome::Empty
        };

        if let Some(reason) = self.detect_breaker(&snapshot) {
            self.state.status = AutonomyStatus::Tripped;
            self.state.tripped_reason = Some(reason.clone());
            snapshot.outcome = CycleOutcome::Tripped;
        }

        snapshot.finished_at = Some(Utc::now());
        snapshot.summarize();
        // 一轮结束：清掉进行中标记，状态与磁盘都回到"有结局"的样子
        self.state.active_cycle = None;
        self.store.save_cycle(&snapshot)?;

        // 每日预算记账
        let mut daily: DailyBudget = self.store.load_daily_budget()?;
        daily.rollover(Utc::now());
        daily.consume();
        self.store.save_daily_budget(&daily)?;

        self.persist()?;
        progress.on_snapshot(&snapshot);

        Ok(CycleReport {
            snapshot,
            published: publish.published,
            skipped: None,
        })
    }

    /// 推进一个步骤：更新进行中周期的步骤表 → 落盘 → 推送前端。
    ///
    /// 落盘是这里的关键：前端刷新/切换页面后靠 `state.active_cycle.steps`
    /// 重放进度，而不是依赖可能丢过一次就没了的 Tauri 事件。
    fn record_step(&mut self, cycle_id: &str, step: &CycleStep, progress: &dyn CycleProgressSink) {
        if let Some(active) = self.state.active_cycle.as_mut() {
            active.upsert_step(step.clone());
        }
        if let Err(e) = self.store.save_state(&self.state) {
            tracing::warn!("[autonomy] 调研进度落盘失败: {}", e);
        }
        progress.on_step(cycle_id, step);
    }

    /// 失败收敛：**唯一**的失败出口。
    ///
    /// 无论失败来自哪一步、是错误还是超时，都走这里，保证：
    /// 落一条 failed 快照（保留已发生的步骤）→ 状态回 Idle →
    /// 清掉进行中标记 → 记账 → 推送前端。调用方永远不会拿到 Err。
    fn fail_cycle(
        &mut self,
        cycle_id: &str,
        trigger: CycleTrigger,
        reason: String,
        progress: &dyn CycleProgressSink,
    ) -> anyhow::Result<CycleReport> {
        let mut snapshot = CycleSnapshot::new(cycle_id.to_string(), trigger, String::new());
        // 保留已经跑过的步骤，让"卡在哪一步"可追溯
        if let Some(active) = self.state.active_cycle.as_ref() {
            snapshot.steps = active.steps.clone();
        }
        if let Some(running) = snapshot
            .steps
            .iter_mut()
            .find(|s| s.status == StepStatus::Running)
        {
            running.status = StepStatus::Failed;
            running.detail = Some(reason.clone());
        }
        snapshot.outcome = CycleOutcome::Failed;
        snapshot.finished_at = Some(Utc::now());
        snapshot.note = Some(format!("这轮调研没跑完：{}。已有的建议不受影响。", reason));

        self.state.status = AutonomyStatus::Idle;
        self.state.active_cycle = None;
        if let Err(e) = self.persist() {
            tracing::warn!("[autonomy] 失败状态落盘失败: {}", e);
        }

        // 失败也记账：真的调过模型，就该算这一轮，避免失败重试无限刷
        match self.store.load_daily_budget() {
            Ok(mut daily) => {
                daily.rollover(Utc::now());
                daily.consume();
                if let Err(e) = self.store.save_daily_budget(&daily) {
                    tracing::warn!("[autonomy] 每日预算记账失败: {}", e);
                }
            }
            Err(e) => tracing::warn!("[autonomy] 读取每日预算失败: {}", e),
        }

        if let Err(e) = self.store.save_cycle(&snapshot) {
            tracing::warn!("[autonomy] 失败快照落盘失败: {}", e);
        }
        progress.on_snapshot(&snapshot);

        Ok(CycleReport {
            snapshot,
            published: Vec::new(),
            skipped: None,
        })
    }

    /// 被跳过的一轮：只回执，**不落盘**。
    ///
    /// 宿主每 30s 一次 tick，被跳过是常态；写进历史会淹没真正的调研轮次。
    fn skipped_report(&self, trigger: CycleTrigger, reason: SkipReason) -> CycleReport {
        let mut snapshot = CycleSnapshot::new(
            format!("cycle-{}", Utc::now().timestamp_millis()),
            trigger,
            String::new(),
        );
        snapshot.outcome = CycleOutcome::Skipped;
        snapshot.finished_at = Some(Utc::now());
        snapshot.summarize();
        CycleReport {
            snapshot,
            published: Vec::new(),
            skipped: Some(reason),
        }
    }

    /// 质量门槛 → 去重 → 状态抑制 → 限量 → 落库。
    fn publish_proposals(
        &mut self,
        drafts: &[IdeaDraft],
        snapshot: &mut CycleSnapshot,
        existing: &[Idea],
    ) -> anyhow::Result<PublishOutcome> {
        let max_publish = crate::instructions::MAX_PROPOSALS_PER_CYCLE;
        let mut published = Vec::new();
        let mut duplicates = 0usize;
        let mut staged: Vec<(String, Idea)> = Vec::new();

        for draft in drafts {
            let fingerprint = compute_fingerprint(&draft.title, &draft.problem);
            let idea = draft.clone().into_idea("pending".to_string(), IdeaSource::Research);

            // ① 质量门槛
            let issues = idea.quality_issues();
            if !issues.is_empty() {
                snapshot
                    .discarded
                    .push(format!("{}：{}", truncate(&draft.title, 40), issues.join("；")));
                continue;
            }

            // ② 去重（同指纹）
            let same_fingerprint = existing
                .iter()
                .find(|i| i.fingerprint == fingerprint)
                .or_else(|| {
                    staged
                        .iter()
                        .find(|(fp, _)| *fp == fingerprint)
                        .map(|(_, i)| i)
                });

            if let Some(previous) = same_fingerprint {
                let evidence_same = previous.evidence_key() == idea.evidence_key();
                if evidence_same {
                    // 状态抑制：证据未变化则不再重提
                    duplicates += 1;
                    snapshot
                        .discarded
                        .push(format!("{}：与已有建议重复", truncate(&draft.title, 40)));
                    continue;
                }
                if previous.status.is_suppressing() {
                    // 证据变化：重新激活既有建议（状态抑制只在证据不变时生效）
                    let mut reactivated = previous.clone();
                    reactivated.evidence = idea.evidence.clone();
                    reactivated.status = IdeaStatus::Proposed;
                    reactivated.updated_at = Utc::now();
                    self.idea_store.save(&reactivated)?;
                    published.push(reactivated.to_summary());
                    continue;
                }
                duplicates += 1;
                snapshot
                    .discarded
                    .push(format!("{}：与已有建议重复", truncate(&draft.title, 40)));
                continue;
            }

            // ③ 限量：超出的留在 Draft 池（存盘但不发布）
            if published.len() >= max_publish {
                let id = self.idea_store.next_idea_id("idea")?;
                let mut draft_idea = idea;
                draft_idea.id = id;
                draft_idea.status = IdeaStatus::Draft;
                draft_idea.created_at = Utc::now();
                draft_idea.updated_at = Utc::now();
                self.idea_store.save(&draft_idea)?;
                snapshot.drafted_idea_ids.push(draft_idea.id.clone());
                continue;
            }

            let id = self.idea_store.next_idea_id("idea")?;
            let mut final_idea = idea;
            final_idea.id = id.clone();
            final_idea.status = IdeaStatus::Proposed;
            final_idea.created_at = Utc::now();
            final_idea.updated_at = Utc::now();
            self.idea_store.save(&final_idea)?;
            staged.push((fingerprint, final_idea.clone()));
            published.push(final_idea.to_summary());
        }

        snapshot.merged_count = duplicates as u32;

        // 冲突检测（设计 §7.3）：两条互相冲突的建议都标出来
        self.detect_conflicts(&staged, &mut published)?;

        Ok(PublishOutcome { published })
    }

    /// 熔断判定（设计 §8-6）。
    fn detect_breaker(&self, snapshot: &CycleSnapshot) -> Option<String> {
        if self.state.consecutive_empty_cycles >= MAX_CONSECUTIVE_EMPTY_CYCLES {
            return Some(format!(
                "连续 {} 轮没有找到值得做的方向，已暂停自主循环",
                self.state.consecutive_empty_cycles
            ));
        }
        if self.state.iteration_create_failures >= MAX_ITERATION_CREATE_FAILURES {
            return Some(format!(
                "连续 {} 次创建迭代失败，已暂停自主循环",
                self.state.iteration_create_failures
            ));
        }
        let duplicates = snapshot
            .discarded
            .iter()
            .filter(|d| d.contains("重复"))
            .count();
        if duplicates >= MAX_DUPLICATES_PER_CYCLE {
            return Some(format!(
                "本轮有 {} 条提案与已有建议重复，已暂停自主循环",
                duplicates
            ));
        }
        None
    }

    fn persist(&self) -> anyhow::Result<()> {
        self.store.save_config(&self.config)?;
        self.store.save_state(&self.state)?;
        Ok(())
    }

    /// 冲突检测：标记互相冲突的建议（设计 §7.3）。
    fn detect_conflicts(
        &mut self,
        staged: &[(String, Idea)],
        published: &mut [IdeaSummary],
    ) -> anyhow::Result<()> {
        let mut updates: Vec<(String, Vec<String>)> = Vec::new();
        for (index, (_, idea)) in staged.iter().enumerate() {
            let mut conflicts = idea.conflicts_with.clone();
            for (other_index, (_, other)) in staged.iter().enumerate() {
                if other_index == index {
                    continue;
                }
                if other.conflicts_with.contains(&idea.id) && !conflicts.contains(&other.id) {
                    conflicts.push(other.id.clone());
                }
            }
            if !conflicts.is_empty() {
                updates.push((idea.id.clone(), conflicts));
            }
        }

        for (id, conflicts) in updates {
            if let Ok(mut idea) = self.idea_store.load(&id) {
                idea.conflicts_with = conflicts.clone();
                self.idea_store.save(&idea)?;
                if let Some(summary) = published.iter_mut().find(|s| s.id == id) {
                    summary.conflicts_with = conflicts;
                }
            }
        }
        Ok(())
    }
}

struct PublishOutcome {
    published: Vec<IdeaSummary>,
}

/// 定时/空闲触发的最小间隔。
fn schedule_interval(config: &AutonomyConfig) -> std::time::Duration {
    std::time::Duration::from_secs(config.triggers.schedule_interval_mins.saturating_mul(60))
}

/// 距上次运行过去了多久。
fn elapsed_since(last: DateTime<Utc>) -> std::time::Duration {
    Utc::now()
        .signed_duration_since(last)
        .to_std()
        .unwrap_or(std::time::Duration::ZERO)
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        format!(
            "{}…",
            text.chars().take(max_chars.saturating_sub(1)).collect::<String>()
        )
    } else {
        text.to_string()
    }
}

/// 组装规划 Agent 的 prompt。
fn build_planner_prompt(ctx: &super::research::ResearchContext, external_allowed: bool) -> String {
    let mut prompt = String::new();
    prompt.push_str("这是一次自主调研。请基于下面的调研上下文判断项目当前最真实的缺口，并提交提案。\n\n");
    prompt.push_str(&ctx.to_prompt());
    prompt.push_str("\n\n");
    if external_allowed {
        prompt.push_str(
            "本地证据不足，你可以使用已挂载的外部检索工具补充事实；\
             外部证据必须写明来源与访问时间，且不能作为唯一证据。\n",
        );
    } else {
        prompt.push_str("本轮只用本地证据，不要发起外部检索。\n");
    }
    prompt.push_str("\n完成后必须调用一次 propose_ideas 提交结论。");
    prompt
}

/// 由建议组装迭代描述（问题 + 假设 + 收益 + 证据摘要）。
fn build_iteration_description(idea: &Idea) -> String {
    let mut description = String::new();
    description.push_str(&format!("## 问题\n{}\n\n", idea.problem));
    description.push_str(&format!("## 假设\n{}\n\n", idea.hypothesis));
    if !idea.expected_benefit.trim().is_empty() {
        description.push_str(&format!("## 预期收益\n{}\n\n", idea.expected_benefit));
    }
    description.push_str("## 证据\n");
    for evidence in &idea.evidence {
        description.push_str(&format!(
            "- [{}] {}\n",
            evidence.credibility().as_str(),
            evidence.locator()
        ));
        let excerpt = evidence.excerpt();
        if !excerpt.trim().is_empty() {
            description.push_str(&format!("  > {}\n", truncate(excerpt, 300)));
        }
    }
    description.push_str(&format!(
        "\n## 建议来源\n自主调研建议 {}（范围：{}，预估工作量：{}）\n",
        idea.id,
        idea.scope.as_str(),
        idea.estimated_effort.as_str()
    ));
    description
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::Priority;
    use crate::domain::idea::{Effort, Evidence, IdeaScope};
    use crate::persistence::WORKSPACE_TEST_LOCK as TEST_LOCK;
    use adk_core::{AdkError, LlmRequest, LlmResponseStream};
    use async_trait::async_trait;
    use std::sync::PoisonError;
    use tempfile::TempDir;

    /// 跳过路径不应该触碰 LLM；一旦被调用就 panic。
    struct PanicLlm;

    #[async_trait]
    impl Llm for PanicLlm {
        fn name(&self) -> &str {
            "panic-llm"
        }

        async fn generate_content(
            &self,
            _req: LlmRequest,
            _stream: bool,
        ) -> Result<LlmResponseStream, AdkError> {
            panic!("skipped cycle must not call the LLM");
        }
    }

    fn setup_workspace() -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());
        crate::persistence::init_project_structure("demo").expect("init");
        ProjectStore::new()
            .save(&crate::domain::Project::new("demo"))
            .expect("save project");
        temp
    }

    fn draft(title: &str, problem: &str) -> IdeaDraft {        IdeaDraft {
            title: title.to_string(),
            problem: problem.to_string(),
            hypothesis: "做完之后 A 会变成 B".to_string(),
            scope: IdeaScope::Fix,
            evidence: vec![Evidence::FileRef {
                path: "src/main.rs".to_string(),
                line_range: None,
                excerpt: "fn retry() {}".to_string(),
            }],
            estimated_effort: Effort::S,
            priority: Priority::High,
            expected_benefit: "更稳".to_string(),
            conflicts_with: vec![],
        }
    }

    #[test]
    fn user_input_idea_follows_same_quality_gate() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");

        let good = controller
            .propose_user_idea(draft("补上幂等", "重试会重复"))
            .expect("good");
        assert_eq!(good.status, IdeaStatus::Proposed);
        assert_eq!(good.source, IdeaSource::UserInput);

        let mut bad = draft("绕过限流", "重试会重复");
        bad.evidence = vec![];
        let rejected = controller.propose_user_idea(bad).expect("bad");
        assert_eq!(rejected.status, IdeaStatus::Draft, "不达标只入池不发布");
    }

    #[test]
    fn accept_creates_iteration_and_backfills_source_idea_id() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        let idea = controller
            .propose_user_idea(draft("补上幂等", "重试会重复"))
            .expect("idea");

        let outcome = controller
            .decide_idea(&idea.id, DecisionAction::Accept, "值得做", None, None)
            .expect("decide");

        let iteration = outcome.iteration.expect("iteration created");
        assert_eq!(iteration.source_idea_id.as_deref(), Some(idea.id.as_str()));
        // 没有已完成迭代 → 按创世迭代处理（inheritance=None）
        assert_eq!(iteration.inheritance, InheritanceMode::None);
        assert_eq!(iteration.status, IterationStatus::Draft);
        assert!(iteration.description.contains("重试会重复"));

        let reloaded = controller.get_idea(&idea.id).expect("idea");
        assert_eq!(reloaded.status, IdeaStatus::Converted);
        assert_eq!(
            reloaded.derived_iteration_id.as_deref(),
            Some(iteration.id.as_str())
        );

        let timeline = controller.decision_timeline(&idea.id).expect("timeline");
        assert_eq!(timeline.len(), 1);
        assert_eq!(
            timeline[0].resulting_iteration_id.as_deref(),
            Some(iteration.id.as_str())
        );
    }

    #[test]
    fn reject_blocks_second_decision_and_undo_restores() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        let idea = controller
            .propose_user_idea(draft("补上幂等", "重试会重复"))
            .expect("idea");

        controller
            .decide_idea(&idea.id, DecisionAction::Reject, "暂时不做", None, None)
            .expect("reject");
        assert_eq!(
            controller.get_idea(&idea.id).expect("idea").status,
            IdeaStatus::Rejected
        );

        let err = controller
            .decide_idea(&idea.id, DecisionAction::Accept, "改主意", None, None)
            .expect_err("must fail");
        assert!(format!("{:#}", err).contains("已经决策过"), "{}", err);

        let restored = controller.undo_decision(&idea.id).expect("undo");
        assert_eq!(restored.status, IdeaStatus::Proposed);
        let timeline = controller.decision_timeline(&idea.id).expect("timeline");
        assert_eq!(timeline.len(), 2, "撤销是追加，不是改写");
        assert!(timeline[1].is_reversal());
    }

    #[test]
    fn rewrite_keeps_original_and_stays_open() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        let idea = controller
            .propose_user_idea(draft("补上幂等", "重试会重复"))
            .expect("idea");

        controller
            .decide_idea(
                &idea.id,
                DecisionAction::Rewrite,
                "范围收窄",
                Some(IdeaRewrite {
                    title: "只补文档写入幂等".to_string(),
                    problem: "文档写入缺少存在性检查".to_string(),
                    hypothesis: "补检查后不覆盖已有文档".to_string(),
                    scope: IdeaScope::Fix,
                }),
                None,
            )
            .expect("rewrite");

        let reloaded = controller.get_idea(&idea.id).expect("idea");
        assert_eq!(reloaded.status, IdeaStatus::Rewritten);
        assert_eq!(reloaded.title, "只补文档写入幂等");
        assert_eq!(reloaded.original.expect("original").title, "补上幂等");
        assert!(reloaded.status.is_open(), "改写后仍可继续决策");
    }

    #[test]
    fn defer_records_deadline() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        let idea = controller
            .propose_user_idea(draft("补上幂等", "重试会重复"))
            .expect("idea");
        let until = Utc::now() + chrono::Duration::days(7);

        controller
            .decide_idea(
                &idea.id,
                DecisionAction::Defer,
                "下周再看",
                None,
                Some(until),
            )
            .expect("defer");

        let reloaded = controller.get_idea(&idea.id).expect("idea");
        assert_eq!(reloaded.status, IdeaStatus::Deferred);
        assert!(reloaded.decision.expect("decision").deferred_until.is_some());
    }

    #[test]
    fn trigger_decision_respects_state_and_budget() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");

        assert_eq!(
            controller.should_trigger(CycleTrigger::Schedule).skip_reason,
            Some(SkipReason::Disabled)
        );
        assert!(controller.should_trigger(CycleTrigger::Manual).should_run);

        controller.enable().expect("enable");
        assert_eq!(
            controller.should_trigger(CycleTrigger::Schedule).skip_reason,
            Some(SkipReason::TriggerDisabled)
        );

        controller.stop().expect("stop");
        controller.enable().expect("enable");
        controller.state.status = AutonomyStatus::Tripped;
        assert_eq!(
            controller.should_trigger(CycleTrigger::Manual).skip_reason,
            Some(SkipReason::Tripped)
        );
        controller.reset_trip().expect("reset");
        assert!(controller.should_trigger(CycleTrigger::Manual).should_run);

        let mut daily = DailyBudget::for_day(Utc::now());
        for _ in 0..controller.config.budget.max_cycles_per_day {
            daily.consume();
        }
        controller.store.save_daily_budget(&daily).expect("save budget");
        assert_eq!(
            controller.should_trigger(CycleTrigger::Manual).skip_reason,
            Some(SkipReason::BudgetExhausted)
        );
    }

    #[test]
    fn publish_proposals_enforces_gate_dedup_and_limit() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        let existing = controller.list_ideas().expect("ideas");
        let mut snapshot =
            CycleSnapshot::new("cycle-test".to_string(), CycleTrigger::Manual, "digest".to_string());

        let drafts: Vec<IdeaDraft> = (0..5)
            .map(|i| draft(&format!("建议 {}", i), &format!("问题 {}", i)))
            .collect();
        let outcome = controller
            .publish_proposals(&drafts, &mut snapshot, &existing)
            .expect("publish");
        assert_eq!(outcome.published.len(), 3, "每轮最多发布 3 条");
        assert_eq!(snapshot.drafted_idea_ids.len(), 2, "超出的进 Draft 池");

        let mut snapshot2 =
            CycleSnapshot::new("cycle-test-2".to_string(), CycleTrigger::Manual, "digest".to_string());
        let mut bad = draft("坏建议", "问题");
        bad.evidence = vec![];
        let outcome2 = controller
            .publish_proposals(&[bad], &mut snapshot2, &existing)
            .expect("publish");
        assert!(outcome2.published.is_empty());
        assert_eq!(snapshot2.discarded.len(), 1);
        assert!(snapshot2.discarded[0].contains("本地证据"));
    }

    #[test]
    fn publish_proposals_merges_same_fingerprint() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        controller
            .propose_user_idea(draft("补上幂等", "重试会重复"))
            .expect("first");
        let existing = controller.list_ideas().expect("ideas");
        assert_eq!(existing.len(), 1);

        let mut snapshot =
            CycleSnapshot::new("cycle-dup".to_string(), CycleTrigger::Manual, "digest".to_string());
        let outcome = controller
            .publish_proposals(
                &[draft("补上幂等", "重试会重复")],
                &mut snapshot,
                &existing,
            )
            .expect("publish");
        assert!(outcome.published.is_empty(), "同指纹必须合并");
        assert_eq!(snapshot.merged_count, 1);
        assert_eq!(controller.list_ideas().expect("ideas").len(), 1);
    }

    #[test]
    fn breaker_trips_after_three_empty_cycles() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        controller.state.consecutive_empty_cycles = MAX_CONSECUTIVE_EMPTY_CYCLES;
        let snapshot =
            CycleSnapshot::new("cycle-empty".to_string(), CycleTrigger::Manual, "digest".to_string());
        let reason = controller.detect_breaker(&snapshot).expect("trip");
        assert!(reason.contains("连续"));
    }

    #[test]
    fn accept_uses_completed_iteration_as_base_with_partial_inheritance() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        // 先造一个已完成的基线迭代
        let project_store = ProjectStore::new();
        let mut project = project_store.load().expect("load").expect("project");
        let mut base = Iteration::create_genesis(&project, "基线".to_string(), "第一版".to_string());
        base.start();
        base.complete();
        IterationStore::new().save(&base).expect("save base");
        project_store
            .add_iteration(&mut project, base.to_summary())
            .expect("add base");

        let mut controller = AutonomyController::new().expect("controller");
        let idea = controller
            .propose_user_idea(draft("补上幂等", "重试会重复"))
            .expect("idea");

        let outcome = controller
            .decide_idea(&idea.id, DecisionAction::Accept, "值得做", None, None)
            .expect("decide");
        let iteration = outcome.iteration.expect("iteration");

        assert_eq!(iteration.inheritance, InheritanceMode::Partial);
        assert_eq!(iteration.base_iteration_id.as_deref(), Some(base.id.as_str()));
        assert_eq!(iteration.source_idea_id.as_deref(), Some(idea.id.as_str()));
    }

    #[tokio::test]
    async fn run_cycle_skips_when_paused_and_records_snapshot() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        controller.enable().expect("enable");
        controller.pause().expect("pause");

        // 暂停时不应调用 LLM，直接跳过
        let model: Arc<dyn Llm> = Arc::new(PanicLlm);
        let report = controller
            .run_cycle(
                CycleTrigger::Manual,
                model,
                &super::super::cycle::NoopProgressSink,
            )
            .await
            .expect("run cycle");

        assert_eq!(report.skipped, Some(SkipReason::Paused));
        assert_eq!(report.snapshot.outcome, CycleOutcome::Skipped);
        assert!(report.published.is_empty());

        // 被跳过的轮次**不**写进历史：宿主每 30s 一次 tick，
        // 记录跳过会瞬间淹没调研历史。历史只记真正跑过的轮次。
        let cycles = controller.list_cycles().expect("cycles");
        assert!(cycles.is_empty(), "跳过的轮次不应落盘: {:?}", cycles.len());
    }

    #[test]
    fn decision_moves_state_machine_between_awaiting_and_idle() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        controller.enable().expect("enable");
        assert_eq!(controller.status().expect("status").status, AutonomyStatus::Idle);

        // 用户输入一条建议 → AwaitingDecision
        let idea = controller
            .propose_user_idea(draft("补上幂等", "重试会重复"))
            .expect("idea");
        assert_eq!(
            controller.status().expect("status").status,
            AutonomyStatus::AwaitingDecision
        );

        // 驳回后没有待决策建议 → 回到 Idle
        controller
            .decide_idea(&idea.id, DecisionAction::Reject, "暂时不做", None, None)
            .expect("reject");
        assert_eq!(controller.status().expect("status").status, AutonomyStatus::Idle);

        // 撤销 → 建议回到未决 → 又是 AwaitingDecision
        controller.undo_decision(&idea.id).expect("undo");
        assert_eq!(
            controller.status().expect("status").status,
            AutonomyStatus::AwaitingDecision
        );
    }

    #[test]
    fn stale_running_state_is_recovered_on_startup() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        controller.enable().expect("enable");
        controller.state.status = AutonomyStatus::Researching;
        controller.state.active_cycle = Some(ActiveCycle::new("cycle-dead", CycleTrigger::Manual));

        // 启动时复位 → 恢复可跑，且进行中标记一并清掉
        assert!(controller.recover_stale_state().expect("recover"));
        let state = controller.status().expect("status");
        assert_eq!(state.status, AutonomyStatus::Idle);
        assert!(state.active_cycle.is_none(), "崩溃残留的进行中周期必须清掉");
        assert!(controller.should_trigger(CycleTrigger::Manual).should_run);

        // 幂等：已经是 Idle 时不再改动
        assert!(!controller.recover_stale_state().expect("recover again"));
    }

    #[test]
    fn orphaned_running_state_is_healed_without_restart() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        controller.enable().expect("enable");
        // 模拟"上次崩溃/超时后状态没收回"：磁盘说在跑，进程内没有持有者
        controller.state.status = AutonomyStatus::Proposing;
        controller
            .store
            .save_state(&controller.state)
            .expect("persist orphaned state");

        // 重新读一份（模拟下一次 tick / 用户点开面板）
        let mut next = AutonomyController::new().expect("controller");
        assert!(next.heal_if_orphaned().expect("heal"));
        let state = next.status().expect("status");
        assert_eq!(state.status, AutonomyStatus::Idle);
        assert!(state.active_cycle.is_none());
    }

    #[test]
    fn persisted_running_state_no_longer_blocks_new_cycle() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        controller.enable().expect("enable");
        // 只剩磁盘状态、进程内没有持有者 → 不该再被 AlreadyRunning 拦下
        controller.state.status = AutonomyStatus::Researching;
        assert!(
            controller.should_trigger(CycleTrigger::Manual).should_run,
            "重入保护必须看进程内持有者，而不是磁盘状态"
        );
    }

    #[test]
    fn in_process_guard_blocks_reentry() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let controller = AutonomyController::new().expect("controller");
        let guard = crate::autonomy::guard::CycleRunGuard::try_acquire("cycle-live", "manual")
            .expect("acquire");

        assert_eq!(
            controller.should_trigger(CycleTrigger::Manual).skip_reason,
            Some(SkipReason::AlreadyRunning)
        );

        drop(guard);
        assert!(controller.should_trigger(CycleTrigger::Manual).should_run);
    }

    #[test]
    fn fail_cycle_converges_state_and_keeps_steps() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _temp = setup_workspace();

        let mut controller = AutonomyController::new().expect("controller");
        controller.enable().expect("enable");
        controller.state.status = AutonomyStatus::Proposing;
        let mut active = ActiveCycle::new("cycle-fail", CycleTrigger::Manual);
        let mut step = CycleStep::new("生成提案");
        step.status = StepStatus::Running;
        active.upsert_step(step);
        controller.state.active_cycle = Some(active);

        let report = controller
            .fail_cycle(
                "cycle-fail",
                CycleTrigger::Manual,
                "模型在 600 秒内没有返回响应".to_string(),
                &crate::autonomy::cycle::NoopProgressSink,
            )
            .expect("fail cycle");

        assert_eq!(report.snapshot.outcome, CycleOutcome::Failed);
        assert!(
            report
                .snapshot
                .note
                .as_deref()
                .expect("note")
                .contains("没有返回响应"),
            "失败原因必须写进 note: {:?}",
            report.snapshot.note
        );
        // 保留已发生的步骤，并标成失败
        assert_eq!(report.snapshot.steps.len(), 1);
        assert_eq!(report.snapshot.steps[0].status, StepStatus::Failed);

        // 状态收敛：必须回 Idle 且清掉进行中标记，否则下一轮会被挡在门外
        let state = controller.status().expect("status");
        assert_eq!(state.status, AutonomyStatus::Idle);
        assert!(state.active_cycle.is_none());
        assert!(controller.should_trigger(CycleTrigger::Manual).should_run);

        // 失败也记账，避免失败重试无限刷
        let cycles = controller.list_cycles().expect("cycles");
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].outcome, CycleOutcome::Failed);
        let budget = controller.budget_status().expect("budget");
        assert_eq!(budget.daily_cycles_used, 1);
    }

    #[test]
    fn iteration_description_contains_problem_hypothesis_and_evidence() {        let idea =
            draft("补上幂等", "重试会重复").into_idea("idea-1".to_string(), IdeaSource::Research);
        let description = build_iteration_description(&idea);
        assert!(description.contains("## 问题"));
        assert!(description.contains("重试会重复"));
        assert!(description.contains("## 假设"));
        assert!(description.contains("## 证据"));
        assert!(description.contains("src/main.rs"));
    }
}
