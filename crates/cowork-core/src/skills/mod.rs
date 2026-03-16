// Skills Module - Agent Skills ecosystem using adk-skill (agentskills.io standard)
//
// This module provides skill management based on the agentskills.io open standard.
// Skills are defined as SKILL.md files with YAML frontmatter and Markdown body.
//
// Key features:
// - Progressive context loading (metadata -> instructions -> resources)
// - Weighted lexical matching for skill selection
// - Convention file support (AGENTS.md, CLAUDE.md, SOUL.md, etc.)
// - Skill injection into user content
//
// Usage:
// ```rust
// use cowork_core::skills::{SkillManager, SelectionPolicy, load_skill_index, select_skills};
//
// // Load skill index
// let index = load_skill_index("./project-root")?;
//
// // Select best matching skill for a query
// let policy = SelectionPolicy::default();
// let matches = select_skills(&index, "search the codebase", &policy);
// for m in matches {
//     println!("{} ({:.2}): {}", m.skill.name, m.score, m.skill.description);
// }
// ```

mod manager;

pub use manager::{SkillManager, SkillManagerConfig};

// Re-export key types from adk-skill for convenience
pub use adk_skill::{
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
