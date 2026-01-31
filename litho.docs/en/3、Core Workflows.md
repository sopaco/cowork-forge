# Core Workflows

## 1. Workflow Overview

### 1.1 System Positioning and Core Value

Cowork Forge is an AI-driven software development automation platform built on the **ADK (Agent Development Kit)** framework. The system employs an **Actor-Critic Multi-Agent Collaboration Architecture**, covering the complete software development lifecycle (SDLC) from project idea conceptualization to final delivery. Through human-in-the-loop (HITL) mode, it introduces manual review at key quality nodes, ensuring AI-generated content complies with the **Simplicity-First** principle and avoids over-engineering.

### 1.2 Core Execution Paths

The system defines four core execution paths to meet different software project development scenarios:

| Workflow Type | Trigger Command | Core Features | Applicable Scenarios |
|---------------|----------------|---------------|---------------------|
| **Complete New Workflow** | `cowork new` | Seven-stage sequential execution, mandatory manual review nodes | New project development from scratch |
| **Incremental Modify Workflow** | `cowork modify` | Change impact analysis, incremental code patches, PR-like reports | Requirement changes and feature enhancements |
| **Checkpoint Recovery Workflow** | `cowork resume` | Artifact status detection, intelligent resumption, checkpoint recovery | Continuing after interruption |
| **Version Rollback Workflow** | `cowork revert` | State snapshot rollback, stage-level re-execution | Re-planning after design flaws or requirement changes |

### 1.3 Key Process Node Architecture

All workflows share a unified layered execution architecture:

```mermaid
flowchart TB
    subgraph CLI["CLI Interaction Layer"]
        CMD[Command Parsing and Parameter Validation]
        CFG[Configuration Loading and Initialization]
        SES[Session Lifecycle Management]
    end

    subgraph Pipeline["Pipeline Orchestration Layer"]
        SE[StageExecutor<br/>Stage Isolation Executor]
        PF[Pipeline Factory<br/>Pipeline Factory]
    end

    subgraph Agents["Agent Execution Layer"]
        IA[Idea Agent<br/>Single Agent]
        LOOP[PRD/Design/Plan/Coding<br/>Actor-Critic Loop]
        CA[Check Agent<br/>Quality Verification]
        DA[Delivery Agent<br/>Delivery Report]
    end

    subgraph Tools["Tool System Layer"]
        DT[Data Operation Tools]
        FT[File Operation Tools]
        HT[HITL Interaction Tools]
        VT[Validation Check Tools]
    end

    subgraph Storage["Persistence Layer"]
        FS[File System Storage<br/>.cowork/ Directory]
        SM[Session State Management]
    end

    CLI --> Pipeline
    Pipeline --> Agents
    Agents --> Tools
    Tools --> Storage
    Agents --> LLM[LLM Service<br/>OpenAI Compatible API]
```

### 1.4 Workflow Coordination Mechanisms

The system adopts the following coordination mechanisms to ensure consistency and reliability of multi-agent collaboration:

1. **Session Isolation Mechanism**: Each workflow instance corresponds to an independent Session, identified by UUID, ensuring data isolation during multi-project parallel development
2. **Stage Isolation Execution**: `StageExecutor` isolates escalate flags of each stage, preventing LoopAgent's exit signals from affecting the overall workflow
3. **State Persistence**: File system-based JSON persistence, supporting checkpoint recovery and state rollback
4. **Anti-Loop Protection**: Critic agents track feedback history, detect repeated issues, and escalate to manual review
5. **Rate Limiting**: LLM calls implement 2-second delay (<30 times/minute) through decorator pattern to ensure API compliance

## 2. Main Workflows Detailed

### 2.1 Complete Project Creation Workflow

**Business Value**: Achieves end-to-end automated conversion from natural language project ideas to deliverable software code, ensuring architectural simplicity and implementation completeness through seven-stage progressive development.

**Execution Flow**:

```mermaid
flowchart TD
    Start([User input: cowork new<br/>--project <name>]) --> Init[CLI initializes session<br/>Generate SessionID<br/>Status: InProgress]
    
    Init --> Idea[Idea Agent<br/>Organize idea → idea.md<br/>6-step workflow]
    
    Idea --> PRD_LOOP[PRD Loop<br/>Actor-Critic mode<br/>Maximum 3-5 iterations]
    subgraph PRD["Stage 3: Requirements Analysis (PRD)"]
        PRD_A[PRD Actor<br/>Analyze requirements generate PRD draft<br/>Constraints: 3-6 core requirements<br/>2-4 core features]
        PRD_H{Human Review<br/>review_with_feedback_content}
        PRD_C[PRD Critic<br/>Verify simplicity<br/>Anti-loop protection]
        
        PRD_A --> PRD_H
        PRD_H -->|edit| PRD_E[User edit<br/>Resave] --> PRD_A
        PRD_H -->|feedback| PRD_F[Record feedback<br/>provide_feedback] --> PRD_A
        PRD_H -->|pass| PRD_C
        PRD_C -->|Needs modification| PRD_F
    end
    
    PRD_LOOP --> Design_LOOP[Design Loop<br/>Constraints: 2-4 components]
    subgraph Design["Stage 4: Architecture Design (Design)"]
        DA[Design Actor<br/>Create minimalist architecture<br/>Prohibited: microservices/cache/queues]
        DH{Human Review}
        DC[Design Critic<br/>Verify component count<br/>Simplicity check]
        
        DA --> DH
        DH -->|edit| DE[Edit design] --> DA
        DH -->|feedback| DF[Record feedback] --> DA
        DH -->|pass| DC
        DC -->|Not passed| DF
    end
    
    Design_LOOP --> Plan_LOOP[Plan Loop<br/>Constraints: 5-12 core tasks]
    subgraph Plan["Stage 5: Task Planning (Plan)"]
        PA[Plan Actor<br/>Create task list<br/>Prohibited: test/optimization/deployment tasks]
        PH{Human Review}
        PC[Plan Critic<br/>Verify task dependencies<br/>Feature coverage check]
        
        PA --> PH
        PH -->|edit| PE[Edit tasks] --> PA
        PH -->|feedback| PF[Record feedback] --> PA
        PH -->|pass| PC
        PC -->|Not passed| PF
    end
    
    Plan_LOOP --> Coding_LOOP[Coding Loop<br/>Implement all pending tasks]
    subgraph Coding["Stage 6: Code Implementation (Coding)"]
        CA[Coding Actor<br/>Full code generation<br/>Complete all tasks in single iteration]
        CC[Coding Critic<br/>Verify task completion<br/>Code simplicity check]
        
        CA --> CC
        CC -->|Needs modification| CF[Provide feedback] --> CA
    end
    
    Coding_LOOP --> Check[Check Agent<br/>Permissive quality check<br/>File existence verification]
    
    Check -->|Not passed| Replan{Need replanning?}
    Replan -->|Yes| Goto[GoToStageTool<br/>Rollback to specified stage]
    Replan -->|No| Fix[Fix issues] --> Check
    
    Check --> Delivery[Delivery Agent<br/>Pre-check: list_files<br/>Generate delivery report]
    
    Delivery --> Save[Update session status<br/>Completed<br/>Save all artifacts]
    Save --> End([Project delivery complete])
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

### 2.2 Incremental Modification Workflow

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
    
    Check -->|Not passed| Fix[Fix issues] --> Check
    
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

### 2.3 Checkpoint Recovery Workflow

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

### 2.4 Version Rollback Workflow

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

## 3. Workflow Coordination and Control

### 3.1 Pipeline Orchestration Mechanism

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

### 3.2 State Management and Data Flow

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

### 3.3 Human-in-the-Loop (HITL) Coordination Mechanism

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

## 4. Exception Handling and Recovery

### 4.1 Resilient Error Recovery Architecture

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

### 4.2 Anti-Loop Protection Mechanism

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

### 4.3 Failover and Degradation Strategies

Degradation behavior when key services are unavailable:

| Failure Scenario | Impact Scope | Degradation Strategy | User Experience |
|------------------|--------------|---------------------|-----------------|
| LLM API Timeout | Current stage | Exponential backoff retry (1s, 2s, 4s) | Display waiting prompt |
| LLM API Completely Unavailable | Entire workflow | Save current state, prompt user to check configuration | Graceful exit, state retained |
| File System Permission Error | Specific tool call | Skip operation, record warning | Continue execution, remind afterwards |
| User Interruption (Ctrl+C) | Current execution | Save session status to InProgress | Resume possible |
| Disk Space Insufficient | Persistence operation | Pause new file writes, clean logs | Prompt to clear space |

## 5. Key Process Implementation

### 5.1 Actor-Critic Dual-Agent Mode

The system adopts **Actor-Critic** architecture in PRD, Design, Plan, and Coding stages, achieving generation and verification decoupling:

**Architecture Pattern**:

```mermaid
flowchart TB
    subgraph LoopAgent["LoopAgent Container"]
        Actor[Actor Agent<br/>Constructive work<br/>Generate content/code]
        Critic[Critic Agent<br/>Verification work<br/>Quality check]
        ExitTool[ExitLoopTool<br/>Loop exit control]
    end
    
    subgraph StateMgmt["State Management"]
        FB[Feedback History<br/>Feedback history tracking]
        Iter[Iteration Counter<br/>Iteration counter]
    end
    
    UserInput[User input/previous stage output] --> Actor
    Actor -->|Generate content| Review{Quality review}
    
    Review --> Critic
    Critic -->|Verification result| Decision{Passed?}
    
    Decision -->|Not passed| FB
    FB -->|History context| Actor
    Decision -->|Passed| ExitTool
    
    ExitTool -->|escalate=false<br/>Local exit| NextStage[Next stage]
    
    Iter -->|max iterations| ForceExit[Force exit<br/>request_human_review]
```

**Actor and Critic Responsibility Division**:

| Dimension | Actor Responsibilities | Critic Responsibilities |
|-----------|----------------------|--------------------------|
| **Core Task** | Create PRD/design documents/task lists/code | Verify simplicity, completeness, consistency |
| **Tool Usage** | create_*, save_*, write_file | get_*, check_*, provide_feedback |
| **Exit Condition** | No direct exit | Verify pass then call exit_loop() |
| **Human Interaction** | Respond to feedback, execute modification | Trigger human upgrade (loop protection) |
| **Constraint Enforcement** | Follow simplicity principles when coding | Force rejection of over-engineering design |

### 5.2 Tool System Integration Architecture

Tool system follows **ADK Tool Standard Interface**, achieving modular extension of agent capabilities:

```mermaid
flowchart TB
    subgraph ToolRegistry["Tool Registry Center"]
        direction TB
        DataTools[Data Operation Tools<br/>CRUD requirements/features/design/tasks]
        FileTools[File Operation Tools<br/>Secure path verification read/write]
        HITLTools[HITL Tools<br/>Interactive review]
        ValidationTools[Validation Tools<br/>Format/coverage/dependency check]
        ControlTools[Control Tools<br/>Feedback/replanning/stage jump]
    end
    
    subgraph SecurityLayer["Security and Governance Layer"]
        PathVal[Path security verification<br/>validate_path_security<br/>Prevent directory traversal]
        Timeout[Command timeout control<br/>30s limit]
        RateLimit[LLM rate limit<br/>2s delay decorator]
    end
    
    subgraph AgentContext["Agent Context"]
        SessionScope[Session scope<br/>session_id binding]
        ToolCtx[ToolContext trait<br/>Asynchronous execution context]
    end
    
    ToolRegistry --> SecurityLayer
    SecurityLayer --> AgentContext
```

**Key Tool Implementation Details**:

1. **File Operation Security**:
   - All paths verified through `validate_path_security`, prohibiting `../`, absolute paths, access outside project root directory
   - `RunCommandTool` implements 30-second timeout, intercepting blocking service commands (`npm start`, `python server.py`)

2. **Data Operation Atomicity**:
   - `CreateTaskTool` automatically handles dependencies, preventing cyclic dependencies (via DFS detection)
   - `UpdateTaskStatusTool` automatically cascades updates to associated Feature status

3. **HITL Tool Integration**:
   - `ReviewWithFeedbackContentTool` uses `dialoguer` crate to implement cross-platform CLI interaction
   - Supports `$EDITOR` environment variable to call user's preferred editor

### 5.3 LLM Service Abstraction and Rate Control

The system achieves decoupling from specific LLM providers through layered abstraction:

```mermaid
flowchart LR
    subgraph Config["Configuration Layer"]
        TOML[Configuration file<br/>cowork.toml]
        ENV[Environment variable fallback<br/>COWORK_LLM_API_KEY]
    end
    
    subgraph Factory["Factory Layer"]
        LLMConfig[LlmConfig<br/>Basic configuration structure]
        CreateClient[create_llm_client<br/>Factory function]
    end
    
    subgraph Client["Client Layer"]
        OpenAI[OpenAI Compatible Client<br/>adk-rust built-in]
        RateLimit[RateLimitedLlm<br/>Decorator wrapper]
    end
    
    subgraph Usage["Usage Layer"]
        Agents[Various stage agents<br/>Unified call via trait]
    end
    
    Config --> Factory
    Factory -->|Instantiate| Client
    RateLimit -->|Wrap| OpenAI
    Client -->|Arc<dyn Llm>| Usage
```

**Rate Limiting Strategy**:

```rust
// Pseudo code example
pub struct RateLimitedLlm {
    inner: Arc<dyn Llm>,
    delay_ms: u64,  // Default 2000ms
}

impl Llm for RateLimitedLlm {
    async fn generate_content(&self, request: Request) -> Result<Response> {
        // Mandatory delay before each call
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        self.inner.generate_content(request).await
    }
}
```

**Performance Characteristics**:
- **Delay**: Fixed 2-second delay ensures <30 times/minute API limit compliance
- **Concurrency**: Multiple LLM client sharing across agents through `Arc<dyn Llm>`
- **Fault Tolerance**: Network errors propagated via anyhow context, handled by ResilientAgent retry

### 5.4 Concurrency Processing and Resource Management

Although the system primarily uses sequential execution, it implements concurrency optimization in the following scenarios:

1. **File Fingerprint Calculation** (Modify workflow):
   - Uses asynchronous file I/O to calculate project file hashes in parallel
   - Ignore pattern matching (`.gitignore` rules) reduces invalid calculations

2. **Agent Initialization**:
   - Parallel initialization of stage agents during pipeline construction
   - Tool sets shared between agents (through `Arc`), avoiding duplicate instantiation

3. **Streaming Output Processing**:
   - LLM-generated content returned asynchronously via `futures::Stream`
   - Terminal real-time rendering and subsequent processing logic execute in parallel

**Resource Limitation Strategies**:

| Resource Type | Limitation Strategy | Implementation Mechanism |
|---------------|---------------------|-------------------------|
| **LLM API Calls** | Rate limiting | 2-second delay decorator |
| **Shell Commands** | Timeout control | 30-second hard timeout |
| **File Handles** | Scope limitation | Rust RAII + explicit close |
| **Session Storage** | Space management | Automatic cleanup of Failed status temporary files |
| **Memory Usage** | Streaming processing | Chunked reading of large files, avoiding one-time load |

---

**Document Generation Time**: 2026-01-31 05:56:42 (UTC)
**Version**: Based on Cowork Forge Architecture Design v1.0