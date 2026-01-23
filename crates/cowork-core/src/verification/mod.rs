//! Cross-language verification layer
//!
//! Why:
//! - Previously, Cowork could generate a large amount of code, but only performed
//!   shallow checks (file existence, basic compilation for Rust, etc.).
//! - For complex projects (especially Node/Web), this led to situations where
//!   `npm start` fails, but Cowork still considers the result "passed".
//!
//! What:
//! - Provide deterministic, cross-language command execution and result capture.
//! - Feed failing command output back into the targeted-fix loop.

use crate::artifacts::{Command, Phase};

pub mod detector;
pub mod runner;
pub mod error_extract;
pub mod safety;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    Rust,
    Node,
    Python,
    Html,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct VerificationCommand {
    pub phase: Phase,
    pub cmd: String,
    pub expect: String,
    /// If optional, failure is recorded but not treated as a hard error.
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub cmd: VerificationCommand,
    pub output: CommandOutput,
    pub passed: bool,
}

pub fn commands_from_code_plan_cmds(cmds: &[Command]) -> Vec<VerificationCommand> {
    cmds.iter()
        .map(|c| VerificationCommand {
            phase: c.phase,
            cmd: c.cmd.clone(),
            expect: c.expect.clone(),
            optional: false,
        })
        .collect()
}

pub fn default_commands_for_kind(kind: ProjectKind) -> Vec<VerificationCommand> {
    match kind {
        ProjectKind::Rust => vec![
            VerificationCommand {
                phase: Phase::Check,
                cmd: "cargo check".to_string(),
                expect: "compiles".to_string(),
                optional: false,
            },
            VerificationCommand {
                phase: Phase::Test,
                cmd: "cargo test".to_string(),
                expect: "tests pass".to_string(),
                optional: true,
            },
        ],
        ProjectKind::Node => vec![
            VerificationCommand {
                phase: Phase::Build,
                cmd: "npm run build".to_string(),
                expect: "build succeeds".to_string(),
                optional: true,
            },
            VerificationCommand {
                phase: Phase::Test,
                cmd: "npm test".to_string(),
                expect: "tests pass".to_string(),
                optional: true,
            },
        ],
        ProjectKind::Python => vec![VerificationCommand {
            phase: Phase::Check,
            cmd: "python3 -m py_compile $(find . -name '*.py' -maxdepth 6 | head -n 200)".to_string(),
            expect: "python syntax ok".to_string(),
            optional: false,
        }],
        ProjectKind::Html => vec![],
        ProjectKind::Unknown => vec![],
    }
}
