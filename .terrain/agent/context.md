---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是 **AI 原生的多 Agent 软件开发平台**：内置 PM、架构师、项目经理、工程师等角色，经 **7 阶段流水线**（Idea→PRD→Design→Plan→Coding→Check→Delivery）将想法变为可交付软件。面向独立开发者（CLI 自动化）、团队（跨迭代记忆与继承）、探索型用户（GUI + HITL）。核心约束：**本地优先**（数据驻留用户机器）、**六边形架构**（`cowork-core` 纯域逻辑）、**Actor-Critic 自优化**（adk-rust `LoopAgent`）、**人在回路门**（关键阶段确认）、**工作区沙箱**（路径与命令校验）。

## 架构设计

| 容器 | 职责 | 主要依赖 |
|------|------|----------|
| **cowork-core** | 域模型、流水线编排、Agent/工具、持久化、LLM 适配 | adk-rust、Tokio、serde |
| **cowork-cli** | CLI 入站适配器（命令解析、终端 HITL） | clap、dialoguer → `InteractiveBackend` |
| **cowork-gui** | 桌面 GUI：React 前端 + Tauri 后端 | Tauri invoke/emit → `InteractiveBackend` |

**分层（core 内部）**

- **Domain**：`Project` / `Iteration` / `ProjectMemory` 聚合与值对象
- **Pipeline**：`Stage` trait + `StageExecutor` + `PipelineExecutor` 编排
- **Agents & Tools**：adk-rust `LlmAgent` / `LoopAgent` + 40+ `Tool` 实现
- **Interaction**：`InteractiveBackend` 端口（消息、流式、HITL 确认）
- **Persistence**：`ProjectStore` / `IterationStore` / `MemoryStore`（JSON）
- **Config**：JSON 驱动的 agents / stages / flows 注册表

**关键模式**：六边形（端口/适配器）、DDD 聚合、Actor-Critic（Actor=`IncludeContents::Default`，Critic=`None`+artifact 工具加载）、Repository、LLM 限流装饰器（并发=1，~30 req/min）。

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| Domain | 项目/迭代/记忆聚合、继承模式 | `crates/cowork-core/src/domain/` |
| Pipeline | 7 阶段编排、阶段执行、工作区管理 | `crates/cowork-core/src/pipeline/` |
| StageExecutor | 连接 Pipeline 与 adk-rust，Actor-Critic 循环 | `crates/cowork-core/src/pipeline/stage_executor.rs` |
| Tools | 文件/制品/验证/控制/知识等 ADK 工具 | `crates/cowork-core/src/tools/` |
| Agents | PM、摘要、知识生成、外部编码 Agent 封装 | `crates/cowork-core/src/agents/` |
| Instructions | 各阶段 Agent 提示词库 | `crates/cowork-core/src/instructions/` |
| ConfigDefinition | Agent/Stage/Flow JSON 配置与注册 | `crates/cowork-core/src/config_definition/` |
| Interaction | `InteractiveBackend` trait 与 CLI 实现 | `crates/cowork-core/src/interaction/` |
| Persistence | JSON 存储：项目、迭代、记忆 | `crates/cowork-core/src/persistence/` |
| CLI | 命令入口：init/iter/import/config 等 | `crates/cowork-cli/src/` |
| GUI Frontend | React 面板、聊天、配置、预览 | `crates/cowork-gui/src/` |
| GUI Backend | Tauri 命令、事件流、进程/预览服务 | `crates/cowork-gui/src-tauri/src/` |

## 核心流程

### 1. Genesis 迭代（首次开发）

1. 用户通过 CLI/GUI 创建项目并启动迭代（`cowork iter` 或 GUI Runner）
2. `PipelineExecutor` 按 flow 配置依次执行 7 阶段（`default.json` 或 `rapid-prototype.json`）
3. 每阶段 `StageExecutor` 构建 Actor-Critic `LoopAgent`，流式调用 LLM 并执行工具
4. 关键阶段触发 HITL：用户通过/编辑/反馈 → 可能 `execute_with_feedback` 重试
5. 制品写入 `.cowork-v2/iterations/{id}/artifacts/`，迭代完成后生成知识快照

### 2. Evolution 迭代（增量演进）

1. 用户描述变更，系统分析关键词确定继承模式（None / Partial / Full）
2. 按变更范围选择起始阶段（Idea/PRD/Design/Plan）
3. 从前序迭代加载代码与/或制品，注入 `PipelineContext`
4. 执行剩余阶段，记忆系统跨迭代累积决策

### 3. 遗留项目导入

1. `cowork import` 或 GUI Import 触发 `importer` 模块
2. `project_analyzer` 检测技术栈与结构，`artifact_generator` 反向生成文档
3. 注册为 Cowork 项目，后续可走 Evolution 流程

### 4. 编码阶段外部 Agent（可选）

1. Coding 阶段可通过 ACP 协议调用外部编码 Agent（OpenCode、Claude CLI 等）
2. `acp/client.rs` 管理子进程通信；核心域通过 `external_coding_agent.rs` 集成

## 技术选型

- **语言/运行时**：Rust 2024 edition、Tokio 全特性异步
- **AI 编排**：adk-rust 1.0（`adk-core` / `adk-agent` / `adk-model` / `adk-tool` / `adk-runner` / `adk-session` / `adk-skill`）
- **LLM**：OpenAI 兼容 API（`adk-model` openai feature），全局限流装饰器
- **CLI**：clap 4、dialoguer、console
- **GUI**：Tauri 2、React 18、Ant Design、Vite
- **持久化**：JSON + serde（项目元数据、迭代状态、记忆条目）
- **配置**：TOML 用户配置 + JSON 内置 Agent/Flow 定义
- **Skills**：agentskills.io 标准（`skills/manager.rs`）
- **集成**：MCP 远程工具、ACP 外部 Agent、Hook 系统（`integration/`）
- **安全**：`runtime_security.rs` 路径校验、命令白名单、工作区边界

## 系统边界

| 边界 | 交互方式 | 信任/约束 |
|------|----------|-----------|
| **LLM 提供商** | OpenAI 兼容 HTTP API | API Key 来自 config/env；30 req/min 限流 |
| **本地文件系统** | 读写项目工作区、`.cowork-v2/` 元数据 | `validate_path()` 防路径穿越 |
| **Shell/子进程** | 构建、测试、lint、开发服务器 | 命令白名单；项目目录内执行 |
| **MCP 服务器** | Tavily、DeepWiki 等远程工具 | 可选集成；经 adk 工具层调用 |
| **ACP 外部 Agent** | 子进程 stdio 协议 | Coding 阶段可选；用户配置启用 |
| **系统编辑器** | 外部编辑器打开制品 | HITL 编辑路径 |
| **用户配置目录** | `~/Library/Application Support/CoworkCreative/` 等 | 平台相关路径；不入库 |

**数据布局**：项目根下 `.cowork-v2/iterations/{id}/` 存放 artifacts、session_history、workspace 代码。

## 代码映射索引

| 概念 | 位置 | 备注 |
|------|------|------|
| 工作区入口 | `Cargo.toml` | 三 crate：core、cli、gui-tauri |
| 域聚合根 | `crates/cowork-core/src/domain/project.rs` | Project + IterationSummary |
| 迭代实体 | `crates/cowork-core/src/domain/iteration.rs` | 状态机、继承模式 |
| 记忆系统 | `crates/cowork-core/src/domain/memory.rs` | 三范围查询（项目/迭代/Smart） |
| Stage trait | `crates/cowork-core/src/pipeline/stages/mod.rs` | 各阶段 `execute()` |
| 流水线执行器 | `crates/cowork-core/src/pipeline/executor/mod.rs` | 阶段转换、上下文 |
| 默认 7 阶段流 | `crates/cowork-core/src/config_definition/default_configs/flows/default.json` | idea→…→delivery |
| 内置 Agent 配置 | `crates/cowork-core/src/config_definition/default_configs/agents/built-in/` | actor/critic JSON |
| InteractiveBackend | `crates/cowork-core/src/interaction/mod.rs` | CLI/GUI 统一端口 |
| GUI Tauri 适配 | `crates/cowork-gui/src-tauri/src/lib.rs` | emit agent_event/streaming |
| CLI 命令 | `crates/cowork-cli/src/commands/` | iter/init/import 等 |
| ACP 客户端 | `crates/cowork-core/src/acp/client.rs` | 外部编码 Agent |
| 遗留导入 | `crates/cowork-core/src/importer/` | 分析与制品生成 |
| 运行时检测 | `crates/cowork-core/src/runtime_analyzer.rs` | 技术栈/预览类型 |
| 安全边界 | `crates/cowork-core/src/runtime_security.rs` | 路径与命令校验 |