# Constraints & Boundaries

## Security Constraints

### Path Validation
- All file operations MUST be within workspace boundaries
- Implementation: `runtime_security.rs`

### Blocked Commands
```
rm -rf, sudo, chmod 777, mkfs, dd, :(){ :|:& };:, > /dev/sda
```

### Allowed Command Prefixes
```
cargo, npm, pip, bun, yarn, python, node, git (read-only)
```

## Rate Limiting

| Resource | Limit |
|----------|-------|
| LLM API | 30 req/min, concurrency=1 |
| Retry on error | 3 attempts with backoff |

## HITL Constraints

### Confirmation Gates
- Critical stages: idea, prd, design, plan, coding
- Non-critical: check, delivery (auto-proceed)

### Limits
- Max feedback loops: 5 per stage
- Max stage retries: 3

### User Actions
| Action | Behavior |
|--------|----------|
| Pass | Continue to next stage |
| View Artifact | Open editor, return to options |
| Provide Feedback | Re-execute with feedback |
| Cancel | Pause iteration |

## Iteration Status Transitions

```
Draft → Running → [Paused → Running] → [Completed | Failed]
```

| Status | Valid Operations |
|--------|------------------|
| Draft | start, delete |
| Running | pause (internal) |
| Paused | continue, delete |
| Completed | create_evolution, view |
| Failed | retry, delete |

## Inheritance Rules

| Mode | Artifacts | Workspace | Start Stage |
|------|-----------|-----------|-------------|
| None | Fresh | Fresh | idea |
| Partial | Copy | Fresh | idea |
| Full | Copy | Copy | configurable |

## Pipeline Constraints

- Stages MUST execute in order (cannot skip ahead)
- Each stage MUST produce artifact before completion
- `GotoStageTool` can jump backward only

## Tool Constraints

| Constraint | Value |
|------------|-------|
| Max file size for read | 1MB |
| Max files in list | 100 |
| ReadFileWithLimitTool calls | 10 per agent execution |

---

## Interaction Layer

### InteractiveBackend Trait

`interaction/mod.rs`

```rust
trait InteractiveBackend: Send + Sync {
    async fn show_message(level, content);
    async fn send_streaming(content, agent_name, is_thinking);
    async fn send_tool_call(tool_name, arguments, agent_name);
    async fn request_input(prompt, options, initial_content) -> InputResponse;
    async fn show_progress(task_id, progress);
}
```

### Implementations

| Backend | Location | Features |
|---------|----------|----------|
| CliBackend | `interaction/cli.rs` | Terminal output, dialoguer prompts |
| TauriBackend | `interaction/tauri.rs` | Event-driven IPC, React frontend |

### HITL Flow

```
Stage generates artifact
    ↓
request_input([Pass, View, Feedback, Cancel])
    ↓
CLI: dialoguer prompt
GUI: emit event, wait on oneshot channel
```

---

## Error Handling

### Recoverable
- LLM timeout → retry with backoff
- Tool failure → retry up to 3 times

### Unrecoverable
- Invalid config → halt, require fix
- Corrupted storage → halt, manual intervention
- Security violation → halt, log

---

## Code Modification Rules

### Adding a Tool
1. Implement in `tools/*.rs`
2. Re-export in `tools/mod.rs`
3. Register in agent builder

### Adding a Stage
1. Implement Stage trait in `pipeline/stages/`
2. Register in `get_all_stages()` and `create_stage_by_id()`
3. Add to `is_critical_stage()` if HITL needed

### Modifying HITL
1. Update `InteractiveBackend` trait if new methods
2. Implement in CliBackend and TauriBackend
3. Use in `pipeline/executor/interaction_ext.rs`