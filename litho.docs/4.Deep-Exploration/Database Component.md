# Database Component Technical Documentation

## Overview

The Database Component in Cowork Forge serves as the persistent storage layer responsible for managing all development artifacts throughout the software development lifecycle. This component abstracts file system operations behind domain-specific APIs, providing consistent storage and retrieval mechanisms for both structured JSON data and human-readable Markdown documents.

## Architecture

### Core Components

#### 1. Storage Layer (V2 Implementation)
**File**: `crates/cowork-core-v2/src/storage/mod.rs`

The V2 storage implementation provides a functional approach to artifact management with dedicated helper functions for each data type:

```rust
// Core directory structure management
pub fn get_cowork_dir() -> Result<PathBuf>  // Creates .cowork/ with subdirectories
fn data_path(filename: &str) -> Result<PathBuf>    // Path to .cowork/data/
fn artifact_path(filename: &str) -> Result<PathBuf> // Path to .cowork/artifacts/
fn session_path(filename: &str) -> Result<PathBuf>  // Path to .cowork/session/
```

**Supported Data Types**:
- `load_requirements()` / `save_requirements()` - Requirements management
- `load_feature_list()` / `save_feature_list()` - Feature tracking
- `load_design_spec()` / `save_design_spec()` - Design specifications
- `load_implementation_plan()` / `save_implementation_plan()` - Implementation plans
- `load_code_metadata()` / `save_code_metadata()` - Code metadata
- `load_session_meta()` / `save_session_meta()` - Session management

#### 2. Artifact Store (V1 Implementation)
**File**: `crates/cowork-core/src/memory/mod.rs`

The V1 implementation provides an object-oriented interface with the `ArtifactStore` struct:

```rust
pub struct ArtifactStore {
    store: FileArtifactStore,
}

impl ArtifactStore {
    pub fn put<T: Serialize>(&self, session_id: &str, stage: Stage, artifact: &T) -> Result<String>
    pub fn get<T: DeserializeOwned>(&self, session_id: &str, artifact_id: &str) -> Result<T>
    pub fn list(&self, session_id: &str) -> Result<Vec<ArtifactMeta>>
    pub fn session_exists(&self, session_id: &str) -> bool
}
```

### Data Models

#### V2 Data Models (`cowork-core-v2/src/data/models.rs`)
The V2 implementation uses strongly-typed data structures:

```rust
// Requirements management
pub struct Requirements {
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub requirements: Vec<Requirement>,
}

// Feature tracking
pub struct FeatureList {
    pub schema_version: String,
    pub features: Vec<Feature>,
}

// Design specifications
pub struct DesignSpec {
    pub schema_version: String,
    pub architecture: Architecture,
    pub technology_stack: TechnologyStack,
}
```

#### V1 Artifact Envelope (`cowork-core/src/artifacts/mod.rs`)
The V1 implementation uses a generic envelope pattern:

```rust
pub struct ArtifactEnvelope<T> {
    pub meta: ArtifactMeta,
    pub summary: Vec<String>,
    pub links: ArtifactLinks,
    pub data: T,
}

pub enum Stage {
    IdeaIntake,
    Requirements,
    Design,
    Plan,
    Coding,
    Check,
    Feedback,
    Delivery,
}
```

## Directory Structure

The Database Component maintains a standardized directory hierarchy:

```
.cowork/
├── data/           # Structured JSON artifacts
│   ├── requirements.json
│   ├── feature_list.json
│   ├── design_spec.json
│   └── implementation_plan.json
├── artifacts/      # Human-readable Markdown documents
│   ├── idea_intake.*.md
│   ├── requirements.*.md
│   └── design.*.md
├── session/        # Session-specific metadata
│   └── meta.json
└── logs/           # System logs and audit trails
```

## Key Features

### 1. Automatic Directory Management
- Creates `.cowork/` directory structure on first access
- Ensures all subdirectories (`data`, `artifacts`, `session`, `logs`) exist
- Handles missing parent directories during file operations

### 2. Dual-Format Persistence
- **JSON files**: Machine-readable structured data with full serialization
- **Markdown files**: Human-readable documentation with templated content
- Automatic synchronization between formats

### 3. Session-Based Organization
- Artifacts organized by session ID for multi-project support
- Session metadata tracks workflow state and progress
- Enables resumable workflows through session detection

### 4. Error Handling and Recovery
- Comprehensive error handling with `anyhow::Result` context
- Graceful handling of missing files with default values
- Atomic write operations to prevent data corruption

## Implementation Details

### Serialization Strategy
```rust
// JSON serialization with pretty formatting
let content = serde_json::to_string_pretty(artifact)?;
fs::write(&path, content)?;

// Deserialization with error context
let artifact: T = serde_json::from_str(&content)
    .with_context(|| format!("Failed to parse {}", path.display()))?;
```

### File Naming Convention
- **JSON files**: `{stage}.{artifact_id}.json`
- **Markdown files**: `{stage}.{artifact_id}.md`
- **Session directories**: `{session_id}/artifacts/`

### Markdown Generation
The system automatically generates human-readable Markdown from JSON metadata:

```rust
fn generate_markdown(&self, json: &serde_json::Value) -> Result<String> {
    let mut md = String::new();
    md.push_str("# Artifact\n\n");
    md.push_str(&format!("- **Session ID**: {}\n", meta["session_id"]));
    md.push_str(&format!("- **Artifact ID**: {}\n", meta["artifact_id"]));
    // ... additional metadata and content
    Ok(md)
}
```

## Integration Points

### Agent Integration
All AI agents (`IdeaIntakeAgent`, `PRDAgent`, `DesignAgent`, etc.) interact with the Database Component through standardized interfaces:

```rust
// Example agent usage
let artifact_store = ArtifactStore::new(".cowork");
let prd_artifact = artifact_store.get::<PRD>(session_id, "prd-001")?;
artifact_store.put(session_id, Stage::Requirements, &updated_prd)?;
```

### Tool Integration
File and artifact tools leverage the storage layer for operations:
- `FileTools` for low-level file operations
- `ArtifactTools` for high-level document management
- `ValidationTools` for data integrity checks

## Error Handling Patterns

### Graceful Defaults
```rust
pub fn load_requirements() -> Result<Requirements> {
    let path = data_path("requirements.json")?;
    if !path.exists() {
        return Ok(Requirements::new()); // Return empty default
    }
    // ... load existing data
}
```

### Context-Aware Errors
```rust
fs::create_dir_all(parent)
    .with_context(|| format!("Failed to create directory {:?}", parent))?;
```

## Performance Considerations

### File System Efficiency
- Lazy directory creation (created only when needed)
- Efficient file scanning during artifact lookup
- Minimal file I/O through batch operations

### Memory Management
- Stream-based file reading for large artifacts
- Structured data models with efficient serialization
- Automatic cleanup of temporary files

## Security Features

### Path Safety
- All file operations validate paths within `.cowork/` boundary
- Prevention of directory traversal attacks
- Secure file permissions on created directories

### Data Integrity
- Atomic write operations prevent partial updates
- Backup mechanisms for critical artifacts
- Validation of serialized data formats

## Usage Examples

### Basic Artifact Storage
```rust
// Save a new requirement artifact
let requirements = Requirements::new();
save_requirements(&requirements)?;

// Load existing design specification
let design_spec = load_design_spec()?;
```

### Session Management
```rust
// Check if session exists
if artifact_store.session_exists(session_id) {
    // Resume workflow
    let artifacts = artifact_store.list(session_id)?;
    // Process existing artifacts
}
```

### Multi-Format Access
```rust
// JSON data for programmatic access
let json_data = load_requirements()?;

// Markdown for human review (auto-generated)
// File available at: .cowork/artifacts/requirements.{id}.md
```

## Evolution and Versioning

The Database Component has evolved from V1's object-oriented `ArtifactStore` to V2's functional approach with dedicated data models. Both versions maintain backward compatibility through consistent file formats and directory structures.

This component forms the foundation for Cowork Forge's resumable workflows, enabling seamless continuation of development sessions and providing a reliable audit trail of all AI-generated artifacts throughout the software development lifecycle.