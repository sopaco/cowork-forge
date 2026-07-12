---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

## 项目概览

Cowork Forge 是一个 AI 原生多 Agent 软件开发平台，通过 7 阶段流水线将自然语言想法转化为生产就绪软件。核心价值主张：通过结构化 AI 编排（Idea→PRD→Design→Plan→Coding→Check→Delivery）实现"一人即一个开发团队"。系统采用六边形架构，以 `cowork-core` 为纯域核心，支持 CLI 自动化和 GUI 交互双模式，内置 Actor-Critic 自优化、迭代继承、知识累积和 HITL 人机协同。

## 架构设计

**核心架构模式**：
- **六边形架构（端口与适配器）**：`cowork-core` 为纯域逻辑中心，通过端口抽象与外部交互
- **领域驱动设计（DDD）**：Project、Iteration、ProjectMemory 聚合根，实施一致性边界
- **Actor-Critic 模式**：每个阶段采用 Actor 生成工件 + Critic 质量验证的自优化循环
- **事件驱动架构**：GUI 层通过 Tauri 事件系统实现实时流式通信

**容器与层次**：

| 层级 | 组件 | 技术 | 架构角色 |
|------|------|------|----------|
| **表现层** | `cowork-cli` | clap + dialoguer | CLI 适配器，命令路由 |
| **表现层** | `cowork-gui/src` | React 18 + Ant Design | GUI 前端，8 面板交互界面 |
| **应用层** | `cowork-gui/src-tauri` | Tauri + Rust | GUI 后端，IPC 命令处理器 |
| **域层** | `cowork-core` | Rust + Tokio | 纯业务逻辑，零外部依赖 |
| **基础设施层** | LLM 集成 | OpenAI 兼容 API | AI 推理，限流装饰器（30 次/分钟） |
| **基础设施层** | 持久化 | JSON + serde | 项目/迭代/记忆存储 |

**主要依赖**：
- **adk-rust**：Agent 开发框架，提供 Agent、Tool、LoopAgent 等核心抽象
- **Tokio**：异步运行时，管理并发流程执行和 I/O
- **Tauri**：桌面应用框架，提供系统集成和事件驱动通信
- **MCP 服务器**：外部 AI 能力集成（Tavily/DeepWiki）

## 模块地图

| 模块 | 职责 | 主要路径 |
|------|------|----------|
| **项目域** | Project 聚合根，项目元数据与迭代摘要 | `crates/cowork-core/src/domain/project.rs` |
| **迭代域** | Iteration 实体，生命周期管理与继承分析 | `crates/cowork-core/src/domain/iteration.rs` |
| **内存域** | ProjectMemory 聚合，跨迭代知识累积 | `crates/cowork-core/src/domain/memory.rs` |
| **流程编排** | 7 阶段流水线控制器，阶段执行器 | `crates/cowork-core/src/pipeline/mod.rs` |
| **阶段实现** | 7 个阶段策略，Actor-Critic 模式 | `crates/cowork-core/src/pipeline/stages/*.rs` |
| **工具系统** | 40+ ADK 工具（文件、数据、验证、HITL） | `crates/cowork-core/src/tools/*.rs` |
| **指令库** | Agent 提示工程（~2000 行） | `crates/cowork-core/src/instructions/*.rs` |
| **配置系统** | 数据驱动的 Agent/Stage/Flow 定义 | `crates/cowork-core/src/config_definition/` |
| **持久化层** | JSON 存储仓库模式 | `crates/cowork-core/src/persistence/*.rs` |
| **安全层** | 路径验证、命令清理、工作区隔离 | `crates/cowork-core/src/runtime_security.rs` |
| **交互抽象** | InteractiveBackend 特性（CLI/GUI 统一） | `crates/cowork-core/src/interaction/mod.rs` |
| **ACP 集成** | Agent Client Protocol 外部 Agent 集成 | `crates/cowork-core/src/acp/client.rs` |
| **技能系统** | agentskills.io 标准技能包管理 | `crates/cowork-core/src/skills/manager.rs` |
| **GUI 组件** | React 前端 8 面板界面 | `crates/cowork-gui/src/components/` |
| **GUI 后端** | Tauri 命令与事件处理 | `crates/cowork-gui/src-tauri/src/commands/` |

## 核心流程

### 1. 创世迭代创建流程（主要工作流）

1. 用户通过 CLI/GUI 提供自然语言想法
2. 流程控制器初始化 PipelineContext，创建初始迭代（草稿状态）
3. 依次执行 7 个阶段：Idea→PRD→Design→Plan→Coding→Check→Delivery
4. 每个阶段：加载指令→创建 ADK Agent→LLM 流式推理→工具调用→工件生成
5. 关键阶段（Idea、PRD、Design、Plan、Coding）触发 HITL 确认门
6. 用户可选择：通过（继续）、编辑（打开外部编辑器）、反馈（重新生成）
7. 阶段完成后持久化工件到迭代工作区（`.cowork-v2/iterations/{id}/`）
8. 完成后生成知识快照，提取决策、模式、技术栈到项目内存

### 2. Actor-Critic 自优化流程

1. Actor Agent 根据阶段指令生成初始工件（文档/代码）
2. Critic Agent 审查工件质量，检查约束和标准
3. 若 Critic 发现问题：通过工具持久化反馈并退出循环（`escalate=true`）
4. Actor 在下一轮迭代中看到 Critic 反馈，修正产出
5. 循环继续直到 Critic 调用 `exit_loop` 或达到最大迭代次数
6. Stage Executor 根据循环历史决定最终重试或继续

### 3. 演进迭代继承流程

1. 用户请求演进，提供变更描述
2. 系统分析变更范围（NLP 关键词匹配：架构→完全继承，功能→部分继承，修复→无继承）
3. 选择起始阶段（"redesign"→Idea，"add feature"→Plan，"fix bug"→Check）
4. 根据继承模式准备工作区：完全（工件+代码）、部分（仅代码）、无（全新）
5. 加载先前迭代的基础知识到 Agent 上下文
6. 从选定阶段恢复流程，执行剩余阶段

### 4. 外部 Agent 集成流程（ACP）

1. 配置外部 Agent（OpenCode、Gemini CLI、Claude CLI）
2. 编码阶段触发时，通过 ACP 协议调用外部 Agent
3. 外部 Agent 在隔离工作区执行编码任务
4. 结果通过 ACP 客户端返回主流程
5. 主流程验证并集成外部 Agent 的输出

## 技术选型

**核心栈**：
- **语言**：Rust（Edition 2024，stable toolchain）
- **异步运行时**：Tokio（full features）
- **Agent 框架**：adk-rust（LlmAgentBuilder、LoopAgent、Tool trait）
- **LLM 集成**：OpenAI 兼容 API，自定义限流装饰器（信号量+延迟）

**前端**：
- **桌面框架**：Tauri（Rust 后端 + WebView2/WebKit）
- **UI 框架**：React 18 + TypeScript
- **组件库**：Ant Design
- **构建工具**：Vite

**持久化**：
- **存储格式**：JSON + Markdown（schema 演化友好）
- **序列化**：serde + derive macros
- **存储位置**：`.cowork-v2/` 目录（项目/迭代/内存）

**开发工具**：
- **CLI**：clap（参数解析）+ dialoguer（终端交互）
- **日志**：tracing crate（结构化日志）
- **测试**：cargo test（单元/集成测试）

## 系统边界

**外部 API**：
- **LLM 提供商**：OpenAI 兼容端点，限流 30 次/分钟，全局信号量控制并发
- **MCP 服务器**：Tavily（网络搜索）、DeepWiki（代码文档查询），通过 Model Context Protocol 集成

**本地资源**：
- **文件系统**：项目工作区（`.cowork-v2/`），路径验证防止遍历攻击
- **Shell 执行器**：项目验证（依赖安装、构建、测试），命令清理和沙箱执行
- **外部编辑器**：系统默认编辑器（vim、VS Code、nano），HITL 内容审查
- **开发服务器**：Vite/Webpack，进程管理（ProcessRunner）

**信任边界**：
- **工作区隔离**：所有文件操作限制在项目工作区内
- **命令清理**：阻止危险命令（`rm -rf`、`sudo`）
- **路径验证**：UNC 规范化，防止 `..` 遍历
- **LLM 限流**：全局信号量确保 API 配额合规

**范围内**：核心域逻辑、7 阶段流水线、40+ 工具、JSON 持久化、CLI/GUI 适配器、跨平台桌面应用

**范围外**：第三方 LLM 训练、版本控制集成（Git）、包注册表管理、云部署、实时多用户协作

## 代码映射索引

| 概念 | 位置 | 备注 |
|------|------|------|
| Project 聚合 | `crates/cowork-core/src/domain/project.rs` | 项目元数据、技术栈检测 |
| Iteration 实体 | `crates/cowork-core/src/domain/iteration.rs` | 迭代生命周期、继承模式分析 |
| ProjectMemory 聚合 | `crates/cowork-core/src/domain/memory.rs` | 跨迭代知识累积、模糊查询 |
| 流程控制器 | `crates/cowork-core/src/pipeline/mod.rs` | 7 阶段编排、HITL 门 |
| 阶段执行器 | `crates/cowork-core/src/pipeline/stage_executor.rs` | 连接域逻辑与 ADK 框架 |
| 阶段实现 | `crates/cowork-core/src/pipeline/stages/*.rs` | 7 个策略模式实现 |
| ADK 工具 | `crates/cowork-core/src/tools/*.rs` | 40+ 工具（文件、数据、验证、HITL） |
| 指令库 | `crates/cowork-core/src/instructions/*.rs` | Agent 提示工程（~2000 行） |
| 配置注册表 | `crates/cowork-core/src/config_definition/registry.rs` | 数据驱动的 Agent/Stage/Flow 定义 |
| Agent 定义 | `crates/cowork-core/src/config_definition/agent_definition.rs` | 内置/自定义 Agent 配置 |
| Stage 定义 | `crates/cowork-core/src/config_definition/stage_definition.rs` | 阶段配置与钩子 |
| Flow 定义 | `crates/cowork-core/src/config_definition/flow_definition.rs` | 工作流配置 |
| 持久化存储 | `crates/cowork-core/src/persistence/*.rs` | JSON 存储仓库模式 |
| 安全验证 | `crates/cowork-core/src/runtime_security.rs` | 路径验证、命令清理 |
| 交互后端 | `crates/cowork-core/src/interaction/mod.rs` | InteractiveBackend 特性（端口） |
| CLI 适配器 | `crates/cowork-cli/src/commands/*.rs` | clap 命令路由、dialoguer 交互 |
| GUI 后端 | `crates/cowork-gui/src-tauri/src/commands/*.rs` | Tauri 命令、事件发射 |
| GUI 前端 | `crates/cowork-gui/src/components/*.tsx` | React 8 面板界面 |
| ACP 客户端 | `crates/cowork-core/src/acp/client.rs` | 外部 Agent 集成协议 |
| 技能管理器 | `crates/cowork-core/src/skills/manager.rs` | agentskills.io 标准实现 |
| LLM 限流器 | `crates/cowork-core/src/llm/rate_limiter.rs` | 信号量+延迟装饰器 |
| 运行时分析器 | `crates/cowork-core/src/runtime_analyzer.rs` | Agent 行为监控 |
| 技术栈检测 | `crates/cowork-core/src/tech_stack.rs` | 项目技术栈自动检测 |
| 项目导入器 | `crates/cowork-core/src/importer/*.rs` | 遗留项目反向工程 |
| 知识工具 | `crates/cowork-core/src/tools/knowledge_tools.rs` | 知识生成与查询 |
| 内存工具 | `crates/cowork-core/src/tools/memory_tools.rs` | 记忆保存、查询、提升 |