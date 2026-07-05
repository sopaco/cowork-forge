//! ACP (Agent Client Protocol) Client Implementation
//!
//! This module uses the official agent-client-protocol SDK to connect to external
//! coding agents like OpenCode, Claude Code, Gemini CLI, etc.
//!
//! The agent-client-protocol SDK uses ?Send futures, which we handle by running
//! in a dedicated thread with its own LocalSet and communicating via channels.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use agent_client_protocol::{self as acp, Agent};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::llm::config::CodingAgentConfig;

/// Timeout for a single ACP prompt turn.
const PROMPT_TIMEOUT_SECONDS: u64 = 600;

/// Max stderr line length to forward as status.
const MAX_STATUS_LINE_LEN: usize = 500;

/// Message types from the agent
#[derive(Debug, Clone)]
pub enum AgentMessage {
    /// Agent thinking/reasoning
    Thinking(String),
    /// Agent output text
    Output(String),
    /// Status update
    Status(String),
    /// Error message
    Error(String),
    /// Task completed
    Completed,
}

/// A simple client implementation that handles notifications
struct CoworkClient {
    output: Arc<std::sync::Mutex<String>>,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
    workspace: PathBuf,
}

impl CoworkClient {
    /// Validate that an absolute path is within the workspace directory.
    /// Returns the path relative to the workspace if valid, or an error.
    fn validate_workspace_path(&self, abs_path: &Path) -> Result<PathBuf, acp::Error> {
        let workspace_abs = self
            .workspace
            .canonicalize()
            .map_err(acp::Error::into_internal_error)?;
        let target_abs = abs_path
            .canonicalize()
            .or_else(|_| {
                // File may not exist yet (write); canonicalize the parent instead.
                let parent = abs_path.parent().unwrap_or(abs_path);
                let file_name = abs_path.file_name().ok_or_else(|| {
                    acp::Error::invalid_params().data("path has no file name")
                })?;
                let parent_abs = parent.canonicalize().map_err(acp::Error::into_internal_error)?;
                Ok::<_, acp::Error>(parent_abs.join(file_name))
            })
            .map_err(acp::Error::into_internal_error)?;

        let workspace_stripped = strip_unc_prefix(&workspace_abs);
        let target_stripped = strip_unc_prefix(&target_abs);

        if target_stripped.starts_with(&workspace_stripped) {
            Ok(target_stripped)
        } else {
            Err(acp::Error::invalid_params().data(format!(
                "path '{}' is outside workspace '{}'",
                target_stripped.display(),
                workspace_stripped.display()
            )))
        }
    }
}

/// Strip the Windows UNC prefix (`\\?\`) so paths can be compared consistently.
fn strip_unc_prefix(path: &Path) -> PathBuf {
    let s = path.display().to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}

#[async_trait::async_trait(?Send)]
impl acp::Client for CoworkClient {
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        tracing::info!(
            session_id = %args.session_id,
            "ACP permission request for tool call: {:?}",
            args.tool_call
        );

        // Auto-approve the first "allow" option so the agent can keep working.
        // In the future this can be wired to the InteractiveBackend for user consent.
        let allow_option = args
            .options
            .into_iter()
            .find(|o| matches!(o.kind, acp::PermissionOptionKind::AllowOnce | acp::PermissionOptionKind::AllowAlways));

        match allow_option {
            Some(option) => {
                tracing::info!(option_id = %option.option_id.0, "Auto-approving ACP permission request");
                Ok(acp::RequestPermissionResponse::new(
                    acp::RequestPermissionOutcome::Selected(
                        acp::SelectedPermissionOutcome::new(option.option_id),
                    ),
                ))
            }
            None => {
                tracing::warn!("ACP permission request had no allow option; cancelling");
                Ok(acp::RequestPermissionResponse::new(
                    acp::RequestPermissionOutcome::Cancelled,
                ))
            }
        }
    }

    async fn write_text_file(
        &self,
        args: acp::WriteTextFileRequest,
    ) -> acp::Result<acp::WriteTextFileResponse> {
        tracing::debug!(session_id = %args.session_id, path = %args.path.display(), "ACP write_text_file");

        let rel_path = self.validate_workspace_path(&args.path)?;
        let full_path = self.workspace.join(&rel_path);

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(acp::Error::into_internal_error)?;
        }
        std::fs::write(&full_path, args.content).map_err(|e| {
            tracing::error!(path = %full_path.display(), error = %e, "ACP write_text_file failed");
            acp::Error::into_internal_error(e)
        })?;

        tracing::info!(path = %full_path.display(), bytes = ?std::fs::metadata(&full_path).map(|m| m.len()).unwrap_or(0), "ACP write_text_file succeeded");
        let _ = self.message_tx.send(AgentMessage::Status(format!(
            "Wrote file {}",
            rel_path.display()
        )));
        Ok(acp::WriteTextFileResponse::new())
    }

    async fn read_text_file(
        &self,
        args: acp::ReadTextFileRequest,
    ) -> acp::Result<acp::ReadTextFileResponse> {
        tracing::debug!(session_id = %args.session_id, path = %args.path.display(), "ACP read_text_file");

        let rel_path = self.validate_workspace_path(&args.path)?;
        let full_path = self.workspace.join(&rel_path);

        let content = std::fs::read_to_string(&full_path).map_err(|e| {
            tracing::error!(path = %full_path.display(), error = %e, "ACP read_text_file failed");
            if e.kind() == std::io::ErrorKind::NotFound {
                acp::Error::resource_not_found(Some(full_path.display().to_string()))
            } else {
                acp::Error::into_internal_error(e)
            }
        })?;

        tracing::info!(path = %full_path.display(), len = content.len(), "ACP read_text_file succeeded");
        Ok(acp::ReadTextFileResponse::new(content))
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
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk {
                content: acp::ContentBlock::Text(text_content),
                ..
            }) => {
                let text = text_content.text.clone();
                tracing::debug!(len = text.len(), "ACP agent message chunk");
                let _ = self.message_tx.send(AgentMessage::Output(text));
                if let Ok(mut out) = self.output.lock() {
                    out.push_str(&text_content.text);
                }
            }
            acp::SessionUpdate::AgentThoughtChunk(acp::ContentChunk {
                content: acp::ContentBlock::Text(text_content),
                ..
            }) => {
                let text = text_content.text.clone();
                tracing::debug!(len = text.len(), "ACP agent thought chunk");
                let _ = self.message_tx.send(AgentMessage::Thinking(text));
            }
            // Ignore other updates (different protocol versions may have different variants)
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
/// tokio runtime and LocalSet, communicating results via channels.
///
/// Returns a receiver for real-time agent messages.
pub fn execute_with_external_agent(
    config: CodingAgentConfig,
    workspace: PathBuf,
    task: String,
) -> (mpsc::UnboundedReceiver<AgentMessage>, impl std::future::Future<Output = Result<Result<String>>>) {
    tracing::info!(
        command = %config.command,
        args = ?config.args,
        workspace = %workspace.display(),
        "Starting external ACP agent"
    );

    // Create channel for real-time messages
    let (message_tx, message_rx) = mpsc::unbounded_channel();

    // Create channel for final result
    let (tx, rx) = oneshot::channel();

    // Spawn a dedicated thread for the non-Send operations
    std::thread::spawn(move || {
        let result = run_acp_in_thread(config, workspace, task, message_tx);
        let _ = tx.send(result);
    });

    // Return both the message receiver and the result future
    // Note: rx.await returns Result<String>, the outer Result is from the channel
    (message_rx, async move {
        rx.await.context("ACP thread disconnected")
    })
}

/// Maximum stderr lines to retain for error diagnostics.
const MAX_STDERR_LINES: usize = 50;

/// Run ACP operations in a dedicated thread with its own runtime
fn run_acp_in_thread(
    config: CodingAgentConfig,
    workspace: PathBuf,
    task: String,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
) -> Result<String> {
    // Create a new tokio runtime for this thread
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create tokio runtime")?;

    rt.block_on(async {
        use tokio::process::Command;

        let _ = message_tx.send(AgentMessage::Status("Starting agent process...".to_string()));

        // Spawn the agent process
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .current_dir(&workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        // Preserve PATH and inject any user-configured environment variables.
        cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
        if let Some(ref env_vars) = config.env {
            for (key, value) in env_vars {
                tracing::debug!(key, value, "Injecting env var into ACP agent");
                cmd.env(key, value);
            }
        }

        // On Windows, use CREATE_NO_WINDOW to prevent console window from appearing
        #[cfg(target_os = "windows")]
        {
            #[allow(unused_imports)]
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn agent: {}", config.command))?;

        tracing::info!(pid = ?child.id(), "ACP agent process spawned");
        let _ = message_tx.send(AgentMessage::Status(format!(
            "ACP command: {} {}",
            config.command,
            config.args.join(" ")
        )));
        let _ = message_tx.send(AgentMessage::Status(format!(
            "Agent process started (pid: {:?})",
            child.id()
        )));

        let stdin = child.stdin.take().context("Failed to open agent stdin")?;
        let stdout = child.stdout.take().context("Failed to open agent stdout")?;
        let outgoing = stdin.compat_write();
        let incoming = stdout.compat();

        // Capture stderr for diagnostics.
        let stderr_buffer: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let stderr_buffer_clone = stderr_buffer.clone();

        let stderr_handle: Option<tokio::task::JoinHandle<()>> = if let Some(stderr) = child.stderr.take() {
            let tx = message_tx.clone();
            let buf = stderr_buffer_clone.clone();
            Some(tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut stderr = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = stderr.next_line().await {
                    tracing::debug!(stderr = %line, "ACP agent stderr");
                    if let Ok(mut b) = buf.lock() {
                        if b.len() >= MAX_STDERR_LINES {
                            b.remove(0);
                        }
                        b.push(line.clone());
                    }
                    let status = if line.len() > MAX_STATUS_LINE_LEN {
                        format!("[stderr] {}...", &line[..MAX_STATUS_LINE_LEN])
                    } else {
                        format!("[stderr] {}", line)
                    };
                    let _ = tx.send(AgentMessage::Status(status));
                }
            }))
        } else {
            None
        };

        // Helper to build an error context that includes recent stderr.
        let with_stderr = |msg: &str| {
            let recent = stderr_buffer
                .lock()
                .map(|b| b.join("\n"))
                .unwrap_or_default();
            if recent.is_empty() {
                msg.to_string()
            } else {
                format!("{}\nRecent agent stderr:\n{}", msg, recent)
            }
        };

        // Use LocalSet for non-Send futures
        let local_set = tokio::task::LocalSet::new();
        let output = Arc::new(std::sync::Mutex::new(String::new()));
        let output_clone = output.clone();
        let workspace_clone = workspace.clone();
        let tx_err = message_tx.clone();
        let status_tx = message_tx.clone();
        // Clone for the completion notification sent after the local_set returns.
        // The original message_tx is moved into CoworkClient inside the local_set block.
        let completion_tx = message_tx.clone();

        let result: Result<String> = local_set
            .run_until(async move {
                let (conn, handle_io) = acp::ClientSideConnection::new(
                    CoworkClient {
                        output: output_clone,
                        message_tx,
                        workspace: workspace_clone,
                    },
                    outgoing,
                    incoming,
                    |fut| {
                        tokio::task::spawn_local(fut);
                    },
                );

                // Handle I/O in the background
                tokio::task::spawn_local(handle_io);

                tracing::info!("Initializing ACP connection");
                let _ = status_tx.send(AgentMessage::Status(
                    "Initializing ACP connection...".to_string(),
                ));

                let init_response = conn
                    .initialize(
                        acp::InitializeRequest::new(acp::ProtocolVersion::V1)
                            .client_info(
                                acp::Implementation::new(
                                    "cowork-forge".to_string(),
                                    env!("CARGO_PKG_VERSION").to_string(),
                                )
                                .title("Cowork Forge".to_string()),
                            )
                            .client_capabilities(
                                acp::ClientCapabilities::new().fs(
                                    acp::FileSystemCapability::new()
                                        .read_text_file(true)
                                        .write_text_file(true),
                                ),
                            ),
                    )
                    .await
                    .context("Failed to initialize ACP connection")?;

                let agent_name = init_response
                    .agent_info
                    .as_ref()
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                let agent_version = init_response
                    .agent_info
                    .as_ref()
                    .map(|i| i.version.clone())
                    .unwrap_or_default();
                tracing::info!(
                    protocol_version = ?init_response.protocol_version,
                    agent = ?init_response.agent_info,
                    auth_methods = ?init_response.auth_methods,
                    "ACP initialized"
                );
                let _ = status_tx.send(AgentMessage::Status(format!(
                    "ACP initialized with {} {}",
                    agent_name, agent_version
                )));

                // Handle authentication if the agent requires it.
                if let Some(first_method) = init_response.auth_methods.first() {
                    tracing::info!(method_id = %first_method.id.0, "ACP agent requires authentication");
                    let _ = status_tx.send(AgentMessage::Status(
                        "ACP authenticating...".to_string(),
                    ));
                    conn.authenticate(acp::AuthenticateRequest::new(first_method.id.clone()))
                        .await
                        .context("Failed to authenticate with ACP agent")?;
                    tracing::info!("ACP authentication completed");
                    let _ = status_tx.send(AgentMessage::Status(
                        "ACP authentication completed".to_string(),
                    ));
                }

                tracing::info!("Creating ACP session");
                let _ = status_tx.send(AgentMessage::Status(format!(
                    "Creating ACP session in {}...",
                    workspace.display()
                )));
                let session_response = conn
                    .new_session(acp::NewSessionRequest::new(workspace))
                    .await
                    .context("Failed to create ACP session")?;

                tracing::info!(session_id = %session_response.session_id, "ACP session created");
                let _ = status_tx.send(AgentMessage::Status(format!(
                    "ACP session created: {}",
                    session_response.session_id
                )));

                tracing::info!("Sending prompt to ACP agent");
                let _ = status_tx.send(AgentMessage::Status("Sending prompt...".to_string()));
                let prompt =
                    acp::PromptRequest::new(session_response.session_id, vec![task.into()]);
                let prompt_result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(PROMPT_TIMEOUT_SECONDS),
                    conn.prompt(prompt),
                )
                .await
                .context(format!(
                    "ACP prompt timed out after {} seconds",
                    PROMPT_TIMEOUT_SECONDS
                ))?
                .context("Failed to send prompt to agent")?;

                tracing::info!(
                    stop_reason = ?prompt_result.stop_reason,
                    "ACP prompt completed"
                );
                let _ = status_tx.send(AgentMessage::Status(format!(
                    "Prompt completed: {:?}",
                    prompt_result.stop_reason
                )));

                // Get accumulated output
                let output = output
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to lock output buffer: {}", e))?
                    .clone();
                Ok::<_, anyhow::Error>(output)
            })
            .await;

        // Cleanup: kill the child process and wait for stderr task to complete.
        // This ensures all clones of message_tx are dropped before we return,
        // which closes the message channel and unblocks the message loop in the
        // pipeline stage. Without this, the stderr task (spawned via tokio::spawn)
        // keeps message_tx alive and the channel never closes.
        tracing::info!("Cleaning up ACP agent process");
        let _ = child.start_kill();
        // Wait for the child to exit (with a timeout to avoid hanging)
        let _ = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            child.wait(),
        ).await;
        // Wait for the stderr task to complete (it should exit once the child's
        // stderr pipe closes, which happens when the child is killed)
        if let Some(handle) = stderr_handle {
            let _ = tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                handle,
            ).await;
        }

        if let Err(ref e) = result {
            let diagnostic = with_stderr(&e.to_string());
            tracing::error!(error = %diagnostic, "ACP agent failed");
            let _ = tx_err.send(AgentMessage::Error(format!(
                "ACP agent failed: {}",
                diagnostic
            )));
            // Signal the message loop to exit even on error
            let _ = completion_tx.send(AgentMessage::Completed);
            return Err(anyhow::anyhow!(diagnostic));
        }
        // Notify the message loop that the task is complete so it can exit.
        // This is the primary signal that unblocks the pipeline stage.
        let _ = completion_tx.send(AgentMessage::Completed);
        result
    })
}

/// ACP Client wrapper
pub struct AcpClient {
    config: CodingAgentConfig,
    workspace: PathBuf,
}

impl AcpClient {
    /// Create from CodingAgentConfig
    pub async fn from_config(config: &CodingAgentConfig, workspace: &Path) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            workspace: workspace.to_path_buf(),
        })
    }

    /// Execute a coding task, returning message receiver and result future
    pub fn execute_task_stream(
        self,
        task: String,
    ) -> (mpsc::UnboundedReceiver<AgentMessage>, impl std::future::Future<Output = Result<Result<String>>>) {
        execute_with_external_agent(self.config, self.workspace, task)
    }

    /// Execute a coding task (simpler API for backward compatibility)
    pub async fn execute_task(&mut self, task: &str) -> Result<String> {
        let (_, result) = execute_with_external_agent(
            self.config.clone(),
            self.workspace.clone(),
            task.to_string(),
        );
        // Flatten the nested Result: Result<Result<String>> -> Result<String>
        result.await?
    }
}
