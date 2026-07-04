# Developer Meta Inputs

Compiled from `terrain-meta.json` before Agent context generation.

## Private knowledge (.terrain\knowledge\00-glossary.md)

_Source: `.terrain/knowledge/00-glossary.md`_

# Cowork Forge - Domain Glossary

## Core Concepts

### Pipeline
- **Stage**: A discrete step in the 7-stage software development process
- **Actor**: Agent that generates initial output for a stage
- **Critic**: Agent that reviews and refines actor output
- **Iteration**: A complete pass through actor-critic cycle

### Agents
- **PM Agent**: Product Manager agent for requirements
- **Architect Agent**: System design and architecture
- **Engineer Agent**: Code generation and implementation
- **Check Agent**: Validation and testing
- **Delivery Agent**: Packaging and deployment

### Domain Entities
- **Project**: Top-level container for a software development effort
- **Iteration**: Versioned snapshot of project state
- **Memory**: Persistent knowledge across iterations
- **Artifact**: Generated output (PRD, design docs, code, etc.)

## Technical Terms

### Architecture
- **Hexagonal Architecture**: Ports and adapters pattern for isolation
- **DDD**: Domain-Driven Design for business logic
- **InteractiveBackend**: Trait for CLI/GUI abstraction
- **ACP**: Agent Client Protocol for external agent integration

### Tools
- **ADK Tools**: Agent Development Kit tool implementations
- **MCP Integration**: Model Context Protocol for external tools
- **File Tools**: Workspace-contained file operations
- **Data Tools**: Project data manipulation

### Configuration
- **Flow**: Sequence of stages with transitions
- **Stage Definition**: Configuration for pipeline stages
- **Agent Definition**: Agent capabilities and parameters

## Business Terms

### Development Process
- **Idea Phase**: Conceptualization and requirement gathering
- **PRD Phase**: Product Requirements Document creation
- **Design Phase**: System architecture and design
- **Plan Phase**: Implementation planning
- **Coding Phase**: Code generation and refinement
- **Check Phase**: Validation and testing
- **Delivery Phase**: Deployment and packaging

### Quality Assurance
- **Actor-Critic Loop**: Self-refinement through generation and review
- **HITL**: Human-in-the-loop for critical decisions
- **Watchdog**: Agent behavior monitoring
- **Objective Deviation**: Performance outside expected parameters

## Private knowledge (.terrain\knowledge\10-internal-framework.md)

_Source: `.terrain/knowledge/10-internal-framework.md` (truncated)_

# Cowork Forge - Internal Framework

## Core Framework Components

### Pipeline Executor
- **Location**: `crates/cowork-core/src/pipeline/executor/`
- **Purpose**: Orchestrates 7-stage development pipeline
- **Key Files**:
  - `mod.rs` - Pipeline execution entry point
  - `stage_executor.rs` - Stage execution logic
  - `knowledge.rs` - Knowledge management during execution

### Agent System
- **Location**: `crates/cowork-core/src/agents/`
- **Purpose**: Agent wrappers and orchestration
- **Key Components**:
  - `iterative_assistant.rs` - Actor-critic pattern implementation
  - `external_coding_agent.rs` - External agent integration
  - `legacy_project_analyzer.rs` - Legacy project analysis

### Tool System
- **Location**: `crates/cowork-core/src/tools/`
- **Purpose**: 40+ ADK tools for development tasks
- **Key Categories**:
  - `file_tools.rs` - Workspace file operations
  - `data_tools.rs` - Project data manipulation
  - `hitl_tools.rs` - Human-in-the-loop tools
  - `pm_tools.rs` - Project management tools

### Configuration System
- **Location**: `crates/cowork-core/src/config_definition/`
- **Purpose**: Data-driven configuration for agents and stages
- **Key Components**:
  - `registry.rs` - Configuration registry
  - `agent_definition.rs` - Agent configuration schema
  - `stage_definition.rs` - Stage configuration schema
  - `flow_definition.rs` - Flow configuration schema

## Development Patterns

### Hexagonal Architecture
- Domain logic has zero external dependencies
- Infrastructure adapters implement domain ports
- Clear separation between business logic and I/O

### Actor-Critic Pattern
- **Actor**: Generates initial output
- **Critic**: Reviews and suggests improvements
- **Iteration**: Continues until quality threshold met
- **Used In**: PRD, Design, Plan, Coding stages

### Strategy Pattern
- Stage implementations are pluggable
- Each stage defines its own behavior
- Common interface via trait implementations

### Template Method Pattern
- Pipeline execution flow is fixed
- Stage sequence with hooks for customization
- Consistent lifecycle across all stages

## Integration Points

### InteractiveBackend Trait
- **Location**: `crates/cowork-core/src/interaction/mod.rs`
- **Purpose**: Abstract CLI/GUI interaction
- **Implementations**:
  - CLI backend (dialoguer-based)
  - GUI backend (Tauri-based)

### ACP Protocol
- **Location**: `crates/cowork-core/src/acp/`
- **Purpose**: Agent Client Protocol for external agents
- **Components**:
  - `client.rs` - ACP client implementation
  - `mod.rs` - Protocol definitions

### Skill System
- **Location**: `crates/cowork-core/src/skills/`
- **Purpose**: agentskills.io standard skill system
- **Features**: Skill loading, execution, and management

## Persistence Layer

### Storage Format
- JSON-based storage for all domain entities
- File-based persistence in workspace
- No external database dependencies

### Key Entities
- **Project**: Top-level container
- **Iteration**: Versioned snapshots
- **Memory**: Persistent knowledge
- **Artifact**: Generated outputs

## Error Handling

### Pattern
- Always use `anyhow::Result`
- No `unwrap()` in production code
- Proper error propagation with `?` operator
- Context added with `.context()` method

### Error Categories
- **Domain Errors**: Business logic violations
- **Infrastructure Errors**: I/O and external system failures
- **Configuration Errors**: Invalid settings or parameters
- **Security Errors**: Path validation or command sanitizati

…

## Private knowledge (.terrain\knowledge\20-api-usage.md)

_Source: `.terrain/knowledge/20-api-usage.md` (truncated)_

# Cowork Forge - API Usage

## CLI Commands

### Core Commands
```bash
# Initialize new project
cargo run --package cowork-cli -- init <project-name>

# List all projects
cargo run --package cowork-cli -- list

# Show project status
cargo run --package cowork-cli -- status <project-id>

# Continue development iteration
cargo run --package cowork-cli -- continue <project-id>

# Delete project
cargo run --package cowork-cli -- delete <project-id>
```

### Knowledge Management
```bash
# Generate project knowledge
cargo run --package cowork-cli -- knowledge <project-id>

# Import external project
cargo run --package cowork-cli -- import <path>
```

### Configuration
```bash
# Show configuration
cargo run --package cowork-cli -- config show

# Update configuration
cargo run --package cowork-cli -- config set <key> <value>
```

## GUI Interface

### Development Mode
```bash
cd crates/cowork-gui
cargo tauri dev
```

### Build for Production
```bash
cd crates/cowork-gui
cargo tauri build
```

### Tauri Commands
- Project management commands
- Configuration commands
- Pipeline execution commands
- Real-time streaming events

## Agent System API

### Agent Types
```rust
// PM Agent - Requirements
pm_agent::generate_prd(idea: &str) -> Result<PrdDocument>

// Architect Agent - Design
architect_agent::create_design(prd: &PrdDocument) -> Result<DesignDocument>

// Engineer Agent - Code
engineer_agent::implement_code(design: &DesignDocument) -> Result<CodeArtifact>

// Check Agent - Validation
check_agent::validate(artifact: &CodeArtifact) -> Result<ValidationReport>

// Delivery Agent - Deployment
delivery_agent::package(artifact: &CodeArtifact) -> Result<PackageArtifact>
```

### Pipeline Execution
```rust
// Execute full pipeline
pipeline::execute(project: &mut Project) -> Result<Iteration>

// Execute single stage
pipeline::execute_stage(
    project: &mut Project,
    stage_id: &str
) -> Result<StageResult>
```

## Tool System API

### File Tools
```rust
// Read file within workspace
file_tools::read_file(path: &Path) -> Result<String>

// Write file within workspace
file_tools::write_file(path: &Path, content: &str) -> Result<()>

// List directory contents
file_tools::list_directory(path: &Path) -> Result<Vec<PathBuf>>
```

### Data Tools
```rust
// Load project data
data_tools::load_project(project_id: &str) -> Result<Project>

// Save project data
data_tools::save_project(project: &Project) -> Result<()>

// Update iteration
data_tools::update_iteration(
    project_id: &str,
    iteration: &Iteration
) -> Result<()>
```

### HITL Tools
```rust
// Request human input
hitl_tools::request_input(prompt: &str) -> Result<String>

// Request approval
hitl_tools::request_approval(
    proposal: &str
) -> Result<bool>

// Show progress
hitl_tools::show_progress(
    stage: &str,
    progress: f32
) -> Result<()>
```

## Configuration API

### Config Structure
```toml
# config.toml
[llm]
provider = "openai"
api_key = "your-api-key"
model = "gpt-4"

[pipeline]
max_iterations = 10
quality_threshold = 0.8

[workspace]
root_dir = "."
```

### Runtime Configuration
```rust
// Load configuration
config::load_config() -> Result<Config>

// Get specific setting
config::get_setting(key: &str) -> Result<Value>

// Update setting
config::update_setting(key: &str, value: Value) -> Result<()>
```

## Event System

### Tauri Events
- `pipeline-progress` - Pipeline execution updates
- `agent-output` - Agent generation output
- `stage-completed` - Stage completion no

…

## Private knowledge (.terrain\knowledge\30-scaffolding.md)

_Source: `.terrain/knowledge/30-scaffolding.md` (truncated)_

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

…

