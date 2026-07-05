---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是一个 AI 原生的多 Agent 软件开发平台，通过 7 阶段流水线（Idea→PRD→Design→Plan→Coding→Check→Delivery）将自然语言想法转化为可交付软件。系统内置 10+ 专业 AI Agent（产品经理、架构师、项目经理、工程师），每个关键角色采用 Actor-Critic 自优化模式。支持 CLI（clap+dialoguer）和 GUI（Tauri+React）双界面，通过六边形架构保持核心域纯净。核心约束：LLM 限流 30 req/min、工作区严格路径隔离、无 unwrap() 生产代码。

## 架构设计

| 层级 | 容器 | 职责 |
|------|------|------|
| **表示层** | `cowork-cli`（CLI）、`cowork-gui`（Tauri+React） | 用户交互，命令路由，实时流展示 |
| **应用层** | CLI/GUI Backend（InteractiveBackend 实现） | 适配核心域到具体界面，HITL 通道 |
| **域层** | `cowork-core` | 纯业务逻辑：聚合、流程编排、工具系统 |
| **基础设施** | Persistence（JSON 存储）、LLM Client、Security | 持久化、AI 集成、工作区安全 |

**核心架构模式**：
- **六边形架构**：`InteractiveBackend` 为入站端口，Repository/LLM Client 为出站端口
- **DDD**：`Project`/`Iteration`/`ProjectMemory` 聚合，三继承模式（None/Partial/Full）
- **Actor-Critic**：每阶段双 Agent 循环，Critic 用 `IncludeContents::None` 节省 token
- **事件驱动**（GUI）：Tauri 事件系统实现 Agent/Tool/Progress 实时流

**关键依赖链**：
```
CLI/GUI → InteractiveBackend → Pipeline Controller → Stage Executor → ADK Agent → LLM
                                                            ↓
                                                     Tools Domain → File System / Shell / Persistence
```

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| **Pipeline** | 7 阶段编排、Stage trait、Stage Executor | `crates/cowork-core/src/pipeline/` |
| **Domain** | Project、Iteration、Memory 聚合与值对象 | `crates/cowork-core/src/domain/` |
| **Tools** | 40+ ADK 工具（文件/数据/HITL/验证/部署/内存） | `crates/cowork-core/src/tools/` |
| **Agents** | 迭代助手、PM Agent、遗留项目分析器、外部编码 Agent | `crates/cowork-core/src/agents/` |
| **Instructions** | 各阶段 Actor/Critic 提示词（约 2000 行） | `crates/cowork-core/src/instructions/` |
| **Interaction** | InteractiveBackend trait（CLI/GUI 抽象层） | `crates/cowork-core/src/interaction/` |
| **Config Definition** | 数据驱动配置：Agent 定义、Flow、Stage 定义 | `crates/cowork-core/src/config_definition/` |
| **Persistence** | JSON 存储：ProjectStore、IterationStore、MemoryStore | `crates/cowork-core/src/persistence/` |
| **LLM** | 限流客户端工厂（30 req/min，concurrency=1） | `crates/cowork-core/src/llm/` |
| **ACP** | Agent Client Protocol，外部 Agent 集成 | `crates/cowork-core/src/acp/` |
| **Integration** | 钩子系统，外部集成管理器 | `crates/cowork-core/src/integration/` |
| **Skills** | agentskills.io 标准技能系统 | `crates/cowork-core/src/skills/` |

## 核心流程

### 1. Genesis 迭代创建
1. 用户通过 `cowork iter --project <name> <idea>` 触发
2. Pipeline Controller 创建 Draft 状态迭代，初始化工作区 (`.cowork-v2/iterations/{id}/`)
3. 按序执行 7 阶段，每阶段：Actor 生成 → Critic 审查 → HITL 确认（可选反馈循环）
4. 阶段产物保存为 markdown/代码文件到迭代工作区
5. 迭代完成后触发知识快照生成，更新 ProjectMemory

### 2. Evolution 迭代（增量演进）
1. 用户输入变更描述，`analyze_change_scope()` 做 NLP 关键词分析
2. 确定继承模式：Full（制品+代码）/ Partial（仅代码）/ None（全新）
3. `LoadBaseKnowledgeTool` 加载历史决策、模式、已知问题
4. 根据变更范围选择起始阶段（Idea/PRD/Design/Plan），跳过已完成阶段
5. 继承模式下复制前序工作区文件到新迭代

### 3. HITL 验证流
1. Agent 调用 HITL 工具（`ReviewAndEditContentTool` / `ReviewAndEditFileTool`）
2. InteractiveBackend 弹出确认界面：CLI 用 dialoguer，GUI 用 `input_request` 事件
3. 用户选择：Pass（继续）/ Edit（打开 `$EDITOR`）/ Feedback（文本反馈）
4. Feedback → Agent 用反馈上下文重新生成 → 再次请求审查
5. Edit → 编辑器保存后 Hash 比对检测变更

### 4. GUI 实时执行监控
1. React 前端调用 Tauri Command 启动迭代执行
2. Backend 异步执行 Pipeline，通过 `AppHandle.emit()` 发射事件流
3. 事件类型：`agent_event` / `agent_streaming` / `tool_call` / `tool_result` / `progress` / `input_request` / `project_log`
4. React 通过 `listen()` 接收事件，实时更新 UI 面板

## 技术选型

- **语言/运行时**：Rust（edition 2024）+ Tokio（async，features = ["full"]）
- **AI 编排**：adk-rust（Agent trait、LlmAgentBuilder、LoopAgent）
- **LLM 接口**：OpenAI 兼容 API（不限供应商）
- **CLI 框架**：clap（参数解析）+ dialoguer（交互式提示）
- **GUI 框架**：Tauri 2.x（桌面壳）+ React 18 + Ant Design（前端）
- **状态管理**：React hooks + zustand 风格 stores
- **持久化**：JSON + serde（`.cowork-v2/` 目录）
- **限流**：Tokio Semaphore（concurrency=1）+ 2s 固定延迟
- **安全**：路径验证（UNC 标准化）+ 危险命令拦截（`rm -rf`/`sudo` 等）
- **外部集成**：MCP HTTP 客户端（Tavily/DeepWiki）

## 系统边界

| 外部系统 | 连接方式 | 信任边界 | 约束 |
|----------|----------|----------|------|
| LLM Provider API | HTTP（OpenAI 兼容） | 低（输入输出均受限流控制） | 30 req/min，concurrency=1 |
| 本地文件系统 | std::fs（路径验证后） | 中（仅限工作区目录） | 禁止路径遍历、UNC 路径需标准化 |
| Shell 命令 | std::process::Command | 低（命令白名单验证） | 禁止危险命令，仅项目工作区内执行 |
| 外部编辑器 | `$EDITOR` 环境变量 | 中（文件 Hash 比对检测变更） | 仅编辑受控的临时文件 |
| 开发服务器 | ProcessRunner（子进程管理） | 低（端口/进程隔离） | 仅 Vite/Webpack 等已知服务器 |
| MCP 服务器 | MCP HTTP 协议 | 低（用户配置） | 通过 `config.toml` 的 `[mcp]` 段配置 |
| 外部 Agent（ACP） | Agent Client Protocol | 中（协议层隔离） | 仅编码阶段可调用 |

**存储边界**（`.cowork-v2/` 目录结构）：
- `project.json` → 项目元数据、技术栈、迭代摘要
- `iterations/{id}/` → 阶段制品（idea.md、prd.md、design.md、plan.md、代码文件）
- `memory.json` → 跨迭代架构决策、模式、已知问题
- `artifacts/` → AI Agent 生成的源码和文档

## 代码映射索引

| 概念 | 位置 | 说明 |
|------|------|------|
| Pipeline 入口 | `crates/cowork-core/src/pipeline/mod.rs` | PipelineController，阶段编排 |
| Stage trait | `crates/cowork-core/src/pipeline/stage_executor.rs` | 7 阶段统一执行框架 |
| 7 阶段实现 | `crates/cowork-core/src/pipeline/stages/` | idea/prd/design/plan/coding/check/delivery |
| 域实体 | `crates/cowork-core/src/domain/` | Project/Iteration/Memory 聚合 |
| 工具系统 | `crates/cowork-core/src/tools/` | 40+ ADK 工具（按类别拆分） |
| Agent 定义 | `crates/cowork-core/src/agents/` | 迭代助手、PM Agent、遗留分析器 |
| 提示词库 | `crates/cowork-core/src/instructions/` | 各阶段 Actor/Critic 指令 |
| 配置注册表 | `crates/cowork-core/src/config_definition/` | JSON 驱动的 Agent/Flow/Stage 定义 |
| 默认配置 | `crates/cowork-core/src/config_definition/default_configs/` | 14 个内置 Agent + 7 阶段 + 默认 Flow |
| InteractiveBackend | `crates/cowork-core/src/interaction/` | CLI/GUI 抽象端口 |
| CLI 实现 | `crates/cowork-core/src/interaction/cli.rs` | dialoguer 实现 |
| Tauri 后端 | `crates/cowork-gui/src-tauri/src/` | 命令处理、事件发射、进程管理 |
| React 前端 | `crates/cowork-gui/src/` | 8 面板布局（项目/迭代/编辑器/运行器等） |
| 限流器 | `crates/cowork-core/src/llm/rate_limiter.rs` | Semaphore + 延迟 |
| ACP 客户端 | `crates/cowork-core/src/acp/client.rs` | 外部 Agent 协议 |