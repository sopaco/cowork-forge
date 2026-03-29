# Pipeline & Agents Domain

## Pipeline Orchestration

### 7-Stage Workflow

| Stage | HITL | Pattern | Output |
|-------|------|---------|--------|
| Idea | No | Simple Agent | idea.md |
| PRD | Yes | Actor-Critic | prd.md |
| Design | Yes | Actor-Critic | design.md |
| Plan | Yes | Actor-Critic | plan.md |
| Coding | Yes | Actor-Critic | workspace/ |
| Check | No | Simple Agent | check_report.md |
| Delivery | No | Simple Agent | delivery_report.md |

### Stage Trait

```rust
trait Stage: Send + Sync {
    fn name(&self) -> &str;
    fn needs_confirmation(&self) -> bool;
    async fn execute(&self, ctx: &PipelineContext, interaction: Arc<dyn InteractiveBackend>) -> StageResult;
}
```

### Execution Flow

```
prepare_workspace() → determine_start_stage() → [stages] → generate_knowledge()
                                              ↓
                                        HITL gate (critical stages)
```

### Code Location

`crates/cowork-core/src/pipeline/`
- `mod.rs` - Stage trait, stage factory
- `executor/mod.rs` - IterationExecutor
- `stages/*.rs` - Individual implementations

---

## Agents

### Agent Types

| Type | Stages | Description |
|------|--------|-------------|
| Simple | Idea, Check, Delivery | Single LlmAgent |
| Actor-Critic | PRD, Design, Plan, Coding | LoopAgent with Actor+Critic |
| PM Agent | Post-Delivery | Interactive project manager |

### Actor-Critic Pattern

```
Actor: Generates artifact with tools
    ↓
Critic: Validates, provides feedback
    ↓
HITL: Human confirms or gives feedback
    ↓
[Loop back if feedback, max 5 times]
```

**Critical Implementation Note**:
- LoopAgent uses `max_iterations=1`
- Reason: SequentialAgent bug causes exit_loop to terminate entire chain
- Do not change this parameter

### Agent Builders Location

`crates/cowork-core/src/agents/mod.rs`

Key functions:
- `create_idea_agent()`, `create_check_agent()`, `create_delivery_agent()`
- `create_prd_loop()`, `create_design_loop()`, `create_plan_loop()`, `create_coding_loop()`
- `create_project_manager_agent()`

### Tool Categories per Agent

| Agent | Key Tools |
|-------|-----------|
| PRD Actor | LoadIdea, CreateRequirement, AddFeature, SavePrdDoc |
| PRD Critic | GetRequirements, ProvideFeedback |
| Design Actor | GetRequirements, GetDesign, SaveDesignDoc |
| Coding Actor | ReadFile, WriteFile, RunCommand, CheckTests |
| PM Agent | PMGotoStage, PMCreateIteration, PMRespond |

### External Coding Agent

`agents/external_coding_agent.rs` - ACP integration for external agents (opencode, iflow, codex, etc.)

---

## Instructions (Prompts)

`crates/cowork-core/src/instructions/`

Each stage has corresponding instruction constants:
- `IDEA_AGENT_INSTRUCTION`
- `PRD_ACTOR_INSTRUCTION`, `PRD_CRITIC_INSTRUCTION`
- etc.