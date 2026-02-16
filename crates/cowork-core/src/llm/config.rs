// LLM configuration using adk-rust's OpenAI client
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use adk_model::openai::{OpenAIClient, OpenAIConfig};
use adk_core::Llm;

/// Configuration for LLM from config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
}

/// External Coding Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAgentConfig {
    /// Enable external agent for coding stage
    pub enabled: bool,
    /// Agent type: "iflow", "codex", "gemini", "claude", "opencode"
    pub agent_type: String,
    /// Command to launch the agent
    pub command: String,
    /// Arguments for the command (e.g., ["acp"] for opencode)
    pub args: Vec<String>,
    /// Working directory for the agent
    pub workspace_path: Option<String>,
    /// Environment variables
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Transport mode: "stdio" or "websocket" (default: "stdio")
    #[serde(default = "default_transport")]
    pub transport: String,
}

fn default_transport() -> String {
    "stdio".to_string()
}

/// Alias for ExternalAgentConfig (for backward compatibility)
pub type CodingAgentConfig = ExternalAgentConfig;

impl Default for ExternalAgentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            agent_type: "iflow".to_string(),
            command: "iflow".to_string(),
            args: vec!["--experimental-acp".to_string()],
            workspace_path: None,
            env: None,
            transport: "stdio".to_string(),
        }
    }
}

/// Configuration for the entire model setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub llm: LlmConfig,
    /// Optional external coding agent configuration
    #[serde(default)]
    pub coding_agent: ExternalAgentConfig,
}

impl ModelConfig {
    /// Load from TOML file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| "Failed to parse config.toml")?;
        Ok(config)
    }

    /// Load from environment variables (fallback)
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            llm: LlmConfig {
                api_base_url: std::env::var("LLM_API_BASE_URL")
                    .with_context(|| "LLM_API_BASE_URL not set")?,
                api_key: std::env::var("LLM_API_KEY")
                    .with_context(|| "LLM_API_KEY not set")?,
                model_name: std::env::var("LLM_MODEL_NAME")
                    .with_context(|| "LLM_MODEL_NAME not set")?,
            },
            coding_agent: ExternalAgentConfig::default(),
        })
    }

    /// Check if external coding agent is enabled
    pub fn is_external_coding_agent_enabled(&self) -> bool {
        self.coding_agent.enabled
    }

    /// Get external coding agent configuration
    pub fn get_external_coding_agent_config(&self) -> Option<&ExternalAgentConfig> {
        if self.coding_agent.enabled {
            Some(&self.coding_agent)
        } else {
            None
        }
    }
}

/// Load config from file or environment
/// 
/// This function looks for config.toml in the following locations:
/// 1. Current working directory
/// 2. Directory containing the executable
/// 3. Project root (heuristic: look for Cargo.toml in parent dirs)
/// 4. Environment variables (as fallback)
pub fn load_config() -> Result<ModelConfig, anyhow::Error> {
    use std::path::Path;

    // Try current directory first
    if Path::new("config.toml").exists() {
        tracing::info!("Loading config from current directory");
        return ModelConfig::from_file("config.toml")
            .context("Failed to load config from file");
    }

    // Try executable directory
    if let Ok(exe_path) = std::env::current_exe() {
        let config_path = exe_path.parent().unwrap_or(&exe_path).join("config.toml");
        if config_path.exists() {
            tracing::info!("Loading config from exe directory: {}", config_path.display());
            return ModelConfig::from_file(config_path.to_str().unwrap())
                .context("Failed to load config from exe directory");
        }
        
        // Try project root (look for Cargo.toml in parent dirs)
        let mut path = exe_path.parent().unwrap_or(&exe_path).to_path_buf();
        for _ in 0..5 {
            if path.join("Cargo.toml").exists() {
                let config_path = path.join("config.toml");
                if config_path.exists() {
                    tracing::info!("Loading config from project root: {}", config_path.display());
                    return ModelConfig::from_file(config_path.to_str().unwrap())
                        .context("Failed to load config from project root");
                }
            }
            if !path.pop() {
                break;
            }
        }
    }

    // Fallback to environment variables
    tracing::warn!("config.toml not found, falling back to environment variables");
    ModelConfig::from_env().context("Failed to load config from environment")
}

/// Create an LLM client using adk-rust's OpenAI client with custom base URL
/// 
/// This uses the built-in OpenAIClient from adk-model and configures it
/// to point to a custom OpenAI-compatible endpoint.
/// 
/// **Rate Limiting**: Automatically wraps the client with:
/// 1. Global semaphore to limit concurrent requests (max 1 at a time)
/// 2. Per-request delay (2 seconds) to ensure <30 calls per minute
pub fn create_llm_client(config: &LlmConfig) -> Result<Arc<dyn Llm>> {
    use crate::llm::rate_limiter::RateLimitedLlm;

    // Initialize global rate limiter (max 1 concurrent request)
    // This ensures no more than 1 request is sent at any given time
    crate::llm::rate_limiter::init_global_rate_limiter(1);

    // Create OpenAI config with custom base URL using OpenAIConfig::compatible
    let openai_config = OpenAIConfig::compatible(
        &config.api_key,
        &config.api_base_url,
        &config.model_name,
    );

    // Create the OpenAI client
    let client = OpenAIClient::new(openai_config)
        .with_context(|| "Failed to create OpenAI client")?;

    // Wrap with rate limiter (2-second delay + global semaphore)
    let rate_limited_client = RateLimitedLlm::with_default_delay(Arc::new(client));

    Ok(Arc::new(rate_limited_client))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parse() {
        let toml_content = r#"
[llm]
api_base_url = "http://localhost:8000/v1"
api_key = "test-key"
model_name = "gpt-4"

[coding_agent]
enabled = true
agent_type = "iflow"
command = "iflow"
args = ["--experimental-acp"]
        "#;

        let config: ModelConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.llm.api_base_url, "http://localhost:8000/v1");
        assert_eq!(config.llm.api_key, "test-key");
        assert_eq!(config.llm.model_name, "gpt-4");
        assert!(config.coding_agent.enabled);
        assert_eq!(config.coding_agent.agent_type, "iflow");
        assert_eq!(config.coding_agent.command, "iflow");
    }

    #[test]
    fn test_default_coding_agent() {
        let toml_content = r#"
[llm]
api_base_url = "http://localhost:8000/v1"
api_key = "test-key"
model_name = "gpt-4"
        "#;

        let config: ModelConfig = toml::from_str(toml_content).unwrap();
        assert!(!config.coding_agent.enabled);
    }
}
