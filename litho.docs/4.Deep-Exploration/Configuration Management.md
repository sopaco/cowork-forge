# Configuration Management Module Documentation

## Overview

The Configuration Management module in Cowork Forge provides a centralized system for managing runtime configuration parameters, primarily focusing on Large Language Model (LLM) service configurations. This module enables flexible configuration sourcing from multiple sources while maintaining type safety and providing a factory pattern for creating rate-limited LLM clients.

## Architecture

### Module Structure

The configuration management functionality is implemented across two parallel codebases:

- **Cowork-Core (Legacy)**: `crates/cowork-core/src/config.rs` - Supports both LLM and embedding services
- **Cowork-Core-V2 (Modern)**: `crates/cowork-core-v2/src/llm/config.rs` - LLM-only configuration with enhanced error handling

### Core Components

#### 1. Configuration Models

**Cowork-Core Implementation:**
```rust[derive(Debug, Clone, Serialize, Deserialize)]
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
```

**Cowork-Core-V2 Implementation:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub llm: LlmConfig,
}
```

#### 2. Configuration Loading

The module supports dual sourcing strategy:

- **Primary Source**: TOML configuration files
- **Fallback Source**: Environment variables

**Loading Methods:**
```rust
impl ModelConfig {
    /// Load from TOML file with proper error context
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| "Failed to parse config.toml")?;
        Ok(config)
    }

    /// Load from environment variables with detailed error messages
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
```

#### 3. LLM Client Factory

**Cowork-Core-V2 Enhanced Implementation:**
```rust
pub fn create_llm_client(config: &LlmConfig) -> Result<Arc<dyn Llm>> {
    use crate::llm::rate_limiter::RateLimitedLlm;

    // Create OpenAI config with custom base URL
    let openai_config = OpenAIConfig::compatible(
        &config.api_key,
        &config.api_base_url,
        &config.model_name,
    );

    // Create the OpenAI client
    let client = OpenAIClient::new(openai_config)
        .with_context(|| "Failed to create OpenAI client")?;

    // Wrap with rate limiter (2-second delay for <30 calls/min)
    let rate_limited_client = RateLimitedLlm::with_default_delay(Arc::new(client));

    Ok(Arc::new(rate_limited_client))
}
```

#### 4. Rate Limiting Mechanism

**Rate Limiter Implementation:**
```rust
pub struct RateLimitedLlm {
    inner: Arc<dyn Llm>,
    delay_ms: u64,
}

impl RateLimitedLlm {
    pub fn new(inner: Arc<dyn Llm>, delay_ms: u64) -> Self {
        Self { inner, delay_ms }
    }

    pub fn with_default_delay(inner: Arc<dyn Llm>) -> Self {
        Self::new(inner, 2000) // 2 seconds = 2000ms
    }
}

#[async_trait]
impl Llm for RateLimitedLlm {
    async fn generate_content(
        &self,
        req: LlmRequest,
        stream: bool,
    ) -> adk_core::Result<LlmResponseStream> {
        // Wait before making the API call
        sleep(Duration::from_millis(self.delay_ms)).await;
        
        // Delegate to the inner LLM
        self.inner.generate_content(req, stream).await
    }
}
```

## Configuration Sources

### TOML Configuration File Format

**Example `config.toml`:**
```toml
[llm]
api_base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
model_name = "gpt-4"

# Cowork-Core only:
[embedding]
api_base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
model_name = "text-embedding-3-small"
```

### Environment Variables

**Required Environment Variables:**
- `LLM_API_BASE_URL` - Base URL for LLM API
- `LLM_API_KEY` - API key for authentication
- `LLM_MODEL_NAME` - Model identifier (e.g., "gpt-4")

**Cowork-Core Additional Variables:**
- `EMBEDDING_API_BASE_URL` - Base URL for embedding service
- `EMBEDDING_API_KEY` - API key for embedding service
- `EMBEDDING_MODEL_NAME` - Embedding model identifier

## Integration Points

### 1. CLI Entry Point Integration

The configuration is loaded during CLI initialization:

```rust
// Load configuration from file or environment
let config = match ModelConfig::from_file("config.toml") {
    Ok(config) => config,
    Err(_) => ModelConfig::from_env()?,
};

// Create LLM client for dependency injection
let llm_client = create_llm_client(&config.llm)?;
```

### 2. Agent Orchestration Integration

All AI agents receive the LLM client through dependency injection:

```rust
pub struct PRDAgent {
    llm: Arc<dyn Llm>,
    artifact_store: Arc<dyn ArtifactStore>,
}

impl PRDAgent {
    pub fn new(llm: Arc<dyn Llm>, artifact_store: Arc<dyn ArtifactStore>) -> Self {
        Self { llm, artifact_store }
    }
}
```

### 3. Rate Limiting Integration

The rate limiter enforces API usage compliance:

- **Default Delay**: 2000ms (2 seconds)
- **Rate Limit**: <30 calls per minute
- **Thread Safety**: Uses `Arc<dyn Llm>` for shared ownership

## Error Handling

**Enhanced Error Context (V2):**
- File I/O failures include file path context
- TOML parsing failures include specific parsing context
- Environment variable missing includes variable name
- LLM client creation failures include detailed error context

**Fallback Strategy:**
1. Attempt TOML file loading
2. Fall back to environment variables
3. Provide clear error messages for missing configurations

## Security Considerations

### API Key Protection
- Configuration files should be excluded from version control
- Environment variables recommended for production deployments
- No hardcoded credentials in source code

### Rate Limiting Safety
- Prevents API overuse and potential service bans
- Configurable delay parameters for different service providers
- Thread-safe implementation for concurrent usage

## Testing

**Unit Test Example:**
```rust
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
```

## Best Practices

### Configuration Management
1. **Environment-Specific Configs**: Use different configuration files for development, staging, and production
2. **Secret Management**: Store API keys in environment variables or secure secret managers
3. **Validation**: Implement configuration validation for required parameters
4. **Documentation**: Maintain clear documentation of configuration options

### Performance Considerations
1. **Singleton Pattern**: LLM clients are created once and shared across the application
2. **Connection Pooling**: Leverages underlying client's connection pooling
3. **Rate Limiting**: Prevents service degradation due to API rate limits

## Evolution from V1 to V2

### Key Improvements in V2:
1. **Enhanced Error Handling**: Better context and error messages
2. **Simplified Model**: Removed embedding configuration (LLM-only)
3. **Modern Dependencies**: Uses `adk-rust` 0.2.1 framework
4. **Improved Factory Pattern**: Better LLM client creation with rate limiting
5. **Type Safety**: Enhanced with `anyhow::Context` for better error reporting

The Configuration Management module provides a robust foundation for managing external service configurations while ensuring security, reliability, and maintainability across the Cowork Forge system.