# Constraints & Boundaries

## Security Constraints

### Path Validation
- **Rule**: All file operations MUST be validated within workspace boundaries
- **Implementation**: `runtime_security.rs` validates paths before operations
- **Error**: Operations outside workspace return error

### Workspace Containment
- **Rule**: No access to files outside project directory
- **Boundary**: `.cowork-v2/` directory is the storage root
- **Violation Handling**: Reject operation, log warning

### Command Sanitization
- **Blocked Commands**:
  ```
  rm -rf, sudo, chmod 777, mkfs, dd, :(){ :|:& };:, > /dev/sda
  ```
- **Allowed Prefixes**: `cargo`, `npm`, `pip`, `bun`, `yarn`, `python`, `node`, `git` (read-only)
- **Implementation**: Regex pattern matching in `runtime_security.rs`

## Rate Limiting

### LLM API Limits
- **Rate**: 30 requests per minute maximum
- **Concurrency**: Single request at a time (semaphore = 1)
- **Implementation**: Decorator pattern with 2-second delay
- **Location**: `llm/config.rs`

### Retry Behavior
- **On Rate Limit**: Wait and retry with exponential backoff
- **On Error**: 3 retries with increasing delay

## HITL Constraints

### Confirmation Gates
- **Critical Stages**: idea, prd, design, plan, coding
- **Non-Critical**: check, delivery (auto-proceed)

### Revision Limits
- **Max Feedback Loops**: 5 per stage
- **Max Stage Retries**: 3 attempts
- **Behavior on Limit Exceeded**: Force proceed with warning

### User Actions
| Action | Behavior |
|--------|----------|
| Pass | Continue to next stage |
| View Artifact | Open in editor, return to options |
| Provide Feedback | Re-execute stage with feedback |
| Cancel | Pause iteration, return error |

## Iteration Constraints

### Status Transitions
```
Draft → Running → [Paused → Running] → [Completed | Failed]
                      ↓
                   [Failed → Running (retry)]
```

### Valid Operations by Status
| Status | Valid Operations |
|--------|------------------|
| Draft | start, delete |
| Running | pause (internal) |
| Paused | continue, delete |
| Completed | create_evolution, view |
| Failed | retry, delete |

### Inheritance Rules
| Mode | Artifacts | Workspace | Start Stage |
|------|-----------|-----------|-------------|
| None | Fresh | Fresh | idea |
| Partial | Copy | Fresh | idea (default) |
| Full | Copy | Copy | configurable |

## Pipeline Constraints

### Stage Execution Order
- **Rule**: Stages MUST execute in order (cannot skip ahead)
- **Exception**: `GotoStageTool` can jump backward for re-execution
- **Jump Forward**: Not allowed

### Artifact Requirements
- **Rule**: Each stage MUST produce its artifact before completion
- **Validation**: Check artifact file exists before marking stage complete
- **Failure**: Retry stage if artifact missing

### Flow Configuration
- `stop_on_failure`: If true, halt pipeline on stage failure
- `memory_scope`: `project` or `iteration` for knowledge queries

## Data Constraints

### Iteration ID Format
```
iter-{number}-{timestamp}
Example: iter-1-1707892800
```

### Project ID Format
```
proj-{timestamp}
Example: proj-1707892800
```

### Storage Paths
```
.cowork-v2/
├── project.json          # Fixed name
├── memory.json           # Fixed name
└── iterations/
    └── iter-{n}-{ts}/
        ├── iteration.json    # Fixed name
        ├── memory.json       # Fixed name
        └── artifacts/
            ├── idea.md       # Fixed name
            ├── prd.md        # Fixed name
            ├── design.md     # Fixed name
            ├── plan.md       # Fixed name
            └── delivery_report.md  # Fixed name
```

## Memory Constraints

### Knowledge Types
| Type | Scope | Promotion |
|------|-------|-----------|
| Decision | Project | - |
| Pattern | Project | Learning → Pattern |
| Issue | Iteration → Project | - |
| Learning | Iteration | Learning → Pattern |
| Insight | Iteration | Insight → Decision |

### Memory Size Limits
- **Per Entry**: 10,000 characters max
- **Total Decisions**: 100 max (oldest dropped)
- **Total Patterns**: 50 max

## Tool Constraints

### Tool Call Limits
- `ReadFileWithLimitTool`: 10 calls max per agent execution
- `QueryMemoryTool`: No limit, but results paginated

### File Operation Limits
- **Max File Size**: 1MB for read operations
- **Max Files in List**: 100 per call
- **Write Validation**: Must be within workspace

## External Agent Constraints

### ACP Integration
- **Transport**: stdio or websocket
- **Timeout**: 5 minutes per task
- **Fallback**: Built-in adk-rust agent on failure

### Supported External Agents
```
opencode, iflow, codex, gemini, claude
```

## Error Handling Boundaries

### Recoverable Errors
- LLM API timeout → Retry with backoff
- Tool execution failure → Retry up to 3 times
- Stage failure → Continue or stop based on config

### Unrecoverable Errors
- Invalid configuration → Halt, require user fix
- Corrupted storage → Halt, require manual intervention
- Security violation → Halt, log and notify

## Code Modification Rules

### When modifying pipeline:
1. Ensure Stage trait is implemented
2. Register in `get_all_stages()` or `create_stage_by_id()`
3. Update `is_critical_stage()` if HITL needed

### When adding tools:
1. Implement adk-rust Tool trait
2. Add to appropriate category file
3. Re-export in `tools/mod.rs`
4. Register in agent builders

### When changing storage:
1. Ensure backward compatibility
2. Add migration logic if format changes
3. Update domain entities accordingly

## Testing Requirements

### Unit Tests Required For:
- All domain entities (status transitions, invariants)
- Stage execution (success/failure paths)
- Tool operations (file, data, memory)
- Security validation (path, command)

### Integration Tests Required For:
- Full iteration lifecycle
- Evolution with inheritance
- HITL flow (mocked backend)
