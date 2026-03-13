// Configuration Loader - Load configurations from file system
//
// Supports loading from:
// - System default directory: .cowork-v3/config/
// - User directory: ~/.cowork/config/
// - Project directory: project/.cowork-v3/config/
//
// Note: Skills are managed separately via adk-skill (SKILL.md format)
// See the `skills` module for skill management.

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use walkdir::WalkDir;

use super::agent_definition::AgentDefinition;
use super::stage_definition::StageDefinition;
use super::flow_definition::FlowDefinition;
use super::integration_definition::IntegrationDefinition;
use super::registry::ConfigRegistry;

/// Configuration loader for file system based config loading
pub struct ConfigLoader {
    /// Base directories to search for configurations (in priority order)
    search_paths: Vec<PathBuf>,
}

impl ConfigLoader {
    /// Create a new loader with default search paths
    pub fn new(project_path: Option<&Path>) -> Self {
        let mut search_paths = Vec::new();
        
        // Project-level config (highest priority)
        if let Some(project) = project_path {
            search_paths.push(project.join(".cowork-v3").join("config"));
        }
        
        // User-level config
        if let Some(home) = dirs::home_dir() {
            search_paths.push(home.join(".cowork").join("config"));
        }
        
        // System-level config (built-in defaults)
        if let Some(exe_dir) = std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())) 
        {
            search_paths.push(exe_dir.join("config"));
        }
        
        Self { search_paths }
    }
    
    /// Create a loader with custom search paths
    pub fn with_search_paths(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }
    
    /// Get the search paths
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }
    
    /// Load all configurations into the registry
    pub fn load_all(&self, registry: &ConfigRegistry) -> Result<LoadReport> {
        let mut report = LoadReport::default();
        
        // Load in order: agents, stages, flows, integrations
        // Note: Skills are managed via adk-skill module
        for path in &self.search_paths {
            if !path.exists() {
                continue;
            }
            
            self.load_agents(path, registry, &mut report)?;
            self.load_stages(path, registry, &mut report)?;
            self.load_flows(path, registry, &mut report)?;
            self.load_integrations(path, registry, &mut report)?;
        }
        
        // Set default flow if not set
        if registry.get_default_flow().is_none() {
            if let Some(flow) = registry.get_flow("default") {
                registry.set_default_flow(Some(flow.id))?;
                report.default_flow_set = true;
            }
        }
        
        Ok(report)
    }
    
    /// Load agent definitions from a directory
    fn load_agents(&self, base: &Path, registry: &ConfigRegistry, report: &mut LoadReport) -> Result<()> {
        let agents_dir = base.join("agents");
        if !agents_dir.exists() {
            return Ok(());
        }
        
        for entry in WalkDir::new(&agents_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        {
            let path = entry.path();
            match self.load_agent_file(path) {
                Ok(agent) => {
                    let id = agent.id.clone();
                    registry.register_agent(agent)?;
                    report.agents_loaded += 1;
                    tracing::debug!("Loaded agent: {} from {:?}", id, path);
                }
                Err(e) => {
                    report.errors.push(format!("Failed to load agent from {:?}: {}", path, e));
                    tracing::warn!("Failed to load agent from {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Load a single agent file
    fn load_agent_file(&self, path: &Path) -> Result<AgentDefinition> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;
        
        let agent: AgentDefinition = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse agent definition: {:?}", path))?;
        
        Ok(agent)
    }
    
    /// Load stage definitions from a directory
    fn load_stages(&self, base: &Path, registry: &ConfigRegistry, report: &mut LoadReport) -> Result<()> {
        let stages_dir = base.join("stages");
        if !stages_dir.exists() {
            return Ok(());
        }
        
        for entry in WalkDir::new(&stages_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        {
            let path = entry.path();
            match self.load_stage_file(path) {
                Ok(stage) => {
                    let id = stage.id.clone();
                    registry.register_stage(stage)?;
                    report.stages_loaded += 1;
                    tracing::debug!("Loaded stage: {} from {:?}", id, path);
                }
                Err(e) => {
                    report.errors.push(format!("Failed to load stage from {:?}: {}", path, e));
                    tracing::warn!("Failed to load stage from {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Load a single stage file
    fn load_stage_file(&self, path: &Path) -> Result<StageDefinition> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;
        
        let stage: StageDefinition = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse stage definition: {:?}", path))?;
        
        Ok(stage)
    }
    
    /// Load flow definitions from a directory
    fn load_flows(&self, base: &Path, registry: &ConfigRegistry, report: &mut LoadReport) -> Result<()> {
        let flows_dir = base.join("flows");
        if !flows_dir.exists() {
            return Ok(());
        }
        
        for entry in WalkDir::new(&flows_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        {
            let path = entry.path();
            match self.load_flow_file(path) {
                Ok(flow) => {
                    let id = flow.id.clone();
                    registry.register_flow(flow)?;
                    report.flows_loaded += 1;
                    tracing::debug!("Loaded flow: {} from {:?}", id, path);
                }
                Err(e) => {
                    report.errors.push(format!("Failed to load flow from {:?}: {}", path, e));
                    tracing::warn!("Failed to load flow from {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Load a single flow file
    fn load_flow_file(&self, path: &Path) -> Result<FlowDefinition> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;
        
        let flow: FlowDefinition = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse flow definition: {:?}", path))?;
        
        Ok(flow)
    }
    
    /// Load integration definitions from a directory
    fn load_integrations(&self, base: &Path, registry: &ConfigRegistry, report: &mut LoadReport) -> Result<()> {
        let integrations_dir = base.join("integrations");
        if !integrations_dir.exists() {
            return Ok(());
        }
        
        for entry in WalkDir::new(&integrations_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        {
            let path = entry.path();
            match self.load_integration_file(path) {
                Ok(integration) => {
                    let id = integration.id.clone();
                    registry.register_integration(integration)?;
                    report.integrations_loaded += 1;
                    tracing::debug!("Loaded integration: {} from {:?}", id, path);
                }
                Err(e) => {
                    report.errors.push(format!("Failed to load integration from {:?}: {}", path, e));
                    tracing::warn!("Failed to load integration from {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Load a single integration file
    fn load_integration_file(&self, path: &Path) -> Result<IntegrationDefinition> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;
        
        let integration: IntegrationDefinition = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse integration definition: {:?}", path))?;
        
        Ok(integration)
    }
}

/// Report of loading results
#[derive(Debug, Clone, Default)]
pub struct LoadReport {
    pub agents_loaded: usize,
    pub stages_loaded: usize,
    pub flows_loaded: usize,
    pub integrations_loaded: usize,
    pub default_flow_set: bool,
    pub errors: Vec<String>,
}

impl LoadReport {
    pub fn total_loaded(&self) -> usize {
        self.agents_loaded + self.stages_loaded + self.flows_loaded + self.integrations_loaded
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_loader_search_paths() {
        let temp = TempDir::new().unwrap();
        let loader = ConfigLoader::new(Some(temp.path()));
        
        assert!(!loader.search_paths().is_empty());
    }
}