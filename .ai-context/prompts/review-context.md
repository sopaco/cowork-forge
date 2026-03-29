# Code Review Context Template

## Purpose
Pre-generated context for AI agents reviewing code changes in this project.

---

## Project Context

You are reviewing code for **Cowork Forge**, an AI-native iterative software development platform.

### Architecture Principles
1. **Hexagonal Architecture**: Domain logic isolated from infrastructure
2. **DDD**: Aggregates enforce consistency boundaries
3. **Ports & Adapters**: `InteractiveBackend` trait pattern
4. **Event-Driven**: Tauri events for GUI updates

### Key Design Patterns
- **Actor-Critic**: Dual-agent validation for critical stages
- **Repository Pattern**: Store traits abstract persistence
- **Strategy Pattern**: Stage implementations
- **Template Method**: Pipeline execution flow

---

## Review Checklist

### Security Review
- [ ] File paths validated within workspace
- [ ] No hardcoded credentials or API keys
- [ ] Shell commands sanitized
- [ ] User input validated at boundaries
- [ ] No SQL injection vectors (if DB added)

### Architecture Review
- [ ] Domain logic remains in `domain/` module
- [ ] No infrastructure dependencies in domain
- [ ] Traits defined in domain, implemented in infrastructure
- [ ] Proper separation: CLI/GUI adapters use core only

### Code Quality Review
- [ ] Error handling uses `anyhow::Result` or domain errors
- [ ] Public APIs documented with doc comments
- [ ] No unwraps in production code (use `?` or proper error)
- [ ] Async functions use `async_trait` where needed
- [ ] Tests added for new logic

### Performance Review
- [ ] No blocking calls in async context
- [ ] Large operations use streaming where possible
- [ ] No unnecessary clones (use references)
- [ ] Rate limiting respected for external calls

---

## Common Anti-Patterns to Flag

### 1. Domain Contamination
```rust
// BAD: Domain depends on infrastructure
use crate::persistence::ProjectStore;
impl Project { ... }

// GOOD: Domain is pure, store injects
impl Project {
    pub fn new(name: &str) -> Self { ... }
}
```

### 2. Blocking in Async
```rust
// BAD: Blocks async runtime
std::thread::sleep(Duration::from_secs(1));

// GOOD: Async sleep
tokio::time::sleep(Duration::from_secs(1)).await;
```

### 3. Unwrap in Production
```rust
// BAD: Can panic
let value = some_option.unwrap();

// GOOD: Proper error handling
let value = some_option.ok_or_else(|| anyhow::anyhow!("Expected value"))?;
```

### 4. Path Without Validation
```rust
// BAD: No workspace check
std::fs::read_to_string(path)?;

// GOOD: Validate first
RuntimeSecurityChecker::validate_path(&path)?;
std::fs::read_to_string(path)?;
```

### 5. Missing Tool Registration
```rust
// BAD: Tool created but not registered
let my_tool = Arc::new(MyNewTool);
// Forgot to add to agent builder!

// GOOD: Register in agent
.agent_tool(my_tool)
```

---

## Module-Specific Review Notes

### `domain/` Module
- Entities should have clear identity
- Value objects should be immutable
- Aggregates should enforce invariants
- No external dependencies

### `pipeline/` Module
- Stages should implement `Stage` trait
- Use `PipelineContext` for shared state
- Return `StageResult` appropriately
- HITL only on critical stages

### `tools/` Module
- Implement `adk_tool::Tool` trait
- Use parameter helpers for JSON extraction
- Notify tool calls/results for GUI
- Validate security constraints

### `interaction/` Module
- Keep trait pure (no implementation details)
- Implementations handle UI specifics
- Use `MessageContext` for rich metadata
- Async methods throughout

### `agents/` Module
- Agent builders should be pure functions
- Register all needed tools
- Use `max_iterations=1` for Actor-Critic loops
- Handle iteration ID in tool context

---

## Test Requirements

### New Domain Logic
- Unit tests for entity behaviors
- Test status transitions
- Test invariants enforcement
- Test edge cases

### New Tools
- Test successful execution
- Test error handling
- Test security validation
- Test parameter extraction

### New Stages
- Test execute path
- Test execute_with_feedback path
- Test HITL integration
- Test artifact generation

---

## Injection Points

- `{{CHANGED_FILES}}`: List of modified files
- `{{DIFF}}`: Git diff of changes
- `{{REVIEW_FOCUS}}`: Specific areas to focus on
- `{{PREVIOUS_FEEDBACK}}`: Prior review comments
