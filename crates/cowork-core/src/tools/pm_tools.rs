// Project Manager Agent Tools
// These tools are used by the Project Manager Agent for post-delivery interactions

use crate::data::*;
use crate::data::models::Stage;
use crate::domain::{Iteration, memory::Decision};
use crate::persistence::{IterationStore, ProjectStore};
use crate::storage::{append_feedback, save_session_meta, load_session_meta};
use crate::data::models::SessionMeta;
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use super::get_required_string_param;
use super::get_optional_string_param;

// ============================================================================
// PM Goto Stage Tool - Restart pipeline from a specific stage
// ============================================================================

pub struct PMGotoStageTool;

#[async_trait]
impl Tool for PMGotoStageTool {
    fn name(&self) -> &str {
        "pm_goto_stage"
    }

    fn description(&self) -> &str {
        "Restart the pipeline from a specific stage. Use this when the user wants to fix bugs, \
         modify requirements, or make changes to the project after delivery. \
         Valid stages: idea, prd, design, plan, coding."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "enum": ["idea", "prd", "design", "plan", "coding"],
                    "description": "Which stage to restart from"
                },
                "reason": {
                    "type": "string",
                    "description": "Why the restart is needed (user's request summary)"
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
            "idea" => Stage::Idea,
            "prd" => Stage::Prd,
            "design" => Stage::Design,
            "plan" => Stage::Plan,
            "coding" => Stage::Coding,
            _ => {
                return Ok(json!({
                    "status": "error",
                    "message": format!("Invalid stage: {}. Valid stages are: idea, prd, design, plan, coding", stage_str)
                }));
            }
        };

        // Save feedback for the stage restart
        let feedback = Feedback {
            stage: "pm_agent".to_string(),
            feedback_type: FeedbackType::QualityIssue,
            severity: Severity::Major,
            details: reason.to_string(),
            suggested_fix: Some(format!("Restart from {} stage via PM Agent", stage_str)),
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) = append_feedback(&feedback) {
            eprintln!("[PMGotoStageTool] Warning: Failed to save feedback: {}", e);
        }

        // Load or create session meta
        let mut meta = load_session_meta()
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?
            .unwrap_or_else(|| SessionMeta {
                session_id: uuid::Uuid::new_v4().to_string(),
                created_at: chrono::Utc::now(),
                current_stage: Some(Stage::Delivery),
                restart_reason: None,
            });

        // Set restart information
        meta.current_stage = Some(stage);
        meta.restart_reason = Some(reason.to_string());

        // Save session meta
        save_session_meta(&meta)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": format!("Pipeline will restart from {} stage. Reason: {}", stage_str, reason),
            "target_stage": stage_str
        }))
    }
}

// ============================================================================
// PM Create Iteration Tool - Create a new iteration for new features
// ============================================================================

pub struct PMCreateIterationTool {
    current_iteration_id: String,
}

impl PMCreateIterationTool {
    pub fn new(current_iteration_id: String) -> Self {
        Self { current_iteration_id }
    }
}

#[async_trait]
impl Tool for PMCreateIterationTool {
    fn name(&self) -> &str {
        "pm_create_iteration"
    }

    fn description(&self) -> &str {
        "Create a new iteration for implementing new features or major changes. \
         Use this when the user wants to add new functionality that is separate from the current project."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Title for the new iteration (concise summary of the new feature)"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed description of what the user wants to implement"
                },
                "inheritance": {
                    "type": "string",
                    "enum": ["none", "full", "partial"],
                    "description": "Inheritance mode: none=fresh start, full=copy all artifacts and code, partial=copy code only (default)"
                }
            },
            "required": ["title", "description"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let title = get_required_string_param(&args, "title")?;
        let description = get_required_string_param(&args, "description")?;
        let inheritance = get_optional_string_param(&args, "inheritance")
            .unwrap_or_else(|| "partial".to_string());

        // Load project
        let project_store = ProjectStore::new();
        let mut project = project_store.load()
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?
            .ok_or_else(|| adk_core::AdkError::Tool("Project not initialized".to_string()))?;

        // Determine inheritance mode
        let inheritance_mode = match inheritance.as_str() {
            "none" => crate::domain::InheritanceMode::None,
            "full" => crate::domain::InheritanceMode::Full,
            _ => crate::domain::InheritanceMode::Partial,
        };

        // Create new iteration
        let new_iteration = Iteration::create_evolution(
            &project,
            title.to_string(),
            description.to_string(),
            self.current_iteration_id.clone(),
            inheritance_mode,
        );

        let new_iteration_id = new_iteration.id.clone();

        // Save iteration
        let iteration_store = IterationStore::new();
        iteration_store.save(&new_iteration)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        // Update project
        project_store.add_iteration(&mut project, new_iteration.to_summary())
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": format!("Created new iteration: {}", title),
            "iteration_id": new_iteration_id,
            "title": title,
            "inheritance": inheritance
        }))
    }
}

// ============================================================================
// PM Respond Tool - Respond to user without taking action
// ============================================================================

pub struct PMRespondTool;

#[async_trait]
impl Tool for PMRespondTool {
    fn name(&self) -> &str {
        "pm_respond"
    }

    fn description(&self) -> &str {
        "Respond to the user without taking any action. Use this when answering questions, \
         asking for clarification, or providing information."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "response": {
                    "type": "string",
                    "description": "The response message to the user"
                },
                "ask_clarification": {
                    "type": "boolean",
                    "description": "Whether this response is asking for clarification (optional)"
                }
            },
            "required": ["response"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let response = get_required_string_param(&args, "response")?;
        let ask_clarification = args.get("ask_clarification")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(json!({
            "status": "success",
            "message": response,
            "ask_clarification": ask_clarification
        }))
    }
}

// ============================================================================
// PM Save Decision Tool - Save important decisions to memory
// ============================================================================

pub struct PMSaveDecisionTool {
    iteration_id: String,
}

impl PMSaveDecisionTool {
    pub fn new(iteration_id: String) -> Self {
        Self { iteration_id }
    }
}

#[async_trait]
impl Tool for PMSaveDecisionTool {
    fn name(&self) -> &str {
        "pm_save_decision"
    }

    fn description(&self) -> &str {
        "Save an important decision or preference to project memory. Use this when the user \
         makes a significant decision that should be remembered for future iterations."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Title of the decision"
                },
                "context": {
                    "type": "string",
                    "description": "Background context of the decision"
                },
                "decision": {
                    "type": "string",
                    "description": "The actual decision made"
                },
                "impact": {
                    "type": "string",
                    "description": "Impact analysis of this decision (optional)"
                }
            },
            "required": ["title", "context", "decision"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let title = get_required_string_param(&args, "title")?;
        let context = get_required_string_param(&args, "context")?;
        let decision = get_required_string_param(&args, "decision")?;
        let impact = get_optional_string_param(&args, "impact").unwrap_or_default();

        // Save to memory store
        let memory_store = crate::persistence::MemoryStore::new();
        
        let memory_decision = Decision::new(
            title,
            context,
            format!("{}\n\nImpact: {}", decision, impact),
            &self.iteration_id,
        );

        memory_store.add_decision(memory_decision)
            .map_err(|e: anyhow::Error| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": format!("Decision saved: {}", title)
        }))
    }
}
