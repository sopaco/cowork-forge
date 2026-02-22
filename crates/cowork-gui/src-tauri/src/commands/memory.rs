use cowork_core::persistence::MemoryStore;

#[tauri::command]
pub async fn query_memory_index(
    iteration_id: String,
    query_type: String,
    category: Option<String>,
    stage: Option<String>,
    limit: Option<u64>,
) -> Result<serde_json::Value, String> {
    let store = MemoryStore::new();
    let limit = limit.unwrap_or(20) as usize;
    
    let mut results: Vec<serde_json::Value> = Vec::new();

    // Load iteration memory if needed
    let iter_mem = if query_type == "all" || query_type == "session" {
        Some(store.load_iteration_memory(&iteration_id).map_err(|e| e.to_string())?)
    } else {
        None
    };

    // Load project memory if needed
    let proj_mem = if query_type == "all" || query_type == "project" {
        Some(store.load_project_memory().map_err(|e| e.to_string())?)
    } else {
        None
    };

    // Helper to check if item matches category filter
    let matches_category = |cat: &str| -> bool {
        match category.as_deref() {
            Some("all") | None => true,
            Some(c) => c == cat,
        }
    };

    // Helper to check if item matches stage filter
    let matches_stage = |s: Option<&str>| -> bool {
        match stage.as_deref() {
            Some("all") | None => true,
            Some(stage_filter) => s.map(|st| st == stage_filter).unwrap_or(false),
        }
    };

    // Collect from iteration memory
    if let Some(ref mem) = iter_mem {
        // Add insights
        if matches_category("decision") || category.is_none() || category.as_deref() == Some("all") {
            for i in &mem.insights {
                if matches_stage(Some(&i.stage)) {
                    results.push(serde_json::json!({
                        "id": format!("insight-{}", i.created_at.timestamp()),
                        "title": format!("Insight from {} stage", i.stage),
                        "summary": i.content.chars().take(200).collect::<String>(),
                        "category": "decision",
                        "stage": i.stage,
                        "impact": format!("{:?}", i.importance).to_lowercase(),
                        "created_at": i.created_at.to_rfc3339(),
                        "file": format!("iterations/{}.json", iteration_id),
                    }));
                }
            }
        }

        // Add issues
        if matches_category("experience") || category.is_none() || category.as_deref() == Some("all") {
            for i in &mem.issues {
                if matches_stage(Some(&i.stage)) {
                    results.push(serde_json::json!({
                        "id": format!("issue-{}", i.created_at.timestamp()),
                        "title": format!("Issue from {} stage", i.stage),
                        "summary": i.content.chars().take(200).collect::<String>(),
                        "category": "experience",
                        "stage": i.stage,
                        "impact": if i.resolved { "low" } else { "high" },
                        "created_at": i.created_at.to_rfc3339(),
                        "file": format!("iterations/{}.json", iteration_id),
                    }));
                }
            }
        }

        // Add learnings
        if matches_category("pattern") || category.is_none() || category.as_deref() == Some("all") {
            for l in &mem.learnings {
                results.push(serde_json::json!({
                    "id": format!("learning-{}", l.created_at.timestamp()),
                    "title": "Learning",
                    "summary": l.content.chars().take(200).collect::<String>(),
                    "category": "pattern",
                    "stage": null,
                    "created_at": l.created_at.to_rfc3339(),
                    "file": format!("iterations/{}.json", iteration_id),
                }));
            }
        }
    }

    // Collect from project memory
    if let Some(ref mem) = proj_mem {
        // Add decisions
        if matches_category("decision") || category.is_none() || category.as_deref() == Some("all") {
            for d in &mem.decisions {
                results.push(serde_json::json!({
                    "id": d.id.clone(),
                    "title": d.title.clone(),
                    "summary": d.decision.chars().take(200).collect::<String>(),
                    "category": "decision",
                    "stage": null,
                    "impact": "high",
                    "created_at": d.created_at.to_rfc3339(),
                    "file": "project/memory.json",
                    "tags": d.consequences.clone(),
                }));
            }
        }

        // Add patterns
        if matches_category("pattern") || category.is_none() || category.as_deref() == Some("all") {
            for p in &mem.patterns {
                results.push(serde_json::json!({
                    "id": p.id.clone(),
                    "title": p.name.clone(),
                    "summary": p.description.chars().take(200).collect::<String>(),
                    "category": "pattern",
                    "stage": null,
                    "created_at": p.created_at.to_rfc3339(),
                    "file": "project/memory.json",
                }));
            }
        }
    }

    // Sort by created_at descending
    results.sort_by(|a, b| {
        let a_time = a.get("created_at").and_then(|t| t.as_str()).unwrap_or("");
        let b_time = b.get("created_at").and_then(|t| t.as_str()).unwrap_or("");
        b_time.cmp(a_time)
    });

    // Truncate to limit
    results.truncate(limit);

    Ok(serde_json::json!({
        "results": results,
        "total": results.len(),
    }))
}

#[tauri::command]
pub async fn load_memory_detail(
    memory_id: String,
    file: Option<String>,
    iteration_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let store = MemoryStore::new();

    // Check if it's a project-level memory (file == "project/memory.json")
    if let Some(ref f) = file {
        if f == "project/memory.json" {
            let proj_mem = store.load_project_memory().map_err(|e| e.to_string())?;
            
            // Check decisions
            for d in &proj_mem.decisions {
                if d.id == memory_id {
                    return Ok(serde_json::json!({
                        "id": memory_id,
                        "content": format!("## {}\n\n### Context\n{}\n\n### Decision\n{}", d.title, d.context, d.decision),
                        "title": d.title,
                        "category": "decision",
                        "created_at": d.created_at.to_rfc3339(),
                    }));
                }
            }
            
            // Check patterns
            for p in &proj_mem.patterns {
                if p.id == memory_id {
                    return Ok(serde_json::json!({
                        "id": memory_id,
                        "content": format!("## {}\n\n{}", p.name, p.description),
                        "title": p.name,
                        "category": "pattern",
                        "created_at": p.created_at.to_rfc3339(),
                    }));
                }
            }
        }
    }

    // Check iteration memory
    if let Some(iter_id) = iteration_id {
        let iter_mem = store.load_iteration_memory(&iter_id).map_err(|e| e.to_string())?;

        if memory_id.starts_with("insight-") {
            if let Ok(ts) = memory_id.replace("insight-", "").parse::<i64>() {
                for i in &iter_mem.insights {
                    if i.created_at.timestamp() == ts {
                        return Ok(serde_json::json!({
                            "id": memory_id,
                            "content": i.content,
                            "title": format!("Insight from {} stage", i.stage),
                            "stage": i.stage,
                            "category": "decision",
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
                            "title": format!("Issue from {} stage", i.stage),
                            "stage": i.stage,
                            "category": "experience",
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
                            "title": "Learning",
                            "category": "pattern",
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
