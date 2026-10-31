// AutonomyHost —— 自主循环常驻任务宿主（GUI 侧薄壳）
//
// 职责：常驻 tokio 任务的生命周期（start/stop/pause/resume）+ 触发调度。
// 业务逻辑全部在 `cowork_core::autonomy::AutonomyController` 里，
// 这里只负责"什么时候该跑一轮"与"把进度推给前端"。

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use cowork_core::autonomy::{AutonomyController, CycleProgressSink, CycleSnapshot, CycleStep, CycleTrigger};
use tauri::Emitter;
use tokio::sync::watch;
use tokio::task::JoinHandle;

/// 触发来源的编码（存进 AtomicU8）。
const TRIGGER_NONE: u8 = 0;
const TRIGGER_ITERATION_COMPLETED: u8 = 1;
const TRIGGER_SCHEDULE: u8 = 2;

struct HostInner {
    join_handle: Option<JoinHandle<()>>,
    stop_tx: Option<watch::Sender<bool>>,
    paused: Arc<AtomicBool>,
    app_handle: Option<tauri::AppHandle>,
}

pub struct AutonomyHost {
    inner: Mutex<HostInner>,
    /// 待处理的触发来源（0 = 无），与常驻任务共享
    pending_trigger: Arc<AtomicU8>,
}

impl AutonomyHost {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HostInner {
                join_handle: None,
                stop_tx: None,
                paused: Arc::new(AtomicBool::new(false)),
                app_handle: None,
            }),
            pending_trigger: Arc::new(AtomicU8::new(TRIGGER_NONE)),
        }
    }

    /// 绑定 AppHandle（用于把进度事件推给前端）。
    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.app_handle = Some(handle);
    }

    /// 请求在下一个 tick 跑一轮调研（用于迭代完成等事件触发）。
    pub fn request_cycle(&self, trigger: CycleTrigger) {
        let code = match trigger {
            CycleTrigger::IterationCompleted => TRIGGER_ITERATION_COMPLETED,
            _ => TRIGGER_SCHEDULE,
        };
        // 已有待处理的触发时不覆盖（迭代完成优先于定时）
        let _ = self.pending_trigger.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |current| {
                if current == TRIGGER_NONE {
                    Some(code)
                } else {
                    None
                }
            },
        );
    }

    /// 启动常驻循环。已在运行时返回可读错误，不重复启动。
    pub fn start(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        if inner.join_handle.is_some() {
            anyhow::bail!("autonomy host is already running");
        }
        let (stop_tx, stop_rx) = watch::channel(false);
        let paused = Arc::clone(&inner.paused);
        paused.store(false, Ordering::SeqCst);
        let app_handle = inner.app_handle.clone();
        let pending_trigger = Arc::clone(&self.pending_trigger);
        let join_handle = tokio::spawn(run_loop(stop_rx, paused, app_handle, pending_trigger));
        inner.stop_tx = Some(stop_tx);
        inner.join_handle = Some(join_handle);
        Ok(())
    }

    /// 通知循环退出并回收任务句柄。
    pub async fn stop(&self) -> anyhow::Result<()> {
        let (stop_tx, join_handle) = {
            let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
            (inner.stop_tx.take(), inner.join_handle.take())
        };
        if let Some(tx) = stop_tx {
            // 接收端已退出时 send 报错，忽略即可（循环已经不在了）
            let _ = tx.send(true);
        }
        if let Some(handle) = join_handle {
            // 合作式退出：等待循环响应停止信号后回收，不 abort、不 panic
            if handle.await.is_err() {
                tracing::warn!("[autonomy] host loop task ended unexpectedly");
            }
        }
        Ok(())
    }

    /// 暂停循环（tick 跳过，任务保持驻留）。
    pub fn pause(&self) -> anyhow::Result<()> {
        let inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.paused.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// 恢复循环。
    pub fn resume(&self) -> anyhow::Result<()> {
        let inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.paused.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// 常驻循环是否在运行。
    pub fn is_running(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .join_handle
            .is_some()
    }

    /// 循环是否处于暂停。
    pub fn is_paused(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .paused
            .load(Ordering::SeqCst)
    }
}

impl Default for AutonomyHost {
    fn default() -> Self {
        Self::new()
    }
}

/// 把 core 的步骤进度转成 Tauri 事件。
struct TauriCycleSink {
    app_handle: tauri::AppHandle,
}

impl CycleProgressSink for TauriCycleSink {
    fn on_step(&self, cycle_id: &str, step: &CycleStep) {
        let _ = self.app_handle.emit(
            "autonomy_cycle_step",
            serde_json::json!({
                "cycle_id": cycle_id,
                "step": step,
            }),
        );
    }

    fn on_snapshot(&self, snapshot: &CycleSnapshot) {
        let _ = self.app_handle.emit(
            "autonomy_cycle_snapshot",
            serde_json::json!({ "snapshot": snapshot }),
        );
    }
}

/// 常驻循环骨架：30s tick + watch 停止信号。
async fn run_loop(
    mut stop_rx: watch::Receiver<bool>,
    paused: Arc<AtomicBool>,
    app_handle: Option<tauri::AppHandle>,
    pending_trigger: Arc<AtomicU8>,
) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    loop {
        tokio::select! {
            _ = stop_rx.changed() => {
                // 收到停止信号（或发送端被回收），退出常驻循环
                tracing::debug!("[autonomy] stop signal received, host loop exiting");
                break;
            }
            _ = ticker.tick() => {
                if paused.load(Ordering::SeqCst) {
                    tracing::debug!("[autonomy] paused, skip this tick");
                    continue;
                }
                if cowork_core::execution_guard::is_execution_in_progress() {
                    // 执行优先（D5）：迭代执行中自主循环让路
                    tracing::debug!("[autonomy] iteration execution in progress, yielding this tick");
                    continue;
                }

                // ① 事件触发（迭代完成等）优先
                let pending = pending_trigger.swap(TRIGGER_NONE, Ordering::SeqCst);
                if pending != TRIGGER_NONE {
                    let trigger = if pending == TRIGGER_ITERATION_COMPLETED {
                        CycleTrigger::IterationCompleted
                    } else {
                        CycleTrigger::Schedule
                    };
                    if let Some(handle) = app_handle.clone() {
                        run_cycle_with_trigger(handle, trigger).await;
                    }
                    continue;
                }

                // ② 定时触发：到点且配置开启就跑一轮
                let schedule_due = AutonomyController::new()
                    .map(|c| c.should_trigger(CycleTrigger::Schedule).should_run)
                    .unwrap_or(false);
                if schedule_due {
                    if let Some(handle) = app_handle.clone() {
                        run_cycle_with_trigger(handle, CycleTrigger::Schedule).await;
                    }
                    continue;
                }

                // ③ 空闲触发：没有别的触发源但配置开启了空闲调研
                let idle_due = AutonomyController::new()
                    .map(|c| c.should_trigger(CycleTrigger::Idle).should_run)
                    .unwrap_or(false);
                if idle_due
                    && let Some(handle) = app_handle.clone()
                {
                    run_cycle_with_trigger(handle, CycleTrigger::Idle).await;
                }
            }
        }
    }
}

/// 跑一轮调研（触发来源可指定）。
async fn run_cycle_with_trigger(app_handle: tauri::AppHandle, trigger: CycleTrigger) {
    let config = match cowork_core::llm::load_config() {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("[autonomy] 读取 LLM 配置失败: {}", e);
            return;
        }
    };
    let model = match cowork_core::llm::create_llm_client(&config.llm) {
        Ok(model) => model,
        Err(e) => {
            tracing::warn!("[autonomy] 创建 LLM 客户端失败: {}", e);
            return;
        }
    };

    let sink = TauriCycleSink {
        app_handle: app_handle.clone(),
    };
    let mut controller = match AutonomyController::new() {
        Ok(controller) => controller,
        Err(e) => {
            tracing::warn!("[autonomy] 读取自主模式状态失败: {}", e);
            return;
        }
    };

    match controller.run_cycle(trigger, model, &sink).await {
        Ok(report) => {
            let _ = app_handle.emit(
                "autonomy_cycle_report",
                serde_json::json!({
                    "report": report,
                    "trigger": trigger.as_str(),
                }),
            );
        }
        Err(e) => {
            tracing::warn!("[autonomy] 调研失败: {}", e);
        }
    }
}
