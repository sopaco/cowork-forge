// Project runner for GUI
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::process::{Command, Child};
use tokio::sync::mpsc;
use tauri::Emitter;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct ProjectRunner {
    processes: Arc<Mutex<HashMap<String, ProjectProcess>>>,
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

struct ProjectProcess {
    child: Child,
    output_tx: mpsc::UnboundedSender<String>,
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

    pub async fn start(&self, iteration_id: String, command: String, code_dir: String) -> Result<u32, String> {
        // Stop existing process if any
        if let Ok(()) = self.stop(iteration_id.clone()).await {
            println!("[Runner] Stopped existing process for iteration: {}", iteration_id);
        }

        // Use provided code_dir instead of hardcoded workspace path
        let code_path = std::path::Path::new(&code_dir);

        if !code_path.exists() {
            return Err(format!("Code directory not found: {}", code_dir));
        }

        println!("[Runner] Starting command: {} in {}", command, code_dir);

        #[cfg(target_os = "windows")]
        let mut child = {
            let mut cmd = std::process::Command::new("cmd");
            cmd.args(["/C", &command])
                .current_dir(&code_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .creation_flags(0x08000000); // CREATE_NO_WINDOW
            
            // Convert std::process::Command to tokio::process::Command
            let std_child = cmd.spawn()
                .map_err(|e| format!("Failed to start: {}", e))?;
            
            // Convert to tokio child
            let pid = std_child.id();
            tokio::process::Child::from(std_child)
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
                            if let Err(e) = handle.emit("project_log", serde_json::json!({
                                "iteration_id": iteration_id_stdout,
                                "session_id": iteration_id_stdout,
                                "stream": "stdout",
                                "content": line.clone()
                            })) {
                                eprintln!("[Runner] Failed to emit project_log event: {}", e);
                            }
                        }
                        
                        line.clear();
                    }
                    Err(e) => {
                        eprintln!("[Runner] Error reading stdout: {}", e);
                        
                        // Emit error event (use expected event name)
                        if let Some(ref handle) = app_handle_stdout {
                            if let Err(e) = handle.emit("project_log", serde_json::json!({
                                "iteration_id": iteration_id_stdout,
                                "session_id": iteration_id_stdout,
                                "stream": "stderr",
                                "content": format!("Error reading output: {}\n", e)
                            })) {
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
                            if let Err(e) = handle.emit("project_log", serde_json::json!({
                                "iteration_id": iteration_id_clone,
                                "session_id": iteration_id_clone,
                                "stream": "stderr",
                                "content": line.clone()
                            })) {
                                eprintln!("[Runner] Failed to emit project_log event: {}", e);
                            }
                        }
                        
                        line.clear();
                    }
                    Err(e) => {
                        eprintln!("[Runner] Error reading stderr: {}", e);
                        
                        // Emit error event
                        if let Some(ref handle) = app_handle_stderr {
                            if let Err(emit_err) = handle.emit("process_error", serde_json::json!({
                                "iteration_id": iteration_id_clone,
                                "error": e.to_string()
                            })) {
                                eprintln!("[Runner] Failed to emit process_error event: {}", emit_err);
                            }
                        }
                        break;
                    }
                }
            }
        });

        let mut processes = self.processes.lock().unwrap();
        processes.insert(iteration_id.clone(), ProjectProcess {
            child,
            output_tx: stdout_tx,
        });
        drop(processes);

        // Spawn task to wait for process exit and emit stopped event
        let iteration_id_exit = iteration_id.clone();
        let processes_ref = Arc::clone(&self.processes);
        
        tokio::spawn(async move {
            // First get the child from processes, then release the lock before waiting
            let child = {
                let mut procs = processes_ref.lock().unwrap();
                procs.remove(&iteration_id_exit).map(|p| p.child)
            };
            
            // Wait for process to exit (without holding the lock)
            let exit_status = if let Some(mut child) = child {
                child.wait().await.ok()
            } else {
                None
            };
            
            println!("[Runner] Process exited with status: {:?}", exit_status);
            
            // Emit stopped event
            if let Some(ref handle) = app_handle_exit {
                let _ = handle.emit("project_stopped", serde_json::json!({
                    "iteration_id": iteration_id_exit,
                    "session_id": iteration_id_exit
                }));
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
                let _ = handle.emit("project_stopped", serde_json::json!({
                    "iteration_id": iteration_id,
                    "session_id": iteration_id
                }));
            }
            
            println!("[Runner] Process stopped");
            Ok(())
        } else {
            // Process already stopped or not found - this is fine, just return success
            println!("[Runner] No running process found for iteration: {} (may already be stopped)", iteration_id);
            Ok(())
        }
    }

    pub async fn execute_command(&self, _session_id: String, command: String) -> Result<String, String> {
        println!("[Runner] Executing command: {}", command);

        let project_root = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        if !project_root.exists() {
            return Err(format!("Project directory not found: {}", project_root.display()));
        }

        #[cfg(target_os = "windows")]
        let output = {
            let mut cmd = std::process::Command::new("cmd");
            cmd.args(["/C", &command])
                .current_dir(&project_root)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .creation_flags(0x08000000); // CREATE_NO_WINDOW
            
            let std_output = cmd.output()
                .map_err(|e| format!("Failed to execute command: {}", e))?;
            
            // Convert std::process::Output to compatible format
            Ok::<_, String>(std_output)
        };

        #[cfg(not(target_os = "windows"))]
        let output = {
            let std_output = std::process::Command::new("sh")
                .args(["-c", &command])
                .current_dir(&project_root)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .map_err(|e| format!("Failed to execute command: {}", e))?;
            
            Ok::<_, String>(std_output)
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                let result = format!("Exit code: {:?}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}", 
                    output.status.code(), stdout, stderr);
                
                Ok(result)
            }
            Err(e) => {
                Err(format!("Failed to execute command: {}", e))
            }
        }
    }
}
