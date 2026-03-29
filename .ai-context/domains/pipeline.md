# Pipeline & Agents Domain

## 7-Stage Workflow

| Stage | HITL | Pattern | Output |
|-------|------|---------|--------|
| Idea | No | Simple | idea.md |
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
                    HITL gate (critical stages)
```

## Actor-Critic Pattern

```
Actor generates → Critic validates → HITL confirms → [Pass|Feedback loop]
```

**Critical**: LoopAgent uses `max_iterations=1` (see ADR-001)

## Agent Types

| Type | Stages | Pattern |
|------|--------|---------|
| Simple | Idea, Check, Delivery | Single LlmAgent |
| Actor-Critic | PRD, Design, Plan, Coding | LoopAgent |
| PM Agent | Post-Delivery | Interactive |

## Code Locations

| Component | Location |
|-----------|----------|
| Stage trait | `pipeline/mod.rs` |
| Executor | `pipeline/executor/mod.rs` |
| Stages | `pipeline/stages/*.rs` |
| Agent builders | `agents/mod.rs` |
| Instructions | `instructions/*.rs` |
| External agent | `agents/external_coding_agent.rs` |
