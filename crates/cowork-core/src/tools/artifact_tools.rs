// Artifact operation tools for Delivery Agent
use crate::storage::*;
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

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
        let content = args["content"].as_str().unwrap();
        
        save_delivery_report(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "Delivery report saved successfully"
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
        let content = args["content"].as_str().unwrap();
        
        save_prd_doc(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "PRD document saved successfully"
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
        let content = args["content"].as_str().unwrap();
        
        save_design_doc(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "Design document saved successfully"
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
