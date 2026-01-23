use crate::verification::{CommandOutput, VerificationCommand, VerificationResult};
use crate::verification::safety::{check_command_safety, SafetyCheckResult};
use std::process::Command;

/// Verification command runner with safety checks
///
/// Runs shell-like commands in a best-effort manner.
/// We intentionally use `/bin/sh -lc` so that commands like `npm run build` and
/// `cd subdir && ...` work. This is a pragmatic choice for cross-language projects.
///
/// Security:
/// - All commands go through safety checks before execution
/// - Dangerous patterns (rm -rf /, dd, fork bombs, etc.) are blocked
/// - System critical paths are protected
/// - Suspicious commands are logged but may be rejected

pub fn run_commands(working_dir: &str, commands: &[VerificationCommand]) -> Vec<VerificationResult> {
    commands
        .iter()
        .map(|cmd| {
            // Safety check before execution
            match check_command_safety(&cmd.cmd, working_dir) {
                SafetyCheckResult::Safe => {
                    let output = run_one(working_dir, cmd);
                    let passed = output.status_code == 0;
                    VerificationResult {
                        cmd: cmd.clone(),
                        output,
                        passed,
                    }
                }
                SafetyCheckResult::Blocked(reason) => {
                    tracing::error!("🚫 Command blocked for safety: {} - Reason: {}", cmd.cmd, reason);
                    VerificationResult {
                        cmd: cmd.clone(),
                        output: CommandOutput {
                            status_code: -2,  // Special code for safety rejection
                            stdout: String::new(),
                            stderr: format!("SAFETY CHECK FAILED: {}\nCommand was blocked and not executed.", reason),
                        },
                        passed: false,
                    }
                }
                SafetyCheckResult::Suspicious(reason) => {
                    tracing::warn!("⚠️  Suspicious command detected: {} - Reason: {}", cmd.cmd, reason);
                    // For now, we log and execute, but you can make this stricter
                    // by returning a blocked result instead
                    let output = run_one(working_dir, cmd);
                    let passed = output.status_code == 0;
                    VerificationResult {
                        cmd: cmd.clone(),
                        output,
                        passed,
                    }
                }
            }
        })
        .collect()
}

fn run_one(working_dir: &str, cmd: &VerificationCommand) -> CommandOutput {
    // Use sh -lc for portability.
    let output = Command::new("sh")
        .arg("-lc")
        .arg(&cmd.cmd)
        .current_dir(working_dir)
        .output();

    match output {
        Ok(out) => CommandOutput {
            status_code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => CommandOutput {
            status_code: -1,
            stdout: String::new(),
            stderr: format!("Failed to spawn command: {}", e),
        },
    }
}
