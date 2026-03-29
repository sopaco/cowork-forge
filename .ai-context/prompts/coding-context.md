# Coding Task Context

## Code Style

### Rust Conventions
- Use `anyhow::Result` for error handling
- Use `thiserror` for custom error types in domain
- Prefer `async_trait` for trait methods
- snake_case functions, PascalCase types

### File Organization
- One module per file when substantial
- `mod.rs` for re-exports
- Tests in same file with `#[cfg(test)]`

---

## Common Task Patterns

### Adding a New Tool
1. Create in `tools/*.rs`
2. Implement `adk_tool::Tool` trait
3. Re-export in `tools/mod.rs`
4. Register in agent builder

### Adding a New Stage
1. Create in `pipeline/stages/*.rs`
2. Implement `Stage` trait
3. Register in `get_all_stages()` and `create_stage_by_id()`
4. Add to `is_critical_stage()` if HITL needed

### Modifying HITL
1. Update `InteractiveBackend` trait if needed
2. Implement in CliBackend and TauriBackend
3. Use in `pipeline/executor/interaction_ext.rs`

### Adding Domain Entity
1. Define in `domain/*.rs`
2. Add to `mod.rs` re-exports
3. Update persistence if new storage needed

---

## Security Reminders

- All file operations within workspace
- No shell commands outside allowed list
- Never expose API keys in logs

---

## Quick Reference

| Need | File |
|------|------|
| Stage execution | `pipeline/executor/mod.rs` |
| Tools | `tools/mod.rs` |
| Domain entities | `domain/mod.rs` |
| HITL | `interaction/mod.rs` |
| Instructions | `instructions/mod.rs` |