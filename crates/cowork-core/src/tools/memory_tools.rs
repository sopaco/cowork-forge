// Memory Tools - Tools for the iteration-based memory system
//
// These tools allow agents to:
// - Query memory (project and iteration level)
// - Save iteration memories (insights, issues, learnings)
// - Promote iteration memories to project level (decisions, patterns)

use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use crate::domain::{
    Decision, Importance, IterationMemory, Learning, MemoryQuery, MemoryQueryType,
    MemoryScope, Pattern,
};
use crate::persistence::MemoryStore;

// ============================================================================
// Query Memory Tool
// ============================================================================

pub struct QueryMemoryTool {
    iteration_id: String,
}

impl QueryMemoryTool {
    pub fn new(iteration_id: String) -> Self {
        Self { iteration_id }
    }
}

#[async_trait]
impl Tool for QueryMemoryTool {
    fn name(&self) -> &str {
        "query_memory"
    }

    fn description(&self) -> &str {
        "Query memory to retrieve decisions, patterns, and insights. Use this to understand project context and previous experiences."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": "string",
                    "description": "Memory scope: 'project' (project-level only), 'iteration' (current iteration only), or 'smart' (merged, recommended)",
                    "enum": ["project", "iteration", "smart"],
                    "default": "smart"
                },
                "query_type": {
                    "type": "string",
                    "description": "Type of memory to query: 'decisions', 'patterns', 'insights', or 'all'",
                    "enum": ["decisions", "patterns", "insights", "all"],
                    "default": "all"
                },
                "keywords": {
                    "type": "array",
                    "description": "Keywords for filtering results (optional)",
                    "items": {"type": "string"},
                    "default": []
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum results per category. Default: 20",
                    "default": 20
                }
            },
            "required": []
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let scope_str = args.get("scope").and_then(|v| v.as_str()).unwrap_or("smart");
        let query_type_str = args.get("query_type").and_then(|v| v.as_str()).unwrap_or("all");
        let keywords: Vec<String> = args.get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as usize;

        let scope = match scope_str {
            "project" => MemoryScope::Project,
            "iteration" => MemoryScope::Iteration,
            "smart" => MemoryScope::Smart,
            _ => MemoryScope::Smart,
        };

        let query_type = match query_type_str {
            "decisions" => MemoryQueryType::Decisions,
            "patterns" => MemoryQueryType::Patterns,
            "insights" => MemoryQueryType::Insights,
            "all" => MemoryQueryType::All,
            _ => MemoryQueryType::All,
        };

        let query = MemoryQuery {
            scope,
            query_type,
            keywords: keywords.clone(),
            limit: Some(limit),
        };

        let store = MemoryStore::new();
        
        let result = store.query(&query, Some(&self.iteration_id))
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to query memory: {}", e)))?;

        Ok(json!({
            "decisions": result.decisions,
            "patterns": result.patterns,
            "insights": result.insights,
            "total_decisions": result.decisions.len(),
            "total_patterns": result.patterns.len(),
            "total_insights": result.insights.len(),
            "context_string": result.to_context_string()
        }))
    }
}

// ============================================================================
// Save Insight Tool
// ============================================================================

pub struct SaveInsightTool {
    iteration_id: String,
}

impl SaveInsightTool {
    pub fn new(iteration_id: String) -> Self {
        Self { iteration_id }
    }
}

#[async_trait]
impl Tool for SaveInsightTool {
    fn name(&self) -> &str {
        "save_insight"
    }

    fn description(&self) -> &str {
        "Save an insight to the current iteration's memory. Use this to record important observations, discoveries, or realizations during development."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "description": "The current stage (e.g., 'idea', 'prd', 'design', 'plan', 'coding', 'check')"
                },
                "content": {
                    "type": "string",
                    "description": "The insight content (what you discovered or realized)"
                },
                "importance": {
                    "type": "string",
                    "description": "Importance level: 'critical', 'important', or 'normal'",
                    "enum": ["critical", "important", "normal"],
                    "default": "important"
                }
            },
            "required": ["stage", "content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let stage = args.get("stage").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("stage is required".to_string()))?;
        let content = args.get("content").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("content is required".to_string()))?;
        let importance_str = args.get("importance").and_then(|v| v.as_str()).unwrap_or("important");

        let importance = match importance_str {
            "critical" => Importance::Critical,
            "important" => Importance::Important,
            "normal" => Importance::Normal,
            _ => Importance::Important,
        };

        let store = MemoryStore::new();
        let mut memory = store.load_iteration_memory(&self.iteration_id)
            .unwrap_or_else(|_| IterationMemory::new(&self.iteration_id));

        memory.insights.push(crate::domain::Insight {
            stage: stage.to_string(),
            content: content.to_string(),
            importance,
            created_at: chrono::Utc::now(),
        });

        store.save_iteration_memory(&memory)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to save insight: {}", e)))?;

        Ok(json!({
            "message": "Insight saved successfully",
            "iteration_id": self.iteration_id,
            "total_insights": memory.insights.len()
        }))
    }
}

// ============================================================================
// Save Issue Tool
// ============================================================================

pub struct SaveIssueTool {
    iteration_id: String,
}

impl SaveIssueTool {
    pub fn new(iteration_id: String) -> Self {
        Self { iteration_id }
    }
}

#[async_trait]
impl Tool for SaveIssueTool {
    fn name(&self) -> &str {
        "save_issue"
    }

    fn description(&self) -> &str {
        "Save an issue to the current iteration's memory. Use this to record problems, bugs, or obstacles encountered during development."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "description": "The current stage where the issue occurred"
                },
                "content": {
                    "type": "string",
                    "description": "The issue description (what problem occurred)"
                }
            },
            "required": ["stage", "content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let stage = args.get("stage").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("stage is required".to_string()))?;
        let content = args.get("content").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("content is required".to_string()))?;

        let store = MemoryStore::new();
        let mut memory = store.load_iteration_memory(&self.iteration_id)
            .unwrap_or_else(|_| IterationMemory::new(&self.iteration_id));

        memory.issues.push(crate::domain::Issue {
            stage: stage.to_string(),
            content: content.to_string(),
            resolved: false,
            created_at: chrono::Utc::now(),
            resolved_at: None,
        });

        store.save_iteration_memory(&memory)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to save issue: {}", e)))?;

        Ok(json!({
            "message": "Issue saved successfully",
            "iteration_id": self.iteration_id,
            "total_issues": memory.issues.len()
        }))
    }
}

// ============================================================================
// Save Learning Tool
// ============================================================================

pub struct SaveLearningTool {
    iteration_id: String,
}

impl SaveLearningTool {
    pub fn new(iteration_id: String) -> Self {
        Self { iteration_id }
    }
}

#[async_trait]
impl Tool for SaveLearningTool {
    fn name(&self) -> &str {
        "save_learning"
    }

    fn description(&self) -> &str {
        "Save a learning to the current iteration's memory. Use this to record knowledge gained, lessons learned, or skills developed."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The learning content (what you learned)"
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let content = args.get("content").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("content is required".to_string()))?;

        let store = MemoryStore::new();
        let mut memory = store.load_iteration_memory(&self.iteration_id)
            .unwrap_or_else(|_| IterationMemory::new(&self.iteration_id));

        memory.learnings.push(Learning {
            content: content.to_string(),
            created_at: chrono::Utc::now(),
        });

        store.save_iteration_memory(&memory)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to save learning: {}", e)))?;

        Ok(json!({
            "message": "Learning saved successfully",
            "iteration_id": self.iteration_id,
            "total_learnings": memory.learnings.len()
        }))
    }
}

// ============================================================================
// Promote to Decision Tool
// ============================================================================

pub struct PromoteToDecisionTool {
    iteration_id: String,
}

impl PromoteToDecisionTool {
    pub fn new(iteration_id: String) -> Self {
        Self { iteration_id }
    }
}

#[async_trait]
impl Tool for PromoteToDecisionTool {
    fn name(&self) -> &str {
        "promote_to_decision"
    }

    fn description(&self) -> &str {
        "Promote an insight or learning to a project-level decision. Use this for important decisions that should be remembered across iterations."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Decision title (concise summary)"
                },
                "context": {
                    "type": "string",
                    "description": "Context and background for this decision"
                },
                "decision": {
                    "type": "string",
                    "description": "The actual decision made"
                },
                "consequences": {
                    "type": "array",
                    "description": "Expected consequences of this decision",
                    "items": {"type": "string"},
                    "default": []
                }
            },
            "required": ["title", "context", "decision"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let title = args.get("title").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("title is required".to_string()))?;
        let context = args.get("context").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("context is required".to_string()))?;
        let decision_str = args.get("decision").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("decision is required".to_string()))?;
        let consequences: Vec<String> = args.get("consequences")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let mut new_decision = Decision::new(title, context, decision_str, &self.iteration_id);
        new_decision.consequences = consequences;

        let store = MemoryStore::new();
        store.add_decision(new_decision)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to promote to decision: {}", e)))?;

        Ok(json!({
            "message": "Promoted to project decision successfully",
            "iteration_id": self.iteration_id
        }))
    }
}

// ============================================================================
// Promote to Pattern Tool
// ============================================================================

pub struct PromoteToPatternTool {
    iteration_id: String,
}

impl PromoteToPatternTool {
    pub fn new(iteration_id: String) -> Self {
        Self { iteration_id }
    }
}

#[async_trait]
impl Tool for PromoteToPatternTool {
    fn name(&self) -> &str {
        "promote_to_pattern"
    }

    fn description(&self) -> &str {
        "Promote an insight or learning to a project-level pattern. Use this for reusable solutions or best practices that apply across iterations."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Pattern name"
                },
                "description": {
                    "type": "string",
                    "description": "Pattern description (when and how to use it)"
                },
                "usage": {
                    "type": "array",
                    "description": "Usage examples or scenarios",
                    "items": {"type": "string"},
                    "default": []
                },
                "tags": {
                    "type": "array",
                    "description": "Tags for categorizing and searching the pattern",
                    "items": {"type": "string"},
                    "default": []
                },
                "code_example": {
                    "type": "string",
                    "description": "Optional code example (if applicable)",
                    "default": null
                }
            },
            "required": ["name", "description"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let name = args.get("name").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("name is required".to_string()))?;
        let description = args.get("description").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("description is required".to_string()))?;
        let usage: Vec<String> = args.get("usage")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let tags: Vec<String> = args.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let code_example = args.get("code_example").and_then(|v| v.as_str()).map(|s| s.to_string());

        let mut new_pattern = Pattern::new(name, description, &self.iteration_id);
        new_pattern.usage = usage;
        new_pattern.tags = tags;
        new_pattern.code_example = code_example;

        let store = MemoryStore::new();
        store.add_pattern(new_pattern)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to promote to pattern: {}", e)))?;

        Ok(json!({
            "message": "Promoted to project pattern successfully",
            "iteration_id": self.iteration_id
        }))
    }
}