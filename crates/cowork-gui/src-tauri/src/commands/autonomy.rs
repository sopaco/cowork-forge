// Autonomy commands —— 自主模式 Tauri 命令层（薄壳）
//
// 只转发 core `AutonomyController` API + 驱动 AUTONOMY_HOST + 事件推送，
// 零业务逻辑：决策落库、状态迁移、去重都只在 core 里有一份实现。

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, Window};

use cowork_core::autonomy::{
    AutonomyConfig, AutonomyController, AutonomyState, BudgetStatus, CycleProgressSink,
    CycleSnapshot, CycleStep, CycleTrigger, DecisionOutcome,
};
use cowork_core::domain::idea::{DecisionAction, Idea, IdeaDraft, IdeaRewrite, IdeaSummary};

use crate::commands::AUTONOMY_HOST;

/// `autonomy_get_status` 返回的状态快照。
#[derive(Debug, Clone, Serialize)]
pub struct AutonomyStatusPayload {
    pub state: AutonomyState,
    pub config: AutonomyConfig,
    pub host_running: bool,
    pub host_paused: bool,
    pub budget: BudgetStatus,
}

fn build_payload(controller: &AutonomyController) -> Result<AutonomyStatusPayload, String> {
    let state = controller.status().map_err(|e| e.to_string())?;
    let config = controller.config().map_err(|e| e.to_string())?;
    let budget = controller.budget_status().map_err(|e| e.to_string())?;
    Ok(AutonomyStatusPayload {
        state,
        config,
        host_running: AUTONOMY_HOST.is_running(),
        host_paused: AUTONOMY_HOST.is_paused(),
        budget,
    })
}

/// 用户主动输入建议的入参。
#[derive(Debug, Clone, Deserialize)]
pub struct UserIdeaRequest {
    pub title: String,
    pub problem: String,
    pub hypothesis: String,
    pub scope: String,
    pub estimated_effort: String,
    pub priority: String,
    #[serde(default)]
    pub expected_benefit: String,
    #[serde(default)]
    pub evidence: Vec<serde_json::Value>,
}

/// 决策入参。
#[derive(Debug, Clone, Deserialize)]
pub struct DecideIdeaRequest {
    pub idea_id: String,
    /// accept | reject | rewrite | defer
    pub action: String,
    #[serde(default)]
    pub comment: String,
    /// 改写内容（rewrite 动作必填）。
    #[serde(default)]
    pub rewrite: Option<IdeaRewrite>,
    /// 置后到什么时候（ISO 8601）。
    #[serde(default)]
    pub deferred_until: Option<String>,
}

/// 把 core 的步骤进度转成 Tauri 事件。
struct TauriCycleSink {
    window: tauri::AppHandle,
}

impl CycleProgressSink for TauriCycleSink {
    fn on_step(&self, cycle_id: &str, step: &CycleStep) {
        let _ = self.window.emit(
            "autonomy_cycle_step",
            serde_json::json!({
                "cycle_id": cycle_id,
                "step": step,
            }),
        );
    }

    fn on_snapshot(&self, snapshot: &CycleSnapshot) {
        let _ = self.window.emit(
            "autonomy_cycle_snapshot",
            serde_json::json!({ "snapshot": snapshot }),
        );
    }
}

#[tauri::command]
pub async fn autonomy_start(window: Window) -> Result<AutonomyStatusPayload, String> {
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.enable().map_err(|e| e.to_string())?;
    AUTONOMY_HOST.start().map_err(|e| e.to_string())?;
    let payload = build_payload(&controller)?;
    let _ = window.emit("autonomy_state_changed", payload.state.clone());
    Ok(payload)
}

#[tauri::command]
pub async fn autonomy_stop(window: Window) -> Result<AutonomyStatusPayload, String> {
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.stop().map_err(|e| e.to_string())?;
    AUTONOMY_HOST.stop().await.map_err(|e| e.to_string())?;
    let payload = build_payload(&controller)?;
    let _ = window.emit("autonomy_state_changed", payload.state.clone());
    Ok(payload)
}

#[tauri::command]
pub async fn autonomy_pause(window: Window) -> Result<AutonomyStatusPayload, String> {
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.pause().map_err(|e| e.to_string())?;
    AUTONOMY_HOST.pause().map_err(|e| e.to_string())?;
    let payload = build_payload(&controller)?;
    let _ = window.emit("autonomy_state_changed", payload.state.clone());
    Ok(payload)
}

#[tauri::command]
pub async fn autonomy_resume(window: Window) -> Result<AutonomyStatusPayload, String> {
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.resume().map_err(|e| e.to_string())?;
    AUTONOMY_HOST.resume().map_err(|e| e.to_string())?;
    let payload = build_payload(&controller)?;
    let _ = window.emit("autonomy_state_changed", payload.state.clone());
    Ok(payload)
}

#[tauri::command]
pub async fn autonomy_get_status() -> Result<AutonomyStatusPayload, String> {
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    // 读状态时顺手自愈：状态说"在跑"但进程里没有在跑的周期，
    // 说明是上次崩溃的残留。否则前端会永远显示"正在调研"。
    if let Err(e) = controller.heal_if_orphaned() {
        tracing::warn!("[autonomy] 状态自愈失败: {}", e);
    }
    build_payload(&controller)
}

/// 熔断后的人工恢复。
#[tauri::command]
pub async fn autonomy_reset_trip(window: Window) -> Result<AutonomyStatusPayload, String> {
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.reset_trip().map_err(|e| e.to_string())?;
    let payload = build_payload(&controller)?;
    let _ = window.emit("autonomy_state_changed", payload.state.clone());
    Ok(payload)
}

/// 更新配置（自主模式的唯一配置源）。
#[tauri::command]
pub async fn autonomy_update_config(
    config: AutonomyConfig,
    window: Window,
) -> Result<AutonomyStatusPayload, String> {
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.update_config(config).map_err(|e| e.to_string())?;
    let payload = build_payload(&controller)?;
    let _ = window.emit("autonomy_state_changed", payload.state.clone());
    Ok(payload)
}

/// 手动跑一轮调研（立即调研）。
///
/// 核心层的 `run_cycle` 自带整轮超时与失败收敛，正常情况**总是**返回一份
/// 带结局的快照（失败在 `report.snapshot.outcome == failed` 里，文案在
/// `note` 里），所以这里的 `Err` 只对应"连状态都读不出来"这类异常。
/// 无论走哪条路，都必须把最新状态推给前端，否则按钮会一直转圈。
#[tauri::command]
pub async fn autonomy_run_cycle(window: Window) -> Result<serde_json::Value, String> {
    let config = cowork_core::llm::load_config().map_err(|e| format!("读取 LLM 配置失败: {}", e))?;
    let model = cowork_core::llm::create_llm_client(&config.llm)
        .map_err(|e| format!("创建 LLM 客户端失败: {}", e))?;

    let sink = TauriCycleSink {
        window: window.app_handle().clone(),
    };
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    let outcome = controller
        .run_cycle(CycleTrigger::Manual, model, &sink)
        .await;

    // 先推状态：失败分支也要推，前端靠它结束 loading
    if let Ok(state) = controller.status() {
        let _ = window.emit("autonomy_state_changed", state);
    }

    match outcome {
        Ok(report) => {
            let _ = window.emit(
                "autonomy_cycle_report",
                serde_json::json!({
                    "report": report,
                    "trigger": "manual",
                }),
            );
            Ok(serde_json::json!({ "report": report }))
        }
        Err(e) => Err(format!("自主调研失败: {:#}", e)),
    }
}

/// 建议列表（列表视图）。
#[tauri::command]
pub async fn autonomy_list_ideas() -> Result<Vec<IdeaSummary>, String> {
    let controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.list_summaries().map_err(|e| e.to_string())
}

/// 建议详情（含证据与决策历史）。
#[tauri::command]
pub async fn autonomy_get_idea(idea_id: String) -> Result<serde_json::Value, String> {
    let controller = AutonomyController::new().map_err(|e| e.to_string())?;
    let idea: Idea = controller.get_idea(&idea_id).map_err(|e| e.to_string())?;
    let timeline = controller
        .decision_timeline(&idea_id)
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "idea": idea,
        "timeline": timeline,
    }))
}

/// 决策门：采纳 / 驳回 / 改写 / 置后。
#[tauri::command]
pub async fn autonomy_decide_idea(
    request: DecideIdeaRequest,
    window: Window,
) -> Result<DecisionOutcome, String> {
    let action = match request.action.as_str() {
        "accept" => DecisionAction::Accept,
        "reject" => DecisionAction::Reject,
        "rewrite" => DecisionAction::Rewrite,
        "defer" => DecisionAction::Defer,
        other => return Err(format!("未知的决策动作: {}", other)),
    };

    let deferred_until = match request.deferred_until.as_deref() {
        Some(raw) if !raw.trim().is_empty() => Some(
            chrono::DateTime::parse_from_rfc3339(raw)
                .map_err(|e| format!("置后时间格式不正确: {}", e))?
                .with_timezone(&chrono::Utc),
        ),
        _ => None,
    };

    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    let outcome = controller
        .decide_idea(
            &request.idea_id,
            action,
            &request.comment,
            request.rewrite,
            deferred_until,
        )
        .map_err(|e| e.to_string())?;

    // 采纳后创建了迭代：通知前端刷新迭代列表
    if let Some(iteration) = &outcome.iteration {
        let _ = window.emit("iteration_created", iteration.id.clone());
    }
    let _ = window.emit("autonomy_ideas_changed", ());

    Ok(outcome)
}

/// 撤销上一次决策。
#[tauri::command]
pub async fn autonomy_undo_decision(
    idea_id: String,
    window: Window,
) -> Result<IdeaSummary, String> {
    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    let idea = controller.undo_decision(&idea_id).map_err(|e| e.to_string())?;
    let _ = window.emit("autonomy_ideas_changed", ());
    Ok(idea.to_summary())
}

/// 用户主动输入一条建议。
#[tauri::command]
pub async fn autonomy_propose_idea(
    request: UserIdeaRequest,
    window: Window,
) -> Result<IdeaSummary, String> {
    let scope = match request.scope.as_str() {
        "feature" => cowork_core::domain::idea::IdeaScope::Feature,
        "refactor" => cowork_core::domain::idea::IdeaScope::Refactor,
        "fix" => cowork_core::domain::idea::IdeaScope::Fix,
        "docs" => cowork_core::domain::idea::IdeaScope::Docs,
        "research" => cowork_core::domain::idea::IdeaScope::Research,
        other => return Err(format!("未知的影响范围: {}", other)),
    };
    let estimated_effort = match request.estimated_effort.as_str() {
        "s" => cowork_core::domain::idea::Effort::S,
        "m" => cowork_core::domain::idea::Effort::M,
        "l" => cowork_core::domain::idea::Effort::L,
        "xl" => cowork_core::domain::idea::Effort::Xl,
        other => return Err(format!("未知的工作量档位: {}", other)),
    };
    let priority = match request.priority.as_str() {
        "high" => cowork_core::data::models::Priority::High,
        "medium" => cowork_core::data::models::Priority::Medium,
        "low" => cowork_core::data::models::Priority::Low,
        other => return Err(format!("未知的优先级: {}", other)),
    };

    let evidence = request
        .evidence
        .iter()
        .map(|value| {
            serde_json::from_value(value.clone())
                .map_err(|e| format!("证据格式不正确: {}", e))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let draft = IdeaDraft {
        title: request.title,
        problem: request.problem,
        hypothesis: request.hypothesis,
        scope,
        evidence,
        estimated_effort,
        priority,
        expected_benefit: request.expected_benefit,
        conflicts_with: vec![],
    };

    let mut controller = AutonomyController::new().map_err(|e| e.to_string())?;
    let idea = controller.propose_user_idea(draft).map_err(|e| e.to_string())?;
    let _ = window.emit("autonomy_ideas_changed", ());
    Ok(idea.to_summary())
}

/// 预算总览。
#[tauri::command]
pub async fn autonomy_get_budget() -> Result<BudgetStatus, String> {
    let controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.budget_status().map_err(|e| e.to_string())
}

/// 历史调研轮次。
#[tauri::command]
pub async fn autonomy_list_cycles() -> Result<Vec<CycleSnapshot>, String> {
    let controller = AutonomyController::new().map_err(|e| e.to_string())?;
    controller.list_cycles().map_err(|e| e.to_string())
}

/// 迭代跑完后通知自主循环（事件触发）。
///
/// 是否真的跑一轮由 core 的 `should_trigger` 判定（配置里没开
/// `on_iteration_completed` 时会跳过），这里只负责把意图送到宿主。
pub fn notify_iteration_finished() {
    AUTONOMY_HOST.request_cycle(CycleTrigger::IterationCompleted);
}
