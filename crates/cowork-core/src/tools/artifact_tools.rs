// Artifact operation tools for Delivery Agent
use crate::storage::*;
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use super::get_required_string_param;

// ============================================================================
// SaveIdeaTool
// ============================================================================

pub struct SaveIdeaTool;

#[async_trait]
impl Tool for SaveIdeaTool {
    fn name(&self) -> &str {
        "save_idea"
    }

    fn description(&self) -> &str {
        "MUST USE THIS TOOL to save the Idea markdown document. Call save_idea(content=<markdown>) to save your generated idea content to artifacts/idea.md. This is REQUIRED to complete the idea stage."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Markdown content of the idea document"
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // Notify tool call
        super::notify_tool_call("save_idea", &json!({"file": "idea.md"}));

        let content = get_required_string_param(&args, "content")?;

        match save_idea(content) {
            Ok(_) => {
                super::notify_tool_result("save_idea", &Ok(json!({"status": "success"})));
                Ok(json!({
                    "status": "success",
                    "message": "Idea document saved successfully",
                    "file_path": "artifacts/idea.md"
                }))
            }
            Err(e) => {
                super::notify_tool_result("save_idea", &Err(adk_core::AdkError::Tool(e.to_string())));
                Err(adk_core::AdkError::Tool(e.to_string()))
            }
        }
    }
}

// ============================================================================
// SaveDeliveryReportTool
// ============================================================================

pub struct SaveDeliveryReportTool;

#[async_trait]
impl Tool for SaveDeliveryReportTool {
    fn name(&self) -> &str {
        "save_delivery_report"
    }

    fn description(&self) -> &str {
        "Save the delivery report markdown document."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Markdown content of the delivery report"
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let content = get_required_string_param(&args, "content")?;

        save_delivery_report(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "Delivery report saved successfully",
            "file_path": "artifacts/delivery_report.md"
        }))
    }
}

// ============================================================================
// SavePlanDocTool
// ============================================================================

pub struct SavePlanDocTool;

#[async_trait]
impl Tool for SavePlanDocTool {
    fn name(&self) -> &str {
        "save_plan_doc"
    }

    fn description(&self) -> &str {
        "Save the Implementation Plan markdown document."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Markdown content of the implementation plan document"
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let content = get_required_string_param(&args, "content")?;

        save_plan_doc(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "Plan document saved successfully",
            "file_path": "artifacts/plan.md"
        }))
    }
}

// ============================================================================
// SavePrdDocTool
// ============================================================================

pub struct SavePrdDocTool;

#[async_trait]
impl Tool for SavePrdDocTool {
    fn name(&self) -> &str {
        "save_prd_doc"
    }

    fn description(&self) -> &str {
        "Save the PRD (Product Requirements Document) markdown file."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Markdown content of the PRD document"
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // Notify tool call
        super::notify_tool_call("save_prd_doc", &json!({"file": "prd.md"}));

        let content = get_required_string_param(&args, "content")?;

        match save_prd_doc(content) {
            Ok(_) => {
                super::notify_tool_result("save_prd_doc", &Ok(json!({"status": "success"})));
                Ok(json!({
                    "status": "success",
                    "message": "PRD document saved successfully",
                    "file_path": "artifacts/prd.md"
                }))
            }
            Err(e) => {
                super::notify_tool_result("save_prd_doc", &Err(adk_core::AdkError::Tool(e.to_string())));
                Err(adk_core::AdkError::Tool(e.to_string()))
            }
        }
    }
}

// ============================================================================
// SaveDesignDocTool
// ============================================================================

pub struct SaveDesignDocTool;

#[async_trait]
impl Tool for SaveDesignDocTool {
    fn name(&self) -> &str {
        "save_design_doc"
    }

    fn description(&self) -> &str {
        "Save the Design Document markdown file."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Markdown content of the design document"
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let content = get_required_string_param(&args, "content")?;

        save_design_doc(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "Design document saved successfully",
            "file_path": "artifacts/design.md"
        }))
    }
}

// ============================================================================
// LoadFeedbackHistoryTool
// ============================================================================

pub struct LoadFeedbackHistoryTool;

#[async_trait]
impl Tool for LoadFeedbackHistoryTool {
    fn name(&self) -> &str {
        "load_feedback_history"
    }

    fn description(&self) -> &str {
        "Load the feedback history from a specific stage. Only returns the most recent feedback entry for that stage."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "description": "The stage to load feedback for (e.g., 'idea', 'prd', 'design', 'plan', 'coding', 'check', 'delivery')",
                    "enum": ["idea", "prd", "design", "plan", "coding", "check", "delivery"]
                }
            },
            "required": ["stage"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let stage = args["stage"].as_str()
            .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter: stage".to_string()))?;

        let history = load_feedback_history()
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        // Filter feedbacks by stage and get the most recent one
        let most_recent_feedback = history.feedbacks
            .into_iter()
            .filter(|f| f.stage == stage)
            .max_by_key(|f| f.timestamp);

        match most_recent_feedback {
            Some(feedback) => Ok(json!({
                "has_feedback": true,
                "feedback": feedback
            })),
            None => Ok(json!({
                "has_feedback": false,
                "message": format!("No feedback found for stage '{}'", stage)
            }))
        }
    }
}