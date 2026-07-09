// Control tools - provide_feedback, ask_user, request_human_review
use crate::data::*;
use crate::persistence::*;
use crate::interaction::{InputOption, InputResponse, MessageLevel};
use crate::tools::hitl_content_tools::get_interaction_backend;
use adk_core::{Tool, ToolContext, EventActions};
use async_trait::async_trait;
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
        "Provide structured feedback to escalate an issue to the executor level. \
         This records the feedback and signals the LoopAgent to exit immediately \
         so the executor can retry the stage with the feedback. \
         Use this ONLY for critical/major issues that need executor-level retry. \
         For minor issues that the Actor can fix in the next loop iteration, \
         just describe them in your response (the Actor sees conversation history). \
         When satisfied, call `exit_loop` instead."
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
                    "enum": [
                        "build_error",
                        "quality_issue",
                        "missing_requirement",
                        "missing_artifact",
                        "architecture_issue",
                        "task_scope_issue",
                        "suggestion"
                    ],
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

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let stage = get_required_string_param(&args, "stage")?;

        let feedback_type = match get_required_string_param(&args, "feedback_type")? {
            "build_error" => FeedbackType::BuildError,
            "quality_issue" => FeedbackType::QualityIssue,
            "missing_requirement" => FeedbackType::MissingRequirement,
            "missing_artifact" => FeedbackType::MissingArtifact,
            "architecture_issue" => FeedbackType::ArchitectureIssue,
            "task_scope_issue" => FeedbackType::TaskScopeIssue,
            _ => FeedbackType::Suggestion,
        };

        let severity = match get_required_string_param(&args, "severity")? {
            "critical" => Severity::Critical,
            "major" => Severity::Major,
            _ => Severity::Minor,
        };

        let feedback = Feedback {
            stage: stage.to_string(),
            feedback_type,
            severity,
            details: get_required_string_param(&args, "details")?.to_string(),
            suggested_fix: args
                .get("suggested_fix")
                .and_then(|v| v.as_str())
                .map(String::from),
            timestamp: chrono::Utc::now(),
        };

        append_feedback(&feedback).map_err(|e| adk_core::AdkError::tool(e.to_string()))?;

        tracing::info!(
            "[ProvideFeedbackTool] Feedback recorded for stage '{}' (severity: {:?}): {}",
            stage, severity, feedback.details.chars().take(100).collect::<String>()
        );

        // Signal the LoopAgent to exit immediately so the executor can retry
        // the stage with the recorded feedback. Per adk-rust semantics, setting
        // `escalate = true` in EventActions causes the LoopAgent to break out
        // of its iteration loop. This is the same mechanism used by ExitLoopTool.
        let mut actions = EventActions::default();
        actions.escalate = true;
        ctx.set_actions(actions);

        Ok(json!({
            "status": "feedback_recorded",
            "message": "Feedback recorded. The loop will exit and the executor will retry the stage with this feedback."
        }))
    }
}

// ============================================================================
// AskUserTool - uses InteractiveBackend trait (works in both CLI and GUI)
// ============================================================================

pub struct AskUserTool;

#[async_trait]
impl Tool for AskUserTool {
    fn name(&self) -> &str {
        "ask_user"
    }

    fn description(&self) -> &str {
        "Ask the user for confirmation or text input."
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

        let interaction = get_interaction_backend()
            .ok_or_else(|| adk_core::AdkError::tool("InteractiveBackend not set - cannot ask user".to_string()))?;

        match question_type {
            "yes_no" => {
                let options = vec![
                    InputOption {
                        id: "yes".to_string(),
                        label: "Yes".to_string(),
                        description: Some("Confirm and proceed".to_string()),
                    },
                    InputOption {
                        id: "no".to_string(),
                        label: "No".to_string(),
                        description: Some("Deny or cancel".to_string()),
                    },
                ];

                let response = interaction.request_input(
                    question,
                    options,
                    None,
                ).await.map_err(|e| adk_core::AdkError::tool(format!("Input error: {}", e)))?;

                let answer = match response {
                    InputResponse::Selection(id) => id == "yes",
                    InputResponse::Text(text) => {
                        let trimmed = text.trim().to_lowercase();
                        trimmed == "yes" || trimmed == "y" || trimmed == "true" || trimmed == "1"
                    }
                    InputResponse::Cancel => false,
                };

                Ok(json!({
                    "answer": answer,
                    "answer_type": "boolean"
                }))
            }
            "text_input" => {
                let response = interaction.request_input(
                    question,
                    vec![],
                    None,
                ).await.map_err(|e| adk_core::AdkError::tool(format!("Input error: {}", e)))?;

                let answer = match response {
                    InputResponse::Text(text) => text,
                    InputResponse::Selection(_) => String::new(),
                    InputResponse::Cancel => String::new(),
                };

                Ok(json!({
                    "answer": answer,
                    "answer_type": "text"
                }))
            }
            _ => Ok(json!({"error": "Invalid question type. Use 'yes_no' or 'text_input'."})),
        }
    }
}

// ============================================================================
// RequestHumanReviewTool - escalate to human when Actor-Critic loop is stuck
// ============================================================================

pub struct RequestHumanReviewTool;

#[async_trait]
impl Tool for RequestHumanReviewTool {
    fn name(&self) -> &str {
        "request_human_review"
    }

    fn description(&self) -> &str {
        "Request human intervention when the Actor-Critic feedback loop cannot resolve an issue. \
         This signals that the agent needs human judgment to proceed, and terminates the current loop."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "reason": {
                    "type": "string",
                    "description": "Why human review is needed (describe the stuck issue)"
                }
            },
            "required": ["reason"]
        }))
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let reason = get_required_string_param(&args, "reason")?;

        tracing::warn!("[RequestHumanReviewTool] Human review requested: {}", reason);

        if let Some(interaction) = get_interaction_backend() {
            interaction.show_message(
                MessageLevel::Warning,
                format!("⚠️ Human review requested\nReason: {}", reason),
            ).await;

            let options = vec![
                InputOption {
                    id: "continue".to_string(),
                    label: "Continue (agent will proceed)".to_string(),
                    description: Some("Allow the agent to continue to the next stage".to_string()),
                },
                InputOption {
                    id: "restart".to_string(),
                    label: "Restart stage".to_string(),
                    description: Some("Send the agent back to try again".to_string()),
                },
            ];

            let _ = interaction.request_input(
                &format!("Human review needed: {}\n\nPlease choose how to proceed:", reason),
                options,
                None,
            ).await;
        }

        let mut actions = EventActions::default();
        actions.escalate = true;
        ctx.set_actions(actions);

        Ok(json!({
            "status": "human_review_requested",
            "reason": reason,
            "message": "Human review has been requested. The loop will terminate."
        }))
    }
}
