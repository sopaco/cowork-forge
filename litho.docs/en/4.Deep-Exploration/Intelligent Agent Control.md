# Intelligent Agent Control: Technical Documentation

## Overview

The Intelligent Agent Control domain in Cowork Forge implements a human-in-the-loop (HITL) actor-critic architecture that orchestrates the complete software development lifecycle—from capturing a user's project idea to delivering a production-ready codebase. This system enforces architectural simplicity, prevents scope creep, and ensures traceability through structured, prompt-driven agent workflows. Each stage of development is governed by paired agent roles: an Actor that generates artifacts and a Critic that validates them against strict quality gates. Human approval is required at critical decision points, creating a controlled, auditable development environment that balances automation with human oversight.

The implementation is entirely configuration-driven, with agent behavior defined in natural language instruction templates stored as Rust constants. These prompts contain explicit workflows, mandatory steps, rejection criteria, and tool usage guidelines, enabling easy iteration and policy updates without code changes. The system leverages a modular design where each agent operates independently, communicating through a well-defined set of tools that interact with persistent session artifacts and the user interface.

## Core Architecture

### Actor-Critic Agent Pairs

The Intelligent Agent Control domain employs a strict actor-critic pattern at every major development stage, creating a feedback loop that ensures quality and alignment with user intent.

#### Actor Agents
Actor agents are responsible for generating deliverables at each stage of the workflow. Each actor follows a rigid, step-by-step instruction set that mandates specific actions and prohibits over-engineering. Key actor agents include:

- **Idea Agent**: Captures and structures the user's initial project concept into a standardized `idea.md` document.
- **PRD Actor**: Translates the approved idea into a minimal Product Requirements Document, focusing exclusively on core business functionality.
- **Design Actor**: Creates a simple system architecture with 2–4 components, avoiding microservices, caching layers, or complex infrastructure unless explicitly required.
- **Plan Actor**: Breaks down the approved design into 5–12 implementation tasks, excluding testing, optimization, or deployment tasks.
- **Coding Actor**: Implements the approved tasks with simple, readable code, avoiding complex patterns and unnecessary abstractions.
- **Modify Triage Agent**: Analyzes change requests and determines which pipeline stages need to be re-executed.
- **Code Patch Agent**: Implements incremental changes to existing code without restarting the full workflow.
- **Delivery Agent**: Generates the final delivery report only after verifying that actual code files exist.

#### Critic Agents
Critic agents serve as gatekeepers, validating the output of their corresponding actor agents before allowing progression. They enforce strict simplicity principles and reject non-core features. Key critic agents include:

- **PRD Critic**: Rejects any requirements involving performance, testing, deployment, or monitoring unless explicitly requested in the idea.
- **Design Critic**: Blocks architectures with more than four components, microservices, message queues, or complex caching layers.
- **Plan Critic**: Ensures all tasks focus solely on core feature implementation and rejects any testing, optimization, or infrastructure-related tasks.
- **Coding Critic**: Verifies that all tasks are completed and that code files exist, while also detecting fundamental architectural conflicts that require replanning.

Each critic agent performs mandatory checks on both structured data (via `get_requirements()`, `get_design()`, `get_plan()`) and persisted artifacts (via `read_file()`). Failure to pass any check results in a `provide_feedback()` call that forces the actor to revise its output.

### Human-in-the-Loop (HITL) Validation

Human intervention is embedded at every critical juncture through the `review_with_feedback_content()` and `review_and_edit_content()` tools. These tools trigger interactive sessions in the user’s terminal or editor, allowing users to review, edit, and approve AI-generated artifacts.

- **Mandatory Review Points**: After each actor generates a draft (PRD, design, plan), the system requires explicit user approval before proceeding.
- **Feedback Handling**: If a user edits content during review, the actor **must** save the revised version using the appropriate `save_*` tool. Ignoring user edits results in an incomplete or incorrect artifact.
- **Resilient Recovery**: When agents exceed iteration limits or encounter unresolvable issues, the `ResilientStream` wrapper triggers a `provide_feedback()` call that escalates to human intervention, allowing users to guide the system or abort the pipeline.

This HITL mechanism ensures that the AI system remains aligned with user intent and prevents automated drift into over-engineered or misaligned solutions.

## Instruction Set Implementation

The behavior of all agents is defined in a set of 10 Rust modules, each containing constant strings with detailed natural language instructions. These instructions are loaded by the agent builders and injected into LLM agents via the ADK framework.

### Key Instruction Modules

#### `idea.rs` – Project Idea Capture
The Idea Agent transforms a user’s initial concept into a structured markdown document (`idea.md`) with clear sections for problem statement, target users, key goals, and technical considerations. The instruction mandates that after saving the initial draft, the agent **must** invoke `review_and_edit_content()` to allow the user to refine it. Crucially, if the user edits the content, the agent **must** call `save_idea()` again with the edited content—otherwise, the user’s changes are lost. This ensures the final `idea.md` reflects the user’s true intent.

#### `prd.rs` – Requirements Definition
The PRD Actor generates a minimal requirements document with 3–6 core requirements and 2–4 essential features, explicitly excluding performance, testing, deployment, or monitoring requirements. The PRD Critic enforces this simplicity by rejecting any requirement mentioning “performance,” “scalability,” “CI/CD,” or “monitoring.” The actor must use `review_with_feedback_content()` to obtain user feedback and **must** use the final edited version (not the original draft) to create requirements via `create_requirement()` and `add_feature()`. Failure to do so results in immediate rejection by the critic.

#### `design.rs` – System Architecture
The Design Actor creates a monolithic architecture with 2–4 components, using simple tech stacks (e.g., Flask + SQLite). The Design Critic strictly rejects architectures with microservices, Redis, RabbitMQ, or more than four components. The actor must use `review_with_feedback_content()` to obtain user input and then create each component via `create_design_component()`. The critic validates both the JSON data model and the persisted `design.md` file, ensuring no critical components are omitted.

#### `plan.rs` – Implementation Planning
The Plan Actor decomposes the design into 5–12 implementation tasks, each focused on a single core feature. Critically, the instruction forbids tasks related to testing, performance optimization, or deployment. The Plan Critic scans each task for keywords like “test,” “optimize,” or “deploy” and rejects the plan if any are found. The actor must use `review_with_feedback_content()` to obtain user feedback and then create tasks via `create_task()`. If the critic identifies non-core tasks, the actor **must** delete them using `delete_task()` and cannot recreate them.

#### `coding.rs` – Code Implementation
The Coding Actor implements all pending tasks in one go, writing simple, clean code without complex patterns. It may dynamically adjust the plan using `create_task()`, `update_task()`, or `delete_task()` if it discovers missing dependencies or architectural flaws. The Coding Critic performs two critical checks:
1. **Completion**: All tasks must be marked as “completed” via `update_task_status()`.
2. **Existence**: Actual code files must exist, verified using `list_files(".")`.

If files are missing despite tasks being marked complete, the critic calls `provide_feedback()` to request implementation. If the critic detects a fundamental architectural mismatch (e.g., stateful code in a stateless environment), it triggers `request_replanning()` to restart the pipeline from an earlier stage.

#### `check.rs` – Quality Validation
The Check Agent performs minimal validation: verifying feature coverage and task dependencies. It does not run tests, lint code, or check performance. Its role is to approve the project if the structure is complete, making it a lenient final gate before delivery.

#### `delivery.rs` – Final Delivery
The Delivery Agent is the final stage and the most critical. It **must** verify that actual code files exist using `list_files(".")` before generating the delivery report. If no code files are found—even if all tasks are marked “completed”—the agent **must not** generate the report and instead outputs a failure message. This prevents false delivery claims and ensures that the final artifact reflects real implementation.

### Instruction Modularity and Re-export

All instruction modules are aggregated in `crates/cowork-core/src/instructions/mod.rs`, which re-exports all agent constants for use by the agent execution layer. This modular structure allows for independent development and testing of each agent’s behavior while maintaining a clean, centralized interface.

```rust
pub mod idea;
pub mod prd;
pub mod design;
pub mod plan;
pub mod coding;
pub mod check;
pub mod delivery;
pub mod modify;
pub mod code_patch;
pub mod modify_delivery;

pub use idea::*;
pub use prd::*;
pub use design::*;
pub use plan::*;
pub use coding::*;
pub use check::*;
pub use delivery::*;
pub use modify::*;
pub use code_patch::*;
pub use modify_delivery::*;
```

## Interaction Protocol

### Agent Workflow Sequence

The system follows a strictly sequential pipeline, with each agent’s output serving as the input for the next:

1. **Idea Agent** → Captures user input → Saves `idea.md`
2. **PRD Actor** → Generates requirements → **PRD Critic** validates → User reviews via `review_with_feedback_content()` → Saves `prd.md`
3. **Design Actor** → Creates architecture → **Design Critic** validates → User reviews → Saves `design.md`
4. **Plan Actor** → Breaks down into tasks → **Plan Critic** validates → User reviews → Saves task list
5. **Coding Actor** → Implements code → **Coding Critic** validates → Files verified → Updates task status
6. **Check Agent** → Verifies coverage and dependencies → Approves
7. **Delivery Agent** → Verifies code files exist → Generates `delivery_report.md`

### Tool-Based Interaction

Agents interact with the system state and environment exclusively through a set of session-scoped tools:

| Tool Category | Tools | Purpose |
|---------------|-------|---------|
| **Artifact Management** | `save_idea()`, `save_prd_doc()`, `save_design_doc()`, `save_delivery_report()` | Persist structured artifacts to `.cowork/sessions/` |
| **Data Access** | `load_idea()`, `get_requirements()`, `get_design()`, `get_plan()` | Retrieve structured data models |
| **User Interaction** | `review_and_edit_content()`, `review_with_feedback_content()` | Enable HITL review and editing |
| **File Operations** | `read_file()`, `write_file()`, `list_files()` | Read, write, and verify code files |
| **Task Control** | `create_task()`, `delete_task()`, `update_task()`, `update_task_status()` | Dynamically adjust implementation plan |
| **Validation** | `check_feature_coverage()`, `check_task_dependencies()` | Verify structural integrity |
| **Feedback & Control** | `provide_feedback()`, `request_replanning()`, `goto_stage()` | Enforce quality gates and trigger recovery |

### Error Handling and Recovery

The system employs a resilient execution model:
- **Critic Rejection**: If a critic rejects output, the actor must revise and resubmit.
- **Iteration Limits**: If an agent exceeds a maximum number of iterations, the `ResilientStream` wrapper triggers human intervention.
- **Replanning**: The Coding Critic can trigger `request_replanning()` for fundamental architectural issues, causing the pipeline to restart from the PRD, Design, or Plan stage.
- **Stage Navigation**: Users can resume the pipeline from any stage using `goto_stage()`, preserving all prior artifacts and feedback.

## Design Principles

### Simplicity Enforcement
The most defining characteristic of the Intelligent Agent Control domain is its uncompromising enforcement of simplicity. Every critic agent is explicitly instructed to reject non-core features:
- **PRD Critic**: Rejects performance, testing, deployment requirements.
- **Design Critic**: Rejects microservices, caching, queues.
- **Plan Critic**: Rejects test, optimization, infrastructure tasks.
- **Coding Critic**: Rejects over-engineered code patterns.

This ensures that the system delivers minimal viable products (MVPs) rather than over-designed systems.

### Configuration-Driven Behavior
All agent behavior is defined in text prompts, not code logic. This enables:
- Rapid iteration of policies without code changes.
- Easy auditing of agent rules via source files.
- Consistent behavior across LLM providers.

### State Persistence
All artifacts (idea.md, prd.md, design.md, tasks, feedback) are persisted to `.cowork/sessions/` using the Data & Artifact Management module. This ensures:
- Full audit trail of decisions.
- Recovery from failures.
- Non-linear workflow resumption via `goto_stage()`.

### Modular and Independent Agents
Each agent operates as a self-contained unit with clear responsibilities. This enables:
- Parallel development of agent instructions.
- Independent testing and debugging.
- Reuse of agents in different workflows (e.g., Modify Pipeline).

## Conclusion

The Intelligent Agent Control domain in Cowork Forge represents a novel approach to AI-assisted software development: a structured, rule-governed, human-in-the-loop system that prioritizes simplicity, traceability, and user control. By encoding complex development workflows into natural language instructions and enforcing them through actor-critic pairs, the system achieves high-quality outcomes without requiring complex code logic or machine learning training.

This implementation ensures that developers, product managers, and engineering leads can accelerate development while maintaining full oversight, preventing scope creep, and delivering clean, minimal, and auditable software products.