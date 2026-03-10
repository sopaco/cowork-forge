// Integration Definition - Data structure for external system integrations
//
// Integrations provide hooks to external services like deployment systems,
// requirement management tools, and notification services.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

/// Integration definition for external system connections
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationDefinition {
    /// Unique identifier for this integration
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this integration does
    pub description: Option<String>,
    /// Integration type
    #[serde(default)]
    pub integration_type: IntegrationType,
    
    /// Connection configuration
    pub connection: ConnectionConfig,
    
    /// Authentication configuration
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    
    /// Retry configuration for failed calls
    #[serde(default)]
    pub retry: RetryConfig,
    
    /// Timeout in seconds
    #[serde(default = "default_integration_timeout")]
    pub timeout_secs: u32,
    
    /// Whether this integration is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Metadata for extensions
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_integration_timeout() -> u32 { 30 }
fn default_true() -> bool { true }

/// Integration type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationType {
    /// REST API integration
    #[default]
    RestApi,
    /// Webhook integration (push events)
    Webhook,
    /// Database integration
    Database,
    /// Message queue integration
    MessageQueue,
    /// Custom integration
    Custom(String),
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConnectionConfig {
    /// Base URL for API calls
    pub base_url: Option<String>,
    /// Endpoint templates
    #[serde(default)]
    pub endpoints: HashMap<String, EndpointConfig>,
    /// Default headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Custom connection parameters
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointConfig {
    /// Endpoint path (relative to base_url)
    pub path: String,
    /// HTTP method (default: POST)
    #[serde(default = "default_method")]
    pub method: String,
    /// Request body template
    pub body_template: Option<String>,
    /// Query parameters
    #[serde(default)]
    pub query_params: HashMap<String, String>,
    /// Headers for this endpoint
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Response mapping (JSONPath)
    #[serde(default)]
    pub response_mapping: Option<String>,
}

fn default_method() -> String { "POST".to_string() }

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthConfig {
    /// Authentication type
    #[serde(default)]
    pub auth_type: AuthType,
    /// Credential source
    pub credentials: CredentialSource,
}

/// Authentication type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    #[default]
    None,
    ApiKey,
    BearerToken,
    BasicAuth,
    OAuth2,
    Custom(String),
}

/// Credential source
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialSource {
    /// Static value (not recommended for production)
    Static { value: String },
    /// Environment variable
    EnvVar { name: String },
    /// File path
    File { path: String },
    /// Keychain/secret manager
    SecretManager { key: String },
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetryConfig {
    /// Maximum retry attempts
    #[serde(default)]
    pub max_attempts: u32,
    /// Initial delay in milliseconds
    #[serde(default)]
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds
    #[serde(default)]
    pub max_delay_ms: u64,
    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f32,
    /// HTTP status codes to retry on
    #[serde(default)]
    pub retry_on_status: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            retry_on_status: vec![429, 500, 502, 503, 504],
        }
    }
}

fn default_backoff_multiplier() -> f32 { 2.0 }

/// Integration event payload for hook execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationEvent {
    /// Integration ID that triggered this event
    pub integration_id: String,
    /// Hook point that triggered this event
    pub hook_point: String,
    /// Stage ID where the hook was triggered
    pub stage_id: String,
    /// Iteration ID
    pub iteration_id: String,
    /// Project ID
    pub project_id: String,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Event data (stage output, context, etc.)
    #[serde(default)]
    pub data: HashMap<String, serde_json::Value>,
    /// Event type
    pub event_type: IntegrationEventType,
}

/// Integration event type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationEventType {
    StageStarted,
    StageCompleted,
    StageFailed,
    ConfirmationRequested,
    ConfirmationReceived,
    ArtifactCreated,
    Custom(String),
}

/// Integration response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationResponse {
    /// Whether the call was successful
    pub success: bool,
    /// Response data from the external system
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Actions to take (e.g., abort, retry, modify artifact)
    #[serde(default)]
    pub actions: Vec<IntegrationAction>,
}

/// Action from integration response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationAction {
    /// Continue with normal flow
    Continue,
    /// Abort the stage
    Abort { reason: String },
    /// Retry the hook
    Retry { delay_ms: Option<u64> },
    /// Skip to a specific stage
    GotoStage { stage_id: String },
    /// Modify the artifact
    ModifyArtifact { path: String, content: String },
    /// Custom action
    Custom { action_type: String, params: HashMap<String, serde_json::Value> },
}

impl IntegrationDefinition {
    /// Create a new REST API integration
    pub fn rest_api(id: impl Into<String>, name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            integration_type: IntegrationType::RestApi,
            connection: ConnectionConfig {
                base_url: Some(base_url.into()),
                endpoints: HashMap::new(),
                headers: HashMap::new(),
                params: HashMap::new(),
            },
            auth: None,
            retry: RetryConfig::default(),
            timeout_secs: 30,
            enabled: true,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add an endpoint configuration
    pub fn with_endpoint(mut self, name: impl Into<String>, endpoint: EndpointConfig) -> Self {
        self.connection.endpoints.insert(name.into(), endpoint);
        self
    }
    
    /// Set authentication
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = Some(auth);
        self
    }
    
    /// Set API key authentication from environment variable
    pub fn with_api_key_env(mut self, env_var: impl Into<String>) -> Self {
        self.auth = Some(AuthConfig {
            auth_type: AuthType::ApiKey,
            credentials: CredentialSource::EnvVar { name: env_var.into() },
        });
        self
    }
    
    /// Set bearer token authentication from environment variable
    pub fn with_bearer_token_env(mut self, env_var: impl Into<String>) -> Self {
        self.auth = Some(AuthConfig {
            auth_type: AuthType::BearerToken,
            credentials: CredentialSource::EnvVar { name: env_var.into() },
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integration_definition() {
        let integration = IntegrationDefinition::rest_api(
            "deployment-system",
            "Deployment System",
            "https://deploy.example.com/api/v1"
        )
        .with_endpoint("deploy", EndpointConfig {
            path: "/deployments".to_string(),
            method: "POST".to_string(),
            body_template: Some(r#"{"iteration_id": "{{iteration_id}}", "project_id": "{{project_id}}"}"#.to_string()),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            response_mapping: Some("$.deployment_id".to_string()),
        })
        .with_bearer_token_env("DEPLOY_API_TOKEN");
        
        let json = serde_json::to_string_pretty(&integration).unwrap();
        println!("{}", json);
        
        let parsed: IntegrationDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "deployment-system");
        assert!(parsed.auth.is_some());
    }
}
