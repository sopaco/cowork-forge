// Path utilities for resolving executable search issues across platforms
// 
// This module provides platform-agnostic utilities to find common executables
// (bun, npm, node, etc.) regardless of PATH limitations on GUI apps.
//
// Platform support:
// - macOS: Finder/Launchpad apps have limited PATH
// - Windows: May need to check Program Files, AppData, etc.
// - Linux: Various package manager locations

use std::path::PathBuf;
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

/// Initialize extended PATH at application startup
/// Call this early in the application lifecycle to ensure all child processes
/// inherit the correct PATH
pub fn init_extended_path() {
    let extended_path = build_extended_path();
    
    // Log for debugging
    eprintln!("[PathUtils] Setting extended PATH ({} platform)", std::env::consts::OS);
    eprintln!("[PathUtils] PATH length: {} characters", extended_path.len());
    
    // Set the PATH environment variable
    // Note: Using unsafe block to work around potential Rust version or environment issues
    // std::env::set_var is safe in standard Rust, but may be marked differently in this environment
    unsafe { std::env::set_var("PATH", &extended_path) };
    
    // Verify that bun/npm can now be found
    if let Some(bun) = find_bun() {
        eprintln!("[PathUtils] Found bun at: {:?}", bun);
    } else {
        eprintln!("[PathUtils] bun not found after PATH extension");
    }
    
    if let Some(npm) = find_npm() {
        eprintln!("[PathUtils] Found npm at: {:?}", npm);
    } else {
        eprintln!("[PathUtils] npm not found after PATH extension");
    }
}