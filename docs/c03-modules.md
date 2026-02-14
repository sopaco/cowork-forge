# Cowork Forge 模块与功能流程

## 1. 模块概览

Cowork Forge 的核心功能由多个模块协同完成，每个模块承担特定职责。本文档详细介绍各模块的功能及其交互流程。

## 2. 模块结构

```mermaid
graph TB
    subgraph "Entry Layer"
        CLI[CLI Entry] --> Core
        GUI[GUI Entry] --> Core
    end
    
    subgraph "cowork-core"
        Core[Pipeline Executor]
        
        subgraph "Agent System"
            Core --> Agents[Agents Module]
            Agents --> Iterative[Iterative Assistant]
            Agents --> Stage[Stage Agent]
        end
        
        subgraph "Tools"
            Core --> Tools[Tool Registry]
            Tools --> FileOps[File Operations]
            Tools --> CmdExec[Command Execution]
            Tools --> CodeAnalysis[Code Analysis]
            Tools --> Validation[Validation Tools]
            Tools --> Knowledge[Knowledge Tools]
        end
        
        subgraph "Domain"
            Core --> Domain[Domain Models]
            Domain --> Iteration[Iteration]
            Domain --> Project[Project]
            Domain --> Memory[Memory]
        end
        
        subgraph "Persistence"
            Core --> Persistence[Persistence Layer]
            Persistence --> IterStore[Iteration Store]
            Persistence --> ProjStore[Project Store]
            Persistence --> MemStore[Memory Store]
        end
    end
    
    subgraph "Infrastructure"
        LLM[LLM Provider]
        FS[File System]
    end
    
    Core --> LLM
    Core --> FS
```

## 3. 核心模块详解

### 3.1 Pipeline 模块

Pipeline 模块是整个系统的调度中心，负责协调各个阶段的执行。

```mermaid
sequenceDiagram
    participant User
    participant Pipeline
    participant Stage
    participant Agent
    participant Tools
    participant Store
    
    User->>Pipeline: start_iteration(request)
    
    Pipeline->>Store: 创建新迭代
    Store-->>Pipeline: iteration_id
    
    Loop For each stage
        Pipeline->>Stage: execute(stage_name, iteration)
        
        Stage->>Agent: 创建智能体实例
        Agent->>Tools: 注册可用工具
        
        Loop While not complete
            Agent->>LLM: 生成决策/代码
            LLM-->>Agent: response
            
            Agent->>Tools: 执行工具操作
            Tools->>Store: 保存中间结果
            
            Agent->>Agent: 评估结果
            Agent->>Stage: 需要确认?
            
            Note over Stage,Agent: HITL检查点
        end
        
        Stage-->>Pipeline: stage_result
    end
    
    Pipeline-->>User: iteration_complete
```

### 3.2 Agents 模块

Agents 模块实现了多种专业AI智能体，用于不同阶段的软件开发任务。

```mermaid
classDiagram
    class Agent {
        <<abstract>>
        +execute(input) AgentResult
        +register_tools(tools)
    }
    
    class IterativeAssistant {
        +execute(input) AgentResult
        -critique_loop()
        -improve_output()
    }
    
    class StageAgent {
        +execute_for_stage(stage) AgentResult
        -prepare_context()
        -process_artifacts()
    }
    
    Agent <|-- IterativeAssistant
    Agent <|-- StageAgent
    
    class IdeaAgent {
        +generate_idea() IdeaArtifact
    }
    class PrdAgent {
        +generate_prd() PrdArtifact
    }
    class DesignAgent {
        +generate_design() DesignArtifact
    }
    class PlanAgent {
        +generate_plan() PlanArtifact
    }
    class CodingAgent {
        +generate_code() CodeArtifact
    }
    class CheckAgent {
        +verify_code() CheckReport
    }
    class DeliveryAgent {
        +deliver_project() DeliveryReport
    }
    
    StageAgent <|-- IdeaAgent
    StageAgent <|-- PrdAgent
    StageAgent <|-- DesignAgent
    StageAgent <|-- PlanAgent
    StageAgent <|-- CodingAgent
    StageAgent <|-- CheckAgent
    StageAgent <|-- DeliveryAgent
```

### 3.3 Tools 模块

Tools 模块提供了智能体执行任务所需的各种工具能力。

```mermaid
graph LR
    subgraph Tools Category
        FileOps[文件操作工具] --> Read[读取文件]
        FileOps --> Write[写入文件]
        FileOps --> Glob[文件搜索]
        
        CmdExec[命令执行工具] --> RunCmd[运行命令]
        CmdExec --> Install[安装依赖]
        CmdExec --> Build[编译构建]
        
        CodeAnalysis[代码分析工具] --> Parse[解析代码]
        CodeAnalysis --> Lint[代码检查]
        
        Validation[验证工具] --> Schema[模式验证]
        Validation --> Logic[逻辑验证]
        
        Knowledge[知识工具] --> Query[查询知识]
        Knowledge --> Store[存储知识]
    end
```

### 3.4 Domain 模块

Domain 模块定义了系统的核心领域模型。

```mermaid
erDiagram
    Project ||--o{ Iteration : contains
    Iteration {
        string id PK
        string title
        string description
        string status
        string current_stage
        string inheritance
        datetime created_at
    }
    
    Iteration ||--o{ Artifact : produces
    Artifact {
        string id PK
        string iteration_id FK
        string stage
        string content
        string artifact_type
    }
    
    Iteration ||--o{ Memory : has
    Memory {
        string id PK
        string iteration_id FK
        string scope
        string content_type
        string content
    }
```

## 4. 迭代执行流程

### 4.1 七阶段流水线

Cowork Forge 采用七阶段迭代模型，每个阶段有明确的输入输出：

```mermaid
flowchart LR
    subgraph Stage1[Idea 阶段]
        I1[用户需求] --> I2[AI生成创意]
        I2 --> I3[创意文档]
    end
    
    subgraph Stage2[PRD 阶段]
        P1[创意文档] --> P2[AI分析需求]
        P2 --> P3[需求规格说明]
    end
    
    subgraph Stage3[Design 阶段]
        D1[需求规格] --> D2[AI设计架构]
        D2 --> D3[技术设计方案]
    end
    
    subgraph Stage4[Plan 阶段]
        Pl1[设计方案] --> Pl2[AI规划任务]
        Pl2 --> Pl3[实现计划]
    end
    
    subgraph Stage5[Coding 阶段]
        C1[实现计划] --> C2[AI生成代码]
        C2 --> C3[代码产物]
    end
    
    subgraph Stage6[Check 阶段]
        Ch1[代码产物] --> Ch2[AI检查验证]
        Ch2 --> Ch3[检查报告]
    end
    
    subgraph Stage7[Delivery 阶段]
        De1[检查报告] --> De2[AI整理交付]
        De2 --> De3[最终交付物]
    end
    
    I3 --> P1
    P3 --> D1
    D3 --> Pl1
    Pl3 --> C1
    C3 --> Ch1
    Ch3 --> De1
```

### 4.2 Actor-Critic 模式

在 PRD、Design、Plan、Coding 阶段，采用 Actor-Critic 双智能体循环优化：

```mermaid
flowchart TB
    subgraph ActorCritic
        A[Actor Agent] -->|生成内容| B
        B{是否满足要求?}
        B -->|否| C[Critic Agent]
        C -->|反馈意见| A
        B -->|是| D[输出结果]
    end
    
    subgraph HITL
        E[需要用户确认?] -->|是| F[等待用户输入]
        F --> A
        E -->|否| D
    end
```

## 5. 数据持久化流程

### 5.1 迭代数据存储

```mermaid
flowchart LR
    subgraph Write
        A[迭代执行] --> B[生成Artifacts]
        B --> C[Iteration Store]
        C --> D[JSON文件]
    end
    
    subgraph Read
        E[查询请求] --> F[Iteration Store]
        F --> G[读取JSON]
        G --> H[返回数据]
    end
```

### 5.2 内存系统

系统维护两类内存：

- **执行内存**: 当前迭代的上下文和决策记录
- **项目内存**: 跨迭代的知识积累

```mermaid
flowchart TB
    subgraph MemorySystem
        A[LLM请求] --> B{查询类型}
        
        B -->|决策相关| C[Decisions]
        B -->|模式相关| D[Patterns]
        B -->|洞察相关| E[Insights]
        B -->|全部| F[Smart Query]
        
        C --> G[Memory Store]
        D --> G
        E --> G
        F --> G
        
        G --> H[语义索引]
        H --> I[返回相关记忆]
    end
```

## 6. 错误处理流程

```mermaid
flowchart TB
    A[执行任务] --> B{成功?}
    B -->|是| C[继续执行]
    B -->|否| D{可重试?}
    
    D -->|是| E[增加重试计数]
    E --> F{未超过上限?}
    F -->|是| A
    F -->|否| G[标记失败]
    
    D -->|否| G
    
    G --> H[记录错误]
    H --> I[通知用户]
    I --> J[等待用户决策]
```
