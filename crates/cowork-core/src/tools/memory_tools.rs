// Memory Tools - Tools for the dual-layer memory system
//
// These tools allow agents to:
// - Query memory indices (project and session level)
// - Load memory details
// - Save session memories
// - Promote session memories to project level
// - Get memory context

use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use crate::memory::*;
use crate::storage::get_cowork_dir;
use std::fs;

// ============================================================================
// Query Memory Index Tool
// ============================================================================

pub struct QueryMemoryIndexTool;

#[async_trait]
impl Tool for QueryMemoryIndexTool {
    fn name(&self) -> &str {
        "query_memory_index"
    }

    fn description(&self) -> &str {
        "Query memory index to get a list of memory items (decisions, experiences, patterns, records). Use this before loading detailed memory content to find relevant items."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "query_type": {
                    "type": "string",
                    "description": "Which memory to query: 'all' (both project and session), 'project' (project-level only), or 'session' (current session only)",
                    "enum": ["all", "project", "session"],
                    "default": "all"
                },
                "category": {
                    "type": "string",
                    "description": "Filter by memory category: 'decision', 'experience', 'pattern', 'record', or 'all' for all categories",
                    "enum": ["decision", "experience", "pattern", "record", "all"],
                    "default": "all"
                },
                "stage": {
                    "type": "string",
                    "description": "Filter by stage (e.g., 'idea', 'prd', 'design', 'plan', 'coding', 'check'). Optional.",
                    "default": null
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of results to return. Default: 20",
                    "default": 20
                }
            },
            "required": []
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let query_type = args.get("query_type").and_then(|v| v.as_str()).unwrap_or("all");
        let category = args.get("category").and_then(|v| v.as_str()).unwrap_or("all");
        let stage = args.get("stage").and_then(|v| v.as_str());
        let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20);

        let mut results = Vec::new();

        // Query project memory
        if query_type == "all" || query_type == "project" {
            if let Ok(project_file) = get_project_memory_file() {
                if let Ok(content) = fs::read_to_string(&project_file) {
                    if let Ok(index) = serde_json::from_str::<ProjectMemoryIndex>(&content) {
                        for item in Self::filter_items(index.key_decisions, category, stage, limit) {
                            results.push(item);
                        }
                        for item in Self::filter_items(index.key_experiences, category, stage, limit) {
                            results.push(item);
                        }
                        for item in Self::filter_items(index.patterns, category, stage, limit) {
                            results.push(item);
                        }
                    }
                }
            }
        }

        // Query session memory (get current session from .cowork/session.json)
        if query_type == "all" || query_type == "session" {
            if let Ok(cow_dir) = get_cowork_dir() {
                if let Ok(index_content) = fs::read_to_string(cow_dir.join("session.json")) {
                    if let Ok(index) = serde_json::from_str::<crate::data::ProjectIndex>(&index_content) {
                        if let Some(current_session_id) = index.latest_successful_session {
                            if let Ok(session_file) = get_session_memory_file(&current_session_id) {
                                if let Ok(content) = fs::read_to_string(&session_file) {
                                    if let Ok(session_index) = serde_json::from_str::<SessionMemoryIndex>(&content) {
                                        for item in Self::filter_items(session_index.decisions, category, stage, limit) {
                                            results.push(item);
                                        }
                                        for item in Self::filter_items(session_index.experiences, category, stage, limit) {
                                            results.push(item);
                                        }
                                        for item in Self::filter_items(session_index.records, category, stage, limit) {
                                            results.push(item);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by created_at descending and apply limit
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results.truncate(limit as usize);

        let total = results.len();

        Ok(json!({
            "results": results,
            "total": total
        }))
    }
}

impl QueryMemoryIndexTool {
    fn filter_items(items: Vec<MemoryItem>, category: &str, stage_filter: Option<&str>, _limit: i64) -> Vec<MemoryItem> {
        items.into_iter()
            .filter(|item| {
                if category != "all" && item.category != category {
                    return false;
                }
                if let Some(stage) = stage_filter {
                    if let Some(item_stage) = &item.stage {
                        if item_stage != stage {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

// ============================================================================
// Load Memory Detail Tool
// ============================================================================

pub struct LoadMemoryDetailTool;

#[async_trait]
impl Tool for LoadMemoryDetailTool {
    fn name(&self) -> &str {
        "load_memory_detail"
    }

    fn description(&self) -> &str {
        "Load the detailed content of a memory item from its markdown file. Use this after querying the index to get the full details of a specific memory item."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "string",
                    "description": "The ID of the memory item to load"
                },
                "file": {
                    "type": "string",
                    "description": "The file path to the memory detail (as returned by query_memory_index)"
                }
            },
            "required": ["memory_id", "file"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let memory_id = args.get("memory_id").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("memory_id is required".to_string()))?;
        let file = args.get("file").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("file is required".to_string()))?;

        let cow_dir = get_cowork_dir()
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get cow dir: {}", e)))?;
        let full_path = cow_dir.join(file);

        let content = fs::read_to_string(&full_path)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read memory file: {}", e)))?;

        Ok(json!({
            "memory_id": memory_id,
            "content": content,
            "file": file
        }))
    }
}

// ============================================================================
// Save Session Memory Tool
// ============================================================================

pub struct SaveSessionMemoryTool;

#[async_trait]
impl Tool for SaveSessionMemoryTool {
    fn name(&self) -> &str {
        "save_session_memory"
    }

    fn description(&self) -> &str {
        "Save a memory item to the current session's memory. Use this to record important decisions, experiences, or observations during the session."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "memory_type": {
                    "type": "string",
                    "description": "Type of memory: 'decision', 'experience', or 'record'",
                    "enum": ["decision", "experience", "record"]
                },
                "title": {
                    "type": "string",
                    "description": "Title of the memory item"
                },
                "summary": {
                    "type": "string",
                    "description": "Brief summary of the memory (1-2 sentences)"
                },
                "content": {
                    "type": "string",
                    "description": "Detailed markdown content of the memory"
                },
                "stage": {
                    "type": "string",
                    "description": "The current stage (e.g., 'idea', 'prd', 'design', 'plan', 'coding', 'check')"
                },
                "category": {
                    "type": "string",
                    "description": "Category for better organization (e.g., 'technical', 'design', 'user-experience')"
                },
                "impact": {
                    "type": "string",
                    "description": "Impact level: 'high', 'medium', or 'low'",
                    "enum": ["high", "medium", "low"],
                    "default": "medium"
                },
                "tags": {
                    "type": "array",
                    "description": "Tags for better searchability",
                    "items": {"type": "string"},
                    "default": []
                }
            },
            "required": ["memory_type", "title", "summary", "content", "stage"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let memory_type = args.get("memory_type").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("memory_type is required".to_string()))?;
        let title = args.get("title").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("title is required".to_string()))?;
        let summary = args.get("summary").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("summary is required".to_string()))?;
        let content = args.get("content").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("content is required".to_string()))?;
        let stage = args.get("stage").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("stage is required".to_string()))?;
        let _category = args.get("category").and_then(|v| v.as_str()).unwrap_or("general");
        let impact = args.get("impact").and_then(|v| v.as_str()).unwrap_or("medium");
        let tags: Vec<String> = args.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        // Get current session ID
        let cow_dir = get_cowork_dir()
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get cow dir: {}", e)))?;
        let index_content = fs::read_to_string(cow_dir.join("session.json"))
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read session index: {}", e)))?;
        let index: crate::data::ProjectIndex = serde_json::from_str(&index_content)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to parse session index: {}", e)))?;
        let session_id = index.latest_successful_session
            .ok_or_else(|| adk_core::AdkError::Tool("No active session found".to_string()))?;

        // Generate memory ID
        let memory_id = format!("{}-{:04}",
            memory_type.chars().next().unwrap_or('X'),
            chrono::Utc::now().timestamp() % 10000
        );

        // Create detail directory and file
        let detail_dir = get_session_memory_detail_dir(&session_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get session memory detail dir: {}", e)))?
            .join(format!("{}s", memory_type));
        fs::create_dir_all(&detail_dir)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to create detail directory: {}", e)))?;

        let file_path = detail_dir.join(format!("{}.md", memory_id));
        let file_relative = format!("sessions/sessions/{}/{}s/{}.md", session_id, memory_type, memory_id);

        // Write markdown content
        fs::write(&file_path, content)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to write memory file: {}", e)))?;

        // Update session memory index
        let session_index_file = get_session_memory_file(&session_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get session memory file: {}", e)))?;
        let mut session_index: SessionMemoryIndex = if session_index_file.exists() {
            let content = fs::read_to_string(&session_index_file)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read session memory index: {}", e)))?;
            serde_json::from_str(&content)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to parse session memory index: {}", e)))?
        } else {
            SessionMemoryIndex {
                session_id: session_id.clone(),
                session_type: "modify".to_string(),
                session_description: "".to_string(),
                schema_version: "1.0".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                status: "active".to_string(),
                overview: SessionOverview {
                    stages_completed: vec![],
                    key_achievements: vec![],
                    challenges_faced: vec![],
                },
                decisions: vec![],
                experiences: vec![],
                records: vec![],
            }
        };

        // Add memory item to appropriate category
        let item = MemoryItem {
            id: memory_id.clone(),
            title: title.to_string(),
            category: memory_type.to_string(),
            summary: summary.to_string(),
            stage: Some(stage.to_string()),
            session_id: Some(session_id.clone()),
            created_at: chrono::Utc::now(),
            impact: impact.to_string(),
            status: "implemented".to_string(),
            file: file_relative.clone(),
            tags: tags.clone(),
        };

        match memory_type {
            "decision" => session_index.decisions.push(item),
            "experience" => session_index.experiences.push(item),
            "record" => session_index.records.push(item),
            _ => return Err(adk_core::AdkError::Tool(format!("Invalid memory type: {}", memory_type))),
        }

        session_index.updated_at = chrono::Utc::now();

        // Save updated index
        fs::write(&session_index_file, serde_json::to_string_pretty(&session_index).unwrap())
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to save session memory index: {}", e)))?;

        Ok(json!({
            "memory_id": memory_id,
            "file": file_relative,
            "message": "Session memory saved successfully"
        }))
    }
}

// ============================================================================
// Promote to Project Memory Tool
// ============================================================================

pub struct PromoteToProjectMemoryTool;

#[async_trait]
impl Tool for PromoteToProjectMemoryTool {
    fn name(&self) -> &str {
        "promote_to_project_memory"
    }

    fn description(&self) -> &str {
        "Promote a session memory item to project-level memory. Use this for decisions or experiences that are valuable across sessions and should be remembered at the project level."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "string",
                    "description": "The ID of the session memory item to promote"
                },
                "reason": {
                    "type": "string",
                    "description": "Why this memory should be promoted to project level"
                }
            },
            "required": ["memory_id", "reason"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let memory_id = args.get("memory_id").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("memory_id is required".to_string()))?;
        let reason = args.get("reason").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::Tool("reason is required".to_string()))?;

        // Get current session ID
        let cow_dir = get_cowork_dir()
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get cow dir: {}", e)))?;
        let index_content = fs::read_to_string(cow_dir.join("session.json"))
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read session index: {}", e)))?;
        let index: crate::data::ProjectIndex = serde_json::from_str(&index_content)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to parse session index: {}", e)))?;
        let session_id = index.latest_successful_session
            .ok_or_else(|| adk_core::AdkError::Tool("No active session found".to_string()))?;

        // Read session memory index
        let session_index_file = get_session_memory_file(&session_id)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get session memory file: {}", e)))?;
        let session_index_content = fs::read_to_string(&session_index_file)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read session memory index: {}", e)))?;
        let session_index: SessionMemoryIndex = serde_json::from_str(&session_index_content)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to parse session memory index: {}", e)))?;

        // Find the memory item
        let memory_item = session_index.decisions.iter()
            .chain(session_index.experiences.iter())
            .chain(session_index.records.iter())
            .find(|item| item.id == memory_id)
            .ok_or_else(|| adk_core::AdkError::Tool(format!("Memory item not found: {}", memory_id)))?;

        // Create project memory ID
        let project_memory_id = format!("{}-{:04}",
            memory_item.id.chars().next().unwrap_or('X'),
            chrono::Utc::now().timestamp() % 10000
        );

        // Copy to project memory directory
        let project_detail_dir = get_project_memory_detail_dir()
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get project memory detail dir: {}", e)))?
            .join(format!("{}s", memory_item.category));
        fs::create_dir_all(&project_detail_dir)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to create project memory directory: {}", e)))?;

        let old_file = cow_dir.join(&memory_item.file);
        let new_file = project_detail_dir.join(format!("{}.md", project_memory_id));
        let new_file_relative = format!("project_memory/{}s/{}.md", memory_item.category, project_memory_id);

        fs::copy(&old_file, &new_file)
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to copy memory file: {}", e)))?;

        // Update project memory index
        let project_index_file = get_project_memory_file()
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to get project memory file: {}", e)))?;
        let mut project_index: ProjectMemoryIndex = if project_index_file.exists() {
            let content = fs::read_to_string(&project_index_file)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to read project memory index: {}", e)))?;
            serde_json::from_str(&content)
                .map_err(|e| adk_core::AdkError::Tool(format!("Failed to parse project memory index: {}", e)))?
        } else {
            ProjectMemoryIndex {
                project_id: "default".to_string(),
                project_name: "Default Project".to_string(),
                schema_version: "1.0".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                key_decisions: vec![],
                key_experiences: vec![],
                patterns: vec![],
                timeline: vec![],
                statistics: MemoryStatistics {
                    total_decisions: 0,
                    total_experiences: 0,
                    total_patterns: 0,
                    total_sessions: 0,
                },
            }
        };

        // Add promoted item
        let promoted_item = MemoryItem {
            id: project_memory_id.clone(),
            title: memory_item.title.clone(),
            category: memory_item.category.clone(),
            summary: memory_item.summary.clone(),
            stage: memory_item.stage.clone(),
            session_id: Some(session_id.clone()),
            created_at: chrono::Utc::now(),
            impact: memory_item.impact.clone(),
            status: "implemented".to_string(),
            file: new_file_relative.clone(),
            tags: memory_item.tags.clone(),
        };

        // Add timeline event
        project_index.timeline.push(TimelineEvent {
            timestamp: chrono::Utc::now(),
            event_type: format!("promoted_{}", memory_item.category),
            description: format!("Promoted from session {}: {}", session_id, reason),
            related_memory_id: Some(project_memory_id.clone()),
        });

        // Add to appropriate category
        match memory_item.category.as_str() {
            "decision" => {
                project_index.key_decisions.push(promoted_item);
                project_index.statistics.total_decisions += 1;
            }
            "experience" => {
                project_index.key_experiences.push(promoted_item);
                project_index.statistics.total_experiences += 1;
            }
            "pattern" => {
                project_index.patterns.push(promoted_item);
                project_index.statistics.total_patterns += 1;
            }
            _ => {}
        }

        project_index.updated_at = chrono::Utc::now();

        // Save updated index
        fs::write(&project_index_file, serde_json::to_string_pretty(&project_index).unwrap())
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to save project memory index: {}", e)))?;

        Ok(json!({
            "project_memory_id": project_memory_id,
            "file": new_file_relative,
            "message": "Memory promoted to project level successfully"
        }))
    }
}

// ============================================================================
// Get Memory Context Tool
// ============================================================================

pub struct GetMemoryContextTool;

#[async_trait]
impl Tool for GetMemoryContextTool {
    fn name(&self) -> &str {
        "get_memory_context"
    }

    fn description(&self) -> &str {
        "Get the current memory context including project memory summary, session memory summary, and general context information. Use this before making decisions to understand the current memory state."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {},
            "required": []
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        // Get project memory
        let project_memory = if let Ok(project_file) = get_project_memory_file() {
            if let Ok(content) = fs::read_to_string(&project_file) {
                if let Ok(index) = serde_json::from_str::<ProjectMemoryIndex>(&content) {
                    MemoryContextProject {
                        total_decisions: index.key_decisions.len(),
                        total_experiences: index.key_experiences.len(),
                        key_decisions: index.key_decisions.iter()
                            .take(5)
                            .map(|d| (d.id.clone(), d.title.clone()))
                            .collect(),
                    }
                } else {
                    MemoryContextProject {
                        total_decisions: 0,
                        total_experiences: 0,
                        key_decisions: vec![],
                    }
                }
            } else {
                MemoryContextProject {
                    total_decisions: 0,
                    total_experiences: 0,
                    key_decisions: vec![],
                }
            }
        } else {
            MemoryContextProject {
                total_decisions: 0,
                total_experiences: 0,
                key_decisions: vec![],
            }
        };

        // Get session memory
        let session_memory = if let Ok(cow_dir) = get_cowork_dir() {
            if let Ok(index_content) = fs::read_to_string(cow_dir.join("session.json")) {
                if let Ok(index) = serde_json::from_str::<crate::data::ProjectIndex>(&index_content) {
                    if let Some(current_session_id) = index.latest_successful_session {
                        if let Ok(session_file) = get_session_memory_file(&current_session_id) {
                            if let Ok(content) = fs::read_to_string(&session_file) {
                                if let Ok(session_index) = serde_json::from_str::<SessionMemoryIndex>(&content) {
                                    MemoryContextSession {
                                        session_id: current_session_id.clone(),
                                        status: session_index.status,
                                        stages_completed: session_index.overview.stages_completed,
                                        current_stage: "coding".to_string(), // Default stage
                                        decisions: session_index.decisions.len(),
                                        experiences: session_index.experiences.len(),
                                    }
                                } else {
                                    MemoryContextSession::default()
                                }
                            } else {
                                MemoryContextSession::default()
                            }
                        } else {
                            MemoryContextSession::default()
                        }
                    } else {
                        MemoryContextSession::default()
                    }
                } else {
                    MemoryContextSession::default()
                }
            } else {
                MemoryContextSession::default()
            }
        } else {
            MemoryContextSession::default()
        };

        // Get context info
        let context = MemoryContextInfo {
            current_time: chrono::Utc::now().to_rfc3339(),
            project_age: "Unknown".to_string(),
            session_age: "Unknown".to_string(),
        };

        Ok(json!({
            "project_memory": project_memory,
            "session_memory": session_memory,
            "context": context
        }))
    }
}

impl Default for MemoryContextSession {
    fn default() -> Self {
        Self {
            session_id: "none".to_string(),
            status: "inactive".to_string(),
            stages_completed: vec![],
            current_stage: "none".to_string(),
            decisions: 0,
            experiences: 0,
        }
    }
}