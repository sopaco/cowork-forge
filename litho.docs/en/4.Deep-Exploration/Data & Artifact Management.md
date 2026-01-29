# Data & Artifact Management in Cowork Forge

## Overview

The Data & Artifact Management module serves as the persistent state backbone of the Cowork Forge system, ensuring complete traceability, auditability, and recovery capability throughout the entire software development lifecycle. This module implements a session-based, file-system-centric architecture that stores all development artifacts—ranging from user ideas to final delivery reports—in a structured, hierarchical directory structure under `.cowork/sessions/`. By leveraging Serde for JSON serialization and Rust's robust file system APIs, the module provides a reliable, human-readable, and machine-parsable persistence layer that enables the system's core principles: simplicity enforcement, human-in-the-loop validation, and resilient execution.

The module is organized into two primary sub-components: **Data Models** and **Storage Layer**. The Data Models define the structured, serializable Rust structs that represent the canonical forms of development artifacts (e.g., `PRD`, `ImplementationTask`, `FeatureList`). The Storage Layer implements the CRUD operations that persist these models to disk, manage session directories, and handle session inheritance. Together, they form a critical bridge between the Intelligent Agent Control domain (which generates content) and the Tooling & Operations domain (which reads and validates it), ensuring that every stage of the workflow is grounded in persistent, verifiable state.

## Data Models: The Canonical Artifacts Schema

The Data Models sub-module defines the authoritative, Serde-serializable data structures that represent every significant artifact generated during the development process. These models are not merely data containers; they are the contract that enforces consistency, traceability, and semantic meaning across the entire system. Each model is designed with explicit schema versioning, timestamps, and structured fields to support validation, comparison, and automated status updates.

### Core Artifact Models

The system defines a comprehensive set of models to capture the evolution of a project from concept to delivery:

- **`ProjectIdea`**: Captures the initial user-provided concept, stored as a Markdown file (`idea.md`) in the `artifacts/` directory. While the raw content is stored as text, its metadata (session ID, creation time) is tracked in the `SessionInput` model.
- **`PRD` (Product Requirements Document)**: Represents the approved feature set, structured as a list of `Requirement` objects. Each requirement includes an ID (`REQ-001`), title, description, priority (`High`, `Medium`, `Low`), category (`Functional`, `NonFunctional`), and acceptance criteria. This model ensures that requirements are granular, testable, and unambiguous.
- **`FeatureList`**: A critical model that maps high-level features (`FEAT-001`) to their underlying requirements and implementation tasks. Each `Feature` has a `status` field (`Pending`, `InProgress`, `Completed`, `Blocked`) and a list of associated `requirement_ids` and `assigned_to_tasks`. This model is the central hub for tracking progress and enabling automated status updates.
- **`DesignSpec`**: Encapsulates the system architecture, including `Architecture` (components, data models), `TechnologyStack` (backend, frontend, database), and `DeploymentInfo`. The `DesignComponent` struct within this model links each component to specific features, creating a direct traceability path from user need to technical implementation.
- **`ImplementationPlan`**: Breaks down the design into discrete, executable `Task` objects. Each `Task` is linked to a `feature_id` and `component_id`, has a `status` (e.g., `Completed`), and lists `files_to_create` and `acceptance_criteria`. This model transforms abstract design into concrete, trackable work items.
- **`CodeMetadata`**: Tracks the state of the codebase, including a list of `FileMetadata` entries (path, task ID, lines of code, test coverage) and build/test status. This model provides the ground truth for the Delivery Agent's validation logic.
- **`SessionMetadata`**: Maintains the workflow state for each session, including the `current_stage` (e.g., `Coding`, `Check`) and `restart_reason`. This enables the Stage Navigator to resume workflows from any point.

### Session and Change Management Models

Beyond core artifacts, the module defines models for session lifecycle and change management:

- **`SessionRecord` and `ProjectIndex`**: The `ProjectIndex` is a single JSON file at `.cowork/index.json` that acts as the project's master registry. It tracks all `SessionRecord` instances, each with a `session_id`, `session_type` (`New`, `Modify`, `Revert`), `status` (`InProgress`, `Completed`, `Failed`), and `base_session_id`. This enables the system to maintain a complete history of all development attempts.
- **`ChangeRequest`**: Used exclusively in `modify` workflows, this model captures the user's change idea, its `scope` (which artifacts need updating), `acceptance_criteria`, and a `ChangeAnalysis` from the Modify Triage Agent. It is persisted as `change_request.json` in the session's root.
- **`PatchMetadata`**: Tracks the delta of a `modify` session, listing `added_files`, `modified_files`, `deleted_files`, and `artifact_updates`. This provides a precise audit trail of what changed during an incremental update.

All models are designed with `#[derive(Serialize, Deserialize)]` for seamless JSON serialization, and use `#[serde(skip_serializing_if = "Option::is_none")]` to avoid bloating files with empty fields. Default implementations (`impl Default`) ensure that models can be instantiated in a valid state even when no prior data exists.

## Storage Layer: File System as the Database

The Storage Layer implements a robust, session-isolated file system hierarchy that treats the local disk as the primary, durable database. This design choice prioritizes simplicity, transparency, and resilience over complex database infrastructure, aligning with Cowork Forge's philosophy of avoiding over-engineering.

### Directory Structure and Session Isolation

The storage architecture is meticulously organized under the `.cowork/` directory, with each development session isolated in its own subdirectory:

```
.cowork/
├── index.json                 # Project-wide session registry
├── sessions/
│   ├── <session_id_1>/        # Session 1 (e.g., "s1-20240101-123456")
│   │   ├── input.json         # Session metadata (type, base session)
│   │   ├── artifacts/         # Human-readable documents
│   │   │   ├── idea.md
│   │   │   ├── prd.md
│   │   │   ├── design.md
│   │   │   └── delivery_report.md
│   │   ├── state/             # Structured, machine-readable data
│   │   │   ├── requirements.json
│   │   │   ├── feature_list.json
│   │   │   ├── design_spec.json
│   │   │   ├── implementation_plan.json
│   │   │   ├── code_metadata.json
│   │   │   ├── feedback.json
│   │   │   └── meta.json
│   │   ├── patch/             # For modify sessions
│   │   │   └── metadata.json
│   │   └── logs/              # Agent execution logs
│   └── <session_id_2>/        # Session 2
└── config.toml                # LLM configuration (not managed by this module)
```

This structure ensures that:
- **Artifacts** (`.md` files) are human-readable and can be edited directly in any text editor.
- **State** (`.json` files) are machine-parsable and serve as the input/output contract for agents and tools.
- **Sessions are immutable**; a new session is created for every `new`, `modify`, or `revert` command, preventing accidental overwrites of prior work.

### Core CRUD Operations

The Storage Layer provides a comprehensive set of functions for managing this structure:

- **Directory Management**: Functions like `get_cowork_dir()`, `get_session_dir(session_id)`, and `get_project_root()` ensure all paths are correctly resolved and directories are created with `fs::create_dir_all()`. This guarantees a clean, predictable environment.
- **Session Initialization**: `init_project_index(project_name)` creates the `.cowork/` directory and `index.json` if it doesn't exist, marking the project as initialized.
- **Artifact Persistence**: Simple functions like `save_prd_doc(session_id, content)` and `load_idea(session_id)` handle the read/write of Markdown files in the `artifacts/` directory.
- **State Management**: Functions like `save_requirements(session_id, &requirements)` and `load_implementation_plan(session_id)` serialize/deserialize the structured data models to/from the `state/` directory. Each function includes error handling with `anyhow` to provide context-rich errors (e.g., "Failed to parse requirements.json").
- **Session Inheritance**: The `init_session_from_base(new_session_id, base_session_id)` function is a cornerstone of the system's resilience. When a user initiates a `modify` or `revert` workflow, this function copies all state files (`requirements.json`, `feature_list.json`, etc.) and relevant artifacts (`idea.md`, `prd.md`) from the base session into the new session's directory. This ensures that agents start with a complete, accurate context, rather than an empty slate, enabling seamless incremental development.

### Automated Status Updates and Session Lifecycle Management

A key innovation of the Storage Layer is its **automatic feature status update** logic. The `update_feature_status_if_needed(session_id, feature_id)` function is called whenever a task's status changes. It queries the `ImplementationPlan` to find all tasks for the given feature, then determines the feature's new status based on the following rules:
- **Completed**: All tasks are `Completed`.
- **Blocked**: Any task is `Blocked`.
- **InProgress**: Any task is `InProgress` and none are `Blocked`.
- **Pending**: All tasks are `Pending`.

This automated logic eliminates manual state management, ensuring that the `FeatureList` always accurately reflects the project's true state. This is critical for the `CheckFeatureCoverageTool` and `Delivery Agent`, which rely on this data to validate that all requirements have been met.

The module also manages the session lifecycle through `ProjectIndex` operations:
- `mark_session_completed(session_id)` updates the session's status to `Completed` and sets `latest_successful_session` in the `ProjectIndex`.
- `mark_session_failed(session_id)` marks a session as failed for audit purposes.
- `get_latest_successful_session()` retrieves the most recent successful session ID, which is used as the default base for `modify` operations.

## Interaction with Other Domains

The Data & Artifact Management module is the central data hub for the entire Cowork Forge system, interacting deeply with all other domains.

### Interaction with Intelligent Agent Control

Agents rely on the Storage Layer to persist their outputs and retrieve their inputs. The `IdeaAgent` uses `save_idea()` to store its summary, while the `PRD Actor` uses `save_requirements()` to persist its generated requirements. Critic agents, such as the `PRD Critic`, use `load_requirements()` to validate the output of the Actor. The `Delivery Agent` is the most dependent, using `load_feature_list()`, `load_code_metadata()`, and `load_requirements()` to cross-reference the implemented code against the original requirements and feature list, ensuring full coverage before declaring delivery complete.

### Interaction with Tooling & Operations

The Tooling & Operations domain provides the interface between agents and the Storage Layer. Tools like `SavePrdDocTool`, `SaveIdeaTool`, and `SaveChangeRequestTool` are thin wrappers around the Storage Layer's functions. For example, `SavePrdDocTool::execute()` calls `storage::save_prd_doc(session_id, content)`. Similarly, `LoadIdeaTool` calls `storage::load_idea()`. The `GotoStageTool` uses `load_session_input()` and `load_session_meta()` to reconstruct the session state before resuming execution from a specified stage.

### Interaction with Core Workflow Orchestration

The Pipeline Coordinator uses the Storage Layer to determine the current state of the workflow. Before initiating the `Coding` stage, it calls `has_implementation_plan(session_id)` to verify that a plan exists. If the user invokes `--goto-stage coding`, the `Stage Navigator` uses `load_session_meta()` to find the current stage and `init_session_from_base()` to prepare the environment. The `create_modify_pipeline()` function relies on `load_change_request()` to understand the scope of the requested change.

## Key Design Principles and Technical Implementation

The Data & Artifact Management module embodies several core design principles that define Cowork Forge's unique value proposition:

1. **Simplicity and Transparency**: By using plain text (Markdown) and JSON files, the system avoids opaque databases. A developer can open `.cowork/sessions/s1-20240101-123456/state/feature_list.json` in any editor and understand the project's state. This transparency builds trust and enables debugging without specialized tools.
2. **Session Isolation**: Every workflow execution is a new session. This prevents state corruption and allows for safe experimentation. A failed `modify` session does not affect the original codebase.
3. **Resilient State Recovery**: The `init_session_from_base()` function is the linchpin of resilience. It ensures that even after a system crash or agent failure, the next run can pick up exactly where it left off, preserving all prior approvals and context.
4. **Automated Consistency**: The automatic feature status update logic (`update_feature_status_if_needed()`) removes a significant source of human error. It ensures that the system's understanding of progress is always synchronized with the actual state of tasks.
5. **Schema Versioning**: Every model includes a `schema_version` field. This allows the system to evolve its data models over time while maintaining backward compatibility. A new version of Cowork Forge can read an old `feature_list.json` and upgrade it on the fly.

The technical implementation is robust and production-ready. It uses `anyhow` for comprehensive error handling, ensuring that every file operation failure is wrapped with a meaningful context (e.g., "Failed to write state/requirements.json for session s1-20240101-123456"). The code is thoroughly tested, as evidenced by the presence of `storage_test.rs`. The use of `chrono::Utc` for timestamps ensures consistent time tracking across different environments.

In summary, the Data & Artifact Management module is not a passive storage system; it is an active, intelligent component that enforces structure, ensures consistency, and enables resilience. It transforms Cowork Forge from a simple AI assistant into a reliable, auditable, and recoverable software development platform.