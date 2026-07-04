use crate::data::*;
use crate::persistence::*;
use crate::pipeline::set_goto_stage_signal;
use adk_core::{Tool, ToolContext, EventActions};
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

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let stage_str = get_required_string_param(&args, "stage")?;
        let reason = get_required_string_param(&args, "reason")?;

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

        let feedback = Feedback {
            stage: stage_str.to_string(),
            feedback_type: FeedbackType::QualityIssue,
            severity: Severity::Critical,
            details: reason.to_string(),
            suggested_fix: Some(format!("Restart from {} stage to address the issue", stage_str)),
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) = crate::persistence::append_feedback(&feedback) {
            eprintln!("[GotoStageTool] Warning: Failed to save feedback: {}", e);
        }

        let mut meta = load_session_meta()
            .map_err(|e| adk_core::AdkError::tool(e.to_string()))?
            .unwrap_or_else(|| SessionMeta {
                session_id: uuid::Uuid::new_v4().to_string(),
                created_at: chrono::Utc::now(),
                current_stage: Some(Stage::Check),
                restart_reason: None,
            });

        meta.current_stage = Some(stage);
        meta.restart_reason = Some(reason.to_string());

        save_session_meta(&meta)
            .map_err(|e| adk_core::AdkError::tool(e.to_string()))?;

        set_goto_stage_signal(stage_str.to_string(), reason.to_string());

        let mut actions = EventActions::default();
        actions.escalate = true;
        actions.state_delta.insert("goto_stage".to_string(), json!(stage_str));
        actions.state_delta.insert("goto_reason".to_string(), json!(reason));
        ctx.set_actions(actions);

        Ok(json!({
            "status": "goto_stage",
            "stage": stage_str,
            "reason": reason
        }))
    }
}
