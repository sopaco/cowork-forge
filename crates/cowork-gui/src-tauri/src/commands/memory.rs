use cowork_core::persistence::MemoryStore;

#[tauri::command]
pub async fn query_memory_index(
    iteration_id: String,
    query_type: String,
    _category: Option<String>,
) -> Result<serde_json::Value, String> {
    let store = MemoryStore::new();
    let iter_mem = store.load_iteration_memory(&iteration_id).map_err(|e| e.to_string())?;

    let results: Vec<serde_json::Value> = match query_type.as_str() {
        "insight" => iter_mem.insights.iter().map(|i| serde_json::json!({
            "id": format!("insight-{}", i.created_at.timestamp()),
            "content": i.content,
            "stage": i.stage,
            "importance": format!("{:?}", i.importance),
            "created_at": i.created_at.to_rfc3339(),
        })).collect(),
        "issue" => iter_mem.issues.iter().map(|i| serde_json::json!({
            "id": format!("issue-{}", i.created_at.timestamp()),
            "content": i.content,
            "stage": i.stage,
            "resolved": i.resolved,
            "created_at": i.created_at.to_rfc3339(),
        })).collect(),
        "learning" => iter_mem.learnings.iter().map(|l| serde_json::json!({
            "id": format!("learning-{}", l.created_at.timestamp()),
            "content": l.content,
            "created_at": l.created_at.to_rfc3339(),
        })).collect(),
        _ => vec![],
    };

    Ok(serde_json::json!(results))
}

#[tauri::command]
pub async fn load_memory_detail(
    memory_id: String,
    iteration_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let store = MemoryStore::new();

    if let Some(iter_id) = iteration_id {
        let iter_mem = store.load_iteration_memory(&iter_id).map_err(|e| e.to_string())?;

        if memory_id.starts_with("insight-") {
            if let Ok(ts) = memory_id.replace("insight-", "").parse::<i64>() {
                for i in &iter_mem.insights {
                    if i.created_at.timestamp() == ts {
                        return Ok(serde_json::json!({
                            "id": memory_id,
                            "content": i.content,
                            "stage": i.stage,
                            "importance": format!("{:?}", i.importance),
                            "created_at": i.created_at.to_rfc3339(),
                        }));
                    }
                }
            }
        }

        if memory_id.starts_with("issue-") {
            if let Ok(ts) = memory_id.replace("issue-", "").parse::<i64>() {
                for i in &iter_mem.issues {
                    if i.created_at.timestamp() == ts {
                        return Ok(serde_json::json!({
                            "id": memory_id,
                            "content": i.content,
                            "stage": i.stage,
                            "resolved": i.resolved,
                            "created_at": i.created_at.to_rfc3339(),
                        }));
                    }
                }
            }
        }

        if memory_id.starts_with("learning-") {
            if let Ok(ts) = memory_id.replace("learning-", "").parse::<i64>() {
                for l in &iter_mem.learnings {
                    if l.created_at.timestamp() == ts {
                        return Ok(serde_json::json!({
                            "id": memory_id,
                            "content": l.content,
                            "created_at": l.created_at.to_rfc3339(),
                        }));
                    }
                }
            }
        }
    }

    Err(format!("Memory item not found: {}", memory_id))
}

#[tauri::command]
pub async fn save_session_memory(
    iteration_id: String,
    content_type: String,
    _category: String,
    content: String,
) -> Result<(), String> {
    let store = MemoryStore::new();
    let mut iter_mem = store.load_iteration_memory(&iteration_id).unwrap_or_else(|_| {
        cowork_core::domain::IterationMemory {
            iteration_id: iteration_id.clone(),
            insights: vec![],
            issues: vec![],
            learnings: vec![],
        }
    });

    let now = chrono::Utc::now();

    match content_type.as_str() {
        "insight" => iter_mem.insights.push(cowork_core::domain::Insight {
            stage: String::new(),
            content,
            importance: cowork_core::domain::Importance::Normal,
            created_at: now,
        }),
        "issue" => iter_mem.issues.push(cowork_core::domain::Issue {
            stage: String::new(),
            content,
            resolved: false,
            created_at: now,
            resolved_at: None,
        }),
        "learning" => iter_mem.learnings.push(cowork_core::domain::Learning {
            content,
            created_at: now,
        }),
        t => return Err(format!("Unknown content type: {}", t)),
    }

    store.save_iteration_memory(&iter_mem).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn promote_to_project_memory(
    memory_id: String,
    iteration_id: String,
) -> Result<serde_json::Value, String> {
    let store = MemoryStore::new();
    let iter_mem = store.load_iteration_memory(&iteration_id).map_err(|e| e.to_string())?;

    if memory_id.starts_with("insight-") {
        if let Ok(ts) = memory_id.replace("insight-", "").parse::<i64>() {
            for i in &iter_mem.insights {
                if i.created_at.timestamp() == ts {
                    let dec = cowork_core::domain::Decision::new(
                        "Insight",
                        "",
                        &i.content,
                        &iteration_id,
                    );
                    store.add_decision(dec).map_err(|e| e.to_string())?;
                    return Ok(serde_json::json!({ "message": "Promoted to decision", "memory_id": memory_id }));
                }
            }
        }
    }

    if memory_id.starts_with("learning-") {
        if let Ok(ts) = memory_id.replace("learning-", "").parse::<i64>() {
            for l in &iter_mem.learnings {
                if l.created_at.timestamp() == ts {
                    let pat = cowork_core::domain::Pattern::new(
                        "Learning",
                        &l.content,
                        &iteration_id,
                    );
                    store.add_pattern(pat).map_err(|e| e.to_string())?;
                    return Ok(serde_json::json!({ "message": "Promoted to pattern", "memory_id": memory_id }));
                }
            }
        }
    }

    Err(format!("Memory item not found: {}", memory_id))
}

#[tauri::command]
pub async fn get_memory_context(iteration_id: Option<String>) -> Result<serde_json::Value, String> {
    let store = MemoryStore::new();
    let proj_mem = store.load_project_memory().map_err(|e| e.to_string())?;

    let iter_mem = if let Some(id) = iteration_id {
        Some(store.load_iteration_memory(&id).map_err(|e| e.to_string())?)
    } else { None };

    Ok(serde_json::json!({
        "project_memory": {
            "total_decisions": proj_mem.decisions.len(),
            "total_patterns": proj_mem.patterns.len(),
            "key_decisions": proj_mem.decisions.iter().take(5)
                .map(|d| serde_json::json!({"id": d.id, "title": d.title})).collect::<Vec<_>>()
        },
        "iteration_memory": iter_mem.map(|m| serde_json::json!({
            "iteration_id": m.iteration_id,
            "total_insights": m.insights.len(),
            "total_issues": m.issues.len(),
            "total_learnings": m.learnings.len()
        }))
    }))
}
