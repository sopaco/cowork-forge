# LLM Client & Rate Limiter Module Documentation

## Overview

The **LLM Client & Rate Limiter** module is a critical infrastructure component in the Cowork Forge system that manages interactions with external Large Language Model (LLM) services while enforcing rate limits to prevent API overuse. This module provides a unified interface for all AI agents to access LLM capabilities while ensuring compliance with external API usage policies.

## Architecture

### Module Structure

```
crates/cowork-core-v2/src/llm/
├── mod.rs          # Module exports and organization
├── config.rs       # Configuration management
└── rate_limiter.rs # Rate limiting implementation
```

### Core Components

#### 1. Rate Limiter (`rate_limiter.rs`)

**Purpose**: Wraps any LLM implementation to enforce rate limits by introducing configurable delays between API calls.

**Key Implementation Details**:
- **Struct**: `RateLimitedLlm` containing an `Arc<dyn Llm>` and `delay_ms` field
- **Thread Safety**: Uses `Arc` for shared ownership across concurrent async tasks
- **Async Support**: Implements `async_trait` for seamless async operation

**Constructor Methods**:
- `new(inner: Arc<dyn Llm>, delay_ms: u64)` - Custom delay configuration
- `with_default_delay(inner: Arc<dyn Llm>)` - Pre-configured 2-second delay (optimized for <30 calls/minute limits)

**Implementation Pattern**:
```rust
async fn generate_content(&self, req: LlmRequest, stream: bool) -> Result<LlmResponseStream> {
    sleep(Duration::from_millis(self.delay_ms)).await;  // Rate limiting
    self.inner.generate_content(req, stream).await      // Delegate to inner LLM
}
```

#### 2. Configuration Management (`config.rs`)

**Purpose**: Loads and manages LLM configuration from TOML files or environment variables.

**Configuration Structure**:
```rust
pub struct LlmConfig {
    pub api_base_url: String,  // OpenAI-compatible endpoint
    pub api_key: String,       // API authentication key
    pub model_name: String,    // Model identifier (e.g., "gpt-4")
}

pub struct ModelConfig {
    pub llm: LlmConfig,        // Complete LLM configuration
}
```

**Configuration Sources**:
- **File-based**: `ModelConfig::from_file(path)` - Loads from `config.toml`
- **Environment-based**: `ModelConfig::from_env()` - Fallback from environment variables

#### 3. Client Factory (`config.rs`)

**Primary Function**: `create_llm_client(config: &LlmConfig) -> Result<Arc<dyn Llm>>`

**Integration Flow**:
1. Creates `OpenAIConfig` using `OpenAIConfig::compatible()` for custom endpoints
2. Instantiates `OpenAIClient` from `adk-model`
3. Automatically wraps with `RateLimitedLlm` using default 2-second delay
4. Returns thread-safe `Arc<dyn Llm>` reference

## Integration with System Architecture

### Dependencies
- **External**: `adk-core`, `adk-model`, `adk-agent` (from adk-rust framework)
- **Internal**: Used by all AI agents through the orchestration pipeline

### Usage in Agent Creation
```rust
// Typical agent creation pattern in agents/mod.rs
pub fn create_idea_agent(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    LlmAgentBuilder::new("idea_agent")
        .instruction(IDEA_AGENT_INSTRUCTION)
        .model(model)  // Rate-limited LLM client
        .tool(Arc::new(WriteFileTool))
        .build()
}
```

### System Integration Points
1. **CLI Entry Point**: Loads configuration and creates the initial LLM client
2. **Agent Orchestration**: Passes rate-limited client to all specialized agents
3. **Pipeline Execution**: Ensures consistent rate limiting across all AI operations

## Rate Limiting Strategy

### Default Configuration
- **Delay**: 2000ms (2 seconds) between API calls
- **Rationale**: Optimized for APIs with <30 calls per minute limits
- **Customization**: Configurable via `delay_ms` parameter

### Implementation Benefits
1. **API Compliance**: Prevents rate limit violations and potential service bans
2. **Cost Control**: Manages API usage to control operational costs
3. **Reliability**: Reduces transient failures due to rate limiting
4. **Predictability**: Ensures consistent system performance

## Operational Characteristics

### Performance Considerations
- **Overhead**: Minimal performance impact (only timing delay)
- **Scalability**: Supports concurrent usage through `Arc` sharing
- **Thread Safety**: Safe for multi-threaded async environments

### Error Handling
- **Transparent Error Propagation**: Errors from inner LLM are passed through unchanged
- **Configuration Validation**: Early validation of API keys and endpoints
- **Graceful Degradation**: Proper error handling for configuration loading failures

## Configuration Management

### Configuration Sources (Priority Order)
1. **config.toml**: Primary configuration file
2. **Environment Variables**: Fallback mechanism
3. **Default Values**: Hardcoded sensible defaults

### Example Configuration
```toml
[llm]
api_base_url = "https://api.openai.com/v1"
api_key = "sk-..."
model_name = "gpt-4"
```

## Best Practices

### Usage Guidelines
1. **Single Instance**: Create one LLM client instance and share via `Arc` across agents
2. **Configuration Validation**: Validate credentials before pipeline execution
3. **Monitoring**: Monitor API usage and adjust rate limits as needed
4. **Error Recovery**: Implement retry logic for transient API failures

### Extension Points
- **Custom Rate Limits**: Modify `delay_ms` based on API provider specifications
- **Additional Providers**: Extend `create_llm_client` for other LLM providers
- **Advanced Throttling**: Implement token-based or cost-based rate limiting

## Conclusion

The LLM Client & Rate Limiter module provides a robust, configurable foundation for AI-powered operations in Cowork Forge. By abstracting LLM interactions behind a rate-limited interface, it enables reliable, cost-effective AI agent execution while maintaining compliance with external API constraints. The module's design emphasizes simplicity, flexibility, and integration with the broader adk-rust agent framework.