// MCP (Model Context Protocol) Tools Integration
//
// This module manages remote MCP server connections and exposes their tools
// to Cowork Forge agents.

use std::sync::Arc;
use std::time::Duration;

use adk_core::Toolset;
use adk_tool::McpHttpClientBuilder;
use anyhow::{Context, Result};
use tracing;

use crate::llm::config::McpConfig;

/// Configuration for a remote MCP server
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Server name/identifier
    pub name: String,
    /// Server endpoint URL
    pub endpoint: String,
    /// Request timeout (default: 30s)
    pub timeout_secs: Option<u64>,
}

/// Connected MCP toolset with its configuration
/// We use a type-erased wrapper to avoid complex generic issues with McpToolset
pub struct ConnectedMcpToolset {
    pub name: String,
    pub toolset: Arc<dyn Toolset>,
}

impl Clone for ConnectedMcpToolset {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            toolset: Arc::clone(&self.toolset),
        }
    }
}

/// MCP Toolset Manager - manages all remote MCP connections
pub struct McpManager {
    /// List of connected MCP toolsets
    toolsets: Vec<ConnectedMcpToolset>,
    /// Server configurations
    configs: Vec<McpServerConfig>,
}

impl McpManager {
    /// Create a new MCP Manager
    pub fn new() -> Self {
        Self {
            toolsets: Vec::new(),
            configs: Vec::new(),
        }
    }

    /// Add an MCP server configuration
    pub fn add_config(&mut self, config: McpServerConfig) {
        self.configs.push(config);
    }

    /// Connect to all configured MCP servers
    pub async fn connect_all(&mut self) -> Result<Vec<ConnectedMcpToolset>> {
        let mut connected_toolsets = Vec::new();

        for config in &self.configs {
            tracing::info!(
                "Connecting to MCP server: {} at {}",
                config.name,
                config.endpoint
            );

            let toolset = Self::connect_to_server(config).await?;
            let connected = ConnectedMcpToolset {
                name: config.name.clone(),
                toolset,
            };

            tracing::info!(
                "Successfully connected to MCP server: {}, tools available",
                config.name
            );

            connected_toolsets.push(connected.clone());
            self.toolsets.push(connected);
        }

        Ok(connected_toolsets)
    }

    /// Connect to a single MCP server
    async fn connect_to_server(config: &McpServerConfig) -> Result<Arc<dyn Toolset>> {
        let builder = McpHttpClientBuilder::new(&config.endpoint);

        // Set timeout
        let builder = if let Some(timeout_secs) = config.timeout_secs {
            builder.timeout(Duration::from_secs(timeout_secs))
        } else {
            builder.timeout(Duration::from_secs(60))
        };

        // Connect to the MCP server
        let toolset = builder
            .connect()
            .await
            .with_context(|| format!("Failed to connect to MCP server: {}", config.name))?;

        // Convert to Arc<dyn Toolset>
        Ok(Arc::new(toolset))
    }

    /// Get all connected toolsets
    pub fn get_toolsets(&self) -> &[ConnectedMcpToolset] {
        &self.toolsets
    }

    /// Check if any MCP servers are configured
    pub fn has_servers(&self) -> bool {
        !self.configs.is_empty()
    }

    /// Get the number of connected toolsets
    pub fn toolset_count(&self) -> usize {
        self.toolsets.len()
    }
}

/// Create MCP configurations from McpConfig
pub fn create_mcp_configs_from_config(mcp_config: &McpConfig) -> Vec<McpServerConfig> {
    let mut configs = Vec::new();

    // Tavily MCP Server - always configured if API key is present
    if !mcp_config.tavily_api_key.is_empty() {
        configs.push(McpServerConfig {
            name: "tavily".to_string(),
            endpoint: format!(
                "https://mcp.tavily.com/mcp/?tavilyApiKey={}",
                mcp_config.tavily_api_key
            ),
            timeout_secs: Some(60),
        });
        tracing::info!("Tavily MCP server configured");
    }

    // DeepWiki MCP Server - always available if enabled
    if mcp_config.deepwiki_enabled {
        configs.push(McpServerConfig {
            name: "deepwiki".to_string(),
            endpoint: "https://mcp.deepwiki.com/mcp".to_string(),
            timeout_secs: Some(60),
        });
        tracing::info!("DeepWiki MCP server configured");
    }

    configs
}

/// Create MCP toolsets from configuration
pub async fn create_mcp_toolsets_from_config(
    mcp_config: &McpConfig,
) -> Result<Vec<ConnectedMcpToolset>> {
    let configs = create_mcp_configs_from_config(mcp_config);

    if configs.is_empty() {
        tracing::debug!("No MCP servers configured in config");
        return Ok(Vec::new());
    }

    let mut manager = McpManager::new();
    for config in configs {
        manager.add_config(config);
    }

    manager.connect_all().await
}
