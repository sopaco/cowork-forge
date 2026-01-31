# Cowork Forge System Architecture Document

**Version**: 1.0
**Generated**: 2025-01-20
**Document Status**: Official Release

---

## 1. Architecture Overview

### 1.1 Architecture Design Philosophy

Cowork Forge adopts a **Human-in-the-Loop First** design philosophy, building an AI agent automation system for the software development lifecycle (SDLC). Its core architecture philosophy includes:

| Design Principle | Technical Implementation | Business Value |
|-----------------|--------------------------|----------------|
| **Simplicity First** | Instruction template hard constraints (2-4 components, 5-12 tasks limit) | Avoid AI over-engineering, ensure generated code maintainability |
| **Human-in-the-Loop (HITL)** | ResilientAgent resilient layer + mandatory review nodes | Human control at key quality nodes, preventing automation runaway |
| **Resilient Fault Tolerance** | Three-level retry mechanism + human decision intervention | Ensures long-process stability, supports checkpoint recovery |
| **Traceability** | Session isolation + file system persistence | Complete decision history recording, supports version rollback |

### 1.2 Core Architecture Patterns

The system adopts a **Layered Multi-Domain Architecture**, implementing multi-agent collaboration based on the Actor-Critic reinforcement learning paradigm:

1. **Actor-Critic Collaboration Mode**: Each key stage is equipped with dual agents (Actor generates content, Critic verifies quality), ensuring output quality through feedback loops
2. **Pipeline Orchestration Mode**: StageExecutor implements stage isolation, supporting sequential execution and conditional branches
3. **Tool Abstraction Mode**: Standardized tool interfaces (adk_core::Tool), enabling extensible AI capabilities
4. **Session Isolation Mode**: UUID-based session space, supporting multi-project parallel development

### 1.3 Technology Stack Overview

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'primaryColor': '#e3f2fd', 'primaryTextColor': '#1565c0', 'primaryBorderColor': '#1976d2', 'lineColor': '#424242'}}}%%
graph TD
    subgraph "Technology Stack Layering"
        A["Programming Language<br/>Rust"] --> B["Agent Framework<br/>ADK (Agent Dev Kit)"]
        B --> C["AI Model<br/>OpenAI/vLLM/Ollama"]
        D["Persistence<br/>JSON File System"] --> E["Configuration Management<br/>TOML + Env"]
        F["CLI Framework<br/>clap + dialoguer"] --> G["Logging & Monitoring<br/>tracing"]
    end
    
    subgraph "External Dependencies"
        C --> H["LLM API Services"]
        D --> I["Local File System"]
        F --> J["Terminal Interaction"]
    end
    
    style A fill:#ffebee
    style B fill:#e8f5e9
    style C fill:#fff3e0
```

---

## 2. System Context

### 2.1 System Positioning and Business Value

Cowork Forge is an **AI-native software development automation tool**, positioned in the niche segment between AI-assisted programming and low-code platforms. Its core value lies in:

- **End-to-End Automation**: Covers the complete SDLC from requirements conceptualization (Idea) to project delivery (Delivery)
- **Architecture Constraint Generation**: Enforces simplicity architecture through prompt engineering to avoid "AI hallucination" causing over-design
- **Controllable Automation**: Introduces mandatory human review at key nodes such as PRD, Design, Plan to balance efficiency and quality

### 2.2 User Roles and Scenarios

```mermaid
%%{init: {'theme': 'base'}}%%
graph LR
    subgraph "User Roles"
        A[Independent Developer/Technical Entrepreneur] -->|Rapid Prototype Verification| B[System]
        C[Technical Architect] -->|Design Quality Control| B
        D[Software Development Engineer] -->|Code Generation Assistance| B
    end
    
    subgraph "Core Requirements"
        A -.->|Automated lifecycle management<br/>Support incremental modification| B
        C -.->|Simplicity-first constraints<br/>HITL collaboration review| B
        D -.->|Checkpoint recovery workflow<br/>File change tracking| B
    end
    
    style A fill:#e3f2fd
    style C fill:#fff3e0
    style D fill:#e8f5e9
```

### 2.3 External System Interactions

| External System | Interaction Method | Data Exchange | Reliability Requirements |
|----------------|-------------------|---------------|-------------------------|
| **Large Language Model Services** | HTTP API (OpenAI compatible protocol) | JSON request/response, streaming output | High (rate limit 30/min) |
| **File System** | Standard IO operations | Text/binary file read/write | High (local disk) |
| **Operating System/Shell** | Command execution (30-second timeout) | Command input/output capture | Medium (timeout protection) |
| **Terminal User Interface** | Synchronous interaction (dialoguer) | Menu selection, text input | High (blocking) |

### 2.4 System Boundary Definition

**Included in System**:
- CLI command line interface (new/resume/revert/modify/status/init)
- Multi-agent orchestration system (7-stage lifecycle management)
- Agent Actor-Critic execution engine
- Human-in-the-Loop (HITL) interaction layer
- LLM client and rate limiter
- Secure file access control module
- Session and state management storage layer
- Internal tool system (file operations, data management, validation)

**Excluded from System**:
- Actual business application runtime environment
- LLM service internal implementation details (model training/inference)
- IDE or editor plugins (only calls system default editor)
- CI/CD pipeline integration (indirectly supported via Shell commands)
- Distributed version control system server-side (Git server-side)
- Container orchestration platforms (K8s/Docker Swarm)

```mermaid
%%{init: {'theme': 'base'}}%%
graph TB
    subgraph "Cowork Forge System Boundary"
        direction TB
        A[CLI Entry] --> B[Agent Orchestration Engine]
        B --> C[Tool System]
        C --> D[File System Access Layer]
    end
    
    E[User] -->|Command Input| A
    F[LLM Service<br/>OpenAI/vLLM] <-->|API Call| B
    G[Local File System] <-->|IO Operations| D
    H[Shell Environment] <-->|Command Execution| C
    
    I[Business Application Runtime] -.->|Not Included| B
    J[IDE Plugin] -.->|Not Included| A
    
    style B fill:#e3f2fd
    style I fill:#ffebee
    style J fill:#ffebee
```

---

## 3. Container View

### 3.1 Domain Module Division

The system adopts a **Layered Domain Architecture**, implementing modularity through Rust Crate physical isolation:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'primaryColor': '#e1f5fe', 'primaryTextColor': '#01579b', 'primaryBorderColor': '#0288d1'}}}%%
graph TB
    subgraph "CLI Interaction Layer"
        CLI["cowork-cli<br/>Command Line Interface"]
    end
    
    subgraph "Core Domain Layer (cowork-core)"
        direction TB
        
        PIP["Pipeline Orchestration Domain<br/>pipeline/mod.rs"]
        AGENT["AI Agent Orchestration Domain<br/>agents/mod.rs"]
        TOOL["Tool System Domain<br/>tools/"]
        INST["Instruction Template Domain<br/>instructions/"]
        DATA["Data and Storage Management Domain<br/>data/ + storage/"]
        LLM["LLM Service Domain<br/>llm/"]
    end
    
    subgraph "Infrastructure Layer"
        FS["File System"]
        ADK["ADK Framework<br/>adk-core/agent/runner"]
        LLM_API["LLM API Services"]
    end
    
    CLI -->|Start Pipeline| PIP
    CLI -->|Load Configuration| LLM
    PIP -->|Sequential Execution| AGENT
    AGENT -->|Call| TOOL
    AGENT -->|Generate Content| LLM
    TOOL -->|Data Operations| DATA
    DATA -->|Persistence| FS
    AGENT -->|Load Prompts| INST
    LLM -->|Request| LLM_API
    
    style CLI fill:#e8eaf6
    style AGENT fill:#ffebee
    style PIP fill:#fff3e0
    style TOOL fill:#e8f5e9
```

### 3.2 Core Container Details

#### 3.2.1 CLI Interaction and Project Entry Domain (cowork-cli)

**Responsibilities**: System entry, responsible for command parsing, configuration loading, session lifecycle management
**Technical Implementation**: Parameter parsing based on `clap`, interactive menu implementation based on `dialoguer`

| Sub-component | Responsibilities | Key Interfaces |
|--------------|------------------|----------------|
| Command Processor | Parse new/resume/revert/modify/status/init commands | `execute_*_command` |
| Configuration Manager | TOML file + environment variable fallback strategy | `load_config_with_fallback` |
| Session Manager | UUID generation, state machine management (InProgress/Completed/Failed) | `generate_session_id`, `fingerprint_project_files` |

#### 3.2.2 AI Agent Orchestration Domain (cowork-core/agents)

**Architecture Pattern**: Actor-Critic Dual-Agent Collaboration
**Core Components**:

```mermaid
%%{init: {'theme': 'base'}}%%
graph LR
    subgraph "Agent Orchestration Domain"
        A[Agent Factory<br/>Agent Factory] --> B[Idea Agent]
        A --> C[PRD Loop]
        A --> D[Design Loop]
        A --> E[Plan Loop]
        A --> F[Coding Loop]
        A --> G[Check Agent]
        A --> H[Delivery Agent]
        
        I[HITL Resilient Layer<br/>ResilientAgent] -->|Wrap| C
        I -->|Wrap| D
        I -->|Wrap| E
        I -->|Wrap| F
    end
    
    subgraph "Dual-Agent Collaboration"
        C -->|Actor| C1[Generate PRD]
        C -->|Critic| C2[Verify Requirements]
        C1 <-->|Feedback Loop| C2
        
        D -->|Actor| D1[Architecture Design<br/>2-4 component constraints]
        D -->|Critic| D2[Verify Simplicity]
        D1 <-->|Feedback Loop| D2
    end
    
    style I fill:#ffebee
    style C1 fill:#e3f2fd
    style C2 fill:#ffebee
```

#### 3.2.3 Pipeline Orchestration Domain (cowork-core/pipeline)

**Core Abstraction**: `StageExecutor` implements stage isolation
**Key Capabilities**:
- **Stage Isolation**: By ignoring the escalate flag, allows LoopAgent to exit independently without affecting the overall workflow
- **Four Pipeline Builders**:
  - `create_cowork_pipeline`: Seven-stage complete workflow
  - `create_resume_pipeline`: Checkpoint resumption based on artifact detection
  - `create_partial_pipeline`: Restart from specified stage (version rollback)
  - `create_modify_pipeline`: Incremental modification dedicated pipeline

#### 3.2.4 Tool System Domain (cowork-core/tools)

**Tool Classification Architecture**:

| Tool Category | Implementation File | Core Capabilities | Security Mechanisms |
|---------------|---------------------|-------------------|---------------------|
| Data Operation Tools | `data_tools.rs` | CRUD requirements/features/tasks | Automatic ID generation (REQ-*/FEAT-*/TASK-*) |
| File Operation Tools | `file_tools.rs` | Read/Write/Execute | Path traversal protection, 30-second timeout |
| Validation Check Tools | `validation_tools.rs` | Dependency cycle detection (DFS algorithm), feature coverage analysis | Set operation verification |
| Flow Control Tools | `control_tools.rs` | Feedback recording, replanning request | Persist to feedback_history.json |
| HITL Interaction Tools | `hitl_*.rs` | Content review, file editing | System editor integration |
| Change Management Tools | `modify_tools.rs` | ChangeRequest persistence | Atomic write |

### 3.3 Storage Design

Adopts **File System Persistence** solution, implementing session isolation based on `.cowork` directory:

```
Project Root Directory/
├── .cowork/
│   ├── project_index.json          # Session index [ProjectIndex]
│   └── sessions/
│       └── {uuid}/
│           ├── metadata.json       # Session metadata [SessionRecord]
│           ├── input.json          # Session input
│           ├── state.json          # Execution state
│           ├── feedback_history.json # Feedback history
│           └── artifacts/
│               ├── idea.md
│               ├── prd.md
│               ├── design.md
│               └── plan.json
└── src/                            # Generated project code
```

**Data Model Core Entities**:
- `ProjectIndex`: Project-level session index, supporting multi-session management
- `SessionRecord`: Session state machine (InProgress → Completed/Failed)
- `Requirements`/`Feature`: Requirement management, supporting priority and status tracking
- `DesignSpec`: Design specification, mandatory component count constraint (2-4)
- `ImplementationPlan`/`Task`: Implementation plan, task dependency graph
- `ChangeRequest`: Change management, supporting incremental modification

### 3.4 Cross-Domain Communication Mechanisms

```mermaid
%%{init: {'theme': 'base'}}%%
graph LR
    subgraph "Dependency Relationships"
        CLI["CLI Domain"] -->|Start| PIP["Pipeline Domain"]
        CLI -->|Configuration| LLM["LLM Domain"]
        CLI -->|Data Dependency| DATA["Data Domain"]
        
        PIP -->|Execute| AGENT["Agent Domain"]
        PIP -->|State Management| DATA
        
        AGENT -->|Tool Dependency| TOOL["Tool Domain"]
        AGENT -->|Instruction Dependency| INST["Instruction Domain"]
        AGENT -->|Generate| LLM
        
        TOOL -->|Data Operations| DATA
    end
    
    subgraph "Communication Mechanisms"
        A["Type 1: Service Call<br/>Arc<dyn Agent>"] 
        B["Type 2: Configuration Injection<br/>LlmConfig"] 
        C["Type 3: Data Dependency<br/>Storage + Models"]
        D["Type 4: Event Stream<br/>EventStream"]
    end
    
    PIP -.->|Type 1| A
    LLM -.->|Type 2| B
    TOOL -.->|Type 3| C
    AGENT -.->|Type 4| D
```

---

## 4. Component View

### 4.1 Core Functional Components

#### 4.1.1 StageExecutor (Stage Executor)

**Responsibilities**: Sequentially execute multi-stage agents, isolating stage states
**Key Design**:

```rust
pub struct StageExecutor {
    name: String,
    stages: Vec<(String, Arc<dyn Agent>)>, // Stage name and agent mapping
}

impl Agent for StageExecutor {
    async fn run(&self, ctx: Arc<dyn InvocationContext>) -> AdkResult<EventStream> {
        for (stage_name, agent) in &self.stages {
            // Key: Ignore escalate flag to prevent LoopAgent's ExitLoopTool from affecting overall workflow
            let event_stream = agent.run(ctx.clone()).await?;
            // Stream processing logic...
        }
    }
}
```

**Architecture Value**: Implements an execution model of "iteration within stages, sequential execution between stages", supporting the Actor-Critic feedback loop mechanism.

#### 4.1.2 ResilientAgent (Resilient Agent Wrapper)

**Three-Level Fault Tolerance Mechanism**:

```mermaid
%%{init: {'theme': 'base'}}%%
flowchart TD
    A[ResilientAgent] --> B{Execute}
    B -->|Success| C[Continue]
    B -->|Failure| D[Error Interception]
    
    D --> E{Error Type}
    E -->|Recoverable| F[Retry Counter<br/>max=3]
    F -->|Not Exceeded| G[Automatic Retry]
    F -->|Exceeded| H[Human Decision Interaction]
    
    H --> I[Option 1: Retry]
    H --> J[Option 2: Provide Guidance]
    H --> K[Option 3: Abort]
    
    J --> L[ProvideFeedbackTool<br/>Record feedback history] --> G
    
    E -->|Unrecoverable| M[Direct Throw]
    
    style A fill:#ffebee
    style H fill:#fff3e0
```

#### 4.1.3 LoopAgent (Actor-Critic Loop)

**Collaboration Mode Implementation**:

| Stage | Actor Responsibilities | Critic Responsibilities | Exit Condition |
|-------|----------------------|------------------------|----------------|
| PRD | Analyze requirements and generate documents | Verify completeness, simplicity | ExitLoopTool or max=3 |
| Design | Create architecture design | Verify component count (2-4), simplicity | ExitLoopTool or max=3 |
| Plan | Create task list | Verify no dependency loops, coverage | ExitLoopTool or max=3 |
| Coding | Implement code | Verify task completion, code quality | ExitLoopTool or max=5 |

### 4.2 Technical Support Components

#### 4.2.1 File Secure Access Control (FileTools)

**Security Mechanisms**:
- **Path Traversal Protection**: `validate_path_security` check, prohibiting access to sensitive paths outside `.cowork`
- **Command Execution Sandbox**: `RunCommandTool` 30-second timeout, blocking background service commands (`nohup`, `&`, `systemctl`)
- **Concurrency Safety**: Based on Rust ownership system, file operation atomicity guarantee

#### 4.2.2 Rate Limiter (RateLimiter)

**Simplified Token Bucket Algorithm Implementation**:
- Default delay: 2 seconds/request (satisfying 30 times/minute limit)
- Decorator pattern: Wraps `Llm` trait, transparently intercepts requests
- Configurable: Dynamically adjust through `RateLimitedLlm::with_delay`

```rust
pub struct RateLimitedLlm {
    inner: Arc<dyn Llm>,
    delay: Duration, // Default 2s
}
```

#### 4.2.3 Data Validation Engine (ValidationTools)

**Algorithm Implementation**:
- **Dependency Cycle Detection**: DFS algorithm to detect task dependency graph
- **Feature Coverage Analysis**: Set operation to verify code implementation covers requirements
- **Data Format Validation**: Based on serde deserialization verification

### 4.3 Component Interaction Relationships

```mermaid
%%{init: {'theme': 'base'}}%%
graph TB
    subgraph "Component Interaction Sequence"
        direction TB
        
        CLI[CLI] -->|1. create_pipeline| PIP[StageExecutor]
        PIP -->|2. execute| RA[ResilientAgent]
        RA -->|3. run| LA[LoopAgent]
        
        subgraph "Actor-Critic Loop"
            LA -->|4a. actor_run| ACTOR[Actor Agent]
            ACTOR -->|5a. use| TOOLS[Tools]
            TOOLS -->|6a. operate| DATA[Data Models]
            ACTOR -->|7a. feedback| CRITIC[Critic Agent]
            CRITIC -->|8a. validate| LA
        end
        
        LA -->|9. exit_loop| RA
        RA -->|10. handle_error/interact| USER[User]
        USER -->|11. feedback| RA
        RA -->|12. complete| PIP
        PIP -->|13. next_stage| PIP
    end
    
    style PIP fill:#fff3e0
    style RA fill:#ffebee
    style ACTOR fill:#e3f2fd
    style CRITIC fill:#f3e5f5
```

---

## 5. Key Processes

### 5.1 Complete Project Creation Workflow

```mermaid
%%{init: {'theme': 'base'}}%%
flowchart TD
    Start([User input: cowork new<br/>--project <name>]) --> Init[CLI initializes session<br/>Generate SessionID<br/>Status: InProgress]
    
    Init --> Idea[Idea Agent<br/>Organize idea → idea.md<br/>6-step workflow]
    
    Idea --> PRD_A[PRD Actor<br/>Analyze requirements generate PRD draft<br/>Constraints: 3-6 core requirements<br/>2-4 core features]
    PRD_A --> HITL1{Human Review<br/>review_with_feedback_content}
    HITL1 -->|edit| EDIT1[User edit<br/>Resave] --> PRD_A
    HITL1 -->|feedback| PRD_F[Record feedback<br/>provide_feedback] --> PRD_A
    HITL1 -->|pass| PRD_C[PRD Critic<br/>Verify simplicity<br/>Anti-loop protection]
    PRD_C -->|fail| PRD_F
    PRD_C -->|pass| DES_A[Design Actor<br/>2-4 component constraints]
    
    DES_A --> HITL2{Human Review}
    HITL2 -->|edit| EDIT2[Edit design] --> DES_A
    HITL2 -->|feedback| DES_F[Record feedback] --> DES_A
    HITL2 -->|pass| DES_C[Design Critic<br/>Verify component count<br/>Simplicity check]
    DES_C -->|fail| DES_F
    DES_C -->|pass| PLN_A[Plan Actor<br/>5-12 core tasks constraints]
    
    PLN_A --> HITL3{Human Review}
    HITL3 -->|edit| EDIT3[Edit tasks] --> PLN_A
    HITL3 -->|feedback| PLN_F[Record feedback] --> PLN_A
    HITL3 -->|pass| PLN_C[Plan Critic<br/>Verify dependencies<br/>Feature coverage check]
    PLN_C -->|fail| PLN_F
    PLN_C -->|pass| COD_A[Coding Actor<br/>Full code generation<br/>Complete all tasks in single iteration]
    
    COD_A --> COD_C[Coding Critic<br/>Verify task completion<br/>Code simplicity check]
    COD_C -->|fix| COD_F[Provide feedback] --> COD_A
    COD_C -->|pass| CHK[Check Agent<br/>Permissive quality check<br/>File existence verification]
    
    CHK -->|fail| DEC{Replan?}
    DEC -->|yes| REPLAN[GoToStageTool<br/>Rollback to specified stage]
    DEC -->|no| REPAIR[Fix] --> CHK
    
    CHK -->|pass| DLV[Delivery Agent<br/>Pre-check: list_files<br/>Generate delivery report]
    
    DLV --> Save[Update session status<br/>Completed<br/>Save all artifacts]
    Save --> End([Project delivery complete])
    
    style HITL1 fill:#fff3e0
    style HITL2 fill:#fff3e0
    style HITL3 fill:#fff3e0
    style Start fill:#e8f5e9
    style End fill:#e8f5e9
```

**Key Execution Details**:

1. **PRD Stage Constraints**:
   - Actor must generate PRD draft containing 3-6 core requirements and 2-4 core features
   - Mandatory rejection of non-core requirements (performance optimization, test infrastructure, CI/CD, monitoring)
   - Human review implemented through `review_with_feedback_content` tool, supporting edit/pass/feedback three operations

2. **Design Stage Constraints**:
   - Component count strictly limited to 2-4
   - Mandatory use of simplest technology stack (recommend SQLite/JSON rather than complex databases)
   - Recommend monolithic architecture, prohibit microservices, caching layers, message queues (unless absolutely necessary)

3. **Coding Stage Characteristics**:
   - Unlike the multi-iteration of the first three stages, Coding Actor completes all pending tasks in a single round
   - Code generation follows existing project patterns, context-aware through `list_files` and `read_file` tools
   - Prohibit automatic generation of test code (unless explicitly required by PRD)

### 5.2 Incremental Modification Workflow

**Business Value**: Supports safe changes to delivered projects, achieving requirement evolution through change impact analysis and incremental patches while maintaining change traceability.

**Execution Flow**:

```mermaid
flowchart TD
    Start([cowork modify<br/>--project <name>]) --> Fingerprint[Calculate project file fingerprints<br/>Change tracking foundation]
    
    Fingerprint --> Init[Initialize based on original session<br/>init_session_from_base<br/>Create change session ID]
    
    Init --> Triage[Change Triage Agent<br/>CHANGE_TRIAGE_INSTRUCTION]
    subgraph TriageProcess["Change Analysis Stage"]
        T1[Load original requirements/design/plan<br/>get_requirements<br/>get_design<br/>get_plan]
        T2[Analyze change impact scope<br/>PRD/Design/Plan/Code]
        T3[Risk assessment<br/>Low/Medium/High<br/>Identify affected components]
        T4[Create ChangeRequest<br/>Include constraints and acceptance criteria]
        
        T1 --> T2 --> T3 --> T4
    end
    
    Triage --> SaveReq[SaveChangeRequestTool<br/>Persist change analysis results]
    
    SaveReq --> Patch[Code Patch Agent<br/>CODE_PATCH_INSTRUCTION]
    subgraph PatchProcess["Code Patch Stage"]
        P1[Load ChangeRequest<br/>Understand change scope]
        P2[Read affected files<br/>read_file<br/>Follow existing code patterns]
        P3[Execute incremental modification<br/>write_file<br/>Update task status]
        P4[Run command verification<br/>run_command]
        
        P1 --> P2 --> P3 --> P4
    end
    
    Patch --> Check[Check Agent<br/>Verify modified quality]
    
    Check -->|fail| Fix[Fix issues] --> Check
    
    Check --> Report[Modify Delivery Agent<br/>MODIFY_DELIVERY_INSTRUCTION]
    subgraph ReportGen["Change Report Generation"]
        R1[Load before/after state comparison]
        R2[Generate PR-like format report<br/>Summary/Change Details/<br/>Scope Analysis/Testing]
        R3[save_delivery_report]
    end
    
    Report --> SaveMeta[Save Patch metadata<br/>Record change history]
    SaveMeta --> End([Change complete])
```

**Key Technical Mechanisms**:

1. **File Fingerprint Calculation**: Calculate project file hashes before modification, used for subsequent change tracking and conflict detection
2. **State Inheritance**: Implement session state inheritance through `init_session_from_base`, copying historical artifacts as context
3. **Change Scope Identification**: Triage Agent analyzes whether changes affect requirements layer (PRD), architecture layer (Design), planning layer (Plan), or only code layer (Code), guiding subsequent execution paths

### 5.3 Checkpoint Recovery Workflow

**Business Value**: Ensures long-running AI development processes have fault tolerance, supporting resumption from any interruption point, avoiding repeated computation.

**Execution Flow**:

```mermaid
flowchart TD
    Start([cowork resume<br/>--session <id>]) --> Parse[Parse resume command<br/>Support specifying ID or auto-select latest]
    
    Parse --> Load[Load target session state<br/>Read project index]
    
    Load --> Detect["Detect existing artifact status<br/>Artifact existence matrix analysis"]
    subgraph Detection["Artifact Detection Logic"]
        D1[Check idea.md<br/>→ Idea stage complete]
        D2[Check prd.md + requirements.json<br/>→ PRD stage complete]
        D3[Check design.md + design_spec.json<br/>→ Design stage complete]
        D4[Check plan.json<br/>→ Plan stage complete]
        D5[Check code file existence<br/>→ Coding stage complete]
        D6[Check delivery_report.md<br/>→ Delivery stage complete]
    end
    
    Detect --> Identify[Identify last completed stage<br/>Determine resume start point]
    
    Identify --> Pipeline["create_resume_pipeline<br/>Intelligent pipeline construction"]
    subgraph PipelineBuild["Resume Pipeline Construction Strategy"]
        PB1[If Idea missing: Start from Idea<br/>Include all 7 stages]
        PB2[If PRD missing: Start from PRD Loop<br/>Skip Idea]
        PB3[If Design complete but Plan missing: Start from Plan Loop]
        PB4[If Coding in progress: Restart Coding Loop]
    end
    
    Pipeline --> Execute[Execute sequentially from breakpoint<br/>Subsequent agent processes]
    
    Execute --> Save[Update session state and artifacts]
    Save --> End([Resume complete])
```

**State Recovery Strategy**:

The system constructs a **stage completion matrix** by detecting file existence in the `.cowork/sessions/<id>/artifacts/` and `.cowork/sessions/<id>/state/` directories. The `create_resume_pipeline` function dynamically builds the execution pipeline based on this matrix, ensuring:

- Completed stages (artifacts exist and are valid) are skipped
- Incomplete or partially completed stages are re-executed
- Dependency relationships remain coherent (e.g., when restarting Coding stage, ensure Plan data is loaded)

### 5.4 Version Rollback Workflow

**Business Value**: When fundamental flaws appear in architecture design or directional requirement changes occur, allows restarting from a specific stage, preserving valuable early analysis results.

**Execution Flow**:

```mermaid
flowchart TD
    Start([cowork revert<br/>--session <id><br/>--to <stage>]) --> Parse[Parse rollback command<br/>Verify target stage parameter<br/>PRD/Design/Plan/Coding]
    
    Parse --> CreateNew[Create new session based on original session<br/>Inherit state before target stage]
    
    CreateNew --> Cleanup[Clear target stage and subsequent artifacts]
    subgraph CleanupDetail["Selective Clear Strategy"]
        C1[Preserve: idea.md, early state/*.json]
        C2[Clear: target stage artifacts/ and state/]
        C3[Clear: all subsequent stage data]
    end
    
    Cleanup --> Pipeline[create_partial_pipeline<br/>Build partial pipeline starting from target stage]
    
    Pipeline --> Execute[Re-execute from target stage<br/>All subsequent stage agents]
    
    Execute --> Save[Save new session state<br/>Preserve original session history]
    Save --> End([Rollback complete<br/>New session continues execution])
```

---

## 6. Process Coordination and Control

### 6.1 Pipeline Orchestration Mechanism

The system adopts **StageExecutor** to achieve precise workflow control, ensuring consistency and reliability of multi-stage sequential execution:

```mermaid
sequenceDiagram
    participant CLI as CLI Entry
    participant PF as Pipeline Factory
    participant SE as StageExecutor
    participant Agent as AI Agent
    participant Store as Storage Layer

    CLI->>PF: Create workflow pipeline<br/>create_cowork_pipeline()
    PF->>PF: Instantiate stage agents<br/>Idea→PRD→Design→Plan→Coding→Check→Delivery
    
    PF->>SE: Register sub-agent list
    loop Stage Execution
        SE->>SE: Isolate escalate flag<br/>Prevent loop exit signal propagation
        SE->>Agent: Execute current stage<br/>execute()
        Agent->>Store: Read/write session state
        Agent-->>SE: Return execution result<br/>Content/FunctionCall
        SE->>SE: Check exit signal<br/>ExitLoopTool handling
    end
    
    SE-->>CLI: Report execution completion status
```

**StageExecutor Core Responsibilities**:

1. **Stage Isolation**: Each stage has independent escalate flag context, ensuring LoopAgent's internal loop exit won't terminate the entire workflow
2. **Streaming Output**: Implements `Agent` trait, supports asynchronous streaming of LLM-generated content, real-time feedback to user
3. **State Verification**: At stage transitions, verifies existence of prerequisite stage artifacts, ensuring data continuity

### 6.2 State Management and Data Flow

The system adopts **session isolation + file system persistence** architecture to manage state:

```mermaid
flowchart TB
    subgraph Session["Session State Model"]
        SID[Session ID<br/>UUID unique identifier]
        ST[Status<br/>InProgress/Completed/Failed]
        META[SessionMeta<br/>Timestamp/parent session/change type]
        BASE[Base Session<br/>Inheritance relationship]
    end

    subgraph Storage["Storage Structure (.cowork/)"]
        INDEX[index.json<br/>Project-level session registry]
        
        subgraph SDir["sessions/{id}/"]
            INPUT[input.json<br/>Session initialization parameters]
            CREQ[change_request.json<br/>Modification request]
            
            subgraph Artifacts["artifacts/"]
                IDEA[idea.md]
                PRD[prd.md]
                DESIGN[design.md]
                DELIVERY[delivery_report.md]
            end
            
            subgraph State["state/"]
                REQ[requirements.json]
                FEAT[features.json]
                PLAN[plan.json]
                CODE[code_metadata.json]
                META_FILE[session_meta.json]
                FEED[feedback_history.json]
            end
            
            subgraph Patch["patch/"]
                PMETA[patch_metadata.json]
            end
        end
    end

    Session --> Storage
```

**Data Flow Control Principles**:

1. **Forward Dependency**: Each stage can only read data from current and previous stages, prohibiting reverse dependency
2. **Immutable Artifacts**: Once a stage is marked as complete, its generated markdown artifacts (idea.md, prd.md, etc.) are read-only in principle, modifications require incremental modification workflow
3. **Automatic State Update**: `auto_update_feature_status` mechanism automatically cascades feature status updates when task status changes

### 6.3 Human-in-the-Loop (HITL) Coordination Mechanism

The system forces human review at key quality nodes, implementing seamless collaboration through tool chains:

```mermaid
flowchart TD
    A[Actor Agent generates content draft] --> B{HITL Tool Chain<br/>Request user review}
    B --> C[User selects: Direct edit]
    C --> E[User directly modifies content]
    B --> F[User selects: Provide feedback]
    F --> G[Actor modifies based on text feedback]
    B --> H[User selects: Pass]
    H --> I[Critic Agent intervenes for verification]
    I --> J{Verification passed?}
    J -- Passed --> K[Exit loop, enter next stage]
    J -- Not passed --> L[Critic provides modification suggestions]
    L --> G
    E & G --> A
```

**HITL Decision Matrix**:

| Stage | Review Tool | Edit Capability | Feedback Handling | Loop Limit |
|-------|-------------|-----------------|-------------------|------------|
| Idea | ReviewAndEditContentTool | Yes | Resave | No limit |
| PRD | ReviewWithFeedbackContentTool | Yes | Record to feedback_history | 3-5 times |
| Design | ReviewWithFeedbackContentTool | Yes | Record to feedback_history | 3-5 times |
| Plan | ReviewWithFeedbackContentTool | Yes | Record to feedback_history | 3-5 times |
| Coding | None (automatic flow) | No | Critic direct feedback | 3 times |

---

## 7. Exception Handling and Recovery

### 7.1 Resilient Error Recovery Architecture

The system achieves robust error handling through the **ResilientAgent** wrapper, ensuring when AI execution exceptions occur, it doesn't fail immediately but provides graded recovery strategies:

```mermaid
flowchart TD
    Start[Agent execution] --> Error{Error occurred?}
    
    Error -->|No| Success[Execution successful]
    Error -->|Yes| Intercept[ResilientAgent intercepts]
    
    Intercept --> Retry{Retry count < 3?}
    Retry -->|Yes| RetryExec[Retry execution<br/>Increment retry counter]
    RetryExec --> Error
    
    Retry -->|No| StreamCheck{Streaming output stage?}
    StreamCheck -->|Yes| ResilientStream[ResilientStream wrapper<br/>Delayed error handling]
    StreamCheck -->|No| DirectHandle[Direct error handling]
    
    ResilientStream --> UserInteract[Start human decision interaction]
    DirectHandle --> UserInteract
    
    UserInteract --> Options[Present options:<br/>1. Retry<br/>2. Provide Guidance<br/>3. Abort]
    
    Options -->|Retry| RetryExec
    Options -->|Abort| AbortProcess[Graceful abort<br/>Update status to Failed<br/>Save error context]
    
    Options -->|Guidance| Input[User inputs guidance suggestions]
    Input --> Feedback[ProvideFeedbackTool<br/>Record to history<br/>Severity: Critical]
    Feedback --> RetryExec
    
    AbortProcess --> End([Process end])
```

**Exception Classification and Handling Strategies**:

| Exception Type | Detection Method | Automatic Retry | Human Intervention | Recovery Mechanism |
|---------------|-----------------|----------------|-------------------|-------------------|
| **Max Iterations** | ADK framework throws | 3 times | Mandatory | User guidance then retry |
| **LLM API Error** | HTTP status code | 3 times | Optional | Delayed retry |
| **Tool Execution Failure** | ToolResult error code | 0 times | Mandatory | Fix then retry |
| **Loop Detection** | Critic feedback history analysis | N/A | Mandatory | RequestHumanReviewTool |

### 7.2 Anti-Loop Protection Mechanism

To prevent Actor-Critic loops from falling into infinite iterations, the system implements protection at multiple levels:

```mermaid
flowchart LR
    subgraph LoopProtection["Anti-Loop Protection Strategies"]
        IP1[Hard iteration limit<br/>PRD/Design/Plan: 3-5 times<br/>Coding: 3 times]
        IP2[Feedback history analysis<br/>Critic detects repeated issues]
        IP3[Manual escalation mechanism<br/>request_human_review]
    end
    
    subgraph DetectionLogic["Detection Logic"]
        D1[Check last 3 feedback entries<br/>Similarity > 80%]
        D2[Check same issue recurring]
        D3[Check Actor non-responsive or deviating]
    end
    
    IP2 --> DetectionLogic
    DetectionLogic -->|Trigger| IP3
    IP3 -->|escalate| HITL[Manual takeover<br/>Boxed warning output]
```

**Critic Decision Rules**:

1. **First Iteration**: Actor generates draft → Critic comprehensive check → Provides specific feedback
2. **Second Iteration**: Actor modifies → Critic compares with feedback history → If unresolved issue found, elevates warning level
3. **Third Iteration**: If issue persists, Critic calls `request_human_review` tool, outputs boxed warning (⚠️ 🚨) on console, and returns Agent Error to suspend execution

### 7.3 Failover and Degradation Strategies

Degradation behavior when key services are unavailable:

| Failure Scenario | Impact Scope | Degradation Strategy | User Experience |
|------------------|--------------|---------------------|-----------------|
| LLM API Timeout | Current stage | Exponential backoff retry (1s, 2s, 4s) | Display waiting prompt |
| LLM API Completely Unavailable | Entire workflow | Save current state, prompt user to check configuration | Graceful exit, state retained |
| File System Permission Error | Specific tool call | Skip operation, record warning | Continue execution, remind afterwards |
| User Interruption (Ctrl+C) | Current execution | Save session status to InProgress | Resume possible |
| Disk Space Insufficient | Persistence operation | Pause new file writes, clean logs | Prompt to clear space |

---

## 8. Technical Implementation Details

### 8.1 Core Module Implementation

#### 8.1.1 Agent Factory Implementation Pattern

Adopts **Builder Pattern** combined with **Dependency Injection**:

```rust
pub fn create_prd_loop(
    llm: Arc<dyn Llm>,
    tools: Vec<Arc<dyn Tool>>,
    storage: Arc<dyn Storage>,
) -> Arc<dyn Agent> {
    // Actor: Generate requirements
    let actor = LlmAgentBuilder::new()
        .with_model(llm.clone())
        .with_instruction(PRD_ACTOR_INSTRUCTION)
        .with_tools(tools.clone())
        .build();
    
    // Critic: Verify requirements
    let critic = LlmAgentBuilder::new()
        .with_model(llm)
        .with_instruction(PRD_CRITIC_INSTRUCTION)
        .with_tools(tools)
        .build();
    
    // LoopAgent: Orchestrate Actor-Critic loop
    let loop_agent = LoopAgentBuilder::new()
        .with_main_agent(actor)
        .with_critic_agent(critic)
        .with_max_iterations(3)
        .build();
    
    // ResilientAgent: Wrap with fault tolerance mechanism
    Arc::new(ResilientAgent::new(
        loop_agent,
        storage,
        "prd_loop".to_string()
    ))
}
```

#### 8.1.2 Stage Isolation Mechanism

**Key Code** (`StageExecutor::run`):

```rust
// Ignore escalate event to prevent LoopAgent's ExitLoopTool from affecting overall workflow
while let Some(event) = stream.next().await {
    match event {
        Event::Action { action, .. } => {
            // Key: Don't propagate escalate flag to outer layer
            if action.name() != "ExitLoopTool" {
                yield event;
            }
        }
        _ => yield event,
    }
}
```

**Architecture Significance**: Implements "structured concurrency" where child task failures don't affect parent tasks, supporting intra-stage retries.

### 8.2 Key Algorithm Design

#### 8.2.1 Checkpoint Recovery Algorithm

Based on **artifact existence detection** state inference:

```rust
fn detect_resume_stage(session_id: &str) -> ResumeStage {
    if has_code_files(session_id) {
        ResumeStage::Check  // Code generated, start from check
    } else if has_implementation_plan(session_id) 
           && has_design_spec(session_id) {
        ResumeStage::Coding // Design complete, start from coding
    } else if has_design_spec(session_id) {
        ResumeStage::Plan   // Requirements complete, start from planning
    } else if has_requirements(session_id) {
        ResumeStage::Design // Requirements complete, start from design
    } else {
        ResumeStage::Prd    // Start from beginning
    }
}
```

#### 8.2.2 Task Dependency Cycle Detection

**DFS Algorithm Implementation** (`CheckTaskDependenciesTool`):

```rust
fn detect_cycles(tasks: &[Task]) -> Option<Vec<String>> {
    let graph = build_dependency_graph(tasks);
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    
    for task in tasks {
        if !visited.contains(&task.id) {
            if let Some(cycle) = dfs_detect(&graph, task.id, &mut visited, &mut recursion_stack) {
                return Some(cycle);
            }
        }
    }
    None
}
```

### 8.3 Data Structure Design

#### 8.3.1 Session State Machine

```mermaid
%%{init: {'theme': 'base'}}%%
stateDiagram-v2
    [*] --> InProgress: cowork new/resume
    
    InProgress --> InProgress: Stage in execution
    InProgress --> Completed: Delivery successful
    InProgress --> Failed: Error/abort
    
    Completed --> InProgress: cowork modify<br/>Create change session
    Failed --> InProgress: cowork resume<br/>Checkpoint recovery
    
    Completed --> [*]: Project delivered
    Failed --> [*]: User abandoned
```

#### 8.3.2 Feature Lifecycle Model

```rust
enum FeatureStatus {
    Planned,      // Planned
    InProgress,   // In progress
    Implemented,  // Implemented (code exists)
    Verified,     // Verified (tests passed)
    Delivered,    // Delivered
}

struct Feature {
    id: String,           // FEAT-001
    name: String,
    status: FeatureStatus,
    requirements: Vec<String>, // Associated requirement IDs
    tasks: Vec<String>,        // Associated task IDs
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### 8.4 Performance Optimization Strategies

#### 8.4.1 LLM Call Optimization

- **Rate Limiting**: Default 2-second delay, preventing API rate limiting
- **Streaming Output**: Supports `--stream` flag, real-time display of generation process
- **Context Compression**: Critic stage only passes key differences, reducing token consumption

#### 8.4.2 Storage Optimization

- **Lazy Loading**: `Storage` implements on-demand JSON file loading
- **Incremental Persistence**: Immediately save state after tool execution, avoiding batch write blocking
- **File Fingerprint Caching**: Cache file hashes during incremental modification, avoiding repeated computation

#### 8.4.3 Concurrency Design

- **Intra-Stage Sequential**: Guarantees causality of Actor-Critic alternating execution
- **Inter-Stage Isolation**: StageExecutor sequential execution, avoiding concurrent conflicts
- **Asynchronous IO**: File operations and LLM calls are both asynchronous, supporting concurrent session management

---

## 9. Deployment Architecture

### 9.1 Runtime Environment Requirements

| Component | Minimum Requirements | Recommended Configuration |
|-----------|---------------------|---------------------------|
| **Operating System** | Linux/macOS/Windows | Linux (Ubuntu 22.04+) |
| **Rust** | 1.70+ | 1.75+ |
| **Memory** | 2GB | 4GB+ (LLM context cache) |
| **Disk** | 1GB | 10GB+ (Project artifact storage) |
| **Network** | Accessible LLM API | Local vLLM/Ollama deployment |

### 9.2 Deployment Topology

```mermaid
%%{init: {'theme': 'base'}}%%
graph TB
    subgraph "Development Workstation"
        CLI[Cowork Forge CLI]
        Config[Configuration file<br/>~/.config/cowork/config.toml]
        Workspace[Working directory<br/>~/projects/]
    end
    
    subgraph "Local Infrastructure"
        LocalLLM[vLLM/Ollama<br/>Local model service]
    end
    
    subgraph "Cloud Services"
        CloudLLM[OpenAI API<br/>Claude API]
    end
    
    subgraph "Storage"
        FS[Local File System]
        Git[Git Repository]
    end
    
    CLI -->|Read| Config
    CLI -->|Operate| Workspace
    CLI -->|API Call| LocalLLM
    CLI -->|API Call| CloudLLM
    CLI -->|Persistence| FS
    Workspace -->|Version Control| Git
    
    style CLI fill:#e3f2fd
    style LocalLLM fill:#e8f5e9
    style CloudLLM fill:#fff3e0
```

### 9.3 Configuration Management Strategy

**Layered Configuration System**:

1. **System-level Configuration** (`/etc/cowork/config.toml`): Global defaults
2. **User-level Configuration** (`~/.config/cowork/config.toml`): Personal preferences
3. **Project-level Configuration** (`.cowork/config.toml`): Project-specific
4. **Environment Variables** (`COWORK_LLM_API_KEY`): Sensitive information
5. **Command Line Arguments** (`--verbose`, --model`): Temporary override

**Configuration Example**:
```toml
[llm]
provider = "openai"  # Or "vllm", "ollama"
api_key = "${OPENAI_API_KEY}"  # Supports env variable interpolation
model = "gpt-4"
rate_limit = "30/m"

[project]
template = "rust"  # Default project template
editor = "code"    # System editor command
```

### 9.4 Scalability Design

#### 9.4.1 Horizontal Scalability

Although Cowork Forge is primarily designed for single-machine CLI, its architecture supports the following extensions:

- **Multi-Session Parallel**: Session ID isolation supports simultaneous management of multiple projects
- **Multi-Model Switching**: Define multiple LLM configurations in config, switch at runtime
- **Custom Tools**: Implement `Tool` trait to extend agent capabilities

#### 9.4.2 Vertical Scalability Path

| Extension Point | Implementation Method | Scenarios |
|----------------|----------------------|-----------|
| **New Development Stage** | Implement `Agent` trait, register in `StageExecutor` | Add security audit stage |
| **Custom Validation Rules** | Add new `ValidationTool`, reference in Critic instruction | Compliance checking |
| **New Language Support** | Extend `coding` stage instruction templates, add file extension mapping | Python/Go project generation |
| **CI/CD Integration** | Use `--no-interactive` mode, coordinate with Shell scripts | Automated pipelines |

### 9.5 Monitoring and Operations

#### 9.5.1 Observability

**Structured Logging** (tracing):
- **INFO**: Stage start/complete, human-computer interaction points
- **DEBUG**: Tool call details, LLM request parameters
- **ERROR**: Execution exceptions, retry counter overflow

**Session Audit Log**:
```json
// .cowork/sessions/{id}/audit.log
{
  "timestamp": "2025-01-20T10:00:00Z",
  "event": "stage_transition",
  "from": "design",
  "to": "plan",
  "actor": "DesignCritic",
  "decision": "approve"
}
```

#### 9.5.2 Troubleshooting Guide

| Failure Phenomenon | Possible Cause | Solution |
|-------------------|----------------|----------|
| Max iterations error | Task too complex or description unclear | Use `cowork modify` to refine requirements, or manually provide guidance |
| API rate limiting | LLM calls too frequent | Check `rate_limit` configuration, or switch to local vLLM |
| File permission error | Security path verification failed | Ensure operation is within `project_root`, avoid system directories |
| Session state inconsistency | Forced exit causing metadata corruption | Use `cowork revert` to rollback to stable stage |

#### 9.5.3 Backup and Recovery

**Automatic Backup Mechanism**:
- Automatically create `.cowork/sessions/{id}/backups/` after each stage completion
- Keep state snapshots of the last 5 stages
- Support quick rollback via `cowork revert --to <stage>`

**Disaster Recovery**:
```bash
# Manual session backup
cp -r .cowork/sessions/{id} backup/

# Restore session
cp -r backup/{id} .cowork/sessions/
cowork resume --session {id}
```

---

## 10. Architecture Decision Records (ADR)

### ADR-001: File System Persistence vs Database Storage

**Decision**: Use JSON file system instead of SQLite/PostgreSQL
**Rationale**:
- Native integration with code repositories for version control
- Zero-configuration deployment, no database service required
- Human-readable artifacts (Markdown/JSON) facilitate auditing
**Trade-offs**: Sacrifice complex query performance for simplicity and portability

### ADR-002: Actor-Critic Pattern vs Single Agent

**Decision**: Adopt dual-agent (Actor-Critic) collaboration for key stages
**Rationale**:
- Separate generation and verification responsibilities, reduce single prompt complexity
- Achieve self-correction through loop mechanism, reduce manual intervention frequency
**Trade-offs**: Increase token consumption and latency, exchange for higher quality output

### ADR-003: Rust Language Selection

**Decision**: Use Rust instead of Python/TypeScript
**Rationale**:
- Type safety guarantees runtime stability (critical for long-process automation)
- Performance advantages: fast file operations, low memory footprint
- Native support from ADK framework
**Trade-offs**: Slower development iteration, steeper learning curve

---

## 11. Summary

Cowork Forge's architecture design embodies the philosophy of **"Controlled Automation"**: ensuring AI output quality through Actor-Critic collaboration mechanism, ensuring human control at key moments through HITL resilient layer, and achieving reliable long-process execution through stage isolation and session management.

This architecture applies to software development scenarios requiring **structured AI assistance**, particularly:
- Startup product rapid prototype verification (MVP development)
- Batch generation of standardized microservices
- Incremental refactoring of legacy systems

**Future Evolution Directions**:
- Support distributed multi-agents (multi-model debate mechanism)
- Integrate IDE plugins for real-time code synchronization
- Introduce vectorized storage for project knowledge base retrieval