# Persistence Domain

## Responsibility
JSON-based storage for projects, iterations, and memory with workspace isolation.

## Storage Structure

```
<project-root>/.cowork-v2/
├── project.json              # Project metadata
├── memory.json               # ProjectMemory (cross-iteration)
└── iterations/
    └── iter-<number>-<timestamp>/
        ├── iteration.json    # Iteration metadata
        ├── memory.json       # IterationKnowledge
        ├── artifacts/
        │   ├── idea.md
        │   ├── prd.md
        │   ├── design.md
        │   ├── plan.md
        │   └── delivery_report.md
        ├── workspace/        # Generated code
        └── summaries/        # Document summaries
            ├── idea_summary.txt
            └── ...
```

## Store Implementations

### ProjectStore
- **Location**: `persistence/project_store.rs`
- **Purpose**: Manage project.json

```rust
impl ProjectStore {
    fn new() -> Self;
    fn save(&self, project: &Project) -> Result<()>;
    fn load(&self) -> Result<Option<Project>>;
    fn add_iteration(&self, project: &mut Project, summary: IterationSummary) -> Result<()>;
}
```

### IterationStore
- **Location**: `persistence/iteration_store.rs`
- **Purpose**: Manage iteration data

```rust
impl IterationStore {
    fn new() -> Self;
    fn save(&self, iteration: &Iteration) -> Result<()>;
    fn load(&self, id: &str) -> Result<Iteration>;
    fn list(&self) -> Result<Vec<IterationSummary>>;
    fn delete(&self, id: &str) -> Result<()>;
    fn iteration_path(&self, id: &str) -> Result<PathBuf>;
    fn workspace_path(&self, id: &str) -> Result<PathBuf>;
}
```

### MemoryStore
- **Location**: `persistence/memory_store.rs`
- **Purpose**: Manage knowledge storage

```rust
impl MemoryStore {
    fn new() -> Self;
    fn load_project_memory(&self) -> Result<ProjectMemory>;
    fn save_project_memory(&self, memory: &ProjectMemory) -> Result<()>;
    fn ensure_iteration_memory(&self, iteration_id: &str) -> Result<()>;
    fn load_iteration_knowledge(&self, iteration_id: &str) -> Result<IterationKnowledge>;
    fn save_iteration_knowledge(&self, knowledge: &IterationKnowledge) -> Result<()>;
    fn promote_insights_to_decisions(&self, iteration_id: &str) -> Result<()>;
}
```

## Feedback History

```rust
// Location: persistence/mod.rs
struct FeedbackHistory {
    feedbacks: Vec<FeedbackEntry>,
}

struct FeedbackEntry {
    stage: String,
    details: String,
    timestamp: DateTime<Utc>,
}

fn save_feedback_history(history: &FeedbackHistory) -> Result<()>;
fn load_feedback_history() -> Result<FeedbackHistory>;
fn clear_stage_feedback(stage: &str) -> Result<()>;
```

## Global State

```rust
// Current iteration ID for tools
fn set_iteration_id(id: String);
fn get_iteration_id() -> Option<String>;
```

## Workspace Preparation

Evolution iterations copy from base:

```rust
// Location: pipeline/executor/workspace.rs
async fn prepare_workspace(
    iteration_store: &IterationStore,
    interaction: &Arc<dyn InteractiveBackend>,
    iteration: &Iteration,
) -> Result<PathBuf>;

// Inheritance modes:
// - None: Create empty workspace
// - Partial: Copy artifacts only
// - Full: Copy artifacts + workspace
```

## Code Locations

```
crates/cowork-core/src/persistence/
├── mod.rs              # Re-exports, feedback history, global state
├── project_store.rs    # ProjectStore
├── iteration_store.rs  # IterationStore
├── memory_store.rs     # MemoryStore
└── iteration_data.rs   # Iteration-specific data utilities
```
