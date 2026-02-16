// External Coding Agent - Adapter for external CLI-based coding agents via ACP
// This module provides an alternative to the built-in adk-rust coding agent

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::acp::{AcpClient, AcpTaskResult};
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
    /// ACP client for communication
    client: AcpClient,
    /// Configuration
    config: CodingAgentConfig,
    /// Workspace path
    workspace: PathBuf,
    /// Whether the agent is ready
    ready: bool,
}

impl ExternalCodingAgent {
    /// Create a new External Coding Agent
    pub async fn new(workspace: &PathBuf) -> Result<Self> {
        eprintln!("DEBUG: ExternalCodingAgent::new called with workspace: {}", workspace.display());
        
        let config = load_config()
            .context("Failed to load config")?;
        
        eprintln!("DEBUG: Config loaded, coding_agent.enabled: {}", config.coding_agent.enabled);

        if !config.coding_agent.enabled {
            anyhow::bail!("External coding agent is not enabled in config");
        }

        eprintln!("DEBUG: Creating ACP client with command: {} {:?}", config.coding_agent.command, config.coding_agent.args);
        
        let client = AcpClient::from_config(&config.coding_agent, workspace).await
            .context("Failed to create ACP client")?;

        eprintln!("DEBUG: ACP client created successfully");

        Ok(Self {
            client,
            config: config.coding_agent,
            workspace: workspace.clone(),
            ready: false,
        })
    }

    /// Check if external agent is enabled in config
    pub fn is_enabled() -> Result<bool> {
        let config = load_config()
            .context("Failed to load config")?;
        Ok(config.coding_agent.enabled)
    }

    /// Execute a coding task
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

        // Execute the task
        match self.client.execute_task(&prompt).await {
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
        format!(
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
        )
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

/// Builder for External Coding Agent
pub struct ExternalCodingAgentBuilder {
    workspace: Option<PathBuf>,
    config: Option<CodingAgentConfig>,
}

impl ExternalCodingAgentBuilder {
    pub fn new() -> Self {
        Self {
            workspace: None,
            config: None,
        }
    }

    pub fn workspace(mut self, path: PathBuf) -> Self {
        self.workspace = Some(path);
        self
    }

    pub fn config(mut self, config: CodingAgentConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub async fn build(self) -> Result<ExternalCodingAgent> {
        let workspace = self.workspace
            .context("Workspace not specified")?;

        // If config not provided, load from file
        let config = if let Some(config) = self.config {
            config
        } else {
            load_config()
                .context("Failed to load config")?
                .coding_agent
                .clone()
        };

        if !config.enabled {
            anyhow::bail!("External coding agent is not enabled");
        }

        let client = AcpClient::from_config(&config, &workspace).await
            .context("Failed to create ACP client")?;

        Ok(ExternalCodingAgent {
            client,
            config,
            workspace,
            ready: false,
        })
    }
}

impl Default for ExternalCodingAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let builder = ExternalCodingAgentBuilder::new()
            .workspace(PathBuf::from("/tmp/test"));
        
        // This would fail without proper config
        // let agent = tokio::runtime::Runtime::new().unwrap().block_on(builder.build());
    }
}
