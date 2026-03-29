# Domain Logic

## Responsibility
Core business entities following DDD (Domain-Driven Design) patterns.

## Aggregates

### Project (Aggregate Root)
- **Location**: `domain/project.rs`
- **Purpose**: Root entity for a software project
- **Consistency Boundary**: Contains all iterations

```rust
struct Project {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    metadata: ProjectMetadata,
    current_iteration_id: Option<String>,
    iterations: Vec<IterationSummary>,
}

struct ProjectMetadata {
    tech_stack: Vec<String>,
    project_type: String,
    language: String,
}
```

### ProjectMemory (Aggregate)
- **Location**: `domain/memory.rs`
- **Purpose**: Cross-iteration knowledge retention
- **Scope**: Project-level decisions, patterns, issues

```rust
struct ProjectMemory {
    decisions: Vec<Decision>,
    patterns: Vec<Pattern>,
    issues: Vec<Issue>,
    learnings: Vec<Learning>,
}
```

## Entities

### Iteration
- **Location**: `domain/iteration.rs`
- **Purpose**: Single development cycle
- **Lifecycle**: Draft → Running → Paused/Completed/Failed

```rust
struct Iteration {
    id: String,                    // iter-{number}-{timestamp}
    number: u32,
    title: String,
    description: String,
    base_iteration_id: Option<String>,  // For evolution
    inheritance: InheritanceMode,
    status: IterationStatus,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    current_stage: Option<String>,
    completed_stages: Vec<String>,
    artifacts: Artifacts,
}

enum InheritanceMode {
    None,     // Genesis: fresh start
    Full,     // Copy artifacts + workspace
    Partial,  // Copy artifacts only
}

enum IterationStatus {
    Draft, Running, Paused, Completed, Failed,
}
```

### IterationKnowledge
- **Location**: `domain/memory.rs`
- **Purpose**: Knowledge extracted from single iteration

```rust
struct IterationKnowledge {
    iteration_id: String,
    decisions: Vec<Decision>,
    patterns: Vec<Pattern>,
    issues: Vec<Issue>,
    learnings: Vec<Learning>,
    tech_stack: Vec<String>,
    summary: Option<String>,
}
```

## Value Objects

### Artifacts
```rust
struct Artifacts {
    idea: Option<String>,      // Path to idea.md
    prd: Option<String>,       // Path to prd.md
    design: Option<String>,    // Path to design.md
    plan: Option<String>,      // Path to plan.md
    coding: Option<String>,    // Workspace path
    delivery: Option<String>,  // Path to delivery_report.md
}
```

### Knowledge Types
```rust
struct Decision {
    title: String,
    decision: String,
    rationale: String,
    alternatives: Vec<String>,
}

struct Pattern {
    name: String,
    description: String,
    usage: String,
}

struct Issue {
    title: String,
    description: String,
    resolution: Option<String>,
}

struct Learning {
    topic: String,
    insight: String,
    application: String,
}
```

## Entity Behaviors

### Iteration Lifecycle Methods
```rust
impl Iteration {
    fn create_genesis(project: &Project, title: String, description: String) -> Self;
    fn create_evolution(project: &Project, title: String, description: String, base_iteration_id: String, inheritance: InheritanceMode) -> Self;
    fn start(&mut self);
    fn pause(&mut self);
    fn resume(&mut self);
    fn complete(&mut self);
    fn fail(&mut self);
    fn set_stage(&mut self, stage: impl Into<String>);
    fn complete_stage(&mut self, stage: impl Into<String>, artifact_path: Option<String>);
    fn determine_start_stage(&self) -> String;
    fn to_summary(&self) -> IterationSummary;
}
```

### Project Methods
```rust
impl Project {
    fn new(name: impl Into<String>) -> Self;
    fn add_iteration(&mut self, summary: IterationSummary);
    fn set_current_iteration(&mut self, iteration_id: String);
    fn get_latest_completed_iteration(&self) -> Option<&IterationSummary>;
    fn next_iteration_number(&self) -> u32;
}
```

## Domain Invariants

1. **Project Consistency**: Iteration numbers are sequential and unique within a project
2. **Iteration Status**: Only Paused iterations can be resumed; only Failed iterations can be retried
3. **Stage Progression**: Stages are completed in order; current_stage indicates next to execute
4. **Artifact Integrity**: Each artifact path corresponds to exactly one stage
5. **Knowledge Promotion**: Insights can be promoted to Decisions; Learnings to Patterns

## Code Locations

```
crates/cowork-core/src/domain/
├── mod.rs         # Re-exports
├── project.rs     # Project, ProjectMetadata, IterationSummary, IterationStatus
├── iteration.rs   # Iteration, Artifacts, InheritanceMode
└── memory.rs      # ProjectMemory, IterationKnowledge, Decision, Pattern, Issue, Learning
```
