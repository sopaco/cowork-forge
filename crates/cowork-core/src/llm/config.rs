use adk_core::Llm;
use adk_model::openai::{OpenAIClient, OpenAIConfig};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

const CONFIG_FILENAME: &str = "config.toml";
const APP_DIR_NAME: &str = "CoworkCreative";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// Tavily MCP server API key (leave empty to disable)
    #[serde(default)]
    pub tavily_api_key: String,
    /// DeepWiki MCP server enabled
    #[serde(default)]
    pub deepwiki_enabled: bool,
}

impl McpConfig {
    /// Check if any MCP server is configured
    pub fn is_any_enabled(&self) -> bool {
        !self.tavily_api_key.is_empty() || self.deepwiki_enabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbeddingConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalAgentConfig {
    #[serde(default)]
    pub enabled: bool,
    /// 外部 Agent 类型（opencode / claude / codex …）。
    ///
    /// 字段级默认值：手写的 `[coding_agent]` 段可能只写了一半，
    /// 缺字段时用默认值兜底，而不是让**整个配置**加载失败。
    #[serde(default = "default_agent_type")]
    pub agent_type: String,
    #[serde(default = "default_agent_command")]
    pub command: String,
    #[serde(default = "default_agent_args")]
    pub args: Vec<String>,
    #[serde(default)]
    pub workspace_path: Option<String>,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default = "default_transport")]
    pub transport: String,
}

/// 外部 Agent 的默认参数（`Default` 实现与 serde 默认值共用同一份来源）。
const DEFAULT_AGENT_TYPE: &str = "opencode";
const DEFAULT_AGENT_COMMAND: &str = "bun";
const DEFAULT_AGENT_ARGS: [&str; 3] = ["x", "opencode-ai", "acp"];

fn default_agent_type() -> String {
    DEFAULT_AGENT_TYPE.to_string()
}

fn default_agent_command() -> String {
    DEFAULT_AGENT_COMMAND.to_string()
}

fn default_agent_args() -> Vec<String> {
    DEFAULT_AGENT_ARGS.iter().map(|s| s.to_string()).collect()
}

fn default_transport() -> String {
    "stdio".to_string()
}

pub type CodingAgentConfig = ExternalAgentConfig;

// 不能 derive Default：默认的 agent_type/command/args 不是空串，
// 而是可直接跑起来的 opencode 预设。
#[allow(clippy::derivable_impls)]
impl Default for ExternalAgentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            agent_type: default_agent_type(),
            command: default_agent_command(),
            args: default_agent_args(),
            workspace_path: None,
            env: None,
            transport: default_transport(),
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
    #[serde(default)]
    pub mcp: McpConfig,
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
            mcp: McpConfig::default(),
        }
    }
}

impl ModelConfig {
    /// 语义校验：结构层面的缺字段已由 serde 默认值兜底，这里查"类型系统
    /// 表达不了"的业务约束，错误信息要能直接指导用户怎么改。
    pub fn validate(&self) -> Result<()> {
        if self.coding_agent.enabled {
            if self.coding_agent.agent_type.trim().is_empty() {
                bail!("外部 Agent 已启用，但 Agent Type 为空：请在设置里选择 Agent 类型");
            }
            if self.coding_agent.command.trim().is_empty() {
                bail!("外部 Agent 已启用，但 Command 为空：请在设置里填写启动命令");
            }
        }
        Ok(())
    }

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
            mcp: McpConfig::default(),
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
    // 写入前做语义校验：宁可明确报错，也不要写出一份启用了却跑不起来的配置
    config.validate()?;
    let config_path = get_config_path()?;
    ensure_config_dir()?;
    config.save_to_file(&config_path)?;
    tracing::info!("Config saved to: {}", config_path.display());
    Ok(config_path)
}

pub fn create_llm_client(config: &LlmConfig) -> Result<Arc<dyn Llm>> {
    use crate::llm::rate_limiter::TokenBucketRateLimiter;

    let openai_config =
        OpenAIConfig::compatible(&config.api_key, &config.api_base_url, &config.model_name);

    let client =
        OpenAIClient::new(openai_config).with_context(|| "Failed to create OpenAI client")?;

    // Use token bucket rate limiter: 5 burst requests, 30 req/min average
    let rate_limited_client = TokenBucketRateLimiter::with_defaults(Arc::new(client));

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
model_name = "gpt-5"

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
        assert_eq!(config.llm.model_name, "gpt-5");
        assert!(config.coding_agent.enabled);
        assert_eq!(config.embedding.api_base_url, "http://localhost:8001/v1");
    }

    #[test]
    fn test_default_coding_agent() {
        let toml_content = r#"
[llm]
api_base_url = "http://localhost:8000/v1"
api_key = "test-key"
model_name = "gpt-5"
        "#;

        let config: ModelConfig = toml::from_str(toml_content).unwrap();
        assert!(!config.coding_agent.enabled);
        assert!(config.embedding.api_base_url.is_empty());
    }

    #[test]
    fn partial_coding_agent_section_falls_back_to_defaults() {
        // 手写的配置只写了 enabled —— 缺字段必须用默认值兜底，
        // 而不是让整个配置加载失败（否则 GUI 连 LLM 配置都读不出来）。
        let toml_content = r#"
[llm]
api_base_url = "http://localhost:8000/v1"
api_key = "test-key"
model_name = "gpt-5"

[coding_agent]
enabled = false
        "#;

        let config: ModelConfig = toml::from_str(toml_content)
            .expect("partial coding_agent section must still parse");
        assert!(!config.coding_agent.enabled);
        assert_eq!(config.coding_agent.agent_type, DEFAULT_AGENT_TYPE);
        assert_eq!(config.coding_agent.command, DEFAULT_AGENT_COMMAND);
        assert_eq!(config.coding_agent.args, default_agent_args());
        assert_eq!(config.coding_agent.transport, "stdio");
        assert!(config.coding_agent.workspace_path.is_none());
    }

    #[test]
    fn missing_coding_agent_section_uses_whole_struct_default() {
        let toml_content = r#"
[llm]
api_base_url = "http://localhost:8000/v1"
api_key = "test-key"
model_name = "gpt-5"
        "#;

        let config: ModelConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.coding_agent, ExternalAgentConfig::default());
    }

    #[test]
    fn validate_rejects_enabled_agent_without_command() {
        let mut config = ModelConfig::default();
        config.coding_agent.enabled = true;
        config.coding_agent.command = "  ".to_string();

        let err = config.validate().expect_err("must reject empty command");
        let msg = err.to_string();
        assert!(msg.contains("Command"), "error should name the field: {}", msg);
        assert!(msg.contains("启用"), "error should explain the cause: {}", msg);
    }

    #[test]
    fn validate_rejects_enabled_agent_without_agent_type() {
        let mut config = ModelConfig::default();
        config.coding_agent.enabled = true;
        config.coding_agent.agent_type = String::new();

        let err = config.validate().expect_err("must reject empty agent type");
        assert!(err.to_string().contains("Agent Type"), "{}", err);
    }

    #[test]
    fn validate_allows_disabled_agent_with_empty_fields() {
        // 关闭外部 Agent 时，字段可以为空（用户可能刚打开设置就禁用）
        let mut config = ModelConfig::default();
        config.coding_agent.enabled = false;
        config.coding_agent.command = String::new();
        config.coding_agent.agent_type = String::new();

        assert!(config.validate().is_ok());
    }

    #[test]
    fn frontend_payload_with_disabled_agent_deserializes() {
        // 复现 GUI 缺陷：关闭「Enable External Agent」后，antd 表单只提交
        // { enabled: false }，缺 agent_type / command / args。
        // 之前这会让整个命令参数反序列化失败：
        //   invalid args `config` for command `save_app_config`: missing field `agent_type`
        let payload = r#"{
            "llm": {
                "api_base_url": "http://localhost:8000/v1",
                "api_key": "test-key",
                "model_name": "gpt-5"
            },
            "coding_agent": { "enabled": false }
        }"#;

        let config: ModelConfig =
            serde_json::from_str(payload).expect("disabled agent payload must deserialize");
        assert!(!config.coding_agent.enabled);
        assert_eq!(config.coding_agent.agent_type, DEFAULT_AGENT_TYPE);
        assert_eq!(config.coding_agent.command, DEFAULT_AGENT_COMMAND);
        assert!(config.validate().is_ok(), "禁用状态不应被校验拦下");
    }

    #[test]
    fn config_round_trip_omits_none_optional_fields() {        // workspace_path / env 为 None 时 TOML 里不写这两个键
        let config = ModelConfig::default();
        let text = toml::to_string_pretty(&config).expect("serialize");
        assert!(!text.contains("workspace_path"), "{}", text);
        assert!(!text.contains("env"), "{}", text);

        let back: ModelConfig = toml::from_str(&text).expect("deserialize");
        assert_eq!(back.coding_agent, config.coding_agent);
    }
}
