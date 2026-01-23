use crate::artifacts::{CheckResult, CodeChange, Issue, Phase};

/// Build CheckResult from a verification command output.
pub fn push_command_check(
    checks: &mut Vec<CheckResult>,
    id: String,
    phase: Phase,
    cmd: String,
    status: &str,
    notes: Vec<String>,
) {
    checks.push(CheckResult {
        id,
        cmd,
        status: status.to_string(),
        out_ref: "".to_string(),
        notes,
        phase,
    });
}

pub fn add_issue(issues: &mut Vec<Issue>, id: String, sev: &str, desc: String, fix_hint: String) {
    issues.push(Issue {
        id,
        sev: sev.to_string(),
        desc,
        fix_hint,
    });
}

pub fn is_node_project(code_change: &CodeChange) -> bool {
    matches!(code_change.target.lang.as_str(), "javascript" | "typescript")
}

pub fn is_rust_project(code_change: &CodeChange) -> bool {
    code_change.target.lang == "rust"
}
