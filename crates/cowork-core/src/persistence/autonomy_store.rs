// Autonomy store —— 自主模式 config/state 的版本化持久化
//
// 文件缺失返回 Default（创建语义），版本无法识别由 schema helper 显式报错。

use std::path::PathBuf;

use crate::autonomy::config::AutonomyConfig;
use crate::autonomy::state::AutonomyState;

use super::{from_versioned_json, get_cowork_dir, to_versioned_json};

/// Autonomy store for persistence
pub struct AutonomyStore;

impl AutonomyStore {
    pub fn new() -> Self {
        Self
    }

    /// Load autonomy config (missing file => default)
    pub fn load_config(&self) -> anyhow::Result<AutonomyConfig> {
        let path = self.config_file_path()?;
        if !path.exists() {
            return Ok(AutonomyConfig::default());
        }
        let content = std::fs::read_to_string(&path)?;
        from_versioned_json(&content, &path.display().to_string())
    }

    /// Save autonomy config to disk
    pub fn save_config(&self, config: &AutonomyConfig) -> anyhow::Result<()> {
        let path = self.config_file_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = to_versioned_json(config)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Load autonomy state (missing file => default)
    pub fn load_state(&self) -> anyhow::Result<AutonomyState> {
        let path = self.state_file_path()?;
        if !path.exists() {
            return Ok(AutonomyState::default());
        }
        let content = std::fs::read_to_string(&path)?;
        from_versioned_json(&content, &path.display().to_string())
    }

    /// Save autonomy state to disk
    pub fn save_state(&self, state: &AutonomyState) -> anyhow::Result<()> {
        let path = self.state_file_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = to_versioned_json(state)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn config_file_path(&self) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("autonomy").join("config.json"))
    }

    fn state_file_path(&self) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("autonomy").join("state.json"))
    }
}

impl Default for AutonomyStore {
    fn default() -> Self {
        Self::new()
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
    fn missing_files_return_defaults() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        let store = AutonomyStore::new();
        assert_eq!(store.load_config().expect("load config"), AutonomyConfig::default());
        assert_eq!(store.load_state().expect("load state"), AutonomyState::default());
    }

    #[test]
    fn save_and_load_round_trip() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        let store = AutonomyStore::new();

        let mut config = AutonomyConfig::default();
        config.enabled = true;
        config.triggers.on_schedule = true;
        config.triggers.schedule_interval_mins = 15;
        config.budget.max_cycles_per_day = 5;
        config.research.external_mcp = crate::autonomy::config::ExternalMcpPolicy::Off;
        store.save_config(&config).expect("save config");
        assert_eq!(store.load_config().expect("load config"), config);

        let mut state = AutonomyState::default();
        state.status = crate::autonomy::state::AutonomyStatus::Tripped;
        state.cycle_count = 7;
        state.last_cycle_at = Some(chrono::Utc::now());
        state.tripped_reason = Some("budget exhausted".to_string());
        store.save_state(&state).expect("save state");
        assert_eq!(store.load_state().expect("load state"), state);
    }

    #[test]
    fn unknown_schema_version_is_rejected() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        let autonomy_dir = temp.path().join(".cowork-v2").join("autonomy");
        std::fs::create_dir_all(&autonomy_dir).expect("create dir");

        // 坏版本（未来版本）→ 显式报错
        std::fs::write(
            autonomy_dir.join("config.json"),
            r#"{"schema_version":99,"enabled":true,"triggers":{"on_iteration_completed":false,"on_idle":false,"on_schedule":false,"schedule_interval_mins":60},"research":{"local":true,"memory":true,"feedback":true,"external_mcp":"as_needed"},"budget":{"max_llm_calls_per_cycle":10,"max_cycles_per_day":20,"max_mcp_calls_per_cycle":3},"auto_create_iteration":false}"#,
        )
        .expect("write bad config");
        let err = AutonomyStore::new()
            .load_config()
            .expect_err("unsupported schema_version must be rejected");
        let msg = format!("{:#}", err);
        assert!(msg.contains("unsupported schema_version 99"), "unexpected error: {}", msg);
        assert!(msg.contains("config.json"), "error should name the source: {}", msg);

        // 缺失版本字段 → 显式报错
        std::fs::write(autonomy_dir.join("state.json"), r#"{"status":"disabled","cycle_count":0,"last_cycle_at":null,"tripped_reason":null}"#)
            .expect("write unversioned state");
        let err = AutonomyStore::new()
            .load_state()
            .expect_err("missing schema_version must be rejected");
        let msg = format!("{:#}", err);
        assert!(msg.contains("missing schema_version"), "unexpected error: {}", msg);
        assert!(msg.contains("state.json"), "error should name the source: {}", msg);
    }
}
