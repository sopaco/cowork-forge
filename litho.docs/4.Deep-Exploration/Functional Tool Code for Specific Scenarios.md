# Functional Tool Code for Specific Scenarios

## Overview

The Functional Tool Code module provides a comprehensive suite of secure, reusable tools that enable AI agents to interact with the file system, execute commands, validate data, and manage project artifacts. These tools enforce strict security constraints while providing rich functionality required for automated software development workflows.

## Architecture and Design Principles

### Security-First Approach
All tools implement robust security measures to prevent unauthorized access or dangerous operations:

- **Path Validation**: All file operations validate paths to ensure they remain within the current working directory
- **Command Safety**: Command execution undergoes safety checks before running
- **Input Sanitization**: All parameters are validated and sanitized
- **Permission Controls**: Tools operate with minimal necessary privileges

### Tool Interface Standardization
Tools implement the `Tool` trait from `adk_core`, providing consistent:
- Parameter schemas via JSON Schema
- Error handling and reporting
- Async execution support
- Structured response formats

## Core Tool Categories

### 1. File System Tools

#### Security-Enforced File Operations
The file tools implement comprehensive security validation:

```rust
fn validate_path_security(path: &str) -> Result<PathBuf, String> {
    // Rule 1: Reject absolute paths
    if path_obj.is_absolute() {
        return Err("Security: Absolute paths are not allowed");
    }
    
    // Rule 2: Reject parent directory access (..)
    if path.contains("..") {
        return Err("Security: Parent directory access not allowed");
    }
    
    // Rule 3: Verify path is within current directory
    let canonical_path = current_dir.join(path).canonicalize()?;
    if !canonical_path.starts_with(&current_dir) {
        return Err("Security: Path escapes current directory");
    }
    
    Ok(canonical_path)
}
```

#### Advanced File Editing Capabilities
The tools support sophisticated file manipulation:

- **Line-range Replacement**: Replace specific line ranges with new content
- **Line Insertion**: Insert content at specific line positions
- **Line Deletion**: Remove lines within specified ranges
- **Append Operations**: Add content to file ends
- **Gitignore-Aware Traversal**: Respect `.gitignore` patterns during directory listing

### 2. Command Execution Tools

#### Safety-Checked Command Execution
Command execution includes comprehensive safety validation:

```rust
match check_command_safety(&params.cmd, cwd) {
    SafetyCheckResult::Blocked(reason) => {
        return Ok(json!({
            "success": false,
            "exit_code": -2,  // Safety rejection code
            "stderr": format!("SAFETY CHECK FAILED: {}", reason),
            "blocked": true,
            "block_reason": reason
        }));
    }
    SafetyCheckResult::Safe => { /* Proceed with execution */ }
}
```

#### Command Execution Features
- **Timeout Support**: Configurable execution time limits
- **Working Directory Control**: Execute commands in specified directories
- **Environment Variable Management**: Custom environment configuration
- **Structured Output Capture**: Comprehensive stdout/stderr/exit code capture

### 3. Validation Tools

#### Data Quality Assurance
Validation tools ensure data integrity throughout the development lifecycle:

```rust
fn validate_requirements_schema() -> Vec<String> {
    let mut errors =[];
    match load_requirements() {
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
```

#### Supported Validation Types
- **Schema Validation**: Ensure JSON data conforms to expected structures
- **Feature Coverage**: Verify all requirements map to implemented features
- **Data Format Checks**: Validate data integrity and completeness
- **Project Structure Validation**: Ensure project organization follows standards

### 4. Data Management Tools

#### Structured Data Creation and Modification
Data tools provide CRUD operations for project artifacts:

```rust
let requirement = Requirement {
    id: req_id.clone(),
    title: args["title"].as_str().unwrap().to_string(),
    description: args["description"].as_str().unwrap().to_string(),
    priority: priority,
    category: category,
    acceptance_criteria: criteria,
    // ... additional fields
};
```

#### Data Tool Capabilities
- **Requirement Management**: Create, update, and validate requirements
- **Feature Tracking**: Manage feature definitions and implementations
- **Design Specification**: Handle design document creation and validation
- **Implementation Planning**: Support planning artifact management

## Tool Integration Patterns

### Agent-Tool Interaction
Tools are designed for seamless integration with AI agents:

1. **Parameter Validation**: Tools automatically validate input parameters against JSON schemas
2. **Structured Responses**: All tools return consistent, structured JSON responses
3. **Error Handling**: Comprehensive error reporting with context
4. **Async Support**: Non-blocking execution for concurrent operations

### Security Integration
Security measures are integrated throughout the tool ecosystem:

- **Path Safety**: All file operations validate path security
- **Command Filtering**: Dangerous commands are blocked pre-execution
- **Input Validation**: All parameters undergo strict validation
- **Access Control**: Tools operate with principle of least privilege

## Implementation Details

### File Tools Implementation (cowork-core-v2)
The V2 implementation provides enhanced functionality:

- **Async Trait Implementation**: All tools implement `async_trait`
- **Improved Error Handling**: More detailed error messages and recovery
- **Enhanced Security**: Additional security validations and constraints
- **Better Performance**: Optimized file operations and memory usage

### Tool Bundles and Registration
Tools are organized into logical bundles for easy management:

```rust
pub struct FileToolsBundle {
    pub list_files: Arc<ListFilesTool>,
    pub read_file: Arc<ReadFileTool>,
    pub write_file: Arc<WriteFileTool>,
    // ... additional tools
}

pub fn create_file_tools() -> FileToolsBundle {
    FileToolsBundle {
        list_files: Arc::new(ListFilesTool),
        read_file: Arc::new(ReadFileTool),
        write_file: Arc::new(WriteFileTool),
        // ... tool initialization
    }
}
```

## Usage Examples

### File Operations
```rust
// List project structure
let result = list_files.execute(ctx, json!({
    "path": "src/",
    "recursive": true
})).await?;

// Read specific file content
let content = read_file.execute(ctx, json!({
    "path": "src/main.rs"
})).await?;

// Edit file with line precision
let result = replace_line_range.execute(ctx, json!({
    "path": "src/lib.rs",
    "start_line": 10,
    "end_line": 15,
    "new_content": "// Updated implementation"
})).await?;
```

### Command Execution
```rust
// Run build verification
let build_result = run_command.execute(ctx, json!({
    "cmd": "cargo check",
    "timeout_ms": 30000
})).await?;

// Execute tests
let test_result = run_command.execute(ctx, json!({
    "cmd": "npm test",
    "cwd": "./frontend"
})).await?;
```

### Data Validation
```rust
// Validate requirements structure
let validation = check_data_format.execute(ctx, json!({
    "data_type": "requirements"
})).await?;

// Check feature coverage
let coverage = check_feature_coverage.execute(ctx, json!({
    "requirements_file": "requirements.json",
    "implementation_dir": "src/"
})).await?;
```

## Security Considerations

### Path Safety Enforcement
- Absolute paths are explicitly rejected
- Parent directory traversal (`..`) is blocked
- All paths are canonicalized and verified to remain within project directory
- Symlink attacks are mitigated through proper path resolution

### Command Safety
- Dangerous commands (e.g., `rm -rf /`, format operations) are blocked
- Suspicious patterns trigger warnings and enhanced logging
- Execution occurs in controlled environments with limited privileges
- Timeout mechanisms prevent hanging processes

### Data Validation
- All input parameters are validated against schemas
- JSON injection attacks are prevented through proper serialization
- File size limits prevent resource exhaustion
- Content validation ensures data integrity

## Performance Optimizations

### Efficient File Operations
- **Batch Operations**: Tools support batch file operations when appropriate
- **Lazy Loading**: File content is loaded on-demand
- **Memory Management**: Large files are handled with streaming when possible
- **Caching**: Directory listings and file metadata are cached appropriately

### Command Execution Optimization
- **Process Management**: Commands are executed with proper resource limits
- **Output Streaming**: Large command outputs are handled efficiently
- **Timeout Handling**: Commands are terminated gracefully when timeouts occur
- **Resource Cleanup**: All resources are properly cleaned up after execution

## Error Handling and Recovery

### Comprehensive Error Reporting
Tools provide detailed error information including:
- Error type categorization
- Contextual information about failures
- Recovery suggestions when appropriate
- Security violation details

### Graceful Degradation
- Tools handle missing files and directories gracefully
- Permission errors are reported with clear explanations
- Invalid parameters result in helpful error messages
- Partial failures are handled with appropriate fallbacks

## Testing and Quality Assurance

### Tool Testing Strategy
- Unit tests for individual tool functions
- Integration tests for tool-agent interactions
- Security tests for path validation and command safety
- Performance tests for large-scale operations

### Quality Metrics
- Code coverage targets for critical security functions
- Performance benchmarks for file operations
- Security audit compliance for all validation logic
- Documentation coverage for all public APIs

The Functional Tool Code module provides the essential building blocks that enable AI agents to safely and effectively interact with the development environment, ensuring that automated software development remains secure, reliable, and productive.