// Cowork Forge - Core Library (Iteration Architecture)

// Domain-driven modules
pub mod domain;
pub mod persistence;

// Data models and storage
pub mod data;
pub mod storage;

// Tech stack configuration
pub mod tech_stack;

// Project runtime configuration (for GUI Preview/Run)
pub mod project_runtime;
pub mod runtime_security;
pub mod runtime_analyzer;

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
pub use data::*;
pub use storage::*;
pub use llm::*;
pub use tech_stack::*;

// Explicit exports for new modules (to avoid glob re-export conflicts)
pub use project_runtime::{
    ProjectRuntimeConfig, RuntimeType, FrontendFramework, FrontendRuntime,
    BackendFramework, BackendRuntime, FullstackRuntime, DependencyConfig,
    ProxyConfig, PackageManager as RuntimePackageManager, SecurityCheckResult,
    get_preset_config,
};
pub use runtime_security::RuntimeSecurityChecker;
pub use runtime_analyzer::{
    RuntimeAnalyzer, ProjectInfo, save_runtime_config, load_runtime_config, has_runtime_config,
};

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
