// Cowork Forge - Core Library (Iteration Architecture)

// New domain-driven modules
pub mod domain;
pub mod persistence;

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

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
