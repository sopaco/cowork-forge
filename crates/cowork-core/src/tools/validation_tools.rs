// Validation tools - Quality assurance and testing
// NOTE: This module contains V1 legacy tools that are not used in V2 iteration architecture
use adk_core::{Tool, ToolContext, AdkError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

// ============================================================================
// CheckDataFormatTool
// ============================================================================

pub struct CheckDataFormatTool {
    session_id: String,
}

impl CheckDataFormatTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for CheckDataFormatTool {
    fn name(&self) -> &str {
        "check_data_format"
    }

    fn description(&self) -> &str {
        "Validate that a JSON data file conforms to its schema. Returns validation errors if any."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "data_type": {
                    "type": "string",
                    "enum": ["requirements", "features", "design", "plan"],
                    "description": "Which data file to validate"
                }
            },
            "required": ["data_type"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let data_type = args["data_type"].as_str().unwrap();

        let errors = match data_type {
            "requirements" => self.validate_requirements_schema(),
            "features" => self.validate_features_schema(),
            "design" => self.validate_design_schema(),
            "plan" => self.validate_plan_schema(),
            _ => return Ok(json!({"status": "error", "message": "Unknown data type"})),
        };

        if errors.is_empty() {
            Ok(json!({
                "status": "valid",
                "message": format!("{} data is valid", data_type)
            }))
        } else {
            Ok(json!({
                "status": "invalid",
                "errors": errors
            }))
        }
    }
}

impl CheckDataFormatTool {
    fn validate_requirements_schema(&self) -> Vec<String> {
        let mut errors = vec![];
        match load_requirements(&self.session_id) {
            Ok(requirements) => {
                for req in &requirements.requirements {
                    if req.title.is_empty() {
                        errors.push(format!("{}: title is empty", req.id));
                    }
                    if req.acceptance_criteria.is_empty() {
                        errors.push(format!("{}: missing acceptance criteria", req.id));
                    }
                }
            }
            Err(e) => errors.push(format!("Failed to load requirements: {}", e)),
        }
        errors
    }

    fn validate_features_schema(&self) -> Vec<String> {
        let mut errors = vec![];
        match load_feature_list(&self.session_id) {
            Ok(features) => {
                for feat in &features.features {
                    if feat.name.is_empty() {
                        errors.push(format!("{}: name is empty", feat.id));
                    }
                    if feat.requirement_ids.is_empty() {
                        errors.push(format!("{}: not linked to any requirement", feat.id));
                    }
                }
            }
            Err(e) => errors.push(format!("Failed to load features: {}", e)),
        }
        errors
    }

    fn validate_design_schema(&self) -> Vec<String> {
        let mut errors = vec![];
        match load_design_spec(&self.session_id) {
            Ok(design) => {
                if design.architecture.components.is_empty() {
                    errors.push("No components defined".to_string());
                }
            }
            Err(e) => errors.push(format!("Failed to load design: {}", e)),
        }
        errors
    }

    fn validate_plan_schema(&self) -> Vec<String> {
        let mut errors = vec![];
        match load_implementation_plan(&self.session_id) {
            Ok(plan) => {
                if plan.tasks.is_empty() {
                    errors.push("No tasks defined".to_string());
                }
            }
            Err(e) => errors.push(format!("Failed to load plan: {}", e)),
        }
        errors
    }
}

// ============================================================================
// CheckFeatureCoverageTool
// ============================================================================

pub struct CheckFeatureCoverageTool {
    session_id: String,
}

impl CheckFeatureCoverageTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for CheckFeatureCoverageTool {
    fn name(&self) -> &str {
        "check_feature_coverage"
    }

    fn description(&self) -> &str {
        "Check if all features are covered by design components."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({"type": "object", "properties": {}}))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let features = load_feature_list(&self.session_id).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;
        let design = load_design_spec(&self.session_id).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        let uncovered: Vec<String> = features
            .features
            .iter()
            .filter(|f| {
                !design
                    .architecture
                    .components
                    .iter()
                    .any(|c| c.related_features.contains(&f.id))
            })
            .map(|f| f.id.clone())
            .collect();

        if uncovered.is_empty() {
            Ok(json!({
                "status": "full_coverage",
                "message": "All features are covered by components"
            }))
        } else {
            Ok(json!({
                "status": "incomplete_coverage",
                "uncovered_features": uncovered,
                "message": format!("{} features are not covered", uncovered.len())
            }))
        }
    }
}

// ============================================================================
// CheckTaskDependenciesTool
// ============================================================================

pub struct CheckTaskDependenciesTool {
    session_id: String,
}

impl CheckTaskDependenciesTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for CheckTaskDependenciesTool {
    fn name(&self) -> &str {
        "check_task_dependencies"
    }

    fn description(&self) -> &str {
        "Analyze task dependencies to detect circular dependencies."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({"type": "object", "properties": {}}))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let plan = load_implementation_plan(&self.session_id).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        // Build dependency graph
        let mut graph: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for task in &plan.tasks {
            graph.insert(task.id.clone(), task.dependencies.clone());
        }

        // Detect cycles using DFS
        let has_cycles = detect_cycle(&graph);

        if has_cycles {
            Ok(json!({
                "status": "invalid",
                "message": "Circular dependencies detected in task graph"
            }))
        } else {
            Ok(json!({
                "status": "valid",
                "message": "No circular dependencies detected"
            }))
        }
    }
}

/// Detect cycles in dependency graph using DFS
fn detect_cycle(graph: &std::collections::HashMap<String, Vec<String>>) -> bool {
    use std::collections::HashSet;

    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    fn dfs(
        node: &str,
        graph: &std::collections::HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if dfs(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true; // Cycle detected
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    for node in graph.keys() {
        if !visited.contains(node) {
            if dfs(node, graph, &mut visited, &mut rec_stack) {
                return true;
            }
        }
    }

    false
}
