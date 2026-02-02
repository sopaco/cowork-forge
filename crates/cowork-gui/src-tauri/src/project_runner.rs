// Project runner for GUI
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::process::{Command, Child};
use tokio::sync::mpsc;

pub struct ProjectRunner {
    processes: Arc<Mutex<HashMap<String, ProjectProcess>>>,
}

struct ProjectProcess {
    child: Child,
    output_tx: mpsc::UnboundedSender<String>,
}

impl ProjectRunner {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self, session_id: String, command: String) -> Result<u32, String> {
        // Stop existing process if any
        if let Ok(()) = self.stop(session_id.clone()).await {
            println!("[Runner] Stopped existing process for session: {}", session_id);
        }

        // Get session directory
        let code_dir = format!(".cowork/sessions/{}/code", session_id);
        let code_path = std::path::Path::new(&code_dir);

        if !code_path.exists() {
            return Err(format!("Code directory not found: {}", code_dir));
        }

        println!("[Runner] Starting command: {}", command);

        let mut child = Command::new("cmd")
            .args(["/C", &command])
            .current_dir(&code_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to start: {}", e))?;

        let pid = child.id().unwrap();

        // Create channels for output
        let (stdout_tx, mut stdout_rx) = mpsc::unbounded_channel();
        let (stderr_tx, mut stderr_rx) = mpsc::unbounded_channel();

        // Clone child for stdout reading
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Clone senders for spawn tasks
        let stdout_tx_spawn = stdout_tx.clone();
        let stderr_tx_spawn = stderr_tx.clone();

        // Spawn task to read stdout
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let _ = stdout_tx_spawn.send(line.clone());
                        line.clear();
                    }
                    Err(e) => {
                        eprintln!("[Runner] Error reading stdout: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn task to read stderr
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let _ = stderr_tx_spawn.send(line.clone());
                        line.clear();
                    }
                    Err(e) => {
                        eprintln!("[Runner] Error reading stderr: {}", e);
                        break;
                    }
                }
            }
        });

        let mut processes = self.processes.lock().unwrap();
        processes.insert(session_id, ProjectProcess {
            child,
            output_tx: stdout_tx,
        });

        println!("[Runner] Process started with PID: {}", pid);
        Ok(pid)
    }

    pub async fn stop(&self, session_id: String) -> Result<(), String> {
        // Remove process from map and release lock before await
        let mut process = {
            let mut processes = self.processes.lock().unwrap();
            processes.remove(&session_id)
        };
        
        if let Some(mut process) = process {
            println!("[Runner] Stopping process for session: {}", session_id);
            
            process.child.kill()
                .await
                .map_err(|e| format!("Failed to stop: {}", e))?;
            
            println!("[Runner] Process stopped");
            Ok(())
        } else {
            Err(format!("No running process for session: {}", session_id))
        }
    }

    pub async fn execute_command(&self, session_id: String, command: String) -> Result<String, String> {
        println!("[Runner] Executing command: {}", command);

        let project_root = cowork_core::storage::get_project_root()
            .map_err(|e| format!("Failed to get project root: {}", e))?;

        if !project_root.exists() {
            return Err(format!("Project directory not found: {}", project_root.display()));
        }

        let output = Command::new("cmd")
            .args(["/C", &command])
            .current_dir(&project_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .await;

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

    // pub fn get_output_receiver(&self, session_id: String) -> Option<mpsc::UnboundedReceiver<String>> {
    //     let processes = self.processes.lock().unwrap();
    //     processes.get(&session_id).map(|p| p.output_tx.clone())
    // }
}
