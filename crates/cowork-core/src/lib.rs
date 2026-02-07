// Cowork Forge - Core Library (Iteration Architecture)

// New domain-driven modules
pub mod domain;
pub mod persistence;

// Tech stack configuration
pub mod tech_stack;

// Existing modules (to be gradually refactored)
pub mod llm;
pub mod tools;
pub mod agents;
pub mod pipeline;
pub mod instructions;
pub mod interaction;
pub mod event_bus;

// Legacy modules (will be removed)
pub mod data;
pub mod storage;
pub mod memory;

// Re-exports for convenience
pub use domain::*;
pub use persistence::*;
pub use llm::*;
pub use tech_stack::*;

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
