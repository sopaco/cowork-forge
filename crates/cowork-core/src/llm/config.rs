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

/// Configuration for the entire model setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub llm: LlmConfig,
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
        })
    }
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
        "#;

        let config: ModelConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.llm.api_base_url, "http://localhost:8000/v1");
        assert_eq!(config.llm.api_key, "test-key");
        assert_eq!(config.llm.model_name, "gpt-4");
    }
}
