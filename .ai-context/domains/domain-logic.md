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

| Type | Scope | Description |
|------|-------|-------------|
| Decision | Project-level | Architecture decisions with rationale |
| Pattern | Project-level | Reusable patterns with tags and usage |
| Issue | Iteration → Project | Known issues and tech debt |
| Learning | Iteration-level | Session-specific learnings |
| Insight | Stage-level | Observations during execution |

## Memory Management

| Component | Location | Purpose |
|-----------|----------|---------|
| ProjectMemory | `domain/memory.rs` | Cross-iteration knowledge aggregate |
| IterationKnowledge | `domain/memory.rs` | Single iteration knowledge snapshot |
| MemoryStore | `persistence/memory_store.rs` | JSON-based memory persistence |
| Knowledge Tools | `tools/knowledge_tools.rs` | Load, save, promote knowledge |
| Knowledge Generator | `pipeline/executor/knowledge.rs` | Post-delivery knowledge extraction |

### Knowledge Promotion Flow
```
Insight (during execution) 
  → SaveInsightTool 
  → PromoteToDecisionTool / PromoteToPatternTool 
  → ProjectMemory (decision/pattern)
```

### Knowledge Query Scopes
- `project`: Historical project-level knowledge
- `iteration`: Current iteration-specific knowledge
- `latest`: Merged view with context aggregation

---
**Note**: For struct fields, read `domain/*.rs` directly.
