# Project Management Domain Documentation

## Overview

The **Project Management Domain** serves as the foundational anchor for all development activities within Cowork Forge. It encapsulates the core business entity—the `Project`—and its lifecycle management, providing the structural context for iterations, artifacts, and persistent state. As a core business domain, it embodies the Entity pattern from Domain-Driven Design (DDD), encapsulating not only data but also the business behaviors that govern how a software project evolves over time.

This domain does not directly interact with external systems or file storage. Instead, it defines the in-memory representation of a project and exposes pure domain logic through well-defined methods. All persistence operations are delegated to the **Project Persistence** sub-module, ensuring a clean separation of concerns between business logic and infrastructure concerns.

The Project entity is the central unit of work in Cowork Forge. Every development cycle—whether initiated via `cowork init`, `cowork run`, or `cowork modify`—is anchored to a Project instance. It maintains a history of all iterations, tracks the current active iteration, and provides critical metadata such as creation time, technology stack, and project name. This makes it the single source of truth for the system’s state and the primary interface through which higher-level orchestration layers (e.g., Pipeline Executor, CLI, GUI) interact with the project’s lifecycle.

## Domain Structure

The Project Management Domain is composed of two tightly coupled sub-modules:

### 1. Project Entity

**Location**: `crates/cowork-core/src/domain/project.rs`

The `Project` struct is the core domain model representing a software project. It is designed as a value object with embedded business logic, following the Entity pattern in Domain-Driven Design. The entity is immutable by design—state changes are achieved through method calls that return updated instances, ensuring predictable and testable behavior.

#### Key Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Auto-generated unique identifier in format `proj-{timestamp}` (e.g., `proj-20240615T103000Z`) |
| `name` | `String` | User-provided project name |
| `created_at` | `DateTime<Utc>` | Timestamp of project creation |
| `updated_at` | `DateTime<Utc>` | Timestamp of last state modification |
| `tech_stack` | `Vec<String>` | Optional list of technologies (e.g., `["Rust", "React", "PostgreSQL"]`) |
| `current_iteration_id` | `Option<String>` | Reference to the currently active iteration |
| `iterations` | `Vec<IterationSummary>` | Ordered list of all iterations executed under this project |

The `IterationSummary` is a lightweight, serializable struct containing only essential metadata about an iteration:
- `id`: Unique identifier
- `number`: Sequential integer (1, 2, 3, ...)
- `status`: `IterationStatus` enum (`Draft`, `Running`, `Paused`, `Completed`, `Failed`)
- `started_at`, `completed_at`: Optional timestamps

#### Core Business Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new(name: &str, tech_stack: Vec<String>) -> Self` | Creates a new Project with auto-generated ID, current timestamp, and empty iteration list. |
| `add_iteration` | `fn add_iteration(&mut self, summary: IterationSummary)` | Appends a new iteration to the history and updates `updated_at`. Ensures iteration numbering is sequential. |
| `set_current_iteration` | `fn set_current_iteration(&mut self, iteration_id: &str)` | Updates the active iteration reference and `updated_at`. Validates that the iteration exists in the history. |
| `get_latest_completed_iteration` | `fn get_latest_completed_iteration(&self) -> Option<&IterationSummary>` | Filters iterations by `status == Completed` and returns the one with the highest `number`. Uses `max_by_key` for efficient lookup. |
| `next_iteration_number` | `fn next_iteration_number(&self) -> u32` | Returns `iterations.len() + 1`, ensuring sequential numbering even if iterations are deleted or failed. |

#### State Management and Immutability

All state mutations are performed in-place on the `Project` struct, with `updated_at` automatically refreshed on every state-changing operation. This ensures traceability and auditability. The `Serialize` and `Deserialize` traits (via `serde`) are implemented to enable seamless JSON persistence via the `ProjectStore`.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tech_stack: Vec<String>,
    pub current_iteration_id: Option<String>,
    pub iterations: Vec<IterationSummary>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum IterationStatus {
    Draft,
    Running,
    Paused,
    Completed,
    Failed,
}
```

#### Design Rationale

- **Auto-generated IDs**: Eliminates manual ID management and ensures uniqueness across distributed or concurrent operations.
- **Timestamp Tracking**: Enables audit trails, conflict detection, and chronological ordering of events.
- **Separation of Summary vs. Full Iteration**: The `Project` holds only `IterationSummary` to avoid loading full iteration artifacts (e.g., PRD.md, code files) during high-level operations, improving performance and memory efficiency.
- **Pure Domain Logic**: No file I/O, no HTTP calls, no external dependencies. This makes the entity highly testable and reusable across interfaces.

### 2. Project Persistence

**Location**: `crates/cowork-core/src/persistence/project_store.rs`

The `ProjectStore` implements the Repository pattern, providing a clean abstraction over the physical storage layer. It is responsible for serializing and deserializing `Project` entities to and from JSON files within the `.cowork` directory structure.

#### Core Operations

| Method | Signature | Description |
|--------|-----------|-------------|
| `create` | `fn create(project: &Project) -> Result<(), StorageError>` | Writes the project to `<project_root>/.cowork/project.json`. Creates the `.cowork` directory if it does not exist. |
| `load` | `fn load(project_root: &Path) -> Result<Project, StorageError>` | Reads and deserializes `project.json` from the project root. Returns `Err` if file is missing or malformed. |
| `save` | `fn save(project: &Project) -> Result<(), StorageError>` | Overwrites `project.json` with the current state of the project. |
| `exists` | `fn exists(project_root: &Path) -> bool` | Checks whether a valid `project.json` exists at the given path. |

#### Storage Path Resolution

The `ProjectStore` relies on the **Storage Domain** for path resolution:

```rust
use crate::storage::cowork_dir;

pub fn project_json_path(project_root: &Path) -> PathBuf {
    cowork_dir(project_root).join("project.json")
}
```

This ensures consistency with the system-wide `.cowork` directory structure and prevents hard-coded paths.

#### Error Handling

All operations return a `Result<Project, StorageError>`, where `StorageError` is a custom enum covering:

- `NotFound`: `project.json` does not exist
- `IOError`: File system access failure
- `DeserializeError`: Malformed or incompatible JSON
- `PermissionError`: Insufficient write permissions

This allows higher layers to handle failures gracefully (e.g., prompting user to re-init a project).

#### Transactional Safety

While not implementing full ACID transactions, the `ProjectStore` ensures atomic writes by:
1. Writing to a temporary file (`project.json.tmp`)
2. Performing a filesystem atomic rename operation
3. Only removing the temporary file on success

This prevents corruption in case of system crashes during write operations.

## Interactions with Other Domains

The Project Management Domain interacts with other system components through well-defined interfaces, maintaining architectural boundaries.

### 1. Interaction with Pipeline Domain

The **Pipeline Domain** is the primary consumer of the Project entity. The `IterationExecutor` loads the current project using `ProjectStore::load()` to determine:
- Which iteration to continue (via `current_iteration_id`)
- Whether to inherit artifacts from the previous iteration
- The project’s tech stack for LLM prompt customization

```rust
let project = ProjectStore::load(&project_root)?;
let current_iteration = project.current_iteration_id()
    .and_then(|id| IterationStore::load(&project_root, &id).ok());
```

The `add_iteration` and `set_current_iteration` methods are called by the executor after a successful `DeliveryStage` to update the project’s state.

### 2. Interaction with Storage Domain

The Project Management Domain has a **data dependency** on the Storage Domain. While the `Project` entity itself is storage-agnostic, the `ProjectStore` relies on `storage::cowork_dir()` and `storage::artifact_dir()` to resolve the correct paths for persistence.

This dependency is managed through dependency injection and abstraction, ensuring that the domain logic remains decoupled from the physical file system layout.

### 3. Interaction with Tool Support Domain

The `Project` entity is indirectly accessed by tools such as `SaveChangeRequestTool` and `GotoStageTool` when they need to validate or update project state. For example:
- `SaveChangeRequestTool` may call `Project::next_iteration_number()` to assign a new iteration number for a modification session.
- `GotoStageTool` may call `Project::get_latest_completed_iteration()` to determine the last known good state before restarting a stage.

These interactions occur via the `ProjectStore`, not directly on the entity, preserving encapsulation.

### 4. Interaction with Interaction Domain

The `InteractiveBackend` (CLI/GUI) displays project metadata (name, current iteration, iteration count) to users. This data is retrieved by calling `ProjectStore::load()` and then accessing the `Project`’s public fields.

No direct communication exists between the `Project` entity and the UI layer—this ensures that the UI can be replaced or extended without modifying domain logic.

## Key Workflows Involving Project Management

### 1. Project Initialization Process

1. **User triggers** `cowork init` or clicks “New Project” in GUI.
2. **CLI/GUI** calls `Project::new("MyApp", vec!["Rust".to_string()])`.
3. **Project Entity** generates `proj-20240615T103000Z`, sets timestamps, initializes empty `iterations`.
4. **ProjectStore::create(project)** writes `project.json` to `.cowork/`.
5. **Pipeline Domain** executes `IdeaStage` to generate `idea.md`.
6. **Persistence Domain** saves `idea.md` to `.cowork/sessions/<id>/artifacts/`.

### 2. Development Iteration Process

1. **Pipeline Executor** loads project via `ProjectStore::load()`.
2. **Executor** retrieves `current_iteration_id` to determine continuation point.
3. After successful `DeliveryStage`:
   - A new `IterationSummary` is created with `status = Completed`.
   - `Project::add_iteration(summary)` is called.
   - `Project::set_current_iteration(new_id)` is called.
   - `ProjectStore::save(project)` persists the updated state.
4. **Event Bus** publishes `EngineEvent::iteration_completed` for UI update.

### 3. Change Request Analysis Process

1. User triggers `cowork modify "Add user login"`.
2. **Change Triage Agent** analyzes scope and determines impact.
3. **SaveChangeRequestTool** creates a new session and calls `Project::next_iteration_number()` to assign a new iteration number.
4. **ProjectStore::load()** is called again to ensure state consistency before creating the new session.
5. A new session is initialized from the base project, inheriting all artifacts.

## Design Principles and Best Practices

### 1. Single Responsibility Principle

The `Project` entity has one clear responsibility: managing the lifecycle and state of a software project. It does not handle file I/O, LLM interactions, or UI rendering.

### 2. Dependency Inversion

The `Project` entity depends on no external modules. The `ProjectStore` depends on the `Project` entity, not the other way around. This enables easy mocking in tests.

### 3. Immutability and Predictability

State changes are explicit and traceable. Every method that modifies state updates `updated_at`, making it trivial to audit changes or detect conflicts.

### 4. Serialization-First Design

The `Project` struct is designed for JSON serialization from the outset. All fields are public and tagged with `#[serde(skip_serializing_if = "Option::is_none")]` where appropriate, ensuring clean, minimal JSON output.

### 5. Performance Optimization

By storing only `IterationSummary` in the `Project` entity (not full `Iteration` objects), the system avoids loading megabytes of artifact data during routine operations like listing projects or checking status.

## Testing Strategy

The Project Management Domain is extensively tested with unit and integration tests:

### Unit Tests (Project Entity)

```rust
#[test]
fn test_project_new_generates_unique_id() {
    let p1 = Project::new("Test1", vec![]);
    let p2 = Project::new("Test2", vec![]);
    assert_ne!(p1.id, p2.id);
    assert!(p1.id.starts_with("proj-"));
}

#[test]
fn test_add_iteration_increments_number() {
    let mut p = Project::new("Test", vec![]);
    p.add_iteration(IterationSummary::new("iter-1", 1, IterationStatus::Completed));
    p.add_iteration(IterationSummary::new("iter-2", 2, IterationStatus::Completed));
    assert_eq!(p.iterations.len(), 2);
    assert_eq!(p.iterations[0].number, 1);
    assert_eq!(p.iterations[1].number, 2);
}

#[test]
fn test_get_latest_completed_iteration() {
    let mut p = Project::new("Test", vec![]);
    p.add_iteration(IterationSummary::new("iter-1", 1, IterationStatus::Failed));
    p.add_iteration(IterationSummary::new("iter-2", 2, IterationStatus::Completed));
    p.add_iteration(IterationSummary::new("iter-3", 3, IterationStatus::Running));
    let latest = p.get_latest_completed_iteration().unwrap();
    assert_eq!(latest.number, 2);
}
```

### Integration Tests (ProjectStore)

```rust
#[test]
fn test_project_store_save_and_load() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let project = Project::new("TestProject", vec!["Rust".to_string()]);
    ProjectStore::create(&project)?;
    
    let loaded = ProjectStore::load(temp_dir.path())?;
    assert_eq!(loaded.name, "TestProject");
    assert_eq!(loaded.iterations.len(), 0);
    
    Ok(())
}
```

## Evolution and Future Considerations

### Potential Enhancements

| Enhancement | Rationale |
|-----------|-----------|
| **Versioned Project Schema** | Introduce `schema_version` field to support backward-compatible evolution of the Project structure (e.g., adding new metadata fields). |
| **Project Locking** | Implement advisory file locking to prevent concurrent modifications from multiple CLI/GUI instances. |
| **Project Templates** | Allow users to define and select project templates (e.g., “Web API”, “CLI Tool”) that pre-populate `tech_stack` and initial `idea.md`. |
| **Project Metadata Index** | Build a lightweight index of project metadata (name, tech stack, last modified) for faster project listing in GUI. |

### Architectural Boundaries

The Project Management Domain will remain intentionally minimal. Future features such as team collaboration, access control, or cloud sync will be implemented as **external services** that interact with Cowork Forge via the `ProjectStore` interface, not by extending the domain model.

## Conclusion

The Project Management Domain is the cornerstone of Cowork Forge’s architecture. By cleanly separating the **Project Entity** (business logic) from the **Project Persistence** (infrastructure), the system achieves a robust, testable, and scalable foundation for AI-driven software development.

Its design principles—pure domain logic, serialization-first, state immutability, and strict separation of concerns—ensure that the core of the system remains stable, predictable, and maintainable as new features and AI agents are added. The domain’s simplicity belies its critical importance: every iteration, every line of code, and every decision made within Cowork Forge begins and ends with the Project entity.