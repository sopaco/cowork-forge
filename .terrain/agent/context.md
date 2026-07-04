---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是一个 AI-native 多智能体软件开发平台，通过 7 阶段流水线（Idea→PRD→Design→Plan→Coding→Check→Delivery）将自然语言想法转化为生产级软件。采用六边形架构 + DDD，核心领域逻辑零外部依赖。支持 CLI 自动化与 Tauri+React GUI 两种交互模式。关键约束：Actor-Critic 自检模式、HITL 人工验证门、30 req/min LLM 限流、工作区路径安全隔离。

## 架构设计

| 层 | 容器 | 职责 |
|----|------|------|
| **领域层** | `cowork-core` | 纯业务逻辑：Project/Iteration/Memory 聚合、7 阶段编排、工具系统、指令库 |
| **应用适配** | `cowork-cli` | clap + dialoguer 终端交互，实现 InteractiveBackend |
| **GUI 适配** | `cowork-gui` | Tauri 桌面壳 + React 18 + Ant Design，事件驱动 IPC |
| **基础设施** | `cowork-core` 内 | LLM 限流客户端、JSON 持久化、路径安全、MCP 集成 |

**关键模式**：
- **六边形架构**：`InteractiveBackend` trait 是核心入站端口，CLI/GUI 为其适配器
- **Actor-Critic**：PRD/Design/Plan/Coding 阶段使用双智能体（生成→批评→迭代）
- **模板方法**：流水线固定 7 阶段 + `Stage` trait 策略实现
- **事件驱动**：GUI 层 Tauri events 实现实时流式传输

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| **pipeline** | 7 阶段编排、Stage trait、阶段执行器 | `crates/cowork-core/src/pipeline/` |
| **domain** | Project/Iteration/Memory 聚合、Artifact 值对象 | `crates/cowork-core/src/domain/` |
| **tools** | 40+ ADK 工具（文件/数据/HITL/验证/部署/记忆）+ MCP | `crates/cowork-core/src/tools/` |
| **agents** | 智能体包装：迭代式助手、外部编码代理、遗留分析 | `crates/cowork-core/src/agents/` |
| **interaction** | InteractiveBackend trait（CLI/GUI 抽象） | `crates/cowork-core/src/interaction/` |
| **acp** | Agent Client Protocol 外部代理协议 | `crates/cowork-core/src/acp/` |
| **config_definition** | 数据驱动配置（智能体/阶段/流定义 + JSON schema） | `crates/cowork-core/src/config_definition/` |
| **instructions** | 智能体提示库（~2000 行） | `crates/cowork-core/src/instructions/` |
| **skills** | agentskills.io 标准技能系统 | `crates/cowork-core/src/skills/` |
| **integration** | 钩子管理器，外部集成扩展点 | `crates/cowork-core/src/integration/` |
| **persistence** | JSON 文件存储（Project/Iteration/Memory Store） | `crates/cowork-core/src/persistence/` |
| **llm** | LLM 客户端工厂 + 限流装饰器 | `crates/cowork-core/src/llm/` |

## 核心流程

### 1. 创世迭代（Genesis Iteration）
1. 用户通过 CLI (`cowork iter`) 或 GUI 提交想法 → `PipelineController` 初始化 PipelineContext
2. 依次执行 7 阶段：Idea → PRD → Design → Plan → Coding → Check → Delivery
3. 每阶段：Stage Executor 创建 ADK 智能体 → LLM 流式推理 → 工具调用 → 输出 artifact
4. Actor-Critic 阶段（PRD/Design/Plan/Coding）：Critic 智能体验证质量，不达标则重新生成
5. HITL 门：用户可 approve / edit（外部编辑器）/ feedback（带反馈重新生成）
6. 完成后：知识快照生成（决策/模式/技术栈）→ 持久化到 MemoryStore

### 2. 演进迭代（Evolution Iteration）
1. 分析变更描述 → 关键词匹配确定继承模式（Full/Partial/None）
2. 继承模式决定起始阶段（Idea/PRD/Design/Plan）和传递的内容（artifact + code / code only / fresh）
3. 加载基础迭代知识 → 从选定阶段恢复流水线 → 执行剩余阶段

### 3. PM Agent 后交付流程
1. Delivery 完成后状态设为 Completed → PM Agent 激活
2. 支持：返回早期阶段重执行（GotoStage）、创建新迭代、保存用户决策

## 技术选型

- **运行时**：Rust edition 2024, Tokio (full), async-trait
- **智能体框架**：adk-rust 1.0.0（adk-core / adk-model / adk-tool / adk-runner / adk-session / adk-skill）
- **LLM 集成**：OpenAI-compatible API, 限流装饰器（全局 Semaphore=1 + 2s 延迟 = 30 req/min）
- **序列化**：serde + serde_json
- **CLI**：clap 4 (derive) + dialoguer 0.12 + console 0.16
- **GUI**：Tauri 1.x + React 18 + Ant Design + Vite
- **持久化**：JSON 文件（无外部数据库）
- **安全**：路径验证（UNC 标准化 + 工作区边界）、命令消毒（黑名单）、运行时监控
- **外部扩展**：MCP（Model Context Protocol）HTTP 客户端 → Tavily / DeepWiki
- **包管理**：Cargo workspace（3 crates）、npm（GUI 前端）

## 系统边界

| 外部系统 | 通信方式 | 约束 |
|----------|----------|------|
| LLM Provider API | HTTP（adk-model OpenAI） | 30 req/min, concurrency=1 |
| 本地文件系统 | std::fs（经路径验证） | 仅限工作区 `.cowork-v2/` |
| Shell/命令执行 | std::process + 超时 30s | 阻塞命令黑名单 |
| 外部编辑器 | 系统默认编辑器（HITL 审阅） | 按需调用 |
| 开发服务器 | ProcessRunner（Vite 等） | GUI 层管理生命周期 |
| MCP 服务器 | HTTP（adk-tool MCP 客户端） | 配置注入，启动时自动连接 |

**信任边界**：
- 核心域（可信） ↔ 适配器（半可信） ↔ 外部 API/FS（不可信）
- 所有文件 IO 经过 `validate_path()` + `strip_unc_prefix()` 安全层
- 所有命令经过 `is_dangerous_command()` 消毒

## 代码映射索引

| 概念 | 位置 | 备注 |
|------|------|------|
| Project 聚合 | `crates/cowork-core/src/domain/project.rs` | Aggregate Root |
| Iteration 实体 | `crates/cowork-core/src/domain/iteration.rs` | 状态机（Draft→Running→Completed/Failed） |
| Memory 聚合 | `crates/cowork-core/src/domain/memory.rs` | 跨迭代知识 |
| PipelineController | `crates/cowork-core/src/pipeline/mod.rs` | 7 阶段编排入口 |
| StageExecutor | `crates/cowork-core/src/pipeline/stage_executor.rs` | ADK 桥接 |
| Stage trait + 实现 | `crates/cowork-core/src/pipeline/stages/` | 7 个策略 |
| InteractiveBackend | `crates/cowork-core/src/interaction/mod.rs` | 端口 trait |
| CLI 后端 | `crates/cowork-core/src/interaction/cli.rs` | dialoguer 实现 |
| Tauri 后端 | `crates/cowork-core/src/interaction/tauri.rs` | 事件驱动实现 |
| Tool trait + 实现 | `crates/cowork-core/src/tools/` | 40+ 工具 |
| LLM 限流器 | `crates/cowork-core/src/llm/rate_limiter.rs` | Semaphore + Delay |
| 配置注册表 | `crates/cowork-core/src/config_definition/registry.rs` | 数据驱动 |
| 路径安全 | `crates/cowork-core/src/runtime_security.rs` | UNC 标准化 + 工作区边界 |
| Tauri 命令 | `crates/cowork-gui/src-tauri/src/commands/` | 25+ JSON-RPC 端点 |
| GUI 前端面板 | `crates/cowork-gui/src/components/` | React + Ant Design 组件 |