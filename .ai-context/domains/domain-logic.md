# Domain Logic

## Core Entities

| Entity | Location | Purpose |
|--------|----------|---------|
| Project | `domain/project.rs` | Aggregate root, contains Iterations |
| Iteration | `domain/iteration.rs` | Single development cycle |
| ProjectMemory | `domain/memory.rs` | Cross-iteration knowledge |
| IterationKnowledge | `domain/memory.rs` | Single iteration learnings |

## Relationships

```
Project 1:N Iteration
Project 1:1 ProjectMemory
Iteration 1:1 IterationKnowledge
```

## Iteration Lifecycle

```
Draft → Running → [Paused] → Completed | Failed
```

| Status | Operations |
|--------|------------|
| Draft | start, delete |
| Running | pause |
| Paused | continue, delete |
| Completed | create_evolution |
| Failed | retry, delete |

## Iteration Types

| Type | Description |
|------|-------------|
| Genesis | Fresh start |
| Evolution | Based on previous iteration |

## Knowledge Types

| Type | Scope |
|------|-------|
| Decision | Project-level |
| Pattern | Project-level |
| Issue | Iteration → Project |
| Learning | Iteration-level |

---
**Note**: For struct fields, read `domain/*.rs` directly.
