// Tools module - adk-rust Tool implementations
//
// Includes a tool notification system that broadcasts tool calls to the GUI

use adk_core::AdkError;
use serde_json::Value;
use std::sync::RwLock;

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

// ============================================================================
// Tool Notification System
// ============================================================================

/// Type alias for tool notification callback
type ToolNotifyFn = Box<dyn Fn(&str, &Value, bool, &str) + Send + Sync>;

/// Global tool notification callback storage
static TOOL_NOTIFIER: RwLock<Option<ToolNotifyFn>> = RwLock::new(None);

/// Set the global tool notification callback
/// This should be called once at application startup (GUI backend)
pub fn set_tool_notify_callback<F>(callback: F)
where
    F: Fn(&str, &Value, bool, &str) + Send + Sync + 'static,
{
    let mut guard = TOOL_NOTIFIER.write().unwrap();
    *guard = Some(Box::new(callback));
}

/// Notify about a tool call (call this before tool execution)
pub fn notify_tool_call(tool_name: &str, args: &Value) {
    // Print to console for debugging
    let args_str = if args.is_object() {
        let keys: Vec<&str> = args.as_object().unwrap().keys().map(|s| s.as_str()).collect();
        format!("{:?}", keys)
    } else {
        args.to_string()
    };
    println!("🔧 Tool call: {} {}", tool_name, args_str);

    // Call registered callback if exists
    if let Ok(guard) = TOOL_NOTIFIER.read() {
        if let Some(ref callback) = *guard {
            callback(tool_name, args, true, "");
        }
    }
}

/// Notify about a tool result (call this after tool execution)
pub fn notify_tool_result(tool_name: &str, result: &Result<Value, AdkError>) {
    // Print to console for debugging
    match result {
        Ok(v) => {
            let preview = if v.is_object() {
                let keys: Vec<&str> = v.as_object().unwrap().keys().map(|s| s.as_str()).collect();
                format!("{:?}", keys)
            } else if v.is_string() {
                let s = v.as_str().unwrap_or("");
                if s.len() > 50 {
                    format!("{}...", &s[..50])
                } else {
                    s.to_string()
                }
            } else {
                v.to_string()
            };
            println!("✓ Tool result: {} -> {}", tool_name, preview);
        }
        Err(e) => println!("✗ Tool result: {} - error: {}", tool_name, e),
    }

    // Call registered callback if exists
    if let Ok(guard) = TOOL_NOTIFIER.read() {
        if let Some(ref callback) = *guard {
            let success = result.is_ok();
            let result_str = match result {
                Ok(v) => v.to_string(),
                Err(e) => e.to_string(),
            };
            callback(tool_name, &Value::Null, success, &result_str);
        }
    }
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

// Project Manager Agent tools
pub mod pm_tools;

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
pub use pm_tools::*;