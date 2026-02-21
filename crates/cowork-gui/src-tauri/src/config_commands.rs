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
    
    let _client = create_llm_client(&llm_config)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;
    
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
