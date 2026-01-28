# Storage Management Domain Technical Documentation

## Overview

The Storage Management Domain serves as the persistent storage layer for Cowork Forge V2, providing comprehensive data persistence capabilities for project artifacts, session data, and development metadata. This domain is crucial for maintaining workflow continuity, enabling pipeline resumption, and ensuring data integrity throughout the AI-powered development lifecycle.

## Architecture and Design

### Directory Structure

The storage domain implements a well-organized directory hierarchy centered around the `.cowork/` directory:

```
.cowork/
├── data/           # Structured JSON data files
├── artifacts/      # Markdown documentation artifacts  
├── session/        # Session metadata and feedback
└── logs/           # Application logs (future use)
```

### Core Components

#### 1. Artifact Storage Module
**Responsibility**: Manages storage and retrieval of development artifacts including requirements, designs, and implementation plans.

**Key Data Types:**
- **Requirements**: Project requirements and specifications
- **Feature List**: Comprehensive feature inventory
- **Design Specifications**: System architecture and component designs
- **Implementation Plan**: Task breakdown and execution strategy
- **Code Metadata**: Code generation metadata and statistics

#### 2. Session Management Module  
**Responsibility**: Handles session data and workflow state persistence for continuity and resumption capabilities.

**Key Data Types:**
- **Session Metadata**: Current workflow state and progress tracking
- **Feedback History**: User feedback and review comments
- **Workflow State**: Pipeline execution context

## Technical Implementation

### File Operations Pattern

The module follows a consistent pattern for all storage operations:

```rust
pub fn load_requirements() -> Result<Requirements> {
    let path = data_path("requirements.json")?;
    if !path.exists() {
        return Ok(Requirements::new());
    }
    let content = fs::read_to_string(&path)?;
    let requirements: Requirements = serde_json::from_str(&content)?;
    Ok(requirements)
}

pub fn save_requirements(requirements: &Requirements) -> Result<()> {
    let path = data_path("requirements.json")?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let content = serde_json::to_string_pretty(requirements)?;
    fs::write(&path, content)?;
    Ok(())
}
```

### Path Management

The module provides utility functions for consistent path resolution:

```rust
/// Get the .cowork directory path, create if not exists
pub fn get_cowork_dir() -> Result<PathBuf> {
    let path = PathBuf::from(COWORK_DIR);
    if !path.exists() {
        fs::create_dir_all(&path)?;
        
        // Create subdirectories automatically
        fs::create_dir_all(path.join("data"))?;
        fs::create_dir_all(path.join("artifacts"))?;
        fs::create_dir_all(path.join("session"))?;
        fs::create_dir_all(path.join("logs"))?;
    }
    Ok(path)
}

fn data_path(filename: &str) -> Result<PathBuf> {
    Ok(get_cowork_dir()?.join("data").join(filename))
}

fn artifact_path(filename: &str) -> Result<PathBuf> {
    Ok(get_cowork_dir()?.join("artifacts").join(filename))
}

fn session_path(filename: &str) -> Result<PathBuf> {
    Ok(get_cowork_dir()?.join("session").join(filename))
}
```

## Data Types and Storage Mapping

### Structured Data Storage (JSON)

| Data Type | File Path | Purpose |
|-----------|-----------|---------|
| `Requirements` | `.cowork/data/requirements.json` | Project requirements and specifications |
| `FeatureList` | `.cowork/data/feature_list.json` | Comprehensive feature inventory |
| `DesignSpec` | `.cowork/data/design_spec.json` | System architecture and design |
| `ImplementationPlan` | `.cowork/data/implementation_plan.json` | Task breakdown and execution plan |
| `CodeMetadata` | `.cowork/data/code_metadata.json` | Code generation metadata |
| `SessionMeta` | `.cowork/session/meta.json` | Session state and progress |
| `FeedbackHistory` | `.cowork/session/feedback.json` | User feedback and reviews |

### Markdown Artifacts

| Artifact Type | File Path | Purpose |
|---------------|-----------|---------|
| Idea Document | `.cowork/artifacts/idea.md` | Initial project concept and scope |
| PRD Document | `.cowork/artifacts/prd.md` | Product Requirements Document |
| Design Document | `.cowork/artifacts/design.md` | System architecture documentation |
| Delivery Report | `.cowork/artifacts/delivery_report.md` | Final project delivery summary |

## Key Features

### 1. Automatic Directory Creation
The module automatically creates the necessary directory structure on first use, ensuring seamless operation without manual setup.

### 2. Error Handling with Context
Comprehensive error handling using `anyhow::Context` provides detailed error information for debugging and user feedback.

### 3. Lazy Initialization
Data loading functions automatically create empty data structures when files don't exist, supporting fresh project initialization.

### 4. Consistent Serialization
All structured data uses `serde_json` for consistent JSON serialization with pretty printing for human readability.

### 5. Feedback Management
The `append_feedback()` function enables incremental feedback collection while maintaining full history:

```rust
pub fn append_feedback(feedback: &Feedback) -> Result<()> {
    let mut history = load_feedback_history()?;
    history.feedbacks.push(feedback.clone());
    save_feedback_history(&history)?;
    Ok(())
}
```

## Integration Points

### Pipeline Orchestration Integration
The storage domain supports pipeline resumption by providing artifact detection and loading capabilities:

```rust
// Used by pipeline orchestrator to detect existing artifacts
pub fn cowork_dir_exists() -> bool {
    Path::new(COWORK_DIR).exists()
}
```

### Tool Infrastructure Integration
File tools in the Tool Infrastructure Domain depend on storage functions for persistent operations:

- **File Operations**: Reading/writing project files
- **Data Management**: Structured data persistence
- **Human Interaction**: Feedback and review storage

### Agent Management Integration
AI agents use storage functions to persist generated artifacts and maintain workflow state across agent executions.

## Error Handling Strategy

The module employs a comprehensive error handling approach:

1. **Context-Aware Errors**: All operations include contextual error information
2. **Graceful Degradation**: Missing files return default/empty data structures
3. **Directory Safety**: Automatic directory creation with proper error propagation
4. **Serialization Validation**: JSON parsing errors include file context

## Performance Considerations

### File I/O Optimization
- **Lazy Loading**: Data is loaded only when needed
- **Incremental Updates**: Feedback is appended rather than rewritten
- **Selective Persistence**: Only modified data is saved

### Memory Management
- **Streaming Operations**: Large files are handled efficiently
- **Minimal Data Retention**: Only necessary data is kept in memory
- **Serialization Efficiency**: JSON format balances readability and performance

## Security and Reliability

### Data Integrity
- **Structured Validation**: All data conforms to predefined schemas
- **Consistent Serialization**: Standardized JSON formatting
- **Error Recovery**: Graceful handling of corrupted files

### File System Safety
- **Path Sanitization**: All file paths are constructed safely
- **Directory Isolation**: `.cowork/` directory contains all project data
- **Permission Management**: Proper file permissions for multi-user environments

## Extension Points

The modular design supports easy extension for:

1. **Additional Data Types**: New domain models can be added with corresponding load/save functions
2. **Storage Backends**: Potential for database or cloud storage integration
3. **Caching Layer**: Performance optimization through in-memory caching
4. **Backup/Restore**: Automated backup mechanisms for project data

## Usage Examples

### Basic Data Persistence
```rust
// Save project requirements
let requirements = generate_requirements();
storage::save_requirements(&requirements)?;

// Load requirements for pipeline resumption
let existing_reqs = storage::load_requirements()?;
```

### Session Management
```rust
// Track session progress
let session_meta = SessionMeta {
    current_stage: "design",
    progress: 75.0,
    // ... other fields
};
storage::save_session_meta(&session_meta)?;
```

### Feedback Collection
```rust
// Collect user feedback during review
let feedback = Feedback {
    stage: "design_review",
    comment: "Need more detailed component specifications",
    timestamp: Utc::now(),
};
storage::append_feedback(&feedback)?;
```

The Storage Management Domain provides a robust foundation for data persistence in Cowork Forge V2, enabling reliable workflow execution, seamless pipeline resumption, and comprehensive artifact management throughout the AI-powered development lifecycle.