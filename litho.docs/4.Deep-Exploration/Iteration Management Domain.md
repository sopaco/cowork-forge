# Iteration Management Domain Documentation

## Overview

The Iteration Management Domain is the core engine of Cowork Forge, responsible for orchestrating individual development cycles through a structured, AI-assisted workflow. Each iteration represents a complete, self-contained software development cycle—from initial concept to final delivery—following a seven-stage pipeline (Idea → PRD → Design → Plan → Coding → Check → Delivery). This domain ensures traceability, consistency, and automation across all development activities, enabling developers to deliver high-quality software with minimal manual intervention.

The domain is implemented as a layered system comprising two primary components: the **Iteration Entity** (domain model) and the **Iteration Persistence Layer** (repository). These components work in concert with the Pipeline Executor to manage lifecycle transitions, inheritance patterns, and artifact tracking, forming the backbone of Cowork Forge’s iterative development paradigm.

## Core Components

### 1. Iteration Entity

The `Iteration` struct is the central domain model representing a single development cycle. It encapsulates all metadata, state, and artifacts associated with an iteration, providing a rich, serializable representation of the development process.

#### Structure and Fields

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iteration {
    pub id: String,                    // Unique identifier (e.g., "iter-123-1770079582")
    pub number: u32,                   // Sequential iteration number within project
    pub title: String,                 // Human-readable title
    pub description: String,           // Natural language description of change scope

    // Inheritance
    pub base_iteration_id: Option<String>, // Reference to parent iteration (for evolution)
    pub inheritance: InheritanceMode,     // Mode of inheritance: None, Full, Partial

    // Execution state
    pub status: IterationStatus,         // Draft, Running, Paused, Completed, Failed
    pub started_at: DateTime<Utc>,       // Timestamp when iteration began
    pub completed_at: Option<DateTime<Utc>>, // Timestamp when iteration completed
    pub current_stage: Option<String>,   // Current stage being executed
    pub completed_stages: Vec<String>,   // Ordered list of completed stages

    // Artifacts
    pub artifacts: Artifacts,            // File paths for generated documents and code
}
```

#### Key Behaviors and Methods

The `Iteration` entity provides a comprehensive set of lifecycle and state management methods:

- **`create_genesis()`**: Initializes a new, independent iteration with no inheritance. Used for project initialization or entirely new feature development.
- **`create_evolution()`**: Creates an iteration based on a previous iteration, supporting inheritance modes (`Full`, `Partial`, or `None`). This enables iterative refinement and change-driven development.
- **`start()`, `pause()`, `resume()`, `complete()`, `fail()`**: Manage the iteration’s execution state, enabling human-in-the-loop control and recovery from failures.
- **`set_stage()`**: Updates the current stage during pipeline execution.
- **`complete_stage()`**: Records completion of a stage and associates its generated artifact (e.g., `prd.md`, `design.md`) with the iteration.
- **`determine_start_stage()`**: The most sophisticated method in the domain, which analyzes the iteration’s `description` field using keyword matching to automatically determine the appropriate starting stage for evolution iterations.

#### Intelligent Stage Determination

The `determine_start_stage()` method implements semantic analysis to reduce manual configuration and accelerate development cycles:

```rust
fn analyze_change_scope(description: &str) -> String {
    let desc_lower = description.to_lowercase();

    // Architecture changes → start at 'idea'
    let arch_keywords = ["架构", "architecture", "重构", "rewrite", "重新设计", "redesign"];
    if arch_keywords.iter().any(|kw| desc_lower.contains(kw)) {
        return "idea".to_string();
    }

    // Requirement changes → start at 'prd'
    let req_keywords = ["需求", "requirement", "功能", "feature", "添加", "add"];
    if req_keywords.iter().any(|kw| desc_lower.contains(kw)) {
        return "prd".to_string();
    }

    // Design changes → start at 'design'
    let design_keywords = ["设计", "design", "数据库", "database", "接口", "api"];
    if design_keywords.iter().any(|kw| desc_lower.contains(kw)) {
        return "design".to_string();
    }

    // Default: code changes → start at 'plan'
    "plan".to_string()
}
```

This intelligent auto-detection enables users to initiate iterations with natural language descriptions like:
- _“Add user authentication with OAuth2”_ → starts at `prd`
- _“Refactor database schema to support multi-tenancy”_ → starts at `idea`
- _“Update API endpoints to v2”_ → starts at `design`
- _“Fix login bug in auth module”_ → starts at `plan`

This reduces cognitive load and ensures the AI agent begins work at the most relevant stage, improving efficiency and relevance.

#### Artifact Tracking

The `Artifacts` struct tracks the location of all generated documents and code files:

```rust
pub struct Artifacts {
    pub idea: Option<String>,
    pub prd: Option<String>,
    pub design: Option<String>,
    pub plan: Option<String>,
    pub delivery: Option<String>,
}
```

Each artifact is stored as a relative file path within the session workspace (e.g., `.cowork/sessions/iter-123/artifacts/prd.md`). This enables seamless integration with the Tool Support Layer for file access and human-in-the-loop editing.

#### Summary Interface

The `to_summary()` method returns a lightweight `IterationSummary` for UI rendering and listing operations, minimizing serialization overhead:

```rust
pub struct IterationSummary {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub status: IterationStatus,
    pub completed_stages: Vec<String>,
    pub created_at: DateTime<Utc>,
}
```

This separation of concerns ensures that UI components can efficiently display iteration lists without loading full artifact content.

### 2. Iteration Persistence Layer

The `IterationStore` implements the Repository pattern, providing a clean abstraction between the domain model and physical storage.

#### Key Operations

| Method | Purpose |
|--------|---------|
| `load(iteration_id)` | Loads an iteration from JSON file (`iterations/iter-xxx.json`) |
| `save(iteration)` | Persists iteration state to disk with pretty-printed JSON |
| `exists(iteration_id)` | Checks if an iteration file exists |
| `delete(iteration_id)` | Removes iteration file and associated artifacts |
| `load_all()` | Retrieves all iterations, sorted by number |
| `load_summaries()` | Returns lightweight summaries for UI listing |
| `workspace_path()` | Returns path to iteration’s workspace directory |
| `ensure_workspace()` | Ensures workspace directory exists and creates it if needed |
| `iteration_path()` | Returns path to iteration’s artifact directory |

#### Storage Structure

All iteration data is persisted in a hierarchical, session-isolated structure:

```
.cowork/
├── sessions/
│   └── iter-123/                 # Session directory
│       ├── artifacts/            # Generated documents (prd.md, design.md, etc.)
│       └── state/                # Temporary state files
├── iterations/
│   ├── iter-123.json             # Iteration metadata (serialized Iteration struct)
│   └── iter-124.json
└── workspace/                    # Shared workspace for active iteration
```

This design ensures:
- **Isolation**: Each iteration has its own workspace and artifacts.
- **Traceability**: All changes are versioned and linked to a specific iteration.
- **Reproducibility**: Any iteration can be reloaded and re-executed from its persisted state.

#### Workspace Inheritance Logic

The `prepare_workspace()` method in the Pipeline Executor leverages `IterationStore` to implement inheritance:

- **Full Inheritance**: Copies all files from the base iteration’s workspace (used for major refactors).
- **Partial Inheritance**: Copies only non-code assets (config files, READMEs, documentation) and artifacts, forcing code regeneration (used for feature additions).
- **None**: Starts with an empty workspace (used for genesis iterations).

This logic is implemented in `inherit_from_base()` and ensures that evolution iterations inherit only what is necessary, avoiding unnecessary code duplication while preserving context.

## Integration with Other Domains

### 1. Project Management Domain

The Iteration Management Domain is tightly coupled with the Project Management Domain:

- **Ownership**: A `Project` contains a collection of `Iteration` entities and references the current iteration.
- **ID Generation**: The `Project` generates unique iteration numbers via `next_iteration_number()`, ensuring sequential numbering.
- **Lifecycle Coordination**: When an iteration completes, the `Project` is updated to reflect the new current iteration.

```rust
// In IterationExecutor::create_genesis_iteration
let iteration = Iteration::create_genesis(project, title.into(), description.into());
self.iteration_store.save(&iteration)?;
self.project_store.add_iteration(project, iteration.to_summary())?; // Updates project
```

### 2. Pipeline Domain

The Pipeline Domain is the primary consumer of the Iteration Management Domain:

- **Executor**: The `IterationExecutor` uses `Iteration` to determine the starting stage, prepare the workspace, and track progress.
- **Stage Execution**: Each pipeline stage (PRD, Design, etc.) reads and writes artifacts via the `Iteration.artifacts` field.
- **Feedback Loops**: If a stage fails or requires human review, the iteration’s `status` is updated to `Paused`, and the executor waits for user input.

### 3. Tool Support Domain

The Tool Support Layer interacts with the Iteration Management Domain through:

- **Artifact Tools**: `SavePrdDocTool`, `SaveDesignDocTool`, etc., update the `Iteration.artifacts` field after generating documents.
- **HITL Tools**: `ReviewAndEditFileTool` opens files referenced in `Iteration.artifacts` in external editors (e.g., VSCode).
- **Goto Stage Tool**: Allows agents to jump to a specific stage, updating `current_stage` and clearing subsequent `completed_stages`.

### 4. Storage Domain

The `IterationStore` depends on the Storage Domain for path resolution:

- Uses `get_cowork_dir()` to locate `.cowork/`
- Creates and validates directory structure (`iterations/`, `workspace/`, `artifacts/`)
- Ensures file paths are safe and within project boundaries

## Workflow Integration

### Development Iteration Process

The Iteration Management Domain is central to the core development workflow:

1. **Initiation**: User triggers `cowork run` → `IterationExecutor.create_genesis_iteration()` → creates new `Iteration`
2. **Stage Execution**: Pipeline stages execute sequentially, calling `complete_stage(stage, path)` after each step
3. **HITL Intervention**: At any stage, user can request review → `Iteration.status` becomes `Paused`
4. **Resume**: User approves changes → `Iteration.resume()` → pipeline continues
5. **Completion**: `DeliveryStage` copies code to project root → `Iteration.complete()` → updates `Project.current_iteration`

### Change Request Analysis

When a user initiates a modification (`cowork modify`):

1. A new `Iteration` is created with `base_iteration_id` and `inheritance: Partial`
2. `determine_start_stage()` analyzes the change description to determine the starting stage
3. The pipeline resumes from that stage, reusing only relevant artifacts
4. The original iteration remains unchanged, preserving history

This enables safe, traceable evolution of software without disrupting prior work.

## Technical Implementation Details

### Serialization and Persistence

- **Format**: JSON with `serde_json`
- **Encoding**: UTF-8
- **Structure**: Human-readable, versioned, and diff-friendly
- **Atomicity**: File writes are atomic (via `std::fs::write`), preventing corruption

### Error Handling

- Uses `anyhow::Result` for unified error handling
- All file operations are wrapped in try-catch blocks
- Invalid paths or corrupted JSON files are gracefully handled with informative errors

### Concurrency and Safety

- Iteration files are read/written synchronously (no concurrent writes)
- Workspace isolation prevents cross-iteration contamination
- Path validation prevents directory traversal attacks

### Performance Considerations

- `to_summary()` minimizes data transfer for UI
- `load_summaries()` avoids deserializing full artifacts during listing
- Workspace inheritance uses async file copying (`tokio::fs`) to avoid blocking the main thread

## Best Practices and Usage Guidelines

### For Developers

- **Use descriptive iteration titles**: “Add user profile page” is better than “Feature 1”
- **Be explicit in change descriptions**: Use keywords like “architecture”, “design”, “requirement” to trigger intelligent stage selection
- **Leverage partial inheritance**: For small feature additions, use `Partial` inheritance to avoid regenerating entire codebases
- **Review artifacts before completion**: Use HITL tools to validate generated documents and code

### For System Integrators

- **Extend `analyze_change_scope()`**: Add domain-specific keywords (e.g., “security”, “performance”) to improve stage detection
- **Customize inheritance rules**: Modify `inherit_from_base()` to support custom artifact filtering
- **Integrate with external systems**: Use `IterationStore.load_all()` to expose iteration history to CI/CD or reporting tools

## Conclusion

The Iteration Management Domain is the architectural heart of Cowork Forge, transforming abstract development concepts into executable, traceable, and automated workflows. By combining a rich domain model with intelligent semantic analysis and robust persistence, it enables a seamless human-AI collaboration model that reduces manual overhead while preserving quality and traceability.

Its design exemplifies clean architecture principles:  
- **Encapsulation**: All iteration state is contained within the `Iteration` entity  
- **Separation of Concerns**: Domain logic is isolated from storage and UI  
- **Extensibility**: New inheritance modes, stages, or artifact types can be added without breaking existing code  

This domain ensures that every software change—no matter how small—is treated as a first-class, documented, and repeatable process, making Cowork Forge not just a code generator, but a true AI-powered development partner.