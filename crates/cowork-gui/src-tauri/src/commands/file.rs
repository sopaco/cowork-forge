use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::gui_types::*;

#[tauri::command]
pub async fn open_in_file_manager(path: String) -> Result<(), String> {
    let resolved_path = if path.starts_with("workspace_") {
        let iteration_id = path.strip_prefix("workspace_").unwrap_or(&path);
        cowork_core::persistence::IterationStore::new()
            .workspace_path(iteration_id)
            .map_err(|e| e.to_string())?
    } else if path.contains("iter-") {
        cowork_core::persistence::IterationStore::new()
            .iteration_path(&path)
            .map_err(|e| e.to_string())?
    } else {
        PathBuf::from(path)
    };

    if !resolved_path.exists() {
        return Err(format!("Path does not exist: {}", resolved_path.display()));
    }

    if cfg!(target_os = "windows") {
        Command::new("explorer").arg(&resolved_path).spawn().map_err(|e| e.to_string())?;
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&resolved_path).spawn().map_err(|e| e.to_string())?;
    } else {
        Command::new("xdg-open").arg(&resolved_path).spawn().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_iteration_artifacts(
    iteration_id: String,
) -> Result<IterationArtifacts, String> {
    let iteration_store = cowork_core::persistence::IterationStore::new();
    iteration_store.load(&iteration_id).map_err(|e| e.to_string())?;

    let iteration_dir = iteration_store
        .iteration_path(&iteration_id)
        .map_err(|e| e.to_string())?;
    let artifacts_dir = iteration_dir.join("artifacts");

    let idea = fs::read_to_string(artifacts_dir.join("idea.md")).ok();
    let prd = fs::read_to_string(artifacts_dir.join("prd.md")).ok();
    let design = fs::read_to_string(artifacts_dir.join("design.md")).ok();
    let plan = fs::read_to_string(artifacts_dir.join("plan.md")).ok();
    let delivery_report = fs::read_to_string(artifacts_dir.join("delivery_report.md")).ok();
    let check_report = fs::read_to_string(artifacts_dir.join("check_report.md")).ok();

    let workspace = iteration_store
        .workspace_path(&iteration_id)
        .map_err(|e| e.to_string())?;
    let code_files = if workspace.exists() { collect_files(&workspace) } else { vec![] };

    Ok(IterationArtifacts {
        iteration_id,
        idea,
        requirements: prd,
        design,
        plan,
        code_files,
        check_report,
        delivery_report,
    })
}

#[tauri::command]
pub async fn read_iteration_file(
    iteration_id: String,
    file_path: String,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<FileReadResult, String> {
    let iteration_store = cowork_core::persistence::IterationStore::new();
    let workspace = iteration_store
        .workspace_path(&iteration_id)
        .map_err(|e| e.to_string())?;

    let full_path = workspace.join(&file_path);
    let metadata = fs::metadata(&full_path).map_err(|e| e.to_string())?;
    let file_size = metadata.len() as usize;
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

    if file_size > MAX_FILE_SIZE || offset.is_some() || limit.is_some() {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(1024 * 1024);

        use std::io::{Read, Seek};
        let mut file = fs::File::open(&full_path).map_err(|e| e.to_string())?;
        file.seek(std::io::SeekFrom::Start(offset as u64)).map_err(|e| e.to_string())?;

        let mut buffer = vec![0; limit.min(file_size.saturating_sub(offset))];
        let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        buffer.truncate(bytes_read);

        let content = String::from_utf8_lossy(&buffer).to_string();
        Ok(FileReadResult {
            content,
            offset: offset as u64,
            total_size: file_size as u64,
            is_partial: true,
        })
    } else {
        let content = fs::read_to_string(&full_path).map_err(|e| e.to_string())?;
        Ok(FileReadResult {
            content,
            offset: 0,
            total_size: file_size as u64,
            is_partial: false,
        })
    }
}

#[tauri::command]
pub async fn save_iteration_file(
    iteration_id: String,
    file_path: String,
    content: String,
) -> Result<(), String> {
    let iteration_store = cowork_core::persistence::IterationStore::new();
    let workspace = iteration_store
        .workspace_path(&iteration_id)
        .map_err(|e| e.to_string())?;

    let full_path = workspace.join(&file_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&full_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_iteration_file_tree(iteration_id: String) -> Result<FileTreeNode, String> {
    let iteration_store = cowork_core::persistence::IterationStore::new();
    let workspace = iteration_store
        .workspace_path(&iteration_id)
        .map_err(|e| e.to_string())?;

    if !workspace.exists() {
        return Ok(FileTreeNode {
            name: workspace.file_name().unwrap_or(workspace.as_os_str()).to_string_lossy().to_string(),
            path: ".".to_string(),
            is_dir: true,
            children: Some(vec![]),
            is_expanded: true,
            language: None,
        });
    }

    build_file_tree(&workspace, &workspace, 0)
}

fn collect_files(dir: &Path) -> Vec<FileInfo> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = entry.metadata() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let relative_path = path.strip_prefix(dir).unwrap().to_string_lossy().to_string();
                let modified_at = meta.modified().ok().map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.to_rfc3339()
                });
                if meta.is_dir() {
                    files.extend(collect_files(&path));
                } else {
                    files.push(FileInfo {
                        name,
                        path: relative_path,
                        size: meta.len(),
                        is_dir: false,
                        language: detect_language(&path),
                        modified_at,
                    });
                }
            }
        }
    }
    files
}

fn detect_language(path: &Path) -> Option<String> {
    match path.extension()?.to_str()? {
        "rs" => Some("rust".to_string()),
        "js" => Some("javascript".to_string()),
        "ts" => Some("typescript".to_string()),
        "jsx" => Some("javascript".to_string()),
        "tsx" => Some("typescript".to_string()),
        "py" => Some("python".to_string()),
        "json" => Some("json".to_string()),
        "html" => Some("html".to_string()),
        "css" => Some("css".to_string()),
        "md" => Some("markdown".to_string()),
        _ => None,
    }
}

fn build_file_tree(dir: &Path, base: &Path, depth: usize) -> Result<FileTreeNode, String> {
    if depth > 50 {
        return Err("Max depth exceeded".to_string());
    }

    let name = dir.file_name().unwrap_or(dir.as_os_str()).to_string_lossy().to_string();
    let relative_path = dir.strip_prefix(base).unwrap().to_string_lossy().to_string();
    let mut children = Vec::new();
    let is_expanded = depth < 2;

    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by(|a, b| {
            let a_is_dir = a.metadata().map(|m| m.is_dir()).unwrap_or(false);
            let b_is_dir = b.metadata().map(|m| m.is_dir()).unwrap_or(false);
            b_is_dir.cmp(&a_is_dir)
        });

        for entry in entries {
            let path = entry.path();
            let entry_name = path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().to_string();

            if entry_name.starts_with('.') || entry_name == "node_modules" || entry_name == "target" {
                continue;
            }

            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    if let Ok(child) = build_file_tree(&path, base, depth + 1) {
                        children.push(child);
                    }
                } else {
                    children.push(FileTreeNode {
                        name: entry_name,
                        path: path.strip_prefix(base).unwrap().to_string_lossy().to_string(),
                        is_dir: false,
                        children: None,
                        is_expanded: false,
                        language: detect_language(&path),
                    });
                }
            }
        }
    }

    Ok(FileTreeNode {
        name,
        path: relative_path,
        is_dir: true,
        children: Some(children),
        is_expanded,
        language: None,
    })
}
