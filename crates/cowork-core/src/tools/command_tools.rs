use adk_rust::prelude::*;
use adk_rust::AdkError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;

// Import safety checker
use crate::verification::safety::{check_command_safety, SafetyCheckResult};

/// 通用命令执行参数
///
/// 设计目标：
/// - 通用（不绑定 rust/node/python）
/// - 简洁（只提供最必要的 cwd/env/timeout）
/// - 可观测（返回 stdout/stderr/exit_code）
/// - 安全（所有命令执行前通过安全检查，危险命令会被阻止）
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct RunCommandParams {
    /// 要执行的命令（shell 字符串），例如 "npm test" 或 "python -m pytest"
    pub cmd: String,

    /// 工作目录（可选）。为空则使用当前进程工作目录
    #[serde(default)]
    pub cwd: Option<String>,

    /// 环境变量（可选）
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,

    /// 超时时间毫秒（可选）。当前实现为 best-effort：仅在结果中回传，不强制 kill
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

pub struct CommandToolsBundle {
    pub run_command: Arc<FunctionTool>,
}

pub fn create_command_tools() -> CommandToolsBundle {
    let run_command = Arc::new(
        FunctionTool::new(
            "run_command",
            "Run a shell command and capture stdout/stderr/exit_code. Use for build/test/check verification. Commands are safety-checked before execution.",
            |_ctx, args| async move {
                let params: RunCommandParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                // Determine working directory for safety check
                let cwd = params.cwd.as_deref().unwrap_or(".");

                // ⚡ Safety check before execution
                match check_command_safety(&params.cmd, cwd) {
                    SafetyCheckResult::Blocked(reason) => {
                        tracing::error!("🚫 Command blocked by safety check: {} - Reason: {}", params.cmd, reason);
                        return Ok(json!({
                            "success": false,
                            "cmd": params.cmd,
                            "cwd": params.cwd,
                            "exit_code": -2,  // Special code for safety rejection
                            "stdout": "",
                            "stderr": format!("SAFETY CHECK FAILED: {}\nCommand was blocked and not executed.", reason),
                            "blocked": true,
                            "block_reason": reason
                        }));
                    }
                    SafetyCheckResult::Suspicious(reason) => {
                        tracing::warn!("⚠️  Suspicious command detected: {} - Reason: {}", params.cmd, reason);
                        // Continue execution but log warning
                    }
                    SafetyCheckResult::Safe => {
                        // Safe to proceed
                    }
                }

                let mut command = Command::new("sh");
                command.arg("-lc").arg(&params.cmd);

                if let Some(cwd) = &params.cwd {
                    command.current_dir(cwd);
                }

                if let Some(env) = &params.env {
                    command.envs(env);
                }

                // NOTE: 这里没有做真正的 timeout kill（需要 tokio + 子进程管理）。
                // 先保证接口通用，后续可以在不破坏 schema 的前提下增强实现。
                let output = command.output().map_err(|e| {
                    AdkError::Tool(format!("Failed to spawn command '{}': {}", params.cmd, e))
                })?;

                let exit_code = output.status.code().unwrap_or(-1);
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                Ok(json!({
                    "success": exit_code == 0,
                    "cmd": params.cmd,
                    "cwd": params.cwd,
                    "timeout_ms": params.timeout_ms,
                    "exit_code": exit_code,
                    "stdout": stdout,
                    "stderr": stderr,
                    "blocked": false
                }))
            },
        )
        .with_parameters_schema::<RunCommandParams>(),
    );

    CommandToolsBundle { run_command }
}
