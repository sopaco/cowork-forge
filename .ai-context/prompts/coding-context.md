# Coding Task Context Template

## Purpose
Pre-generated context for AI agents performing coding tasks on this project.

---

## Project Context

You are working on **Cowork Forge**, an AI-native iterative software development platform.

### Technology Stack
- **Language**: Rust (edition 2024)
- **Async Runtime**: Tokio
- **Agent Framework**: adk-rust (v0.4.0)
- **GUI Framework**: Tauri + React
- **Serialization**: serde / serde_json

### Architecture Pattern
- Hexagonal Architecture (Ports and Adapters)
- Domain-Driven Design (DDD)
- Event-Driven (GUI layer)

### Key Directories
```
crates/
├── cowork-core/    # Domain logic, pipeline, tools
├── cowork-cli/     # CLI adapter
└── cowork-gui/     # Tauri + React GUI
```

---

## Code Style Guidelines

### Rust Conventions
- Use `anyhow::Result` for error handling in application code
- Use `thiserror` for custom error types in domain
- Prefer `async_trait` for trait methods
- Document public APIs with doc comments (`///`)

### Naming Conventions
- **Crates**: `cowork-*` prefix
- **Modules**: lowercase, snake_case
- **Types**: PascalCase
- **Functions**: snake_case
- **Constants**: SCREAMING_SNAKE_CASE

### File Organization
- One module per file when module is substantial
- `mod.rs` for module root and re-exports
- Tests in same file with `#[cfg(test)]` module

---

## Common Task Patterns

### Adding a New Tool
1. Create tool struct in `crates/cowork-core/src/tools/*.rs`
2. Implement `adk_tool::Tool` trait
3. Re-export in `tools/mod.rs`
4. Register in relevant agent builder (`agents/mod.rs`)

### Adding a New Stage
1. Create stage struct in `crates/cowork-core/src/pipeline/stages/*.rs`
2. Implement `Stage` trait
3. Register in `get_all_stages()` and `create_stage_by_id()`
4. Add to `is_critical_stage()` if HITL needed

### Modifying HITL Flow
1. Update `InteractiveBackend` trait if new methods needed
2. Implement in `CliBackend` and `TauriBackend`
3. Use in `pipeline/executor/interaction_ext.rs`

### Adding Domain Entity
1. Define in `crates/cowork-core/src/domain/*.rs`
2. Add to `mod.rs` re-exports
3. Update persistence if new storage needed
4. Add tests for invariants

---

## Security Reminders

- All file operations must validate paths within workspace
- No shell commands outside allowed list
- Never expose API keys in logs or errors
- Validate all user input at system boundaries

---

## Quick Reference Files

| Need | File |
|------|------|
| Stage execution logic | `pipeline/executor/mod.rs` |
| Tool implementations | `tools/mod.rs` |
| Domain entities | `domain/mod.rs` |
| HITL interface | `interaction/mod.rs` |
| Agent builders | `agents/mod.rs` |
| Instructions/Prompts | `instructions/mod.rs` |

---

## Injection Points

The following sections are replaced at runtime:

- `{{TASK_DESCRIPTION}}`: Specific coding task
- `{{AFFECTED_FILES}}`: List of files to modify
- `{{CONSTRAINTS}}`: Task-specific constraints
- `{{CONTEXT_FILES}}`: Relevant file contents
