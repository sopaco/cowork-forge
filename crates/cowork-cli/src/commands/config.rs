//! Configure LLM settings command

use anyhow::{Context, Result};
use cowork_core::llm::config::{get_config_path, load_config, save_config, ModelConfig};

/// Configure LLM settings
pub async fn execute() -> Result<()> {
    println!("⚙️  Cowork Configuration\n");
    
    let config_path = get_config_path()
        .context("Failed to get config path")?;
    
    println!("Config file location: {}", config_path.display());
    
    let existing_config = load_config().ok();
    
    if let Some(ref config) = existing_config {
        println!("\nCurrent LLM Configuration:");
        println!("  API Base URL: {}", config.llm.api_base_url);
        println!("  Model Name:   {}", config.llm.model_name);
        println!("  API Key:      {}...", &config.llm.api_key.chars().take(8).collect::<String>());
        
        if config.coding_agent.enabled {
            println!("\n  Coding Agent: enabled ({})", config.coding_agent.agent_type);
        }
    } else {
        println!("\nNo configuration found. Creating default config...");
    }
    
    println!("\nTo edit the configuration, open the file in your editor:");
    println!("  {}", config_path.display());
    
    if existing_config.is_none() {
        let default_config = ModelConfig::default();
        save_config(&default_config)?;
        println!("\n✅ Created default config file at: {}", config_path.display());
        println!("   Please edit the file to add your LLM API settings.");
    }
    
    Ok(())
}

