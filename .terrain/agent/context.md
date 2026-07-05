# Cowork Forge — Agent Architecture Context

> Generated: 2026-07-05 | Project version: 2.5.2 | Source: `.terrain/agent/context.md`

---

## 项目概览

Cowork Forge 是一个 **AI 原生的多 Agent 软件开发平台**，将自然语言想法通过 7 阶段流水线转化为可交付软件。系统内置 10+ 专业 AI Agent（产品经理、架构师、项目经理、工程师等），采用 Actor-Critic 自优化模式，在关键节点设置 Human-in-the-Loop 验证门。核心价值：**一个人拥有完整虚拟开发团队**。运行在本地桌面环境，通过 OpenAI 兼容 API 驱动 LLM 推理，无外部数据库依赖。支持 Genesis（首次）和 Evolution（演化）两种迭代模式，具备跨迭代记忆累积能力。提供 CLI 自动化与 Tauri GUI 交互双界面。

---

## 架构设计

### 整体架构模式

| 模式 | 应用 | 说明 |
|------|------|------|
| **六边形架构** | `cowork-core` 纯领域逻辑 + `cowork-cli`/`cowork-gui` 适配器 | 领域零外部依赖，UI 通过 `InteractiveBackend` trait 桥接 |
| **DDD 战术模式** | `Project` 聚合根、`Iteration` 实体、`InheritanceMode` 值对象 | 一致性边界明确，仓储模式抽象持久化 |
| **Template Method** | Pipeline 控制器 | 7 阶段固定序列，各阶段通过 trait 自定义行为 |
| **Strategy** | 阶段执行器 | 每个 Stage 是独立的策略实现 |
| **Actor-Critic** | PRD/Design/Plan/Coding 阶段 | Actor 生成 → Critic 审查 → 迭代至阈值 |
| **事件驱动** | GUI 层 | Tauri Event 用于流式响应，Command 用于请求-响应 |

### 容器架构

```
┌─────────────────────────────────────────────────────────┐
│                    Cowork Forge                          │
│  ┌─────────────┐  ┌──────────────────────────────────┐  │
│  │ cowork-cli  │  │           cowork-gui              │  │
│  │ clap +      │  │  ┌──────────┐ ┌────────────────┐ │  │
│  │ dialoguer   │  │  │ Tauri    │ │ React 18 +     │ │  │
│  │             │  │  │ Backend  │ │ Ant Design     │ │  │
│  └──────┬──────┘  │  └────┬─────┘ └───────┬────────┘ │  │
│         │         │       │               │          │  │
│         └─────────┼───────┼───────────────┘          │  │
│                   │       │  InteractiveBackend trait │  │
│  ┌────────────────┴───────┴──────────────────────┐   │  │
│  │              cowork-core                        │   │  │
│  │  ┌─────────┐ ┌──────────┐ ┌─────────────────┐  │   │  │
│  │  │Pipeline │ │ Agents   │ │ Tools (40+ ADK) │  │   │  │
│  │  │(7-stage)│ │(Actor-   │ │ File/Data/HITL/ │  │   │  │
│  │  │         │ │ Critic)  │ │ Memory/Control  │  │   │  │
│  │  ├─────────┤ ├──────────┤ ├─────────────────┤  │   │  │
│  │  │Domain   │ │Persistence│ │ Config/LLM/ACP │  │   │  │
│  │  │Aggregate│ │JSON Stores│ │ Skills/Hooks   │  │   │  │
│  │  └─────────┘ └──────────┘ └─────────────────┘  │   │  │
│  └────────────────────────────────────────────────┘   │  │
│                                                        │  │
│  外部依赖: LLM API · 本地文件系统 · Shell · MCP 服务器  │  │
└─────────────────────────────────────────────────────────┘
```

### 关键依赖关系

- Pipeline → `InteractiveBackend` trait → CLI 或 GUI 实现
- Pipeline → Stage Executor → ADK Agent → Tool Registry + LLM
- Tools → Persistence (ProjectStore/IterationStore/MemoryStore)
- Tools → Security (路径验证、命令清洗)
- 全局 LLM Rate Limiter: Semaphore(concurrency=1) + 2s delay = 30 req/min

---

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| **Pipeline** | 7 阶段编排、StageExecutor、HITL 门控 | `crates/cowork-core/src/pipeline/` |
| **Domain** | Project/Iteration/Memory 聚合根、InheritanceMode | `crates/cowork-core/src/domain/` |
| **Agents** | IterativeAssistant、ExternalCodingAgent、LegacyAnalyzer | `crates/cowork-core/src/agents/` |
| **Tools** | 40+ ADK 工具 (File/Data/HITL/PM/Memory/Control/Validation/Deployment) | `crates/cowork-core/src/tools/` |
| **Config** | 数据驱动配置注册表 (Agent/Stage/Flow/Skill/Integration 定义) | `crates/cowork-core/src/config_definition/` |
| **Instructions** | 各阶段 Actor/Critic 提示词库 (~2000 行) | `crates/cowork-core/src/instructions/` |
| **Persistence** | ProjectStore/IterationStore/MemoryStore (JSON) | `crates/cowork-core/src/persistence/` |
| **Interaction** | `InteractiveBackend` trait + CLI/GUI 实现 | `crates/cowork-core/src/interaction/` |
| **Skills** | agentskills.io 标准技能系统 (发现/选择/注入) | `crates/cowork-core/src/skills/` |
| **ACP** | Agent Client Protocol 外部 Agent 集成 | `crates/cowork-core/src/acp/` |
| **LLM** | 客户端工厂 + Rate Limiter 装饰器 | `crates/cowork-core/src/llm/` |
| **Integration** | Hook 管理器 (外部集成) | `crates/cowork-core/src/integration/` |
| **Importer** | 遗留项目导入、分析、技术栈检测 | `crates/cowork-core/src/importer/` |
| **CLI** | clap 命令路由 + dialoguer 交互 | `crates/cowork-cli/src/` |
| **GUI Backend** | Tauri 命令 + 事件系统 (IPC) | `crates/cowork-gui/src-tauri/src/` |
| **GUI Frontend** | React 18 + Ant Design 界面 (9 面板) | `crates/cowork-gui/src/` |

---

## 核心流程

### 1. Genesis 迭代创建

```
用户输入想法
  → Pipeline 初始化 PipelineContext
  → Stage 1: Idea (需求捕获 → idea.md) → [HITL 门]
  → Stage 2: PRD (Actor 生成 → Critic 审查 → prd.md) → [HITL 门]
  → Stage 3: Design (架构设计 → design.md) → [HITL 门]
  → Stage 4: Plan (任务分解 → plan.md) → [HITL 门]
  → Stage 5: Coding (代码实现 → 源文件) → [HITL 门]
  → Stage 6: Check (质量验证 → check_report.md) → [HITL 门]
  → Stage 7: Delivery (交付报告 → 部署到项目根目录)
  → 知识快照生成 (决策/模式/技术栈) → 持久化到 MemoryStore
```

### 2. Evolution 迭代创建

```
用户描述变更请求
  → analyze_change_scope() 关键词分析确定继承模式
    - "redesign/refactor" → Full (制品+代码)
    - "feature/add" → Partial (仅代码)
    - "fix/bug" → None (全新)
  → 确定起始阶段 (Idea/PRD/Design/Plan/Coding)
  → 从基础迭代加载知识 (决策/模式/问题)
  → 从选定阶段恢复执行剩余流水线
```

### 3. HITL 验证门

```
Agent 调用 review 工具
  → 触发 InteractiveBackend.request_confirmation()
  → CLI: dialoguer 弹窗 (Pass/Edit/Feedback)
  → GUI: Tauri Event "input_request" → React Modal
  → 用户选择:
    - Pass → 继续流水线
    - Edit → 打开 $EDITOR, 等待修改
    - Feedback → Agent 重新生成 (Actor-Critic 迭代)
```

### 4. GUI 实时执行流

```
React 前端 Invoke Tauri Command → Core Pipeline
  → 循环中:
    - Core 通过 Tauri Event 发射 agent_streaming (逐 token)
    - Core 发射 tool_call/tool_result 事件
    - HITL 触发时发射 input_request 事件 → React Modal
    - 用户响应通过 Command 回传
  → 迭代完成 → 发射 iteration_complete 事件
```

---

## 技术选型

### 后端 (Rust 工作区)

- **语言/版本**: Rust 2024 edition, stable toolchain
- **异步运行时**: Tokio (`features = ["full"]`)
- **Agent 框架**: adk-rust v1.0 (adk-core/adk-agent/adk-model/adk-tool/adk-runner/adk-session/adk-skill)
- **LLM 客户端**: adk-model (OpenAI-compatible, streaming)
- **序列化**: serde + serde_json
- **配置**: toml (解析用户 `config.toml`)
- **错误处理**: anyhow (全局), thiserror (领域错误)

### CLI

- **参数解析**: clap v4 (derive)
- **交互**: dialoguer v0.12 (选择/确认/输入), console v0.16 (着色/格式化)

### GUI

- **桌面框架**: Tauri v1 (WebView2/WebKit)
- **前端框架**: React 18 + TypeScript
- **UI 组件库**: Ant Design
- **构建工具**: Vite
- **通信**: Tauri Command (请求-响应) + Event (流式推送)

### 存储

- **格式**: JSON 文件 (项目/迭代/记忆各自独立文件)
- **位置**: `.cowork-v2/` 工作区目录
- **策略**: 增量序列化, 懒加载

### 其他工具

- **文件系统监看**: `notify` crate (开发服务器日志)
- **路径处理**: `walkdir`, `ignore` (文件遍历)
- **进程管理**: `std::process` (命令执行, 开发服务器管理)
- **日志**: `tracing` + `tracing-subscriber`
- **时间**: `chrono` + `uuid`

---

## 系统边界

| 方向 | 外部系统 | 通信方式 | 约束 |
|------|----------|----------|------|
| **出站** | LLM Provider (OpenAI/etc.) | HTTPS, Streaming | 30 req/min, concurrency=1 |
| **出站** | MCP 服务器 (Tavily/DeepWiki) | HTTP (MCP 协议) | 配置驱动, 启动时自动注入 |
| **出站** | Shell (构建/测试命令) | `std::process` | 命令清洗白名单 |
| **出站** | 默认编辑器 (vim/VS Code/nano) | `std::process::Command` | 仅在 HITL Edit 模式使用 |
| **入站** | 用户 (CLI) | stdin/stdout | dialoguer 交互式提示 |
| **入站** | 用户 (GUI) | Tauri IPC (Command + Event) | WebView2/WebKit |
| **文件** | 本地文件系统 | `std::fs` | 路径验证 + 工作区边界检查 |
| **管理** | 包注册表 (npm/crates.io) | 间接 (用户运行) | 不在系统边界内 |

### 信任边界

- LLM API 响应: **不可信** — 所有工具/路径调用经过验证
- 用户输入: **可信但需清洗** — HITL 输入不绕过安全层
- 项目工作区: **受限** — 仅 `validate_path()` 批准的路径可访问

### 安全机制

| 机制 | 位置 | 说明 |
|------|------|------|
| 路径验证 | `crates/cowork-core/src/runtime_security.rs` | UNC 正规化 + 工作区边界检查 |
| 命令清洗 | `crates/cowork-core/src/pipeline/executor/workspace.rs` | 危险命令黑名单 (`rm -rf`, `sudo`) |
| 运行时监控 | `crates/cowork-core/src/runtime_analyzer.rs` | 检测 Agent 偏离目标 |

---

## 代码映射索引

| 概念 | 位置 | 说明 |
|------|------|------|
| Pipeline 控制器 | `crates/cowork-core/src/pipeline/mod.rs` | 流水线入口 + 状态管理 |
| Stage 执行器 | `crates/cowork-core/src/pipeline/stage_executor.rs` | ADK Agent 生命周期桥接 |
| 阶段实现 (7个) | `crates/cowork-core/src/pipeline/stages/` | 各阶段独立文件 |
| HITL 扩展 | `crates/cowork-core/src/pipeline/executor/` | interaction_ext / knowledge / workspace |
| Project 聚合 | `crates/cowork-core/src/domain/project.rs` | 聚合根, 元数据, 迭代摘要 |
| Iteration 实体 | `crates/cowork-core/src/domain/iteration.rs` | 继承模式, 状态机, 变化分析 |
| Memory 聚合 | `crates/cowork-core/src/domain/memory.rs` | 决策/模式/洞察/问题 |
| Tool 注册 + 实现 | `crates/cowork-core/src/tools/` | 40+ 工具, 分类子模块 |
| Agent 包装 | `crates/cowork-core/src/agents/` | iterative_assistant, external_coding_agent |
| InteractiveBackend trait | `crates/cowork-core/src/interaction/mod.rs` | CLI/GUI 抽象端口 |
| CLI 实现 | `crates/cowork-core/src/interaction/cli.rs` | dialoguer 适配器 |
| 配置注册表 | `crates/cowork-core/src/config_definition/registry.rs` | Agent/Stage/Flow 定义管理 |
| 内置配置 JSON | `crates/cowork-core/src/config_definition/default_configs/` | 14 个 Agent + 7 个 Stage + 默认 Flow |
| 指令库 | `crates/cowork-core/src/instructions/` | ~2000 行 Actor/Critic 提示词 |
| 持久化仓储 | `crates/cowork-core/src/persistence/` | ProjectStore, IterationStore, MemoryStore |
| LLM + 限流器 | `crates/cowork-core/src/llm/` | 客户端工厂, Semaphore+Delay |
| ACP 客户端 | `crates/cowork-core/src/acp/` | 外部 Agent 协议 |
| Skills 管理器 | `crates/cowork-core/src/skills/` | agentskills.io 标准 |
| CLI 命令 | `crates/cowork-cli/src/commands/` | init/list/status/continue/delete/config |
| Tauri 命令 | `crates/cowork-gui/src-tauri/src/commands/` | IPC 端点 |
| GUI 前端面板 | `crates/cowork-gui/src/components/` | Projects/Chat/Runner/Config 等 9 面板 |
| 安全验证 | `crates/cowork-core/src/runtime_security.rs` | 路径验证 + 命令清洗 |
| 运行时分析 | `crates/cowork-core/src/runtime_analyzer.rs` | Agent 行为监控 |
