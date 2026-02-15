// Project runner for GUI
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::process::Child;
use tokio::sync::mpsc;

// Import PreviewInfo from gui_types
use super::gui_types::PreviewInfo;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct ProjectRunner {
    processes: Arc<Mutex<HashMap<String, ProjectProcess>>>,
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

struct ProjectProcess {
    child: Child,
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
            println!(
                "[Runner] Stopped existing process for iteration: {}",
                iteration_id
            );
        }

        // Use provided code_dir instead of hardcoded workspace path
        let code_path = std::path::Path::new(&code_dir);

        if !code_path.exists() {
            return Err(format!("Code directory not found: {}", code_dir));
        }

        println!("[Runner] Starting command: {} in {}", command, code_dir);

        #[cfg(target_os = "windows")]
        let mut child = {
            let mut cmd = tokio::process::Command::new("cmd");
            cmd.args(["/C", &command])
                .current_dir(&code_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .creation_flags(0x08000000); // CREATE_NO_WINDOW

            cmd.spawn().map_err(|e| format!("Failed to start: {}", e))?
        };

        #[cfg(not(target_os = "windows"))]
        let mut child = Command::new("sh")
            .args(["-c", &command])
            .current_dir(&code_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start: {}", e))?;

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

        // Clone child for stdout reading
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Clone senders for spawn tasks
        let stdout_tx_spawn = stdout_tx.clone();
        let stderr_tx_spawn = stderr_tx.clone();

        // Clone for stdout task
        let iteration_id_stdout = iteration_id_clone.clone();
        
        // Check if process exited immediately (command error detection)
        // Give it a brief moment to potentially fail
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process already exited - command likely failed
                println!("[Runner] Process exited immediately with status: {:?}", status);
                return Err(format!(
                    "Command failed immediately. Exit status: {}. Check if the command is correct.",
                    status
                ));
            }
            Ok(None) => {
                // Process is still running - good
            }
            Err(e) => {
                eprintln!("[Runner] Error checking process status: {}", e);
            }
        }

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
                                eprintln!("[Runner] Failed to emit project_log event: {}", e);
                            }
                        }

                        line.clear();
                    }
                    Err(e) => {
                        eprintln!("[Runner] Error reading stdout: {}", e);

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
                                eprintln!("[Runner] Failed to emit project_log event: {}", e);
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
                                eprintln!("[Runner] Failed to emit project_log event: {}", e);
                            }
                        }

                        line.clear();
                    }
                    Err(e) => {
                        eprintln!("[Runner] Error reading stderr: {}", e);

                        // Emit error event
                        if let Some(ref handle) = app_handle_stderr {
                            if let Err(emit_err) = handle.emit(
                                "process_error",
                                serde_json::json!({
                                    "iteration_id": iteration_id_clone,
                                    "error": e.to_string()
                                }),
                            ) {
                                eprintln!(
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
                                println!("[Runner] Process {} exited with status: {:?}", iteration_id_exit, status);
                                procs.remove(&iteration_id_exit);
                                true
                            }
                            Ok(None) => false, // Still running
                            Err(e) => {
                                eprintln!("[Runner] Error checking process status: {}", e);
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

        println!("[Runner] Process started with PID: {}", pid);
        Ok(pid)
    }

    pub async fn stop(&self, iteration_id: String) -> Result<(), String> {
        // Remove process from map and release lock before await
        let process = {
            let mut processes = self.processes.lock().unwrap();
            processes.remove(&iteration_id)
        };

        if let Some(mut process) = process {
            println!("[Runner] Stopping process for iteration: {}", iteration_id);

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

            println!("[Runner] Process stopped");
            Ok(())
        } else {
            // Process already stopped or not found - this is fine, just return success
            println!(
                "[Runner] No running process found for iteration: {} (may already be stopped)",
                iteration_id
            );
            Ok(())
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
