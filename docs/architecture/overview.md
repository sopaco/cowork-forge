# Cowork Forge 架构概览

## 项目简介

Cowork Forge 是一个完整的 AI 驱动软件开发团队系统，通过模拟真实开发团队的角色分工，实现从创意到交付的全链路智能化开发。

### 核心功能

- **7 阶段开发工作流**：Idea → PRD → Design → Plan → Coding → Check → Delivery
- **专业化 AI Agents**：8 个专业智能体，每个阶段都有专门的角色处理
- **迭代驱动架构**：支持 Genesis（起源）和 Evolution（演化）两种迭代类型
- **人机协作验证**：关键决策点保留人工确认机制（HITL）
- **增量代码更新**：智能增量分析，只更新受影响文件
- **文件安全机制**：所有文件操作限制在 iteration workspace 内

### 设计理念

Cowork Forge 的核心理念是"**迭代驱动 + 人机协作**"。系统通过模拟真实开发团队的角色分工和协作流程，将大型语言模型的能力结构化、流程化，使其能够像人类开发团队一样完成完整的软件开发任务。

**关键设计原则**：

1. **迭代驱动**：将软件开发抽象为可管理、可继承、可演进的迭代单元
2. **角色分工**：每个阶段都有专门的角色处理，职责清晰
3. **质量保证**：Actor-Critic 模式确保产出质量
4. **人机协作**：关键决策点保留人工参与
5. **安全隔离**：所有文件操作限制在 workspace 内
6. **版本化管理**：提供版本化的制品管理和上下文传递

## 架构分层

### 系统架构图

```
┌─────────────────────────────────────────────────────────┐
│                    用户接口层                            │
│   ┌──────────────┐         ┌──────────────┐            │
│   │   CLI 界面   │         │   GUI 界面   │            │
│   └──────────────┘         └──────────────┘            │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  交互抽象层                             │
│              (InteractiveBackend)                       │
│   ┌──────────────┐         ┌──────────────┐            │
│   │  CLI Backend │         │  GUI Backend │            │
│   └──────────────┘         └──────────────┘            │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   核心服务层                            │
│   ┌─────────────────────────────────────────────────┐   │
│   │          IterationExecutor                      │   │
│   │    (迭代执行器 - 管理完整生命周期)              │   │
│   └─────────────────────────────────────────────────┘   │
│                          ↓                               │
│   ┌─────────────────────────────────────────────────┐   │
│   │            Pipeline                             │   │
│   │       (工作流编排 - 阶段序列管理)                │   │
│   └─────────────────────────────────────────────────┘   │
│                          ↓                               │
│   ┌─────────────────────────────────────────────────┐   │
│   │          StageExecutor                          │   │
│   │       (阶段引擎 - 单阶段执行逻辑)                │   │
│   └─────────────────────────────────────────────────┘   │
│                          ↓                               │
│   ┌─────────────────────────────────────────────────┐   │
│   │         Agent Runtime                           │   │
│   │       (Agent 运行时 - 基于 adk-rust)             │   │
│   └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   领域模型层                            │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│   │   Project    │  │  Iteration   │  │   Memory     │ │
│   │  项目管理    │  │  迭代控制    │  │  记忆管理    │ │
│   └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   基础设施层                            │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│   │  LLM 服务    │  │  文件存储    │  │  配置管理    │ │
│   └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 层级职责

| 层级 | 职责 | 关键组件 |
|------|------|----------|
| **用户接口层** | 提供用户交互界面 | CLI、GUI (Tauri) |
| **交互抽象层** | 统一交互接口，支持多种前端 | InteractiveBackend |
| **核心服务层** | 核心业务逻辑和流程编排 | IterationExecutor、Pipeline、StageExecutor |
| **领域模型层** | 业务领域实体和规则 | Project、Iteration、Memory |
| **基础设施层** | 技术支撑服务 | LLM、文件存储、配置管理 |

## 技术栈

### 后端技术栈

| 技术领域 | 技术选型 | 版本要求 |
|---------|---------|---------|
| **语言** | Rust | 1.70+ |
| **AI 框架** | adk-rust | 最新版 |
| **异步运行时** | tokio | 1.0+ |
| **序列化** | serde | 1.0+ |
| **CLI** | clap | 4.0+ |

### 前端技术栈

| 技术领域 | 技术选型 | 版本要求 |
|---------|---------|---------|
| **桌面框架** | Tauri | 2.0+ |
| **构建工具** | Vite | 5.0+ |
| **运行时** | bun | 1.0+ |

### LLM 服务

- 支持多种 LLM 提供商（OpenAI、Anthropic、本地模型等）
- 速率限制和错误重试机制
- 上下文管理优化

## 目录结构

### 项目根目录

```
cowork-forge/
├── crates/                  # Cargo 工作空间
│   ├── cowork-cli/         # CLI 命令行接口
│   ├── cowork-core/        # 核心业务逻辑
│   └── cowork-gui/         # GUI 桌面应用
├── docs/                   # 项目文档
├── assets/                 # 静态资源
├── Cargo.toml             # 工作空间配置
└── README.md              # 项目说明
```

### cowork-core 目录结构

```
cowork-core/
├── src/
│   ├── domain/            # 领域模型
│   │   ├── iteration.rs   # 迭代实体
│   │   ├── memory.rs      # 记忆系统
│   │   └── project.rs     # 项目实体
│   ├── pipeline/          # 工作流编排
│   │   ├── executor.rs    # 迭代执行器
│   │   ├── stage_executor.rs  # 阶段执行器
│   │   └── stages/        # 各阶段实现
│   ├── agents/            # Agent 构建器
│   │   └── mod.rs         # Agent 配置
│   ├── instructions/      # Agent 指令模板
│   │   ├── idea.rs        # Idea Agent 指令
│   │   ├── prd.rs         # PRD Agent 指令
│   │   ├── design.rs      # Design Agent 指令
│   │   ├── plan.rs        # Plan Agent 指令
│   │   ├── coding.rs      # Coding Agent 指令
│   │   ├── check.rs       # Check Agent 指令
│   │   └── delivery.rs    # Delivery Agent 指令
│   ├── tools/             # 工具实现
│   │   ├── file_tools.rs      # 文件操作工具
│   │   ├── deployment_tools.rs # 部署工具
│   │   ├── data_tools.rs      # 数据管理工具
│   │   ├── validation_tools.rs # 验证工具
│   │   └── mod.rs              # 工具模块
│   ├── llm/               # LLM 集成
│   │   ├── config.rs      # LLM 配置
│   │   └── rate_limiter.rs    # 速率限制
│   ├── data/              # 数据模型
│   │   ├── models.rs      # 数据结构
│   │   └── schemas/       # 数据模式
│   ├── persistence/       # 持久化
│   │   ├── iteration_store.rs
│   │   ├── memory_store.rs
│   │   └── project_store.rs
│   ├── interaction/       # 交互抽象
│   │   ├── cli.rs         # CLI 实现
│   │   └── tauri.rs       # Tauri 实现
│   └── storage/           # 文件存储
└── Cargo.toml
```

### .cowork-v2 数据目录

```
.cowork-v2/
├── iterations/              # 迭代数据
│   └── {iteration_id}/      # 每个迭代一个目录
│       ├── iteration.json   # 迭代元数据
│       ├── artifacts/       # 迭代制品
│       │   ├── idea.md
│       │   ├── prd.md
│       │   ├── design.md
│       │   ├── plan.md
│       │   └── delivery.md
│       ├── data/            # 结构化数据
│       │   ├── requirements.json
│       │   ├── feature_list.json
│       │   ├── design_spec.json
│       │   └── implementation_plan.json
│       ├── session/         # 会话数据
│       │   ├── meta.json
│       │   └── feedback.json
│       ├── workspace/       # 代码工作空间
│       │   ├── src/
│       │   ├── components/
│       │   └── ...
│       └── logs/            # 日志文件
└── memory/                  # 记忆系统
    ├── project/             # 项目级记忆
    └── iterations/          # 迭代级记忆
```

## 核心概念

### Iteration（迭代）

Iteration 是 Cowork Forge 的核心概念，代表一个完整的开发周期。每个 Iteration 包含：

- **独立的制品**：idea.md、prd.md、design.md、plan.md、delivery.md
- **结构化数据**：requirements.json、design_spec.json、implementation_plan.json
- **代码工作空间**：workspace 目录，存放生成的代码
- **会话数据**：meta.json、feedback.json
- **记忆系统**：迭代级记忆，记录关键决策和学习内容

### Iteration 类型

| 类型 | 说明 | 适用场景 |
|------|------|---------|
| **Genesis** | 起源迭代，从零开始 | 新项目创建 |
| **Evolution** | 演化迭代，基于现有迭代 | 功能增强、Bug 修复 |

### Inheritance Mode（继承模式）

演化迭代支持三种继承模式：

| 模式 | 说明 | 使用场景 |
|------|------|---------|
| **None** | 不继承，全新开始 | 需要完全重写的场景 |
| **Full** | 完全继承，复制所有 workspace 文件 | 小幅修改、Bug 修复 |
| **Partial** | 部分继承，只复制 artifacts 和配置 | 大幅修改、架构调整 |

### Agent 角色

Cowork Forge 有 8 个专业智能体，每个负责不同的阶段：

| Agent | 类型 | 职责 |
|-------|------|------|
| **Idea Agent** | 单一 Agent | 理解用户创意，生成 idea.md |
| **PRD Loop** | LoopAgent (Actor + Critic) | 生成产品需求文档，创建结构化需求 |
| **Design Loop** | LoopAgent (Actor + Critic) | 设计系统架构，创建设计规范 |
| **Plan Loop** | LoopAgent (Actor + Critic) | 制定实施计划，分解任务 |
| **Coding Loop** | LoopAgent (Actor + Critic) | 生成代码，实现功能 |
| **Check Agent** | 单一 Agent | 质量检查，验证功能完整性 |
| **Delivery Agent** | 单一 Agent | 生成交付报告，部署代码 |

### 工具体系

Cowork Forge 使用专用工具体系，遵循权限最小化原则：

| 工具类型 | 工具示例 | 使用场景 |
|---------|---------|---------|
| **Artifact 工具** | SaveIdeaTool、SavePrdDocTool | 保存文档 |
| **文件工具** | ReadFileTool、WriteFileTool | 操作代码文件 |
| **命令工具** | RunCommandTool | 执行构建和测试 |
| **数据工具** | CreateRequirementTool | 管理结构化数据 |
| **验证工具** | CheckTestsTool、CheckLintTool | 质量验证 |
| **部署工具** | CopyWorkspaceToProjectTool | 部署代码 |

## 安全机制

### 文件安全

- **Workspace 隔离**：所有文件操作限制在 iteration workspace 内
- **相对路径验证**：只接受相对路径，拒绝绝对路径和 `..` 访问
- **路径安全检查**：验证路径是否在 workspace 目录内
- **Delivery 同步**：只在 Delivery 阶段将代码复制到项目根目录

### 权限控制

- **权限最小化**：非编码阶段不提供通用的文件读写权限
- **工具专用化**：每个工具只完成特定功能
- **参数验证**：所有工具使用安全的参数提取函数
- **错误处理**：缺失参数时返回清晰的错误信息

### 命令安全

- **命令白名单**：阻止危险命令（rm -rf、sudo 等）
- **超时控制**：命令执行 30 秒超时
- **阻塞检测**：自动拒绝长时间运行的命令

## 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/sopaco/cowork-forge.git
cd cowork-forge

# 构建项目
cargo build --release
```

### 运行 CLI

```bash
# 创建新项目
cargo run --release -- new my-project

# 启动迭代
cargo run --release -- start

# 查看迭代状态
cargo run --release -- status
```

### 运行 GUI

```bash
# 启动 GUI 应用
cargo run --release --manifest-path=crates/cowork-gui/Cargo.toml
```

## 相关文档

- [Agent 系统](./agent-system.md)
- [迭代架构](./iteration-architecture.md)
- [Pipeline 流程](./pipeline.md)
- [文件安全机制](./file-security.md)
- [Artifacts 验证](./artifacts-validation.md)
- [领域模型](../development/domain.md)
- [工具系统](../development/tools.md)