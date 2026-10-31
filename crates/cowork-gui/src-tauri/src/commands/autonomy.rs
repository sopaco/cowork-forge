// Autonomy commands —— 自主模式 Tauri 命令层（薄壳）
//
// 只转发 core AutonomyController API + 驱动 AUTONOMY_HOST + 事件推送，
// 零业务逻辑。

use serde::Serialize;
use tauri::{Emitter, Window};

use cowork_core::autonomy::{AutonomyConfig, AutonomyController, AutonomyState};

use crate::commands::AUTONOMY_HOST;

/// `autonomy_get_status` 返回的状态快照。
#[derive(Debug, Clone, Serialize)]
pub struct AutonomyStatusPayload {
    pub state: AutonomyState,
    pub config: AutonomyConfig,
    pub host_running: bool,
    pub host_paused: bool,
}

fn build_payload(controller: &AutonomyController) -> Result<AutonomyStatusPayload, String> {
    let state = controller.status().map_err(|e| e.to_string())?;
    let config = controller.config().map_err(|e| e.to_string())?;
    Ok(AutonomyStatusPayload {
        state,
        config,
        host_running: AUTONOMY_HOST.is_running(),
        host_paused: AUTONOMY_HOST.is_paused(),
    })
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
    let controller = AutonomyController::new().map_err(|e| e.to_string())?;
    build_payload(&controller)
}
