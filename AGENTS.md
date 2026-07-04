# AGENTS.md — Cowork Forge

> This file provides AI coding agents with the context needed to work effectively on this project.

---

## Project Overview

**Cowork Forge** is an AI-native multi-agent software development platform. It orchestrates specialized AI agents (Product Manager, Architect, Project Manager, Engineer) through a 7-stage pipeline to transform ideas into production-ready software.

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

## Security Considerations

- **Path validation**: All file operations are validated against workspace boundaries. Never bypass `validate_path()` checks.
- **Command sanitization**: Dangerous commands (`rm -rf`, `sudo`, etc.) are blocked. Do not circumvent the command whitelist.
- **LLM rate limiting**: Global semaphore (concurrency=1) + 2s delay = 30 req/min. Do not bypass rate limiting.
- **Workspace containment**: File tools must not access paths outside the project workspace.
- **No secrets in code**: API keys are loaded from `config.toml` or environment variables, never hardcoded.
- **Watchdog monitoring**: Agent behavior is monitored for objective deviation.

---

## Common Pitfalls

- **Don't bypass `InteractiveBackend`**: Never call CLI-specific functions (e.g., `dialoguer`) from `cowork-core`. All user interaction must go through the `InteractiveBackend` trait.
- **Don't ignore rate limiting**: LLM calls are serialized for a reason. Don't try to parallelize them.
- **Don't access files outside workspace**: The security layer validates all paths. If you need to access a new path, update the validation logic, don't bypass it.
- **Don't hardcode stage IDs**: Use `create_stage_by_id()` or flow configuration instead of string matching.
- **Don't use `unwrap()`**: Use `anyhow::Result` with proper error propagation (`?` operator or `.context()`).

<!-- terrain:begin env-overview v3 -->
## AI 工程环境（Terrain）

本仓库由 Terrain 配置了 AI 工程环境。Coding Agent 请遵循以下约定：

- **知识资产**位于本仓库 **`.terrain/`**（Agent 友好的知识资产、人类友好的知识库、私域知识、源码索引；可随 Git 协作）
- **项目登记**在本地 `~/.terrain/registry.json`（仅记录仓库路径，不含知识正文）
- **Skills** 位于 `.agents/skills/`（由 Terrain 注入，可按需重新集成）
- **Agent 工具**约定在 `~/.terrain/bin/`（`rtk` / `codegraph` / `terrain`）；可选本地清单 `.terrain/env/agent-tools.json`（不入库）
- **无 Terrain 安装**时：RTK / CodeGraph 可降级为 `bunx` / `npx`（见 `rtk-skill`、`codegraph-skill`）
- **工作流**：先读知识 → 再查关系 → 最后读源码；shell 输出优先走 RTK
<!-- terrain:end env-overview -->

<!-- terrain:begin knowledge-guide v3 -->
## Terrain 知识资产

Coding Agent **必须先加载** `terrain-knowledge-skill`，并按其中分层策略查询 **`.terrain/`**（仓库内路径，非全局目录）。

| 层级 | 路径 | 何时使用 |
|------|------|----------|
| Agent 友好 | `.terrain/agent/context.md` | 模块划分、核心流程、系统边界 |
| 私域 | `.terrain/knowledge/` | 业务术语、内部框架/API/脚手架 |
| 人类友好 | `.terrain/human/` | Litho 人类友好的知识库（可选参考） |
| 源码 | `.terrain/agent/repomix.md`（见 `repomix-context-skill`） | 实现细节（本地索引，不入库） |
| 关系 | codegraph CLI（见 `codegraph-skill`） | 调用链、依赖关系、影响分析 |

**原则**：先宏观后微观；优先读已生成文档，再 grep 源码索引。

## 知识保鲜（必读）

1. 回答架构/模块问题前，读取 `.terrain/.meta/freshness.json`（或 `freshness` 工具输出）
2. `freshness_score < 70` 时：不得仅凭 `agent/context.md` 下结论，须用 `grep repomix` 或 `codegraph` 交叉验证
3. `freshness_score < 50` 时：宏观架构上下文不可信，以 repomix 源码切片为准
4. 发现矛盾时的优先级：**repomix 源码 > codegraph > agent/context.md > human/**
5. `knowledge/` 私域文档视为人为维护；若 `refs` 指向的源码路径已删除，应降权处理
<!-- terrain:end knowledge-guide -->

<!-- terrain:begin skills v2 -->
### 可用 Skills

| Skill | 用途 |
|-------|------|
| `terrain-knowledge-skill` | `.terrain/` 知识分层与查询顺序（先读） |
| `repomix-context-skill` | grep/读取 `repomix.md` 源码切片 |
| `codegraph-skill` | 符号关系；`~/.terrain/bin/codegraph` 或 `bunx codegraph` |
| `rtk-skill` | 冗长 shell 加 rtk 前缀；`~/.terrain/bin/rtk` 或 `bunx @terrain-ai/rtk` |

加载顺序建议：knowledge → codegraph / repomix → rtk（执行命令时）。
<!-- terrain:end skills -->

<!-- terrain:begin tools v2 -->
### 工具链

| 工具 | 约定路径 | 无 Terrain 时降级 |
|------|----------|-------------------|
| RTK | `~/.terrain/bin/rtk` | `bunx @terrain-ai/rtk` 或 `npx @terrain-ai/rtk` |
| CodeGraph | `~/.terrain/bin/codegraph` | `bunx codegraph` 或 `npx codegraph` |
| Terrain CLI | `~/.terrain/bin/terrain` | `bunx @terrain-ai/cli` 或 `npx @terrain-ai/cli` |
| 知识文件 | `.terrain/` 仓库内路径 | 直接 Read/Grep，无需 CLI |

| 场景 | 用法 |
|------|------|
| 架构、私域知识 | 加载 `terrain-knowledge-skill` |
| 源码片段 | `repomix-context-skill`；`<rtk> grep` 搜索 pack |
| 符号关系 | `codegraph-skill`；检查 `~/.terrain/bin/codegraph` 是否存在（见 codegraph-skill） |
| git/test/build | `rtk-skill`；检查 `~/.terrain/bin/rtk` 是否存在（见 rtk-skill） |
| ACP 知识查询 | `~/.terrain/bin/terrain tools …` |

### Agent 工具解析（必读）

**一律使用约定路径**（`~/.terrain/bin/…`、`.terrain/…`），**不要**写机器相关的绝对路径（如 `/Users/…` 或 `C:\Users\…`）。

Windows 上工具部署在 `%USERPROFILE%\.terrain\bin\`（Git Bash / PowerShell 7+ 中可写为 `~/.terrain/bin/`），二进制带 `.exe` 后缀。

1. 执行前检查工具是否存在 — 见 `rtk-skill` / `codegraph-skill` 中的跨平台检查表（**不要**在 Windows 上使用 Unix 专用的 `test -x`）
2. 存在 → 用 `~/.terrain/bin/<tool> …`（词首 `~` 在 bash/zsh/Git Bash/PowerShell 7+ 会展开）
3. 不存在 → RTK / CodeGraph 用上表 `bunx` / `npx` 降级；Terrain CLI 请用户通过桌面应用操作
4. 可选参考：`.terrain/env/agent-tools.json`（本地生成、不入库），内容与约定路径一致

**不要**把 manifest 里的 `~` 路径赋给变量再引号调用（`"$VAR"` 不会展开 `~`）。直接写 `~/.terrain/bin/rtk` 或选用 `bunx` 前缀。

### RTK 要点（必读 `rtk-skill`）

- **必须显式**加 rtk 前缀 — Terrain 不启用 `rtk init` 全局 hook
- 内置 Read/Grep 不会自动走 RTK — 大文件用 `<rtk> read`，搜索用 `<rtk> grep`

**注意**：不要运行 `codegraph install` 或 `rtk init`（已由 Terrain + Skills 配置）。
<!-- terrain:end tools -->
