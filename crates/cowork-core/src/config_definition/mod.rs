// Configuration Definition Module for V3
// 
// This module provides data-driven configuration for Agents, Stages, Flows,
// Skills, and Integrations. It enables the transition from hardcoded definitions
// to configurable, extensible system architecture.

pub mod agent_definition;
pub mod stage_definition;
pub mod flow_definition;
pub mod skill_definition;
pub mod integration_definition;
pub mod registry;
pub mod loader;
pub mod validator;
pub mod builtin;
pub mod agent_factory;

pub use agent_definition::*;
pub use stage_definition::*;
pub use flow_definition::*;
pub use skill_definition::*;
pub use integration_definition::*;
pub use registry::*;
pub use loader::*;
pub use validator::*;
pub use builtin::load_builtin_configs;
pub use agent_factory::{create_agent_for_stage, create_agent_from_config, initialize_config_registry};
