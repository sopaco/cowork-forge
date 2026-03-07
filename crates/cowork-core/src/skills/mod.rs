// Skills Module - Skill ecosystem for domain-specific agent capabilities
//
// This module provides:
// - Skill discovery and loading
// - Skill validation
// - Skill runtime management
// - Skill context for tool and prompt injection

mod loader;
mod validator;
mod runtime;
mod context;

pub use loader::{SkillLoader, SkillLoadReport};
pub use validator::{SkillValidator, SkillValidationResult};
pub use runtime::{SkillRuntime, SkillRuntimeError, LoadedSkill};
pub use context::{SkillContext, SkillContextBuilder, PromptInjection};

// Re-export from config_definition for convenience
pub use crate::config_definition::{
    SkillDefinition, SkillCategory, SkillTool, SkillPrompt,
    SkillManifest, SkillSource, SkillDependency,
    ToolImplementation, SkillPromptType, PromptInjectionPoint,
};
