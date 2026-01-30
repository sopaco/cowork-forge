// Control tools - provide_feedback, ask_user, etc. (Session-scoped)
use crate::data::*;
use crate::storage::*;
use adk_core::{Tool, ToolContext};

use async_trait::async_trait;
use dialoguer::{Confirm, Input};
use serde_json::{json, Value};
use std::sync::Arc;

// ============================================================================
// ProvideFeedbackTool
// ============================================================================

pub struct ProvideFeedbackTool {
    session_id: String,
}

impl ProvideFeedbackTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

// ============================================================================
// RequestReplanningTool
// ============================================================================

pub struct RequestReplanningTool {
    session_id: String,
}

impl RequestReplanningTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for RequestReplanningTool {
    fn name(&self) -> &str {
        "request_replanning"
    }

    fn description(&self) -> &str {
        "Request replanning when you discover fundamental issues with the current plan \
         during implementation. This records the request and provides guidance to revisit \
         the planning phase. Use this for major architectural issues, not minor task adjustments."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "issue_type": {
                    "type": "string",
                    "enum": ["design_flaw", "missing_dependency", "architecture_conflict", "requirement_mismatch"],
                    "description": "Type of issue requiring replanning"
                },
                "severity": {
                    "type": "string",
                    "enum": ["critical", "major", "moderate"],
                    "description": "How severe is this issue"
                },
                "details": {
                    "type": "string",
                    "description": "Detailed description of the problem"
                },
                "affected_features": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Feature IDs affected by this issue"
                },
                "suggested_approach": {
                    "type": "string",
                    "description": "Your suggested approach to resolve this (optional)"
                }
            },
            "required": ["issue_type", "severity", "details"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        use crate::data::{Feedback, FeedbackType, Severity};
        use crate::storage::append_feedback;

        let issue_type = args.get("issue_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("Missing 'issue_type'".to_string()))?;

        let severity_str = args.get("severity")
            .and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("Missing 'severity'".to_string()))?;

        let details = args.get("details")
            .and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("Missing 'details'".to_string()))?;

        let severity = match severity_str {
            "critical" => Severity::Critical,
            "major" => Severity::Major,
            _ => Severity::Minor,
        };

        let affected_features: Vec<String> = args.get("affected_features")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();

        let suggested_approach = args.get("suggested_approach")
            .and_then(|v| v.as_str());

        // Compose detailed feedback message
        let mut feedback_details = format!(
            "REPLANNING REQUEST\n\
             Issue Type: {}\n\
             Severity: {}\n\
             Details: {}\n",
            issue_type, severity_str, details
        );

        if !affected_features.is_empty() {
            feedback_details.push_str(&format!("Affected Features: {}\n", affected_features.join(", ")));
        }

        if let Some(approach) = suggested_approach {
            feedback_details.push_str(&format!("Suggested Approach: {}\n", approach));
        }

        // Record as critical feedback
        let feedback = Feedback {
            feedback_type: FeedbackType::MissingRequirement, // Use this to indicate planning issue
            severity,
            details: feedback_details.clone(),
            suggested_fix: suggested_approach.map(String::from),
            timestamp: chrono::Utc::now(),
        };

        append_feedback(&self.session_id, &feedback)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        // Print warning to console
        println!("\n⚠️  REPLANNING REQUESTED ⚠️");
        println!("Type: {} | Severity: {}", issue_type, severity_str);
        println!("Details: {}", details);
        if !affected_features.is_empty() {
            println!("Affected: {}", affected_features.join(", "));
        }
        println!();

        let message = format!(
            "Replanning request recorded with {} severity. \
             The coding loop will continue, but this issue should be addressed. \
             Consider using 'goto_stage' in the check phase if fundamental changes are needed.",
            severity_str
        );

        Ok(json!({
            "status": "replanning_requested",
            "issue_type": issue_type,
            "severity": severity_str,
            "affected_features": affected_features,
            "message": message,
            "guidance": "Continue with current implementation if possible, or mark tasks as blocked. \
                        The Check Agent will review this request and may trigger goto_stage if needed."
        }))
    }
}


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
            "required": ["feedback_type", "severity", "details"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let feedback_type_str = args.get("feedback_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter 'feedback_type'".to_string()))?;
        
        let feedback_type = match feedback_type_str {
            "build_error" => FeedbackType::BuildError,
            "quality_issue" => FeedbackType::QualityIssue,
            "missing_requirement" => FeedbackType::MissingRequirement,
            _ => FeedbackType::Suggestion,
        };

        let severity_str = args.get("severity")
            .and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter 'severity'".to_string()))?;
        
        let severity = match severity_str {
            "critical" => Severity::Critical,
            "major" => Severity::Major,
            _ => Severity::Minor,
        };

        let details = args.get("details")
            .and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter 'details'".to_string()))?
            .to_string();

        let feedback = Feedback {
            feedback_type,
            severity,
            details,
            suggested_fix: args
                .get("suggested_fix")
                .and_then(|v| v.as_str())
                .map(String::from),
            timestamp: chrono::Utc::now(),
        };

        append_feedback(&self.session_id, &feedback).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

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
        let question = args["question"].as_str().unwrap();
        let question_type = args["question_type"].as_str().unwrap();

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

// ============================================================================
// RequestHumanReviewTool
// ============================================================================

pub struct RequestHumanReviewTool {
    session_id: String,
}

impl RequestHumanReviewTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for RequestHumanReviewTool {
    fn name(&self) -> &str {
        "request_human_review"
    }

    fn description(&self) -> &str {
        "Request human intervention when you detect an infinite loop, unclear situation, \
         or when you're about to repeat the same feedback. This prevents endless loops \
         and escalates issues that require human judgment."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "reason": {
                    "type": "string",
                    "description": "Brief reason for requesting human review (e.g., 'Detected infinite loop', 'Unclear if task is non-core')"
                },
                "details": {
                    "type": "string",
                    "description": "Detailed explanation of the situation, including what you observed and why you cannot proceed"
                },
                "suspected_issue": {
                    "type": "string",
                    "enum": ["infinite_loop", "unclear_requirements", "actor_not_responding", "hallucination_detected", "other"],
                    "description": "Type of issue detected"
                }
            },
            "required": ["reason", "details"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        use crate::data::{Feedback, FeedbackType, Severity};
        use crate::storage::append_feedback;

        let reason = args.get("reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("Missing 'reason'".to_string()))?;

        let details = args.get("details")
            .and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("Missing 'details'".to_string()))?;

        let suspected_issue = args.get("suspected_issue")
            .and_then(|v| v.as_str())
            .unwrap_or("other");

        // Record as critical feedback requiring human review
        let feedback_details = format!(
            "🚨 HUMAN REVIEW REQUESTED 🚨\n\
             Reason: {}\n\
             Suspected Issue: {}\n\
             Details: {}\n\
             \n\
             The Critic agent has detected a situation that requires human intervention. \
             Please review the session logs and decide how to proceed.",
            reason, suspected_issue, details
        );

        let feedback = Feedback {
            feedback_type: FeedbackType::Suggestion, // Use suggestion type for human review
            severity: Severity::Critical,
            details: feedback_details.clone(),
            suggested_fix: Some("Human review required - please examine session state and provide guidance".to_string()),
            timestamp: chrono::Utc::now(),
        };

        append_feedback(&self.session_id, &feedback)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        // Print prominent warning to console
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║  🚨 HUMAN REVIEW REQUESTED - CRITIC NEEDS HELP 🚨        ║");
        println!("╚═══════════════════════════════════════════════════════════╝");
        println!("Reason: {}", reason);
        println!("Type: {}", suspected_issue);
        println!("Details: {}", details);
        println!("═══════════════════════════════════════════════════════════\n");

        // Return error to stop the loop - this will trigger ResilientAgent HITL
        Err(adk_core::AdkError::Agent(format!(
            "Human review requested: {}. The loop will stop to prevent infinite iteration.",
            reason
        )))
    }
}
