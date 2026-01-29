// Tools for modify workflow - Save/Load ChangeRequest and PatchMetadata
use crate::data::*;
use crate::storage::*;
use adk_core::{Tool, ToolContext, AdkError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

// ============================================================================
// SaveChangeRequestTool
// ============================================================================

pub struct SaveChangeRequestTool {
    session_id: String,
}

impl SaveChangeRequestTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for SaveChangeRequestTool {
    fn name(&self) -> &str {
        "save_change_request"
    }

    fn description(&self) -> &str {
        "Save the analyzed ChangeRequest. This is the output of the Change Triage Agent."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "requires_prd_update": {
                    "type": "boolean",
                    "description": "Does PRD need updating?"
                },
                "requires_design_update": {
                    "type": "boolean",
                    "description": "Does design need updating?"
                },
                "requires_plan_update": {
                    "type": "boolean",
                    "description": "Does plan need updating?"
                },
                "requires_code_change": {
                    "type": "boolean",
                    "description": "Does code need changing?"
                },
                "affected_components": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of component IDs affected"
                },
                "affected_features": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of feature IDs affected"
                },
                "risk_level": {
                    "type": "string",
                    "enum": ["low", "medium", "high"],
                    "description": "Risk assessment"
                },
                "estimated_effort": {
                    "type": "string",
                    "description": "Brief effort estimate"
                },
                "acceptance_criteria": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "What defines 'done'"
                },
                "constraints": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Things to preserve"
                }
            },
            "required": ["requires_prd_update", "requires_design_update", "requires_plan_update", "requires_code_change", "risk_level"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // Load existing change request
        let mut change_request = load_change_request(&self.session_id)
            .map_err(|e| AdkError::Tool(e.to_string()))?;

        // Update scope
        change_request.scope.requires_prd_update = args["requires_prd_update"].as_bool().unwrap_or(false);
        change_request.scope.requires_design_update = args["requires_design_update"].as_bool().unwrap_or(false);
        change_request.scope.requires_plan_update = args["requires_plan_update"].as_bool().unwrap_or(false);
        change_request.scope.requires_code_change = args["requires_code_change"].as_bool().unwrap_or(true);

        // Update analysis
        let risk_level = match args["risk_level"].as_str().unwrap_or("medium") {
            "low" => RiskLevel::Low,
            "high" => RiskLevel::High,
            _ => RiskLevel::Medium,
        };

        let analysis = ChangeAnalysis {
            affected_components: args.get("affected_components")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            affected_features: args.get("affected_features")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            risk_level,
            estimated_effort: args.get("estimated_effort")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
        };

        change_request.analysis = Some(analysis);

        // Update acceptance criteria
        if let Some(criteria) = args.get("acceptance_criteria").and_then(|v| v.as_array()) {
            change_request.acceptance_criteria = criteria.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }

        // Update constraints
        if let Some(constraints) = args.get("constraints").and_then(|v| v.as_array()) {
            change_request.constraints = constraints.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }

        // Save
        save_change_request(&self.session_id, &change_request)
            .map_err(|e| AdkError::Tool(e.to_string()))?;

        println!("✅ Change request analyzed and saved");
        println!("   Scope: PRD={}, Design={}, Plan={}, Code={}",
            change_request.scope.requires_prd_update,
            change_request.scope.requires_design_update,
            change_request.scope.requires_plan_update,
            change_request.scope.requires_code_change);
        println!("   Risk: {:?}", change_request.analysis.as_ref().unwrap().risk_level);

        Ok(json!({
            "status": "success",
            "message": "Change request saved successfully"
        }))
    }
}

// ============================================================================
// LoadChangeRequestTool
// ============================================================================

pub struct LoadChangeRequestTool {
    session_id: String,
}

impl LoadChangeRequestTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for LoadChangeRequestTool {
    fn name(&self) -> &str {
        "load_change_request"
    }

    fn description(&self) -> &str {
        "Load the ChangeRequest for this session. Use this to understand what needs to be implemented."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({"type": "object", "properties": {}}))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let change_request = load_change_request(&self.session_id)
            .map_err(|e| AdkError::Tool(e.to_string()))?;

        Ok(serde_json::to_value(change_request)
            .map_err(|e| AdkError::Tool(e.to_string()))?)
    }
}
