# Cowork Forge - Architecture Context

## Project Overview

**Cowork Forge** is an AI-native multi-agent software development platform. It orchestrates specialized AI agents through a 7-stage pipeline to transform ideas into production-ready software.

| Aspect | Detail |
|--------|--------|
| Language | Rust (edition 2024) |
| Agent Framework | adk-rust 0.5.0 |
| GUI | Tauri + React 18 + Ant Design |
| Architecture | Hexagonal + DDD |
| License | MIT |

## Module Map

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

## Core Flows

### 7-Stage Pipeline

1. **Idea** - Transform raw ideas into structured concepts
2. **PRD** - Generate Product Requirements Documents
3. **Design** - Create system architecture and design
4. **Plan** - Develop implementation plans
5. **Coding** - Generate and refine code
6. **Check** - Validate and test implementations
7. **Delivery** - Package and deploy solutions

### Agent Orchestration

- Each stage uses specialized agents (Actor-Critic pattern)
- Agents self-iterate until quality thresholds are met
- Human-in-the-loop (HITL) for critical decisions

## System Boundaries

- **Workspace**: All file operations validated against workspace boundaries
- **LLM Integration**: Rate-limited (30 req/min) with global semaphore
- **External Agents**: ACP protocol for external agent integration
- **Persistence**: JSON-based storage for projects and iterations

## Key Patterns

| Pattern | Where | Purpose |
|---------|-------|---------|
| Actor-Critic | PRD, Design, Plan, Coding stages | Iterative self-refinement |
| Strategy | Stage trait implementations | Pluggable stage behavior |
| Template Method | Pipeline execution flow | Fixed stage sequence with hooks |
| Repository | Persistence stores | Abstract data access |
| Decorator | LLM rate limiting | Transparent cross-cutting concern |

## Tech Stack

- **Backend**: Rust with Tokio async runtime
- **Frontend**: React 18 + TypeScript + Ant Design
- **Desktop**: Tauri framework
- **Agent Framework**: adk-rust 0.5.0
- **Serialization**: serde with derive macros
- **Error Handling**: anyhow::Result (no unwrap in production)

## Security Model

- Path validation for all file operations
- Command sanitization (dangerous commands blocked)
- LLM rate limiting with concurrency control
- Workspace containment for file operations
- No secrets in code (API keys from config/env)