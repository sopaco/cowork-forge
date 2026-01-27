# Human-in-the-Loop Controller Technical Documentation

## Overview

The Human-in-the-Loop (HITL) Controller is a critical component of the Cowork Forge system that enables human oversight in automated AI-driven workflows. It provides interactive tools for file review, editing, feedback collection, and decision confirmation, ensuring that human judgment is integrated at key decision points throughout the software development lifecycle.

## Architecture

### Core Components

The HITL Controller consists of two primary implementation layers:

#### 1. HitlController (crates/cowork-core/src/hitl/mod.rs)
A direct controller implementation providing core interaction utilities:
- **Text Input**: `input()` - Collects free-form text input from users
- **Confirmation**: `confirm()` - Presents yes/no confirmation dialogs
- **JSON Review**: `review_and_edit_json()` - Allows users to review and modify JSON content with validation
- **Feedback Collection**: `collect_feedback()` and `collect_feedback_with_default()` - Gather structured user feedback
- **Menu Selection**: `select()` - Presents multiple choice options to users

#### 2. HITL Tools (crates/cowork-core-v2/src/tools/hitl_tools.rs and control_tools.rs)
Tool-based implementations adhering to the `adk_core::Tool` trait for integration with AI agents:

##### ReviewAndEditFileTool
- **Purpose**: Basic file review and editing capability
- **Interaction Flow**: 
  1. Reads file content
  2. Shows preview (first 10 lines)
  3. Asks "Do you want to edit this file? (y/n)"
  4. Opens system editor if user confirms
  5. Saves changes back to file

##### ReviewWithFeedbackTool (Enhanced)
- **Purpose**: Advanced review with three interaction modes
- **Interaction Modes**:
  - **"edit"**: Opens file in system editor
  - **"pass"**: Continues without changes
  - **Free-form text**: Captured as feedback for agent processing

##### ProvideFeedbackTool
- **Purpose**: Structured feedback collection
- **Feedback Types**: Build errors, quality issues, missing requirements, suggestions
- **Severity Levels**: Critical, major, minor
- **Storage**: Persists feedback via `append_feedback()` function

##### AskUserTool
- **Purpose**: Interactive questioning
- **Question Types**: 
  - Yes/No confirmation (`yes_no`)
  - Text input (`text_input`)

## Implementation Details

### Technical Stack

- **Primary Framework**: `dialoguer` crate for terminal-based UI components
- **Serialization**: `serde_json` for JSON validation and manipulation
- **Async Support**: `async_trait` for async tool execution
- **Error Handling**: Comprehensive error wrapping with `adk_core::AdkError`

### Key Design Patterns

#### 1. Preview-First Approach
```rust
// Show preview before asking for confirmation
println!("📝 {} - {}", title, file_path);
println!("  ────────────────────────────────────────");
for (i, line) in content.lines().take(10).enumerate() {
    println!("  {}: {}", i + 1, line);
}
```

#### 2. Validation-Driven Editing
```rust
// Validate JSON after user edits
match serde_json::from_str::<serde_json::Value>(&text) {
    Ok(_) => Ok(Some(text)), // Valid JSON
    Err(e) => {
        println!("❌ JSON 格式错误: {}", e);
        // Offer retry option with validation feedback
    }
}
```

#### 3. Progressive Enhancement
The tools provide increasing levels of sophistication:
- **Basic**: Simple yes/no confirmation
- **Intermediate**: File editing with preview
- **Advanced**: Multiple interaction modes with feedback capture

### Integration Points

#### Agent Integration
HITL tools are invoked by AI agents through the `adk_core::Tool` interface:

```rust
// Agent calls HITL tool
let result = ctx.call_tool("review_with_feedback", json!({
    "path": "requirements.md",
    "title": "Review Requirements Document"
})).await?;
```

#### Workflow Integration
HITL interactions are strategically placed at critical workflow stages:

1. **PRD Generation**: User reviews and edits requirements document
2. **Design Phase**: User approves architectural decisions
3. **Implementation Plan**: User confirms technical approach
4. **Code Generation**: User reviews generated code files

## Usage Patterns

### File Review Workflow
```
1. Agent generates file content
2. HITL tool shows preview to user
3. User chooses action (edit/pass/feedback)
4. System processes user decision
5. Agent receives structured response
6. Workflow continues based on user input
```

### Feedback Integration
```
1. User provides feedback via HITL tool
2. Feedback is serialized and stored
3. Agent retrieves feedback in next iteration
4. Agent adjusts behavior based on feedback
5. System maintains feedback history for auditability
```

## Configuration and Customization

### Prompt Customization
Tools support customizable prompts:
```rust
let prompt = args["prompt"].as_str().unwrap_or("输入 'edit' 编辑，'pass' 继续，或直接输入修改建议");
```

### Preview Settings
- **Line Limits**: Configurable preview length (default: 10-15 lines)
- **Format Options**: JSON formatting with syntax highlighting
- **Validation**: Optional JSON schema validation

## Error Handling

### Graceful Failure Modes
The HITL Controller implements robust error handling:

1. **File I/O Errors**: Wrapped with descriptive error messages
2. **User Input Errors**: Retry mechanisms for invalid input
3. **Editor Failures**: Fallback to alternative interaction modes
4. **Validation Failures**: Clear feedback with retry options

### Recovery Strategies
```rust
if retry {
    self.review_and_edit_json(title, data) // Recursive retry
} else {
    println!("⚠️  放弃修改，使用原始内容");
    Ok(None) // Graceful fallback
}
```

## Security Considerations

### Path Safety
- All file operations validate paths relative to project root
- Prevents directory traversal attacks
- Validates file existence before operations

### Input Validation
- JSON content validated before saving
- Command injection prevention in editor invocation
- Safe handling of user-provided content

## Performance Characteristics

### Interactive Latency
- **Editor Launch**: System-dependent (typically < 2 seconds)
- **File Operations**: O(n) where n is file size
- **User Response Time**: Variable (human-dependent factor)

### Resource Usage
- **Memory**: Minimal (file content buffering only)
- **Storage**: Feedback persistence adds minimal overhead
- **CPU**: Primarily I/O-bound operations

## Best Practices

### For Tool Implementers
1. Always provide clear previews before requesting user input
2. Implement validation for user-modified content
3. Support retry mechanisms for invalid input
4. Maintain consistent return formats across tools

### For Agent Developers
1. Choose appropriate HITL tools based on interaction complexity needed
2. Handle all possible return states from HITL interactions
3. Incorporate user feedback into subsequent agent reasoning
4. Log HITL interactions for auditability

## Future Enhancements

### Planned Features
- **Batch Operations**: Review multiple files in single interaction
- **Template Support**: Pre-defined editing templates for common artifacts
- **Visual Diff**: Side-by-side comparison for edited content
- **Collaborative Review**: Multi-user feedback collection

### Integration Roadmap
- **Git Integration**: HITL tools for code review workflows
- **CI/CD Integration**: Human approval gates in automated pipelines
- **Multi-modal Interfaces**: Web-based and mobile HITL interfaces

The HITL Controller represents a sophisticated balance between automation efficiency and human oversight, enabling Cowork Forge to deliver AI-powered development while maintaining human control over critical decisions.