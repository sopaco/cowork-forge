// Path utilities for resolving executable search issues across platforms
// 
// This module provides platform-agnostic utilities to find common executables
// (bun, npm, node, etc.) regardless of PATH limitations on GUI apps.
//
// Platform support:
// - macOS: Finder/Launchpad apps have limited PATH
// - Windows: May need to check Program Files, AppData, etc.
// - Linux: Various package manager locations

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Mutex, Once};
use std::env;

/// Platform-specific path separator
#[cfg(windows)]
const PATH_SEP: char = ';';

#[cfg(not(windows))]
const PATH_SEP: char = ':';

/// Get user home directory path
fn get_home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Ok(userprofile) = env::var("USERPROFILE") {
            return Some(PathBuf::from(userprofile));
        }
        if let Ok(home) = env::var("HOMEDRIVE").and_then(|d| env::var("HOMEPATH").map(|p| PathBuf::from(d).join(p))) {
            return Some(home);
        }
    }
    
    #[cfg(not(windows))]
    {
        if let Ok(home) = env::var("HOME") {
            return Some(PathBuf::from(home));
        }
    }
    
    None
}

/// Get platform-specific bun installation paths
fn get_bun_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Check PATH first (fastest path)
    if let Ok(path_env) = env::var("PATH") {
        for dir in path_env.split(PATH_SEP) {
            paths.push(PathBuf::from(dir).join("bun"));
        }
    }
    
    // macOS-specific paths
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = get_home_dir() {
            paths.push(home.join(".bun").join("bin").join("bun"));
            paths.push(home.join(".nvm").join("versions").join("node").join("*").join("bin").join("bun"));
        }
        paths.push(PathBuf::from("/opt/homebrew/bin/bun"));  // Apple Silicon
        paths.push(PathBuf::from("/usr/local/bin/bun"));      // Intel Mac
    }
    
    // Linux-specific paths
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = get_home_dir() {
            paths.push(home.join(".bun").join("bin").join("bun"));
            paths.push(home.join(".nvm").join("versions").join("node").join("*").join("bin").join("bun"));
        }
        paths.push(PathBuf::from("/usr/local/bin/bun"));
        paths.push(PathBuf::from("/snap/bin/bun"));
    }
    
    // Windows-specific paths
    #[cfg(windows)]
    {
        if let Some(home) = get_home_dir() {
            paths.push(home.join(".bun").join("bin").join("bun.exe"));
        }
        
        // Program Files
        if let Ok(program_files) = env::var("ProgramFiles") {
            paths.push(PathBuf::from(&program_files).join("bun").join("bin").join("bun.exe"));
        }
        if let Ok(program_files_x86) = env::var("ProgramFiles(x86)") {
            paths.push(PathBuf::from(&program_files_x86).join("bun").join("bin").join("bun.exe"));
        }
        
        // Local AppData
        if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
            paths.push(PathBuf::from(&local_appdata).join("bun").join("bin").join("bun.exe"));
        }
    }
    
    paths
}

/// Get platform-specific npm installation paths
fn get_npm_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Check PATH first
    if let Ok(path_env) = env::var("PATH") {
        for dir in path_env.split(PATH_SEP) {
            paths.push(PathBuf::from(dir).join("npm"));
        }
    }
    
    // macOS/Linux
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        if let Some(home) = get_home_dir() {
            // nvm installations
            paths.push(home.join(".nvm").join("versions").join("node").join("*").join("bin").join("npm"));
            // fnm installations
            paths.push(home.join(".fnm").join("node-versions").join("*").join("installation").join("bin").join("npm"));
            // volta installations
            paths.push(home.join(".volta").join("bin").join("npm"));
        }
        
        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/opt/homebrew/bin/npm"));
            paths.push(PathBuf::from("/usr/local/bin/npm"));
        }
        
        #[cfg(target_os = "linux")]
        {
            paths.push(PathBuf::from("/usr/local/bin/npm"));
            paths.push(PathBuf::from("/snap/bin/npm"));
        }
    }
    
    // Windows
    #[cfg(windows)]
    {
        if let Some(home) = get_home_dir() {
            paths.push(home.join(".nvm").join("versions").join("node").join("*").join("npm.cmd"));
            paths.push(home.join(".fnm").join("node-versions").join("*").join("installation").join("npm.cmd"));
            paths.push(home.join(".volta").join("bin").join("npm.cmd"));
        }
        
        if let Ok(program_files) = env::var("ProgramFiles") {
            paths.push(PathBuf::from(&program_files).join("nodejs").join("npm.cmd"));
        }
        if let Ok(program_files_x86) = env::var("ProgramFiles(x86)") {
            paths.push(PathBuf::from(&program_files_x86).join("nodejs").join("npm.cmd"));
        }
    }
    
    paths
}

/// Get platform-specific node installation paths
#[allow(dead_code)]
fn get_node_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Check PATH first
    if let Ok(path_env) = env::var("PATH") {
        for dir in path_env.split(PATH_SEP) {
            paths.push(PathBuf::from(dir).join("node"));
        }
    }
    
    // macOS/Linux
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        if let Some(home) = get_home_dir() {
            paths.push(home.join(".nvm").join("versions").join("node").join("*").join("bin").join("node"));
            paths.push(home.join(".fnm").join("node-versions").join("*").join("installation").join("bin").join("node"));
            paths.push(home.join(".volta").join("bin").join("node"));
        }
        
        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/opt/homebrew/bin/node"));
            paths.push(PathBuf::from("/usr/local/bin/node"));
        }
        
        #[cfg(target_os = "linux")]
        {
            paths.push(PathBuf::from("/usr/local/bin/node"));
            paths.push(PathBuf::from("/snap/bin/node"));
        }
    }
    
    // Windows
    #[cfg(windows)]
    {
        if let Some(home) = get_home_dir() {
            paths.push(home.join(".nvm").join("versions").join("node").join("*").join("node.exe"));
            paths.push(home.join(".fnm").join("node-versions").join("*").join("installation").join("node.exe"));
            paths.push(home.join(".volta").join("bin").join("node.exe"));
        }
        
        if let Ok(program_files) = env::var("ProgramFiles") {
            paths.push(PathBuf::from(&program_files).join("nodejs").join("node.exe"));
        }
        if let Ok(program_files_x86) = env::var("ProgramFiles(x86)") {
            paths.push(PathBuf::from(&program_files_x86).join("nodejs").join("node.exe"));
        }
    }
    
    paths
}

/// Find an executable by checking common paths
fn find_executable(paths: &[PathBuf]) -> Option<PathBuf> {
    for path in paths {
        // For paths with wildcards, we need to expand them
        if path.to_string_lossy().contains('*') {
            if let Some(parent) = path.parent() {
                if let Some(file_name) = path.file_name() {
                    if let Ok(entries) = std::fs::read_dir(parent) {
                        for entry in entries.flatten() {
                            let full_path = entry.path().join(file_name);
                            if full_path.exists() && is_executable(&full_path) {
                                return Some(full_path);
                            }
                        }
                    }
                }
            }
        } else if path.exists() && is_executable(path) {
            return Some(path.clone());
        }
    }
    None
}

/// Check if a path is executable
#[cfg(unix)]
fn is_executable(path: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.is_file() && path.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(path: &PathBuf) -> bool {
    path.is_file()
        && path.extension()
            .map(|ext| matches!(ext.to_str(), Some("exe") | Some("cmd") | Some("bat")))
            .unwrap_or(false)
}

/// Find bun executable, returns the full path if found
pub fn find_bun() -> Option<PathBuf> {
    find_executable(&get_bun_paths())
}

/// Find npm executable, returns the full path if found  
pub fn find_npm() -> Option<PathBuf> {
    find_executable(&get_npm_paths())
}

/// Find node executable, returns the full path if found
#[allow(dead_code)]
pub fn find_node() -> Option<PathBuf> {
    find_executable(&get_node_paths())
}

/// Check if bun is available
pub fn has_bun() -> bool {
    find_bun().is_some()
}

/// Check if npm is available
pub fn has_npm() -> bool {
    find_npm().is_some()
}

/// Get the preferred package manager (bun or npm)
/// Returns ("bun" or "npm", full_path)
#[allow(dead_code)]
pub fn get_package_manager() -> Option<(&'static str, PathBuf)> {
    if let Some(bun) = find_bun() {
        return Some(("bun", bun));
    }
    if let Some(npm) = find_npm() {
        return Some(("npm", npm));
    }
    None
}

static PATH_INIT: Once = Once::new();
static EXECUTABLE_CACHE: Mutex<Option<HashMap<String, Option<PathBuf>>>> = Mutex::new(None);

/// Ensure PATH has been augmented for GUI-launched apps. Safe to call multiple times.
pub fn ensure_path_initialized() {
    PATH_INIT.call_once(init_extended_path);
}

fn merge_paths(primary: &str, secondary: &str) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut merged = Vec::new();
    for entry in primary
        .split(PATH_SEP)
        .chain(secondary.split(PATH_SEP))
    {
        if entry.is_empty() || !seen.insert(entry.to_string()) {
            continue;
        }
        merged.push(entry);
    }
    merged.join(&PATH_SEP.to_string())
}

#[cfg(unix)]
fn default_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "/bin/zsh".into()
        } else {
            "/bin/bash".into()
        }
    })
}

#[cfg(unix)]
fn run_shell_output(args: &[&str]) -> Option<String> {
    use std::sync::mpsc;
    use std::time::Duration;

    let shell = default_shell();
    let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let result = std::process::Command::new(&shell)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        let _ = tx.send(result);
    });

    let output = rx.recv_timeout(Duration::from_secs(5)).ok()?.ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8(output.stdout).ok()?;
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

#[cfg(unix)]
fn interactive_shell_path() -> Option<String> {
    run_shell_output(&["-il", "-c", "printf %s \"$PATH\""])
}

#[cfg(unix)]
fn login_shell_path() -> Option<String> {
    run_shell_output(&["-l", "-c", "printf %s \"$PATH\""])
}

#[cfg(target_os = "macos")]
fn path_helper_path() -> Option<String> {
    let output = std::process::Command::new("/usr/libexec/path_helper")
        .arg("-s")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.split(';') {
        let line = line.trim();
        let Some((_, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
fn path_helper_path() -> Option<String> {
    None
}

/// Merge an interactive login-shell PATH into the current process environment.
/// macOS `.app` bundles started from Finder inherit a minimal PATH; tools like
/// bun/nvm/fnm are often configured only in interactive shell init files.
fn augment_path_from_login_shell() {
    let current = env::var("PATH").unwrap_or_default();
    #[cfg(unix)]
    let shell_path = interactive_shell_path()
        .or_else(login_shell_path)
        .or_else(path_helper_path)
        .unwrap_or_default();
    #[cfg(not(unix))]
    let shell_path = String::new();

    let merged = merge_paths(
        &merge_paths(&shell_path, &build_extended_path()),
        &current,
    );
    if merged != current {
        // SAFETY: called during single-threaded app startup before worker threads spawn.
        unsafe { env::set_var("PATH", &merged) };
        tracing::debug!(
            "[PathUtils] Augmented PATH from login shell ({} entries)",
            merged.split(PATH_SEP).filter(|s| !s.is_empty()).count()
        );
    }
}

fn path_directories() -> Vec<PathBuf> {
    env::var("PATH")
        .unwrap_or_default()
        .split(PATH_SEP)
        .filter(|entry| !entry.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn standard_user_bin_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
        dirs.push(PathBuf::from("/usr/local/bin"));
    }
    #[cfg(target_os = "linux")]
    {
        dirs.push(PathBuf::from("/usr/local/bin"));
        dirs.push(PathBuf::from("/snap/bin"));
    }
    if let Some(home) = get_home_dir() {
        #[cfg(windows)]
        {
            for rel in [
                ".cargo\\bin",
                ".bun\\bin",
                "AppData\\Local\\pnpm",
                "AppData\\Roaming\\npm",
                ".local\\bin",
                "go\\bin",
                ".volta\\bin",
            ] {
                dirs.push(home.join(rel));
            }
        }
        #[cfg(not(windows))]
        {
            for rel in [
                ".bun/bin",
                ".local/bin",
                ".cargo/bin",
                "go/bin",
                ".npm-global/bin",
                "Library/pnpm",
                ".volta/bin",
                ".fnm/aliases/default/bin",
            ] {
                dirs.push(home.join(rel));
            }
        }
    }
    dirs
}

fn has_path_component(path: &Path) -> bool {
    path.is_absolute()
        || path.starts_with(".")
        || path
            .to_str()
            .is_some_and(|s| s.starts_with("~/") || s.starts_with("./"))
}

fn expand_user_path(path: &Path) -> PathBuf {
    if let Some(raw) = path.to_str() {
        if let Some(rest) = raw.strip_prefix("~/") {
            if let Some(home) = get_home_dir() {
                return home.join(rest);
            }
        }
    }
    path.to_path_buf()
}

fn shell_lookup_executable(name: &str) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        use std::sync::mpsc;
        use std::time::Duration;

        if name.contains('\0') {
            return None;
        }
        let escaped = name.replace('\'', r"'\''");
        let script = format!("command -v -- '{escaped}'");
        let shell = default_shell();
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let result = std::process::Command::new(&shell)
                .args(["-il", "-c", &script])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output();
            let _ = tx.send(result);
        });

        let output = rx.recv_timeout(Duration::from_secs(5)).ok()?.ok()?;
        if !output.status.success() {
            return None;
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            return None;
        }
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            Some(candidate)
        } else {
            None
        }
    }
    #[cfg(not(unix))]
    {
        let _ = name;
        None
    }
}

fn resolve_executable_uncached(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    if has_path_component(path) {
        let expanded = expand_user_path(path);
        if expanded.is_file() && is_executable(&expanded) {
            return Some(expanded);
        }
        return None;
    }

    let mut dirs = standard_user_bin_dirs();
    dirs.extend(path_directories());
    let mut seen = std::collections::HashSet::new();
    for dir in dirs {
        let key = dir.to_string_lossy().into_owned();
        if !seen.insert(key) {
            continue;
        }
        let candidate = dir.join(name);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let with_exe = dir.join(format!("{name}.exe"));
            if with_exe.is_file() {
                return Some(with_exe);
            }
        }
    }

    shell_lookup_executable(name)
}

fn cache_lookup(name: &str, resolved: Option<PathBuf>) {
    if let Ok(mut guard) = EXECUTABLE_CACHE.lock() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(name.to_string(), resolved);
    }
}

fn cached_lookup(name: &str) -> Option<Option<PathBuf>> {
    EXECUTABLE_CACHE
        .lock()
        .ok()
        .and_then(|cache| cache.as_ref().and_then(|map| map.get(name).cloned()))
}

/// Resolve an executable name to an absolute path when possible.
pub fn resolve_executable(name: &str) -> Option<PathBuf> {
    ensure_path_initialized();
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(cached) = cached_lookup(trimmed) {
        return cached;
    }
    let resolved = resolve_executable_uncached(trimmed);
    cache_lookup(trimmed, resolved.clone());
    resolved
}

/// Rewrite the first token of a shell command to an absolute path when resolvable.
pub fn resolve_command(command: &str) -> String {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut parts = trimmed.split_whitespace();
    let binary = parts.next().unwrap_or_default();
    let rest = parts.collect::<Vec<_>>().join(" ");

    let Some(resolved) = resolve_executable(binary) else {
        return trimmed.to_string();
    };

    let mut out = resolved.to_string_lossy().into_owned();
    if !rest.is_empty() {
        out.push(' ');
        out.push_str(&rest);
    }
    out
}

/// Shell used for compound dev commands on Unix (login, non-interactive).
#[cfg(unix)]
pub fn command_shell() -> String {
    default_shell()
}

#[cfg(windows)]
pub fn command_shell() -> String {
    "cmd".to_string()
}

/// Rewrite pnpm/yarn invocations in package.json scripts to bun or npm.
pub fn rewrite_script_package_manager(script: &str) -> String {
    let use_bun = find_bun().is_some();
    let pm_bin = find_bun()
        .or_else(find_npm)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            if use_bun {
                "bun".into()
            } else {
                "npm".into()
            }
        });

    let mut out = rewrite_pnpm_filter_commands(script, use_bun, &pm_bin);

    if use_bun {
        out = out.replace("pnpm run ", &format!("{pm_bin} run "));
        out = out.replace("pnpm install", &format!("{pm_bin} install"));
        out = out.replace("pnpm i ", &format!("{pm_bin} install "));
        out = out.replace("pnpm i", &format!("{pm_bin} install"));
        out = out.replace("pnpm ", &format!("{pm_bin} run "));
        out = out.replace("yarn run ", &format!("{pm_bin} run "));
        out = out.replace("yarn install", &format!("{pm_bin} install"));
        out = out.replace("yarn ", &format!("{pm_bin} run "));
    } else {
        out = out.replace("pnpm run ", &format!("{pm_bin} run "));
        out = out.replace("pnpm install", &format!("{pm_bin} install"));
        out = out.replace("pnpm i ", &format!("{pm_bin} install "));
        out = out.replace("pnpm i", &format!("{pm_bin} install"));
        out = out.replace("pnpm ", &format!("{pm_bin} run "));
        out = out.replace("yarn run ", &format!("{pm_bin} run "));
        out = out.replace("yarn install", &format!("{pm_bin} install"));
        out = out.replace("yarn ", &format!("{pm_bin} run "));
    }

    out
}

/// Whether a script body delegates to pnpm or yarn.
pub fn script_uses_alternate_package_manager(script: &str) -> bool {
    script.contains("pnpm") || script.contains("yarn")
}

/// Build the shell command to run a package.json script using bun/npm (never pnpm/yarn).
pub fn build_start_command_for_script(script_body: &str, script_name: &str) -> Option<String> {
    let pm_bin = find_bun().or_else(find_npm)?;
    let pm_str = pm_bin.to_string_lossy();

    let rewritten = rewrite_script_package_manager(script_body);

    if script_uses_alternate_package_manager(script_body) || needs_shell_wrapper(&rewritten) {
        tracing::debug!(
            "[PathUtils] Rewrote dev script: {:?} -> {:?}",
            script_body, rewritten
        );
        return Some(rewritten);
    }

    Some(format!("{pm_str} run {script_name}"))
}

/// Normalize a start command using package.json script bodies when needed.
pub fn normalize_project_start_command(code_dir: &Path, command: &str) -> String {
    if let Some(script_name) = extract_pm_run_script_name(command) {
        let pkg = code_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&pkg) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(body) = json
                    .get("scripts")
                    .and_then(|s| s.get(script_name))
                    .and_then(|v| v.as_str())
                {
                    if let Some(cmd) = build_start_command_for_script(body, script_name) {
                        return cmd;
                    }
                }
            }
        }
    }

    rewrite_script_package_manager(command)
}

fn extract_pm_run_script_name(command: &str) -> Option<&str> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let script_name = parts.last()?;
    let run_token = parts[parts.len() - 2];
    if run_token != "run" {
        return None;
    }
    let pm_token = parts[parts.len() - 3];
    if pm_token.ends_with("bun") || pm_token.ends_with("npm") || pm_token == "bun" || pm_token == "npm"
    {
        Some(script_name)
    } else {
        None
    }
}

fn rewrite_pnpm_filter_commands(script: &str, use_bun: bool, pm_bin: &str) -> String {
    let mut result = script.to_string();
    for prefix in ["pnpm --filter ", "pnpm -F "] {
        while let Some(start) = result.find(prefix) {
            let after_prefix = start + prefix.len();
            let rest = &result[after_prefix..];
            let Some(pkg_end) = rest.find(' ') else {
                break;
            };
            let package = &rest[..pkg_end];
            let after_pkg = rest[pkg_end..].trim_start();
            let script_end = after_pkg
                .find(|c: char| c.is_whitespace() || c == '&' || c == ';')
                .unwrap_or(after_pkg.len());
            let script_cmd = &after_pkg[..script_end];
            let tail = &after_pkg[script_end..];

            let replacement = if use_bun {
                format!("{pm_bin} run --filter {package} {script_cmd}")
            } else {
                format!("{pm_bin} run {script_cmd} -w {package}")
            };

            result = format!(
                "{}{}{tail}",
                &result[..start],
                replacement,
            );
        }
    }
    result
}

/// Whether a command string should be executed as an external long-running process.
pub fn is_runnable_external_command(command: &str) -> bool {
    let trimmed = command.trim();
    !trimmed.is_empty()
        && trimmed != "(built-in static server)"
        && !trimmed.contains("built-in static server")
}

/// True when the command needs a shell (`&&`, pipes, etc.).
pub fn needs_shell_wrapper(command: &str) -> bool {
    command.contains(|c| matches!(c, '|' | '&' | ';' | '>' | '<' | '$' | '`' | '(' | ')'))
}

/// Split a simple `binary arg1 arg2` command for direct spawning (no shell).
pub fn parse_direct_command(command: &str) -> Option<(PathBuf, Vec<String>)> {
    let trimmed = command.trim();
    if trimmed.is_empty() || needs_shell_wrapper(trimmed) {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let program = parts.next()?;
    let args: Vec<String> = parts.map(str::to_string).collect();
    let path = PathBuf::from(program);

    if path.is_absolute() && path.is_file() {
        return Some((path, args));
    }

    resolve_executable(program).map(|resolved| (resolved, args))
}

/// Apply environment variables GUI child processes typically need on macOS.
pub fn apply_gui_child_env(cmd: &mut std::process::Command) {
    cmd.env("PATH", current_path());
    if let Ok(home) = env::var("HOME") {
        cmd.env("HOME", home);
    }
    if let Ok(user) = env::var("USER") {
        cmd.env("USER", user);
    }
    if let Ok(lang) = env::var("LANG") {
        cmd.env("LANG", lang);
    } else {
        cmd.env("LANG", "en_US.UTF-8");
    }
    cmd.env("TERM", "dumb");
}

/// Apply environment variables for async child processes.
pub fn apply_gui_child_env_async(cmd: &mut tokio::process::Command) {
    cmd.env("PATH", current_path());
    if let Ok(home) = env::var("HOME") {
        cmd.env("HOME", home);
    }
    if let Ok(user) = env::var("USER") {
        cmd.env("USER", user);
    }
    if let Ok(lang) = env::var("LANG") {
        cmd.env("LANG", lang);
    } else {
        cmd.env("LANG", "en_US.UTF-8");
    }
    cmd.env("TERM", "dumb");
}

/// Current PATH after initialization (safe for child processes).
pub fn current_path() -> String {
    ensure_path_initialized();
    env::var("PATH").unwrap_or_else(|_| build_extended_path())
}

/// Build an extended PATH that includes common development tool locations
/// This should be called at application startup to fix the PATH issue on GUI apps
pub fn build_extended_path() -> String {
    let mut path_dirs: Vec<String> = Vec::new();
    
    // Get existing PATH
    if let Ok(existing) = env::var("PATH") {
        for dir in existing.split(PATH_SEP) {
            if !dir.is_empty() && !path_dirs.contains(&dir.to_string()) {
                path_dirs.push(dir.to_string());
            }
        }
    }
    
    // Add platform-specific paths
    #[cfg(target_os = "macos")]
    {
        let extra_paths = [
            "/opt/homebrew/bin",
            "/opt/homebrew/sbin",
            "/usr/local/bin",
            "/usr/local/sbin",
        ];
        for path in extra_paths {
            if !path_dirs.contains(&path.to_string()) {
                path_dirs.push(path.to_string());
            }
        }
        
        // Add user-specific paths
        if let Some(home) = get_home_dir() {
            let home_str = home.to_string_lossy().to_string();
            let user_paths = [
                format!("{}/.bun/bin", home_str),
                format!("{}/.cargo/bin", home_str),
                format!("{}/.volta/bin", home_str),
                format!("{}/.local/bin", home_str),
                format!("{}/go/bin", home_str),
            ];
            for path in user_paths {
                if !path_dirs.contains(&path) {
                    path_dirs.push(path);
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        let extra_paths = [
            "/usr/local/bin",
            "/snap/bin",
        ];
        for path in extra_paths {
            if !path_dirs.contains(&path.to_string()) {
                path_dirs.push(path.to_string());
            }
        }
        
        if let Some(home) = get_home_dir() {
            let home_str = home.to_string_lossy().to_string();
            let user_paths = [
                format!("{}/.bun/bin", home_str),
                format!("{}/.cargo/bin", home_str),
                format!("{}/.volta/bin", home_str),
                format!("{}/.local/bin", home_str),
                format!("{}/go/bin", home_str),
            ];
            for path in user_paths {
                if !path_dirs.contains(&path) {
                    path_dirs.push(path);
                }
            }
        }
    }
    
    #[cfg(windows)]
    {
        // Windows-specific paths
        if let Ok(program_files) = env::var("ProgramFiles") {
            let pf = PathBuf::from(&program_files);
            if !path_dirs.contains(&pf.to_string_lossy().to_string()) {
                path_dirs.push(pf.to_string_lossy().to_string());
            }
        }
        if let Ok(program_files_x86) = env::var("ProgramFiles(x86)") {
            let pf_x86 = PathBuf::from(&program_files_x86);
            if !path_dirs.contains(&pf_x86.to_string_lossy().to_string()) {
                path_dirs.push(pf_x86.to_string_lossy().to_string());
            }
        }
        
        if let Some(home) = get_home_dir() {
            let home_str = home.to_string_lossy().to_string();
            let user_paths = [
                format!("{}/.bun/bin", home_str),
                format!("{}/.cargo/bin", home_str),
                format!("{}/AppData/Local/Programs", home_str),
            ];
            for path in user_paths {
                if !path_dirs.contains(&path) {
                    path_dirs.push(path);
                }
            }
        }
    }
    
    path_dirs.join(&PATH_SEP.to_string())
}

/// Initialize extended PATH at application startup.
/// Must only run inside `PATH_INIT.call_once` — do not call directly.
fn init_extended_path() {
    tracing::debug!(
        "[PathUtils] Setting extended PATH ({} platform)",
        std::env::consts::OS
    );

    // Merge interactive login-shell PATH (nvm/fnm/Homebrew from shell rc files),
    // then fall back to hard-coded developer tool locations.
    augment_path_from_login_shell();

    let extended_path = env::var("PATH").unwrap_or_else(|_| build_extended_path());
    tracing::debug!("[PathUtils] PATH length: {} characters", extended_path.len());

    if let Some(bun) = find_bun() {
        tracing::debug!("[PathUtils] Found bun at: {:?}", bun);
    } else {
        tracing::debug!("[PathUtils] bun not found after PATH extension");
    }

    if let Some(npm) = find_npm() {
        tracing::debug!("[PathUtils] Found npm at: {:?}", npm);
    } else {
        tracing::debug!("[PathUtils] npm not found after PATH extension");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_paths_deduplicates_and_preserves_order() {
        let merged = merge_paths("/a:/b:/c", "/b:/d");
        assert_eq!(merged, "/a:/b:/c:/d");
    }

    #[test]
    fn is_runnable_external_command_filters_builtin_server() {
        assert!(!is_runnable_external_command(""));
        assert!(!is_runnable_external_command("(built-in static server)"));
        assert!(is_runnable_external_command("bun run dev"));
    }

    #[test]
    fn parse_direct_command_splits_binary_and_args() {
        let (bin, args) = parse_direct_command("bun run dev").expect("should parse");
        assert!(bin.ends_with("bun"));
        assert_eq!(args, vec!["run", "dev"]);
        assert!(parse_direct_command("cd app && bun run dev").is_none());
    }

    #[test]
    fn rewrite_pnpm_monorepo_script_to_bun() {
        let script = "pnpm --filter @hytech/server start:dev & pnpm --filter @hytech/web dev & wait";
        let rewritten = rewrite_script_package_manager(script);
        assert!(!rewritten.contains("pnpm"));
        assert!(rewritten.contains("run --filter @hytech/server start:dev"));
        assert!(rewritten.contains("run --filter @hytech/web dev"));
        assert!(needs_shell_wrapper(&rewritten));
    }

    #[test]
    fn rewrite_pnpm_script_to_npm_when_no_bun() {
        let script = "pnpm --filter @app/web dev";
        let rewritten = rewrite_pnpm_filter_commands(script, false, "npm");
        assert_eq!(rewritten, "npm run dev -w @app/web");
    }

    #[test]
    fn extract_pm_run_script_name_parses_absolute_bun_path() {
        assert_eq!(
            extract_pm_run_script_name("/Users/test/.bun/bin/bun run dev"),
            Some("dev")
        );
    }
}