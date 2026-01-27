// Test and Lint tools
use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::path::Path;

// ============================================================================
// CheckTestsTool
// ============================================================================

pub struct CheckTestsTool;

#[async_trait]
impl Tool for CheckTestsTool {
    fn name(&self) -> &str {
        "check_tests"
    }

    fn description(&self) -> &str {
        "Run project tests and return results. Automatically detects project type \
         (Rust, Node.js, Python, etc.) and runs appropriate test command."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project directory path (default: current directory)"
                },
                "test_command": {
                    "type": "string",
                    "description": "Optional: Override auto-detected test command (e.g., 'cargo test', 'npm test')"
                }
            }
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        // Auto-detect test command if not provided
        let test_command = if let Some(cmd) = args.get("test_command").and_then(|v| v.as_str()) {
            cmd.to_string()
        } else {
            detect_test_command(path)?
        };

        // Execute test command
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&test_command)
            .current_dir(path)
            .output()
            .await
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to run tests: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        // Parse test results
        let (passed, failed, total) = parse_test_output(&stdout, &stderr);

        Ok(json!({
            "status": if success { "passed" } else { "failed" },
            "command": test_command,
            "exit_code": output.status.code(),
            "tests_passed": passed,
            "tests_failed": failed,
            "tests_total": total,
            "stdout": stdout,
            "stderr": stderr
        }))
    }
}

// ============================================================================
// CheckLintTool
// ============================================================================

pub struct CheckLintTool;

#[async_trait]
impl Tool for CheckLintTool {
    fn name(&self) -> &str {
        "check_lint"
    }

    fn description(&self) -> &str {
        "Run linter/code quality checks and return results. Automatically detects \
         project type and runs appropriate linter (clippy for Rust, eslint for Node.js, etc.)."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project directory path (default: current directory)"
                },
                "lint_command": {
                    "type": "string",
                    "description": "Optional: Override auto-detected lint command (e.g., 'cargo clippy', 'npm run lint')"
                },
                "fix": {
                    "type": "boolean",
                    "description": "Whether to auto-fix issues if supported (default: false)"
                }
            }
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let fix = args.get("fix")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Auto-detect lint command if not provided
        let lint_command = if let Some(cmd) = args.get("lint_command").and_then(|v| v.as_str()) {
            cmd.to_string()
        } else {
            detect_lint_command(path, fix)?
        };

        // Execute lint command
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&lint_command)
            .current_dir(path)
            .output()
            .await
            .map_err(|e| adk_core::AdkError::Tool(format!("Failed to run linter: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        // Parse lint results
        let (warnings, errors) = parse_lint_output(&stdout, &stderr);

        Ok(json!({
            "status": if success { "clean" } else { "issues_found" },
            "command": lint_command,
            "exit_code": output.status.code(),
            "warnings": warnings,
            "errors": errors,
            "total_issues": warnings + errors,
            "stdout": stdout,
            "stderr": stderr
        }))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Detect appropriate test command based on project type
fn detect_test_command(path: &str) -> adk_core::Result<String> {
    let path_buf = Path::new(path);

    // Check for Cargo.toml (Rust)
    if path_buf.join("Cargo.toml").exists() {
        return Ok("cargo test".to_string());
    }

    // Check for package.json (Node.js)
    if path_buf.join("package.json").exists() {
        return Ok("npm test".to_string());
    }

    // Check for pytest/setup.py (Python)
    if path_buf.join("pytest.ini").exists() || path_buf.join("setup.py").exists() {
        return Ok("pytest".to_string());
    }

    // Check for go.mod (Go)
    if path_buf.join("go.mod").exists() {
        return Ok("go test ./...".to_string());
    }

    Err(adk_core::AdkError::Tool(
        "Could not detect project type. Please specify test_command manually.".to_string()
    ))
}

/// Detect appropriate lint command based on project type
fn detect_lint_command(path: &str, fix: bool) -> adk_core::Result<String> {
    let path_buf = Path::new(path);

    // Check for Cargo.toml (Rust)
    if path_buf.join("Cargo.toml").exists() {
        return Ok(if fix {
            "cargo clippy --fix --allow-dirty --allow-staged".to_string()
        } else {
            "cargo clippy -- -D warnings".to_string()
        });
    }

    // Check for package.json (Node.js)
    if path_buf.join("package.json").exists() {
        return Ok(if fix {
            "npm run lint -- --fix".to_string()
        } else {
            "npm run lint".to_string()
        });
    }

    // Check for Python
    if path_buf.join("setup.py").exists() || path_buf.join("pyproject.toml").exists() {
        return Ok(if fix {
            "black . && isort .".to_string()
        } else {
            "flake8 .".to_string()
        });
    }

    // Check for Go
    if path_buf.join("go.mod").exists() {
        return Ok("golangci-lint run".to_string());
    }

    Err(adk_core::AdkError::Tool(
        "Could not detect project type. Please specify lint_command manually.".to_string()
    ))
}

/// Parse test output to extract pass/fail counts
fn parse_test_output(stdout: &str, _stderr: &str) -> (u32, u32, u32) {
    // Rust cargo test format: "test result: ok. X passed; Y failed"
    if let Some(line) = stdout.lines().find(|l| l.contains("test result:")) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        let passed = parts.iter()
            .position(|&s| s == "passed;")
            .and_then(|i| parts.get(i.saturating_sub(1)))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        let failed = parts.iter()
            .position(|&s| s == "failed;")
            .and_then(|i| parts.get(i.saturating_sub(1)))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        return (passed, failed, passed + failed);
    }

    // Default: count test mentions
    let total = stdout.matches("test ").count() as u32;
    (0, 0, total)
}

/// Parse lint output to extract warning/error counts
fn parse_lint_output(stdout: &str, stderr: &str) -> (u32, u32) {
    let combined = format!("{}\n{}", stdout, stderr);

    // Rust clippy format
    if combined.contains("warning:") || combined.contains("error:") {
        let warnings = combined.matches("warning:").count() as u32;
        let errors = combined.matches("error:").count() as u32;
        return (warnings, errors);
    }

    // Generic format
    let warnings = combined.matches("warn").count() as u32;
    let errors = combined.matches("error").count() as u32;
    
    (warnings, errors)
}
