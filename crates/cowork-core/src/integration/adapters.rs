// Integration Adapters - Trait and error types for integration adapters
//
// Provides:
// - IntegrationAdapter trait for different integration types
// - Adapter error handling

use async_trait::async_trait;
use anyhow::Result;

use crate::config_definition::{
    IntegrationDefinition, IntegrationEvent, IntegrationResponse,
};

/// Adapter error types
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Timeout exceeded")]
    TimeoutExceeded,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Retry limit exceeded: {0}")]
    RetryLimitExceeded(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Integration adapter trait for different integration types
#[async_trait]
pub trait IntegrationAdapter: Send + Sync {
    /// Execute integration call with the given event
    async fn execute(
        &self,
        integration: &IntegrationDefinition,
        event: IntegrationEvent,
    ) -> Result<IntegrationResponse>;

    /// Test the integration connection
    async fn test_connection(&self, integration: &IntegrationDefinition) -> Result<bool>;

    /// Get adapter type name
    fn adapter_type(&self) -> &str;
}

/// No-op adapter for testing
#[allow(dead_code)]
pub struct NoOpAdapter;

#[async_trait]
impl IntegrationAdapter for NoOpAdapter {
    async fn execute(
        &self,
        _integration: &IntegrationDefinition,
        _event: IntegrationEvent,
    ) -> Result<IntegrationResponse> {
        Ok(IntegrationResponse {
            success: true,
            data: None,
            error: None,
            actions: vec![],
        })
    }

    async fn test_connection(&self, _integration: &IntegrationDefinition) -> Result<bool> {
        Ok(true)
    }

    fn adapter_type(&self) -> &str {
        "noop"
    }
}
