// Cowork Forge - Core Library (Iteration Architecture)

// Global configuration
pub mod config;

// V3 Configuration Definition Layer
pub mod config_definition;

// ACP (Agent Client Protocol) for external coding agent integration
pub mod acp;

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

// Skills ecosystem (agentskills.io standard via adk-skill)
pub mod skills;

// Integration system (V3)
pub mod integration;

// Re-exports for convenience
pub use domain::*;
pub use persistence::*;
pub use data::*;
pub use storage::*;
pub use llm::*;
pub use agents::{create_project_manager_agent, execute_pm_agent_message, execute_pm_agent_message_streaming, PMAgentResult, PMAgentAction, PMAgentStreamCallback};
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

// Re-exports for config
pub use config::{get_system_locale, set_system_locale, get_language_instruction};

// Re-exports for ACP module
pub use acp::{AcpClient, AcpTaskResult};

// Re-exports for config_definition (V3)
pub use config_definition::{
    AgentDefinition, AgentType, ModelConfig, ToolReference, IncludeContentsMode,
    StageDefinition, StageType, HookConfig, HookPoint, ArtifactConfig, StageRetryConfig,
    FlowDefinition, StageReference, FlowConfig, MemoryScope, InheritanceConfig, InheritanceMode,
    IntegrationDefinition, IntegrationType, ConnectionConfig, AuthConfig, IntegrationEvent,
    ConfigRegistry, global_registry, ConfigLoader, LoadReport, ConfigValidator, ValidationResult,
    create_agent_for_stage, create_agent_from_config, initialize_config_registry,
};

// Re-exports for skills (agentskills.io standard via adk-skill)
pub use skills::{
    SkillManager, SkillManagerConfig,
    // Core types
    SkillDocument, SkillIndex, SkillSummary, SkillMatch,
    // Selection
    SelectionPolicy,
    // Injection
    SkillInjector, SkillInjectorConfig,
    apply_skill_injection, select_skill_prompt_block,
    // Loading and parsing
    load_skill_index, parse_skill_markdown, parse_instruction_markdown,
    // Discovery
    discover_skill_files, discover_instruction_files,
    // Errors
    SkillError, SkillResult,
};

// Re-exports for integration (V3)
pub use integration::{
    HookManager, HookExecutionContext, HookExecutionResult,
    IntegrationAdapter, AdapterError, RestAdapter,
};

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");