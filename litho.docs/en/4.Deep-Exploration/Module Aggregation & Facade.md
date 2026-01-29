# Module Aggregation & Facade in Cowork Forge

## Overview

The **Module Aggregation & Facade** pattern in Cowork Forge serves as the architectural cornerstone for the `cowork-core` library, providing a clean, unified, and simplified public interface to the entire system. This module does not contain business logic itself but acts as the central gateway through which external consumers—most notably the `cowork-cli` application—access all core functionality.

By leveraging Rust’s module system and re-exporting mechanisms, the facade consolidates seven distinct domain modules into a single, coherent import surface. This design dramatically reduces cognitive overhead for users of the library, eliminates deep nesting in import statements, and enforces a stable, versioned API contract that insulates downstream code from internal refactoring.

The implementation is minimal yet profoundly effective: a single `lib.rs` file at the root of `crates/cowork-core` declares public modules and re-exports their public APIs, while also exposing the project version via `CARGO_PKG_VERSION`. This adheres to Rust’s best practices for library design and ensures that Cowork Forge maintains a professional, production-ready API surface.

## Implementation Details

The complete implementation resides in `crates/cowork-core/src/lib.rs`:

```rust
// Cowork Forge - Core Library
// Built with adk-rust 0.2.1

pub mod data;
pub mod storage;
pub mod llm;
pub mod tools;
pub mod agents;
pub mod pipeline;
pub mod instructions;

// Re-exports for convenience
pub use data::*;
pub use storage::*;
pub use llm::*;

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

This file performs two critical functions:

### 1. **Module Declaration**
The `pub mod` declarations expose each core domain as a submodule:
- `data`: Structured models for requirements, features, design, tasks, and session metadata.
- `storage`: File system abstraction for session persistence in `.cowork/sessions/`.
- `llm`: Configuration and rate limiting for external LLM providers (e.g., OpenAI).
- `tools`: All functional tools used by agents (file I/O, validation, HITL, control).
- `agents`: Agent builders and execution wrappers (Actor-Critic, ResilientStream).
- `pipeline`: Orchestration logic for workflow stages (idea → PRD → coding → delivery).
- `instructions`: Prompt templates and workflow rules for every agent role.

### 2. **Public API Re-Exporting**
The `pub use` statements re-export the public items from `data`, `storage`, and `llm` modules directly into the root namespace. This allows external code to import critical types and functions without referencing nested modules:

```rust
// Instead of:
use cowork_core::data::ProjectIdea;
use cowork_core::storage::get_cowork_dir;
use cowork_core::llm::create_llm_client;

// Consumers can write:
use cowork_core::{ProjectIdea, get_cowork_dir, create_llm_client};
```

> **Note**: The `tools`, `agents`, `pipeline`, and `instructions` modules are *not* fully re-exported at the root. This is intentional. These modules contain a large number of tools and agents, and exposing them all at the root would pollute the namespace. Instead, consumers are expected to import from these submodules when needed (e.g., `use cowork_core::tools::WriteFileTool;`), maintaining clarity and preventing API bloat.

### 3. **Version Exposure**
The `VERSION` constant is dynamically injected at compile time via `env!("CARGO_PKG_VERSION")`, ensuring that the library version is always synchronized with the Cargo manifest. This enables:
- Accurate dependency resolution in downstream crates.
- Runtime version checks for compatibility.
- Clear audit trails in logs and reports.

## Integration with Core Domains

The Facade serves as the central hub connecting all five core domains of Cowork Forge:

| Domain | Role | Facade Interaction |
|--------|------|-------------------|
| **Data & Artifact Management** | Defines structured data models (`ProjectIdea`, `PRD`, `ImplementationTask`) and session persistence logic | Re-exported via `pub use data::*;` and `pub use storage::*;` |
| **Tooling & Operations** | Provides 11+ tools for file I/O, validation, HITL, and control | Accessed via `use cowork_core::tools::{WriteFileTool, ReviewWithFeedbackTool, ...};` |
| **Intelligent Agent Control** | Implements actor-critic agents and resilient execution | Accessed via `use cowork_core::agents::{create_idea_agent, create_prd_loop, ...};` |
| **Core Workflow Orchestration** | Orchestrates pipeline stages and state transitions | Accessed via `use cowork_core::pipeline::{create_full_pipeline, create_resume_pipeline, ...};` |
| **Infrastructure Support** | Manages LLM configuration and rate limiting | Re-exported via `pub use llm::*;` |

### Example: Importing Core Functionality

A typical consumer (e.g., `cowork-cli`) imports and uses the facade as follows:

```rust
use cowork_core::{
    // Re-exported from data and storage
    ProjectIdea, get_cowork_dir, load_session,
    
    // Re-exported from llm
    create_llm_client, LlmConfig,
    
    // Imported from submodules
    tools::{WriteFileTool, ReviewWithFeedbackTool},
    agents::{create_idea_agent, create_prd_loop},
    pipeline::{create_full_pipeline},
    instructions::{IDEA_AGENT_INSTRUCTION, PRD_ACTOR_INSTRUCTION},
};

fn main() {
    let llm = create_llm_client(LlmConfig::load().unwrap());
    let session_id = "session-123";
    
    let idea_agent = create_idea_agent(llm.clone(), session_id).unwrap();
    let prd_loop = create_prd_loop(llm.clone(), session_id).unwrap();
    
    let pipeline = create_full_pipeline(session_id);
    pipeline.execute().unwrap();
}
```

This pattern ensures that:
- **Simplicity**: Developers import from one place (`cowork_core`) without navigating deep module hierarchies.
- **Stability**: The facade’s public API is versioned and stable, even as internal modules evolve.
- **Flexibility**: Submodules remain accessible for granular control when needed.
- **Maintainability**: Changes to internal module structure (e.g., renaming `data/models.rs` to `data/schema.rs`) do not break external code, as long as the re-exports remain intact.

## Design Rationale and Benefits

### Why This Pattern Was Chosen

Cowork Forge’s architecture is inherently complex, with over 70 distinct components across 7 domains. Without a facade, developers would need to remember and import from 7+ submodules for even basic operations:

```rust
// Without Facade — Cumbersome and error-prone
use cowork_core::data::models::ProjectIdea;
use cowork_core::storage::mod::get_cowork_dir;
use cowork_core::llm::config::LlmConfig;
use cowork_core::tools::file_tools::WriteFileTool;
use cowork_core::agents::mod::create_idea_agent;
use cowork_core::pipeline::mod::create_full_pipeline;
use cowork_core::instructions::idea::IDEA_AGENT_INSTRUCTION;
```

The facade eliminates this cognitive burden, aligning with Rust’s philosophy of “zero-cost abstractions” — providing a clean, ergonomic interface without runtime overhead.

### Key Benefits

1. **Reduced Cognitive Load**: Developers interact with a single, well-known entry point.
2. **Improved Discoverability**: IDE autocompletion and documentation tools surface a unified API surface.
3. **Backward Compatibility**: Internal refactoring (e.g., moving `storage/mod.rs` to `storage/session.rs`) does not break client code.
4. **Version Control**: The `VERSION` constant provides a single source of truth for dependency management.
5. **Modular Encapsulation**: Each domain remains independently testable and maintainable, while the facade ensures seamless integration.

## Practical Usage and Best Practices

### For Library Consumers (e.g., `cowork-cli`)

- **Always import from `cowork_core` first**. Use the facade as your primary entry point.
- **Use full paths for tools and agents** unless you’re using them frequently:
  ```rust
  use cowork_core::tools::WriteFileTool; // Good
  use cowork_core::tools::*; // Avoid — pollutes namespace
  ```
- **Leverage the `VERSION` constant** for compatibility checks:
  ```rust
  if cowork_core::VERSION != "0.3.1" {
      eprintln!("Incompatible cowork-core version detected");
      std::process::exit(1);
  }
  ```

### For Contributors to `cowork-core`

- **Add new public types/functions** to the appropriate module (`data`, `tools`, etc.).
- **Update `lib.rs`** only if you are exposing a new top-level type or function from `data`, `storage`, or `llm`.
- **Never expose internal implementation details** through the facade. Keep the public API minimal and intentional.
- **Document re-exports** in the module’s `README.md` or `lib.rs` comments to guide users.

## Conclusion

The Module Aggregation & Facade in Cowork Forge is a masterclass in practical Rust library design. It transforms a complex, multi-domain system into a simple, intuitive, and professional API. By acting as a single point of access, it ensures that the power of AI-driven software development is accessible not just to the system’s internal agents, but to every developer who uses Cowork Forge — whether they are building the CLI, extending the toolset, or integrating with external systems.

This pattern is not merely a convenience — it is a necessity for maintaining the system’s usability, scalability, and long-term maintainability. It embodies the core philosophy of Cowork Forge: **simplify complexity without sacrificing power**.