// Load artifact tools - Load specific artifact files with path restrictions

use crate::storage::artifact_path;
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

// ============================================================================
// LoadIdeaTool
// ============================================================================

pub struct LoadIdeaTool;

#[async_trait]
impl Tool for LoadIdeaTool {
    fn name(&self) -> &str {
        "load_idea"
    }

    fn description(&self) -> &str {
        "Load the Idea markdown document from the artifacts directory."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {},
            "required": []
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let path = artifact_path("idea.md")
            .map_err(|e: anyhow::Error| adk_core::AdkError::Tool(e.to_string()))?;

        let content = std::fs::read_to_string(&path)
            .map_err(|e: std::io::Error| adk_core::AdkError::Tool(format!("Failed to read idea.md: {}", e)))?;

        Ok(json!({
            "status": "success",
            "content": content,
            "file_path": "artifacts/idea.md"
        }))
    }
}

// ============================================================================
// LoadPrdDocTool
// ============================================================================

pub struct LoadPrdDocTool;

#[async_trait]
impl Tool for LoadPrdDocTool {
    fn name(&self) -> &str {
        "load_prd_doc"
    }

    fn description(&self) -> &str {
        "Load the PRD (Product Requirements Document) markdown from the artifacts directory."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {},
            "required": []
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let path = artifact_path("prd.md")
            .map_err(|e: anyhow::Error| adk_core::AdkError::Tool(e.to_string()))?;

        let content = std::fs::read_to_string(&path)
            .map_err(|e: std::io::Error| adk_core::AdkError::Tool(format!("Failed to read prd.md: {}", e)))?;

        Ok(json!({
            "status": "success",
            "content": content,
            "file_path": "artifacts/prd.md"
        }))
    }
}

// ============================================================================
// LoadDesignDocTool
// ============================================================================

pub struct LoadDesignDocTool;

#[async_trait]
impl Tool for LoadDesignDocTool {
    fn name(&self) -> &str {
        "load_design_doc"
    }

    fn description(&self) -> &str {
        "Load the Design Document markdown from the artifacts directory."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {},
            "required": []
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let path = artifact_path("design.md")
            .map_err(|e: anyhow::Error| adk_core::AdkError::Tool(e.to_string()))?;

        let content = std::fs::read_to_string(&path)
            .map_err(|e: std::io::Error| adk_core::AdkError::Tool(format!("Failed to read design.md: {}", e)))?;

        Ok(json!({
            "status": "success",
            "content": content,
            "file_path": "artifacts/design.md"
        }))
    }
}

// ============================================================================
// LoadPlanDocTool
// ============================================================================

pub struct LoadPlanDocTool;

#[async_trait]
impl Tool for LoadPlanDocTool {
    fn name(&self) -> &str {
        "load_plan_doc"
    }

    fn description(&self) -> &str {
        "Load the Implementation Plan markdown from the artifacts directory."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {},
            "required": []
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let path = artifact_path("plan.md")
            .map_err(|e: anyhow::Error| adk_core::AdkError::Tool(e.to_string()))?;

        let content = std::fs::read_to_string(&path)
            .map_err(|e: std::io::Error| adk_core::AdkError::Tool(format!("Failed to read plan.md: {}", e)))?;

        Ok(json!({
            "status": "success",
            "content": content,
            "file_path": "artifacts/plan.md"
        }))
    }
}