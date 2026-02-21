use cowork_core::persistence::IterationStore;
use tauri::{Emitter, Window};

#[tauri::command]
pub async fn pm_send_message(
    iteration_id: String,
    message: String,
    history: Vec<serde_json::Value>,
    window: Window,
) -> Result<serde_json::Value, String> {
    let store = IterationStore::new();
    let iteration = store.load(&iteration_id).map_err(|e| format!("Failed to load iteration: {}", e))?;
    
    let is_first_message = history.is_empty();
    
    if is_first_message {
        let welcome_msg = format!(
            "👋 你好！我是项目经理助手。\n\n项目 **{}** 已经完成开发！\n\n接下来你可以：",
            iteration.title
        );
        
        let actions = vec![
            serde_json::json!({ "action_type": "pm_start_app", "label": "🚀 启动应用" }),
            serde_json::json!({ "action_type": "pm_open_folder", "label": "📁 打开项目文件夹" }),
            serde_json::json!({ "action_type": "pm_view_artifacts", "label": "📄 查看设计文档" }),
            serde_json::json!({ "action_type": "pm_view_code", "label": "💻 查看源代码" }),
            serde_json::json!({ "action_type": "pm_view_knowledge", "label": "📚 查看项目知识库" }),
        ];
        
        let result = serde_json::json!({
            "agent_message": welcome_msg,
            "actions": actions,
            "needs_restart": false
        });
        
        let _ = window.emit("pm_message", &result);
        return Ok(result);
    }
    
    let config = cowork_core::llm::config::load_config().map_err(|e| format!("Failed to load config: {}", e))?;

    let prompt = format!(
        "You are a Project Manager Agent helping with iteration: {}.\nTitle: {}\nDescription: {}\n\nUser message: {}\n\nPlease provide a helpful response in Chinese.",
        iteration_id, iteration.title, iteration.description, message
    );

    let client = cowork_core::llm::create_llm_client(&config.llm).map_err(|e| format!("Failed to create LLM client: {}", e))?;
    
    let req = adk_core::model::LlmRequest {
        model: config.llm.model_name.clone(),
        contents: vec![adk_core::Content {
            role: "user".to_string(),
            parts: vec![adk_core::Part::Text { text: prompt }],
        }],
        config: None,
        tools: std::collections::HashMap::new(),
    };

    let mut stream = client.generate_content(req, false).await.map_err(|e| format!("Failed to generate content: {}", e))?;
    
    use futures::StreamExt;
    
    let mut all_text = String::new();
    while let Some(chunk) = stream.next().await {
        if let Ok(r) = chunk {
            if let Some(c) = r.content {
                for p in c.parts.iter() {
                    if let adk_core::Part::Text { text } = p {
                        all_text.push_str(text);
                    }
                }
            }
        }
    }
    
    let response = if all_text.is_empty() { "No response".to_string() } else { all_text };
    
    let result = serde_json::json!({
        "agent_message": response,
        "actions": [],
        "needs_restart": false
    });
    
    let _ = window.emit("pm_message", &result);
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

#[tauri::command]
pub async fn pm_get_welcome_message(iteration_id: String) -> Result<serde_json::Value, String> {
    let store = IterationStore::new();
    let iteration = store.load(&iteration_id).map_err(|e| format!("Failed to load iteration: {}", e))?;
    
    let welcome_msg = format!(
        "👋 你好！我是项目经理助手。\n\n项目 **{}** 已经完成开发！\n\n接下来你可以：",
        iteration.title
    );
    
    let actions = vec![
        serde_json::json!({ "action_type": "pm_start_app", "label": "🚀 启动应用" }),
        serde_json::json!({ "action_type": "pm_open_folder", "label": "📁 打开项目文件夹" }),
        serde_json::json!({ "action_type": "pm_view_artifacts", "label": "📄 查看设计文档" }),
        serde_json::json!({ "action_type": "pm_view_code", "label": "💻 查看源代码" }),
        serde_json::json!({ "action_type": "pm_view_knowledge", "label": "📚 查看项目知识库" }),
    ];
    
    Ok(serde_json::json!({
        "agent_message": welcome_msg,
        "actions": actions
    }))
}
