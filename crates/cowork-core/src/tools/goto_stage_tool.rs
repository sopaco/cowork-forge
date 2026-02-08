// Goto Stage tool for Check Agent
use crate::data::*;
use crate::storage::*;
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use super::get_required_string_param;

pub struct GotoStageTool;

#[async_trait]
impl Tool for GotoStageTool {
    fn name(&self) -> &str {
        "goto_stage"
    }

    fn description(&self) -> &str {
        "Restart pipeline from a specific stage. Use this when critical issues \
         require going back to an earlier phase. Valid stages: prd, design, plan, coding."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "enum": ["prd", "design", "plan", "coding"],
                    "description": "Which stage to restart from"
                },
                "reason": {
                    "type": "string",
                    "description": "Why the restart is needed"
                }
            },
            "required": ["stage", "reason"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let stage_str = get_required_string_param(&args, "stage")?;
        let reason = get_required_string_param(&args, "reason")?;

        // Parse stage
        let stage = match stage_str {
            "prd" => Stage::Prd,
            "design" => Stage::Design,
            "plan" => Stage::Plan,
            "coding" => Stage::Coding,
            _ => {
                return Ok(json!({
                    "status": "error",
                    "message": format!("Invalid stage: {}", stage_str)
                }));
            }
        };

        // Save detailed feedback to FeedbackHistory for incremental update support
        let feedback = Feedback {
            feedback_type: FeedbackType::QualityIssue,
            severity: Severity::Critical,
            details: reason.to_string(),
            suggested_fix: Some(format!("Restart from {} stage to address the issue", stage_str)),
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) = crate::storage::append_feedback(&feedback) {
            // Log warning but don't fail the operation
            eprintln!("[GotoStageTool] Warning: Failed to save feedback: {}", e);
        }

        // Load or create session meta
        let mut meta = load_session_meta()
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?
            .unwrap_or_else(|| SessionMeta {
                session_id: uuid::Uuid::new_v4().to_string(),
                created_at: chrono::Utc::now(),
                current_stage: Some(Stage::Check),
                restart_reason: None,
            });

        // Set restart information by updating current_stage and reason
        meta.current_stage = Some(stage);
        meta.restart_reason = Some(reason.to_string());

        // Save session meta
        save_session_meta(&meta)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "restart_scheduled",
            "stage": stage_str,
            "reason": reason,
            "message": format!("Pipeline will restart from {} stage. Feedback saved for incremental update.", stage_str)
        }))
    }
}
