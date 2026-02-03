// Memory System - Dual-layer architecture for project asset precipitation
//
// This module implements a dual-layer memory system:
// - Project Memory: Cross-session project-level decisions, experiences, and patterns
// - Session Memory: Session-dimension detailed records and experiences
//
// Architecture:
// - System-defined structure for control
// - Agent-friendly query interface
// - JSON indices + Markdown details
// - File system based storage

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ============================================================================
// Memory Index Types
// ============================================================================

/// Main Memory Index - Project-level memory summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMemoryIndex {
    pub project_id: String,
    pub project_name: String,
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    /// Project-level decisions
    pub key_decisions: Vec<MemoryItem>,
    
    /// Cross-session experiences
    pub key_experiences: Vec<MemoryItem>,
    
    /// Common patterns and best practices
    pub patterns: Vec<MemoryItem>,
    
    /// Project timeline
    pub timeline: Vec<TimelineEvent>,
    
    /// Statistics
    pub statistics: MemoryStatistics,
}

/// Session Memory Index - Session-level memory summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMemoryIndex {
    pub session_id: String,
    pub session_type: String,
    pub session_description: String,
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
    
    /// Session overview
    pub overview: SessionOverview,
    
    /// Session decisions
    pub decisions: Vec<MemoryItem>,
    
    /// Session experiences
    pub experiences: Vec<MemoryItem>,
    
    /// Session records
    pub records: Vec<MemoryItem>,
}

/// Memory item in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub title: String,
    pub category: String, // decision, experience, pattern, record
    pub summary: String,
    pub stage: Option<String>,
    pub session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub impact: String, // high, medium, low
    pub status: String, // implemented, pending, archived
    pub file: String, // Path to the detail markdown file
    pub tags: Vec<String>,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String, // decision, experience, milestone
    pub description: String,
    pub related_memory_id: Option<String>,
}

/// Session overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOverview {
    pub stages_completed: Vec<String>,
    pub key_achievements: Vec<String>,
    pub challenges_faced: Vec<String>,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub total_decisions: usize,
    pub total_experiences: usize,
    pub total_patterns: usize,
    pub total_sessions: usize,
}

// ============================================================================
// Memory Detail Types (stored in Markdown files)
// ============================================================================

/// Memory detail content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDetail {
    pub id: String,
    pub title: String,
    pub category: String,
    pub stage: Option<String>,
    pub session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    
    /// Markdown content
    pub content: String,
    
    /// Metadata
    pub metadata: MemoryMetadata,
}

/// Memory metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub impact: String,
    pub status: String,
    pub tags: Vec<String>,
    pub related_items: Vec<String>,
    pub confidence: Option<f64>,
}

// ============================================================================
// Memory Query Results
// ============================================================================

/// Memory index query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryIndexResult {
    pub results: Vec<MemoryItem>,
    pub total: usize,
}

/// Memory detail result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDetailResult {
    pub memory_id: String,
    pub content: String,
    pub file: String,
}

/// Memory save result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySaveResult {
    pub memory_id: String,
    pub file: String,
    pub message: String,
}

/// Promote to project memory result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteResult {
    pub project_memory_id: String,
    pub file: String,
    pub message: String,
}

/// Memory context result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContextResult {
    pub project_memory: MemoryContextProject,
    pub session_memory: MemoryContextSession,
    pub context: MemoryContextInfo,
}

/// Project memory context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContextProject {
    pub total_decisions: usize,
    pub total_experiences: usize,
    pub key_decisions: Vec<(String, String)>, // (id, title)
}

/// Session memory context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContextSession {
    pub session_id: String,
    pub status: String,
    pub stages_completed: Vec<String>,
    pub current_stage: String,
    pub decisions: usize,
    pub experiences: usize,
}

/// Memory context info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContextInfo {
    pub current_time: String,
    pub project_age: String,
    pub session_age: String,
}

// ============================================================================
// Memory Storage Paths
// ============================================================================

pub fn get_memory_dir() -> anyhow::Result<std::path::PathBuf> {
    let cow_dir = crate::storage::get_cowork_dir()?;
    Ok(cow_dir.join("memory"))
}

pub fn get_project_memory_file() -> anyhow::Result<std::path::PathBuf> {
    let memory_dir = get_memory_dir()?;
    Ok(memory_dir.join("project_memory.json"))
}

pub fn get_session_memory_file(session_id: &str) -> anyhow::Result<std::path::PathBuf> {
    let memory_dir = get_memory_dir()?;
    Ok(memory_dir.join("sessions").join(format!("{}.json", session_id)))
}

pub fn get_project_memory_detail_dir() -> anyhow::Result<std::path::PathBuf> {
    let memory_dir = get_memory_dir()?;
    Ok(memory_dir.join("project_memory"))
}

pub fn get_session_memory_detail_dir(session_id: &str) -> anyhow::Result<std::path::PathBuf> {
    let memory_dir = get_memory_dir()?;
    Ok(memory_dir.join("sessions").join(format!("sessions/{}", session_id)))
}

pub fn get_memory_detail_file(memory_type: &str, memory_id: &str, session_id: Option<&str>) -> anyhow::Result<std::path::PathBuf> {
    match session_id {
        Some(sid) => {
            let dir = get_session_memory_detail_dir(sid)?;
            Ok(dir.join(format!("{}s/{}.md", memory_type, memory_id)))
        }
        None => {
            let dir = get_project_memory_detail_dir()?;
            Ok(dir.join(format!("{}s/{}.md", memory_type, memory_id)))
        }
    }
}