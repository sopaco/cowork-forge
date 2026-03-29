# Pipeline Domain

## Responsibility
Orchestrate the 7-stage development workflow lifecycle.

## Entry Points

| Function | Location | Description |
|----------|----------|-------------|
| `IterationExecutor::execute()` | `pipeline/executor/mod.rs:72` | Main iteration execution |
| `IterationExecutor::create_genesis_iteration()` | `pipeline/executor/mod.rs:35` | Create first iteration |
| `IterationExecutor::create_evolution_iteration()` | `pipeline/executor/mod.rs:55` | Create evolution iteration |
| `get_stages_from_flow()` | `pipeline/mod.rs:94` | Get stages from config |
| `create_stage_by_id()` | `pipeline/mod.rs:76` | Factory for stages |

## Key Types

```rust
// Stage execution result
enum StageResult {
    Success(Option<String>),    // Artifact path
    Failed(String),             // Error message
    Paused,                     // Waiting for HITL
    NeedsRevision(String),      // Needs feedback
    GotoStage(String, String),  // Jump to stage
}

// Pipeline execution context
struct PipelineContext {
    project: Project,
    iteration: Iteration,
    workspace_path: PathBuf,
}

// Stage trait
trait Stage: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn needs_confirmation(&self) -> bool;
    async fn execute(&self, ctx: &PipelineContext, interaction: Arc<dyn InteractiveBackend>) -> StageResult;
    async fn execute_with_feedback(&self, ctx: &PipelineContext, interaction: Arc<dyn InteractiveBackend>, feedback: &str) -> StageResult;
}
```

## Stage Implementations

| Stage | File | HITL | Pattern | Output |
|-------|------|------|---------|--------|
| Idea | `stages/idea.rs` | No | Simple Agent | idea.md |
| PRD | `stages/prd.rs` | Yes | Actor-Critic | prd.md |
| Design | `stages/design.rs` | Yes | Actor-Critic | design.md |
| Plan | `stages/plan.rs` | Yes | Actor-Critic | plan.md |
| Coding | `stages/coding.rs` | Yes | Actor-Critic | workspace/ |
| Check | `stages/check.rs` | No | Simple Agent | check_report.md |
| Delivery | `stages/delivery.rs` | No | Simple Agent | delivery_report.md |

## Execution Flow

```
1. prepare_workspace() → Copy from base if evolution
2. determine_start_stage() → Based on inheritance mode
3. For each stage:
   a. set_stage() on iteration
   b. stage.execute()
   c. If Success and critical: HITL gate
   d. If feedback: execute_with_feedback() (max 5 loops)
   e. complete_stage() on iteration
4. generate_knowledge() → Extract decisions, patterns
5. complete() iteration
```

## Inter-Module Interactions

```
Pipeline Domain
    │
    ├──► Interaction Domain: HITL confirmation gates
    │
    ├──► Domain Logic: Project, Iteration entities
    │
    ├──► Tools Domain: Stage uses tools via agents
    │
    ├──► Agents: Stage creates and executes agents
    │
    └──► Persistence: Save iteration state
```

## Configuration (V3)

Flow configuration affects pipeline behavior:
- `stages`: Ordered list of stage references
- `stop_on_failure`: Whether to stop or continue on failure
- `memory_scope`: Which memory to use (project/iteration)
- `inheritance.stage_mapping`: Start stage per inheritance mode

## Code Locations

```
crates/cowork-core/src/pipeline/
├── mod.rs              # Stage trait, stage factory
├── stage_executor.rs   # StageExecutor implementation
├── executor/
│   ├── mod.rs          # IterationExecutor
│   ├── interaction_ext.rs  # HITL extensions
│   ├── knowledge.rs    # Knowledge generation
│   └── workspace.rs    # Workspace preparation
└── stages/
    └── *.rs            # Individual stage implementations
```
