// REST API Adapter - Execute REST API integration calls
//
// Provides:
// - HTTP request execution
// - Authentication handling
// - Retry logic with exponential backoff
// - Response parsing

use std::time::Duration;
use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;
use tokio::time::sleep;

use crate::config_definition::{
    IntegrationDefinition, IntegrationEvent, IntegrationResponse,
    AuthType, CredentialSource,
};
use super::adapters::{IntegrationAdapter, AdapterError};

/// REST API adapter for HTTP-based integrations
pub struct RestAdapter {
    http_client: reqwest::Client,
}

impl RestAdapter {
    /// Create a new REST adapter
    pub fn new(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }

    /// Build authentication headers
    fn build_auth_headers(
        &self,
        auth: &crate::config_definition::AuthConfig,
    ) -> Result<HashMap<String, String>, AdapterError> {
        let mut headers = HashMap::new();

        match &auth.credentials {
            CredentialSource::Static { value } => {
                match auth.auth_type {
                    AuthType::ApiKey => {
                        headers.insert("X-API-Key".to_string(), value.clone());
                    }
                    AuthType::BearerToken => {
                        headers.insert("Authorization".to_string(), format!("Bearer {}", value));
                    }
                    AuthType::BasicAuth => {
                        headers.insert("Authorization".to_string(), format!("Basic {}", value));
                    }
                    _ => {}
                }
            }
            CredentialSource::EnvVar { name } => {
                let value = std::env::var(name)
                    .map_err(|_| AdapterError::AuthenticationFailed(
                        format!("Environment variable not found: {}", name)
                    ))?;

                match auth.auth_type {
                    AuthType::ApiKey => {
                        headers.insert("X-API-Key".to_string(), value);
                    }
                    AuthType::BearerToken => {
                        headers.insert("Authorization".to_string(), format!("Bearer {}", value));
                    }
                    AuthType::BasicAuth => {
                        headers.insert("Authorization".to_string(), format!("Basic {}", value));
                    }
                    _ => {}
                }
            }
            CredentialSource::File { path } => {
                let value = std::fs::read_to_string(path)
                    .map_err(|e| AdapterError::AuthenticationFailed(
                        format!("Failed to read credential file: {}", e)
                    ))?;
                let value = value.trim().to_string();

                match auth.auth_type {
                    AuthType::ApiKey => {
                        headers.insert("X-API-Key".to_string(), value);
                    }
                    AuthType::BearerToken => {
                        headers.insert("Authorization".to_string(), format!("Bearer {}", value));
                    }
                    AuthType::BasicAuth => {
                        headers.insert("Authorization".to_string(), format!("Basic {}", value));
                    }
                    _ => {}
                }
            }
            CredentialSource::SecretManager { key } => {
                // Placeholder for secret manager integration
                return Err(AdapterError::AuthenticationFailed(
                    format!("Secret manager not implemented for key: {}", key)
                ));
            }
        }

        Ok(headers)
    }

    /// Interpolate template variables
    fn interpolate_template(&self, template: &str, event: &IntegrationEvent) -> String {
        let mut result = template.to_string();

        // Replace event fields
        result = result.replace("{{integration_id}}", &event.integration_id);
        result = result.replace("{{hook_point}}", &event.hook_point);
        result = result.replace("{{stage_id}}", &event.stage_id);
        result = result.replace("{{iteration_id}}", &event.iteration_id);
        result = result.replace("{{project_id}}", &event.project_id);
        result = result.replace("{{timestamp}}", &event.timestamp);
        result = result.replace("{{event_type}}", &format!("{:?}", event.event_type));

        // Replace data fields
        for (key, value) in &event.data {
            let placeholder = format!("{{{{data.{}}}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }

        result
    }

    /// Execute HTTP request with retry logic
    async fn execute_with_retry(
        &self,
        request_builder: reqwest::RequestBuilder,
        retry_config: &crate::config_definition::RetryConfig,
        timeout_secs: u32,
    ) -> Result<reqwest::Response, AdapterError> {
        let mut last_error = None;
        let mut delay = Duration::from_millis(retry_config.initial_delay_ms);

        for attempt in 0..=retry_config.max_attempts {
            let request = request_builder.try_clone()
                .ok_or_else(|| AdapterError::RequestFailed("Failed to clone request".to_string()))?;

            match request.timeout(Duration::from_secs(timeout_secs as u64)).send().await {
                Ok(response) => {
                    let status = response.status();

                    // Check if we should retry
                    if retry_config.retry_on_status.contains(&status.as_u16())
                        && attempt < retry_config.max_attempts
                    {
                        last_error = Some(AdapterError::RequestFailed(
                            format!("HTTP {} - will retry", status)
                        ));
                        sleep(delay).await;
                        delay = Duration::from_millis(
                            (delay.as_millis() as f32 * retry_config.backoff_multiplier) as u64
                        );
                        delay = delay.min(Duration::from_millis(retry_config.max_delay_ms));
                        continue;
                    }

                    return Ok(response);
                }
                Err(e) => {
                    if attempt < retry_config.max_attempts {
                        last_error = Some(AdapterError::from(e));
                        sleep(delay).await;
                        delay = Duration::from_millis(
                            (delay.as_millis() as f32 * retry_config.backoff_multiplier) as u64
                        );
                        delay = delay.min(Duration::from_millis(retry_config.max_delay_ms));
                        continue;
                    }

                    return Err(AdapterError::from(e));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AdapterError::RetryLimitExceeded("Unknown error".to_string())))
    }
}

#[async_trait]
impl IntegrationAdapter for RestAdapter {
    async fn execute(
        &self,
        integration: &IntegrationDefinition,
        event: IntegrationEvent,
    ) -> Result<IntegrationResponse> {
        // Get base URL
        let base_url = integration.connection.base_url.as_ref()
            .ok_or_else(|| AdapterError::ConfigurationError("No base URL configured".to_string()))?;

        // Determine endpoint
        let (path, method, body_template) = if !integration.connection.endpoints.is_empty() {
            // Use first endpoint
            let endpoint = integration.connection.endpoints.values().next()
                .ok_or_else(|| AdapterError::ConfigurationError("No endpoints configured".to_string()))?;

            (
                endpoint.path.clone(),
                endpoint.method.clone(),
                endpoint.body_template.clone(),
            )
        } else {
            // Use default endpoint based on event type
            let path = format!("/hooks/{}", event.hook_point);
            (path, "POST".to_string(), None)
        };

        // Build URL
        let url = format!("{}{}", base_url.trim_end_matches('/'), path);

        // Build request
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            "PATCH" => self.http_client.patch(&url),
            _ => self.http_client.post(&url),
        };

        // Add headers
        for (key, value) in &integration.connection.headers {
            request_builder = request_builder.header(key, value);
        }

        // Add auth headers
        if let Some(ref auth) = integration.auth {
            let auth_headers = self.build_auth_headers(auth)?;
            for (key, value) in auth_headers {
                request_builder = request_builder.header(key, value);
            }
        }

        // Add body
        if let Some(template) = body_template {
            let body = self.interpolate_template(&template, &event);
            request_builder = request_builder.body(body);
            request_builder = request_builder.header("Content-Type", "application/json");
        } else {
            // Send event as JSON
            request_builder = request_builder.json(&event);
        }

        // Execute request
        let response = self.execute_with_retry(
            request_builder,
            &integration.retry,
            integration.timeout_secs,
        ).await?;

        // Parse response
        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Ok(IntegrationResponse {
                success: false,
                data: None,
                error: Some(format!("HTTP {}: {}", status, error_text)),
                actions: vec![],
            });
        }

        // Try to parse as IntegrationResponse
        let response_text = response.text().await
            .map_err(|e| AdapterError::InvalidResponse(e.to_string()))?;

        // Try to parse as IntegrationResponse
        let integration_response: Result<IntegrationResponse, _> =
            serde_json::from_str(&response_text);

        match integration_response {
            Ok(resp) => Ok(resp),
            Err(_) => {
                // If parsing fails, return the raw response as data
                Ok(IntegrationResponse {
                    success: true,
                    data: Some(serde_json::Value::String(response_text)),
                    error: None,
                    actions: vec![],
                })
            }
        }
    }

    async fn test_connection(&self, integration: &IntegrationDefinition) -> Result<bool> {
        let base_url = integration.connection.base_url.as_ref()
            .ok_or_else(|| AdapterError::ConfigurationError("No base URL configured".to_string()))?;

        // Try to make a simple GET request to the base URL
        let response = self.http_client
            .get(base_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                // Consider any response as successful connection
                Ok(resp.status().is_success() || resp.status().is_client_error())
            }
            Err(e) => {
                Err(AdapterError::ConnectionFailed(e.to_string()).into())
            }
        }
    }

    fn adapter_type(&self) -> &str {
        "rest"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_definition::IntegrationEventType;

    #[test]
    fn test_interpolate_template() {
        let adapter = RestAdapter::new(reqwest::Client::new());

        let event = IntegrationEvent {
            integration_id: "test".to_string(),
            hook_point: "pre_stage".to_string(),
            stage_id: "idea".to_string(),
            iteration_id: "iter-001".to_string(),
            project_id: "proj-001".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            data: vec![("key".to_string(), serde_json::json!("value"))]
                .into_iter()
                .collect(),
            event_type: IntegrationEventType::StageStarted,
        };

        let template = "Stage: {{stage_id}}, Iteration: {{iteration_id}}, Key: {{data.key}}";
        let result = adapter.interpolate_template(template, &event);

        assert_eq!(result, "Stage: idea, Iteration: iter-001, Key: value");
    }

    #[test]
    fn test_interpolate_with_numeric_data() {
        let adapter = RestAdapter::new(reqwest::Client::new());

        let event = IntegrationEvent {
            integration_id: "test".to_string(),
            hook_point: "post_stage".to_string(),
            stage_id: "coding".to_string(),
            iteration_id: "iter-002".to_string(),
            project_id: "proj-002".to_string(),
            timestamp: "2024-01-02T00:00:00Z".to_string(),
            data: vec![("count".to_string(), serde_json::json!(42))]
                .into_iter()
                .collect(),
            event_type: IntegrationEventType::StageCompleted,
        };

        let template = "Count: {{data.count}}";
        let result = adapter.interpolate_template(template, &event);

        assert_eq!(result, "Count: 42");
    }

    #[test]
    fn test_adapter_type() {
        let adapter = RestAdapter::new(reqwest::Client::new());
        assert_eq!(adapter.adapter_type(), "rest");
    }

    #[test]
    fn test_build_auth_headers_api_key() {
        let adapter = RestAdapter::new(reqwest::Client::new());

        let auth = crate::config_definition::AuthConfig {
            auth_type: AuthType::ApiKey,
            credentials: CredentialSource::Static { value: "test-api-key".to_string() },
        };

        let headers = adapter.build_auth_headers(&auth).unwrap();
        assert_eq!(headers.get("X-API-Key"), Some(&"test-api-key".to_string()));
    }

    #[test]
    fn test_build_auth_headers_bearer_token() {
        let adapter = RestAdapter::new(reqwest::Client::new());

        let auth = crate::config_definition::AuthConfig {
            auth_type: AuthType::BearerToken,
            credentials: CredentialSource::Static { value: "test-token".to_string() },
        };

        let headers = adapter.build_auth_headers(&auth).unwrap();
        assert_eq!(headers.get("Authorization"), Some(&"Bearer test-token".to_string()));
    }

    #[test]
    fn test_build_auth_headers_basic_auth() {
        let adapter = RestAdapter::new(reqwest::Client::new());

        let auth = crate::config_definition::AuthConfig {
            auth_type: AuthType::BasicAuth,
            credentials: CredentialSource::Static { value: "dXNlcjpwYXNz".to_string() },
        };

        let headers = adapter.build_auth_headers(&auth).unwrap();
        assert_eq!(headers.get("Authorization"), Some(&"Basic dXNlcjpwYXNz".to_string()));
    }
}
