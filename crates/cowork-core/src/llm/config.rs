use adk_core::Llm;
use adk_model::openai::{OpenAIClient, OpenAIConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

const CONFIG_FILENAME: &str = "config.toml";
const APP_DIR_NAME: &str = "CoworkCreative";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            api_base_url: String::new(),
            api_key: String::new(),
            model_name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAgentConfig {
    pub enabled: bool,
    pub agent_type: String,
    pub command: String,
    pub args: Vec<String>,
    pub workspace_path: Option<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default = "default_transport")]
    pub transport: String,
}

fn default_transport() -> String {
    "stdio".to_string()
}

pub type CodingAgentConfig = ExternalAgentConfig;

impl Default for ExternalAgentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            agent_type: "opencode".to_string(),
            command: "bun".to_string(),
            args: vec![
                "x".to_string(),
                "opencode-ai".to_string(),
                "acp".to_string(),
            ],
            workspace_path: None,
            env: None,
            transport: "stdio".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub llm: LlmConfig,
    #[serde(default)]
    pub embedding: EmbeddingConfig,
    #[serde(default)]
    pub coding_agent: ExternalAgentConfig,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                api_base_url: String::new(),
                api_key: String::new(),
                model_name: String::new(),
            },
            embedding: EmbeddingConfig::default(),
            coding_agent: ExternalAgentConfig::default(),
        }
    }
}

impl ModelConfig {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Self =
            toml::from_str(&content).with_context(|| "Failed to parse config.toml")?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        std::fs::write(path, content).context("Failed to write config file")?;

        Ok(())
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self {
            llm: LlmConfig {
                api_base_url: std::env::var("LLM_API_BASE_URL")
                    .with_context(|| "LLM_API_BASE_URL not set")?,
                api_key: std::env::var("LLM_API_KEY").with_context(|| "LLM_API_KEY not set")?,
                model_name: std::env::var("LLM_MODEL_NAME")
                    .with_context(|| "LLM_MODEL_NAME not set")?,
            },
            embedding: EmbeddingConfig::default(),
            coding_agent: ExternalAgentConfig::default(),
        })
    }

    pub fn is_external_coding_agent_enabled(&self) -> bool {
        self.coding_agent.enabled
    }

    pub fn get_external_coding_agent_config(&self) -> Option<&ExternalAgentConfig> {
        if self.coding_agent.enabled {
            Some(&self.coding_agent)
        } else {
            None
        }
    }

    pub fn is_embedding_configured(&self) -> bool {
        !self.embedding.api_base_url.is_empty()
            && !self.embedding.api_key.is_empty()
            && !self.embedding.model_name.is_empty()
    }
}

pub fn get_app_data_dir() -> Result<PathBuf> {
    let base_dir = if cfg!(target_os = "windows") {
        let appdata =
            std::env::var("APPDATA").context("Failed to get APPDATA environment variable")?;
        PathBuf::from(appdata)
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").context("Failed to get HOME environment variable")?;
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
    } else {
        let home = std::env::var("HOME").context("Failed to get HOME environment variable")?;
        PathBuf::from(home).join(".config")
    };

    Ok(base_dir.join(APP_DIR_NAME))
}

pub fn get_config_path() -> Result<PathBuf> {
    let app_dir = get_app_data_dir()?;
    Ok(app_dir.join(CONFIG_FILENAME))
}

pub fn ensure_config_dir() -> Result<PathBuf> {
    let app_dir = get_app_data_dir()?;
    std::fs::create_dir_all(&app_dir).context("Failed to create config directory")?;
    Ok(app_dir)
}

pub fn load_config() -> Result<ModelConfig> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        tracing::info!("Loading config from: {}", config_path.display());
        return ModelConfig::from_file(&config_path).context("Failed to load config from file");
    }

    tracing::warn!(
        "Config file not found at {}, falling back to environment variables",
        config_path.display()
    );
    ModelConfig::from_env().context("Failed to load config from environment")
}

pub fn save_config(config: &ModelConfig) -> Result<PathBuf> {
    let config_path = get_config_path()?;
    ensure_config_dir()?;
    config.save_to_file(&config_path)?;
    tracing::info!("Config saved to: {}", config_path.display());
    Ok(config_path)
}

pub fn create_llm_client(config: &LlmConfig) -> Result<Arc<dyn Llm>> {
    use crate::llm::rate_limiter::RateLimitedLlm;

    crate::llm::rate_limiter::init_global_rate_limiter(1);

    let openai_config =
        OpenAIConfig::compatible(&config.api_key, &config.api_base_url, &config.model_name);

    let client =
        OpenAIClient::new(openai_config).with_context(|| "Failed to create OpenAI client")?;

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

[embedding]
api_base_url = "http://localhost:8001/v1"
api_key = "embedding-key"
model_name = "text-embedding-3-small"

[coding_agent]
enabled = true
agent_type = "opencode"
command = "bun"
args = ["x", "opencode-ai", "acp"]
        "#;

        let config: ModelConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.llm.api_base_url, "http://localhost:8000/v1");
        assert_eq!(config.llm.api_key, "test-key");
        assert_eq!(config.llm.model_name, "gpt-4");
        assert!(config.coding_agent.enabled);
        assert_eq!(config.embedding.api_base_url, "http://localhost:8001/v1");
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
        assert!(config.embedding.api_base_url.is_empty());
    }
}
