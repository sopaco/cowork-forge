# Cowork Forge - Agent Context

> AGENTS.md is the README for AI coding agents. This file provides project context and instructions.

---

## Project Overview

**Cowork Forge** is an AI-native iterative software development platform that orchestrates autonomous multi-agent workflows through a 7-stage development pipeline.

### Technology Stack
- **Language**: Rust (edition 2024)
- **Agent Framework**: adk-rust
- **GUI**: Tauri + React + Ant Design
- **Architecture**: Hexagonal + DDD

### Key Directories
```
crates/
├── cowork-core/    # Domain logic, pipeline, tools (MAIN)
├── cowork-cli/     # CLI adapter
└── cowork-gui/     # Tauri + React GUI
```

---

## Setup Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run CLI
cargo run --package cowork-cli -- <command>

# Run GUI (development)
cd crates/cowork-gui && cargo tauri dev
```

---

## Context Files

This project uses `.ai-context/` for detailed AI-friendly documentation. Load based on task:

### For Coding Tasks
```
Read: .ai-context/project.snapshot
Read: .ai-context/core/modules.map
Read: .ai-context/core/constraints.md
Read: .ai-context/domains/tools.md (if working with tools)
Read: .ai-context/domains/pipeline.md (if working with pipeline)
```

### For Debugging
```
Read: .ai-context/project.snapshot
Read: .ai-context/core/modules.map
Read: .ai-context/prompts/debug-context.md
```

### For Code Review
```
Read: .ai-context/core/constraints.md
Read: .ai-context/domains/domain-logic.md
Read: .ai-context/prompts/review-context.md
```

---

## Code Style

### Error Handling
```rust
use anyhow::Result;

fn my_function() -> Result<()> {
    let data = std::fs::read_to_string("file.txt")?;
    Ok(())
}
```

### Async Traits
```rust
use async_trait::async_trait;

#[async_trait]
pub trait MyTrait: Send + Sync {
    async fn do_something(&self) -> Result<()>;
}
```

### Rules
1. Use `anyhow::Result` for error handling
2. No `unwrap()` in production code - use `?` or proper error types
3. Validate all file paths within workspace boundaries
4. Use `async_trait` for async trait methods
5. snake_case for functions, PascalCase for types

---

## Key Files

| Task | File |
|------|------|
| Pipeline execution | `crates/cowork-core/src/pipeline/executor/mod.rs` |
| Stage implementations | `crates/cowork-core/src/pipeline/stages/*.rs` |
| Tool implementations | `crates/cowork-core/src/tools/*.rs` |
| Domain entities | `crates/cowork-core/src/domain/*.rs` |
| HITL interface | `crates/cowork-core/src/interaction/mod.rs` |
| Agent builders | `crates/cowork-core/src/agents/mod.rs` |
| Instructions/Prompts | `crates/cowork-core/src/instructions/*.rs` |

---

## Testing Instructions

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p cowork-core

# Run specific test
cargo test test_create_genesis_iteration
```

---

## PR Instructions

1. Run `cargo test` before committing
2. Run `cargo clippy` for linting
3. Ensure all tests pass
4. Follow conventional commit format

---

## Context Files Index

| File | Purpose |
|------|---------|
| `.ai-context/project.snapshot` | Project structure overview |
| `.ai-context/core/modules.map` | Module dependencies, code locations |
| `.ai-context/core/constraints.md` | Security, rate limits, constraints |
| `.ai-context/domains/pipeline.md` | Pipeline orchestration |
| `.ai-context/domains/domain-logic.md` | DDD entities |
| `.ai-context/domains/tools.md` | Tool ecosystem |
| `.ai-context/domains/interaction.md` | User interaction abstraction |
| `.ai-context/domains/persistence.md` | Storage layer |
| `.ai-context/domains/agents.md` | Agent builders |
| `.ai-context/api/traits.md` | Key trait definitions |
| `.ai-context/prompts/coding-context.md` | Coding task template |
| `.ai-context/prompts/debug-context.md` | Debug template |
| `.ai-context/prompts/review-context.md` | Review template |
