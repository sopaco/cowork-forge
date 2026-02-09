// Storage layer for .cowork-v2/iterations/{iteration_id}/ directory
use crate::data::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[cfg(test)]
mod storage_test;

// Thread-local storage for current iteration ID
static CURRENT_ITERATION_ID: Mutex<Option<String>> = Mutex::new(None);

/// Set the current iteration ID for data operations
pub fn set_iteration_id(iteration_id: String) {
    let mut id = CURRENT_ITERATION_ID.lock().unwrap();
    *id = Some(iteration_id);
}

/// Get the current iteration ID
pub fn get_iteration_id() -> Option<String> {
    let id = CURRENT_ITERATION_ID.lock().unwrap();
    id.clone()
}

/// Clear the current iteration ID
pub fn clear_iteration_id() {
    let mut id = CURRENT_ITERATION_ID.lock().unwrap();
    *id = None;
}

/// Get the iteration-specific directory path
pub fn get_iteration_dir() -> Result<PathBuf> {
    let iteration_id = get_iteration_id()
        .ok_or_else(|| anyhow::anyhow!("Iteration ID not set. Call set_iteration_id() first."))?;

    let path = PathBuf::from(".cowork-v2/iterations").join(&iteration_id);

    // Create iteration directory and subdirectories
    fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create iteration directory at {:?}", path))?;
    fs::create_dir_all(path.join("data"))?;
    fs::create_dir_all(path.join("artifacts"))?;
    fs::create_dir_all(path.join("session"))?;
    fs::create_dir_all(path.join("logs"))?;

    Ok(path)
}

/// Helper to get data file path (iteration-specific)
fn data_path(filename: &str) -> Result<PathBuf> {
    Ok(get_iteration_dir()?.join("data").join(filename))
}

/// Helper to get artifact file path (iteration-specific)
pub fn artifact_path(filename: &str) -> Result<PathBuf> {
    Ok(get_iteration_dir()?.join("artifacts").join(filename))
}

/// Helper to get session file path (iteration-specific)
fn session_path(filename: &str) -> Result<PathBuf> {
    Ok(get_iteration_dir()?.join("session").join(filename))
}

// ============================================================================
// Requirements
// ============================================================================

pub fn load_requirements() -> Result<Requirements> {
    let path = data_path("requirements.json")?;
    if !path.exists() {
        return Ok(Requirements::new());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    let requirements: Requirements = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse requirements.json"))?;
    Ok(requirements)
}

pub fn save_requirements(requirements: &Requirements) -> Result<()> {
    let path = data_path("requirements.json")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    let content = serde_json::to_string_pretty(requirements)?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write {:?}", path))?;
    Ok(())
}

// ============================================================================
// Feature List
// ============================================================================

pub fn load_feature_list() -> Result<FeatureList> {
    let path = data_path("feature_list.json")?;
    if !path.exists() {
        return Ok(FeatureList::new());
    }
    let content = fs::read_to_string(&path)?;
    let features: FeatureList = serde_json::from_str(&content)?;
    Ok(features)
}

pub fn save_feature_list(features: &FeatureList) -> Result<()> {
    let path = data_path("feature_list.json")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    let content = serde_json::to_string_pretty(features)?;
    fs::write(&path, content)?;
    Ok(())
}

// ============================================================================
// Design Spec
// ============================================================================

pub fn load_design_spec() -> Result<DesignSpec> {
    let path = data_path("design_spec.json")?;
    if !path.exists() {
        return Ok(DesignSpec::new());
    }
    let content = fs::read_to_string(&path)?;
    let design: DesignSpec = serde_json::from_str(&content)?;
    Ok(design)
}

pub fn save_design_spec(design: &DesignSpec) -> Result<()> {
    let path = data_path("design_spec.json")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    let content = serde_json::to_string_pretty(design)?;
    fs::write(&path, content)?;
    Ok(())
}

// ============================================================================
// Implementation Plan
// ============================================================================

pub fn load_implementation_plan() -> Result<ImplementationPlan> {
    let path = data_path("implementation_plan.json")?;
    if !path.exists() {
        return Ok(ImplementationPlan::new());
    }
    let content = fs::read_to_string(&path)?;
    let plan: ImplementationPlan = serde_json::from_str(&content)?;
    Ok(plan)
}

pub fn save_implementation_plan(plan: &ImplementationPlan) -> Result<()> {
    let path = data_path("implementation_plan.json")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    let content = serde_json::to_string_pretty(plan)?;
    fs::write(&path, content)?;
    Ok(())
}

// ============================================================================
// Code Metadata
// ============================================================================

pub fn load_code_metadata() -> Result<CodeMetadata> {
    let path = data_path("code_metadata.json")?;
    if !path.exists() {
        return Ok(CodeMetadata::new());
    }
    let content = fs::read_to_string(&path)?;
    let metadata: CodeMetadata = serde_json::from_str(&content)?;
    Ok(metadata)
}

pub fn save_code_metadata(metadata: &CodeMetadata) -> Result<()> {
    let path = data_path("code_metadata.json")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    let content = serde_json::to_string_pretty(metadata)?;
    fs::write(&path, content)?;
    Ok(())
}

// ============================================================================
// Session Meta
// ============================================================================

pub fn load_session_meta() -> Result<Option<SessionMeta>> {
    let path = session_path("meta.json")?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    let meta: SessionMeta = serde_json::from_str(&content)?;
    Ok(Some(meta))
}

pub fn save_session_meta(meta: &SessionMeta) -> Result<()> {
    let path = session_path("meta.json")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    let content = serde_json::to_string_pretty(meta)?;
    fs::write(&path, content)?;
    Ok(())
}

// ============================================================================
// Feedback History
// ============================================================================

pub fn load_feedback_history() -> Result<FeedbackHistory> {
    let path = session_path("feedback.json")?;
    if !path.exists() {
        return Ok(FeedbackHistory::new());
    }
    let content = fs::read_to_string(&path)?;
    let history: FeedbackHistory = serde_json::from_str(&content)?;
    Ok(history)
}

pub fn save_feedback_history(history: &FeedbackHistory) -> Result<()> {
    let path = session_path("feedback.json")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    let content = serde_json::to_string_pretty(history)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn append_feedback(feedback: &Feedback) -> Result<()> {
    let mut history = load_feedback_history()?;
    history.feedbacks.push(feedback.clone());
    save_feedback_history(&history)?;
    Ok(())
}

/// Clear all feedback for a specific stage
pub fn clear_stage_feedback(stage: &str) -> Result<()> {
    let mut history = load_feedback_history()?;
    history.feedbacks.retain(|f| f.stage != stage);
    save_feedback_history(&history)?;
    Ok(())
}

/// Clear all feedback (used when starting a fresh iteration)
pub fn clear_all_feedback() -> Result<()> {
    let path = session_path("feedback.json")?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

// ============================================================================
// Artifacts (Markdown files)
// ============================================================================

pub fn load_idea() -> Result<String> {
    let path = artifact_path("idea.md")?;
    if !path.exists() {
        return Ok(String::new());
    }
    let content = fs::read_to_string(&path)?;
    Ok(content)
}

pub fn save_idea(content: &str) -> Result<()> {
    let path = artifact_path("idea.md")?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }

    fs::write(&path, content)?;
    Ok(())
}

pub fn save_plan_doc(content: &str) -> Result<()> {
    let path = artifact_path("plan.md")?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }

    fs::write(&path, content)?;
    Ok(())
}

pub fn save_prd_doc(content: &str) -> Result<()> {
    let path = artifact_path("prd.md")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    fs::write(&path, content)?;
    Ok(())
}

pub fn save_design_doc(content: &str) -> Result<()> {
    let path = artifact_path("design.md")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    fs::write(&path, content)?;
    Ok(())
}

pub fn save_delivery_report(content: &str) -> Result<()> {
    let path = artifact_path("delivery_report.md")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }
    
    fs::write(&path, content)?;
    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

/// Generate ID with prefix and counter
pub fn generate_id(prefix: &str, counter: usize) -> String {
    format!("{}-{:03}", prefix, counter + 1)
}

/// Check if .cowork-v2 directory exists
pub fn cowork_dir_exists() -> bool {
    Path::new(".cowork-v2").exists()
}

/// Check if current iteration directory exists
pub fn iteration_dir_exists() -> bool {
    match get_iteration_id() {
        Some(iteration_id) => {
            Path::new(".cowork-v2/iterations").join(&iteration_id).exists()
        }
        None => false,
    }
}