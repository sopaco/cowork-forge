// Execution exclusivity guard
//
// 流水线依赖若干进程级全局单例（当前迭代 ID、goto-stage 信号、执行用 LLM），
// 隐含"同一时刻只有一个迭代在执行"的假设。本模块把该假设显式化为执行互斥：
// 同一时刻只允许一个执行持有者，冲突时返回可读错误，而不是覆盖共享状态。
//
// 覆盖点（Phase 0）：
// - `IterationExecutor::execute`（continue/retry 委托它，一并覆盖）
// - PM 工具中会触碰全局单例/共享工程状态的操作（pm_goto_stage、pm_create_iteration）
//
// 自主循环宿主可通过 [`is_execution_in_progress`] 查询并在迭代执行时让路（D5）。

use std::sync::{Mutex, PoisonError};

#[derive(Debug, Clone)]
struct ExecutionOwner {
    holder: String,
    iteration_id: String,
}

static EXECUTION_OWNER: Mutex<Option<ExecutionOwner>> = Mutex::new(None);

/// Poison 容错的锁获取：持有者 panic 不应让后续执行永久卡死。
fn lock_owner() -> std::sync::MutexGuard<'static, Option<ExecutionOwner>> {
    EXECUTION_OWNER.lock().unwrap_or_else(PoisonError::into_inner)
}

/// 执行互斥的 RAII 守卫。持有期间其他执行尝试会被拒绝。
#[derive(Debug)]
pub struct ExecutionGuard {
    holder: String,
    iteration_id: String,
}

impl ExecutionGuard {
    /// 尝试为 `iteration_id` 获取执行槽。
    ///
    /// `holder` 描述获取者（如 `"pipeline"`、`"pm_agent"`），会出现在冲突错误里。
    /// 冲突时返回可读错误，不覆盖既有持有者状态。
    pub fn try_acquire(holder: &str, iteration_id: &str) -> anyhow::Result<Self> {
        let mut owner = lock_owner();
        if let Some(current) = owner.as_ref() {
            anyhow::bail!(
                "execution already in progress: iteration '{}' is being executed by '{}'; \
                 only one iteration may execute at a time. Wait for it to finish \
                 (or pause/stop it) and try again.",
                current.iteration_id,
                current.holder
            );
        }
        *owner = Some(ExecutionOwner {
            holder: holder.to_string(),
            iteration_id: iteration_id.to_string(),
        });
        Ok(Self {
            holder: holder.to_string(),
            iteration_id: iteration_id.to_string(),
        })
    }
}

impl Drop for ExecutionGuard {
    fn drop(&mut self) {
        let mut owner = lock_owner();
        let is_self = owner
            .as_ref()
            .is_some_and(|o| o.holder == self.holder && o.iteration_id == self.iteration_id);
        if is_self {
            *owner = None;
        }
    }
}

/// 是否有迭代执行正持有执行槽（自主循环据此让路）。
pub fn is_execution_in_progress() -> bool {
    lock_owner().is_some()
}

/// 当前执行持有者 `(holder, iteration_id)`（状态查询用）。
pub fn current_execution() -> Option<(String, String)> {
    lock_owner()
        .as_ref()
        .map(|o| (o.holder.clone(), o.iteration_id.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};

    // 执行槽是进程级单例：与所有会触碰该单例的测试共用同一把锁串行执行
    use crate::persistence::WORKSPACE_TEST_LOCK as TEST_LOCK;

    #[test]
    fn conflicting_acquire_is_rejected_with_readable_error() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        let guard = ExecutionGuard::try_acquire("pipeline", "iter-001").expect("first acquire");
        assert!(is_execution_in_progress());
        assert_eq!(
            current_execution(),
            Some(("pipeline".to_string(), "iter-001".to_string()))
        );

        let err = ExecutionGuard::try_acquire("pm_agent", "iter-002")
            .expect_err("second acquire must be rejected");
        let msg = format!("{:#}", err);
        assert!(msg.contains("iter-001"), "error should name the busy iteration: {}", msg);
        assert!(msg.contains("pipeline"), "error should name the holder: {}", msg);
        assert!(msg.contains("only one iteration"), "error should state the rule: {}", msg);

        drop(guard);
    }

    #[test]
    fn slot_is_released_after_drop() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        {
            let _guard = ExecutionGuard::try_acquire("pipeline", "iter-001").expect("acquire");
            assert!(is_execution_in_progress());
        }
        assert!(!is_execution_in_progress());

        let _again = ExecutionGuard::try_acquire("pipeline", "iter-001").expect("re-acquire");
    }

    #[test]
    fn concurrent_acquire_lets_exactly_one_winner() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        const THREADS: usize = 8;
        let barrier = Arc::new(Barrier::new(THREADS));
        let winners = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..THREADS)
            .map(|_i| {
                let barrier = barrier.clone();
                let winners = winners.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    if let Ok(_guard) = ExecutionGuard::try_acquire("pipeline", "iter-001") {
                        winners.fetch_add(1, Ordering::SeqCst);
                        // 持有片刻，放大竞争窗口
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread must not panic");
        }

        assert_eq!(winners.load(Ordering::SeqCst), 1);
        assert!(!is_execution_in_progress());
    }
}
