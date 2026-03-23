// Workspace management logic for iteration executor

use std::sync::Arc;

use crate::domain::{InheritanceMode, Iteration, IterationStatus};
use crate::interaction::InteractiveBackend;
use crate::persistence::IterationStore;

/// Prepare workspace for iteration execution
pub async fn prepare_workspace(
    iteration_store: &IterationStore,
    interaction: &Arc<dyn InteractiveBackend>,
    iteration: &Iteration,
) -> anyhow::Result<std::path::PathBuf> {
    let workspace = iteration_store.ensure_workspace(&iteration.id)?;

    // Only inherit from base when iteration is first starting (Draft status)
    if let Some(base_id) = &iteration.base_iteration_id {
        if iteration.status == IterationStatus::Draft {
            inherit_from_base(
                iteration_store,
                interaction,
                &workspace,
                base_id,
                iteration.inheritance,
            ).await?;
        }
    }

    Ok(workspace)
}

/// Inherit workspace from base iteration
async fn inherit_from_base(
    iteration_store: &IterationStore,
    interaction: &Arc<dyn InteractiveBackend>,
    workspace: &std::path::PathBuf,
    base_iteration_id: &str,
    inheritance_mode: InheritanceMode,
) -> anyhow::Result<()> {
    match inheritance_mode {
        InheritanceMode::None => {
            // Genesis iteration - fresh start
            interaction
                .show_message(
                    crate::interaction::MessageLevel::Info,
                    "Starting fresh (no inheritance)".to_string(),
                )
                .await;
        }
        InheritanceMode::Full => {
            // Full inheritance - copy all files including artifacts
            let base_workspace = iteration_store.workspace_path(base_iteration_id)?;

            interaction
                .show_message(
                    crate::interaction::MessageLevel::Info,
                    format!(
                        "Inheriting fully from iteration {}...",
                        base_iteration_id
                    ),
                )
                .await;

            copy_dir_all(&base_workspace, workspace).await?;
        }
        InheritanceMode::Partial => {
            // Partial inheritance - copy code files only, not artifacts
            let base_workspace = iteration_store.workspace_path(base_iteration_id)?;

            interaction
                .show_message(
                    crate::interaction::MessageLevel::Info,
                    format!(
                        "Inheriting code from iteration {} (partial)...",
                        base_iteration_id
                    ),
                )
                .await;

            // Copy only code files (workspace/src, etc.), skip artifacts
            copy_code_files(&base_workspace, workspace).await?;
        }
    }

    Ok(())
}

/// Check if artifact exists for a stage
pub async fn check_artifact_exists(stage_name: &str, workspace: &std::path::Path) -> bool {
    let iteration_dir = workspace.parent().unwrap_or(workspace);
    let artifacts_dir = iteration_dir.join("artifacts");

    let artifact_name = match stage_name {
        "idea" => "idea.md",
        "prd" => "prd.md",
        "design" => "design.md",
        "plan" => "plan.md",
        "coding" => return true, // Coding stage doesn't have a single artifact file
        "check" => "check_report.md",
        "delivery" => "delivery_report.md",
        _ => return false,
    };

    let artifact_path = artifacts_dir.join(artifact_name);
    if !artifact_path.exists() {
        return false;
    }

    // Check if the file has content
    match std::fs::read_to_string(&artifact_path) {
        Ok(content) => !content.trim().is_empty(),
        Err(_) => false,
    }
}

/// Copy all files from source to destination
async fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            Box::pin(copy_dir_all(&src_path, &dst_path)).await?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Copy only code files (skip artifacts directory)
async fn copy_code_files(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // Skip artifacts directory
        if entry.file_name() == "artifacts" {
            continue;
        }

        if ty.is_dir() {
            Box::pin(copy_code_files(&src_path, &dst_path)).await?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper to create proper directory structure:
    /// iteration_dir/
    /// ├── artifacts/
    /// │   └── idea.md
    /// └── workspace/  <-- this is passed to check_artifact_exists
    fn create_test_structure() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let iteration_dir = temp_dir.path().join("iteration");
        let workspace = iteration_dir.join("workspace");
        let artifacts_dir = iteration_dir.join("artifacts");
        
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&artifacts_dir).unwrap();
        
        (temp_dir, workspace)
    }

    #[tokio::test]
    async fn test_check_artifact_exists_file_present() {
        let (_temp_dir, workspace) = create_test_structure();
        
        // Get iteration dir and create artifact
        let iteration_dir = workspace.parent().unwrap();
        let artifacts_dir = iteration_dir.join("artifacts");
        
        // Create idea.md with content
        let idea_path = artifacts_dir.join("idea.md");
        let mut file = std::fs::File::create(&idea_path).unwrap();
        file.write_all(b"Some content").unwrap();
        drop(file);

        let exists = check_artifact_exists("idea", &workspace).await;
        assert!(exists);
    }

    #[tokio::test]
    async fn test_check_artifact_exists_file_missing() {
        let (_temp_dir, workspace) = create_test_structure();
        let exists = check_artifact_exists("idea", &workspace).await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_check_artifact_exists_file_empty() {
        let (_temp_dir, workspace) = create_test_structure();
        
        // Get iteration dir and create empty artifact
        let iteration_dir = workspace.parent().unwrap();
        let artifacts_dir = iteration_dir.join("artifacts");
        
        // Create empty idea.md
        let idea_path = artifacts_dir.join("idea.md");
        std::fs::File::create(&idea_path).unwrap();

        let exists = check_artifact_exists("idea", &workspace).await;
        assert!(!exists); // Empty file should return false
    }

    #[tokio::test]
    async fn test_check_artifact_exists_coding_stage() {
        let temp_dir = TempDir::new().unwrap();
        // Coding stage always returns true (no single artifact file)
        let exists = check_artifact_exists("coding", temp_dir.path()).await;
        assert!(exists);
    }

    #[tokio::test]
    async fn test_check_artifact_exists_unknown_stage() {
        let temp_dir = TempDir::new().unwrap();
        let exists = check_artifact_exists("unknown", temp_dir.path()).await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_copy_dir_all() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");

        // Create source structure
        std::fs::create_dir_all(src.join("subdir")).unwrap();
        let mut file1 = std::fs::File::create(src.join("file1.txt")).unwrap();
        file1.write_all(b"content1").unwrap();
        let mut file2 = std::fs::File::create(src.join("subdir/file2.txt")).unwrap();
        file2.write_all(b"content2").unwrap();

        // Copy
        copy_dir_all(&src, &dst).await.unwrap();

        // Verify
        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("subdir/file2.txt").exists());
        assert_eq!(std::fs::read_to_string(dst.join("file1.txt")).unwrap(), "content1");
    }

    #[tokio::test]
    async fn test_copy_code_files_skips_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");

        // Create source structure with artifacts
        std::fs::create_dir_all(src.join("artifacts")).unwrap();
        std::fs::create_dir_all(src.join("src")).unwrap();
        
        let mut code_file = std::fs::File::create(src.join("src/main.rs")).unwrap();
        code_file.write_all(b"fn main() {}").unwrap();
        
        let mut artifact_file = std::fs::File::create(src.join("artifacts/idea.md")).unwrap();
        artifact_file.write_all(b"# Idea").unwrap();

        // Copy
        copy_code_files(&src, &dst).await.unwrap();

        // Verify: code files copied, artifacts not
        assert!(dst.join("src/main.rs").exists());
        assert!(!dst.join("artifacts").exists());
    }
}
