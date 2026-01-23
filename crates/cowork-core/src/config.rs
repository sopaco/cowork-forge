use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 大模型配置（从文件加载）
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub llm: LlmConfig,
    pub embedding: EmbeddingConfig,
}

impl ModelConfig {
    /// 从 TOML 文件加载配置
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// 从环境变量加载配置（备用）
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            llm: LlmConfig {
                api_base_url: std::env::var("LLM_API_BASE_URL")?,
                api_key: std::env::var("LLM_API_KEY")?,
                model_name: std::env::var("LLM_MODEL_NAME")?,
            },
            embedding: EmbeddingConfig {
                api_base_url: std::env::var("EMBEDDING_API_BASE_URL")?,
                api_key: std::env::var("EMBEDDING_API_KEY")?,
                model_name: std::env::var("EMBEDDING_MODEL_NAME")?,
            },
        })
    }
}
