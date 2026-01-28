# Agent Management Domain Documentation

## Overview

The Agent Management Domain is a core business domain responsible for creating, configuring, and managing specialized AI agents that orchestrate the complete software development lifecycle in Cowork Forge. This domain implements the factory pattern to produce seven distinct agent types that handle specific stages of development, from initial idea capture to final delivery.

## Architecture Context

**Location**: `crates/cowork-core/src/agents/mod.rs`

**Domain Type**: Core Business Domain

**Complexity**: High (9.0/10.0)

**Importance**: Critical (9.0/10.0)

The Agent Management Domain sits at the heart of Cowork Forge, serving as the primary interface between the Pipeline Orchestration Domain and the specialized AI agents that execute development workflows. It leverages the adk-rust framework to create sophisticated agent configurations with precise tool assignments and instruction sets.

## Critical Technical Solution

### SequentialAgent Termination Bug Resolution

The domain implements a critical workaround for a fundamental limitation in the adk-rust framework:

```rust
// PROBLEM: When a sub-agent in LoopAgent calls exit_loop(), 
// it terminates the ENTIRE SequentialAgent, not just the LoopAgent.
// 
// SOLUTION: Remove exit_loop tools and use max_iterations=1 
// to let LoopAgent complete naturally, allowing SequentialAgent 
// to continue to next agent.
```

This solution ensures that pipeline execution continues seamlessly between stages by configuring loop agents with explicit iteration limits rather than relying on exit conditions that would prematurely terminate the entire workflow.

## Agent Types and Responsibilities

### 1. Idea Agent
**Purpose**: Capture and structure initial project ideas
- **Tools**: WriteFileTool, ReviewAndEditFileTool
- **Output**: `idea.md` file with structured project concept
- **Workflow**: Idea capture → File creation → User review → Handoff

### 2. PRD Loop Agent (Actor-Critic Pattern)
**Purpose**: Generate Product Requirements Document with iterative refinement
- **Actor Tools**: File operations, requirement creation, HITL integration
- **Critic Tools**: Requirement analysis, feedback provision
- **Max Iterations**: 1 (prevents infinite loops)
- **HITL Integration**: User feedback collection and requirement refinement

### 3. Design Loop Agent (Actor-Critic Pattern)  
**Purpose**: Create system architecture with validation
- **Actor Tools**: Design component creation, technology stack definition
- **Critic Tools**: Feature coverage verification, architecture validation
- **Max Iterations**: 1

### 4. Plan Loop Agent (Actor-Critic Pattern)
**Purpose**: Generate implementation task plans
- **Actor Tools**: Task creation, dependency management
- **Critic Tools**: Dependency validation, scope verification
- **Max Iterations**: 1

### 5. Coding Loop Agent (Actor-Critic Pattern)
**Purpose**: Implement code with quality validation
- **Actor Tools**: File operations, task status updates, command execution
- **Critic Tools**: Code simplicity verification, completion validation
- **Max Iterations**: 5 (allows for implementation and review cycles)
- **Philosophy**: Emphasizes simplicity and direct implementation

### 6. Check Agent
**Purpose**: Perform minimal quality validation
- **Tools**: Coverage checking, dependency analysis, file verification
- **Philosophy**: Lenient validation focusing on structural completeness
- **Fallback**: Can restart pipeline from appropriate stage if critical issues found

### 7. Delivery Agent
**Purpose**: Generate final delivery reports
- **Tools**: Project data aggregation, report generation
- **Critical Pre-check**: Verifies actual code files exist before report generation
- **Safety Measure**: Prevents false completion reporting

## Implementation Pattern: Actor-Critic Loop

The domain implements a sophisticated Actor-Critic pattern for iterative refinement in critical development stages:

```mermaid
graph LR
   [Pipeline Orchestrator] --> B[LoopAgent Factory]
    B --> C[Actor Agent]
    B --> D[Critic Agent]
    C --> E[Artifact Creation]
    D --> F[Quality Validation]
    E --> G[User Review]
    F --> H[Iteration Control]
    G --> I[Next Pipeline Stage]
    H --> I
```

### LoopAgent Configuration
```rust
let mut loop_agent = LoopAgent::new("prd_loop", vec![Arc::new(prd_actor), Arc::new(prd_critic)]);
loop_agent = loop_agent.with_max_iterations(1);  // Critical stability fix
```

## Tool Integration Strategy

Each agent is configured with precisely curated tool sets:

### Core Tool Categories
1. **File Operations**: ReadFileTool, WriteFileTool, ListFilesTool
2. **Data Management**: Requirement/Feature/Task CRUD operations
3. **Human Interaction**: ReviewAndEditFileTool, ReviewWithFeedbackTool
4. **Validation Tools**: Coverage checking, dependency analysis
5. **Execution Tools**: RunCommandTool for build/validation commands

### Tool Assignment Philosophy
- **Minimal Tool Sets**: Each agent receives only necessary tools
- **Security Focus**: File operations are sandboxed and validated
- **HITL Integration**: Human feedback tools at strategic decision points

## Instruction Management System

**Location**: `crates/cowork-core/src/instructions/mod.rs`

The domain maintains comprehensive instruction sets for each agent role:

### Instruction Structure
- **Role Definition**: Clear agent responsibilities and boundaries
- **Workflow Specification**: Step-by-step execution guidelines
- **Tool Usage Patterns**: Specific examples and best practices
- **Error Handling**: Graceful failure recovery strategies

### Key Instruction Features
1. **HITL Integration**: Explicit human review workflows
2. **Iteration Control**: Clear exit conditions and loop management
3. **Quality Gates**: Validation criteria at each stage
4. **Progressive Refinement**: Draft → Review → Formal artifact creation

## Integration Patterns

### Pipeline Orchestration Integration
```rust
// Pipeline orchestrator calls agent factory functions
let idea_agent = create_idea_agent(model.clone())?;
let prd_loop = create_prd_loop(model.clone())?;
// ... additional agent creation
```

### Tool Infrastructure Dependency
Agents depend on the Tool Infrastructure Domain for:
- File system operations
- Data persistence and retrieval
- Validation and quality checks
- Human interaction interfaces

### Data Modeling Integration
Agents utilize domain models from the Data Modeling Domain for:
- Structured requirement/feature/task definitions
- Type-safe data operations
- Schema validation and integrity checking

## Quality Assurance Mechanisms

### Built-in Validation
1. **Feature Coverage**: Verify all requirements have implementing features
2. **Task Completion**: Ensure all planned tasks are implemented
3. **Dependency Integrity**: Check for circular dependencies
4. **File Existence**: Validate actual code file creation

### Human Oversight
1. **Strategic Review Points**: User validation at critical decision stages
2. **Feedback Integration**: Incorporation of user suggestions
3. **Approval Gates**: Human approval required for stage progression

## Error Handling and Recovery

### Graceful Failure Modes
- **File Not Found**: Skip optional file operations and proceed
- **Tool Errors**: Provide clear feedback and continue
- **Validation Failures**: Suggest appropriate corrective actions

### Pipeline Recovery
- **Stage Resumption**: Check Agent can restart from appropriate stage
- **Incremental Development**: Support for partial pipeline execution
- **State Persistence**: Session management for workflow continuity

## Performance Considerations

### Resource Management
- **LLM Rate Limiting**: Integration with LLM Domain for API call optimization
- **Tool Efficiency**: Minimal tool invocation for maximum effect
- **Iteration Control**: Bounded loop iterations to prevent infinite execution

### Scalability
- **Agent Reusability**: Factory pattern enables agent instance reuse
- **Configuration Flexibility**: Model and tool configuration at runtime
- **Extensibility**: Modular design supports new agent types

## Best Practices and Patterns

### Agent Design Principles
1. **Single Responsibility**: Each agent handles one specific development stage
2. **Clear Boundaries**: Well-defined interfaces between agent types
3. **Progressive Refinement**: Iterative improvement with quality gates
4. **Human Collaboration**: Strategic HITL integration points

### Implementation Guidelines
1. **Tool Minimalism**: Assign only necessary tools to each agent
2. **Instruction Clarity**: Provide explicit, actionable guidance
3. **Error Resilience**: Design for graceful degradation
4. **Validation Focus**: Emphasize quality over quantity

## Future Evolution

The Agent Management Domain is designed for extensibility, supporting:
- **New Agent Types**: Additional specialized development roles
- **Enhanced Tooling**: Integration with new development tools
- **Advanced Patterns**: More sophisticated agent collaboration models
- **Domain Specialization**: Industry-specific agent configurations

This domain represents the intelligent core of Cowork Forge, balancing AI automation with human oversight to deliver robust, collaborative software development workflows.