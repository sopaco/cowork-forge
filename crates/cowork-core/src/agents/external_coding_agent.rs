// External Coding Agent - Adapter for external CLI-based coding agents via ACP
// This module provides an alternative to the built-in adk-rust coding agent

use std::path::PathBuf;

use anyhow::{Context, Result};
use tokio::sync::mpsc;

use crate::acp::{AcpClient, AcpTaskResult, AgentMessage};
use crate::domain::{InheritanceMode, Iteration};
use crate::instructions::coding::CODING_ACTOR_INSTRUCTION;
use crate::llm::config::{load_config, CodingAgentConfig};

/// External Coding Agent Adapter
/// 
/// This adapter allows Cowork to use external coding CLI tools (like iFlow, Gemini CLI, Codex)
/// as the underlying coding agent instead of the built-in adk-rust agent.
/// 
/// The adapter communicates with the external agent via ACP (Agent Client Protocol),
/// either through stdio or WebSocket.
pub struct ExternalCodingAgent {
    /// Configuration
    config: CodingAgentConfig,
    /// Workspace path
    workspace: PathBuf,
    /// Whether the agent is ready
    ready: bool,
    /// Optional iteration context for evolution iterations
    iteration: Option<Iteration>,
}

/// Result of starting a streaming task
pub struct StreamingTask {
    /// Receiver for real-time agent messages
    pub messages: mpsc::UnboundedReceiver<AgentMessage>,
    /// The result future - outer Result is from channel, inner Result is from ACP
    pub result: std::pin::Pin<Box<dyn std::future::Future<Output = Result<Result<String>>> + Send>>,
}

impl ExternalCodingAgent {
    /// Create a new External Coding Agent
    pub async fn new(workspace: &PathBuf) -> Result<Self> {
        Self::new_with_iteration(workspace, None).await
    }

    /// Create a new External Coding Agent with iteration context
    pub async fn new_with_iteration(workspace: &PathBuf, iteration: Option<Iteration>) -> Result<Self> {
        eprintln!("DEBUG: ExternalCodingAgent::new_with_iteration called with workspace: {}", workspace.display());
        if let Some(ref iter) = iteration {
            eprintln!("DEBUG: Iteration context: id={}, base_id={:?}, inheritance={:?}", 
                iter.id, iter.base_iteration_id, iter.inheritance);
        }
        
        let config = load_config()
            .context("Failed to load config")?;
        
        eprintln!("DEBUG: Config loaded, coding_agent.enabled: {}", config.coding_agent.enabled);

        if !config.coding_agent.enabled {
            anyhow::bail!("External coding agent is not enabled in config");
        }

        Ok(Self {
            config: config.coding_agent,
            workspace: workspace.clone(),
            ready: false,
            iteration,
        })
    }

    /// Check if external agent is enabled in config
    pub fn is_enabled() -> Result<bool> {
        let config = load_config()
            .context("Failed to load config")?;
        Ok(config.coding_agent.enabled)
    }

    /// Execute a coding task with streaming messages
    /// 
    /// Returns a StreamingTask with a message receiver for real-time updates
    /// and a result future for the final output.
    pub fn execute_task_stream(
        self,
        task_description: &str,
        project_context: &str,
    ) -> StreamingTask {
        let prompt = self.build_prompt(task_description, project_context);
        
        // Use the execute_with_external_agent directly to avoid async issues
        let (messages, result) = crate::acp::execute_with_external_agent(
            self.config,
            self.workspace,
            prompt,
        );

        StreamingTask {
            messages,
            result: Box::pin(result),
        }
    }

    /// Execute a coding task (simpler API)
    /// 
    /// This method sends the task to the external agent and returns the result.
    /// It builds a comprehensive prompt including:
    /// - The base instruction
    /// - The task description
    /// - Context about the project
    pub async fn execute_task(
        &mut self,
        task_description: &str,
        project_context: &str,
    ) -> Result<AcpTaskResult> {
        // Build the prompt
        let prompt = self.build_prompt(task_description, project_context);

        tracing::info!("Executing coding task via external agent: {}", &prompt[..prompt.len().min(200)]);

        // Create client and execute
        let mut client = AcpClient::from_config(&self.config, &self.workspace).await?;
        
        // Execute the task
        match client.execute_task(&prompt).await {
            Ok(result) => {
                self.ready = true;
                Ok(AcpTaskResult::new(result, true))
            }
            Err(e) => {
                tracing::error!("External agent execution failed: {}", e);
                Ok(AcpTaskResult::error(e.to_string()))
            }
        }
    }

    /// Build a comprehensive prompt for the external agent
    fn build_prompt(&self, task_description: &str, project_context: &str) -> String {
        let mut prompt = String::new();

        // Check if this is an evolution iteration
        let is_evolution = self.iteration.as_ref()
            .map(|i| i.base_iteration_id.is_some())
            .unwrap_or(false);
        
        let inheritance_mode = self.iteration.as_ref()
            .map(|i| i.inheritance)
            .unwrap_or(InheritanceMode::None);

        if is_evolution {
            prompt.push_str("═══════════════════════════════════════════════════════════════\n");
            prompt.push_str("🚨 CRITICAL: THIS IS AN EVOLUTION ITERATION\n");
            prompt.push_str("═══════════════════════════════════════════════════════════════\n");
            prompt.push_str("\n");
            prompt.push_str("⚠️ DO NOT DELETE EXISTING CODE! ⚠️\n");
            prompt.push_str("\n");
            prompt.push_str("This iteration builds upon an EXISTING project.\n");
            prompt.push_str("The workspace directory already contains code from a previous iteration.\n");
            prompt.push_str("\n");
            
            match inheritance_mode {
                InheritanceMode::Partial => {
                    prompt.push_str("📋 INHERITANCE MODE: PARTIAL\n");
                    prompt.push_str("- Code files from the base iteration have been copied to the workspace\n");
                    prompt.push_str("- Artifacts (PRD, Design, Plan) are regenerated fresh\n");
                    prompt.push_str("- You MUST preserve existing code and add new features incrementally\n");
                }
                InheritanceMode::Full => {
                    prompt.push_str("📋 INHERITANCE MODE: FULL\n");
                    prompt.push_str("- All files (code + artifacts) from base iteration are available\n");
                    prompt.push_str("- You MUST preserve existing code and only make necessary modifications\n");
                }
                InheritanceMode::None => {}
            }
            
            prompt.push_str("\n");
            prompt.push_str("🎯 YOUR TASK:\n");
            prompt.push_str("1. FIRST, list the existing files in the workspace to understand the current structure\n");
            prompt.push_str("2. Read relevant existing code files before making changes\n");
            prompt.push_str("3. Add new features incrementally - DO NOT rewrite from scratch\n");
            prompt.push_str("4. Only modify files that need changes for the new features\n");
            prompt.push_str("5. Preserve all existing functionality\n");
            prompt.push_str("\n");
            prompt.push_str("═══════════════════════════════════════════════════════════════\n\n");
        }

        prompt.push_str(&format!(
            r#"# Coding Task

## Project Context
{}

## Base Instruction
{}

## Task Description
{}

## Working Directory
{}

## Requirements
1. Implement the task according to the description
2. Write clean, maintainable code
3. Ensure the code compiles and runs correctly
4. If you encounter any issues, report them clearly

Please start implementing the task."#,
            project_context,
            CODING_ACTOR_INSTRUCTION,
            task_description,
            self.workspace.display()
        ));

        prompt
    }

    /// Check if the agent is ready
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Get the agent type
    pub fn agent_type(&self) -> &str {
        &self.config.agent_type
    }
}
