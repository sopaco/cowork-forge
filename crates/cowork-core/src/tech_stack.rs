// Tech Stack Configuration - Defines project type defaults and constraints
//
// This module enforces technology selection rules:
// - Web: HTML/JS/CSS (vanilla) or React only, vanilla preferred
// - Tool Scripts: Node.js only
// - Backend: Rust only
// - Desktop: Rust (Tauri) only
// - Mobile: Native Android/iOS only
// - Runtime: bun preferred, fallback to npm for web projects

use serde::{Deserialize, Serialize};
use std::fmt;

/// Project types supported by Cowork Forge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectType {
    /// Web application (HTML/JS/CSS or React)
    Web,
    /// Tool/Utility scripts
    Tool,
    /// Backend API/Service
    Backend,
    /// FullStack application (Frontend + Backend)
    FullStack,
    /// Desktop application
    Desktop,
    /// Mobile application
    Mobile,
    /// CLI tool
    Cli,
    /// Unknown project type
    Unknown,
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectType::Web => write!(f, "Web"),
            ProjectType::Tool => write!(f, "Tool"),
            ProjectType::Backend => write!(f, "Backend"),
            ProjectType::FullStack => write!(f, "FullStack"),
            ProjectType::Desktop => write!(f, "Desktop"),
            ProjectType::Mobile => write!(f, "Mobile"),
            ProjectType::Cli => write!(f, "CLI"),
            ProjectType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Technology stack for a project type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    pub project_type: ProjectType,
    pub primary_language: String,
    pub framework: Option<String>,
    pub package_manager: PackageManager,
    pub build_tool: String,
    pub runtime: String,
    pub additional_deps: Vec<String>,
}

/// Package manager preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageManager {
    Bun,
    Npm,
    Cargo,
    Pip,
    Maven,
    Gradle,
    None,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManager::Bun => write!(f, "bun"),
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Cargo => write!(f, "cargo"),
            PackageManager::Pip => write!(f, "pip"),
            PackageManager::Maven => write!(f, "maven"),
            PackageManager::Gradle => write!(f, "gradle"),
            PackageManager::None => write!(f, "none"),
        }
    }
}

/// Get default tech stack for a project type
pub fn get_default_tech_stack(project_type: ProjectType) -> TechStack {
    match project_type {
        ProjectType::Web => TechStack {
            project_type: ProjectType::Web,
            primary_language: "JavaScript".to_string(),
            framework: None, // Vanilla JS preferred
            package_manager: PackageManager::Bun, // bun preferred
            build_tool: "vite".to_string(),
            runtime: "bun".to_string(),
            additional_deps: vec![
                "vite".to_string(),
                "@vitejs/plugin-react".to_string(), // Include React support
            ],
        },
        ProjectType::Tool => TechStack {
            project_type: ProjectType::Tool,
            primary_language: "JavaScript".to_string(),
            framework: None,
            package_manager: PackageManager::Bun,
            build_tool: "tsc".to_string(),
            runtime: "node".to_string(),
            additional_deps: vec![
                "@types/node".to_string(),
            ],
        },
        ProjectType::Backend => TechStack {
            project_type: ProjectType::Backend,
            primary_language: "Rust".to_string(),
            framework: None,
            package_manager: PackageManager::Cargo,
            build_tool: "cargo".to_string(),
            runtime: "native".to_string(),
            additional_deps: vec![
                "tokio".to_string(),
                "serde".to_string(),
                "serde_json".to_string(),
            ],
        },
        ProjectType::FullStack => TechStack {
            project_type: ProjectType::FullStack,
            primary_language: "Rust + JavaScript".to_string(),
            framework: None,
            package_manager: PackageManager::Bun, // Frontend uses bun
            build_tool: "vite + cargo".to_string(),
            runtime: "bun + native".to_string(),
            additional_deps: vec![
                "vite".to_string(),
                "tokio".to_string(),
                "serde".to_string(),
                "serde_json".to_string(),
            ],
        },
        ProjectType::Desktop => TechStack {
            project_type: ProjectType::Desktop,
            primary_language: "Rust".to_string(),
            framework: Some("Tauri".to_string()),
            package_manager: PackageManager::Cargo,
            build_tool: "cargo".to_string(),
            runtime: "native".to_string(),
            additional_deps: vec![
                "tauri".to_string(),
            ],
        },
        ProjectType::Mobile => {
            // Mobile has platform-specific stacks
            TechStack {
                project_type: ProjectType::Mobile,
                primary_language: "Kotlin/Swift".to_string(),
                framework: None,
                package_manager: PackageManager::None,
                build_tool: "gradle/xcodebuild".to_string(),
                runtime: "native".to_string(),
                additional_deps: vec![],
            }
        }
        ProjectType::Cli => TechStack {
            project_type: ProjectType::Cli,
            primary_language: "Rust".to_string(),
            framework: None,
            package_manager: PackageManager::Cargo,
            build_tool: "cargo".to_string(),
            runtime: "native".to_string(),
            additional_deps: vec![
                "clap".to_string(),
            ],
        },
        ProjectType::Unknown => TechStack {
            project_type: ProjectType::Unknown,
            primary_language: "Unknown".to_string(),
            framework: None,
            package_manager: PackageManager::None,
            build_tool: "unknown".to_string(),
            runtime: "unknown".to_string(),
            additional_deps: vec![],
        },
    }
}

/// Detect project type from description
pub fn detect_project_type(description: &str) -> ProjectType {
    let desc_lower = description.to_lowercase();
    
    // Check for web project indicators
    let web_keywords = ["网页", "网站", "web", "front", "frontend", "前端", "网页应用", "web应用", "h5"];
    let is_web = web_keywords.iter().any(|kw| desc_lower.contains(kw));
    
    // Check for backend project indicators
    let backend_keywords = ["后端", "backend", "api", "接口", "服务", "server", "service"];
    let is_backend = backend_keywords.iter().any(|kw| desc_lower.contains(kw));
    
    // Check for desktop project indicators
    let desktop_keywords = ["桌面", "desktop", "gui", "客户端", "pc"];
    let is_desktop = desktop_keywords.iter().any(|kw| desc_lower.contains(kw));
    
    // Check for mobile project indicators
    let mobile_keywords = ["移动", "mobile", "android", "ios", "手机"];
    let is_mobile = mobile_keywords.iter().any(|kw| desc_lower.contains(kw));
    
    // Check for tool script indicators
    let tool_keywords = ["脚本", "script", "tool", "工具", "脚本程序"];
    let is_tool = tool_keywords.iter().any(|kw| desc_lower.contains(kw));
    
    // Additional keywords for full-stack detection
    let fullstack_keywords = [
        "前后端", "full-stack", "fullstack", "全栈",
        "前后端分离", "前后端一起", "前后端完整",
        "完整应用", "完整项目"
    ];
    let is_fullstack_explicit = fullstack_keywords.iter().any(|kw| desc_lower.contains(kw));
    
    // Priority: Mobile > Desktop > FullStack > Web > Backend > Tool
    if is_mobile {
        return ProjectType::Mobile;
    } else if is_desktop {
        return ProjectType::Desktop;
    } else if is_fullstack_explicit || (is_web && is_backend) {
        return ProjectType::FullStack;
    } else if is_web {
        return ProjectType::Web;
    } else if is_backend {
        return ProjectType::Backend;
    } else if is_tool {
        return ProjectType::Tool;
    }
    
    ProjectType::Unknown
}

/// Get tech stack constraints as instruction text
pub fn get_tech_stack_instructions(project_type: ProjectType) -> String {
    match project_type {
        ProjectType::Web => {
            r#"**TECHNOLOGY STACK CONSTRAINTS:**
- **MUST use either Vanilla HTML/JS/CSS OR React only**
- **PREFER Vanilla HTML/JS/CSS** (no framework) unless React is explicitly required
- **DO NOT use Vue, Angular, Svelte, or other frameworks**
- **DO NOT use TypeScript** unless explicitly requested
- Use Vite as build tool
- Use bun as package manager (preferred), npm as fallback
- Modern ES6+ JavaScript syntax is required

**Project Structure (Vanilla):**
```
index.html
styles.css
script.js
```

**Project Structure (React):**
```
index.html
package.json
src/
  main.jsx
  App.jsx
  styles.css
```"#.to_string()
        }
        ProjectType::Tool => {
            r#"**TECHNOLOGY STACK CONSTRAINTS:**
- **MUST use Node.js** for tool scripts
- Use bun as package manager (preferred), npm as fallback
- Modern ES6+ JavaScript syntax
- TypeScript is optional but recommended

**Project Structure:**
```
package.json
src/
  index.ts (or index.js)
  types.ts (if using TypeScript)
```"#.to_string()
        }
        ProjectType::Backend => {
            r#"**TECHNOLOGY STACK CONSTRAINTS:**
- **MUST use Rust** for backend services
- Use Cargo as package manager
- Use Tokio for async runtime
- Use Serde for JSON serialization
- Follow Rust best practices and idioms

**Project Structure:**
```
Cargo.toml
src/
  main.rs
  lib.rs
  models.rs
  handlers.rs
```"#.to_string()
        }
        ProjectType::FullStack => {
            r#"**TECHNOLOGY STACK CONSTRAINTS:**

**Frontend:**
- **MUST use either Vanilla HTML/JS/CSS OR React only**
- **PREFER Vanilla HTML/JS/CSS** (no framework) unless React is explicitly required
- **DO NOT use Vue, Angular, Svelte, or other frameworks**
- Use bun as package manager (preferred), npm as fallback

**Backend:**
- **MUST use Rust** for backend API
- Use Cargo as package manager
- Use Tokio for async runtime
- Use Serde for JSON serialization
- Provide REST API endpoints

**Communication:**
- Frontend communicates with backend via REST API
- Use JSON for data exchange
- Handle CORS properly

**Project Structure (Frontend + Backend Separation):**
```
frontend/
  index.html
  styles.css
  script.js
  package.json  (if using React)
backend/
  Cargo.toml
  src/
    main.rs
    lib.rs
    models.rs
    handlers.rs
```"#.to_string()
        }
        ProjectType::Desktop => {
            r#"**TECHNOLOGY STACK CONSTRAINTS:**
- **MUST use Rust with Tauri** for desktop applications
- Use Cargo as package manager
- Frontend: Vanilla HTML/JS/CSS or React only
- Use Tauri for native integration

**Project Structure:**
```
Cargo.toml
src-tauri/
  src/
    main.rs
  tauri.conf.json
index.html
```"#.to_string()
        }
        ProjectType::Mobile => {
            r#"**TECHNOLOGY STACK CONSTRAINTS:**
- **MUST use native platform solutions:**
  - Android: Kotlin + Gradle
  - iOS: Swift + Xcode
- **DO NOT use cross-platform frameworks** (React Native, Flutter, etc.)
- Follow platform-specific best practices

**Project Structure (Android):**
```
app/src/main/
  AndroidManifest.xml
  MainActivity.kt
build.gradle
```"#.to_string()
        }
        ProjectType::Cli => {
            r#"**TECHNOLOGY STACK CONSTRAINTS:**
- **MUST use Rust** for CLI tools
- Use Cargo as package manager
- Use clap for command-line argument parsing
- Follow Rust CLI best practices

**Project Structure:**
```
Cargo.toml
src/
  main.rs
```"#.to_string()
        }
        ProjectType::Unknown => {
            r#"**TECHNOLOGY STACK CONSTRAINTS:**
- Project type could not be determined
- Please provide more specific requirements in the project description
- Consider specifying: web, backend, desktop, mobile, or tool script"#.to_string()
        }
    }
}

/// Get package manager command with bun preference
pub fn get_package_manager_command(package_manager: &PackageManager, command: &str) -> String {
    match package_manager {
        PackageManager::Bun | PackageManager::Npm => {
            // Try bun first, fall back to npm
            format!("{} {}", package_manager, command)
        }
        PackageManager::Cargo => {
            format!("cargo {}", command)
        }
        PackageManager::Pip => {
            format!("pip {}", command)
        }
        PackageManager::Maven => {
            format!("mvn {}", command)
        }
        PackageManager::Gradle => {
            format!("./gradlew {}", command)
        }
        PackageManager::None => {
            String::new()
        }
    }
}

/// Validate generated project structure against tech stack requirements
pub fn validate_project_structure(project_type: &ProjectType, files: &[String]) -> Vec<String> {
    let mut issues = Vec::new();
    
    match project_type {
        ProjectType::Web => {
            let has_index_html = files.iter().any(|f| f.contains("index.html"));
            let has_package_json = files.iter().any(|f| f.contains("package.json"));
            
            if !has_index_html && !has_package_json {
                issues.push("Web project must have either index.html or package.json".to_string());
            }
            
            // Check for forbidden frameworks
            let forbidden = files.iter().any(|f| {
                f.contains("vue") || f.contains("angular") || f.contains("svelte")
                    || f.contains("next.config") || f.contains("nuxt.config")
            });
            
            if forbidden {
                issues.push("Web project must not use Vue, Angular, Svelte, Next.js, or Nuxt.js".to_string());
            }
        }
        ProjectType::Tool => {
            let has_package_json = files.iter().any(|f| f.contains("package.json"));
            if !has_package_json {
                issues.push("Tool project must have package.json".to_string());
            }
        }
        ProjectType::Backend => {
            let has_cargo_toml = files.iter().any(|f| f.contains("Cargo.toml"));
            let has_main_rs = files.iter().any(|f| f.contains("main.rs"));
            
            if !has_cargo_toml {
                issues.push("Backend project must have Cargo.toml".to_string());
            }
            if !has_main_rs {
                issues.push("Backend project must have main.rs".to_string());
            }
        }
        ProjectType::FullStack => {
            let has_frontend = files.iter().any(|f| f.contains("index.html") || f.contains("package.json"));
            let has_backend = files.iter().any(|f| f.contains("Cargo.toml"));
            
            if !has_frontend {
                issues.push("FullStack project must have frontend files (index.html or package.json)".to_string());
            }
            if !has_backend {
                issues.push("FullStack project must have backend files (Cargo.toml)".to_string());
            }
            
            // Check for forbidden frameworks in frontend
            let forbidden = files.iter().any(|f| {
                f.contains("vue") || f.contains("angular") || f.contains("svelte")
                    || f.contains("next.config") || f.contains("nuxt.config")
            });
            
            if forbidden {
                issues.push("Frontend must not use Vue, Angular, Svelte, Next.js, or Nuxt.js".to_string());
            }
        }
        ProjectType::Desktop => {
            let has_cargo_toml = files.iter().any(|f| f.contains("Cargo.toml"));
            let has_tauri_conf = files.iter().any(|f| f.contains("tauri.conf.json"));
            
            if !has_cargo_toml {
                issues.push("Desktop project must have Cargo.toml".to_string());
            }
            if !has_tauri_conf {
                issues.push("Desktop project must have tauri.conf.json".to_string());
            }
        }
        ProjectType::Mobile => {
            // Mobile projects would have platform-specific files
            let has_android = files.iter().any(|f| f.contains("build.gradle") || f.contains("AndroidManifest"));
            let has_ios = files.iter().any(|f| f.contains("Info.plist") || f.contains("Podfile"));
            
            if !has_android && !has_ios {
                issues.push("Mobile project must have Android (build.gradle) or iOS (Info.plist) files".to_string());
            }
        }
        ProjectType::Cli => {
            let has_cargo_toml = files.iter().any(|f| f.contains("Cargo.toml"));
            if !has_cargo_toml {
                issues.push("CLI project must have Cargo.toml".to_string());
            }
        }
        ProjectType::Unknown => {
            // No validation for unknown project type
        }
    }
    
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_web_project() {
        assert_eq!(detect_project_type("build a web app"), ProjectType::Web);
        assert_eq!(detect_project_type("create a website"), ProjectType::Web);
        assert_eq!(detect_project_type("build a UI"), ProjectType::Web);
    }

    #[test]
    fn test_detect_backend_project() {
        assert_eq!(detect_project_type("build a REST API"), ProjectType::Backend);
        assert_eq!(detect_project_type("create a server"), ProjectType::Backend);
    }

    #[test]
    fn test_detect_desktop_project() {
        assert_eq!(detect_project_type("build a desktop app"), ProjectType::Desktop);
        assert_eq!(detect_project_type("create a GUI application"), ProjectType::Desktop);
    }

    #[test]
    fn test_detect_mobile_project() {
        assert_eq!(detect_project_type("build a mobile app"), ProjectType::Mobile);
        assert_eq!(detect_project_type("create an Android app"), ProjectType::Mobile);
    }

    #[test]
    fn test_get_default_web_stack() {
        let stack = get_default_tech_stack(ProjectType::Web);
        assert_eq!(stack.primary_language, "JavaScript");
        assert_eq!(stack.framework, None); // Vanilla preferred
        assert_eq!(stack.package_manager, PackageManager::Bun);
    }

    #[test]
    fn test_get_default_backend_stack() {
        let stack = get_default_tech_stack(ProjectType::Backend);
        assert_eq!(stack.primary_language, "Rust");
        assert_eq!(stack.package_manager, PackageManager::Cargo);
    }
}