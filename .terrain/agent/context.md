# Cowork Forge — Agent Architecture Context

> AI-native multi-agent software development platform. Orchestrates specialized AI agents through a 7-stage pipeline (Idea → PRD → Design → Plan → Coding → Check → Delivery) to transform natural language ideas into production-ready software. Dual-interface (CLI + GUI), local-first, with Actor-Critic quality assurance and cross-iteration memory.

---

## 项目概览

Cowork Forge is an AI-native iterative software development platform that orchestrates autonomous multi-agent workflows. It replaces a full dev team (PM, architect, engineer, QA) with specialized AI agents running a deterministic 7-stage pipeline. Key constraints: local-first (no cloud dependency), rate-limited LLM access (30 req/min, concurrency=1), workspace-contained file operations, and Human-in-the-Loop (HITL) validation gates at critical stages. Supports Genesis (new) and Evolution (incremental) iterations with three inheritance modes (Full/Partial/None). External AI agents can integrate via ACP protocol. Dual delivery: CLI for automation, Tauri+React GUI for interactive use.

---

## 架构设计

**Pattern**: Hexagonal (Ports & Adapters) + Domain-Driven Design

| Layer | Container | Role |
|-------|-----------|------|
| **Presentation** | `cowork-cli` (clap+dialoguer) | Automation CLI |
| **Presentation** | `cowork-gui` (React 18 + Ant Design) | Desktop GUI (Tauri shell) |
| **Application** | `cowork-core::pipeline` | Stage orchestration, executor |
| **Domain** | `cowork-core::domain` | Pure business entities (Project, Iteration, Memory) |
| **Infrastructure** | `cowork-core::tools` | 30+ ADK tools (file, data, HITL, validation, deploy, memory) |
| **Infrastructure** | `cowork-core::llm` | Rate-limited LLM client (decorator pattern) |
| **Infrastructure** | `cowork-core::persistence` | JSON file stores (ProjectStore, IterationStore, MemoryStore) |
| **Cross-cutting** | `cowork-core::interaction` | InteractiveBackend trait (CLI/GUI bridge) |
| **Cross-cutting** | `cowork-core::config_definition` | Data-driven agent/stage/flow configuration |

Key patterns: Actor-Critic (per stage), Strategy (Stage trait), Template Method (pipeline execution), Repository (persistence), Decorator (rate limiting), Event-Driven (GUI IPC).

---

## 模块地图

| Module | Responsibility | Primary path |
|--------|---------------|--------------|
| **Pipeline Controller** | 7-stage orchestration, context state, transitions | `cowork-core/src/pipeline/mod.rs` |
| **Stage Executor** | Bridges pipeline ↔ ADK framework, agent lifecycle | `cowork-core/src/pipeline/stage_executor.rs` |
| **Domain Logic** | Project/Iteration aggregates, value objects, state machines | `cowork-core/src/domain/` |
| **Tools Domain** | 30+ ADK tools (file, artifact, data, HITL, validation, deploy, memory, PM) | `cowork-core/src/tools/` |
| **LLM Integration** | Rate-limited client factory (concurrency + temporal) | `cowork-core/src/llm/` |
| **Persistence** | JSON stores for projects, iterations, memory | `cowork-core/src/persistence/` |
| **Interaction** | InteractiveBackend trait (CLI/GUI abstraction) | `cowork-core/src/interaction/` |
| **Agent Definitions** | Built-in agent configs (actor/critic per stage) | `cowork-core/src/config_definition/default_configs/agents/built-in/` |
| **Instructions** | Agent prompt library per stage | `cowork-core/src/instructions/` |
| **ACP Client** | External agent integration protocol | `cowork-core/src/acp/` |
| **Memory System** | Cross-iteration knowledge (decisions, patterns, insights) | `cowork-core/src/domain/memory.rs` |
| **CLI Adapter** | Clap commands + dialoguer prompts | `cowork-cli/src/` |
| **GUI Backend** | Tauri commands, events, process runner | `cowork-gui/src-tauri/src/` |
| **GUI Frontend** | React panels, stores, hooks, Monaco editor | `cowork-gui/src/` |

---

## 核心流程

### 1. Genesis Iteration (new project)
1. User provides idea text → CLI/GUI creates `Project` aggregate + genesis `Iteration` (Draft state)
2. Pipeline Controller iterates 7 stages sequentially via Stage Executor
3. Per stage: Stage Executor creates LoopAgent (Actor → Critic) via ADK framework
4. Actor generates artifact (idea.md, prd.md, design.md, plan.md, code)
5. Critic validates via tool-loaded artifact review; passes → HITL gate; fails → regenerate
6. HITL gate: user passes/edits/provides feedback (triggers re-execution)
7. On completion: artifacts persisted to `.cowork-v2/iterations/{id}/`; knowledge snapshot generated

### 2. Evolution Iteration (incremental)
1. User provides change description → keyword analysis determines InheritanceMode (Full/Partial/None)
2. Stage selection heuristic: fundamental→Idea, requirements→PRD, architecture→Design, implementation→Plan
3. Load base knowledge from prior iteration(s); resume pipeline at determined entry stage
4. Same Actor-Critic execution model, with prior iteration artifacts as context

### 3. Actor-Critic Loop (per stage)
1. Actor agent (IncludeContents::Default) generates artifact using stage-specific tools
2. Critic agent (IncludeContents::None) loads artifact via tool (not history), validates quality
3. Critic approves → calls `exit_loop` (escalate=true) → stage success
4. Critic identifies issues → provides textual feedback in reply → Actor regenerates
5. Critic detects major problems → calls `provide_feedback` (escalate=true) → Stage executor handles retry
6. Max iterations reached without exit → stage completes; executor decides based on history

### 4. External Agent Coding (ACP)
1. ACP-enabled coding stage delegates to external agent (OpenCode, Gemini CLI, Claude CLI)
2. Workspace context packaged and sent via ACP protocol
3. External agent executes, returns results; results validated and integrated

---

## 技术选型

- **Language**: Rust (edition 2024, stable) — entire backend + CLI + GUI backend
- **Async Runtime**: Tokio (features = full)
- **Agent Framework**: adk-rust (LlmAgentBuilder, LoopAgent, Tool trait, Session)
- **LLM**: OpenAI-compatible APIs via adk-model
- **Rate Limiting**: Custom Semaphore(1) + 2s delay decorator (~30 req/min)
- **Serialization**: serde + serde_json
- **Error Handling**: anyhow (production code, no unwrap)
- **CLI**: clap v4 (derive) + dialoguer + tokio
- **GUI Backend**: Tauri v2 (Rust) — IPC (invoke/emit), process management
- **GUI Frontend**: React 18 + TypeScript + Ant Design 5 + Monaco Editor
- **GUI State**: Zustand stores (project, config, agent, UI)
- **Persistence**: JSON files under `.cowork-v2/` (project.json, iterations/, memory/)
- **Security**: path validation (UNC stripping, traversal detection, workspace boundary)
- **Build**: cargo workspace (3 crates: core, cli, gui); vite for frontend

---

## 系统边界

| Boundary | Direction | Interface | Details |
|----------|-----------|-----------|---------|
| **LLM Provider API** | Outbound | HTTP (OpenAI-compatible) | Rate-limited (30 req/min, concurrency=1); config via TOML/env |
| **Local File System** | Outbound | Filesystem I/O | Workspace-contained under `.cowork-v2/`; path traversal protection |
| **Shell/Command Executor** | Outbound | Process spawn | Sanitized commands; blocked dangerous ops (rm -rf, sudo); 30s timeout |
| **External Editor** | Outbound | OS default editor | Invoked during HITL file review flows |
| **Dev Server** | Outbound | HTTP process | User-provided (Vite, etc.); managed via ProcessRunner |
| **MCP Servers** | Outbound | HTTP | External tool queries (Tavily search, DeepWiki) via adk-tool |
| **External Coding Agent** | Outbound | ACP protocol | OpenCode/Gemini CLI/Claude CLI integration in coding stage |
| **Human User (CLI)** | Inbound | Terminal (stdin/stdout) | async HITL prompts via InteractiveBackend + dialoguer |
| **Human User (GUI)** | Inbound | Desktop events | Tauri invoke/listen; streaming events + modal confirmations |

**Trust boundaries**: LLM output is untrusted → validated by Critic agent + HITL gate before persistence. File operations validated against workspace root. Shell commands sanitized. No secrets in code — loaded from config.toml or env vars.

---

## 代码映射索引

| Concept | Location |
|---------|----------|
| Project aggregate | `cowork-core/src/domain/project.rs` |
| Iteration aggregate | `cowork-core/src/domain/iteration.rs` |
| Memory aggregate | `cowork-core/src/domain/memory.rs` |
| Pipeline controller | `cowork-core/src/pipeline/mod.rs` |
| Stage executor | `cowork-core/src/pipeline/stage_executor.rs` |
| Stage implementations (7) | `cowork-core/src/pipeline/stages/` |
| InteractiveBackend trait | `cowork-core/src/interaction/mod.rs` |
| CLI backend (CliBackend) | `cowork-core/src/interaction/cli.rs` |
| LLM rate limiter | `cowork-core/src/llm/rate_limiter.rs` |
| Tool registry & impls | `cowork-core/src/tools/` |
| ACP client | `cowork-core/src/acp/client.rs` |
| Agent definitions (JSON configs) | `cowork-core/src/config_definition/default_configs/agents/built-in/` |
| Flow & stage configs | `cowork-core/src/config_definition/default_configs/flows/`, `stages/` |
| Stage prompts | `cowork-core/src/instructions/` |
| Project store | `cowork-core/src/persistence/project_store.rs` |
| Iteration store | `cowork-core/src/persistence/iteration_store.rs` |
| Memory store | `cowork-core/src/persistence/memory_store.rs` |
| CLI commands | `cowork-cli/src/commands/` |
| Tauri command handlers | `cowork-gui/src-tauri/src/commands/` |
| GUI React panels | `cowork-gui/src/components/` |
| GUI React stores | `cowork-gui/src/stores/` |
| Runtime analyzer | `cowork-core/src/runtime_analyzer.rs` |
| Security validation | `cowork-core/src/runtime_security.rs` |
| Tech stack detection | `cowork-core/src/tech_stack.rs` |
| Skills manager | `cowork-core/src/skills/manager.rs` |
| Import/legacy project analyzer | `cowork-core/src/importer/` |
| Integration hooks | `cowork-core/src/integration/` |
