# Cowork Forge - Meta Inputs

## Project Metadata

### Basic Information
- **Project Name**: Cowork Forge
- **Repository Path**: .
- **Language**: Rust (edition 2024)
- **Framework**: adk-rust 0.5.0
- **GUI**: Tauri + React 18 + Ant Design
- **Architecture**: Hexagonal + DDD
- **License**: MIT

### Repository Statistics
- **Total Files**: 262
- **Total Tokens**: 401,973
- **Total Characters**: 1,503,951
- **Last Synced**: 2026-07-04T02:30:40.995935300+00:00
- **Baseline Git Head**: 0063d857ce13b366ae440fa1073a0b61cf14ebd0

## Module Structure

### Core Modules
1. **cowork-core** - Domain logic, pipeline, tools, agents
2. **cowork-cli** - CLI adapter (clap + dialoguer)
3. **cowork-gui** - Tauri + React GUI

### Cowork Core Submodules
- **pipeline/** - 7-stage orchestration & stage executor
- **domain/** - Project, Iteration, Memory aggregates
- **tools/** - 40+ ADK tools + MCP integration
- **agents/** - Agent wrappers (iterative, PM, legacy analyzer)
- **interaction/** - InteractiveBackend trait (CLI/GUI abstraction)
- **acp/** - Agent Client Protocol for external agents
- **config_definition/** - Data-driven config (agents, stages, flows)
- **instructions/** - Agent prompt library
- **skills/** - agentskills.io standard skill system
- **integration/** - Hook manager for external integrations
- **persistence/** - JSON-based storage

## Key Files by Token Count

### Documentation
1. `litho.docs/zh/2、架构概览.md` - 15,541 tokens
2. `litho.docs/en/2.Architecture.md` - 13,666 tokens
3. `litho.docs/zh/4、深入探索/4.8、Cowork GUI前端.md` - 8,452 tokens
4. `litho.docs/en/4.Deep-Exploration/GUI Frontend Domain.md` - 7,266 tokens
5. `litho.docs/zh/3、工作流程.md` - 7,061 tokens

### Source Code
1. `crates/cowork-core/src/tools/data_tools.rs` - 8,207 tokens
2. `crates/cowork-core/src/tools/file_tools.rs` - 6,550 tokens
3. `crates/cowork-core/src/runtime_analyzer.rs` - 6,108 tokens
4. `crates/cowork-gui/src/components/config/AgentConfigForm.tsx` - 6,072 tokens
5. `crates/cowork-core/src/config_definition/registry.rs` - 5,960 tokens

## Configuration Schema

### Agent Configuration
- **id**: Unique identifier
- **name**: Human-readable name
- **description**: Agent purpose
- **model**: LLM model to use
- **temperature**: Generation temperature (0.0-1.0)
- **max_tokens**: Maximum tokens per generation
- **system_prompt**: System instructions
- **tools**: Available tool identifiers

### Stage Configuration
- **id**: Unique identifier
- **name**: Human-readable name
- **description**: Stage purpose
- **agent**: Agent to use for this stage
- **max_iterations**: Maximum actor-critic cycles
- **quality_threshold**: Minimum quality score (0.0-1.0)
- **inputs**: Required input artifacts
- **outputs**: Generated output artifacts
- **next_stage**: Next stage in pipeline

### Flow Configuration
- **id**: Unique identifier
- **name**: Human-readable name
- **description**: Flow purpose
- **stages**: Ordered list of stage IDs
- **transitions**: Stage transition rules

## Domain Entities

### Project
- **id**: UUID
- **name**: Project name
- **description**: Project description
- **created_at**: Creation timestamp
- **updated_at**: Last update timestamp
- **iterations**: List of iterations
- **memory**: Persistent knowledge

### Iteration
- **id**: UUID
- **project_id**: Parent project ID
- **version**: Version number
- **created_at**: Creation timestamp
- **artifacts**: Generated artifacts
- **status**: Current status

### Memory
- **id**: UUID
- **project_id**: Parent project ID
- **knowledge**: Persistent knowledge base
- **created_at**: Creation timestamp
- **updated_at**: Last update timestamp

## Integration Points

### External Systems
- **LLM Providers**: OpenAI-compatible endpoints
- **File System**: Workspace-contained operations
- **Git**: Version control integration
- **MCP**: Model Context Protocol for external tools

### Protocols
- **ACP**: Agent Client Protocol for external agents
- **InteractiveBackend**: CLI/GUI abstraction
- **Tauri Events**: Real-time GUI communication

## Security Model

### Path Validation
- All file operations validated against workspace boundaries
- No access to paths outside project workspace
- Path traversal prevention

### Command Sanitization
- Dangerous commands blocked (rm -rf, sudo, etc.)
- Command whitelist for allowed operations
- Input validation for all commands

### LLM Rate Limiting
- Global semaphore (concurrency=1)
- 2-second delay between requests
- 30 requests per minute maximum

## Development Patterns

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