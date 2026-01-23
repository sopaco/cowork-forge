use super::check_agent_verification::add_issue;
use super::check_agent_verification::push_command_check;
use crate::artifacts::{CheckResult, CodeChange, Issue, Phase};
use crate::verification;
use crate::verification::detector;
use crate::verification::error_extract;
use crate::verification::runner;

/// Run verification commands and convert failures into Issues.
///
/// Key behavior:
/// 1. Prefer CodePlan.cmds (from LLM) if present.
/// 2. Otherwise, fall back to default commands based on deterministic project detection.
/// 3. For Node projects, avoid `npm run start` (long-running). We validate `start` script existence,
///    and run `npm run build` / `npm test` if scripts exist.
pub async fn run_verification_commands(
    code_change: &CodeChange,
    checks: &mut Vec<CheckResult>,
    issues: &mut Vec<Issue>,
) {
    let root = code_change.project.root.as_str();
    let kind = detector::detect_project_kind(root);

    // Build command list
    let mut cmds: Vec<verification::VerificationCommand> = if !code_change.cmds.is_empty() {
        verification::commands_from_code_plan_cmds(&code_change.cmds)
    } else {
        verification::default_commands_for_kind(kind)
    };

    // Node special-case: prefer safe commands (no long-running start)
    if kind == verification::ProjectKind::Node {
        // Keep only build/test/lint/check phases; drop run phase by default.
        cmds.retain(|c| c.phase != Phase::Run);

        // If we have package.json, ensure scripts exist.
        let pkg_path = std::path::Path::new(root).join("package.json");
        if pkg_path.exists() {
            let missing = crate::agents::command_validator::validate_node_scripts(
                pkg_path.to_string_lossy().as_ref(),
                &["start"],
            );

            if let Ok(missing) = missing {
                if !missing.is_empty() {
                    add_issue(
                        issues,
                        "ISSUE-NODE-MISSING-SCRIPT-start".to_string(),
                        "error",
                        "package.json is missing required scripts".to_string(),
                        format!("Add scripts: {:?}", missing),
                    );
                    push_command_check(
                        checks,
                        "NODE-SCRIPTS".to_string(),
                        Phase::Check,
                        "validate package.json scripts".to_string(),
                        "failed",
                        vec![format!("Missing scripts: {:?}", missing)],
                    );
                    // Don't run further commands if scripts structure is already broken.
                    return;
                }
            }
        }
    }

    if cmds.is_empty() {
        return;
    }

    let results = runner::run_commands(root, &cmds);

    for (idx, r) in results.iter().enumerate() {
        let check_id = format!("VERIFY-{:?}-{}", r.cmd.phase, idx);
        let status = if r.passed { "passed" } else { "failed" };

        let mut notes = Vec::new();
        if !r.output.stdout.trim().is_empty() {
            notes.push(format!("stdout:\n{}", truncate(&r.output.stdout, 4000)));
        }
        if !r.output.stderr.trim().is_empty() {
            notes.push(format!("stderr:\n{}", truncate(&r.output.stderr, 4000)));
        }
        notes.push(format!("exit_code={}", r.output.status_code));
        notes.push(format!("expect={}", r.cmd.expect));

        push_command_check(
            checks,
            check_id,
            r.cmd.phase,
            r.cmd.cmd.clone(),
            status,
            notes,
        );

        if !r.passed {
            if r.cmd.optional {
                // Optional commands record as warning.
                add_issue(
                    issues,
                    format!("ISSUE-VERIFY-OPTIONAL-{}", idx),
                    "warning",
                    format!("Optional verification failed: {}", r.cmd.cmd),
                    truncate(&r.output.stderr, 2000),
                );
                continue;
            }

            // Hard failure: try to extract affected file hints.
            let mut text = String::new();
            text.push_str(&r.output.stdout);
            text.push_str("\n");
            text.push_str(&r.output.stderr);
            let paths = error_extract::extract_paths(&text);

            let hint = if paths.is_empty() {
                truncate(&text, 2000)
            } else {
                format!(
                    "Affected files: {:?}\n\n{}",
                    paths,
                    truncate(&text, 1500)
                )
            };

            add_issue(
                issues,
                format!("ISSUE-VERIFY-{}", idx),
                "error",
                format!("Verification failed: {}", r.cmd.cmd),
                hint,
            );
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut out = s.chars().take(max).collect::<String>();
    out.push_str("\n...(truncated)...");
    out
}
