// AutonomyController —— 自主模式生命周期状态机
//
// 只负责 enable/pause/resume/stop/status 的迁移校验与落盘，
// 不实现 run_cycle/decide_idea 等 Phase 1 业务逻辑。

use crate::autonomy::config::AutonomyConfig;
use crate::autonomy::state::{AutonomyState, AutonomyStatus};
use crate::persistence::AutonomyStore;

/// 自主模式生命周期控制器。
///
/// 构造时从磁盘读取现有 config/state（文件缺失时为 [`Default`]），
/// 迁移成功后立即落盘。
pub struct AutonomyController {
    store: AutonomyStore,
    config: AutonomyConfig,
    state: AutonomyState,
}

impl AutonomyController {
    /// 读取现有 config/state 构造控制器。
    pub fn new() -> anyhow::Result<Self> {
        let store = AutonomyStore::new();
        let config = store.load_config()?;
        let state = store.load_state()?;
        Ok(Self { store, config, state })
    }

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

    /// 当前状态快照。
    pub fn status(&self) -> anyhow::Result<AutonomyState> {
        Ok(self.state.clone())
    }

    /// 当前配置快照。
    pub fn config(&self) -> anyhow::Result<AutonomyConfig> {
        Ok(self.config.clone())
    }

    fn persist(&self) -> anyhow::Result<()> {
        self.store.save_config(&self.config)?;
        self.store.save_state(&self.state)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // workspace 全局路径是进程级单例，所有涉及 workspace 的单测共用同一把锁串行执行
    use crate::persistence::WORKSPACE_TEST_LOCK as TEST_LOCK;
    use std::sync::PoisonError;
    use tempfile::TempDir;

    #[test]
    fn full_lifecycle_chain_enable_pause_resume_stop() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        let mut controller = AutonomyController::new().expect("construct");
        assert_eq!(controller.status().expect("status").status, AutonomyStatus::Disabled);

        controller.enable().expect("enable");
        assert_eq!(controller.status().expect("status").status, AutonomyStatus::Idle);
        assert!(controller.config().expect("config").enabled);

        controller.pause().expect("pause");
        assert_eq!(controller.status().expect("status").status, AutonomyStatus::Paused);

        controller.resume().expect("resume");
        assert_eq!(controller.status().expect("status").status, AutonomyStatus::Idle);

        controller.stop().expect("stop");
        assert_eq!(controller.status().expect("status").status, AutonomyStatus::Disabled);
        assert!(!controller.config().expect("config").enabled);
    }

    #[test]
    fn illegal_transitions_report_current_status() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        let mut controller = AutonomyController::new().expect("construct");

        // Disabled 时 pause 非法
        let err = controller.pause().expect_err("pause while disabled must fail");
        let msg = format!("{:#}", err);
        assert!(msg.contains("disabled"), "error should contain current status: {}", msg);

        // Disabled 时 resume 非法
        let err = controller.resume().expect_err("resume while disabled must fail");
        let msg = format!("{:#}", err);
        assert!(msg.contains("disabled"), "error should contain current status: {}", msg);

        // Paused 时再次 pause 非法
        controller.enable().expect("enable");
        controller.pause().expect("pause");
        let err = controller.pause().expect_err("second pause must fail");
        let msg = format!("{:#}", err);
        assert!(msg.contains("paused"), "error should contain current status: {}", msg);
    }

    #[test]
    fn config_and_state_round_trip_through_store() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        {
            let mut controller = AutonomyController::new().expect("construct");
            controller.enable().expect("enable");
            controller.pause().expect("pause");
            let saved_state = controller.status().expect("status");
            let saved_config = controller.config().expect("config");

            // save 后重新 load，应与保存值相等
            let reloaded = AutonomyController::new().expect("reload");
            assert_eq!(reloaded.status().expect("status"), saved_state);
            assert_eq!(reloaded.config().expect("config"), saved_config);
            assert_eq!(saved_state.status, AutonomyStatus::Paused);
            assert!(saved_config.enabled);
        }

        // 落盘文件位于 .cowork-v2/autonomy/ 下
        let autonomy_dir = temp.path().join(".cowork-v2").join("autonomy");
        assert!(autonomy_dir.join("config.json").exists());
        assert!(autonomy_dir.join("state.json").exists());
    }
}
