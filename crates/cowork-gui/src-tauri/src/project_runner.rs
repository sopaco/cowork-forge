// Project runner for GUI
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::process::Child;
use tokio::sync::mpsc;
use tracing;

use crate::commands::path_utils;

// Import PreviewInfo from gui_types
use super::gui_types::PreviewInfo;

#[cfg(target_os = "windows")]
#[allow(unused_imports)]
use std::os::windows::process::CommandExt;

/// Kill a process and all its descendants (the entire process tree).
///
/// On Windows, uses `taskkill /T /F` which walks the process tree and force-kills
/// all descendants. This is critical because `child.kill()` only terminates the
/// direct child (e.g. `cmd.exe`), leaving the real dev server (`bun.exe`/`node.exe`)
/// orphaned and still holding the port.
///
/// On Unix, kills the entire process group. The child must have been spawned with
/// `process_group(0)` so that it leads its own group; the PGID equals the child PID.
fn kill_process_tree(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        // /T = kill tree (all descendants), /F = force
        let result = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        match result {
            Ok(status) if status.success() => {
                tracing::info!("[Runner] taskkill succeeded for PID {}", pid);
            }
            Ok(status) => {
                tracing::debug!(
                    "[Runner] taskkill for PID {} exited with non-success status: {:?}",
                    pid,
                    status
                );
            }
            Err(e) => {
                tracing::warn!("[Runner] taskkill failed for PID {}: {}", pid, e);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Kill the process group (negative PID targets the whole group).
        // SIGTERM first for graceful shutdown, then SIGKILL to force.
        let pgid = format!("-{}", pid);
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &pgid])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = std::process::Command::new("kill")
            .args(["-KILL", &pgid])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        // Fallback: kill the direct process too
        let _ = std::process::Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

pub struct ProjectRunner {
    processes: Arc<Mutex<HashMap<String, ProjectProcess>>>,
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

fn command_exists(cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("where")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("which")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

struct ProjectProcess {
    child: Child,
    /// OS-level PID of the spawned child. Used to kill the entire process tree
    /// (shell wrapper + dev server + any workers) on stop / exit.
    pid: u32,
    #[allow(dead_code)]
    output_tx: mpsc::UnboundedSender<String>,
    url: Option<String>,
    port: Option<u16>,
}

impl ProjectRunner {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        let mut app_handle_guard = self.app_handle.lock().unwrap();
        *app_handle_guard = Some(handle);
    }

    pub fn is_running(&self, iteration_id: &str) -> bool {
        let processes = self.processes.lock().unwrap();
        processes.contains_key(iteration_id)
    }

    pub async fn start(
        &self,
        iteration_id: String,
        command: String,
        code_dir: String,
        url: Option<String>,
        port: Option<u16>,
    ) -> Result<u32, String> {
        // Stop existing process if any
        if let Ok(()) = self.stop(iteration_id.clone()).await {
            tracing::info!(
                "[Runner] Stopped existing process for iteration: {}",
                iteration_id
            );
        }

        // Use provided code_dir instead of hardcoded workspace path
        let code_path = std::path::Path::new(&code_dir);

        if !code_path.exists() {
            return Err(format!("Code directory not found: {}", code_dir));
        }

        // Debug: Print PATH and check commands
        let path_env = std::env::var("PATH").unwrap_or_else(|_| "PATH not found".to_string());
        tracing::debug!("[Runner] PATH = {}", path_env);

        // Check if bun or sh exists
        tracing::debug!("[Runner] Checking commands...");
        if command_exists("bun") {
            tracing::debug!("[Runner] bun found");
        } else {
            tracing::debug!("[Runner] bun NOT found");
        }
        if command_exists("sh") {
            tracing::debug!("[Runner] sh found");
        } else {
            tracing::debug!("[Runner] sh NOT found");
        }

        tracing::info!("[Runner] Starting command: {} in {}", command, code_dir);

        let normalized_command =
            path_utils::normalize_project_start_command(code_path, &command);
        let resolved_command = path_utils::resolve_command(&normalized_command);
        if !path_utils::is_runnable_external_command(&resolved_command) {
            return Err(format!(
                "Invalid or empty start command: {:?}. Check project runtime configuration.",
                command
            ));
        }

        #[cfg(target_os = "windows")]
        let mut child = {
            let mut cmd = tokio::process::Command::new("cmd");
            cmd.args(["/C", &resolved_command])
                .current_dir(&code_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .creation_flags(0x08000000); // CREATE_NO_WINDOW
            path_utils::apply_gui_child_env_async(&mut cmd);

            cmd.spawn().map_err(|e| format!("Failed to start: {}", e))?
        };

        #[cfg(not(target_os = "windows"))]
        let mut child = {
            // Spawn the child in its own process group so that `kill_process_tree`
            // can kill the entire group (dev server + any workers) on stop/exit.
            if let Some((program, args)) = path_utils::parse_direct_command(&resolved_command) {
                tracing::info!(
                    "[Runner] Spawning directly: {:?} {:?}",
                    program,
                    args
                );
                let mut cmd = tokio::process::Command::new(&program);
                cmd.args(&args)
                    .current_dir(&code_path)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .process_group(0);
                path_utils::apply_gui_child_env_async(&mut cmd);
                cmd.spawn().map_err(|e| format!("Failed to start: {}", e))?
            } else {
                // Compound commands only: login shell without -i (non-TTY safe).
                // `exec` replaces the shell so we monitor the real dev-server process.
                let shell = path_utils::command_shell();
                let shell_script = format!("exec {resolved_command}");
                tracing::info!(
                    "[Runner] Spawning via shell: {} -lc {}",
                    shell,
                    shell_script
                );
                let mut cmd = tokio::process::Command::new(&shell);
                cmd.args(["-lc", &shell_script])
                    .current_dir(&code_path)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .process_group(0);
                path_utils::apply_gui_child_env_async(&mut cmd);
                cmd.spawn().map_err(|e| format!("Failed to start: {}", e))?
            }
        };

        let pid = child.id().unwrap();

        // Get app handle for event emission - create multiple clones upfront
        let app_handle_opt = self.app_handle.lock().unwrap().clone();
        let app_handle_stdout = app_handle_opt.clone();
        let app_handle_stderr = app_handle_opt.clone();
        let app_handle_exit = app_handle_opt.clone();
        let iteration_id_clone = iteration_id.clone();

        // Create channels for output
        let (stdout_tx, _stdout_rx) = mpsc::unbounded_channel();
        let (stderr_tx, _stderr_rx) = mpsc::unbounded_channel();

        // Clone for stdout task
        let iteration_id_stdout = iteration_id_clone.clone();

        // Check if process exited immediately (command error detection)
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        match child.try_wait() {
            Ok(Some(status)) => {
                let mut output_preview = String::new();
                {
                    use tokio::io::AsyncReadExt;
                    if let Some(mut stdout) = child.stdout.take() {
                        let mut stdout_buf = vec![0u8; 4096];
                        if let Ok(n) = stdout.read(&mut stdout_buf).await {
                            if n > 0 {
                                output_preview
                                    .push_str(&String::from_utf8_lossy(&stdout_buf[..n]));
                            }
                        }
                    }
                    if let Some(mut stderr) = child.stderr.take() {
                        let mut stderr_buf = vec![0u8; 4096];
                        if let Ok(n) = stderr.read(&mut stderr_buf).await {
                            if n > 0 {
                                if !output_preview.is_empty() {
                                    output_preview.push('\n');
                                }
                                output_preview
                                    .push_str(&String::from_utf8_lossy(&stderr_buf[..n]));
                            }
                        }
                    }
                }

                tracing::warn!(
                    "[Runner] Process exited immediately with status: {:?}, output: {}",
                    status,
                    output_preview
                );

                let hint = if status.success() {
                    "The dev process exited immediately. Check package.json scripts and that dependencies are installed."
                } else {
                    "The command failed to start. Check that bun/npm/cargo is installed and the project directory is correct."
                };

                return Err(format!(
                    "Command failed immediately. Exit status: {}. {}\nCommand: {}\n{}",
                    status,
                    hint,
                    resolved_command,
                    if output_preview.is_empty() {
                        String::new()
                    } else {
                        format!("Output:\n{output_preview}")
                    }
                ));
            }
            Ok(None) => {
                // Process is still running - good
            }
            Err(e) => {
                tracing::error!("[Runner] Error checking process status: {}", e);
            }
        }

        // Clone child for stdout/stderr reading (only after liveness check passes)
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Clone senders for spawn tasks
        let stdout_tx_spawn = stdout_tx.clone();
        let stderr_tx_spawn = stderr_tx.clone();

        // Spawn task to read stdout and emit events
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            loop {
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let _ = stdout_tx_spawn.send(line.clone());

                        // Emit event to frontend (use expected event names)
                        if let Some(ref handle) = app_handle_stdout {
                            if let Err(e) = handle.emit(
                                "project_log",
                                serde_json::json!({
                                    "iteration_id": iteration_id_stdout,
                                    "session_id": iteration_id_stdout,
                                    "stream": "stdout",
                                    "content": line.clone()
                                }),
                            ) {
                                tracing::warn!("[Runner] Failed to emit project_log event: {}", e);
                            }
                        }

                        line.clear();
                    }
                    Err(e) => {
                        tracing::error!("[Runner] Error reading stdout: {}", e);

                        // Emit error event (use expected event name)
                        if let Some(ref handle) = app_handle_stdout {
                            if let Err(e) = handle.emit(
                                "project_log",
                                serde_json::json!({
                                    "iteration_id": iteration_id_stdout,
                                    "session_id": iteration_id_stdout,
                                    "stream": "stderr",
                                    "content": format!("Error reading output: {}\n", e)
                                }),
                            ) {
                                tracing::warn!("[Runner] Failed to emit project_log event: {}", e);
                            }
                        }
                        break;
                    }
                }
            }
        });

        // Spawn task to read stderr and emit events
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();

            loop {
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let _ = stderr_tx_spawn.send(line.clone());

                        // Emit event to frontend (use expected event names)
                        if let Some(ref handle) = app_handle_stderr {
                            if let Err(e) = handle.emit(
                                "project_log",
                                serde_json::json!({
                                    "iteration_id": iteration_id_clone,
                                    "session_id": iteration_id_clone,
                                    "stream": "stderr",
                                    "content": line.clone()
                                }),
                            ) {
                                tracing::warn!("[Runner] Failed to emit project_log event: {}", e);
                            }
                        }

                        line.clear();
                    }
                    Err(e) => {
                        tracing::error!("[Runner] Error reading stderr: {}", e);

                        // Emit error event
                        if let Some(ref handle) = app_handle_stderr {
                            if let Err(emit_err) = handle.emit(
                                "process_error",
                                serde_json::json!({
                                    "iteration_id": iteration_id_clone,
                                    "error": e.to_string()
                                }),
                            ) {
                                tracing::warn!(
                                    "[Runner] Failed to emit process_error event: {}",
                                    emit_err
                                );
                            }
                        }
                        break;
                    }
                }
            }
        });

        let mut processes = self.processes.lock().unwrap();
        processes.insert(
            iteration_id.clone(),
            ProjectProcess {
                child,
                pid,
                output_tx: stdout_tx,
                url,
                port,
            },
        );
        drop(processes);

        // Spawn task to wait for process exit and emit stopped event
        // Note: We DON'T remove the process here - we keep it in the map so is_running() works correctly
        // The process will be removed when:
        // 1. stop() is called explicitly, or
        // 2. The process exits naturally (we'll detect this by checking child.try_wait())
        let iteration_id_exit = iteration_id.clone();
        let processes_ref = Arc::clone(&self.processes);
        let app_handle_for_cleanup = app_handle_exit.clone();

        tokio::spawn(async move {
            // Periodically check if process has exited
            loop {
                // Check if process still exists in map
                let should_check = {
                    let procs = processes_ref.lock().unwrap();
                    procs.contains_key(&iteration_id_exit)
                };
                
                if !should_check {
                    // Process was removed by stop(), exit the loop
                    break;
                }
                
                // Try to check if process has exited (non-blocking)
                let exited = {
                    let mut procs = processes_ref.lock().unwrap();
                    if let Some(proc) = procs.get_mut(&iteration_id_exit) {
                        // Try non-blocking wait to check if process has exited
                        match proc.child.try_wait() {
                            Ok(Some(status)) => {
                                // Process has exited, remove it
                                tracing::info!("[Runner] Process {} exited with status: {:?}", iteration_id_exit, status);
                                procs.remove(&iteration_id_exit);
                                true
                            }
                            Ok(None) => false, // Still running
                            Err(e) => {
                                tracing::error!("[Runner] Error checking process status: {}", e);
                                false
                            }
                        }
                    } else {
                        true // Process not in map
                    }
                };
                
                if exited {
                    // Emit stopped event
                    if let Some(ref handle) = app_handle_for_cleanup {
                        let _ = handle.emit(
                            "project_stopped",
                            serde_json::json!({
                                "iteration_id": iteration_id_exit,
                                "session_id": iteration_id_exit
                            }),
                        );
                    }
                    break;
                }
                
                // Wait a bit before next check
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });

        tracing::info!("[Runner] Process started with PID: {}", pid);
        Ok(pid)
    }

    pub async fn stop(&self, iteration_id: String) -> Result<(), String> {
        // Remove process from map and release lock before await
        let process = {
            let mut processes = self.processes.lock().unwrap();
            processes.remove(&iteration_id)
        };

        if let Some(mut process) = process {
            tracing::info!(
                "[Runner] Stopping process tree for iteration: {} (PID: {})",
                iteration_id,
                process.pid
            );

            // Kill the entire process tree (shell wrapper + dev server + workers).
            // This is critical: `child.kill()` only terminates the direct child
            // (e.g. `cmd.exe`), leaving the real dev server orphaned and still
            // holding the port.
            kill_process_tree(process.pid);

            // Also call child.kill() as a fallback for the direct child.
            let _ = process.child.kill().await;

            // Emit stopped event (use expected event name)
            if let Some(ref handle) = *self.app_handle.lock().unwrap() {
                let _ = handle.emit(
                    "project_stopped",
                    serde_json::json!({
                        "iteration_id": iteration_id,
                        "session_id": iteration_id
                    }),
                );
            }

            tracing::info!("[Runner] Process stopped");
            Ok(())
        } else {
            // Process already stopped or not found - this is fine, just return success
            tracing::debug!(
                "[Runner] No running process found for iteration: {} (may already be stopped)",
                iteration_id
            );
            Ok(())
        }
    }

    /// Synchronously stop ALL running project processes.
    ///
    /// Intended to be called from `RunEvent::Exit` (where the async runtime may
    /// no longer be usable). Kills every tracked process tree and clears the map.
    /// Does NOT emit frontend events (the frontend is going away).
    pub fn stop_all_sync(&self) {
        let entries: Vec<(String, u32)> = {
            let mut processes = self.processes.lock().unwrap();
            let entries: Vec<(String, u32)> = processes
                .iter()
                .map(|(k, v)| (k.clone(), v.pid))
                .collect();
            processes.clear();
            entries
        };

        if entries.is_empty() {
            return;
        }

        tracing::info!(
            "[Runner] stop_all_sync: killing {} process tree(s)",
            entries.len()
        );

        for (iteration_id, pid) in &entries {
            tracing::info!(
                "[Runner] stop_all_sync: killing PID {} for iteration {}",
                pid,
                iteration_id
            );
            kill_process_tree(*pid);
        }
    }

    pub fn get_info(&self, iteration_id: &str) -> Option<PreviewInfo> {
        let processes = self.processes.lock().unwrap();
        if let Some(process) = processes.get(iteration_id) {
            // Return the actual URL and port stored in the process
            if let (Some(url), Some(port)) = (&process.url, process.port) {
                Some(PreviewInfo {
                    url: url.clone(),
                    port,
                    status: super::gui_types::PreviewStatus::Running,
                    project_type: super::gui_types::ProjectType::Unknown,
                })
            } else {
                // Fallback to default if URL/port not set
                None
            }
        } else {
            None
        }
    }
}
