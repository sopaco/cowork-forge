---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是 **AI 原生的多 Agent 软件开发平台**：内置 PM、架构师、项目经理、工程师等 10+ 角色 Agent，通过 **7 阶段流水线**（Idea→PRD→Design→Plan→Coding→Check→Delivery）将用户想法自动推进为可交付软件。消费方：独立开发者（CLI 自动化）、技术负责人（GUI 交互）、团队（跨迭代知识累积）。

核心约束：**本地优先**（数据存 `.cowork-v2/`）、**人在回路**（关键阶段 HITL 确认）、**Actor-Critic 自优化**（adk-rust LoopAgent）、**接口不可知**（域逻辑经 `InteractiveBackend` 端口适配 CLI/GUI）、**工作区沙箱**（路径/命令校验）、**LLM 限流**（并发=1，约 30 req/min）。

## 架构设计

| 容器 | 职责 | 依赖 |
|------|------|------|
| **cowork-core** | 域模型、流水线、Agent/Tool、持久化、安全 | adk-rust, Tokio, serde |
| **cowork-cli** | clap 命令路由，实现 `InteractiveBackend` | cowork-core |
| **cowork-gui** | Tauri 后端 + React 前端，命令/事件双通道 | cowork-core, Tauri, Ant Design |

**分层（六边形）**

| 层 | 模块 | 说明 |
|----|------|------|
| 表现层 | `cowork-cli/`, `cowork-gui/src/` | 用户入口；GUI 用 invoke + emit 流式事件 |
| 应用层 | `project_runtime.rs`, `src-tauri/project_runner.rs` | 项目/迭代生命周期、进程管理 |
| 域层 | `domain/`, `pipeline/`, `instructions/` | 聚合根、阶段编排、提示词库 |
| 基础设施 | `persistence/`, `llm/`, `tools/`, `acp/`, `integration/` | JSON 存储、LLM 适配、40+ Tool、外部 Agent/MCP |

**关键模式**：Actor-Critic（LoopAgent）、Strategy（Stage trait）、Template Method（固定 7 阶段 + hooks）、Repository（Store 抽象）、Decorator（LLM rate limiter）。

## 模块地图

| Module | Responsibility | Primary paths |
|--------|----------------|---------------|
| Pipeline | 7 阶段编排、上下文传递、阶段转换 | `crates/cowork-core/src/pipeline/` |
| Stage Executor | ADK Agent 构建/运行、流式输出、HITL 桥接 | `crates/cowork-core/src/pipeline/stage_executor.rs`, `executor/` |
| Domain | Project / Iteration / Memory 聚合与继承模式 | `crates/cowork-core/src/domain/` |
| Config Definition | 数据驱动 Agent/Stage/Flow JSON 注册与校验 | `crates/cowork-core/src/config_definition/` |
| Tools | 文件、Artifact、验证、HITL、内存、部署等 ADK Tool | `crates/cowork-core/src/tools/` |
| Agents | PM Agent、迭代助手、遗留分析、外部编码 Agent 包装 | `crates/cowork-core/src/agents/` |
| Instructions | 各阶段 Actor/Critic 提示词库 | `crates/cowork-core/src/instructions/` |
| Interaction | `InteractiveBackend` 端口（CLI/GUI 抽象） | `crates/cowork-core/src/interaction/` |
| Persistence | Project/Iteration/Memory JSON 存储 | `crates/cowork-core/src/persistence/` |
| Importer | 遗留项目导入与分析 | `crates/cowork-core/src/importer/` |
| CLI | init/iter/continue/import 等命令 | `crates/cowork-cli/src/commands/` |
| GUI | React 面板 + Tauri 命令/事件后端 | `crates/cowork-gui/src/`, `src-tauri/src/` |

## 核心流程

### 1. 创世迭代（Genesis）

1. 用户通过 CLI/GUI 创建项目并提交初始想法
2. `PipelineContext` 初始化，在 `.cowork-v2/iterations/{id}/` 创建工作区
3. 按 Flow 配置依次执行 7 阶段：每阶段 Stage Executor 创建 Actor-Critic LoopAgent
4. Actor（`IncludeContents::Default`）生成 Artifact；Critic（`IncludeContents::None`）经 Tool 加载 Artifact 审查
5. 关键阶段触发 HITL：用户通过/编辑/反馈 → 可能 `execute_with_feedback` 重试
6. 阶段工件持久化为 markdown；完成后生成 Memory 快照

### 2. 演进迭代（Evolution）

1. 基于已有迭代创建 Evolution，选择继承模式（None / Partial / Full）
2. 系统分析变更描述，决定起始阶段与继承范围（工件 vs 代码）
3. 加载前序 Memory 与 Artifact 摘要注入上下文
4. 从选定阶段恢复流水线，剩余阶段同 Genesis 流程

### 3. 遗留项目导入

1. 用户指定现有代码库路径
2. `LegacyProjectAnalyzer` 检测技术栈、结构，反向生成文档类 Artifact
3. 导入后纳入正常迭代管理（Genesis/Evolution）

### 4. 外部编码 Agent（ACP）

1. Coding 阶段可经 ACP 协议调用外部 Agent（OpenCode、Claude CLI 等）
2. `acp/client.rs` 管理会话；产出仍受工作区安全与 Artifact 规范约束

## 技术选型

- **语言/运行时**：Rust（edition 2024）、Tokio 全特性异步
- **AI 编排**：adk-rust（`LlmAgentBuilder`、`LoopAgent`、`Tool` trait）
- **LLM**：OpenAI 兼容 API；全局信号量 + 延迟限流
- **CLI**：clap + dialoguer
- **GUI**：Tauri 2、React 18、TypeScript、Ant Design、Vite
- **持久化**：serde + JSON 文件（无外部 DB）
- **配置**：TOML（`config.toml`）+ 内置 JSON 默认 Agent/Stage/Flow
- **扩展**：MCP 远程 Tool（Tavily、DeepWiki）；agentskills.io Skill 系统；Integration hooks
- **工程**：Cargo workspace 三 crate；anyhow 错误处理；async_trait

## 系统边界

| 边界 | 类型 | 交互方式 | 信任/约束 |
|------|------|----------|-----------|
| LLM 提供商 API | 外部 | HTTP OpenAI 兼容 | API Key 来自 config/env；限流 30/min |
| 本地文件系统 | 外部 | 读写项目与 `.cowork-v2/` | `runtime_security` 工作区路径校验 |
| Shell/子进程 | 外部 | 构建/测试/开发服务器 | 命令白名单；禁止危险指令 |
| 系统默认编辑器 | 外部 | HITL 内容编辑 | 用户本地进程 |
| MCP 服务器 | 第三方 | Tavily 搜索、DeepWiki 文档 | `[mcp]` 配置；启动时注入 Tool |
| ACP 外部 Agent | 第三方 | Coding 阶段编码委托 | 会话隔离；产出回写工作区 |
| 开发预览服务器 | 用户提供 | Vite 等 via ProcessRunner | 进程生命周期由 GUI 管理 |

**范围外**：Git 操作、云 CI/CD、包注册表、远程多人协作。**域内纯逻辑**：cowork-core 不依赖 UI/LLM 具体实现，经端口/适配器连接。

## 代码映射索引

| Concept | Location | Notes |
|---------|----------|-------|
| 7 阶段默认 Flow | `crates/cowork-core/src/config_definition/default_configs/flows/default.json` | idea→…→delivery |
| Stage 实现 | `crates/cowork-core/src/pipeline/stages/*.rs` | 每阶段 Strategy |
| Actor-Critic Agent JSON | `crates/cowork-core/src/config_definition/default_configs/agents/built-in/` | *_actor / *_critic |
| InteractiveBackend 端口 | `crates/cowork-core/src/interaction/mod.rs` | CLI/GUI 必须经此 trait |
| CLI 后端实现 | `crates/cowork-core/src/interaction/cli.rs` | dialoguer HITL |
| GUI Tauri 后端 | `crates/cowork-gui/src-tauri/src/lib.rs` | emit agent_event/streaming |
| 项目运行时入口 | `crates/cowork-core/src/project_runtime.rs` | 迭代启动/恢复 |
| 迭代工作区布局 | `.cowork-v2/iterations/{id}/` | Artifact + session_history |
| 继承模式枚举 | `crates/cowork-core/src/domain/iteration.rs` | None/Partial/Full |
| Memory 三范围查询 | `crates/cowork-core/src/domain/memory.rs` | 项目/迭代/合并 |
| LLM 限流装饰器 | `crates/cowork-core/src/llm/rate_limiter.rs` | 并发=1 |
| 路径/命令安全 | `crates/cowork-core/src/runtime_security.rs` | 工作区 containment |
| ACP 客户端 | `crates/cowork-core/src/acp/client.rs` | 外部编码 Agent |
| MCP Tool 初始化 | `crates/cowork-core/src/config_definition/registry.rs` | `initialize_mcp_toolsets` |
| CLI 命令入口 | `crates/cowork-cli/src/main.rs` | iter/init/continue 等 |
| GUI React 入口 | `crates/cowork-gui/src/App.tsx` | 面板 + 事件 hooks |