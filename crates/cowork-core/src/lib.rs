// Cowork Forge - Core Library
// Built with adk-rust 0.2.1

pub mod data;
pub mod storage;
pub mod llm;
pub mod tools;
pub mod agents;
pub mod pipeline;
pub mod instructions;
pub mod interaction;
pub mod event_bus;

// Re-exports for convenience
pub use data::*;
pub use storage::*;
pub use llm::*;

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
