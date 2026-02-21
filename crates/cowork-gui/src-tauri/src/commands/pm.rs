use cowork_core::persistence::IterationStore;
use tauri::{Emitter, Window};

#[tauri::command]
pub async fn pm_send_message(
    iteration_id: String,
    message: String,
    _history: Vec<serde_json::Value>,
    window: Window,
) -> Result<serde_json::Value, String> {
    println!("[PM] pm_send_message called: iteration_id={}, message={}", iteration_id, message);
    
    let store = IterationStore::new();
    let iteration = store.load(&iteration_id).map_err(|e| {
        let err = format!("Failed to load iteration: {}", e);
        println!("[PM] Error: {}", err);
        err
    })?;

    let config = cowork_core::llm::config::load_config().map_err(|e| {
        let err = format!("Failed to load config: {}", e);
        println!("[PM] Error: {}", err);
        err
    })?;

    let prompt = format!(
        "You are a Project Manager Agent helping with iteration: {}.\nTitle: {}\nDescription: {}\n\nUser message: {}\n\nPlease provide a helpful response.",
        iteration_id, iteration.title, iteration.description, message
    );

    println!("[PM] Creating LLM client...");
    let client = cowork_core::llm::create_llm_client(&config.llm).map_err(|e| {
        let err = format!("Failed to create LLM client: {}", e);
        println!("[PM] Error: {}", err);
        err
    })?;
    
    let req = adk_core::model::LlmRequest {
        model: config.llm.model_name.clone(),
        contents: vec![adk_core::Content {
            role: "user".to_string(),
            parts: vec![adk_core::Part::Text { text: prompt }],
        }],
        config: None,
        tools: std::collections::HashMap::new(),
    };

    println!("[PM] Calling generate_content...");
    let mut stream = client.generate_content(req, false).await.map_err(|e| {
        let err = format!("Failed to generate content: {}", e);
        println!("[PM] Error: {}", err);
        err
    })?;
    
    use futures::StreamExt;
    println!("[PM] Reading from stream...");
    
    let mut all_text = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(r) => {
                println!("[PM] Got chunk, content: {:?}", r.content);
                if let Some(c) = r.content {
                    println!("[PM] Parts count: {}", c.parts.len());
                    for (i, p) in c.parts.iter().enumerate() {
                        println!("[PM] Part {}: {:?}", i, p);
                        if let adk_core::Part::Text { text } = p {
                            all_text.push_str(text);
                        }
                    }
                }
            }
            Err(e) => {
                let err = format!("Stream error: {}", e);
                println!("[PM] Error: {}", err);
                return Err(err);
            }
        }
    }
    
    let response = if all_text.is_empty() {
        println!("[PM] No text found in any chunk");
        "No response".to_string()
    } else {
        all_text
    };

    println!("[PM] Response: {}", response);
    
    let result = serde_json::json!({
        "agent_message": response,
        "actions": [],
        "needs_restart": false
    });

    window.emit("pm_message", &result).map_err(|e| {
        let err = format!("Failed to emit event: {}", e);
        println!("[PM] Error: {}", err);
        err
    })?;
    
    println!("[PM] Success!");
    Ok(result)
}

#[tauri::command]
pub async fn pm_restart_iteration(iteration_id: String, target_stage: String) -> Result<(), String> {
    let store = IterationStore::new();
    let mut iter = store.load(&iteration_id).map_err(|e| e.to_string())?;

    iter.current_stage = Some(target_stage);
    iter.status = cowork_core::domain::IterationStatus::Paused;

    store.save(&iter).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pm_get_iteration_context(iteration_id: String) -> Result<serde_json::Value, String> {
    let store = IterationStore::new();
    let iter = store.load(&iteration_id).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "id": iter.id,
        "title": iter.title,
        "description": iter.description,
        "status": format!("{:?}", iter.status),
        "current_stage": iter.current_stage,
        "completed_stages": iter.completed_stages,
    }))
}
