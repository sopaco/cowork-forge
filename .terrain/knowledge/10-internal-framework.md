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
- **Security Errors**: Path validation or command sanitization failures