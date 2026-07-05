---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是一个 **AI 原生的多 Agent 软件开发平台**。它通过 7 阶段流水线（Idea → PRD → Design → Plan → Coding → Check → Delivery）将原始想法逐步转化为可交付软件。系统内置 10+ 专业 AI Agent（产品经理、架构师、项目经理、工程师等），采用 Actor-Critic 自优化模式。支持 Genesis/Evolution 迭代模式、遗留项目导入、外部 ACP Agent 集成（OpenCode/Gemini CLI/Claude CLI）和跨迭代记忆累积。架构约束：零外部依赖的六边形架构核心、所有用户交互通过 `InteractiveBackend` trait、全局 LLM 并发限制为 1。

---

## 架构设计

| 层 | 容器 | Responsibility |
|---|---|---|
| **核心域** | `cowork-core` | 领域逻辑、管道编排、工具系统、Agent 编排、持久化 — 零外部依赖 |
| **CLI 适配** | `cowork-cli` | clap + dialoguer 命令行界面；实现 `InteractiveBackend` |
| **GUI 适配** | `cowork-gui` (Tauri) | Rust 后端（Tauri commands/events）+ React/TypeScript 前端 (Ant Design) |

### 跨切关注点

- **安全层** (`runtime_security.rs`): 路径验证、危险命令拦截、工作区边界检查
- **LLM 速率限制** (`llm/rate_limiter.rs`): 全局 semaphore (concurrency=1) + 2s 延迟
- **管道监控** (`runtime_analyzer.rs`): Agent 行为监控与目标偏差检测

### 关键依赖方向

```
cli/gui → cowork-core (InteractiveBackend 实现)
cowork-core → LLM API (通过 adk-rust LlmAgent)
cowork-core ↔ 文件系统 (工作区工具/持久化)
cowork-core ↔ 外部 Agent (ACP 协议)
```

---

## 模块地图

| 模块 | Responsibility | 主要路径 |
|---|---|---|
| **pipeline** | 7 阶段编排 + stage executor + workspace/interaction/knowledge executors | `crates/cowork-core/src/pipeline/` |
| **domain** | 聚合根：Project、Iteration、Memory | `crates/cowork-core/src/domain/` |
| **tools** | 40+ ADK Tool trait 实现（artifact/control/data/deployment/file/knowledge/memory/pm/test/validation 等） | `crates/cowork-core/src/tools/` |
| **agents** | IterativeAssistant (Actor-Critic 循环器)、LegacyProjectAnalyzer、ExternalCodingAgent (ACP) | `crates/cowork-core/src/agents/` |
| **acp** | Agent Client Protocol 客户端 — 外部 Agent 通信 | `crates/cowork-core/src/acp/` |
| **config_definition** | 数据驱动定义（agents/stages/flows/integrations）+ registry + 校验 | `crates/cowork-core/src/config_definition/` |
| **instructions** | 各阶段 Agent system prompt 库（中文，含可用工具列表） | `crates/cowork-core/src/instructions/` |
| **persistence** | JSON 文件存储（IterationStore/ProjectStore/MemoryStore） | `crates/cowork-core/src/persistence/` |
| **interaction** | InteractiveBackend trait — 所有用户 I/O 的抽象端口 | `crates/cowork-core/src/interaction/` |
| **llm** | LLM 客户端封装 + 速率限制器 | `crates/cowork-core/src/llm/` |
| **integration** | Hook 管理器 — 外部集成扩展点 | `crates/cowork-core/src/integration/` |
| **skills** | agentskills.io 标准 skill 系统 | `crates/cowork-core/src/skills/` |

---

## 核心流程

### 1. 7 阶段开发流水线

```
Idea → PRD → Design → Plan → Coding → Check → Delivery
```

每阶段流程：
1. **阶段初始化**：加载阶段配置（instruction + 可用工具列表）并创建 `LlmAgent`
2. **Actor 执行**：Agent 使用工具（读/写 artifact、文件操作、知识查询）生成产出
3. **Critic 审查**（PRD/Design/Plan/Coding 阶段）：独立 Agent 加载 artifact 审查
   - 通过 → 调用 `exit_loop` 终止循环
   - 有问题 → Critic 反馈，Actor 下轮修正
   - 大问题 → `provide_feedback` + `escalate`，触发阶段重试
4. **人类确认**（HITL）：`InteractiveBackend::confirm()` 检查点
5. **产物持久化** + **记忆抽取**：写入 JSON 存储，提取决策/模式

### 2. Actor-Critic 自优化循环

1. LoopAgent 创建 `HistoryTrackingSession`，包裹父上下文
2. **Actor**（`IncludeContents::Default`）：每轮可见前一轮 Critic 反馈文字 → 修正产出
3. **Critic**（`IncludeContents::None`）：通过工具加载 artifact 审查，不依赖对话历史
4. 达到 `max_iterations` 未退出 → LoopAgent 正常结束 → Stage executor 根据历史决定重试

### 3. 迭代继承 (Evolution)

1. 用户选择迭代模式：Genesis（全新）或 Evolution（增量）
2. Evolution 选择继承模式：None / Full（继承全部代码）/ Partial（选择性继承）
3. 初始化时加载前次迭代的 artifact 作为上下文
4. 管道执行完毕后触发记忆抽取

### 4. 外部 Agent 集成 (ACP)

1. Pipeline 到达 Coding 阶段 → 检查是否配置了外部 Agent
2. ACP 客户端通过 SSE/stdio 连接外部 Agent（OpenCode/Gemini CLI/Claude CLI）
3. 发送编码任务 + 工作区上下文 → 接收工具调用/文件变更事件
4. 外部 Agent 完成后，进入 Check 阶段验证产出

---

## 技术选型

- **语言**: Rust (edition 2024, stable toolchain)
- **Agent 编排**: `adk-rust` — Agent trait、LlmAgentBuilder、LoopAgent、Tool trait
- **异步运行时**: Tokio (features = ["full"])
- **序列化**: serde (derive macros for all domain entities)
- **错误处理**: anyhow::Result（禁止 unwrap）
- **CLI**: clap（参数解析）+ dialoguer（交互提示）
- **GUI**: Tauri v2（Rust 后端）+ React/TypeScript + Ant Design（前端）
- **GUI 构建**: Vite + TypeScript
- **LLM 协议**: OpenAI-compatible API（从 config.toml/env 加载密钥）
- **持久化**: JSON 文件存储（无数据库依赖）

---

## 系统边界

| 边界 | 方向 | 协议/机制 | 安全约束 |
|---|---|---|---|
| **LLM API** | 出站 | HTTP (OpenAI-compatible) | 密钥从 config.toml/env 加载；全局速率限制 (30 req/min) |
| **文件系统** | 双向 | std::fs + 路径验证 | `validate_path()` 检查工作区边界；危险命令（rm -rf/sudo）拦截 |
| **外部 Agent** | 双向 | ACP 协议（SSE/stdio） | 通过 `external_coding_agent.rs` 桥接 |
| **MCP Server** | 双向 | MCP 协议 | tools/ 中有 MCP 集成点 |
| **用户（CLI）** | 交互 | stdin/stdout (dialoguer) | 通过 `InteractiveBackend` trait |
| **用户（GUI）** | 交互 | Tauri IPC (commands + events) | 通过 `InteractiveBackend` trait |
| **配置** | 读 | `config.toml`（OS 配置目录） | 不包含硬编码密钥 |
| **持久化存储** | 读/写 | JSON 文件（workspace 内 `.cowork/`） | 路径验证约束 |

---

## 代码映射索引

| Concept | Location | Notes |
|---|---|---|
| Pipeline orchestration | `crates/cowork-core/src/pipeline/mod.rs` | 7-stage sequencing |
| Stage executor | `crates/cowork-core/src/pipeline/stage_executor.rs` | Template Method pattern |
| Per-stage impls | `crates/cowork-core/src/pipeline/stages/` | 7 files: idea/prd/design/plan/coding/check/delivery |
| Domain aggregates | `crates/cowork-core/src/domain/` | project.rs, iteration.rs, memory.rs |
| Tool trait + impls | `crates/cowork-core/src/tools/` | 14+ tool modules |
| Iterative assistant | `crates/cowork-core/src/agents/iterative_assistant.rs` | Actor-Critic LoopAgent |
| External coding agent | `crates/cowork-core/src/agents/external_coding_agent.rs` | ACP protocol bridge |
| ACP client | `crates/cowork-core/src/acp/client.rs` | SSE/stdio connection |
| Config definitions | `crates/cowork-core/src/config_definition/` | agent_definition.rs, builtin.rs, flow_definition.rs, stage_definition.rs, registry.rs, validator.rs |
| Default configs | `crates/cowork-core/src/config_definition/default_configs/` | agents/ (14 JSONs), flows/, stages/ |
| Agent instructions | `crates/cowork-core/src/instructions/` | Per-stage + PM + summary + knowledge_gen |
| JSON persistence | `crates/cowork-core/src/persistence/` | project_store.rs, iteration_store.rs, memory_store.rs, iteration_data.rs |
| InteractiveBackend trait | `crates/cowork-core/src/interaction/mod.rs` | Port; impl in cli.rs |
| Rate limiter | `crates/cowork-core/src/llm/rate_limiter.rs` | Tokio semaphore |
| Security layer | `crates/cowork-core/src/runtime_security.rs` | Path validation, command blocklist |
| Project runtime | `crates/cowork-core/src/project_runtime.rs` | Lifecycle management |
| CLI commands | `crates/cowork-cli/src/commands/` | 10 command modules |
| GUI Rust backend | `crates/cowork-gui/src-tauri/src/` | commands/, iteration_commands.rs, project_manager.rs, project_runner.rs, static_server.rs |
| GUI React frontend | `crates/cowork-gui/src/` | components/, hooks/, stores/, types/, constants/ |
| Integration hooks | `crates/cowork-core/src/integration/` | hooks.rs, adapters.rs |