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
        "Save the Idea markdown document."
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
        let content = get_required_string_param(&args, "content")?;

        save_idea(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "Idea document saved successfully",
            "file_path": "artifacts/idea.md"
        }))
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
        let content = get_required_string_param(&args, "content")?;

        save_prd_doc(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "PRD document saved successfully",
            "file_path": "artifacts/prd.md"
        }))
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
        "Load the feedback history from all development stages."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {}
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let history = load_feedback_history()
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(serde_json::to_value(history)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?)
    }
}