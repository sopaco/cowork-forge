// Cowork Forge - Core Library (Iteration Architecture)

// Domain-driven modules
pub mod domain;
pub mod persistence;

// Tech stack configuration
pub mod tech_stack;

// Core modules
pub mod llm;
pub mod tools;
pub mod agents;
pub mod pipeline;
pub mod instructions;
pub mod interaction;

// Re-exports for convenience
pub use domain::*;
pub use persistence::*;
pub use llm::*;
pub use tech_stack::*;

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
