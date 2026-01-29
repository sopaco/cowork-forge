# Core Workflow Orchestration in Cowork Forge

## Overview

The **Core Workflow Orchestration** domain is the central nervous system of Cowork Forge, responsible for orchestrating the end-to-end software development lifecycle—from capturing a user’s initial idea to delivering production-ready code. It governs the sequence, transitions, and conditional branching of AI-driven agent pipelines, ensuring that each stage of development is executed in a controlled, auditable, and human-in-the-loop (HITL) manner.

Unlike traditional development tools that treat coding as the primary activity, Cowork Forge treats *planning, validation, and alignment* as first-class citizens. The orchestration system enforces a disciplined workflow where each phase—idea refinement, requirements specification, system design, task decomposition, implementation, and delivery verification—is governed by an actor-critic agent pair and requires explicit human approval before progression. This prevents scope creep, ensures architectural simplicity, and maintains full traceability of decisions through persistent session artifacts.

The orchestration layer is not a monolithic controller but a modular, state-aware coordinator that dynamically constructs pipelines based on user intent and system state. It supports four distinct execution modes: **full pipeline initiation**, **resumption from any stage**, **partial restart**, and **incremental modification**, enabling both new project creation and evolutionary development.

## Core Responsibilities

The Core Workflow Orchestration domain has four primary responsibilities:

1. **Pipeline Assembly**: Dynamically construct agent sequences based on user commands (`new`, `resume`, `revert`, `modify`) and the current state of the project session.
2. **State-Driven Navigation**: Determine the appropriate starting point for execution by inspecting persisted artifacts (e.g., `prd.md`, `design.md`, code files) rather than relying solely on metadata.
3. **Stage Transition Management**: Enforce strict progression rules between workflow stages, ensuring that each stage is completed and validated before proceeding to the next.
4. **Resilient Workflow Control**: Enable recovery from failures or misalignments by triggering replanning, stage re-entry, or modification pipelines without losing context.

This domain acts as the bridge between user intent (expressed via CLI commands) and the execution of AI agents, ensuring that the system remains predictable, auditable, and user-controlled at all times.

## Pipeline Types and Execution Flows

Cowork Forge supports four distinct pipeline types, each tailored to a specific development scenario. These pipelines are assembled by the **Pipeline Coordinator** using a `SequentialAgent` pattern from the ADK framework, chaining together specialized agents with defined inputs, outputs, and tool dependencies.

### 1. Full Pipeline (`create_cowork_pipeline`)

**Use Case**: Starting a new project from scratch.

**Flow**:
```
Idea → PRD Loop → Design Loop → Plan Loop → Coding Loop → Check Agent → Delivery Agent
```

- **Agents Involved**: `IdeaAgent`, `PRDActor`, `PRDCritic`, `DesignActor`, `DesignCritic`, `PlanActor`, `PlanCritic`, `CodingActor`, `CodingCritic`, `CheckAgent`, `DeliveryAgent`
- **Behavior**: Executes all stages sequentially. Each stage generates an artifact (e.g., `idea.md`, `prd.md`, `design.md`, `tasks.json`) and waits for human approval via `ReviewWithFeedbackTool` before proceeding.
- **Validation Gates**: Critical validation occurs after PRD, Design, Plan, and Delivery stages. Rejection triggers a restart of the current stage.
- **Outcome**: A complete, validated software delivery with a `delivery_report.md` and all session artifacts preserved in `.cowork/sessions/<session-id>/`.

### 2. Resume Pipeline (`create_resume_pipeline`)

**Use Case**: Continuing a previously interrupted or paused development session.

**Flow**:
```
Check for existing artifacts → Determine last completed stage → Resume from next stage
```

- **State Detection Logic**:
  - If `code/` directory contains files → Start from `CheckAgent`
  - Else if `tasks.json` exists → Start from `Coding Loop`
  - Else if `design.md` exists → Start from `Plan Loop`
  - Else if `prd.md` exists → Start from `Design Loop`
  - Else → Start from `PRD Loop`

- **Key Mechanism**: Relies on **file system existence checks** rather than session metadata to determine progress. This ensures resilience against metadata corruption or manual edits.
- **Advantage**: Enables recovery from crashes, interruptions, or tool failures without requiring users to manually track their progress.

### 3. Partial Pipeline (`create_partial_pipeline`)

**Use Case**: Restarting from a specific stage (e.g., redesigning the architecture or re-planning tasks).

**Flow**:
```
User specifies --goto-stage <stage> → Validate stage → Skip all prior stages → Execute from target onward
```

- **Supported Stages**: `idea`, `prd`, `design`, `plan`, `coding`, `check`, `delivery`
- **Behavior**: Loads the full session state (including prior approvals and artifacts) but skips all stages before the target. For example, invoking `cowork --goto-stage design` will:
  - Load `idea.md` and `prd.md`
  - Skip `PRD Loop`
  - Restart `Design Loop` with existing PRD as context
  - Continue through `Plan Loop`, `Coding Loop`, etc.
- **Use Case Example**: A product owner realizes the initial design is too complex. They use `--goto-stage design` to regenerate the architecture without redefining requirements.

### 4. Modify Pipeline (`create_modify_pipeline`)

**Use Case**: Applying incremental changes to an already delivered project.

**Flow**:
```
Change Request → Change Triage Agent → Delta Analysis → Targeted Pipeline Segment → Modify Delivery Agent
```

- **Agents Involved**: `ModifyTriageAgent`, `CodePatchAgent`, `CheckAgent`, `ModifyDeliveryAgent`
- **Process**:
  1. `ModifyTriageAgent` analyzes the change request and determines its impact across PRD, design, plan, and code layers.
  2. System computes the delta between the base session and current state.
  3. If the change affects PRD or design, the system may restart those loops. Otherwise, it initiates a **targeted coding loop**.
  4. `CodePatchAgent` implements changes using `WriteFileTool` and `ReadFileTool`, with validation by `CodingCritic`.
  5. `CheckAgent` verifies feature coverage and task completion.
  6. `ModifyDeliveryAgent` generates a comprehensive change report integrating feedback history and version context.
- **Outcome**: A versioned, traceable update that preserves the original delivery context while enabling evolutionary development.

## Key Components

### Pipeline Coordinator

**Location**: `crates/cowork-core/src/pipeline/mod.rs`

The **Pipeline Coordinator** is the central orchestrator that exposes the public API for pipeline creation. It does not execute agents directly but assembles them into a `SequentialAgent` chain and returns it to the CLI entry point for execution.

#### Key Functions:
```rust
pub fn create_cowork_pipeline(session_id: &str, llm_client: Arc<dyn LlmClient>) -> Arc<dyn Agent>
pub fn create_resume_pipeline(session_id: &str, llm_client: Arc<dyn LlmClient>) -> Arc<dyn Agent>
pub fn create_partial_pipeline(stage: &str, session_id: &str, llm_client: Arc<dyn LlmClient>) -> Arc<dyn Agent>
pub fn create_modify_pipeline(session_id: &str, llm_client: Arc<dyn LlmClient>) -> Arc<dyn Agent>
```

- Each function:
  - Loads the session state via the **Storage Layer**
  - Constructs the appropriate sequence of agents
  - Binds each agent to the shared `llm_client` and toolset
  - Returns a single `Arc<dyn Agent>` that encapsulates the entire pipeline

#### Design Philosophy:
- **Immutability**: Pipelines are constructed once and executed once. No runtime mutation.
- **Decoupling**: The coordinator has no knowledge of agent internals—only their interfaces and dependencies.
- **Session Isolation**: Each pipeline operates within a unique `.cowork/sessions/<session-id>/` directory, preventing cross-session contamination.

### Stage Navigator

**Location**: `crates/cowork-core/src/tools/goto_stage_tool.rs`

The **Stage Navigator** enables non-linear progression through the workflow via the `GotoStageTool`. It is the mechanism that allows users to jump to any stage and resume execution.

#### Key Responsibilities:
- Validates that the target stage is logically reachable (e.g., cannot jump from `idea` to `delivery` without intermediate steps).
- Loads the full session state from disk using `load_session()`.
- Determines which stages to skip based on the presence of artifacts.
- Updates session metadata (`index.json`) to reflect the new starting point.

#### Example: `GotoStageTool::execute()`
```rust
pub fn execute(&self, stage: &str, session_id: &str) -> Result<()> {
    let session_dir = get_session_dir(session_id)?;
    let session_meta = load_session_metadata(&session_dir)?;

    // Validate stage
    let valid_stages = ["idea", "prd", "design", "plan", "coding", "check", "delivery"];
    if !valid_stages.contains(&stage) {
        return Err(WorkflowError::InvalidStage(stage.to_string()));
    }

    // Load all artifacts
    let idea = load_idea(&session_dir)?;
    let prd = load_prd(&session_dir)?;
    let design = load_design(&session_dir)?;
    let tasks = load_tasks(&session_dir)?;
    let code_files = list_code_files(&session_dir)?;

    // Determine starting point
    let start_stage = determine_start_stage(idea, prd, design, tasks, code_files, stage)?;

    // Update metadata to reflect new starting point
    update_session_metadata(&session_dir, start_stage)?;

    Ok(())
}
```

This tool is critical for enabling iterative refinement and error recovery. It ensures that users retain full context (feedback, approvals, prior artifacts) while focusing on the specific stage they wish to revise.

## Interaction with Other Domains

The Core Workflow Orchestration domain is deeply integrated with all other core domains, acting as the primary consumer of their services.

| Domain | Interaction Type | Key Dependencies |
|--------|------------------|------------------|
| **Intelligent Agent Control** | Service Call | Loads agent instructions (`DESIGN_ACTOR_INSTRUCTION`, `CODING_CRITIC_INSTRUCTION`) to instantiate agents. Uses `SequentialAgent` wrapper for execution. |
| **Data & Artifact Management** | Data Dependency | Relies on `ProjectIdea`, `PRD`, `DesignComponent`, `ImplementationTask`, and `SessionMetadata` structs to maintain state. Uses `load_session()` and `save_artifact()` to persist and retrieve artifacts. |
| **Tooling & Operations** | Service Call | Invokes `ReviewWithFeedbackTool`, `CheckFeatureCoverageTool`, `SaveDeliveryReportTool`, `GotoStageTool`, and `RequestReplanningTool` to enforce validation, HITL, and control logic. |
| **Infrastructure Support** | Service Call | Obtains `LlmClient` instances via `create_llm_client()` to enable agent reasoning. Uses `RateLimiter` to ensure compliance with API usage policies. |

### Example: Full Pipeline Construction
```rust
// crates/cowork-core/src/pipeline/mod.rs
pub fn create_cowork_pipeline(session_id: &str, llm_client: Arc<dyn LlmClient>) -> Arc<dyn Agent> {
    let session_dir = get_session_dir(session_id)?;
    
    let idea_agent = create_idea_agent(llm_client.clone(), &session_dir);
    let prd_loop = create_prd_loop(llm_client.clone(), &session_dir);
    let design_loop = create_design_loop(llm_client.clone(), &session_dir);
    let plan_loop = create_plan_loop(llm_client.clone(), &session_dir);
    let coding_loop = create_coding_loop(llm_client.clone(), &session_dir);
    let check_agent = create_check_agent(llm_client.clone(), &session_dir);
    let delivery_agent = create_delivery_agent(llm_client.clone(), &session_dir);

    SequentialAgent::new(vec![
        idea_agent,
        prd_loop,
        design_loop,
        plan_loop,
        coding_loop,
        check_agent,
        delivery_agent,
    ])
}
```

Here, the coordinator does not implement logic for generating PRDs or writing code—it delegates to agents defined in the **Intelligent Agent Control** domain and uses tools from **Tooling & Operations** to interact with the environment.

## State Management and Persistence

Cowork Forge avoids in-memory state management entirely. Instead, it uses **file-based session persistence** as the single source of truth.

### Session Directory Structure
```
.cowork/
└── sessions/
    └── <session-id>/
        ├── input.json          // User-provided command and parameters
        ├── index.json          // Session metadata: current stage, status, timestamps
        ├── idea.md             // Finalized project idea
        ├── prd.md              // Approved Product Requirements Document
        ├── design.md           // System architecture diagram and components
        ├── tasks.json          // Structured list of implementation tasks
        ├── code/               // Generated source files
        │   ├── main.rs
        │   └── utils.rs
        ├── feedback/           // Human feedback logs
        │   ├── prd_1.json
        │   └── design_2.json
        ├── delivery_report.md  // Final delivery documentation
        └── change_requests/    // For modified sessions
            └── cr_001.json
```

### Key Persistence Mechanisms:
- **`load_session()`**: Reads all artifacts into structured data models (`PRD`, `DesignComponent`, etc.) for agent consumption.
- **`save_artifact()`**: Persists generated content (e.g., `prd.md`, `delivery_report.md`) to disk with versioned naming.
- **`update_task_status()`**: Updates task completion status in `tasks.json` after each coding iteration.
- **`get_session_dir()`**: Ensures session isolation and path safety.

This approach ensures:
- **Auditability**: Every decision and artifact is traceable to a file.
- **Recoverability**: Sessions can be restored even after system crashes.
- **Transparency**: Users can inspect, edit, or version-control artifacts directly.

## Human-in-the-Loop (HITL) Integration

The orchestration system is fundamentally **human-centered**. Every major transition requires explicit user approval, enforced through HITL tools:

| Stage | HITL Tool | Purpose |
|-------|-----------|---------|
| Idea Finalization | `ReviewAndEditContentTool` | Allows user to edit and approve the summarized idea |
| PRD Approval | `ReviewWithFeedbackTool` | Validates scope, rejects non-core features |
| Design Approval | `ReviewWithFeedbackTool` | Ensures architecture is simple and complete |
| Plan Approval | `ReviewWithFeedbackTool` | Confirms task granularity and dependencies |
| Code Feedback | `ProvideFeedbackTool` | Captures user comments for agent learning |
| Delivery Validation | `ReviewWithFeedbackTool` | Final sign-off before report generation |

These tools open the user’s default editor (e.g., `vim`, `code`) to display generated content. The pipeline pauses until the user saves and exits, ensuring no stage proceeds without conscious approval.

## Resilience and Error Handling

The orchestration system is designed to handle failures gracefully:

- **Agent Failure**: If an agent exceeds its iteration limit or produces invalid output, the `ResilientStream` wrapper triggers a HITL recovery prompt: _“The AI agent failed to complete this task. Would you like to: (1) Provide feedback, (2) Restart this stage, or (3) Skip to next?”_
- **Validation Failure**: If `CheckFeatureCoverageTool` detects missing features, the system triggers `RequestReplanningTool`, looping back to the Plan or Design stage.
- **Metadata Corruption**: By relying on file existence rather than metadata alone, the system can recover even if `index.json` is deleted or corrupted.
- **Tool Failure**: All tools are sandboxed with path validation (e.g., `WriteFileTool` prevents directory traversal), ensuring system safety.

## Practical Usage Scenarios

### Scenario 1: New Project Initiation
```bash
cowork new --name my-app
```
- Creates a new session
- Runs full pipeline
- Pauses after each stage for user review
- Generates `delivery_report.md` upon completion

### Scenario 2: Redesigning Architecture
```bash
cowork --goto-stage design
```
- Loads existing `idea.md` and `prd.md`
- Skips PRD stage
- Re-runs Design Actor/Critic with prior context
- User approves new design → continues to Plan and Coding

### Scenario 3: Adding a Feature Post-Delivery
```bash
cowork modify --request "Add user authentication"
```
- Launches `ModifyTriageAgent`
- Analyzes impact: requires new API endpoints and auth middleware
- Auto-restarts Design and Plan loops
- Implements changes via `CodePatchAgent`
- Generates `change_report_v2.md` integrating feedback history

### Scenario 4: Recovery from Failed Coding
```bash
# Agent detects code complexity exceeds limits
> Critical issue detected. Requesting replanning...
> Would you like to: (1) Revise task plan, (2) Provide feedback, (3) Skip?
```
User selects (1) → System restarts Plan Loop → New task list generated → Coding resumes.

## Design Principles and Best Practices

1. **Simplicity Enforcement**: Critic agents reject non-core features (e.g., “Add unit tests” or “Deploy to Kubernetes”) unless explicitly requested. This prevents over-engineering.
2. **Artifact-Driven State**: Progress is determined by file existence, not metadata. This ensures resilience and transparency.
3. **Modular Agents**: Each stage is an independent, reusable agent. This enables future extension (e.g., adding a “Testing Agent”).
4. **HITL as a Gatekeeper**: Human approval is mandatory at every major milestone. AI generates; humans decide.
5. **Non-Linear Progression**: Users can jump to any stage. This supports iterative refinement and agile development.
6. **Session Isolation**: Each project is self-contained. No global state. No interference between projects.

## Conclusion

The Core Workflow Orchestration in Cowork Forge is not merely a task runner—it is a **structured, auditable, human-guided development framework** that transforms how software is built. By enforcing discipline through agent pairs, persistent artifacts, and mandatory human validation, it eliminates the chaos of unstructured AI-assisted development while preserving the speed and creativity of automation.

For developers, it reduces cognitive load by automating planning and scaffolding.  
For product managers, it ensures alignment between vision and implementation.  
For engineering leads, it enforces consistency and provides full traceability.

This orchestration layer is what makes Cowork Forge not just an AI tool, but a **new paradigm for software development**: one where AI is the assistant, and the human remains the architect.