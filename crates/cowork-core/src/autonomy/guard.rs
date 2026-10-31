// CycleRunGuard —— 自主调研的进程内互斥
//
// 为什么需要它：`state.json` 里的 `researching` / `proposing` 是**持久化的
// 意图**，不是"这个进程里真的有一轮在跑"。用它做重入保护有两个致命问题：
//
// 1. 进程崩溃或被强杀时状态会永久停在 researching，下一轮永远被
//    `AlreadyRunning` 拒绝——自主循环就此死锁。
// 2. 宿主 tick 与人工「立即调研」各自 `AutonomyController::new()`，
//    读到的是同一份磁盘状态，谁也没有真正的锁，两个周期可以同时开跑。
//
// 所以重入保护交给进程内的 RAII 守卫（真相），磁盘状态只用于**展示**与
// **启动时的崩溃恢复**。进程挂了守卫自然消失，不存在残留。

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CycleOwner {
    cycle_id: String,
    trigger: &'static str,
    started_at: Instant,
}

static CYCLE_OWNER: Mutex<Option<CycleOwner>> = Mutex::new(None);

/// Poison 容错的锁获取：持有者 panic 不应让后续调研永久卡死。
fn lock_owner() -> std::sync::MutexGuard<'static, Option<CycleOwner>> {
    CYCLE_OWNER.lock().unwrap_or_else(PoisonError::into_inner)
}

/// 一轮调研的互斥守卫。持有期间其他周期会被拒绝。
#[derive(Debug)]
pub struct CycleRunGuard {
    cycle_id: String,
}

impl CycleRunGuard {
    /// 尝试为 `cycle_id` 占用调研槽。
    ///
    /// 已被占用时返回 `None`（调用方据此给出"上一轮还在跑"的跳过回执），
    /// 不覆盖既有持有者。
    pub fn try_acquire(cycle_id: &str, trigger: &'static str) -> Option<Self> {
        let mut owner = lock_owner();
        if owner.is_some() {
            return None;
        }
        *owner = Some(CycleOwner {
            cycle_id: cycle_id.to_string(),
            trigger,
            started_at: Instant::now(),
        });
        Some(Self {
            cycle_id: cycle_id.to_string(),
        })
    }
}

impl Drop for CycleRunGuard {
    fn drop(&mut self) {
        let mut owner = lock_owner();
        let is_self = owner.as_ref().is_some_and(|o| o.cycle_id == self.cycle_id);
        if is_self {
            *owner = None;
        }
    }
}

/// 是否有调研正持有槽位。
pub fn is_cycle_running() -> bool {
    lock_owner().is_some()
}

/// 当前调研持有者 `(cycle_id, trigger)`（状态查询与日志用）。
pub fn current_cycle() -> Option<(String, &'static str)> {
    lock_owner()
        .as_ref()
        .map(|o| (o.cycle_id.clone(), o.trigger))
}

/// 当前调研已运行多久（无调研时为 `None`）。
pub fn current_cycle_elapsed() -> Option<Duration> {
    lock_owner().as_ref().map(|o| o.started_at.elapsed())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::WORKSPACE_TEST_LOCK as TEST_LOCK;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};

    #[test]
    fn second_acquire_is_rejected_while_first_holds() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        let guard = CycleRunGuard::try_acquire("cycle-1", "manual").expect("first acquire");
        assert!(is_cycle_running());
        assert_eq!(
            current_cycle(),
            Some(("cycle-1".to_string(), "manual"))
        );

        assert!(
            CycleRunGuard::try_acquire("cycle-2", "schedule").is_none(),
            "同一进程内不允许两轮调研并行"
        );

        drop(guard);
    }

    #[test]
    fn slot_is_released_after_drop() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        {
            let _guard = CycleRunGuard::try_acquire("cycle-1", "manual").expect("acquire");
            assert!(is_cycle_running());
        }
        assert!(!is_cycle_running());
        assert!(current_cycle_elapsed().is_none());

        let _again = CycleRunGuard::try_acquire("cycle-2", "idle").expect("re-acquire");
    }

    #[test]
    fn concurrent_acquire_lets_exactly_one_winner() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        const THREADS: usize = 8;
        let barrier = Arc::new(Barrier::new(THREADS));
        let winners = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..THREADS)
            .map(|i| {
                let barrier = barrier.clone();
                let winners = winners.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    if let Some(_guard) =
                        CycleRunGuard::try_acquire(&format!("cycle-{}", i), "schedule")
                    {
                        winners.fetch_add(1, Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(20));
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread must not panic");
        }

        assert_eq!(winners.load(Ordering::SeqCst), 1);
        assert!(!is_cycle_running());
    }
}
