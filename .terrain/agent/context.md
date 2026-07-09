---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是 AI 原生的多 Agent 软件开发平台：将产品经理、架构师、项目经理、工程师等角色编排为虚拟开发团队，通过七阶段流水线（Idea→PRD→Design→Plan→Coding→Check→Delivery）把自然语言想法转化为可交付软件。面向独立开发者、技术负责人与产品团队；提供 CLI（自动化）与 Tauri 桌面 GUI（交互式）双入口。核心约束：本地优先、工作区沙箱、LLM 限流（30 req/min）、关键阶段 HITL 确认门、Actor-Critic 自优化、迭代继承（Genesis/Evolution）。

## 架构设计

| 容器/层 | 职责 | 主要路径 |
|---------|------|----------|
| **cowork-core** | 域逻辑、流水线、Agent/Tool、持久化、安全 | `crates/cowork-core/src/` |
| **cowork-cli** | CLI 适配器（clap + dialoguer） | `crates/cowork-cli/` |
| **cowork-gui** | React 前端 + Tauri 后端 | `crates/cowork-gui/src/`, `src-tauri/` |
| **配置驱动层** | Agent/Stage/Flow JSON 定义与注册 | `config_definition/` |
| **知识资产** | Terrain 私域与 Litho 人类文档 | `.terrain/`, `litho.docs/` |

**架构模式**：六边形（`InteractiveBackend` 入站端口；Store/LLM 出站端口）、DDD 聚合（Project/Iteration/Memory）、Actor-Critic（adk-rust `LoopAgent`）、事件驱动 GUI（Tauri invoke + emit）。

**依赖关系**：`cowork-cli` / `cowork-gui` → `cowork-core` → adk-rust 生态（`adk-core`, `adk-agent`, `adk-model`, `adk-tool`）+ Tokio + OpenAI 兼容 LLM API。

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| Pipeline | 七阶段编排、上下文传递、阶段转换 | `pipeline/`, `pipeline/stages/`, `stage_executor.rs` |
| Config Definition | 数据驱动 Agent/Stage/Flow 注册与校验 | `config_definition/`, `default_configs/` |
| Domain | Project、Iteration、Memory 聚合与继承模式 | `domain/` |
| Tools | 40+ ADK 工具（文件/工件/HITL/验证/内存/部署） | `tools/` |
| Agents | PM Agent、迭代助手、外部编码 Agent、遗留分析 | `agents/` |
| Instructions | 各阶段 Actor/Critic 提示词库 | `instructions/` |
| Interaction | `InteractiveBackend` trait；CLI/GUI 抽象 | `interaction/`, `cowork-gui/src-tauri/` |
| Persistence | JSON 项目/迭代/内存存储 | `persistence/` |
| LLM | 模型工厂、全局限流装饰器 | `llm/` |
| ACP | Agent Client Protocol 外部编码工具集成 | `acp/` |
| Importer | 遗留项目导入与反向工程 | `importer/` |
| Skills | agentskills.io 标准技能管理 | `skills/` |

## 核心流程

### 1. Genesis 迭代（想法→交付）

1. 用户通过 CLI/GUI 创建项目与迭代，输入初始想法
2. `PipelineExecutor` 按 `default.json` 顺序执行七阶段
3. 每阶段 `StageExecutor` 构建 adk-rust Agent（Actor→Critic `LoopAgent`），流式调用 LLM 与工具
4. 产出工件（idea/prd/design/plan 等 markdown）写入 `.cowork-v2/iterations/{id}/`
5. 关键阶段触发 HITL：用户通过/编辑/反馈 → `execute_with_feedback` 重试
6. 完成后生成知识快照，更新迭代状态

### 2. Actor-Critic 自优化（单阶段内）

1. Actor（`IncludeContents::Default`）生成 artifact 并持久化
2. Critic（`IncludeContents::None`）经工具加载 artifact 审查（非对话历史）
3. 通过 → `exit_loop`；小问题 → 文字反馈供下轮 Actor；大问题 → `provide_feedback` + escalate 触发 Stage 级重试

### 3. Evolution 迭代（增量演进）

1. 用户选择继承模式（None/Full/Partial）创建 Evolution 迭代
2. 系统分析变更范围，决定起始阶段与可复用工件
3. 合并项目级/迭代级 Memory，注入 Pipeline 上下文
4. 从映射阶段继续流水线，复用或增量修改代码与文档

### 4. 遗留项目导入

1. CLI/GUI 指定现有代码库路径
2. `LegacyProjectAnalyzer` 检测技术栈、结构、依赖
3. 生成初始 Project/Iteration 记录与反向工程文档
4. 纳入常规范畴管理与后续迭代

## 技术选型

- **语言/运行时**：Rust 2024 edition、Tokio 全特性异步
- **AI 编排**：adk-rust 1.0（`LlmAgentBuilder`, `LoopAgent`, `Tool` trait）
- **LLM**：OpenAI 兼容 API（`adk-model` openai feature）；信号量+延迟限流
- **CLI**：clap 4、dialoguer、console
- **GUI**：Tauri 2、React 18、TypeScript、Ant Design、Vite
- **序列化/配置**：serde/serde_json、toml
- **持久化**：本地 JSON 文件（ProjectStore/IterationStore/MemoryStore）
- **外部协议**：agent-client-protocol 0.9（ACP）、MCP（Tavily/DeepWiki 等）
- **错误处理**：anyhow、thiserror
- **可观测**：tracing + tracing-subscriber

## 系统边界

| 边界 | 类型 | 说明 |
|------|------|------|
| LLM 提供商 API | 外部、需密钥 | OpenAI 兼容端点；限流 30 req/min |
| 本地文件系统 | 信任域内 | 项目根、`.cowork-v2/` 工件；路径校验防逃逸 |
| Shell/子进程 | 受限执行 | 构建/测试/开发服务器；命令白名单与沙箱 |
| MCP 服务器 | 可选外部 | Tavily 搜索、DeepWiki 文档；`config.toml [mcp]` 配置 |
| ACP 外部 Agent | 可选外部 | OpenCode/Gemini CLI/Claude CLI 等编码阶段 |
| 外部编辑器 | OS 集成 | HITL 编辑阶段调用系统默认编辑器 |
| 用户配置 | 本地 | `~/Library/Application Support/CoworkCreative/config.toml`（macOS） |
| GUI 配置 | 本地 | `com.cowork-forge.app/config/` 用户 Agent/Flow 覆盖 |

**范围外**：LLM 训练、Git VCS 集成、包注册表、云 CI/CD、多用户实时协作。

**信任边界**：`runtime_security` 校验所有文件/命令操作不越出 workspace；API Key 仅存 config/env，不入库。

## 代码映射索引

| 概念 | 位置 | 备注 |
|------|------|------|
| 流水线入口 | `pipeline/executor/mod.rs` | PipelineContext 与阶段调度 |
| 阶段执行 | `pipeline/stage_executor.rs` | ADK Agent 生命周期、流式、HITL |
| 七阶段实现 | `pipeline/stages/*.rs` | 各 Stage trait 实现 |
| 默认流程定义 | `config_definition/default_configs/flows/default.json` | idea→delivery 顺序 |
| 内置 Agent 配置 | `config_definition/default_configs/agents/built-in/` | Actor/Critic JSON |
| 交互抽象 | `interaction/mod.rs`, `interaction/cli.rs` | InteractiveBackend trait |
| GUI 后端适配 | `cowork-gui/src-tauri/src/lib.rs` | TauriBackend 实现 |
| GUI 命令层 | `cowork-gui/src-tauri/src/commands/` | runner/pm/import 等 |
| 项目运行时 | `project_runtime.rs`, `runtime_analyzer.rs` | 技术栈检测与预览 |
| 安全层 | `runtime_security.rs` | 路径与命令校验 |
| 领域模型 | `domain/project.rs`, `iteration.rs`, `memory.rs` | 核心聚合 |
| 持久化 | `persistence/*_store.rs` | JSON Repository |
| 工具注册 | `tools/mod.rs` | 40+ Tool 聚合导出 |
| 控制流工具 | `tools/control_tools.rs`, `goto_stage_tool.rs` | exit_loop/provide_feedback/goto_stage |
| PM Agent | `agents/mod.rs`, `instructions/project_manager.rs` | 阶段跳转与迭代创建 |
| 外部编码 Agent | `agents/external_coding_agent.rs`, `acp/client.rs` | ACP 协议 |
| CLI 命令 | `cowork-cli/src/commands/` | init/iter/continue/import 等 |
| 技能系统 | `skills/manager.rs` | agentskills.io 加载 |
| 集成钩子 | `integration/hooks.rs`, `adapters.rs` | 外部集成扩展点 |
| 工作区约定 | `.cowork-v2/iterations/{id}/` | artifacts、session_history |