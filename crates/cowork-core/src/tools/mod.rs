// Tools module - adk-rust Tool implementations

use adk_core::AdkError;
use serde_json::Value;

// Helper functions for safe parameter extraction
/// Safely get a required string parameter from args
pub fn get_required_string_param<'a>(args: &'a Value, key: &str) -> Result<&'a str, AdkError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| AdkError::Tool(format!("Missing required parameter: {}", key)))
}

/// Safely get an optional string parameter from args
pub fn get_optional_string_param(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Safely get a required array parameter from args
pub fn get_required_array_param<'a>(args: &'a Value, key: &str) -> Result<&'a Vec<Value>, AdkError> {
    args.get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| AdkError::Tool(format!("Missing required parameter: {}", key)))
}

// Core tools
pub mod file_tools;
pub mod hitl_tools;
pub mod hitl_content_tools;
pub mod test_lint_tools;

// Data operation tools
pub mod data_tools;

// Validation tools
pub mod validation_tools;

// Control tools
pub mod control_tools;

// Artifact tools
pub mod artifact_tools;

// Load artifact tools
pub mod load_artifacts;

// Deployment tools
pub mod deployment_tools;

// Control tools
pub mod goto_stage_tool;
pub mod memory_tools;

// Knowledge generation tools
pub mod knowledge_tools;

// Re-exports
pub use file_tools::*;
pub use hitl_tools::*;
pub use hitl_content_tools::*;
pub use test_lint_tools::*;
pub use test_lint_tools::ExecuteShellCommandTool;
pub use data_tools::*;
pub use validation_tools::*;
pub use control_tools::*;
pub use artifact_tools::*;
pub use load_artifacts::*;
pub use deployment_tools::*;
pub use goto_stage_tool::*;
pub use memory_tools::*;
pub use knowledge_tools::*;