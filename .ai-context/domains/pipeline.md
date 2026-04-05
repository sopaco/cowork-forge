# Pipeline & Agents Domain

## 7-Stage Workflow

| Stage | HITL | Pattern | Output |
|-------|------|---------|--------|
| Idea | Yes | Simple | idea.md |
| PRD | Yes | Actor-Critic | prd.md |
| Design | Yes | Actor-Critic | design.md |
| Plan | Yes | Actor-Critic | plan.md |
| Coding | Yes | Actor-Critic | workspace/ |
| Check | No | Simple | check_report.md |
| Delivery | No | Simple | delivery_report.md |

## Execution Flow

```
prepare_workspace() → stages[idea→delivery] → generate_knowledge()
                              ↓
                    HITL gate (all stages)
```

## Actor-Critic Pattern

```
Actor generates → Critic validates → HITL confirms → [Pass|Feedback loop]
```

**Critical**: LoopAgent uses `max_iterations=1` (see ADR-001)
**Coding Stage**: Allows `max_iterations=5` for iterative code refinement (ADR-003)

## Agent Types

| Type | Stages | Pattern |
|------|--------|---------|
| Simple | Idea, Check, Delivery | Single LlmAgent |
| Actor-Critic | PRD, Design, Plan, Coding | LoopAgent |
| PM Agent | Post-Delivery | Interactive |

## IterationExecutor Architecture

The executor uses a modular architecture in `pipeline/executor/`:

| Module | Purpose |
|--------|---------|
| `mod.rs` | Main entry, iteration lifecycle, stage execution loop |
| `interaction_ext.rs` | ConfirmationAction enum, HITL flow extension |
| `knowledge.rs` | Knowledge generation, injection, document summaries |
| `workspace.rs` | Workspace preparation, inheritance (Full/Partial/None) |

### Knowledge Injection (knowledge.rs)
- Priority: Knowledge Summary → fallback to baseline artifacts
- Injects project context into evolution iterations

### Inheritance Flow (workspace.rs)
- `None`: Fresh start, no files inherited
- `Partial`: Copy code files only, skip artifacts/
- `Full`: Copy all files and artifacts

## Code Locations

| Component | Location |
|-----------|----------|
| Stage trait | `pipeline/mod.rs` |
| Executor | `pipeline/executor/mod.rs` |
| HITL extension | `pipeline/executor/interaction_ext.rs` |
| Knowledge logic | `pipeline/executor/knowledge.rs` |
| Workspace logic | `pipeline/executor/workspace.rs` |
| Stages | `pipeline/stages/*.rs` |
| Agent builders | `agents/iterative_assistant.rs` |
| External agent | `agents/external_coding_agent.rs` |
| Instructions | `instructions/*.rs` |
