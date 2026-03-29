# Cowork Forge - Agent Context

> AGENTS.md is the README for AI coding agents. This file provides project context and instructions.

---

## Project Overview

**Cowork Forge** is an AI-native iterative software development platform with multi-agent orchestration.

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
cargo build
cargo test
cargo run --package cowork-cli -- <command>
cd crates/cowork-gui && cargo tauri dev
```

---

## Context Files

Load `.ai-context/` based on task type:

### For Coding Tasks
```
Read: .ai-context/project.snapshot
Read: .ai-context/core/modules.map
Read: .ai-context/core/constraints.md
Read: .ai-context/domains/tools.md (if working with tools)
Read: .ai-context/domains/pipeline.md (if working with pipeline)
Read: .ai-context/architecture-decisions.md
```

### For Debugging
```
Read: .ai-context/project.snapshot
Read: .ai-context/core/constraints.md
Read: .ai-context/architecture-decisions.md
```

---

## Code Style

- Use `anyhow::Result` for error handling
- No `unwrap()` in production code
- Use `async_trait` for async trait methods
- snake_case for functions, PascalCase for types

---

## Key Files

| Task | File |
|------|------|
| Pipeline execution | `crates/cowork-core/src/pipeline/executor/mod.rs` |
| Stage implementations | `crates/cowork-core/src/pipeline/stages/*.rs` |
| Tool implementations | `crates/cowork-core/src/tools/*.rs` |
| Domain entities | `crates/cowork-core/src/domain/*.rs` |
| HITL interface | `crates/cowork-core/src/interaction/mod.rs` |

---

## Testing

```bash
cargo test
cargo test -p cowork-core
```

---

## Updating AI Context

### When Agent Should Remind User

At start of coding session, check recent changes:
```bash
git diff --name-only HEAD~10
```

If changed files include:
- `crates/cowork-core/src/tools/*.rs` → Remind: "Tools changed, update tools.md?"
- `crates/cowork-core/src/pipeline/stages/*.rs` → Remind: "Stages changed, update pipeline.md?"
- `crates/cowork-core/src/domain/*.rs` → Remind: "Domain entities changed, update domain-logic.md?"

### Manual Update Command

When making significant changes, ask the agent to update context:

```
"Update .ai-context because I [added/changed] [specific thing]"
```

Examples:
- "Update .ai-context because I added a new tool"
- "Update .ai-context because I added a new stage"
- "Update .ai-context because I made an architecture decision"

The agent will read `manifest.yaml` for maintenance guide.

**No update needed for**: struct fields, function signatures, refactoring.

---

## Context Files Index

| File | Purpose |
|------|---------|
| `project.snapshot` | Project structure, storage, core concepts |
| `core/modules.map` | Module dependencies, navigation |
| `core/constraints.md` | Security, rate limits, HITL, interaction |
| `domains/pipeline.md` | Pipeline + Agents |
| `domains/domain-logic.md` | Core entities and relationships |
| `domains/tools.md` | Tool ecosystem |
| `architecture-decisions.md` | Non-obvious design decisions |
| `prompts/coding-context.md` | Code style and patterns |