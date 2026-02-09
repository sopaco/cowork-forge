// Control tools - provide_feedback, ask_user, etc.
use crate::data::*;
use crate::storage::*;
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use dialoguer::{Confirm, Input};
use serde_json::{json, Value};
use std::sync::Arc;
use super::get_required_string_param;

// ============================================================================
// ProvideFeedbackTool
// ============================================================================

pub struct ProvideFeedbackTool;

#[async_trait]
impl Tool for ProvideFeedbackTool {
    fn name(&self) -> &str {
        "provide_feedback"
    }

    fn description(&self) -> &str {
        "Provide structured feedback to the Actor agent. \
         This feedback will be visible to the Actor in the next iteration."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "description": "The stage providing this feedback (e.g., 'idea', 'prd', 'design', 'plan', 'coding', 'check', 'delivery')",
                    "enum": ["idea", "prd", "design", "plan", "coding", "check", "delivery"]
                },
                "feedback_type": {
                    "type": "string",
                    "enum": ["build_error", "quality_issue", "missing_requirement", "suggestion"],
                },
                "severity": {
                    "type": "string",
                    "enum": ["critical", "major", "minor"],
                },
                "details": {"type": "string"},
                "suggested_fix": {"type": "string"}
            },
            "required": ["stage", "feedback_type", "severity", "details"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let stage = get_required_string_param(&args, "stage")?;

        let feedback_type = match get_required_string_param(&args, "feedback_type")? {
            "build_error" => FeedbackType::BuildError,
            "quality_issue" => FeedbackType::QualityIssue,
            "missing_requirement" => FeedbackType::MissingRequirement,
            _ => FeedbackType::Suggestion,
        };

        let severity = match get_required_string_param(&args, "severity")? {
            "critical" => Severity::Critical,
            "major" => Severity::Major,
            _ => Severity::Minor,
        };

        let feedback = Feedback {
            stage: stage.to_string(),  // 设置 stage 字段
            feedback_type,
            severity,
            details: get_required_string_param(&args, "details")?.to_string(),
            suggested_fix: args
                .get("suggested_fix")
                .and_then(|v| v.as_str())
                .map(String::from),
            timestamp: chrono::Utc::now(),
        };

        append_feedback(&feedback).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "feedback_recorded",
            "message": "Feedback will be available to Actor in next iteration"
        }))
    }
}

// ============================================================================
// AskUserTool
// ============================================================================

pub struct AskUserTool;

#[async_trait]
impl Tool for AskUserTool {
    fn name(&self) -> &str {
        "ask_user"
    }

    fn description(&self) -> &str {
        "Ask the user for confirmation or input via CLI interface."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "question": {
                    "type": "string",
                    "description": "The question to ask the user"
                },
                "question_type": {
                    "type": "string",
                    "enum": ["yes_no", "text_input"],
                    "description": "Type of question"
                }
            },
            "required": ["question", "question_type"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let question = get_required_string_param(&args, "question")?;
        let question_type = get_required_string_param(&args, "question_type")?;

        match question_type {
            "yes_no" => {
                let answer = Confirm::new()
                    .with_prompt(question)
                    .default(false)
                    .interact()
                    .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

                Ok(json!({
                    "answer": answer,
                    "answer_type": "boolean"
                }))
            }
            "text_input" => {
                let answer: String = Input::new()
                    .with_prompt(question)
                    .interact_text()
                    .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

                Ok(json!({
                    "answer": answer,
                    "answer_type": "text"
                }))
            }
            _ => Ok(json!({"error": "Invalid question type"})),
        }
    }
}