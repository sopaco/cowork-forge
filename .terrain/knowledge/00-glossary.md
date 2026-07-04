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