//! ACP (Agent Client Protocol) Client Implementation
//! 
//! This module uses the official agent-client-protocol SDK to connect to external
//! coding agents like OpenCode, Claude Code, Gemini CLI, etc.
//!
//! The agent-client-protocol SDK uses ?Send futures, which we handle by running
//! in a dedicated thread with its own LocalSet and communicating via channels.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use agent_client_protocol::{self as acp, Agent};
use tokio::sync::oneshot;

use crate::llm::config::CodingAgentConfig;

/// A simple client implementation that handles notifications
struct CoworkClient {
    output: Arc<std::sync::Mutex<String>>,
}

#[async_trait::async_trait(?Send)]
impl acp::Client for CoworkClient {
    async fn request_permission(
        &self,
        _args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn write_text_file(
        &self,
        _args: acp::WriteTextFileRequest,
    ) -> acp::Result<acp::WriteTextFileResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn read_text_file(
        &self,
        _args: acp::ReadTextFileRequest,
    ) -> acp::Result<acp::ReadTextFileResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn create_terminal(
        &self,
        _args: acp::CreateTerminalRequest,
    ) -> Result<acp::CreateTerminalResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn terminal_output(
        &self,
        _args: acp::TerminalOutputRequest,
    ) -> acp::Result<acp::TerminalOutputResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn release_terminal(
        &self,
        _args: acp::ReleaseTerminalRequest,
    ) -> acp::Result<acp::ReleaseTerminalResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn wait_for_terminal_exit(
        &self,
        _args: acp::WaitForTerminalExitRequest,
    ) -> acp::Result<acp::WaitForTerminalExitResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn kill_terminal_command(
        &self,
        _args: acp::KillTerminalCommandRequest,
    ) -> acp::Result<acp::KillTerminalCommandResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> acp::Result<(), acp::Error> {
        match args.update {
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) => {
                if let acp::ContentBlock::Text(text_content) = content {
                    eprintln!("AGENT: {}", text_content.text);
                    // Also store in output
                    if let Ok(mut out) = self.output.lock() {
                        out.push_str(&text_content.text);
                    }
                }
            }
            acp::SessionUpdate::AgentThoughtChunk(acp::ContentChunk { content, .. }) => {
                if let acp::ContentBlock::Text(text_content) = content {
                    eprintln!("AGENT THINKING: {}", text_content.text);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn ext_method(&self, _args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: acp::ExtNotification) -> acp::Result<()> {
        Err(acp::Error::method_not_found())
    }
}

/// Result from executing a coding task
#[derive(Debug, Clone)]
pub struct AcpTaskResult {
    /// The generated code/content
    pub content: String,
    /// Whether the task was completed
    pub completed: bool,
    /// Any error message
    pub error: Option<String>,
}

impl AcpTaskResult {
    pub fn new(content: String, completed: bool) -> Self {
        Self {
            content,
            completed,
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            content: String::new(),
            completed: false,
            error: Some(msg),
        }
    }
}

/// Execute a task using an external agent via ACP
/// 
/// This function runs the ACP client in a dedicated thread with its own
/// tokio runtime and LocalSet, communicating results via a oneshot channel.
pub async fn execute_with_external_agent(
    config: CodingAgentConfig,
    workspace: PathBuf,
    task: String,
) -> Result<String> {
    eprintln!("DEBUG: Starting external agent with {} {:?}", config.command, config.args);

    // Create channel for result
    let (tx, rx) = oneshot::channel();

    // Spawn a dedicated thread for the non-Send operations
    std::thread::spawn(move || {
        let result = run_acp_in_thread(config, workspace, task);
        let _ = tx.send(result);
    });

    // Wait for result from the dedicated thread
    rx.await.context("ACP thread disconnected")?
}

/// Run ACP operations in a dedicated thread with its own runtime
fn run_acp_in_thread(
    config: CodingAgentConfig,
    workspace: PathBuf,
    task: String,
) -> Result<String> {
    // Create a new tokio runtime for this thread
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create tokio runtime")?;

    rt.block_on(async {
        use tokio::process::Command;
        use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

        // Spawn the agent process
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .current_dir(&workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("Failed to spawn agent: {}", config.command))?;

        eprintln!("DEBUG: Agent process spawned, pid: {:?}", child.id());

        let outgoing = child.stdin.take().unwrap().compat_write();
        let incoming = child.stdout.take().unwrap().compat();

        // Handle stderr for debugging
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut stderr = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = stderr.next_line().await {
                    eprintln!("AGENT STDERR: {}", line);
                }
            });
        }

        // Use LocalSet for non-Send futures
        let local_set = tokio::task::LocalSet::new();
        let output = Arc::new(std::sync::Mutex::new(String::new()));
        let output_clone = output.clone();

        local_set.run_until(async move {
            let (conn, handle_io) = acp::ClientSideConnection::new(
                CoworkClient { output: output_clone },
                outgoing,
                incoming,
                |fut| {
                    tokio::task::spawn_local(fut);
                },
            );

            // Handle I/O in the background
            tokio::task::spawn_local(handle_io);

            eprintln!("DEBUG: Initializing ACP connection...");

            // Initialize
            conn.initialize(
                acp::InitializeRequest::new(acp::ProtocolVersion::V1)
                    .client_info(acp::Implementation::new(
                        "cowork-forge".to_string(),
                        "2.0.0".to_string(),
                    ).title("Cowork Forge".to_string()))
            )
            .await
            .context("Failed to initialize ACP connection")?;

            eprintln!("DEBUG: ACP initialized, creating session...");

            // Create session
            let session_response = conn
                .new_session(acp::NewSessionRequest::new(workspace))
                .await
                .context("Failed to create ACP session")?;

            eprintln!("DEBUG: Session created: {:?}", session_response.session_id);

            // Send prompt
            eprintln!("DEBUG: Sending prompt to agent...");
            
            let result = conn
                .prompt(acp::PromptRequest::new(
                    session_response.session_id,
                    vec![task.into()],
                ))
                .await
                .context("Failed to send prompt to agent")?;

            eprintln!("DEBUG: Prompt completed, stop reason: {:?}", result.stop_reason);

            // Get accumulated output
            let output = output.lock().unwrap().clone();
            Ok::<_, anyhow::Error>(output)
        }).await
    })
}

/// ACP Client wrapper
pub struct AcpClient {
    config: CodingAgentConfig,
    workspace: PathBuf,
}

impl AcpClient {
    /// Create from CodingAgentConfig
    pub async fn from_config(config: &CodingAgentConfig, workspace: &PathBuf) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            workspace: workspace.clone(),
        })
    }

    /// Execute a coding task
    pub async fn execute_task(&mut self, task: &str) -> Result<String> {
        execute_with_external_agent(self.config.clone(), self.workspace.clone(), task.to_string()).await
    }
}
