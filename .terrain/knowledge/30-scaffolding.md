# Cowork Forge - Scaffolding Guide

## Project Structure

### New Project Layout
```
my-project/
├── .terrain/                    # Terrain knowledge assets
├── .ai-context/                 # AI context files
├── crates/                      # Rust crates
│   ├── core/                    # Core domain logic
│   ├── api/                     # API layer
│   └── cli/                     # CLI interface
├── docs/                        # Documentation
├── tests/                       # Integration tests
├── Cargo.toml                   # Rust workspace
└── README.md                    # Project documentation
```

### Cowork Forge Specific
```
cowork-forge/
├── .terrain/                    # Terrain knowledge assets
│   ├── agent/                   # Agent context and metadata
│   ├── knowledge/               # Domain knowledge files
│   └── .meta/                   # Freshness and sync metadata
├── .ai-context/                 # AI context files
├── crates/                      # Rust workspace
│   ├── cowork-core/             # Core domain logic
│   ├── cowork-cli/              # CLI adapter
│   └── cowork-gui/              # GUI application
├── docs/                        # Documentation
└── README.md                    # Project documentation
```

## Crate Templates

### Core Crate
```toml
# Cargo.toml
[package]
name = "cowork-core"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1.0"
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### CLI Crate
```toml
# Cargo.toml
[package]
name = "cowork-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
cowork-core = { path = "../cowork-core" }
clap = { version = "4.0", features = ["derive"] }
dialoguer = "0.10"
console = "0.15"
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

### GUI Crate
```toml
# Cargo.toml
[package]
name = "cowork-gui"
version = "0.1.0"
edition = "2024"

[dependencies]
cowork-core = { path = "../cowork-core" }
tauri = { version = "1.0", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Domain Entity Templates

### Aggregate Root
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub iterations: Vec<Iteration>,
    pub memory: Memory,
}

impl Project {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            iterations: Vec::new(),
            memory: Memory::new(),
        }
    }
}
```

### Value Object
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StageId(String);

impl StageId {
    pub fn new(id: &str) -> anyhow::Result<Self> {
        if id.is_empty() {
            anyhow::bail!("Stage ID cannot be empty");
        }
        Ok(Self(id.to_string()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### Domain Event
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectEvent {
    Created {
        project_id: Uuid,
        name: String,
        timestamp: DateTime<Utc>,
    },
    IterationStarted {
        project_id: Uuid,
        iteration_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    IterationCompleted {
        project_id: Uuid,
        iteration_id: Uuid,
        timestamp: DateTime<Utc>,
    },
}
```

## Agent Templates

### Actor Agent
```rust
use async_trait::async_trait;

#[async_trait]
pub trait Actor {
    async fn generate(&self, context: &Context) -> anyhow::Result<Output>;
    async fn refine(&self, output: &Output, feedback: &Feedback) -> anyhow::Result<Output>;
}
```

### Critic Agent
```rust
use async_trait::async_trait;

#[async_trait]
pub trait Critic {
    async fn review(&self, output: &Output) -> anyhow::Result<Review>;
    async fn suggest(&self, review: &Review) -> anyhow::Result<Feedback>;
}
```

## Tool Templates

### File Tool
```rust
use std::path::Path;

pub fn read_file(path: &Path) -> anyhow::Result<String> {
    // Validate path is within workspace
    validate_path(path)?;
    
    // Read file content
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

pub fn write_file(path: &Path, content: &str) -> anyhow::Result<()> {
    // Validate path is within workspace
    validate_path(path)?;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Write content
    std::fs::write(path, content)?;
    Ok(())
}
```

### Data Tool
```rust
use serde::{de::DeserializeOwned, Serialize};

pub fn load_json<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let content = read_file(path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn save_json<T: Serialize>(path: &Path, data: &T) -> anyhow::Result<()> {
    let content = serde_json::to_string_pretty(data)?;
    write_file(path, &content)?;
    Ok(())
}
```

## Configuration Templates

### Agent Configuration
```json
{
  "id": "pm-agent",
  "name": "Product Manager Agent",
  "description": "Transforms ideas into structured PRDs",
  "model": "gpt-4",
  "temperature": 0.7,
  "max_tokens": 4096,
  "system_prompt": "You are a product manager...",
  "tools": ["file_tools", "data_tools", "hitl_tools"]
}
```

### Stage Configuration
```json
{
  "id": "prd",
  "name": "PRD Generation",
  "description": "Generate Product Requirements Document",
  "agent": "pm-agent",
  "max_iterations": 3,
  "quality_threshold": 0.8,
  "inputs": ["idea"],
  "outputs": ["prd_document"],
  "next_stage": "design"
}
```

### Flow Configuration
```json
{
  "id": "default",
  "name": "Default Development Flow",
  "description": "Standard 7-stage development pipeline",
  "stages": ["idea", "prd", "design", "plan", "coding", "check", "delivery"],
  "transitions": {
    "idea": "prd",
    "prd": "design",
    "design": "plan",
    "plan": "coding",
    "coding": "check",
    "check": "delivery"
  }
}
```

## Testing Templates

### Unit Test
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_creation() {
        let project = Project::new(
            "Test Project".to_string(),
            "A test project".to_string(),
        );
        
        assert!(!project.id.is_nil());
        assert_eq!(project.name, "Test Project");
    }
    
    #[test]
    fn test_stage_id_validation() {
        assert!(StageId::new("valid-stage").is_ok());
        assert!(StageId::new("").is_err());
    }
}
```

### Integration Test
```rust
#[tokio::test]
async fn test_pipeline_execution() {
    let project = Project::new(
        "Test Project".to_string(),
        "A test project".to_string(),
    );
    
    let result = pipeline::execute(&mut project).await;
    assert!(result.is_ok());
}
```

## Documentation Templates

### API Documentation
```rust
/// Reads a file from the workspace
///
/// # Arguments
///
/// * `path` - Path to the file (must be within workspace)
///
/// # Returns
///
/// * `Result<String>` - File content or error
///
/// # Errors
///
/// * Returns error if path is outside workspace
/// * Returns error if file doesn't exist
/// * Returns error if file cannot be read
pub fn read_file(path: &Path) -> anyhow::Result<String> {
    // Implementation
}
```

### README Template
```markdown
# Project Name

Brief description of the project.

## Features

- Feature 1
- Feature 2
- Feature 3

## Installation

```bash
# Installation commands
```

## Usage

```bash
# Usage examples
```

## Development

```bash
# Development setup
```

## License

MIT
```