// Storage layer for .cowork/ directory - Session-scoped architecture
use crate::data::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod storage_test;

const COWORK_DIR: &str = ".cowork";
const INDEX_FILE: &str = "index.json";
const SESSIONS_DIR: &str = "sessions";

// ============================================================================
// Core Directory Structure
// ============================================================================

/// Get the .cowork directory path, create if not exists
pub fn get_cowork_dir() -> Result<PathBuf> {
    let path = PathBuf::from(COWORK_DIR);
    
    // Create main directory and subdirectories
    fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create .cowork directory at {:?}", path))?;
    fs::create_dir_all(path.join(SESSIONS_DIR))?;
    
    Ok(path)
}

/// Get path for a specific session directory
pub fn get_session_dir(session_id: &str) -> Result<PathBuf> {
    let cowork_dir = get_cowork_dir()?;
    let session_path = cowork_dir.join(SESSIONS_DIR).join(session_id);
    
    // Create session subdirectories
    fs::create_dir_all(&session_path)?;
    fs::create_dir_all(session_path.join("artifacts"))?;
    fs::create_dir_all(session_path.join("state"))?;
    fs::create_dir_all(session_path.join("patch"))?;
    fs::create_dir_all(session_path.join("logs"))?;
    
    Ok(session_path)
}

/// Get the project root directory (where .cowork/ is located)
/// This is the actual workspace where code files are written
pub fn get_project_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
    Ok(current_dir)
}

/// Check if .cowork directory exists
pub fn cowork_dir_exists() -> bool {
    Path::new(COWORK_DIR).exists()
}

/// Check if project has been initialized (has index.json)
pub fn is_project_initialized() -> bool {
    Path::new(COWORK_DIR).join(INDEX_FILE).exists()
}

// ============================================================================
// Project Index (index.json at root of .cowork/)
// ============================================================================

pub fn load_project_index() -> Result<ProjectIndex> {
    let path = PathBuf::from(COWORK_DIR).join(INDEX_FILE);
    if !path.exists() {
        anyhow::bail!("Project not initialized. Run 'cowork new' first.");
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    let index: ProjectIndex = serde_json::from_str(&content)
        .with_context(|| "Failed to parse index.json")?;
    Ok(index)
}

pub fn save_project_index(index: &ProjectIndex) -> Result<()> {
    let cowork_dir = get_cowork_dir()?;
    let path = cowork_dir.join(INDEX_FILE);
    let content = serde_json::to_string_pretty(index)?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write {:?}", path))?;
    Ok(())
}

pub fn init_project_index(project_name: String) -> Result<ProjectIndex> {
    if is_project_initialized() {
        anyhow::bail!(".cowork directory already initialized");
    }
    let index = ProjectIndex::new(project_name);
    save_project_index(&index)?;
    Ok(index)
}

// ============================================================================
// Session Input (sessions/<id>/input.json)
// ============================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInput {
    pub session_id: String,
    pub session_type: SessionType,
    pub description: String,
    pub base_session_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn save_session_input(session_id: &str, input: &SessionInput) -> Result<()> {
    let session_dir = get_session_dir(session_id)?;
    let path = session_dir.join("input.json");
    let content = serde_json::to_string_pretty(input)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_session_input(session_id: &str) -> Result<SessionInput> {
    let session_dir = get_session_dir(session_id)?;
    let path = session_dir.join("input.json");
    if !path.exists() {
        anyhow::bail!("Session input not found for session {}", session_id);
    }
    let content = fs::read_to_string(&path)?;
    let input: SessionInput = serde_json::from_str(&content)?;
    Ok(input)
}

// ============================================================================
// Change Request (sessions/<id>/change_request.json - only for modify sessions)
// ============================================================================

pub fn save_change_request(session_id: &str, change_request: &ChangeRequest) -> Result<()> {
    let session_dir = get_session_dir(session_id)?;
    let path = session_dir.join("change_request.json");
    let content = serde_json::to_string_pretty(change_request)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_change_request(session_id: &str) -> Result<ChangeRequest> {
    let session_dir = get_session_dir(session_id)?;
    let path = session_dir.join("change_request.json");
    if !path.exists() {
        anyhow::bail!("Change request not found for session {}", session_id);
    }
    let content = fs::read_to_string(&path)?;
    let cr: ChangeRequest = serde_json::from_str(&content)?;
    Ok(cr)
}

// ============================================================================
// Session-scoped Artifacts (sessions/<id>/artifacts/)
// ============================================================================

fn artifact_path(session_id: &str, filename: &str) -> Result<PathBuf> {
    let session_dir = get_session_dir(session_id)?;
    Ok(session_dir.join("artifacts").join(filename))
}

pub fn save_idea(session_id: &str, content: &str) -> Result<()> {
    let path = artifact_path(session_id, "idea.md")?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_idea(session_id: &str) -> Result<String> {
    let path = artifact_path(session_id, "idea.md")?;
    if !path.exists() {
        return Ok(String::new());
    }
    let content = fs::read_to_string(&path)?;
    Ok(content)
}

pub fn save_prd_doc(session_id: &str, content: &str) -> Result<()> {
    let path = artifact_path(session_id, "prd.md")?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn save_design_doc(session_id: &str, content: &str) -> Result<()> {
    let path = artifact_path(session_id, "design.md")?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn save_delivery_report(session_id: &str, content: &str) -> Result<()> {
    let path = artifact_path(session_id, "delivery_report.md")?;
    fs::write(&path, content)?;
    Ok(())
}

// ============================================================================
// Session-scoped State (sessions/<id>/state/)
// ============================================================================

fn state_path(session_id: &str, filename: &str) -> Result<PathBuf> {
    let session_dir = get_session_dir(session_id)?;
    Ok(session_dir.join("state").join(filename))
}

pub fn state_file_exists(session_id: &str, filename: &str) -> Result<bool> {
    Ok(state_path(session_id, filename)?.exists())
}

pub fn has_requirements(session_id: &str) -> Result<bool> {
    state_file_exists(session_id, "requirements.json")
}

pub fn has_design_spec(session_id: &str) -> Result<bool> {
    state_file_exists(session_id, "design_spec.json")
}

pub fn has_implementation_plan(session_id: &str) -> Result<bool> {
    state_file_exists(session_id, "implementation_plan.json")
}

pub fn has_code_metadata(session_id: &str) -> Result<bool> {
    state_file_exists(session_id, "code_metadata.json")
}

/// Check if coding stage has made progress (has written files)
pub fn has_code_files(session_id: &str) -> Result<bool> {
    if !has_code_metadata(session_id)? {
        return Ok(false);
    }
    
    let metadata = load_code_metadata(session_id)?;
    Ok(!metadata.files.is_empty())
}

pub fn save_requirements(session_id: &str, requirements: &Requirements) -> Result<()> {
    let path = state_path(session_id, "requirements.json")?;
    let content = serde_json::to_string_pretty(requirements)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_requirements(session_id: &str) -> Result<Requirements> {
    let path = state_path(session_id, "requirements.json")?;
    if !path.exists() {
        return Ok(Requirements::new());
    }
    let content = fs::read_to_string(&path)?;
    let requirements: Requirements = serde_json::from_str(&content)?;
    Ok(requirements)
}

pub fn save_feature_list(session_id: &str, features: &FeatureList) -> Result<()> {
    let path = state_path(session_id, "feature_list.json")?;
    let content = serde_json::to_string_pretty(features)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_feature_list(session_id: &str) -> Result<FeatureList> {
    let path = state_path(session_id, "feature_list.json")?;
    if !path.exists() {
        return Ok(FeatureList::new());
    }
    let content = fs::read_to_string(&path)?;
    let features: FeatureList = serde_json::from_str(&content)?;
    Ok(features)
}

pub fn save_design_spec(session_id: &str, design: &DesignSpec) -> Result<()> {
    let path = state_path(session_id, "design_spec.json")?;
    let content = serde_json::to_string_pretty(design)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_design_spec(session_id: &str) -> Result<DesignSpec> {
    let path = state_path(session_id, "design_spec.json")?;
    if !path.exists() {
        return Ok(DesignSpec::new());
    }
    let content = fs::read_to_string(&path)?;
    let design: DesignSpec = serde_json::from_str(&content)?;
    Ok(design)
}

pub fn save_implementation_plan(session_id: &str, plan: &ImplementationPlan) -> Result<()> {
    let path = state_path(session_id, "implementation_plan.json")?;
    let content = serde_json::to_string_pretty(plan)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_implementation_plan(session_id: &str) -> Result<ImplementationPlan> {
    let path = state_path(session_id, "implementation_plan.json")?;
    if !path.exists() {
        return Ok(ImplementationPlan::new());
    }
    let content = fs::read_to_string(&path)?;
    let plan: ImplementationPlan = serde_json::from_str(&content)?;
    Ok(plan)
}

pub fn save_code_metadata(session_id: &str, metadata: &CodeMetadata) -> Result<()> {
    let path = state_path(session_id, "code_metadata.json")?;
    let content = serde_json::to_string_pretty(metadata)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_code_metadata(session_id: &str) -> Result<CodeMetadata> {
    let path = state_path(session_id, "code_metadata.json")?;
    if !path.exists() {
        return Ok(CodeMetadata::new());
    }
    let content = fs::read_to_string(&path)?;
    let metadata: CodeMetadata = serde_json::from_str(&content)?;
    Ok(metadata)
}

// ============================================================================
// Session Metadata (sessions/<id>/state/meta.json)
// ============================================================================

pub fn save_session_meta(session_id: &str, meta: &SessionMeta) -> Result<()> {
    let path = state_path(session_id, "meta.json")?;
    let content = serde_json::to_string_pretty(meta)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_session_meta(session_id: &str) -> Result<Option<SessionMeta>> {
    let path = state_path(session_id, "meta.json")?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    let meta: SessionMeta = serde_json::from_str(&content)?;
    Ok(Some(meta))
}

// ============================================================================
// Feedback History (sessions/<id>/state/feedback.json)
// ============================================================================

pub fn save_feedback_history(session_id: &str, history: &FeedbackHistory) -> Result<()> {
    let path = state_path(session_id, "feedback.json")?;
    let content = serde_json::to_string_pretty(history)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_feedback_history(session_id: &str) -> Result<FeedbackHistory> {
    let path = state_path(session_id, "feedback.json")?;
    if !path.exists() {
        return Ok(FeedbackHistory::new());
    }
    let content = fs::read_to_string(&path)?;
    let history: FeedbackHistory = serde_json::from_str(&content)?;
    Ok(history)
}

pub fn append_feedback(session_id: &str, feedback: &Feedback) -> Result<()> {
    let mut history = load_feedback_history(session_id)?;
    history.feedbacks.push(feedback.clone());
    save_feedback_history(session_id, &history)?;
    Ok(())
}

// ============================================================================
// Patch Metadata (sessions/<id>/patch/metadata.json - for modify sessions)
// ============================================================================

pub fn save_patch_metadata(session_id: &str, patch: &PatchMetadata) -> Result<()> {
    let session_dir = get_session_dir(session_id)?;
    let path = session_dir.join("patch").join("metadata.json");
    let content = serde_json::to_string_pretty(patch)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn load_patch_metadata(session_id: &str) -> Result<PatchMetadata> {
    let session_dir = get_session_dir(session_id)?;
    let path = session_dir.join("patch").join("metadata.json");
    if !path.exists() {
        anyhow::bail!("Patch metadata not found for session {}", session_id);
    }
    let content = fs::read_to_string(&path)?;
    let patch: PatchMetadata = serde_json::from_str(&content)?;
    Ok(patch)
}

// ============================================================================
// Session Inheritance / Bootstrap
// ============================================================================

/// Initialize a new session by copying state/artifacts from a base session.
///
/// This is critical for `modify` / `revert` / `resume` flows: a fresh session directory
/// should not start with empty state, otherwise agents will see empty requirements/design/plan.
///
/// What we copy:
/// - state/*.json (requirements, feature_list, design_spec, implementation_plan, code_metadata, feedback, meta)
/// - artifacts/*.md (idea, prd, design, delivery_report) if present
///
/// Notes:
/// - This function does NOT copy code files in the project root.
/// - Missing files are skipped.
pub fn init_session_from_base(new_session_id: &str, base_session_id: &str) -> Result<()> {
    let base_dir = get_session_dir(base_session_id)?;
    let new_dir = get_session_dir(new_session_id)?;

    // helper to copy a file if it exists
    fn copy_if_exists(src: &Path, dst: &Path) -> Result<()> {
        if !src.exists() {
            return Ok(());
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst).with_context(|| format!("Failed to copy {:?} -> {:?}", src, dst))?;
        Ok(())
    }

    // state files
    let state_files = [
        "requirements.json",
        "feature_list.json",
        "design_spec.json",
        "implementation_plan.json",
        "code_metadata.json",
        "feedback.json",
        "meta.json",
    ];

    for name in state_files {
        let src = base_dir.join("state").join(name);
        let dst = new_dir.join("state").join(name);
        copy_if_exists(&src, &dst)?;
    }

    // artifact files
    let artifact_files = ["idea.md", "prd.md", "design.md", "delivery_report.md"]; 
    for name in artifact_files {
        let src = base_dir.join("artifacts").join(name);
        let dst = new_dir.join("artifacts").join(name);
        copy_if_exists(&src, &dst)?;
    }

    Ok(())
}

// ============================================================================
// Helper utilities
// ============================================================================

/// Generate ID with prefix and counter
pub fn generate_id(prefix: &str, counter: usize) -> String {
    format!("{}-{:03}", prefix, counter + 1)
}

/// Get the latest successful session ID from index
pub fn get_latest_successful_session() -> Result<Option<String>> {
    if !is_project_initialized() {
        return Ok(None);
    }
    let index = load_project_index()?;
    Ok(index.latest_successful_session)
}

/// Mark a session as completed successfully
pub fn mark_session_completed(session_id: &str) -> Result<()> {
    let mut index = load_project_index()?;
    
    // Update session record
    for session in &mut index.sessions {
        if session.session_id == session_id {
            session.status = SessionStatus::Completed;
            session.completed_at = Some(chrono::Utc::now());
            break;
        }
    }
    
    // Update latest successful session
    index.update_latest_successful(session_id.to_string());
    save_project_index(&index)?;
    Ok(())
}

/// Mark a session as failed
pub fn mark_session_failed(session_id: &str) -> Result<()> {
    let mut index = load_project_index()?;
    
    for session in &mut index.sessions {
        if session.session_id == session_id {
            session.status = SessionStatus::Failed;
            session.completed_at = Some(chrono::Utc::now());
            break;
        }
    }
    
    save_project_index(&index)?;
    Ok(())
}
