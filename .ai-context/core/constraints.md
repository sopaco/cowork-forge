# Constraints & Boundaries

## Security Constraints

| Constraint | Rule |
|------------|------|
| Path Validation | All file ops within workspace |
| Blocked Commands | rm -rf, sudo, chmod 777, mkfs, dd |
| Allowed Commands | cargo, npm, pip, bun, yarn, python, node, git (read-only) |

## Rate Limiting

| Resource | Limit |
|----------|-------|
| LLM API | 30 req/min, concurrency=1 |
| Retry | 3 attempts with backoff |

## HITL Gates

- Critical stages (prd, design, plan, coding): require confirmation
- Non-critical (idea, check, delivery): auto-proceed
- Max feedback loops: 5 per stage

| Action | Behavior |
|--------|----------|
| Pass | Continue |
| Feedback | Re-execute stage |
| Cancel | Pause iteration |

## Tool Limits

| Constraint | Value |
|------------|-------|
| Max file size | 1MB |
| Max files per list | 100 |

## Error Handling

| Type | Behavior |
|------|----------|
| LLM timeout | Retry with backoff |
| Tool failure | Retry up to 3 times |
| Invalid config | Halt, require fix |
| Security violation | Halt, log |

## Interaction Layer

| Backend | Location | Use Case |
|---------|----------|----------|
| CliBackend | `interaction/cli.rs` | Terminal + dialoguer |
| TauriBackend | `interaction/tauri.rs` | Event-driven IPC |

HITL Flow: `request_input([Pass, Feedback, Cancel])` → wait for response
