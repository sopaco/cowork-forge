// Integration Module - External system integration system
//
// This module provides:
// - Hook management for stage lifecycle events
// - REST API adapter for external service calls
// - Integration execution and error handling

mod hooks;
mod adapters;
mod rest_adapter;

pub use hooks::{HookManager, HookExecutionContext, HookExecutionResult};
pub use adapters::{IntegrationAdapter, AdapterError};
pub use rest_adapter::RestAdapter;

// Re-export from config_definition for convenience
pub use crate::config_definition::{
    IntegrationDefinition, IntegrationType, ConnectionConfig, AuthConfig,
    IntegrationEvent, IntegrationEventType, IntegrationResponse, IntegrationAction,
    CredentialSource, EndpointConfig, RetryConfig,
};
