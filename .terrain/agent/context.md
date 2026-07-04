---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是一个 **AI 原生的多 Agent 软件开发平台**，通过 7 阶段流水线（Idea→PRD→Design→Plan→Coding→Check→Delivery）将自然语言想法转化为可交付软件。系统内置 10+ 专业 AI Agent（产品经理、架构师、项目经理、工程师等），采用 Actor-Critic 自优化模式。核心约束：单用户本地执行、LLM 速率限制（30 req/min）、工作区路径隔离。提供 CLI（自动化）和 Tauri GUI（交互式）双界面，共享 `cowork-core` 领域内核。

## 架构设计

| 层级 | 容器 | 职责 |
|------|------|------|
| **Presentation** | `cowork-cli` (clap+dialoguer) / `cowork-gui` (Tauri+React) | 用户交互入口 |
| **Application** | CLI/Tauri InteractiveBackend 实现 + ProjectRunner | 用例编排 |
| **Domain** | `cowork-core` — Pipeline, Tools, Agents, Memory, Domain | 纯业务逻辑 |
| **Infrastructure** | Persistence (JSON Stores), LLM Integration (Rate-Limited), Security | 存储/AI/安全适配 |

**架构风格**：六边形架构（Ports & Adapters）+ DDD（Aggregate: Project, Iteration, ProjectMemory）。

**关键模式**：
- **Actor-Critic** — 各阶段先生成后自审（PRD/Design/Plan/Coding）
- **Template Method** — Pipeline 固定 7 阶段序列 + 各阶段 Strategy 实现
- **Repository** — `ProjectStore`/`IterationStore`/`MemoryStore` 抽象 JSON 持久化
- **Decorator** — LLM 速率限制（Semaphore+Delay）包裹 Client Factory
- **Event-Driven** — GUI 层 Tauri Backend 通过事件流推送 agent_event/tool_call/progress

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| **Pipeline** | 7 阶段编排、Stage Executor、HITL 门控 | `cowork-core/src/pipeline/` |
| **Domain** | Project/Iteration/Memory 聚合根、继承模式 | `cowork-core/src/domain/` |
| **Tools** | 40+ ADK 工具（文件/数据/HITL/内存/验证/部署/MCP） | `cowork-core/src/tools/` |
| **Agents** | IterativeAssistant、ExternalCodingAgent、LegacyAnalyzer | `cowork-core/src/agents/` |
| **Interaction** | InteractiveBackend trait（CLI/Tauri 双实现） | `cowork-core/src/interaction/` |
| **Persistence** | JSON 文件存储（ProjectStore/IterationStore/MemoryStore） | `cowork-core/src/persistence/` |
| **Config Definition** | 数据驱动配置（Agent/Stage/Flow/Skill/Integration 定义） | `cowork-core/src/config_definition/` |
| **Instructions** | Agent 提示词库（~2000 行） | `cowork-core/src/instructions/` |
| **ACP** | Agent Client Protocol（外部 Agent 集成） | `cowork-core/src/acp/` |
| **LLM** | OpenAI 兼容客户端 + 速率限制（Semaphore+2s delay） | `cowork-core/src/llm/` |
| **Skills** | agentskills.io 标准技能系统（加载/执行/管理） | `cowork-core/src/skills/` |
| **Integration** | Hook 管理器（外部集成回调） | `cowork-core/src/integration/` |
| **GUI Frontend** | React 18 + Ant Design（9 面板） | `cowork-gui/src/` |
| **GUI Backend** | Tauri Rust 命令层 | `cowork-gui/src-tauri/src/` |
| **CLI** | clap 命令路由 + dialoguer 交互 | `cowork-cli/src/` |

## 核心流程

### 1. Genesis 迭代创建
1. 用户输入想法 → PipelineController 初始化 PipelineContext
2. 顺序执行 7 个 Stage（Idea→Delivery），每阶段：Actor 生成制品 → Critic 审查 → HITL 门控（Pass/Edit/Feedback）
3. 完成 → 写入 IterationStore + 生成 Knowledge Snapshot → 更新 ProjectMemory

### 2. Evolution 迭代（增量继承）
1. 变更描述 → `analyze_change_scope()` 关键词匹配决定继承模式（Full/Partial/None）
2. 自动选择起始阶段（架构→Idea，加功能→Coding，修 Bug→Check）
3. 加载 Base Knowledge → 从选定阶段继续执行剩余 Pipeline

### 3. HITL 人工验证
1. Agent 调用 Review 工具 → `InteractiveBackend.request_confirmation()`
2. CLI：dialoguer 终端预览 → Pass/Edit（$EDITOR）/ Feedback
3. GUI：Tauri 事件 `input_request` → React Modal → Command 回复
4. Feedback 时：Agent 带批评上下文重新生成 → 再审查

### 4. GUI 实时执行与监控
1. React Invoke Tauri Command → Backend 异步启动 IterationExecutor
2. Core 通过 AppHandle 发射事件流（agent_event, agent_streaming, tool_call 等）
3. React `listen()` 订阅 → 实时更新 UI
4. ProjectRunner 异步读取 dev server 日志 → `project_log` 事件

## 技术选型

- **语言**：Rust (edition 2024) + TypeScript (GUI)
- **异步**：Tokio (features = full)
- **Agent 框架**：adk-rust
- **LLM**：OpenAI-compatible API（30 req/min, 并发=1）
- **CLI**：clap (derive) + dialoguer + console
- **GUI 桌面**：Tauri 1.x + React 18 + Ant Design + Vite
- **序列化**：serde + serde_json
- **错误处理**：anyhow::Result（禁止 unwrap）
- **外部协议**：ACP + MCP

## 系统边界

| 外部系统 | 方向 | 信任边界 |
|----------|------|----------|
| LLM Provider API | 出站 HTTPS | 低 — 30rpm 限速 |
| 本地文件系统（`.cowork-v2/`） | 双向 | 中 — 路径验证+隔离 |
| Shell 命令 | 出站 | 低 — 命令白名单 |
| 系统编辑器 | 出站 | 中 — 仅 HITL 编辑 |
| MCP Servers (Tavily/DeepWiki) | 出站 HTTP | 低 — 远程查询 |
| 开发服务器 (Vite/Webpack) | 出站 ProcessRunner | 中 — 异步管道 |

**不处理**：Git、包管理器、CI/CD、云部署、多用户协作。

## 代码映射索引

| 概念 | 位置 | 备注 |
|------|------|------|
| PipelineController | `cowork-core/src/pipeline/mod.rs` | 7 阶段编排入口 |
| Stage Executor | `cowork-core/src/pipeline/stage_executor.rs` | 桥接 adk-rust |
| Stage 实现 | `cowork-core/src/pipeline/stages/` | 7 个文件 |
| Project | `cowork-core/src/domain/project.rs` | 聚合根 |
| Iteration | `cowork-core/src/domain/iteration.rs` | 继承逻辑 |
| ProjectMemory | `cowork-core/src/domain/memory.rs` | 决策/模式 |
| InteractiveBackend | `cowork-core/src/interaction/mod.rs` | CLI/GUI 端口 |
| CLI impl | `cowork-core/src/interaction/cli.rs` | dialoguer |
| Tauri impl | `cowork-core/src/interaction/tauri.rs` | oneshot 通道 |
| Config Registry | `cowork-core/src/config_definition/registry.rs` | 全局注册 |
| Rate Limiter | `cowork-core/src/llm/rate_limiter.rs` | Semaphore+2s |
| File Tools | `cowork-core/src/tools/file_tools.rs` | 路径验证 |
| ACP Client | `cowork-core/src/acp/client.rs` | 外部 Agent |
| Tauri Commands | `cowork-gui/src-tauri/src/commands/mod.rs` | IPC 处理 |
| Tauri Events | `cowork-gui/src-tauri/src/project_manager.rs` | 事件发射 |
| GUI Types | `cowork-gui/src/types/` | TS 类型定义 |