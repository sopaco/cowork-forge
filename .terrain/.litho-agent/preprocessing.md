# 预处理报告

## 项目基本信息
- **项目名称**：Cowork Forge
- **版本**：2.5.2
- **项目类型**：AI-native 多 Agent 软件开发平台（CLI + GUI）
- **主要编程语言**：Rust（主要）, TypeScript + React（GUI 前端）
- **核心框架/运行时**：adk-rust（Agent Development Kit）, Tokio 异步运行时, Tauri 桌面框架

## 技术栈
- **运行时**：Rust (edition 2024), Node.js (GUI 前端构建)
- **Web 框架**：无（Tauri 内置 WebView）
- **数据库**：无关系型数据库，采用 JSON 文件持久化
- **LLM/AI 集成**：OpenAI-compatible API, TokenBucketRateLimiter（30 req/min, concurrency=1）
- **主要依赖库**：
  - `adk-rust / adk-core / adk-agent / adk-model` — ADK Agent 框架
  - `tokio`（full features）— 异步运行时
  - `clap` + `dialoguer` — CLI 交互
  - `serde / serde_json` — 序列化
  - `tracing` — 日志
  - `agent-client-protocol` — 外部 Agent 集成（ACP）

## 目录结构摘要
```
cowork-forge/
├── crates/
│   ├── cowork-core/src/     # 核心逻辑（领域、管道、工具、Agent）
│   │   ├── pipeline/        # 7 阶段开发流水线编排
│   │   ├── domain/          # 核心领域实体（Project, Iteration, Memory）
│   │   ├── tools/           # 30+ ADK 工具（文件、命令、验证等）
│   │   ├── agents/          # Agent 构建器（LoopAgent + Actor-Critic）
│   │   ├── llm/             # LLM 集成 + 速率限制
│   │   ├── config_definition/  # 数据驱动配置（Agent/Stage/Flow/Integration）
│   │   ├── interaction/     # InteractiveBackend trait（CLI/GUI 抽象）
│   │   ├── persistence/     # JSON 文件存储（Project/Iteration/Memory）
│   │   ├── instructions/    # Agent 提示词库
│   │   ├── acp/             # Agent Client Protocol 外部 Agent 集成
│   │   ├── skills/          # agentskills.io 标准技能系统
│   │   ├── integration/     # 外部集成 Hook 管理器
│   │   ├── importer/        # 遗留项目导入分析器
│   │   └── data/            # 数据模型
│   ├── cowork-cli/src/      # CLI 适配器（clap + dialoguer）
│   │   └── commands/        # CLI 命令实现（11 个命令）
│   └── cowork-gui/          # Tauri + React GUI
│       ├── src-tauri/src/   # Rust 后端（Tauri 命令 + 事件）
│       └── src/             # 前端（TypeScript + Ant Design）
├── .terrain/                # Terrain AI 工程环境
├── litho.docs/              # 已有的 Litho 文档
└── assets/                  # 资源文件
```

## 识别到的核心模块
| 模块名称 | 路径 | DDD 分类 | 职责描述 |
|---------|------|---------|---------|
| pipeline | `crates/cowork-core/src/pipeline/` | 核心域 | 7 阶段开发流水线编排（Idea→PRD→Design→Plan→Coding→Check→Delivery） |
| domain | `crates/cowork-core/src/domain/` | 通用域 | 核心领域实体：Project、Iteration、Memory 聚合 |
| agents | `crates/cowork-core/src/agents/` | 核心域 | AI Agent 构建器，包含 Idea/PRD/Design/Plan/Coding/Check/Delivery Agent |
| tools | `crates/cowork-core/src/tools/` | 支撑域 | 30+ ADK 工具实现（文件操作、命令执行、验证、Memory 等） |
| llm | `crates/cowork-core/src/llm/` | 支撑域 | LLM 客户端集成与 TokenBucket 速率限制 |
| config_definition | `crates/cowork-core/src/config_definition/` | 支撑域 | 数据驱动配置系统（Agent/Stage/Flow/Integration 定义） |
| interaction | `crates/cowork-core/src/interaction/` | 通用域 | InteractiveBackend trait（CLI/GUI 抽象） |
| persistence | `crates/cowork-core/src/persistence/` | 支撑域 | JSON 文件持久化（Project/Iteration/Memory 存储） |
| instructions | `crates/cowork-core/src/instructions/` | 支撑域 | Agent 提示词库（每个阶段的 Actor/Critic 指令） |
| acp | `crates/cowork-core/src/acp/` | 支撑域 | Agent Client Protocol 外部 Agent 集成 |
| skills | `crates/cowork-core/src/skills/` | 通用域 | agentskills.io 标准技能系统（SkillManager、SkillInjector） |
| integration | `crates/cowork-core/src/integration/` | 支撑域 | 外部集成 Hook 管理器（REST Adapter + Webhook） |
| importer | `crates/cowork-core/src/importer/` | 核心域 | 遗留项目导入分析器（反向工程生成文档） |
| data | `crates/cowork-core/src/data/` | 通用域 | 数据模型定义 |
| project_runtime | `crates/cowork-core/src/project_runtime.rs` | 支撑域 | 项目运行时配置（GUI Preview/Run） |
| cowork-cli | `crates/cowork-cli/src/` | 核心域 | CLI 命令行接口 |
| cowork-gui | `crates/cowork-gui/` | 核心域 | Tauri + React 图形界面 |

## 关键文件清单
- **入口文件**：`crates/cowork-cli/src/main.rs`、`crates/cowork-core/src/lib.rs`、`crates/cowork-gui/src-tauri/src/main.rs`
- **核心抽象**：`crates/cowork-core/src/interaction/mod.rs`（InteractiveBackend trait）、`crates/cowork-core/src/pipeline/mod.rs`（Stage trait）
- **数据类型**：`crates/cowork-core/src/domain/project.rs`（Project）、`crates/cowork-core/src/domain/iteration.rs`（Iteration）、`crates/cowork-core/src/domain/memory.rs`（ProjectMemory）

## 依赖关系摘要
- **pipeline → domain, agents, llm, config_definition, interaction, persistence**（管道编排需要所有模块）
- **agents → instructions, tools, domain**（Agent 构建需要指令、工具、数据）
- **config_definition → (无核心依赖，自成体系)**（配置定义模块）
- **tools → domain, persistence**（工具需要访问领域数据和持久化）
- **acp → (无核心依赖)**（外部 Agent 协议客户端）
- **importer → domain, persistence**（导入器需要创建项目和迭代）

## README 核心内容
Cowork Forge 是一个完整的 AI 驱动虚拟开发团队系统。AI Agent 扮演产品经理、架构师、项目经理和工程师的角色，通过 7 阶段流水线协作完成从构思到交付的全流程。核心特性包括：Actor-Critic 自优化模式、人类在环验证、增量代码更新、多语言项目支持、外部 ACP Agent 集成、PVS（Project Version System）迭代架构。

## 注意事项
- 采用六边形（Hexagonal）架构，domain 层零外部依赖
- 所有用户交互通过 `InteractiveBackend` trait 抽象，CLI 和 GUI 各自实现
- 速率限制使用 TokenBucket 算法（5 突发令牌，30 req/min）
- 无关系型数据库，使用 JSON 文件持久化
