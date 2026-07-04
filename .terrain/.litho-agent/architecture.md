# 架构研究报告

## 架构模式

Cowork Forge 的核心架构可以理解为一个"AI 驱动的流水线工厂"——它的整体骨架是**流水线-过滤器（Pipeline-Filter）模式**，7 个开发阶段像流水线上的工位一样串联执行。但每个工位内部又采用了**Actor-Critic 自优化模式**（Agent 先干活、再自我审查、根据反馈迭代改进），使得每个阶段的输出质量在自我博弈中不断提升。

这种架构设计解决了一个核心矛盾：AI 生成的内容往往质量不稳定，单一模型的一次输出很难达到专业水准。通过 Actor-Critic 循环，系统让一个 Agent 当"生产者"、另一个当"评审者"，模拟人类团队中的"写代码→Code Review"工作流。

此外，系统采用**六边形架构（Hexagonal Architecture）**作为模块组织原则——核心领域逻辑（domain）零外部依赖，所有基础设施适配器（持久化、LLM、交互界面）都通过 trait 接口与核心解耦。这使得替换任何一个基础设施组件（比如从 CLI 切换到 GUI）都不会影响业务逻辑。

架构模式的第三个关键选择是**策略模式（Strategy）用于阶段行为**——每个开发阶段都实现统一的 `Stage` trait，使得流水线可以在运行时动态组合和替换阶段，而不需要修改流水线引擎本身。

## 核心设计原则

1. **领域驱动设计（DDD）**——核心领域实体（Project、Iteration、Memory）封装了所有业务规则和生命周期逻辑，外部模块只能通过这些实体提供的公共方法与之交互。这解决了"业务逻辑散落在各处"的问题，使得系统行为可预测、可测试。

2. **交互后端抽象（InteractiveBackend trait）**——所有用户交互都通过 `InteractiveBackend` trait 进行，CLI 和 GUI 各自实现这个 trait。这一设计解决了"核心引擎不能同时服务于命令行和图形界面"的问题——Tauri 后端通过 `TauriBackend` 实现，CLI 通过 `CliBackend` 实现，核心代码完全不需要知道用户是用什么界面在交互。

3. **数据驱动配置（ConfigRegistry）**——Agent、Stage、Flow、Integration 的定义全部从硬编码迁移到可配置的 JSON 格式。这使得用户可以不修改代码就定义新的 Agent 角色、自定义工作流、或者集成外部系统。这是一个从"写死的 SDK"到"可配置的平台"的关键架构跃迁。

4. **记忆即基础设施（Memory as Infrastructure）**——项目记忆（Decisions、Patterns、Context）被设计为一级基础设施，跨迭代自动累积。每次迭代完成后的 Knowledge Generation Agent 自动提取关键决策和模式，使得系统"越用越聪明"——后续迭代可以查询前序迭代的知识。

## 技术栈详情

| 层次/领域 | 技术选型 | 选择理由 |
|---------|---------|---------|
| 语言与运行时 | Rust (edition 2024) + Tokio | 内存安全、高性能、零成本抽象，适合 IO 密集型多 Agent 并发 |
| Agent 框架 | adk-rust (adk-core, adk-agent, adk-model) | 提供标准化的 Agent 构建 API，支持 LoopAgent 和流式输出 |
| CLI 框架 | clap (v4, derive) + dialoguer + console | 声明式参数解析，跨平台中文支持良好 |
| GUI 框架 | Tauri 2 + React + TypeScript + Ant Design | 跨平台原生应用，Rust 后端安全，React 前端灵活 |
| 序列化 | serde + serde_json | Rust 生态标准序列化方案，零成本抽象 |
| 持久化 | JSON 文件存储 | 无需数据库，简单可靠，适合桌面工具场景 |
| 速率限制 | TokenBucket 算法 | 允许突发请求同时保证长期速率，比固定延迟更高效 |
| 外部 Agent 集成 | Agent Client Protocol (ACP) | 开放标准协议，支持多种外部编码 Agent |

## 关键数据结构

| 类型名 | 文件路径 | 用途 |
|-------|---------|------|
| `Project` | `crates/cowork-core/src/domain/project.rs:6` | 项目根实体，管理名称、迭代列表、元数据 |
| `Iteration` | `crates/cowork-core/src/domain/iteration.rs:8` | 迭代实体，一个开发周期，包含状态、制品、阶段记录 |
| `Artifacts` | `crates/cowork-core/src/domain/iteration.rs` | 迭代制品集合（idea.md, prd.md, design.md, plan.md 等） |
| `InheritanceMode` | `crates/cowork-core/src/domain/iteration.rs` | 迭代继承模式（None/Full/Partial） |
| `ProjectMemory` | `crates/cowork-core/src/domain/memory.rs:7` | 项目级记忆，跨迭代积累决策、模式、上下文 |
| `PipelineContext` | `crates/cowork-core/src/pipeline/mod.rs:29` | 管道执行上下文，携带项目和迭代信息 |
| `StageResult` | `crates/cowork-core/src/pipeline/mod.rs:19` | 阶段执行结果枚举（Success/Failed/Paused/NeedsRevision/GotoStage） |
| `ConfigRegistry` | `crates/cowork-core/src/config_definition/registry.rs:41` | 全局配置注册表，管理 Agent/Stage/Flow/Integration 定义 |

## 核心接口/Trait/协议

| 名称 | 实现数量 | 核心职责 |
|-----|---------|---------|
| `Stage` trait | 7（Idea/PRD/Design/Plan/Coding/Check/Delivery） | 定义开发阶段的统一接口——每个阶段必须实现 execute 方法 |
| `InteractiveBackend` trait | 2（CliBackend, TauriBackend） | 抽象用户交互方式——消息展示、用户输入、进度通知 |
| `Llm` trait | 2（真实 LLM + TokenBucketRateLimiter 装饰器） | LLM 调用抽象，RateLimiter 作为装饰器透明添加速率限制 |
| `Agent` trait (adk_core) | 多个（LoopAgent, LlmAgentBuilder 产物） | 统一的 Agent 执行接口，支持流式输出 |

## 架构决策记录

- **决策1**：选择了 Actor-Critic 自优化循环，放弃了单次 Agent 输出，因为 AI 单次生成内容质量不稳定，自我博弈能显著提升输出质量。观察依据：`crates/cowork-core/src/agents/mod.rs:68-99`（PRD Loop 的 Actor+Cirtic 构建）
- **决策2**：选择了 JSON 文件持久化，放弃了关系型数据库，因为桌面工具场景不需要多用户并发访问，JSON 文件简单可靠、无需运维。观察依据：`crates/cowork-core/src/persistence/mod.rs`（基于文件的 ProjectStore/IterationStore/MemoryStore）
- **决策3**：选择了 TokenBucket 速率限制，放弃了固定延迟方案，因为允许突发请求可以在需要时快速响应，同时保证长期平均速率不超过限制。观察依据：`crates/cowork-core/src/llm/rate_limiter.rs:32-60`
- **决策4**：选择了 InteractiveBackend trait 抽象交互，放弃了 CLI 和 GUI 直接调用核心 API，因为核心引擎需要同时支持 CLI 和 GUI 而不感知具体交互方式。观察依据：`crates/cowork-core/src/interaction/mod.rs:108-160`
- **决策5**：选择了 LoopAgent + max_iterations=1 解决 SequentialAgent 终止 bug，放弃了修复 adk-rust 框架本身，因为框架边界决策更稳妥。观察依据：`crates/cowork-core/src/agents/mod.rs:4-10`（Bug 注释）
