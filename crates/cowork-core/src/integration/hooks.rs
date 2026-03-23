// Hook Manager - Manage and execute integration hooks
//
// Provides:
// - Hook registration and execution
// - Hook execution context
// - Error handling and retry logic

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, Context};

use crate::config_definition::{
    HookConfig, HookPoint, IntegrationDefinition, IntegrationEvent,
    IntegrationEventType, IntegrationResponse, IntegrationAction,
};
use super::adapters::IntegrationAdapter;
use super::rest_adapter::RestAdapter;

/// Hook execution context
#[derive(Debug, Clone)]
pub struct HookExecutionContext {
    /// Stage ID where the hook is triggered
    pub stage_id: String,
    /// Iteration ID
    pub iteration_id: String,
    /// Project ID
    pub project_id: String,
    /// Hook point (pre_stage, post_stage, etc.)
    pub hook_point: HookPoint,
    /// Additional context data
    pub data: HashMap<String, serde_json::Value>,
}

impl HookExecutionContext {
    pub fn new(
        stage_id: impl Into<String>,
        iteration_id: impl Into<String>,
        project_id: impl Into<String>,
        hook_point: HookPoint,
    ) -> Self {
        Self {
            stage_id: stage_id.into(),
            iteration_id: iteration_id.into(),
            project_id: project_id.into(),
            hook_point,
            data: HashMap::new(),
        }
    }
    
    /// Add context data
    pub fn with_data(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
    
    /// Convert to IntegrationEvent
    pub fn to_event(&self, integration_id: impl Into<String>) -> IntegrationEvent {
        IntegrationEvent {
            integration_id: integration_id.into(),
            hook_point: format!("{:?}", self.hook_point),
            stage_id: self.stage_id.clone(),
            iteration_id: self.iteration_id.clone(),
            project_id: self.project_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: self.data.clone(),
            event_type: match self.hook_point {
                HookPoint::PreExecute => IntegrationEventType::StageStarted,
                HookPoint::PostExecute => IntegrationEventType::StageCompleted,
                HookPoint::OnFailure => IntegrationEventType::StageFailed,
                HookPoint::PreConfirmation => IntegrationEventType::ConfirmationRequested,
                HookPoint::PostConfirmation => IntegrationEventType::ConfirmationReceived,
            },
        }
    }
}

/// Hook execution result
#[derive(Debug, Clone)]
pub struct HookExecutionResult {
    /// Integration ID
    pub integration_id: String,
    /// Whether the hook execution was successful
    pub success: bool,
    /// Response from the integration
    pub response: Option<IntegrationResponse>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Hook manager for executing integration hooks
pub struct HookManager {
    /// Registered integrations by ID
    integrations: HashMap<String, IntegrationDefinition>,
    /// REST adapter for API calls
    rest_adapter: Arc<RestAdapter>,
}

impl HookManager {
    /// Create a new hook manager
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self {
            integrations: HashMap::new(),
            rest_adapter: Arc::new(RestAdapter::new(http_client)),
        }
    }
    
    /// Register an integration
    pub fn register_integration(&mut self, integration: IntegrationDefinition) {
        self.integrations.insert(integration.id.clone(), integration);
    }
    
    /// Remove an integration
    pub fn remove_integration(&mut self, integration_id: &str) {
        self.integrations.remove(integration_id);
    }
    
    /// Get an integration by ID
    pub fn get_integration(&self, integration_id: &str) -> Option<&IntegrationDefinition> {
        self.integrations.get(integration_id)
    }
    
    /// List all integrations
    pub fn list_integrations(&self) -> Vec<&IntegrationDefinition> {
        self.integrations.values().collect()
    }
    
    /// Execute a hook for a specific integration
    pub async fn execute_hook(
        &self,
        integration_id: &str,
        context: HookExecutionContext,
    ) -> Result<HookExecutionResult> {
        let integration = self.integrations.get(integration_id)
            .with_context(|| format!("Integration not found: {}", integration_id))?;
        
        if !integration.enabled {
            return Ok(HookExecutionResult {
                integration_id: integration_id.to_string(),
                success: true,
                response: None,
                error: Some("Integration is disabled".to_string()),
            });
        }
        
        // Create integration event
        let event = context.to_event(integration_id);
        
        // Execute based on integration type
        let response = match integration.integration_type {
            crate::config_definition::IntegrationType::RestApi => {
                self.rest_adapter.execute(integration, event).await
            }
            crate::config_definition::IntegrationType::Webhook => {
                // Webhook is similar to REST API
                self.rest_adapter.execute(integration, event).await
            }
            _ => {
                // Unsupported integration type
                return Ok(HookExecutionResult {
                    integration_id: integration_id.to_string(),
                    success: false,
                    response: None,
                    error: Some(format!("Unsupported integration type: {:?}", integration.integration_type)),
                });
            }
        };
        
        match response {
            Ok(resp) => Ok(HookExecutionResult {
                integration_id: integration_id.to_string(),
                success: resp.success,
                response: Some(resp),
                error: None,
            }),
            Err(e) => Ok(HookExecutionResult {
                integration_id: integration_id.to_string(),
                success: false,
                response: None,
                error: Some(e.to_string()),
            }),
        }
    }
    
    /// Execute all hooks for a specific hook point
    pub async fn execute_hooks(
        &self,
        hooks: &[HookConfig],
        context: HookExecutionContext,
    ) -> Vec<HookExecutionResult> {
        let mut results = Vec::new();
        
        for hook in hooks {
            // Check if hook point matches
            if hook.point != context.hook_point {
                continue;
            }
            
            // Execute the hook
            let result = self.execute_hook(&hook.integration_id, context.clone()).await;
            
            match result {
                Ok(r) => results.push(r),
                Err(e) => results.push(HookExecutionResult {
                    integration_id: hook.integration_id.clone(),
                    success: false,
                    response: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        
        results
    }
    
    /// Execute hooks and process actions
    pub async fn execute_and_process(
        &self,
        hooks: &[HookConfig],
        context: HookExecutionContext,
    ) -> Result<Vec<IntegrationAction>> {
        let results = self.execute_hooks(hooks, context).await;
        
        let mut actions = Vec::new();
        
        for result in results {
            if !result.success {
                if let Some(error) = result.error {
                    tracing::warn!(
                        "Hook execution failed for integration {}: {}",
                        result.integration_id, error
                    );
                }
                continue;
            }
            
            if let Some(response) = result.response {
                actions.extend(response.actions);
            }
        }
        
        Ok(actions)
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_definition::HookPoint;
    
    #[test]
    fn test_hook_context() {
        let context = HookExecutionContext::new(
            "idea",
            "iter-001",
            "proj-001",
            HookPoint::PreExecute,
        )
        .with_data("test", serde_json::json!("value"));
        
        assert_eq!(context.stage_id, "idea");
        assert_eq!(context.hook_point, HookPoint::PreExecute);
    }
    
    #[test]
    fn test_hook_manager_creation() {
        let manager = HookManager::new();
        assert!(manager.list_integrations().is_empty());
    }
}
