# Domain Logic

## Responsibility
Core business entities following DDD patterns.

## Core Concepts

### Project (Aggregate Root)
- **Location**: `domain/project.rs`
- **Purpose**: Root entity for a software project
- **Contains**: Multiple Iterations

### Iteration (Entity)
- **Location**: `domain/iteration.rs`
- **Purpose**: Single development cycle
- **Lifecycle**: Draft → Running → Paused → Completed/Failed
- **Types**: Genesis (fresh) | Evolution (based on previous)

### Memory (Aggregates)
- **Location**: `domain/memory.rs`
- **ProjectMemory**: Cross-iteration knowledge (decisions, patterns)
- **IterationKnowledge**: Single iteration's learnings

## Entity Relationships

```
Project 1:N Iteration
Project 1:1 ProjectMemory
Iteration 1:1 IterationKnowledge

Iteration ──inherits from──► Iteration (evolution)
```

## Iteration Lifecycle

```
Draft → start() → Running
                  │
                  ├── pause() → Paused → resume() → Running
                  │
                  ├── complete() → Completed
                  │
                  └── fail() → Failed → retry() → Running
```

## Inheritance Modes

| Mode | Artifacts | Workspace | Use Case |
|------|-----------|-----------|----------|
| None | Fresh | Fresh | Genesis iteration |
| Partial | Copy | Fresh | Keep requirements, new code |
| Full | Copy | Copy | Continue from previous |

## Knowledge Types

| Type | Scope | Description |
|------|-------|-------------|
| Decision | Project | Architecture/tech choices |
| Pattern | Project | Reusable solutions |
| Issue | Iteration→Project | Problems encountered |
| Learning | Iteration | Insights gained |

## Domain Invariants

1. Iteration numbers are sequential within a project
2. Only Paused iterations can be resumed
3. Stages execute in order (cannot skip ahead)
4. All file paths must be within workspace

## Code Locations

```
crates/cowork-core/src/domain/
├── mod.rs         # Re-exports
├── project.rs     # Project, ProjectMetadata
├── iteration.rs   # Iteration, Artifacts, InheritanceMode
└── memory.rs      # ProjectMemory, IterationKnowledge
```

---
**Note**: For detailed struct fields, read the corresponding .rs files directly.