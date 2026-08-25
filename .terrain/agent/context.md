---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是 **AI 原生的多 Agent 软件开发平台**：内置 PM、架构师、项目经理、工程师等 10+ 角色 Agent，通过 **7 阶段流水线**（Idea→PRD→Design→Plan→Coding→Check→Delivery）将自然语言想法转化为可交付软件。消费方包括独立开发者（CLI 自动化）、技术团队（标准化迭代与记忆继承）、以及偏好可视化监督的 GUI 用户。

核心约束：**本地优先**（数据驻留本机）、**人在回路（HITL）** 关键门控、**Actor-Critic** 自优化、工作区沙箱与路径校验、LLM 全局限流（30 req/min）。双入口（CLI/GUI）共享 `cowork-core` 域逻辑，配置驱动 Agent/Stage/Flow。

## 架构设计

**六边形架构（端口-适配器）**：`cowork-core` 为纯域核心；`cowork-cli` 与 `cowork-gui` 为入站适配器；LLM、文件系统、Shell、外部 Agent 为出站适配器。

| 容器/层 | 职责 | 主要路径 |
|---------|------|----------|
| **cowork-core** | 域模型、流水线编排、ADK Agent、工具、持久化 | `crates/cowork-core/src/` |
| **cowork-cli** | clap 命令入口、终端 HITL | `crates/cowork-cli/src/` |
| **cowork-gui 前端** | React 交互、状态、流式展示 | `crates/cowork-gui/src/` |
| **cowork-gui 后端** | Tauri 命令/事件桥接、进程管理 | `crates/cowork-gui/src-tauri/src/` |
| **配置层** | JSON 驱动的 Agent/Stage/Flow 定义 | `crates/cowork-core/src/config_definition/` |
| **交互端口** | `InteractiveBackend` trait 抽象 CLI/GUI | `crates/cowork-core/src/interaction/` |

**依赖流向**：CLI/GUI → Pipeline Controller → StageExecutor → adk-rust（LoopAgent + LlmAgent）→ Tools → LLM/FS/Shell。GUI 采用 **invoke 请求 + emit 事件** 非对称通信。

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| **Domain** | Project/Iteration/Memory 聚合、继承模式、状态机 | `crates/cowork-core/src/domain/` |
| **Pipeline** | 7 阶段编排、Stage trait、StageExecutor | `crates/cowork-core/src/pipeline/` |
| **Tools** | 40+ ADK 工具（文件/工件/验证/控制/HITL） | `crates/cowork-core/src/tools/` |
| **Agents** | 外部编码 Agent、迭代助手、遗留分析器 | `crates/cowork-core/src/agents/` |
| **Instructions** | 各阶段 Actor/Critic 提示词库 | `crates/cowork-core/src/instructions/` |
| **Config Definition** | Agent/Stage/Flow 注册表与校验 | `crates/cowork-core/src/config_definition/` |
| **Persistence** | JSON 文件存储（项目/迭代/记忆） | `crates/cowork-core/src/persistence/` |
| **Interaction** | HITL 抽象（消息/确认/流式） | `crates/cowork-core/src/interaction/`、`cli.rs` |
| **ACP** | Agent Client Protocol 外部 Agent 集成 | `crates/cowork-core/src/acp/` |
| **Importer** | 遗留项目导入与分析 | `crates/cowork-core/src/importer/` |
| **Skills** | agentskills.io 标准技能管理 | `crates/cowork-core/src/skills/` |
| **GUI** | 桌面 UI、Runner、预览、配置面板 | `crates/cowork-gui/src/`、`src-tauri/` |

## 核心流程

### 1. 创世迭代（Genesis）

1. 用户通过 CLI/GUI 创建项目并提交想法
2. Pipeline Controller 初始化 `PipelineContext`，IterationStore 创建 Draft 迭代
3. 按 Flow 配置顺序执行 7 阶段：StageExecutor 构建 adk-rust Agent（Actor-Critic LoopAgent）
4. 每阶段产出工件（markdown/代码）写入 `.cowork-v2/iterations/{id}/`
5. HITL 门（Idea/PRD/Design/Plan/Coding）经 `InteractiveBackend` 等待用户通过/编辑/反馈
6. 完成后更新迭代状态，Knowledge Gen 提取跨迭代记忆

### 2. 演进迭代（Evolution）

1. 基于已完成迭代创建 Evolution 迭代，选择继承模式（None/Full/Partial）
2. 变更范围分析决定工件/代码继承范围
3. 流水线从指定阶段继续，Actor 可见前序 Critic 反馈（`IncludeContents::Default`）
4. Critic 通过工具加载 artifact 审查（`IncludeContents::None`），调用 `exit_loop` 或 `provide_feedback` 控制循环

### 3. 遗留项目导入

1. CLI/GUI 触发 import 命令
2. `legacy_project_analyzer` Agent 扫描代码库、检测技术栈
3. ArtifactGenerator 反向生成 PRD/Design 等文档
4. 注册为 Cowork 项目，纳入迭代管理体系

### 4. 外部编码 Agent（ACP）

1. Coding 阶段可选启用 `ExternalCodingAgent`（配置 `coding_agent.enabled`）
2. 通过 ACP 协议调用外部 CLI Agent（OpenCode、Claude CLI、Gemini CLI 等）
3. `CoworkClient` 实现 ACP 服务端，约束工作区路径
4. 流式消息经 `AgentMessage` 通道回传 GUI/CLI

## 技术选型

- **语言/运行时**：Rust 2024 edition、Tokio 异步
- **AI 编排**：adk-rust 1.0（LoopAgent、LlmAgentBuilder、Tool trait、EventActions）
- **LLM**：OpenAI 兼容 API（adk-model openai feature）、自定义 RateLimiter（信号量 + 2s 延迟）
- **外部 Agent**：agent-client-protocol 0.9（ACP）
- **CLI**：clap 4、dialoguer、colored
- **GUI**：Tauri 2、React 18、Ant Design、Vite、Zustand hooks
- **持久化**：JSON + serde 文件存储（无数据库）
- **配置**：TOML（`config.toml`）+ JSON 数据驱动 Agent/Flow
- **安全**：路径校验、命令白名单、工作区沙箱、`runtime_security.rs`
- **文档**：Litho docs（`litho.docs/`）、Terrain 知识资产（`.terrain/`）

## 系统边界

| 边界 | 类型 | 说明 |
|------|------|------|
| **LLM Provider API** | 外部出站 | OpenAI 兼容端点；30 req/min 限流；API Key 来自 config/env |
| **本地文件系统** | 信任域内 | 项目工作区、`.cowork-v2/` 迭代数据、用户 config 目录 |
| **Shell/子进程** | 受限出站 | 构建/测试/开发服务器；命令清理与白名单 |
| **MCP 服务器** | 可选出站 | Tavily 搜索、DeepWiki 文档等第三方能力扩展 |
| **外部编码 Agent** | 可选出站 | ACP 协议；工作区路径强制校验 |
| **Tauri 系统 API** | 平台集成 | 托盘、文件对话框、进程管理、静态预览服务器 |
| **用户** | 入站 | CLI 命令或 GUI 交互；HITL 确认/编辑/反馈 |

**信任边界**：`cowork-core` 不直接依赖 UI；所有用户交互经 `InteractiveBackend`。文件工具禁止访问工作区外路径。

**配置路径**：macOS `~/Library/Application Support/CoworkCreative/config.toml`；GUI 配置 `com.cowork-forge.app/config/`。

## 代码映射索引

| 概念 | 位置 | 备注 |
|------|------|------|
| 项目/迭代聚合 | `crates/cowork-core/src/domain/` | Project、Iteration、Memory、InheritanceMode |
| 流水线控制器 | `crates/cowork-core/src/pipeline/mod.rs` | PipelineContext、阶段转换 |
| 阶段执行器 | `crates/cowork-core/src/pipeline/stage_executor.rs` | 连接 Stage 与 adk-rust |
| 7 阶段实现 | `crates/cowork-core/src/pipeline/stages/` | idea/prd/design/plan/coding/check/delivery |
| 默认 Flow | `crates/cowork-core/src/config_definition/default_configs/flows/default.json` | 7 阶段顺序定义 |
| 内置 Agent 配置 | `crates/cowork-core/src/config_definition/default_configs/agents/built-in/` | Actor/Critic JSON |
| ADK 工具集 | `crates/cowork-core/src/tools/` | artifact/control/file/validation 等 |
| Actor-Critic 指令 | `crates/cowork-core/src/instructions/` | 各阶段 prompt 库 |
| HITL 交互端口 | `crates/cowork-core/src/interaction/cli.rs` | InteractiveBackend 实现 |
| 外部 Agent ACP | `crates/cowork-core/src/acp/client.rs` | CoworkClient、路径校验 |
| 外部编码封装 | `crates/cowork-core/src/agents/external_coding_agent.rs` | ACP 流式任务 |
| JSON 持久化 | `crates/cowork-core/src/persistence/` | ProjectStore、IterationStore、MemoryStore |
| CLI 命令 | `crates/cowork-cli/src/commands/` | init/iter/import/config 等 |
| GUI Tauri 命令 | `crates/cowork-gui/src-tauri/src/commands/` | runner/pm/preview/file 等 |
| GUI 前端组件 | `crates/cowork-gui/src/components/` | 项目/迭代/聊天/配置面板 |
| 运行时检测 | `crates/cowork-core/src/project_runtime.rs` | 技术栈/预览类型识别 |