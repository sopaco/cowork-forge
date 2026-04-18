# Architecture — Cowork Forge

> How components fit together. Last updated: 2026-04-17.
>
> **Update this when:** New component added, responsibilities shift, data flow changes.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Presentation Layer                           │
│  ┌──────────────────┐              ┌──────────────────────────┐    │
│  │   cowork-cli     │              │   cowork-gui             │    │
│  │   (clap+dialoguer)│              │   (Tauri + React+AntD)   │    │
│  └────────┬─────────┘              └────────────┬─────────────┘    │
│           │ implements                          │ invokes           │
└───────────┼────────────────────────────────────┼───────────────────┘
            ▼                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Application Layer (cowork-core)                 │
│                                                                     │
│  ┌───────────────────────┐    ┌──────────────────────────────┐     │
│  │  Interaction Domain   │    │   Pipeline Domain            │     │
│  │  InteractiveBackend   │◄───│   7-Stage Orchestration      │     │
│  │  (CLI/GUI trait impl) │    │   Stage Executor             │     │
│  └───────────┬───────────┘    │   Flow Config                │     │
│              │                └──────────┬───────────────────┘     │
│              │ drives                     │ manages                 │
│              ▼                            ▼                         │
│  ┌───────────────────────┐    ┌──────────────────────────────┐     │
│  │  Domain Layer         │    │   Supporting Domains          │     │
│  │  Project (Aggregate)  │    │   Tools (40+ ADK+MCP)        │     │
│  │  Iteration (Entity)   │    │   Agents (adk-rust)          │     │
│  │  Memory (Aggregate)   │    │   Instructions (~2000 lines)  │     │
│  └───────────┬───────────┘    │   Skills (agentskills.io)    │     │
│              │ persists        │   ACP (External Agent)       │     │
│              ▼                └──────────────────────────────┘     │
│  ┌───────────────────────────────────────────────────────────┐     │
│  │  Infrastructure Layer                                     │     │
│  │  Persistence (JSON) │ LLM (Rate-Limited) │ Security       │     │
│  └───────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
            │                            │
            ▼                            ▼
     ┌────────────┐              ┌──────────────┐
     │ File System │              │ LLM Provider │
     │ (.cowork-v2)│              │ (OpenAI API) │
     └────────────┘              └──────────────┘
```

---

## Component Responsibilities

### cowork-core
- **Purpose:** Domain logic, pipeline orchestration, tools, agents, persistence
- **Entry:** `crates/cowork-core/src/lib.rs`

### cowork-cli
- **Purpose:** Command-line interface with HITL terminal interaction
- **Entry:** `crates/cowork-cli/src/main.rs`

### cowork-gui
- **Purpose:** Desktop GUI (Tauri + React + Ant Design) with real-time streaming
- **Entry:** `crates/cowork-gui/src-tauri/src/main.rs`

### Pipeline Domain
- **Purpose:** 7-stage workflow orchestration with Actor-Critic pattern
- **Entry:** `crates/cowork-core/src/pipeline/mod.rs`
- Key stages: idea → prd → design → plan → coding → check → delivery

### Domain Layer
- **Purpose:** Core entities: Project (aggregate root), Iteration, ProjectMemory
- **Entry:** `crates/cowork-core/src/domain/mod.rs`

### Tools Domain
- **Purpose:** 40+ secure ADK tools + MCP remote tools (file, data, HITL, memory, validation, deployment, legacy analysis)
- **Entry:** `crates/cowork-core/src/tools/mod.rs`

### Config Definition
- **Purpose:** Data-driven configuration system (agents, stages, flows, skills, integrations as JSON)
- **Entry:** `crates/cowork-core/src/config_definition/mod.rs`

### Interaction Domain
- **Purpose:** `InteractiveBackend` trait abstracting CLI/GUI interaction
- **Entry:** `crates/cowork-core/src/interaction/mod.rs`

### Skills Module
- **Purpose:** agentskills.io standard skill discovery, selection, and injection
- **Entry:** `crates/cowork-core/src/skills/`

### ACP Module
- **Purpose:** Agent Client Protocol for external coding agent integration (OpenCode, Codex, Claude CLI, etc.)
- **Entry:** `crates/cowork-core/src/acp/client.rs`

---

## Data Flow

1. User provides idea via CLI or GUI
2. Pipeline creates Iteration context (Genesis or Evolution)
3. Each Stage: Agent receives instructions → calls LLM → uses Tools → produces Artifacts
4. Critical stages (idea/prd/design/plan/coding) require HITL confirmation
5. Artifacts persisted as Markdown in `.cowork-v2/iterations/`
6. On completion: knowledge snapshot extracted into ProjectMemory
7. PM Agent available for post-delivery interaction

---

## Key Dependencies

| Package | Purpose | Version |
|---------|---------|---------|
| adk-rust | Agent framework, tool ecosystem, session management | 0.4.0 |
| tokio | Async runtime | 1 |
| anyhow | Error handling | 1 |
| serde/serde_json | Serialization | 1 |
| agent-client-protocol | ACP for external agents | 0.9 |
| clap | CLI argument parsing | 4 |

---

*This file describes component relationships. For implementation details, explore the source code.*
