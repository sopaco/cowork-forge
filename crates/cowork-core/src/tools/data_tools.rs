// Data operation tools - Create and modify structured data
use crate::data::*;
use crate::storage::*;
use adk_core::{Tool, ToolContext, AdkError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use super::get_required_string_param;
use super::get_required_array_param;

// ============================================================================
// CreateRequirementTool
// ============================================================================

pub struct CreateRequirementTool;

#[async_trait]
impl Tool for CreateRequirementTool {
    fn name(&self) -> &str {
        "create_requirement"
    }

    fn description(&self) -> &str {
        "Create a new requirement in requirements.json. Requirements define what \
         the system must do. Each requirement should be SMART (Specific, Measurable, \
         Achievable, Relevant, Time-bound) with clear acceptance criteria."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Brief requirement title"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed description of the requirement"
                },
                "priority": {
                    "type": "string",
                    "enum": ["high", "medium", "low"],
                    "description": "Priority level"
                },
                "category": {
                    "type": "string",
                    "enum": ["functional", "non_functional"],
                    "description": "Requirement category"
                },
                "acceptance_criteria": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of acceptance criteria"
                }
            },
            "required": ["title", "description", "priority", "category", "acceptance_criteria"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // Notify tool call
        let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("");
        super::notify_tool_call("create_requirement", &json!({"title": title}));

        let mut reqs = load_requirements().map_err(|e| AdkError::Tool(e.to_string()))?;

        let req_id = generate_id("REQ", reqs.requirements.len());

        let priority = match get_required_string_param(&args, "priority")? {
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium,
        };

        let category = match get_required_string_param(&args, "category")? {
            "functional" => RequirementCategory::Functional,
            "non_functional" => RequirementCategory::NonFunctional,
            _ => RequirementCategory::Functional,
        };

        let requirement = Requirement {
            id: req_id.clone(),
            title: get_required_string_param(&args, "title")?.to_string(),
            description: get_required_string_param(&args, "description")?.to_string(),
            priority,
            category,
            acceptance_criteria: get_required_array_param(&args, "acceptance_criteria")?
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            related_features: vec![],
        };

        reqs.requirements.push(requirement.clone());
        reqs.updated_at = chrono::Utc::now();
        save_requirements(&reqs).map_err(|e| AdkError::Tool(e.to_string()))?;

        // Log for user visibility
        println!("✅ Created: {} - {}", req_id, requirement.title);

        // Notify tool result
        super::notify_tool_result("create_requirement", &Ok(json!({"status": "success", "requirement_id": req_id})));

        Ok(json!({
            "status": "success",
            "requirement_id": req_id,
            "message": format!("Requirement {} created successfully", req_id)
        }))
    }
}

// ============================================================================
// AddFeatureTool
// ============================================================================

pub struct AddFeatureTool;

#[async_trait]
impl Tool for AddFeatureTool {
    fn name(&self) -> &str {
        "add_feature"
    }

    fn description(&self) -> &str {
        "Add a new feature to feature_list.json. Features are concrete \
         functionalities that implement one or more requirements. Each \
         feature will later be broken down into implementation tasks."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Feature name"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed description"
                },
                "requirement_ids": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "IDs of requirements this feature implements"
                },
                "completion_criteria": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Criteria for feature completion"
                }
            },
            "required": ["name", "description", "requirement_ids", "completion_criteria"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let name = get_required_string_param(&args, "name")?;
        super::notify_tool_call("add_feature", &json!({"name": name}));

        let mut features = load_feature_list().map_err(|e| AdkError::Tool(e.to_string()))?;

        let feat_id = generate_id("FEAT", features.features.len());

        let feature = Feature {
            id: feat_id.clone(),
            name: get_required_string_param(&args, "name")?.to_string(),
            description: get_required_string_param(&args, "description")?.to_string(),
            requirement_ids: get_required_array_param(&args, "requirement_ids")?
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            status: FeatureStatus::Pending,
            assigned_to_tasks: vec![],
            completion_criteria: get_required_array_param(&args, "completion_criteria")?
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            created_at: chrono::Utc::now(),
            completed_at: None,
            metadata: FeatureMetadata::default(),
        };

        features.features.push(feature);
        save_feature_list(&features).map_err(|e| AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "feature_id": feat_id,
            "message": format!("Feature {} created successfully", feat_id)
        }))
    }
}

// ============================================================================
// CreateDesignComponentTool
// ============================================================================

pub struct CreateDesignComponentTool;

#[async_trait]
impl Tool for CreateDesignComponentTool {
    fn name(&self) -> &str {
        "create_design_component"
    }

    fn description(&self) -> &str {
        "Create a new component in design_spec.json. Components are the \
         architectural building blocks (services, modules, UI components) \
         that implement features."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Component name"
                },
                "component_type": {
                    "type": "string",
                    "enum": ["backend_service", "frontend_component", "database", "api_gateway"],
                    "description": "Type of component"
                },
                "responsibilities": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of responsibilities"
                },
                "technology": {
                    "type": "string",
                    "description": "Technology stack"
                },
                "related_features": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Related feature IDs"
                }
            },
            "required": ["name", "component_type", "responsibilities", "technology"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
        super::notify_tool_call("create_design_component", &json!({"name": name}));

        let mut design = load_design_spec().map_err(|e| AdkError::Tool(e.to_string()))?;

        let comp_id = generate_id("COMP", design.architecture.components.len());

        // Parse component_type with error handling
        let component_type = args.get("component_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'component_type' parameter".to_string()))?;
        
        let component_type = match component_type {
            "backend_service" => ComponentType::BackendService,
            "frontend_component" => ComponentType::FrontendComponent,
            "database" => ComponentType::Database,
            "api_gateway" => ComponentType::ApiGateway,
            other => ComponentType::Other(other.to_string()),
        };

        // Parse required fields with error handling
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'name' parameter".to_string()))?
            .to_string();

        let technology = args.get("technology")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'technology' parameter".to_string()))?
            .to_string();

        // Parse responsibilities array with error handling
        let responsibilities = args.get("responsibilities")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'responsibilities' parameter (must be an array)".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<String>>();

        if responsibilities.is_empty() {
            return Err(AdkError::Tool("'responsibilities' array cannot be empty".to_string()));
        }

        // Parse optional related_features
        let related_features = args.get("related_features")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let component = DesignComponent {
            id: comp_id.clone(),
            name,
            component_type,
            responsibilities,
            technology,
            interfaces: vec![],
            related_features,
        };

        design.architecture.components.push(component.clone());
        save_design_spec(&design).map_err(|e| AdkError::Tool(e.to_string()))?;

        // Log for user visibility
        println!("🏗️  Created component: {} - {}", comp_id, component.name);

        Ok(json!({
            "status": "success",
            "component_id": comp_id,
            "message": format!("Component {} created successfully", comp_id)
        }))
    }
}

// ============================================================================
// CreateTaskTool
// ============================================================================

pub struct CreateTaskTool;

#[async_trait]
impl Tool for CreateTaskTool {
    fn name(&self) -> &str {
        "create_task"
    }

    fn description(&self) -> &str {
        "Create an implementation task in implementation_plan.json. Tasks \
         are concrete coding work items that implement features."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "description": {"type": "string"},
                "feature_id": {"type": "string"},
                "component_id": {"type": "string"},
                "files_to_create": {
                    "type": "array",
                    "items": {"type": "string"}
                },
                "dependencies": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Task IDs that must be completed first"
                },
                "acceptance_criteria": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            },
            "required": ["title", "description", "feature_id", "component_id", "acceptance_criteria"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let title = get_required_string_param(&args, "title")?;
        super::notify_tool_call("create_task", &json!({"title": title}));

        let mut plan = load_implementation_plan().map_err(|e| AdkError::Tool(e.to_string()))?;

        let task_id = generate_id("TASK", plan.tasks.len());

        let task = Task {
            id: task_id.clone(),
            title: get_required_string_param(&args, "title")?.to_string(),
            description: get_required_string_param(&args, "description")?.to_string(),
            feature_id: get_required_string_param(&args, "feature_id")?.to_string(),
            component_id: get_required_string_param(&args, "component_id")?.to_string(),
            status: TaskStatus::Pending,
            dependencies: args.get("dependencies")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().map(|v| v.as_str().unwrap().to_string()).collect())
                .unwrap_or_default(),
            estimated_effort: None,
            files_to_create: args.get("files_to_create")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().map(|v| v.as_str().unwrap().to_string()).collect())
                .unwrap_or_default(),
            acceptance_criteria: get_required_array_param(&args, "acceptance_criteria")?
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        plan.tasks.push(task);
        save_implementation_plan(&plan).map_err(|e| AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "task_id": task_id,
            "message": format!("Task {} created successfully", task_id)
        }))
    }
}

// ============================================================================
// Update Status Tools
// ============================================================================

pub struct UpdateFeatureStatusTool;

#[async_trait]
impl Tool for UpdateFeatureStatusTool {
    fn name(&self) -> &str {
        "update_feature_status"
    }

    fn description(&self) -> &str {
        "Update the status of a feature. Valid transitions: \
         pending → in_progress → completed."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "feature_id": {"type": "string"},
                "new_status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "blocked"]
                }
            },
            "required": ["feature_id", "new_status"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let feature_id = get_required_string_param(&args, "feature_id")?;
        let new_status_str = get_required_string_param(&args, "new_status")?;
        super::notify_tool_call("update_feature_status", &json!({"feature_id": feature_id, "status": new_status_str}));

        let mut features = load_feature_list().map_err(|e| AdkError::Tool(e.to_string()))?;

        let new_status = match new_status_str {
            "pending" => FeatureStatus::Pending,
            "in_progress" => FeatureStatus::InProgress,
            "completed" => FeatureStatus::Completed,
            "blocked" => FeatureStatus::Blocked,
            _ => FeatureStatus::Pending,
        };

        if let Some(feature) = features.features.iter_mut().find(|f| f.id == feature_id) {
            feature.status = new_status;
            if new_status_str == "completed" {
                feature.completed_at = Some(chrono::Utc::now());
            }
            save_feature_list(&features).map_err(|e| AdkError::Tool(e.to_string()))?;

            Ok(json!({
                "status": "success",
                "feature_id": feature_id,
                "new_status": new_status_str,
                "message": format!("Feature {} status updated to {}", feature_id, new_status_str)
            }))
        } else {
            Ok(json!({
                "status": "error",
                "message": format!("Feature {} not found", feature_id)
            }))
        }
    }
}

pub struct UpdateTaskStatusTool;

#[async_trait]
impl Tool for UpdateTaskStatusTool {
    fn name(&self) -> &str {
        "update_task_status"
    }

    fn description(&self) -> &str {
        "Update task status. Call this as you start and complete tasks."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task_id": {"type": "string"},
                "new_status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "blocked"]
                }
            },
            "required": ["task_id", "new_status"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // Parse parameters with error handling
        let task_id = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'task_id' parameter".to_string()))?;
        let new_status_str = args.get("new_status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'new_status' parameter".to_string()))?;
        super::notify_tool_call("update_task_status", &json!({"task_id": task_id, "status": new_status_str}));

        let mut plan = load_implementation_plan().map_err(|e| AdkError::Tool(e.to_string()))?;

        let new_status = match new_status_str {
            "pending" => TaskStatus::Pending,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            "blocked" => TaskStatus::Blocked,
            _ => return Err(AdkError::Tool(format!("Invalid status: {}. Must be one of: pending, in_progress, completed, blocked", new_status_str))),
        };

        if let Some(task) = plan.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = new_status;
            match new_status_str {
                "in_progress" => task.started_at = Some(chrono::Utc::now()),
                "completed" => task.completed_at = Some(chrono::Utc::now()),
                _ => {}
            }
            save_implementation_plan(&plan).map_err(|e| AdkError::Tool(e.to_string()))?;

            // Log for user visibility
            println!("✓ Task {} → {}", task_id, new_status_str);

            Ok(json!({
                "status": "success",
                "task_id": task_id,
                "new_status": new_status_str
            }))
        } else {
            Ok(json!({
                "status": "error",
                "message": format!("Task {} not found", task_id)
            }))
        }
    }
}

// ============================================================================
// Get/Read Tools
// ============================================================================

pub struct GetRequirementsTool;

#[async_trait]
impl Tool for GetRequirementsTool {
    fn name(&self) -> &str {
        "get_requirements"
    }

    fn description(&self) -> &str {
        "Retrieve all requirements and features."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {}
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let requirements = load_requirements().map_err(|e| AdkError::Tool(e.to_string()))?;
        let features = load_feature_list().map_err(|e| AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "requirements": requirements.requirements,
            "features": features.features
        }))
    }
}

pub struct GetDesignTool;

#[async_trait]
impl Tool for GetDesignTool {
    fn name(&self) -> &str {
        "get_design"
    }

    fn description(&self) -> &str {
        "Retrieve the design specification."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({"type": "object", "properties": {}}))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let design = load_design_spec().map_err(|e| AdkError::Tool(e.to_string()))?;
        Ok(serde_json::to_value(design).map_err(|e| AdkError::Tool(e.to_string()))?)
    }
}

pub struct GetPlanTool;

#[async_trait]
impl Tool for GetPlanTool {
    fn name(&self) -> &str {
        "get_plan"
    }

    fn description(&self) -> &str {
        "Retrieve the implementation plan with all tasks."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "status_filter": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed"],
                    "description": "Optional: only return tasks with this status"
                }
            }
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let plan = load_implementation_plan().map_err(|e| AdkError::Tool(e.to_string()))?;

        if let Some(status_filter) = args.get("status_filter").and_then(|v| v.as_str()) {
            let status = match status_filter {
                "pending" => TaskStatus::Pending,
                "in_progress" => TaskStatus::InProgress,
                "completed" => TaskStatus::Completed,
                _ => TaskStatus::Pending,
            };

            let filtered_tasks: Vec<&Task> = plan.tasks.iter()
                .filter(|t| t.status == status)
                .collect();

            Ok(json!({
                "tasks": filtered_tasks,
                "milestones": plan.milestones
            }))
        } else {
            Ok(serde_json::to_value(plan).map_err(|e| AdkError::Tool(e.to_string()))?)
        }
    }
}

// ============================================================================
// Incremental Update Tools for GotoStage Support
// ============================================================================

/// Update an existing requirement without recreating the entire list
/// Used when GotoStage restarts a stage with feedback
pub struct UpdateRequirementTool;

#[async_trait]
impl Tool for UpdateRequirementTool {
    fn name(&self) -> &str {
        "update_requirement"
    }

    fn description(&self) -> &str {
        "Update an existing requirement. Use this when restarting from a stage with feedback \
         to modify specific requirements without recreating the entire list."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Requirement ID to update (e.g., REQ-001)"
                },
                "title": {
                    "type": "string",
                    "description": "New title for the requirement"
                },
                "description": {
                    "type": "string",
                    "description": "New description for the requirement"
                },
                "priority": {
                    "type": "string",
                    "enum": ["high", "medium", "low"],
                    "description": "New priority level"
                },
                "acceptance_criteria": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "New list of acceptance criteria"
                }
            },
            "required": ["id"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let mut requirements = load_requirements().map_err(|e| AdkError::Tool(e.to_string()))?;

        let id = args.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing 'id' parameter".to_string()))?;

        if let Some(req) = requirements.requirements.iter_mut().find(|r| r.id == id) {
            // Update fields if provided
            if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
                req.title = title.to_string();
            }
            if let Some(description) = args.get("description").and_then(|v| v.as_str()) {
                req.description = description.to_string();
            }
            if let Some(priority_str) = args.get("priority").and_then(|v| v.as_str()) {
                req.priority = match priority_str {
                    "high" => Priority::High,
                    "medium" => Priority::Medium,
                    "low" => Priority::Low,
                    _ => req.priority,
                };
            }
            if let Some(criteria) = args.get("acceptance_criteria").and_then(|v| v.as_array()) {
                req.acceptance_criteria = criteria.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }

            save_requirements(&requirements).map_err(|e| AdkError::Tool(e.to_string()))?;

            println!("✅ Updated requirement: {}", id);

            Ok(json!({
                "status": "success",
                "requirement_id": id,
                "message": format!("Requirement {} updated successfully", id)
            }))
        } else {
            Ok(json!({
                "status": "error",
                "message": format!("Requirement {} not found", id)
            }))
        }
    }
}

/// Delete an existing requirement
pub struct DeleteRequirementTool;

#[async_trait]
impl Tool for DeleteRequirementTool {
    fn name(&self) -> &str {
        "delete_requirement"
    }

    fn description(&self) -> &str {
        "Delete a requirement by ID. Use this when feedback indicates a requirement is no longer needed."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Requirement ID to delete (e.g., REQ-001)"
                }
            },
            "required": ["id"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let mut requirements = load_requirements().map_err(|e| AdkError::Tool(e.to_string()))?;

        let id = args.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing 'id' parameter".to_string()))?;

        let original_len = requirements.requirements.len();
        requirements.requirements.retain(|r| r.id != id);

        if requirements.requirements.len() < original_len {
            save_requirements(&requirements).map_err(|e| AdkError::Tool(e.to_string()))?;

            println!("🗑️  Deleted requirement: {}", id);

            Ok(json!({
                "status": "success",
                "requirement_id": id,
                "message": format!("Requirement {} deleted successfully", id)
            }))
        } else {
            Ok(json!({
                "status": "error",
                "message": format!("Requirement {} not found", id)
            }))
        }
    }
}

/// Update an existing feature without recreating the entire list
pub struct UpdateFeatureTool;

#[async_trait]
impl Tool for UpdateFeatureTool {
    fn name(&self) -> &str {
        "update_feature"
    }

    fn description(&self) -> &str {
        "Update an existing feature. Use this when restarting from a stage with feedback \
         to modify specific features without recreating the entire list."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Feature ID to update (e.g., FEAT-001)"
                },
                "name": {
                    "type": "string",
                    "description": "New name for the feature"
                },
                "description": {
                    "type": "string",
                    "description": "New description for the feature"
                },
                "requirement_ids": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "New list of requirement IDs this feature implements"
                },
                "completion_criteria": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "New list of completion criteria"
                }
            },
            "required": ["id"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let mut features = load_feature_list().map_err(|e| AdkError::Tool(e.to_string()))?;

        let id = args.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing 'id' parameter".to_string()))?;

        if let Some(feature) = features.features.iter_mut().find(|f| f.id == id) {
            // Update fields if provided
            if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
                feature.name = name.to_string();
            }
            if let Some(description) = args.get("description").and_then(|v| v.as_str()) {
                feature.description = description.to_string();
            }
            if let Some(req_ids) = args.get("requirement_ids").and_then(|v| v.as_array()) {
                feature.requirement_ids = req_ids.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(criteria) = args.get("completion_criteria").and_then(|v| v.as_array()) {
                feature.completion_criteria = criteria.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }

            save_feature_list(&features).map_err(|e| AdkError::Tool(e.to_string()))?;

            println!("✅ Updated feature: {}", id);

            Ok(json!({
                "status": "success",
                "feature_id": id,
                "message": format!("Feature {} updated successfully", id)
            }))
        } else {
            Ok(json!({
                "status": "error",
                "message": format!("Feature {} not found", id)
            }))
        }
    }
}