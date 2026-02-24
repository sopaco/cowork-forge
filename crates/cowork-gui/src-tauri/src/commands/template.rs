use crate::gui_types::{ProjectTemplate, TemplateConfig, TemplateFile};
use std::fs;
use std::path::PathBuf;

fn templates_dir() -> Result<PathBuf, String> {
    let cfg = dirs::config_dir().ok_or("Cannot determine config directory")?;
    Ok(cfg.join("cowork-forge").join("templates"))
}

fn built_in_templates() -> Vec<ProjectTemplate> {
    vec![
        ProjectTemplate {
            id: "builtin-rest-api".into(), name: "REST API 服务".into(),
            description: "创建 REST API 服务".into(), category: "backend".into(),
            technology_stack: vec!["Rust".into(), "Axum".into()],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            is_built_in: true, files: vec![],
            config: TemplateConfig { variables: vec![], post_creation_commands: vec!["cargo build".into()] },
        },
        ProjectTemplate {
            id: "builtin-web-app".into(), name: "Web 应用".into(),
            description: "创建 Web 前端应用".into(), category: "frontend".into(),
            technology_stack: vec!["JavaScript".into(), "React".into()],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            is_built_in: true, files: vec![],
            config: TemplateConfig { variables: vec![], post_creation_commands: vec!["npm install".into()] },
        },
    ]
}

#[tauri::command]
pub async fn get_templates() -> Result<Vec<ProjectTemplate>, String> {
    let dir = templates_dir()?;
    let mut templates = built_in_templates();

    if dir.exists() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                if e.metadata().map(|m| m.is_file()).unwrap_or(false) {
                    let p = e.path();
                    if p.extension().and_then(|x| x.to_str()) == Some("json") {
                        if let Ok(c) = fs::read_to_string(&p) {
                            if let Ok(t) = serde_json::from_str(&c) { templates.push(t); }
                        }
                    }
                }
            }
        }
    }
    Ok(templates)
}

#[tauri::command]
pub async fn export_template(
    _session_id: String,
    name: String,
    description: String,
    category: String,
) -> Result<ProjectTemplate, String> {
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    if !root.exists() { return Err("Project directory not found".into()); }

    let mut files = vec![];
    collect_files(&root, &mut files)?;

    let id = format!("template-{}", chrono::Utc::now().timestamp_millis());
    let tpl = ProjectTemplate {
        id: id.clone(), name, description, category,
        technology_stack: vec![], files,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        is_built_in: false,
        config: TemplateConfig { variables: vec![], post_creation_commands: vec![] },
    };

    let dir = templates_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{}.json", id)), serde_json::to_string_pretty(&tpl).unwrap())
        .map_err(|e| e.to_string())?;

    Ok(tpl)
}

#[tauri::command]
pub async fn import_template(template_data: String) -> Result<ProjectTemplate, String> {
    let tpl: ProjectTemplate = serde_json::from_str(&template_data).map_err(|e| e.to_string())?;
    if tpl.id.is_empty() || tpl.name.is_empty() { return Err("Invalid template".into()); }

    let dir = templates_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{}.json", tpl.id)), serde_json::to_string_pretty(&tpl).unwrap())
        .map_err(|e| e.to_string())?;

    Ok(tpl)
}

#[tauri::command]
pub async fn delete_template(template_id: String) -> Result<(), String> {
    let f = templates_dir()?.join(format!("{}.json", template_id));
    if !f.exists() { return Err("Template not found".into()); }
    fs::remove_file(&f).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_template(
    template_id: String,
    _variables: serde_json::Value,
    target_dir: String,
) -> Result<Vec<String>, String> {
    let dir = templates_dir()?;
    let file = dir.join(format!("{}.json", template_id));

    let tpl = if file.exists() {
        let c = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        serde_json::from_str(&c).map_err(|e| e.to_string())?
    } else {
        built_in_templates().iter().find(|t| t.id == template_id)
            .ok_or("Template not found")?.clone()
    };

    apply_files(&tpl, &target_dir)
}

fn apply_files(tpl: &ProjectTemplate, target: &str) -> Result<Vec<String>, String> {
    let tdir = PathBuf::from(target);
    fs::create_dir_all(&tdir).map_err(|e| e.to_string())?;

    let mut created = vec![];
    for f in &tpl.files {
        let fp = tdir.join(&f.path);
        if let Some(p) = fp.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
        fs::write(&fp, &f.content).map_err(|e| e.to_string())?;
        created.push(f.path.clone());
    }

    for cmd in &tpl.config.post_creation_commands {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if !parts.is_empty() {
            let _ = std::process::Command::new(parts[0]).args(&parts[1..]).current_dir(&tdir).status();
        }
    }

    Ok(created)
}

fn collect_files(d: &std::path::Path, files: &mut Vec<TemplateFile>) -> Result<(), String> {
    if let Ok(entries) = fs::read_dir(d) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == ".cowork-v2" { continue; }

            if let Ok(m) = e.metadata() {
                if m.is_dir() { collect_files(&p, files)?; }
                else {
                    let rel = p.strip_prefix(d).unwrap().to_string_lossy();
                    files.push(TemplateFile {
                        path: rel.to_string(),
                        content: fs::read_to_string(&p).map_err(|e| e.to_string())?,
                        is_binary: false,
                    });
                }
            }
        }
    }
    Ok(())
}
