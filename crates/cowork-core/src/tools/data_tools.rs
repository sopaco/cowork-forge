// Data operation tools - Create and modify structured data (Session-scoped)
use crate::data::*;
use crate::storage::*;
use adk_core::{Tool, ToolContext, AdkError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

// ============================================================================
// CreateRequirementTool
// ============================================================================

pub struct CreateRequirementTool {
    session_id: String,
}

impl CreateRequirementTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let mut reqs = load_requirements(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        let req_id = generate_id("REQ", reqs.requirements.len());

        let priority = match args["priority"].as_str().unwrap() {
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium,
        };

        let category = match args["category"].as_str().unwrap() {
            "functional" => RequirementCategory::Functional,
            "non_functional" => RequirementCategory::NonFunctional,
            _ => RequirementCategory::Functional,
        };

        let requirement = Requirement {
            id: req_id.clone(),
            title: args["title"].as_str().unwrap().to_string(),
            description: args["description"].as_str().unwrap().to_string(),
            priority,
            category,
            acceptance_criteria: args["acceptance_criteria"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            related_features: vec![],
        };

        reqs.requirements.push(requirement.clone());
        reqs.updated_at = chrono::Utc::now();
        save_requirements(&self.session_id, &reqs).map_err(|e| AdkError::Tool(e.to_string()))?;

        println!("✅ Created: {} - {}", req_id, requirement.title);

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

pub struct AddFeatureTool {
    session_id: String,
}

impl AddFeatureTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let mut features = load_feature_list(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        let feat_id = generate_id("FEAT", features.features.len());

        let feature = Feature {
            id: feat_id.clone(),
            name: args["name"].as_str().unwrap().to_string(),
            description: args["description"].as_str().unwrap().to_string(),
            requirement_ids: args["requirement_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            status: FeatureStatus::Pending,
            assigned_to_tasks: vec![],
            completion_criteria: args["completion_criteria"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            created_at: chrono::Utc::now(),
            completed_at: None,
            metadata: FeatureMetadata::default(),
        };

        features.features.push(feature);
        save_feature_list(&self.session_id, &features).map_err(|e| AdkError::Tool(e.to_string()))?;

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

pub struct CreateDesignComponentTool {
    session_id: String,
}

impl CreateDesignComponentTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let mut design = load_design_spec(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        let comp_id = generate_id("COMP", design.architecture.components.len());

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

        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'name' parameter".to_string()))?
            .to_string();

        let technology = args.get("technology")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'technology' parameter".to_string()))?
            .to_string();

        let responsibilities = args.get("responsibilities")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'responsibilities' parameter (must be an array)".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<String>>();

        if responsibilities.is_empty() {
            return Err(AdkError::Tool("'responsibilities' array cannot be empty".to_string()));
        }

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
        save_design_spec(&self.session_id, &design).map_err(|e| AdkError::Tool(e.to_string()))?;

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

pub struct CreateTaskTool {
    session_id: String,
}

impl CreateTaskTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let mut plan = load_implementation_plan(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        let task_id = generate_id("TASK", plan.tasks.len());

        let task = Task {
            id: task_id.clone(),
            title: args["title"].as_str().unwrap().to_string(),
            description: args["description"].as_str().unwrap().to_string(),
            feature_id: args["feature_id"].as_str().unwrap().to_string(),
            component_id: args["component_id"].as_str().unwrap().to_string(),
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
            acceptance_criteria: args["acceptance_criteria"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        plan.tasks.push(task);
        save_implementation_plan(&self.session_id, &plan).map_err(|e| AdkError::Tool(e.to_string()))?;

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

pub struct UpdateFeatureStatusTool {
    session_id: String,
}

impl UpdateFeatureStatusTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let mut features = load_feature_list(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        let feature_id = args["feature_id"].as_str().unwrap();
        let new_status_str = args["new_status"].as_str().unwrap();

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
            save_feature_list(&self.session_id, &features).map_err(|e| AdkError::Tool(e.to_string()))?;

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

pub struct UpdateTaskStatusTool {
    session_id: String,
}

impl UpdateTaskStatusTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let mut plan = load_implementation_plan(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        let task_id = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'task_id' parameter".to_string()))?;
        
        let new_status_str = args.get("new_status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing or invalid 'new_status' parameter".to_string()))?;

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
            save_implementation_plan(&self.session_id, &plan).map_err(|e| AdkError::Tool(e.to_string()))?;

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
// UpdateTaskTool - Modify task properties
// ============================================================================

pub struct UpdateTaskTool {
    session_id: String,
}

impl UpdateTaskTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for UpdateTaskTool {
    fn name(&self) -> &str {
        "update_task"
    }

    fn description(&self) -> &str {
        "Update task properties such as title, description, dependencies, or files. \
         Use this when you discover that task requirements have changed during implementation."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "ID of the task to update"
                },
                "title": {
                    "type": "string",
                    "description": "New title (optional)"
                },
                "description": {
                    "type": "string",
                    "description": "New description (optional)"
                },
                "dependencies": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "New list of task IDs that must be completed first (optional)"
                },
                "files_to_create": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "New list of files to create (optional)"
                },
                "acceptance_criteria": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "New acceptance criteria (optional)"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for this update"
                }
            },
            "required": ["task_id", "reason"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let task_id = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing 'task_id' parameter".to_string()))?;
        
        let reason = args.get("reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing 'reason' parameter".to_string()))?;

        let mut plan = load_implementation_plan(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        // First, find the task index
        let task_idx = plan.tasks.iter()
            .position(|t| t.id == task_id)
            .ok_or_else(|| AdkError::Tool(format!("Task {} not found", task_id)))?;

        let mut updates = Vec::new();

        // Update title if provided
        if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
            plan.tasks[task_idx].title = title.to_string();
            updates.push(format!("title → {}", title));
        }

        // Update description if provided
        if let Some(desc) = args.get("description").and_then(|v| v.as_str()) {
            plan.tasks[task_idx].description = desc.to_string();
            updates.push("description updated".to_string());
        }

        // Update dependencies if provided
        if let Some(deps) = args.get("dependencies").and_then(|v| v.as_array()) {
            let new_deps: Vec<String> = deps.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            
            // Validate that all dependency task IDs exist
            for dep_id in &new_deps {
                if !plan.tasks.iter().any(|t| &t.id == dep_id) {
                    return Err(AdkError::Tool(format!("Dependency task {} not found", dep_id)));
                }
            }
            
            plan.tasks[task_idx].dependencies = new_deps.clone();
            updates.push(format!("dependencies → {:?}", new_deps));
        }

        // Update files_to_create if provided
        if let Some(files) = args.get("files_to_create").and_then(|v| v.as_array()) {
            plan.tasks[task_idx].files_to_create = files.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            updates.push("files_to_create updated".to_string());
        }

        // Update acceptance_criteria if provided
        if let Some(criteria) = args.get("acceptance_criteria").and_then(|v| v.as_array()) {
            plan.tasks[task_idx].acceptance_criteria = criteria.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            updates.push("acceptance_criteria updated".to_string());
        }

        save_implementation_plan(&self.session_id, &plan).map_err(|e| AdkError::Tool(e.to_string()))?;

        // Log the update with reason
        println!("✓ Task {} updated: {}", task_id, updates.join(", "));
        println!("  Reason: {}", reason);

        // Record to feedback for audit trail
        let feedback = crate::data::Feedback {
            feedback_type: crate::data::FeedbackType::Suggestion,
            severity: crate::data::Severity::Minor,
            details: format!("Task {} updated: {}. Reason: {}", task_id, updates.join(", "), reason),
            suggested_fix: None,
            timestamp: chrono::Utc::now(),
        };
        let _ = crate::storage::append_feedback(&self.session_id, &feedback);

        Ok(json!({
            "status": "success",
            "task_id": task_id,
            "updates": updates,
            "reason": reason
        }))
    }
}

// ============================================================================
// DeleteTaskTool - Remove task and clean up dependencies
// ============================================================================

pub struct DeleteTaskTool {
    session_id: String,
}

impl DeleteTaskTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for DeleteTaskTool {
    fn name(&self) -> &str {
        "delete_task"
    }

    fn description(&self) -> &str {
        "Delete a task from the plan. This will also remove references to this task \
         from other tasks' dependencies. Use this when a task is no longer needed \
         or was incorrectly planned."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "ID of the task to delete"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for deleting this task"
                }
            },
            "required": ["task_id", "reason"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let task_id = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing 'task_id' parameter".to_string()))?;
        
        let reason = args.get("reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdkError::Tool("Missing 'reason' parameter".to_string()))?;

        let mut plan = load_implementation_plan(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        // Check if task exists
        let task_exists = plan.tasks.iter().any(|t| t.id == task_id);
        if !task_exists {
            return Err(AdkError::Tool(format!("Task {} not found", task_id)));
        }

        // Check if this task is in_progress or completed
        if let Some(task) = plan.tasks.iter().find(|t| t.id == task_id) {
            if task.status == crate::data::TaskStatus::InProgress {
                return Err(AdkError::Tool(format!(
                    "Cannot delete task {} because it's currently in progress. \
                     Set status to pending or blocked first.", task_id
                )));
            }
            if task.status == crate::data::TaskStatus::Completed {
                return Err(AdkError::Tool(format!(
                    "Cannot delete task {} because it's already completed. \
                     Consider keeping it for documentation.", task_id
                )));
            }
        }

        // Remove the task
        plan.tasks.retain(|t| t.id != task_id);

        // Clean up dependencies - remove this task_id from other tasks' dependencies
        let mut affected_tasks = Vec::new();
        for task in &mut plan.tasks {
            let before_len = task.dependencies.len();
            task.dependencies.retain(|dep| dep != task_id);
            let after_len = task.dependencies.len();
            
            if before_len != after_len {
                affected_tasks.push(task.id.clone());
            }
        }

        save_implementation_plan(&self.session_id, &plan).map_err(|e| AdkError::Tool(e.to_string()))?;

        println!("✓ Task {} deleted", task_id);
        if !affected_tasks.is_empty() {
            println!("  Cleaned dependencies from: {}", affected_tasks.join(", "));
        }
        println!("  Reason: {}", reason);

        // Record to feedback for audit trail
        let feedback = crate::data::Feedback {
            feedback_type: crate::data::FeedbackType::Suggestion,
            severity: crate::data::Severity::Minor,
            details: format!(
                "Task {} deleted. Reason: {}. Affected tasks: {}", 
                task_id, reason, 
                if affected_tasks.is_empty() { "none".to_string() } else { affected_tasks.join(", ") }
            ),
            suggested_fix: None,
            timestamp: chrono::Utc::now(),
        };
        let _ = crate::storage::append_feedback(&self.session_id, &feedback);

        Ok(json!({
            "status": "success",
            "task_id": task_id,
            "affected_tasks": affected_tasks,
            "reason": reason,
            "message": format!("Task {} deleted successfully", task_id)
        }))
    }
}

// ============================================================================
// Get/Read Tools
// ============================================================================

pub struct GetRequirementsTool {
    session_id: String,
}

impl GetRequirementsTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let requirements = load_requirements(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;
        let features = load_feature_list(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "requirements": requirements.requirements,
            "features": features.features
        }))
    }
}

pub struct GetDesignTool {
    session_id: String,
}

impl GetDesignTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

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
        let design = load_design_spec(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;
        Ok(serde_json::to_value(design).map_err(|e| AdkError::Tool(e.to_string()))?)
    }
}

pub struct GetPlanTool {
    session_id: String,
}

impl GetPlanTool {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[async_trait]
impl Tool for GetPlanTool {
    fn name(&self) -> &str {
        "get_plan"
    }

    fn description(&self) -> &str {
        "Retrieve the implementation plan with all tasks."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({"type": "object", "properties": {}}))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        let plan = load_implementation_plan(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;
        Ok(serde_json::to_value(plan).map_err(|e| AdkError::Tool(e.to_string()))?)
    }
}
