// AutonomyHost —— 自主循环常驻任务宿主（GUI 侧薄壳）
//
// 只负责常驻 tokio 任务的生命周期（start/stop/pause/resume），
// 不承载任何业务逻辑；业务接入点在循环体内以注释标注（Phase 1）。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::sync::watch;
use tokio::task::JoinHandle;

struct HostInner {
    join_handle: Option<JoinHandle<()>>,
    stop_tx: Option<watch::Sender<bool>>,
    paused: Arc<AtomicBool>,
}

pub struct AutonomyHost {
    inner: Mutex<HostInner>,
}

impl AutonomyHost {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HostInner {
                join_handle: None,
                stop_tx: None,
                paused: Arc::new(AtomicBool::new(false)),
            }),
        }
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
        let join_handle = tokio::spawn(run_loop(stop_rx, paused));
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

/// 常驻循环骨架：30s tick + watch 停止信号。
async fn run_loop(mut stop_rx: watch::Receiver<bool>, paused: Arc<AtomicBool>) {
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
                // Phase 1 接入点（Idea 聚合/提案/调研）：本期循环体为空
            }
        }
    }
}
