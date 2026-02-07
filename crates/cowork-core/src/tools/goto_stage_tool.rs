// Goto Stage tool for Check Agent (Session-scoped)
// NOTE: This module contains V1 legacy tools that are not used in V2 iteration architecture
use crate::data::*;
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct GotoStageTool {
    session_id: String,
}

impl GotoStageTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let stage_str = args["stage"].as_str().unwrap();
        let reason = args["reason"].as_str().unwrap();

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

        // Load or create session meta
        let mut meta = load_session_meta(&self.session_id)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?
            .unwrap_or_else(|| SessionMeta {
                session_id: self.session_id.clone(),
                created_at: chrono::Utc::now(),
                current_stage: Some(Stage::Check),
                restart_reason: None,
            });

        // Set restart information by updating current_stage and reason
        meta.current_stage = Some(stage);
        meta.restart_reason = Some(reason.to_string());

        // Save session meta
        save_session_meta(&self.session_id, &meta)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "restart_scheduled",
            "stage": stage_str,
            "reason": reason,
            "message": format!("Pipeline will restart from {} stage. User should re-run with 'cowork revert --from {}' command.", stage_str, stage_str)
        }))
    }
}
