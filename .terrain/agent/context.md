---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是**本地优先**的 AI 原生多 Agent 软件开发平台：编排 PM、架构师、项目经理、工程师等 10+ 角色 Agent，经 **7 阶段流水线**（Idea→PRD→Design→Plan→Coding→Check→Delivery）将自然语言想法变为可交付软件。消费者为独立开发者、小团队与技术负责人，通过 **CLI**（自动化）或 **Tauri 桌面 GUI**（交互式 HITL）使用。核心约束：域逻辑集中于 `cowork-core`（六边形架构）；各阶段采用 **Actor-Critic + adk-rust LoopAgent**；迭代数据隔离于 `.cowork-v2/`；LLM 调用限流（约 30 次/分钟）；工作区路径沙箱校验。

## 架构设计

| 容器/层 | 职责 | 主要 crate/路径 |
|---------|------|-----------------|
| **表现层** | CLI 命令路由、React 桌面 UI | `cowork-cli/`, `cowork-gui/src/` |
| **适配层** | `InteractiveBackend` 实现、Tauri 命令/事件 | `cowork-core/src/interaction/`, `cowork-gui/src-tauri/` |
| **应用/编排层** | 7 阶段流程控制、阶段执行、项目运行器 | `cowork-core/src/pipeline/` |
| **域层** | Project/Iteration/Memory 聚合、继承模式 | `cowork-core/src/domain/` |
| **Agent 层** | ADK Agent 构建、外部编码 Agent、指令库 | `cowork-core/src/agents/`, `instructions/`, `config_definition/` |
| **工具层** | 40+ ADK Tool、MCP 远程工具、Skills | `cowork-core/src/tools/`, `skills/` |
| **基础设施** | JSON 持久化、限流 LLM、安全、ACP | `persistence/`, `llm/`, `runtime_security.rs`, `acp/` |

**依赖关系**：`cowork-cli` 与 `cowork-gui`（Tauri 后端）均依赖 `cowork-core`；GUI 前端经 invoke/emit 与 Rust 后端通信；核心不依赖 UI。AI 编排基于 **adk-rust**（`LlmAgentBuilder`、`LoopAgent`、`Tool` trait）。

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| Pipeline | 7 阶段编排、上下文传递、阶段转换 | `crates/cowork-core/src/pipeline/` |
| Stage Executor | 连接流程与 ADK，Actor-Critic 执行、HITL、流式输出 | `crates/cowork-core/src/pipeline/stage_executor.rs` |
| Domain | Project/Iteration/Memory 实体与继承逻辑 | `crates/cowork-core/src/domain/` |
| Config Definition | 数据驱动 Agent/Stage/Flow JSON 注册与校验 | `crates/cowork-core/src/config_definition/` |
| Tools | 文件、制品、验证、HITL、内存、部署等 ADK 工具 | `crates/cowork-core/src/tools/` |
| Interaction | `InteractiveBackend` 端口（CLI/GUI 适配） | `crates/cowork-core/src/interaction/` |
| Persistence | 项目/迭代/内存 JSON 存储 | `crates/cowork-core/src/persistence/` |
| LLM | OpenAI 兼容客户端、限流装饰器 | `crates/cowork-core/src/llm/` |
| ACP / External Agent | Agent Client Protocol，编码阶段外部 Agent | `crates/cowork-core/src/acp/`, `agents/external_coding_agent.rs` |
| Importer | 遗留项目导入与分析 | `crates/cowork-core/src/importer/` |
| CLI | init/iter/import 等命令入口 | `crates/cowork-cli/src/commands/` |
| GUI | React 面板 + Tauri 命令/事件桥接 | `crates/cowork-gui/src/`, `src-tauri/src/` |

## 核心流程

### 1. 创世迭代（Genesis）

1. 用户经 CLI/GUI 提交项目名与想法 → 创建 `Iteration`（草稿）并初始化 `PipelineContext`
2. 流程控制器按 Flow 配置（`default.json`）顺序执行 7 阶段
3. 每阶段：`StageExecutor` 加载 Stage/Agent JSON → 构建 `LoopAgent`(Actor+Critic) → 流式调用 LLM 与工具 → 持久化制品至 `.cowork-v2/iterations/{id}/artifacts/`
4. HITL 门：用户通过/编辑/反馈 → 可选带反馈重跑 Actor
5. 全部阶段完成 → 知识提取写入 Memory → 迭代标记完成

### 2. 阶段内 Actor-Critic（adk-rust）

1. **Actor**（`IncludeContents::Default`）：生成/更新阶段制品（PRD、设计、代码等）
2. **Critic**（`IncludeContents::None`）：经工具从磁盘加载制品审查，不看 Actor 完整对话历史
3. 通过 → Critic 调用 `exit_loop`（`escalate=true`）；小问题 → 文字反馈供下轮 Actor；大问题 → `provide_feedback` 持久化并触发 Stage 级重试
4. 达 `max_iterations` 未退出 → StageExecutor 根据历史决定后续动作

### 3. 演化迭代（Evolution）

1. 基于已有项目创建新迭代，选择继承模式（None/Full/Partial）
2. 继承分析确定代码与制品范围 → 工作区复制/链接
3. 从指定阶段或默认起点进入流水线，复用项目级 Memory

### 4. 遗留项目导入

1. `import` 命令扫描目标仓库 → `ProjectAnalyzer` 检测技术栈与结构
2. `LegacyProjectAnalyzer` Agent 反向工程文档与配置
3. 生成 Cowork 项目元数据与初始 Memory，纳入正常迭代管理

## 技术选型

- **语言/运行时**：Rust（edition 2024）、Tokio 异步
- **AI 编排**：adk-rust（`Agent`/`LoopAgent`/`LlmAgentBuilder`/`Tool`）
- **LLM**：OpenAI 兼容 API；自定义信号量限流（~30 req/min，并发=1）
- **CLI**：clap、dialoguer
- **GUI**：Tauri 2、React 18、TypeScript、Ant Design、Vite、Zustand
- **持久化**：JSON + serde（项目/迭代/制品/内存）
- **配置**：TOML 用户配置 + JSON 内置 Agent/Stage/Flow 定义
- **协议集成**：ACP（外部编码 Agent）、MCP（Tavily/DeepWiki 等远程工具）
- **Skills**：agentskills.io 标准（`skills/manager.rs`）
- **安全**：工作区路径校验、命令沙箱（`runtime_security.rs`）

## 系统边界

| 边界类型 | 系统内 | 系统外 / 信任边界 |
|----------|--------|-------------------|
| **LLM** | 限流客户端、流式解析 | 第三方 OpenAI 兼容端点（需 API Key，网络出站） |
| **存储** | `.cowork-v2/` 迭代工作区、用户配置目录 JSON | 不管理 Git、云数据库 |
| **文件/Shell** | 验证后读写项目工作区、测试/构建子进程 | 系统默认编辑器（HITL 外链）、用户 DevServer（Vite 等） |
| **外部 Agent** | ACP 客户端调用编码阶段 | OpenCode、Claude CLI、Gemini CLI 等独立进程 |
| **MCP** | 启动时注入远程 Tool 到 Agents | Tavily、DeepWiki 等 MCP Server |
| **包生态** | 检测技术栈、运行本地命令 | 不代理 npm/crates.io 注册表 |
| **协作/部署** | 交付报告与本地预览 | 无多用户实时协作、无内置 CI/CD/云部署 |

**数据 locality**：本地优先；项目与迭代数据存于用户机器文件系统。

## 代码映射索引

| 概念 | 位置（路径） | 备注 |
|------|--------------|------|
| 库入口与公开 API | `crates/cowork-core/src/lib.rs` | 重导出 domain、pipeline、tools 等 |
| 7 阶段 Flow 定义 | `crates/cowork-core/src/config_definition/default_configs/flows/default.json` | idea→…→delivery |
| 内置 Agent 配置 | `crates/cowork-core/src/config_definition/default_configs/agents/built-in/` | *_actor.json / *_critic.json |
| 阶段执行核心 | `crates/cowork-core/src/pipeline/stage_executor.rs` | ADK 桥接、HITL、反馈重试 |
| 流程编排器 | `crates/cowork-core/src/pipeline/executor/mod.rs` | PipelineContext、阶段转换 |
| 交互端口 | `crates/cowork-core/src/interaction/mod.rs` | `InteractiveBackend` trait |
| CLI 后端实现 | `crates/cowork-core/src/interaction/cli.rs` | 终端 HITL |
| GUI 后端实现 | `crates/cowork-gui/src-tauri/src/lib.rs` | `TauriBackend`，emit 事件流 |
| 项目/迭代存储 | `crates/cowork-core/src/persistence/project_store.rs`, `iteration_store.rs` | JSON 仓库 |
| 制品与迭代数据 | `crates/cowork-core/src/persistence/iteration_data.rs` | `.cowork-v2/iterations/` |
| 控制流工具 | `crates/cowork-core/src/tools/control_tools.rs` | exit_loop、provide_feedback、goto_stage |
| 外部编码 Agent | `crates/cowork-core/src/agents/external_coding_agent.rs` | Coding 阶段 ACP |
| ACP 客户端 | `crates/cowork-core/src/acp/client.rs` | Agent Client Protocol |
| Agent 指令库 | `crates/cowork-core/src/instructions/` | 各阶段 prompt 模块 |
| CLI 命令 | `crates/cowork-cli/src/commands/` | init, iter, import, continue 等 |
| GUI 运行控制 | `crates/cowork-gui/src-tauri/src/project_runner.rs` | 迭代执行与进程管理 |