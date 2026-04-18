# AGENTS.md — Cowork Forge

> This file provides AI coding agents with the context needed to work effectively on this project.
> For project knowledge (architecture, decisions, issues), see [`.ai-context/SKILL.md`](.ai-context/SKILL.md).

---

## Project Overview

**Cowork Forge** is an AI-native multi-agent software development platform. It orchestrates specialized AI agents (Product Manager, Architect, Project Manager, Engineer) through a 7-stage pipeline to transform ideas into production-ready software.

| Aspect | Detail |
|--------|--------|
| Language | Rust (edition 2024) |
| Agent Framework | adk-rust 0.4.0 |
| GUI | Tauri + React 18 + Ant Design |
| Architecture | Hexagonal + DDD |
| License | MIT |

### Workspace Structure

```
crates/
├── cowork-core/         # Domain logic, pipeline, tools, agents (MAIN crate)
│   └── src/
│       ├── pipeline/    # 7-stage orchestration & stage executor
│       ├── domain/      # Project, Iteration, Memory aggregates
│       ├── tools/       # 40+ ADK tools + MCP integration
│       ├── agents/      # Agent wrappers (iterative, PM, legacy analyzer)
│       ├── interaction/ # InteractiveBackend trait (CLI/GUI abstraction)
│       ├── acp/         # Agent Client Protocol for external agents
│       ├── config_definition/  # Data-driven config (agents, stages, flows)
│       ├── instructions/       # Agent prompt library
│       ├── skills/      # agentskills.io standard skill system
│       ├── integration/ # Hook manager for external integrations
│       └── persistence/ # JSON-based storage
├── cowork-cli/          # CLI adapter (clap + dialoguer)
└── cowork-gui/          # Tauri + React GUI
    ├── src-tauri/       # Rust backend (Tauri commands + events)
    └── src/             # React frontend (TypeScript + Ant Design)
```

---

## Dev Environment Setup

### Prerequisites

- **Rust** (edition 2024, stable toolchain)
- **Node.js** (for GUI frontend build)
- **LLM API Key** (OpenAI-compatible endpoint)

### Build

```bash
# Build entire workspace
cargo build

# Release build
cargo build --release

# Build GUI only (installs frontend deps automatically)
cd crates/cowork-gui && cargo tauri dev
```

### Run

```bash
# CLI
cargo run --package cowork-cli -- <command>

# GUI (development mode)
cd crates/cowork-gui && cargo tauri dev
```

### Configuration

Config file location:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\CoworkCreative\config.toml` |
| macOS | `~/Library/Application Support/CoworkCreative/config.toml` |
| Linux | `~/.config/CoworkCreative/config.toml` |

User-facing config directory:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\com.cowork-forge.app\config\` |
| macOS | `~/Library/Application Support/com.cowork-forge.app/config/` |
| Linux | `~/.config/com.cowork-forge.app/config/` |

---

## Build and Test Commands

```bash
# Run all tests
cargo test

# Test a specific crate
cargo test -p cowork-core

# Test a specific module
cargo test -p cowork-core pipeline

# Run with all features
cargo test --all-features

# Check compilation without building
cargo check

# Lint (if clippy configured)
cargo clippy
```

### GUI Frontend

```bash
cd crates/cowork-gui

# Install dependencies
npm install    # or: bun install

# Build frontend only
npm run build

# Development server
npm run dev
```

---

## Code Style and Conventions

### Rust

- **Error handling**: Always use `anyhow::Result`. Never use `unwrap()` in production code.
- **Async traits**: Use `async_trait` for async trait methods.
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types/traits.
- **Architecture**: Follow hexagonal architecture — domain logic has zero external dependencies. Infrastructure adapters implement domain ports.
- **Trait-based abstraction**: `InteractiveBackend` is the key port for CLI/GUI abstraction. All user interaction flows through this trait.
- **Serialization**: `serde` with derive macros for all domain entities.
- **Async runtime**: Tokio with `features = ["full"]`.

### TypeScript / React (GUI)

- Component-based architecture with Ant Design.
- Tauri commands for request-response, events for streaming.
- State management via React hooks.

### Key Patterns

| Pattern | Where | Purpose |
|---------|-------|---------|
| Actor-Critic | PRD, Design, Plan, Coding stages | Iterative self-refinement |
| Strategy | Stage trait implementations | Pluggable stage behavior |
| Template Method | Pipeline execution flow | Fixed stage sequence with hooks |
| Repository | Persistence stores | Abstract data access |
| Decorator | LLM rate limiting | Transparent cross-cutting concern |

---

## Key Files

When working on specific areas, start from these files:

| Area | Primary File | Related |
|------|-------------|---------|
| Pipeline execution | `crates/cowork-core/src/pipeline/executor/mod.rs` | `stage_executor.rs`, `knowledge.rs` |
| Stage implementations | `crates/cowork-core/src/pipeline/stages/*.rs` | 7 stage files: idea, prd, design, plan, coding, check, delivery |
| Tool implementations | `crates/cowork-core/src/tools/mod.rs` | `file_tools.rs`, `data_tools.rs`, `hitl_tools.rs`, `pm_tools.rs`, etc. |
| Domain entities | `crates/cowork-core/src/domain/mod.rs` | `project.rs`, `iteration.rs`, `memory.rs` |
| HITL interface | `crates/cowork-core/src/interaction/mod.rs` | `cli.rs`, `tauri.rs` |
| Agent configs | `crates/cowork-core/src/config_definition/` | `default_configs/*.json` |
| Agent prompts | `crates/cowork-core/src/instructions/*.rs` | 12 instruction modules |
| External agent | `crates/cowork-core/src/acp/client.rs` | ACP protocol client |
| Skill system | `crates/cowork-core/src/skills/` | agentskills.io standard |

---

## Security Considerations

- **Path validation**: All file operations are validated against workspace boundaries. Never bypass `validate_path()` checks.
- **Command sanitization**: Dangerous commands (`rm -rf`, `sudo`, etc.) are blocked. Do not circumvent the command whitelist.
- **LLM rate limiting**: Global semaphore (concurrency=1) + 2s delay = 30 req/min. Do not bypass rate limiting.
- **Workspace containment**: File tools must not access paths outside the project workspace.
- **No secrets in code**: API keys are loaded from `config.toml` or environment variables, never hardcoded.
- **Watchdog monitoring**: Agent behavior is monitored for objective deviation.

---

## Project Knowledge (.ai-context)

This project uses a tiered knowledge base in `.ai-context/` for architectural context that code alone cannot convey. **AGENTS.md tells you *how to work*; `.ai-context/` tells you *what the project is*.**

### When to Read `.ai-context`

| Situation | What to Read |
|-----------|-------------|
| Starting a new session | `.ai-context/references/PROJECT-ESSENCE.md` |
| Working across components | `.ai-context/references/ARCHITECTURE.md` |
| Changing established patterns | `.ai-context/references/DECISIONS.md` |
| Debugging unexpected behavior | `.ai-context/DYNAMICS.md` |
| Unsure *why* something is designed a way | `.ai-context/references/DECISIONS.md` |

### Session Start Protocol

```
1. Read this file (AGENTS.md)
2. Read .ai-context/references/PROJECT-ESSENCE.md
3. Scan .ai-context/DYNAMICS.md for active issues
4. Proceed with code exploration
```

### Knowledge Tiers

| Tier | File | Update Frequency |
|------|------|------------------|
| 0 | `references/PROJECT-ESSENCE.md` | Quarterly / Major version |
| 1 | `references/ARCHITECTURE.md` | Monthly / Sprint |
| 2 | `references/DECISIONS.md` | Per decision change |
| 3 | `DYNAMICS.md` | As needed |

Full entry point: [`.ai-context/SKILL.md`](.ai-context/SKILL.md)

### Updating `.ai-context`

When making significant changes, update the corresponding knowledge file:

| What Changed | Update |
|-------------|--------|
| New crate or major component | `.ai-context/references/ARCHITECTURE.md` |
| Architecture decision | `.ai-context/references/DECISIONS.md` |
| New active issue / constraint | `.ai-context/DYNAMICS.md` |
| Project scope change | `.ai-context/references/PROJECT-ESSENCE.md` |

Before updating, read `.ai-context/meta/MAINTENANCE.md` for writing guidelines.

**No update needed for**: struct fields, function signatures, refactoring, bug fixes.

---

## PR Guidelines

- Run `cargo test` and `cargo clippy` before committing.
- Ensure no `unwrap()` in production code paths.
- If you added a new tool or stage, update `.ai-context/references/ARCHITECTURE.md`.
- If you made a non-obvious design choice, add an ADR to `.ai-context/references/DECISIONS.md`.
- Commit messages: use conventional commits format (`feat:`, `fix:`, `refactor:`, etc.).

---

## Common Pitfalls

- **Don't bypass `InteractiveBackend`**: Never call CLI-specific functions (e.g., `dialoguer`) from `cowork-core`. All user interaction must go through the `InteractiveBackend` trait.
- **Don't ignore rate limiting**: LLM calls are serialized for a reason. Don't try to parallelize them.
- **Don't access files outside workspace**: The security layer validates all paths. If you need to access a new path, update the validation logic, don't bypass it.
- **Don't hardcode stage IDs**: Use `create_stage_by_id()` or flow configuration instead of string matching.
- **Don't use `unwrap()`**: Use `anyhow::Result` with proper error propagation (`?` operator or `.context()`).
- **Don't duplicate knowledge**: If information exists in `.ai-context/`, link to it rather than repeating it here.
