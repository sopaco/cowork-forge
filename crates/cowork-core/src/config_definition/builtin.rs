// Built-in Configuration Provider
//
// Loads default configurations that are embedded in the binary.
// These configurations provide behavior by default.

use anyhow::Result;
use include_dir::{include_dir, Dir};

use super::registry::ConfigRegistry;
use super::loader::LoadReport;

// Embed the default configurations directory
static DEFAULT_CONFIGS: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/config_definition/default_configs");

/// Load built-in default configurations into the registry
pub fn load_builtin_configs(registry: &ConfigRegistry) -> Result<LoadReport> {
    let mut report = LoadReport::default();

    // Load agents
    if let Some(agents_dir) = DEFAULT_CONFIGS.get_dir("agents/built-in") {
        for file in agents_dir.files() {
            let path = file.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match load_agent_from_embedded(file.contents()) {
                    Ok(agent) => {
                        let id = agent.id.clone();
                        registry.register_agent(agent)?;
                        report.agents_loaded += 1;
                        tracing::debug!("Loaded built-in agent: {}", id);
                    }
                    Err(e) => {
                        report.errors.push(format!(
                            "Failed to load built-in agent from {:?}: {}",
                            path, e
                        ));
                    }
                }
            }
        }
    }

    // Load stages
    if let Some(stages_dir) = DEFAULT_CONFIGS.get_dir("stages") {
        for file in stages_dir.files() {
            let path = file.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match load_stage_from_embedded(file.contents()) {
                    Ok(stage) => {
                        let id = stage.id.clone();
                        registry.register_stage(stage)?;
                        report.stages_loaded += 1;
                        tracing::debug!("Loaded built-in stage: {}", id);
                    }
                    Err(e) => {
                        report.errors.push(format!(
                            "Failed to load built-in stage from {:?}: {}",
                            path, e
                        ));
                    }
                }
            }
        }
    }

    // Load flows
    if let Some(flows_dir) = DEFAULT_CONFIGS.get_dir("flows") {
        for file in flows_dir.files() {
            let path = file.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match load_flow_from_embedded(file.contents()) {
                    Ok(flow) => {
                        let id = flow.id.clone();
                        registry.register_flow(flow)?;
                        report.flows_loaded += 1;
                        tracing::debug!("Loaded built-in flow: {}", id);
                    }
                    Err(e) => {
                        report.errors.push(format!(
                            "Failed to load built-in flow from {:?}: {}",
                            path, e
                        ));
                    }
                }
            }
        }
    }

    // Set default flow
    if registry.get_flow("default").is_some() {
        registry.set_default_flow(Some("default".to_string()))?;
        report.default_flow_set = true;
    }

    Ok(report)
}

fn load_agent_from_embedded(contents: &[u8]) -> Result<super::agent_definition::AgentDefinition> {
    let content = std::str::from_utf8(contents)?;
    let agent: super::agent_definition::AgentDefinition = serde_json::from_str(content)?;
    Ok(agent)
}

fn load_stage_from_embedded(contents: &[u8]) -> Result<super::stage_definition::StageDefinition> {
    let content = std::str::from_utf8(contents)?;
    let stage: super::stage_definition::StageDefinition = serde_json::from_str(content)?;
    Ok(stage)
}

fn load_flow_from_embedded(contents: &[u8]) -> Result<super::flow_definition::FlowDefinition> {
    let content = std::str::from_utf8(contents)?;
    let flow: super::flow_definition::FlowDefinition = serde_json::from_str(content)?;
    Ok(flow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_builtin_configs() {
        let registry = ConfigRegistry::new();
        let report = load_builtin_configs(&registry).unwrap();

        println!("Loaded: {} agents, {} stages, {} flows",
            report.agents_loaded, report.stages_loaded, report.flows_loaded);

        assert!(report.agents_loaded > 0, "Should load built-in agents");
        assert!(report.stages_loaded > 0, "Should load built-in stages");
        assert!(report.flows_loaded > 0, "Should load built-in flows");
    }
}
