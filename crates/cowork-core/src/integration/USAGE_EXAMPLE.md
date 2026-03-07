# Integration Hook Usage Example

This document demonstrates how to integrate hooks into the Stage execution flow.

## Basic Usage

```rust
use cowork_core::integration::{HookManager, HookExecutionContext, HookExecutionResult};
use cowork_core::config_definition::{HookPoint, IntegrationDefinition};

// 1. Create a HookManager
let mut hook_manager = HookManager::new();

// 2. Register integrations
let deployment_integration = IntegrationDefinition::rest_api(
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

hook_manager.register_integration(deployment_integration);

// 3. Execute hooks at stage lifecycle points

// Before stage execution
let pre_context = HookExecutionContext::new(
    "coding",           // stage_id
    "iter-001",         // iteration_id
    "proj-001",         // project_id
    HookPoint::PreExecute,
)
.with_data("stage_type", serde_json::json!("ActorCritic"))
.with_data("agent", serde_json::json!("coding_actor"));

let pre_results = hook_manager.execute_hooks(&stage.hooks, pre_context).await;

// After stage execution
let post_context = HookExecutionContext::new(
    "coding",
    "iter-001",
    "proj-001",
    HookPoint::PostExecute,
)
.with_data("artifacts", serde_json::json!([
    {"path": "output/code", "type": "directory"}
]))
.with_data("duration_secs", serde_json::json!(120));

let actions = hook_manager.execute_and_process(&stage.hooks, post_context).await?;

// 4. Process integration actions
for action in actions {
    match action {
        IntegrationAction::Abort { reason } => {
            tracing::error!("Integration requested abort: {}", reason);
            return Err(anyhow::anyhow!(reason));
        }
        IntegrationAction::GotoStage { stage_id } => {
            tracing::info!("Integration requested goto stage: {}", stage_id);
            // Jump to different stage
        }
        IntegrationAction::ModifyArtifact { path, content } => {
            tracing::info!("Integration requested artifact modification: {}", path);
            // Modify artifact
        }
        _ => {}
    }
}
```

## Integration with StageExecutor

```rust
// In stage_executor.rs

use crate::integration::{HookManager, HookExecutionContext};
use crate::config_definition::HookPoint;

pub struct StageExecutor {
    hook_manager: Arc<HookManager>,
    // ... other fields
}

impl StageExecutor {
    pub async fn execute_stage(&self, stage: &StageDefinition) -> Result<StageResult> {
        // Execute pre-stage hooks
        let pre_context = HookExecutionContext::new(
            &stage.id,
            &self.iteration_id,
            &self.project_id,
            HookPoint::PreExecute,
        );
        
        let pre_actions = self.hook_manager
            .execute_and_process(&stage.hooks, pre_context)
            .await?;
        
        // Process pre-stage actions (e.g., abort, skip)
        for action in pre_actions {
            if let IntegrationAction::Abort { reason } = action {
                return Err(anyhow::anyhow!(reason));
            }
        }
        
        // Execute stage logic
        let result = match stage.stage_type {
            StageType::Simple => self.execute_simple_stage(stage).await?,
            StageType::ActorCritic => self.execute_loop_stage(stage).await?,
        };
        
        // Execute post-stage hooks
        let post_context = HookExecutionContext::new(
            &stage.id,
            &self.iteration_id,
            &self.project_id,
            HookPoint::PostExecute,
        )
        .with_data("success", serde_json::json!(result.is_success))
        .with_data("artifacts", serde_json::json!(result.artifacts));
        
        let post_actions = self.hook_manager
            .execute_and_process(&stage.hooks, post_context)
            .await?;
        
        // Process post-stage actions
        for action in post_actions {
            match action {
                IntegrationAction::GotoStage { stage_id } => {
                    // Modify flow to jump to different stage
                    result.next_stage = Some(stage_id);
                }
                IntegrationAction::ModifyArtifact { path, content } => {
                    // Modify artifact
                    std::fs::write(&path, &content)?;
                }
                _ => {}
            }
        }
        
        Ok(result)
    }
    
    async fn execute_simple_stage(&self, stage: &StageDefinition) -> Result<StageResult> {
        // ... existing logic
    }
    
    async fn execute_loop_stage(&self, stage: &StageDefinition) -> Result<StageResult> {
        // ... existing logic
    }
}
```

## Hook Points

Available hook points in the stage lifecycle:

1. **PreExecute**: Before stage execution starts
   - Use case: Preparation, validation, external system initialization
   
2. **PostExecute**: After stage execution completes
   - Use case: Cleanup, artifact processing, deployment
   
3. **PreConfirmation**: Before HITL confirmation prompt
   - Use case: External approval workflows, additional validation
   
4. **PostConfirmation**: After HITL confirmation received
   - Use case: Logging, notifications, state updates
   
5. **OnFailure**: When stage execution fails
   - Use case: Error handling, rollback, notifications

## Integration Configuration

Example integration configuration in JSON:

```json
{
  "id": "jira-integration",
  "name": "JIRA Issue Tracker",
  "description": "Sync artifacts and status with JIRA",
  "integration_type": "RestApi",
  "connection": {
    "base_url": "https://company.atlassian.net/rest/api/3",
    "endpoints": {
      "update_status": {
        "path": "/issue/{{data.issue_key}}/transitions",
        "method": "POST",
        "body_template": "{\"transition\": {\"id\": \"{{data.transition_id}}\"}}"
      }
    }
  },
  "auth": {
    "auth_type": "BasicAuth",
    "credentials": {
      "EnvVar": { "name": "JIRA_API_TOKEN" }
    }
  },
  "timeout_secs": 15,
  "enabled": true
}
```

Stage hook configuration:

```json
{
  "id": "delivery",
  "name": "Delivery Stage",
  "hooks": [
    {
      "integration_id": "jira-integration",
      "point": "PostExecute",
      "blocking": true,
      "timeout_secs": 30,
      "continue_on_failure": false,
      "params": {
        "issue_key": "{{iteration_id}}",
        "transition_id": "31"
      }
    }
  ]
}
```
