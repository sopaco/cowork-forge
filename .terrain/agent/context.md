---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是一个 **AI 原生的多 Agent 软件开发平台**，将自然语言想法通过 7 阶段流水线转化为可交付软件。系统内置 10+ 专业 AI Agent（产品经理、架构师、项目经理、工程师等），采用 Actor-Critic 自优化模式，在关键节点设置 Human-in-the-Loop 验证门。核心价值：**一个人拥有完整虚拟开发团队**。运行在本地桌面环境，通过 OpenAI 兼容 API 驱动 LLM 推理，无外部数据库依赖。支持 Genesis（首次）和 Evolution（演化）两种迭代模式，具备跨迭代记忆累积能力。提供 CLI 自动化与 Tauri GUI 交互双界面。

## 架构设计

### 六边形架构 + DDD 战术模式

| 模式 | 应用 | 说明 |
|------|------|------|
| **六边形架构** | `cowork-core` 领域内核 + `cowork-cli`/`cowork-gui` 适配器 | 领域零外部依赖，UI 通过 `InteractiveBackend` trait 桥接 |
| **DDD 聚合** | `Project` 聚合根、`Iteration` 实体、`InheritanceMode` 值对象 | 一致性边界明确，仓储模式抽象持久化 |
| **Template Method** | Pipeline 控制器 | 7 阶段固定序列，各阶段通过 trait 自定义行为 |
| **Strategy** | 阶段执行器 | 每个 Stage 是独立的策略实现 |
| **Actor-Critic** | PRD/Design/Plan/Coding 阶段 | Actor 生成 → Critic 审查 → 迭代至阈值 (LoopAgent) |
| **事件驱动** | GUI 层 | Tauri Event 流式推送，Command 请求-响应 |

### 容器拓扑

```
┌──────────────────────────────────────────────────────┐
│                   Cowork Forge                        │
│  ┌──────────────┐  ┌──────────────────────────────┐  │
│  │ cowork-cli   │  │        cowork-gui             │  │
│  │ clap +       │  │  ┌────────┐ ┌──────────────┐ │  │
│  │ dialoguer    │  │  │ Tauri  │ │ React 18 +   │ │  │
│  │              │  │  │ Backend│ │ Ant Design 5 │ │  │
│  └──────┬───────┘  │  └───┬────┘ └──────┬───────┘ │  │
│         │          │      │             │         │  │
│         └──────────┼──────┼─────────────┘         │  │
│                    │      │ InteractiveBackend     │  │
│  ┌─────────────────┴──────┴────────────────────┐  │  │
│  │               cowork-core                    │  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌────────────┐   │  │  │
│  │  │ Pipeline │ │ Agents   │ │ Tools(40+) │   │  │  │
│  │  │ (7-stage)│ │ Actor-   │ │ ADK-based  │   │  │  │
│  │  │          │ │ Critic   │ │ File/Data/ │   │  │  │
│  │  │          │ │ LoopAgt  │ │ HITL/PM/   │   │  │  │
│  │  ├──────────┤ ├──────────┤ ├────────────┤   │  │  │
│  │  │ Domain   │ │Persistence│ │ Config/LLM │   │  │  │
│  │  │ Aggregts │ │JSON 存储  │ │ ACP/Skills │   │  │  │
│  │  └──────────┘ └──────────┘ └────────────┘   │  │  │
│  └─────────────────────────────────────────────┘  │  │
│   LLM API (OpenAI) · adk-rust v1 · 本地文件系统    │  │
└──────────────────────────────────────────────────────┘
```

### 关键依赖链

- Pipeline → `InteractiveBackend` trait → CLI/GUI 实现
- Pipeline → StageExecutor → ADK Agent → ToolRegistry + LLM
- Tools → Persistence (ProjectStore/IterationStore/MemoryStore)
- Tools → Security (`validate_path()` 路径验证, 命令清洗)
- 全局 LLM Rate Limiter: `Semaphore(concurrency=1)` + 2s delay = 30 req/min

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| **Pipeline** | 7 阶段编排 + StageExecutor + HITL 门控 + 工作区 | `crates/cowork-core/src/pipeline/` |
| **Domain** | Project/Iteration/Memory 聚合根、InheritanceMode | `crates/cowork-core/src/domain/` |
| **Agents** | IterativeAssistant (Actor-Critic)、ExternalCodingAgent (ACP)、LegacyAnalyzer | `crates/cowork-core/src/agents/` |
| **Tools** | 40+ ADK 工具：文件/数据/HITL/PM/记忆/控制/验证/部署/Artifact | `crates/cowork-core/src/tools/` |
| **Config** | 数据驱动注册表：Agent/Stage/Flow/Skill/Integration 定义 + 内置 JSON | `crates/cowork-core/src/config_definition/` |
| **Instructions** | 各阶段 Actor/Critic 系统提示词 + Legacy 分析器提示词 | `crates/cowork-core/src/instructions/` |
| **Persistence** | ProjectStore/IterationStore/MemoryStore (JSON, 增量序列化) | `crates/cowork-core/src/persistence/` |
| **LLM** | OpenAI 兼容 API 封装 + 全局限流器 | `crates/cowork-core/src/llm/` |
| **Interaction** | `InteractiveBackend` trait + CLI dialoguer 实现 | `crates/cowork-core/src/interaction/` |
| **ACP** | Agent Client Protocol 客户端 (外部 Coding Agent) | `crates/cowork-core/src/acp/` |
| **Skills** | agentskills.io 标准技能管理器 | `crates/cowork-core/src/skills/` |
| **Security** | 路径验证 (UNC 正规化) + 命令清洗白名单 | `crates/cowork-core/src/runtime_security.rs` |
| **Runtime** | Agent 行为偏离监控 (Watchdog) | `crates/cowork-core/src/runtime_analyzer.rs` |
| **CLI** | clap 命令组 (init/list/status/continue/iter/config/import/show/knowledge) | `crates/cowork-cli/src/commands/` |
| **GUI 后端** | Tauri 命令 + 事件发射 + ProjectRunner | `crates/cowork-gui/src-tauri/src/` |
| **GUI 前端** | React 18 + Ant Design 面板 (Chat/Runner/Config/Projects/Memory/Knowledge/Iterations) | `crates/cowork-gui/src/components/` |

## 核心流程

### 1. Genesis 迭代 (完整 7 阶段)

```
用户输入想法
  → Stage 1: Idea (需求捕获 → idea.md) → [HITL]
  → Stage 2: PRD (Actor→Critic 循环 → prd.md) → [HITL]
  → Stage 3: Design (Actor→Critic → design.md) → [HITL]
  → Stage 4: Plan (Actor→Critic → plan.md) → [HITL]
  → Stage 5: Coding (Actor→Critic → 源文件) → [HITL]
  → Stage 6: Check (质量验证 → check_report.md) → [HITL]
  → Stage 7: Delivery (交付报告 → 部署)
  → 知识快照: 决策/模式/技术栈 → MemoryStore
```

### 2. Evolution 迭代 (继承模式)

```
用户变更请求 → analyze_change_scope() 关键词分析:
  "redesign/refactor" → Full (制品+代码继承)
  "feature/add"      → Partial (仅代码继承, 重生成制品)
  "fix/bug"          → None (全新, 只参考上下文)
→ 确定起始阶段 (Idea/PRD/Design/Plan/Coding)
→ 加载基础迭代记忆 → 从选定阶段恢复流水线
```

### 3. Actor-Critic 自优化循环 (LoopAgent)

```
LoopAgent(max_iterations=N):
  1. Actor Agent 运行 → 生成产出 (写入共享 Session)
  2. Critic Agent 读取 Session → 审查产出
     - 通过 → ExitLoopTool, 循环终止
     - 不通过 → 写入反馈, 继续下一轮
  3. Actor 读取反馈 → 修改 → Critic 再审
  关键: include_contents(IncludeContents::Default)
```

### 4. HITL 验证门

```
Agent 调用 review 工具 → InteractiveBackend.request_confirmation()
  → CLI: dialoguer (Pass/Edit/Feedback)
  → GUI: Tauri Event "input_request" → React Modal
  用户选择: Pass → 继续 | Edit → $EDITOR | Feedback → 重新生成
```

### 5. GUI 实时执行流

```
React Invoke Tauri Command → Core Pipeline
  → Core 发射 Tauri Events:
    agent_event / agent_streaming / tool_call / tool_result / input_request
  → 用户响应通过 Command 回传
  → iteration_complete 事件
```

## 技术选型

### 后端
- **语言**: Rust 2024 edition, stable toolchain
- **异步**: Tokio (features = ["full"])
- **Agent 框架**: adk-rust v1.0 (adk-core, adk-agent, adk-model[openai], adk-tool, adk-runner, adk-session, adk-skill)
- **序列化**: serde + serde_json + toml
- **错误处理**: anyhow (全局) + thiserror (领域)
- **CLI 框架**: clap v4 (derive) + dialoguer v0.12 + console v0.16

### GUI
- **桌面框架**: Tauri (WebView2/WebKit)
- **前端**: React 18 + TypeScript + Ant Design 5 + Vite
- **通信**: Tauri Command (请求-响应) + Event (流式推送)

### 存储
- **格式**: JSON 文件，三个独立仓储
- **位置**: `.cowork-v2/` 工作区目录
- **策略**: 增量序列化，懒加载

### 其他依赖
- chrono + uuid (时间/标识)
- walkdir + ignore (文件遍历)
- tracing + tracing-subscriber (日志)
- futures (异步组合)
- agent-client-protocol v0.9 (ACP 外部 Agent)

## 系统边界

| 方向 | 外部系统 | 通信方式 | 约束 |
|------|----------|----------|------|
| **出站** | LLM Provider (OpenAI) | HTTPS, Streaming | 30 req/min, concurrency=1 |
| **出站** | MCP 服务器 (Tavily/DeepWiki) | HTTP (MCP 协议) | 配置驱动, 启动时注入 |
| **出站** | Shell (构建/测试) | `std::process` | 命令白名单清洗 |
| **出站** | 编辑器 (vim/VS Code/nano) | `std::process::Command` | HITL Edit 模式 |
| **入站** | 用户 (CLI) | stdin/stdout | dialoguer 交互式提示 |
| **入站** | 用户 (GUI) | Tauri IPC (Command+Event) | WebView2/WebKit |
| **文件** | 本地文件系统 | `std::fs` | `validate_path()` 边界检查 |
| **入站** | 外部 Coding Agent | ACP 协议 | 仅在 Coding 阶段启用 |

### 信任边界

- **LLM 响应**: 不可信 — 所有工具/路径调用经过验证
- **用户输入**: 可信但需清洗 — HITL 不绕过安全层
- **工作区**: 受限 — 仅 `validate_path()` 批准的路径可访问

### 安全机制

| 机制 | 路径 |
|------|------|
| 路径验证 (UNC 正规化 + 边界检查) | `crates/cowork-core/src/runtime_security.rs` |
| 命令清洗 (危险命令黑名单) | `crates/cowork-core/src/pipeline/executor/workspace.rs` |
| 运行时监控 (Agent 目标偏离检测) | `crates/cowork-core/src/runtime_analyzer.rs` |

## 代码映射索引

| 概念 | 位置 | 说明 |
|------|------|------|
| Pipeline 编排入口 | `crates/cowork-core/src/pipeline/mod.rs` | 状态机 + 次序 |
| StageExecutor | `crates/cowork-core/src/pipeline/stage_executor.rs` | ADK Agent 生命周期 |
| 阶段实现 (7 个) | `crates/cowork-core/src/pipeline/stages/` | idea/prd/design/plan/coding/check/delivery |
| 执行器扩展 | `crates/cowork-core/src/pipeline/executor/` | 交互/知识/工作区 |
| Domain 聚合根 | `crates/cowork-core/src/domain/` | project.rs, iteration.rs, memory.rs |
| Tool 实现 (40+) | `crates/cowork-core/src/tools/` | 14 个模块文件 |
| Agent 工厂 | `crates/cowork-core/src/agents/` | iterative_assistant, external_coding_agent, legacy_analyzer |
| InteractiveBackend | `crates/cowork-core/src/interaction/` | trait + CLI 适配器 |
| 配置注册表 | `crates/cowork-core/src/config_definition/registry.rs` | Agent/Stage/Flow |
| 内置配置 | `crates/cowork-core/src/config_definition/default_configs/` | 14 Agent + 7 Stage + 1 Flow JSON |
| 指令库 | `crates/cowork-core/src/instructions/` | 各阶段提示词 |
| 持久化仓储 | `crates/cowork-core/src/persistence/` | project_store, iteration_store, memory_store |
| LLM 限流器 | `crates/cowork-core/src/llm/rate_limiter.rs` | 全局 Semaphore + delay |
| ACP 客户端 | `crates/cowork-core/src/acp/client.rs` | Agent Client Protocol |
| Skills 管理器 | `crates/cowork-core/src/skills/manager.rs` | agentskills.io 标准 |
| CLI 命令组 | `crates/cowork-cli/src/commands/` | 10 个子命令 |
| Tauri 命令 | `crates/cowork-gui/src-tauri/src/commands/` | 8 个模块 |
| GUI 前端面板 | `crates/cowork-gui/src/components/` | 9 面板 |
| GUI 状态管理 | `crates/cowork-gui/src/stores/` | 4 stores |
| 安全层 | `crates/cowork-core/src/runtime_security.rs` | 路径 + 命令验证 |
| 运行时监控 | `crates/cowork-core/src/runtime_analyzer.rs` | Agent 偏离检测 |
| 遗留项目导入 | `crates/cowork-core/src/importer/` | 分析/逆向生成 |
| 集成钩子 | `crates/cowork-core/src/integration/` | HookManager |
| Artifact 存储 | `crates/cowork-core/src/tools/artifact_tools.rs` | 制品持久化 |