use cowork_core::llm::config::{self, ModelConfig, LlmConfig};

#[tauri::command]
pub async fn get_app_config() -> Result<ModelConfig, String> {
    config::load_config()
        .map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
pub async fn save_app_config(config: ModelConfig) -> Result<String, String> {
    let path = config::save_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_config_path() -> Result<String, String> {
    let path = config::get_config_path()
        .map_err(|e| format!("Failed to get config path: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_default_config() -> ModelConfig {
    ModelConfig::default()
}

#[tauri::command]
pub async fn open_config_folder() -> Result<(), String> {
    let config_path = config::get_config_path()
        .map_err(|e| format!("Failed to get config path: {}", e))?;
    
    let parent = config_path.parent()
        .ok_or_else(|| "Failed to get config directory".to_string())?;
    
    open_folder_in_explorer(parent)
}

#[tauri::command]
pub async fn test_llm_connection(llm_config: LlmConfig) -> Result<bool, String> {
    use cowork_core::llm::create_llm_client;
    use adk_core::{Content, Part, LlmRequest};
    use futures::StreamExt;

    if llm_config.api_base_url.is_empty() || llm_config.api_key.is_empty() || llm_config.model_name.is_empty() {
        return Err("Please fill in all LLM settings (API URL, API Key, and Model Name)".to_string());
    }

    let client = create_llm_client(&llm_config)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;
    
    let contents = vec![Content {
        role: "user".to_string(),
        parts: vec![Part::Text { text: "Hello, this is a connection test. Please respond with 'OK'.".to_string() }],
    }];
    
    let test_request = LlmRequest {
        model: llm_config.model_name.clone(),
        contents,
        config: None,
        tools: Default::default(),
    };
    
    let mut stream = client.generate_content(test_request, false).await
        .map_err(|e| format!("Connection test failed: {}", e))?;
    
    let mut response_text = String::new();
    while let Some(chunk) = stream.next().await {
        if let Ok(r) = chunk {
            if let Some(c) = r.content {
                for p in c.parts.iter() {
                    if let Part::Text { text } = p {
                        response_text.push_str(text);
                    }
                }
            }
        }
    }
    
    if response_text.is_empty() {
        return Err("Connection test failed: Empty response from API".to_string());
    }
    
    Ok(true)
}

#[tauri::command]
pub async fn has_valid_config() -> bool {
    config::load_config().is_ok()
}

fn open_folder_in_explorer(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}
