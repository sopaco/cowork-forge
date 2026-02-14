# Cowork Forge 架构设计

## 1. 架构概述

Cowork Forge 采用分层 + 模块化架构，整体分为三个核心层次：入口层、核心引擎层和交互层。这种架构设计确保了系统的高内聚低耦合，便于扩展和维护。

## 2. 容器图

```mermaid
C4Container
    title Container Diagram for Cowork Forge

    Person(user, "User", "软件开发者或产品负责人")
    
    Container_Boundary(c1, "Cowork Forge") {
        Container(cowork_cli, "Cowork CLI", "Rust + Clap", "命令行交互入口")
        Container(cowork_gui, "Cowork GUI", "React + Tauri", "图形用户界面")
        Container(cowork_core, "Cowork Core", "Rust + Tokio", "核心业务引擎")
    }
    
    ContainerDb(persistence, "File System", "Local Disk", "项目文件和迭代数据存储")
    ContainerDb(llm_config, "Config File", "TOML", "LLM API配置存储")
    
    System_Ext(llm, "LLM Provider", "OpenAI/Claude API", "大语言模型服务")
    System_Ext(terminal, "Terminal", "System", "执行编译运行命令")
    System_Ext(browser, "Browser", "Web", "前端页面预览")

    Rel(user, cowork_cli, "使用", "CLI命令")
    Rel(user, cowork_gui, "使用", "GUI交互")
    
    Rel(cowork_cli, cowork_core, "调用核心功能", "IPC/Tauri")
    Rel(cowork_gui, cowork_core, "调用核心功能", "IPC/Tauri")
    
    Rel(cowork_core, persistence, "读写项目文件", "File API")
    Rel(cowork_core, llm_config, "读取配置", "File API")
    Rel(cowork_core, llm, "调用LLM API", "HTTP/REST")
    Rel(cowork_core, terminal, "执行命令", "Process")
    Rel(cowork_gui, browser, "预览页面", "HTTP")
```

## 3. 核心组件架构

### 3.1 工作空间结构

```mermaid
graph TB
    subgraph cowork-core
        A[CLI/GUI Interface] --> B[Pipeline Executor]
        B --> C[Stage Executor]
        C --> D[Agent System]
        D --> E[Tool Registry]
        E --> F[File Operations]
        E --> G[Command Execution]
        E --> H[Code Analysis]
    end
    
    subgraph Data Layer
        I[Iteration Store] --> J[File System]
        K[Project Store] --> J
        L[Memory Store] --> J
    end
    
    subgraph LLM Layer
        M[LLM Config] --> N[Rate Limiter]
        N --> O[HTTP Client]
        O --> P[LLM Provider]
    end
    
    D --- M
    E --- I
    E --- K
```

### 3.2 模块职责

| 模块 | 职责 | 关键类/函数 |
|------|------|-------------|
| `pipeline` | 迭代流程编排 | `PipelineExecutor`, `StageExecutor` |
| `agents` | AI智能体管理 | `IterativeAssistant`, `StageAgent` |
| `tools` | 工具注册与执行 | `ToolRegistry`, 各类Tool |
| `domain` | 领域实体定义 | `Iteration`, `Project`, `Memory` |
| `persistence` | 数据持久化 | `IterationStore`, `ProjectStore` |
| `llm` | LLM调用封装 | `LLMClient`, `RateLimiter` |
| `instructions` | 智能体提示词 | 各阶段指令模板 |
| `interaction` | 交互后端抽象 | `CliBackend`, `TauriBackend` |

## 4. 核心技术选型

### 4.1 技术栈矩阵

| 层次 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 入口层 | Rust CLI | stable | 命令行工具 |
| 入口层 | Tauri | 2.x | 桌面应用框架 |
| 入口层 | React | 18.x | UI框架 |
| 核心层 | Rust | stable | 核心业务逻辑 |
| 核心层 | Tokio | 1.x | 异步运行时 |
| 核心层 | adk-rust | 0.2.x | AI智能体框架 |
| 数据层 | JSON | - | 结构化数据存储 |
| 数据层 | TOML | - | 配置文件格式 |

### 4.2 架构模式

**分层架构**:
- **入口层**: 处理用户输入，解析命令或GUI事件
- **核心层**: 包含所有业务逻辑，处理迭代执行
- **数据层**: 负责数据持久化和状态管理
- **交互层**: 抽象不同交互后端（CLI/GUI）

**异步架构**:
- 使用 Tokio 异步运行时处理 I/O 密集型任务
- LLM API 调用完全异步化
- 文件操作和进程管理采用异步 API

**事件驱动**:
- 智能体事件通过事件总线传播
- 前端通过 Tauri 事件监听后端状态变化
- 支持实时日志流式输出

## 5. 数据流设计

### 5.1 迭代执行数据流

```mermaid
flowchart LR
    subgraph Input
        A[User Request] --> B[CLI/GUI]
    end
    
    subgraph Core
        B --> C[Pipeline Executor]
        C --> D{Stage Loop}
        D -->|Idea| E[IdeaStage]
        D -->|PRD| F[PRDStage]
        D -->|Design| G[DesignStage]
        D -->|Plan| H[PlanStage]
        D -->|Coding| I[CodingStage]
        D -->|Check| J[CheckStage]
        D -->|Delivery| K[DeliveryStage]
    end
    
    subgraph Agent
        E --> L[Create Agent]
        L --> M[Execute Agent]
        M --> N{HITL?}
        N -->|Yes| O[Wait User Confirm]
        O --> M
        N -->|No| P[Next Stage]
    end
    
    subgraph Output
        P --> Q[Save to Store]
        Q --> R[Update UI]
    end
```

### 5.2 状态管理

```mermaid
stateDiagram-v2
    [*] --> ProjectCreated
    ProjectCreated --> IterationCreated
    IterationCreated --> IdeaStage
    IdeaStage --> PRDDraft
    PRDDraft --> DesignStage
    DesignStage --> PlanStage
    PlanStage --> CodingStage
    CodingStage --> CheckStage
    CheckStage --> DeliveryStage
    DeliveryStage --> [*]
    
    IdeaStage --> UserConfirm: HITL
    PRDDraft --> UserConfirm: HITL
    DesignStage --> UserConfirm: HITL
    CodingStage --> UserConfirm: HITL
    
    UserConfirm --> IterationCreated: Approved
    UserConfirm --> [*]: Rejected
```

## 6. 扩展性设计

### 6.1 工具扩展

系统采用插件化工具注册机制，新增工具只需：

1. 实现 `Tool` trait
2. 在工具注册表中添加实例
3. 定义工具描述和参数模式

### 6.2 智能体扩展

新增智能体类型需要：

1. 在 `agents` 模块中添加智能体实现
2. 在 `instructions` 中定义提示词模板
3. 在 `pipeline/stages` 中集成新阶段

### 6.3 项目类型支持

通过 `TechStack` 抽象支持新项目类型：

- 检测项目特征（文件结构、配置文件）
- 匹配对应的生成器
- 生成类型感知的运行配置

## 7. 安全架构

### 7.1 运行时安全

```mermaid
graph TB
    subgraph Security
        A[Runtime Security Checker] --> B[Path Validation]
        A --> C[Command Whitelist]
        A --> D[Resource Limits]
        
        B --> E[Block: Path Traversal]
        C --> F[Block: Dangerous Commands]
        D --> G[Block: Resource Exhaustion]
    end
    
    subgraph Input
        H[LLM Generated Config] --> A
    end
```

### 7.2 安全策略

- **路径验证**: 防止路径遍历攻击
- **命令白名单**: 只允许安全的系统命令
- **资源限制**: 防止恶意代码耗尽系统资源
- **沙箱执行**: 代码生成在隔离环境中进行
