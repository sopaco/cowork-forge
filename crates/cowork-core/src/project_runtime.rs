// Project Runtime Configuration Module
// Provides standardized runtime configuration for different project types
// Used by GUI Preview/Run functionality

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Runtime type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    // Frontend
    #[serde(rename = "vanilla_html")]
    VanillaHtml,
    ReactVite,
    ReactCra,
    VueVite,
    VueCli,
    SolidVite,
    SvelteVite,
    
    // Backend
    RustBackend,
    NodeExpress,
    NodeFastify,
    NodeNest,
    PythonFastapi,
    PythonFlask,
    PythonDjango,
    
    // FullStack
    #[serde(rename = "fullstack_react_rust")]
    FullstackReactRust,
    #[serde(rename = "fullstack_react_node")]
    FullstackReactNode,
    #[serde(rename = "fullstack_vanilla_rust")]
    FullstackVanillaRust,
    #[serde(rename = "fullstack_vanilla_node")]
    FullstackVanillaNode,
    
    // Desktop
    TauriReact,
    TauriVanilla,
    ElectronReact,
    
    // CLI/Tools
    NodeTool,
    RustCli,
    
    Unknown,
}

impl Default for RuntimeType {
    fn default() -> Self {
        RuntimeType::Unknown
    }
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeType::VanillaHtml => write!(f, "Vanilla HTML"),
            RuntimeType::ReactVite => write!(f, "React + Vite"),
            RuntimeType::ReactCra => write!(f, "React + CRA"),
            RuntimeType::VueVite => write!(f, "Vue + Vite"),
            RuntimeType::VueCli => write!(f, "Vue + CLI"),
            RuntimeType::SolidVite => write!(f, "Solid + Vite"),
            RuntimeType::SvelteVite => write!(f, "Svelte + Vite"),
            RuntimeType::RustBackend => write!(f, "Rust Backend"),
            RuntimeType::NodeExpress => write!(f, "Node.js Express"),
            RuntimeType::NodeFastify => write!(f, "Node.js Fastify"),
            RuntimeType::NodeNest => write!(f, "Node.js NestJS"),
            RuntimeType::PythonFastapi => write!(f, "Python FastAPI"),
            RuntimeType::PythonFlask => write!(f, "Python Flask"),
            RuntimeType::PythonDjango => write!(f, "Python Django"),
            RuntimeType::FullstackReactRust => write!(f, "Fullstack React + Rust"),
            RuntimeType::FullstackReactNode => write!(f, "Fullstack React + Node"),
            RuntimeType::FullstackVanillaRust => write!(f, "Fullstack Vanilla + Rust"),
            RuntimeType::FullstackVanillaNode => write!(f, "Fullstack Vanilla + Node"),
            RuntimeType::TauriReact => write!(f, "Tauri + React"),
            RuntimeType::TauriVanilla => write!(f, "Tauri + Vanilla"),
            RuntimeType::ElectronReact => write!(f, "Electron + React"),
            RuntimeType::NodeTool => write!(f, "Node.js Tool"),
            RuntimeType::RustCli => write!(f, "Rust CLI"),
            RuntimeType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Frontend framework type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FrontendFramework {
    React,
    Vue,
    Solid,
    Svelte,
    Vanilla,
}

impl Default for FrontendFramework {
    fn default() -> Self {
        FrontendFramework::Vanilla
    }
}

/// Backend framework type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackendFramework {
    Axum,
    ActixWeb,
    Warp,
    Rocket,
    Express,
    Fastify,
    Nest,
    FastApi,
    Flask,
    Django,
}

impl Default for BackendFramework {
    fn default() -> Self {
        BackendFramework::Express
    }
}

/// Package manager type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Bun,
    Yarn,
    Pnpm,
    Cargo,
    Pip,
    Uv,
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Bun => write!(f, "bun"),
            PackageManager::Yarn => write!(f, "yarn"),
            PackageManager::Pnpm => write!(f, "pnpm"),
            PackageManager::Cargo => write!(f, "cargo"),
            PackageManager::Pip => write!(f, "pip"),
            PackageManager::Uv => write!(f, "uv"),
        }
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        PackageManager::Npm
    }
}

impl PackageManager {
    pub fn install_command(&self) -> &str {
        match self {
            PackageManager::Npm => "npm install",
            PackageManager::Bun => "bun install",
            PackageManager::Yarn => "yarn install",
            PackageManager::Pnpm => "pnpm install",
            PackageManager::Cargo => "cargo build",
            PackageManager::Pip => "pip install -r requirements.txt",
            PackageManager::Uv => "uv pip install -r requirements.txt",
        }
    }
}

/// Frontend runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendRuntime {
    pub framework: FrontendFramework,
    pub dev_port: u16,
    pub dev_host: String,
    pub dev_command: String,
    pub build_command: String,
    pub output_dir: String,
    pub has_hmr: bool,
    pub proxy_config: Option<ProxyConfig>,
    pub entry_file: String,
}

impl Default for FrontendRuntime {
    fn default() -> Self {
        Self {
            framework: FrontendFramework::Vanilla,
            dev_port: 5173,
            dev_host: "localhost".to_string(),
            dev_command: "npm run dev".to_string(),
            build_command: "npm run build".to_string(),
            output_dir: "dist".to_string(),
            has_hmr: true,
            proxy_config: None,
            entry_file: "index.html".to_string(),
        }
    }
}

/// Backend runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendRuntime {
    pub framework: BackendFramework,
    pub port: u16,
    pub host: String,
    pub dev_command: String,
    pub build_command: String,
    pub start_command: Option<String>,
    pub binary_name: Option<String>,
    pub entry_file: String,
    pub api_base: String,
}

impl Default for BackendRuntime {
    fn default() -> Self {
        Self {
            framework: BackendFramework::Express,
            port: 3000,
            host: "0.0.0.0".to_string(),
            dev_command: "cargo run".to_string(),
            build_command: "cargo build --release".to_string(),
            start_command: None,
            binary_name: None,
            entry_file: "src/main.rs".to_string(),
            api_base: "/api".to_string(),
        }
    }
}

/// Proxy configuration for fullstack projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub target: String,
    pub change_origin: bool,
    pub ws: bool,
}

/// Fullstack runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullstackRuntime {
    pub frontend_dev_command: String,
    pub backend_dev_command: String,
    pub frontend_port: u16,
    pub backend_port: u16,
    pub concurrent: bool,
    pub proxy: Option<ProxyConfig>,
}

impl Default for FullstackRuntime {
    fn default() -> Self {
        Self {
            frontend_dev_command: "npm run dev".to_string(),
            backend_dev_command: "cargo run".to_string(),
            frontend_port: 5173,
            backend_port: 3000,
            concurrent: true,
            proxy: Some(ProxyConfig {
                enabled: true,
                target: "http://localhost:3000".to_string(),
                change_origin: true,
                ws: true,
            }),
        }
    }
}

/// Dependency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    pub install_command: String,
    pub package_manager: PackageManager,
    pub install_timeout_secs: u64,
}

impl Default for DependencyConfig {
    fn default() -> Self {
        Self {
            install_command: "npm install".to_string(),
            package_manager: PackageManager::Npm,
            install_timeout_secs: 300,
        }
    }
}

/// Project runtime configuration - main structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRuntimeConfig {
    /// Runtime type
    pub runtime_type: RuntimeType,
    
    /// Frontend configuration (for frontend/fullstack projects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend: Option<FrontendRuntime>,
    
    /// Backend configuration (for backend/fullstack projects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<BackendRuntime>,
    
    /// Fullstack configuration (for fullstack projects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fullstack: Option<FullstackRuntime>,
    
    /// Dependency configuration
    pub dependencies: DependencyConfig,
    
    /// Project root directory
    #[serde(skip)]
    pub project_root: Option<PathBuf>,
    
    /// Security warnings from analysis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_warnings: Option<Vec<String>>,
    
    /// Analysis timestamp
    pub analyzed_at: String,
    
    /// Original analysis data (for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_analysis: Option<RawAnalysis>,
}

impl Default for ProjectRuntimeConfig {
    fn default() -> Self {
        Self {
            runtime_type: RuntimeType::Unknown,
            frontend: None,
            backend: None,
            fullstack: None,
            dependencies: DependencyConfig::default(),
            project_root: None,
            security_warnings: None,
            analyzed_at: chrono::Utc::now().to_rfc3339(),
            raw_analysis: None,
        }
    }
}

/// Raw analysis data for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAnalysis {
    pub file_tree: String,
    pub package_json: Option<String>,
    pub cargo_toml: Option<String>,
    pub vite_config: Option<String>,
    pub detected_files: Vec<String>,
}

/// Security check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheckResult {
    pub is_safe: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl SecurityCheckResult {
    pub fn safe() -> Self {
        Self {
            is_safe: true,
            warnings: vec![],
            errors: vec![],
        }
    }
    
    pub fn unsafe_with_errors(errors: Vec<String>) -> Self {
        Self {
            is_safe: false,
            warnings: vec![],
            errors,
        }
    }
}

/// Preset configurations for common project types
pub fn get_preset_configs() -> HashMap<RuntimeType, ProjectRuntimeConfig> {
    let mut configs = HashMap::new();
    
    // React + Vite
    configs.insert(RuntimeType::ReactVite, ProjectRuntimeConfig {
        runtime_type: RuntimeType::ReactVite,
        frontend: Some(FrontendRuntime {
            framework: FrontendFramework::React,
            dev_port: 5173,
            dev_host: "localhost".to_string(),
            dev_command: "bun run dev".to_string(),
            build_command: "bun run build".to_string(),
            output_dir: "dist".to_string(),
            has_hmr: true,
            proxy_config: None,
            entry_file: "index.html".to_string(),
        }),
        backend: None,
        fullstack: None,
        dependencies: DependencyConfig {
            install_command: "bun install".to_string(),
            package_manager: PackageManager::Bun,
            install_timeout_secs: 180,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    // Vanilla HTML - uses built-in static server (no Python dependency)
    configs.insert(RuntimeType::VanillaHtml, ProjectRuntimeConfig {
        runtime_type: RuntimeType::VanillaHtml,
        frontend: Some(FrontendRuntime {
            framework: FrontendFramework::Vanilla,
            dev_port: 8000,
            dev_host: "localhost".to_string(),
            dev_command: "(built-in static server)".to_string(), // Handled by GUI, not external command
            build_command: "".to_string(),
            output_dir: ".".to_string(),
            has_hmr: false,
            proxy_config: None,
            entry_file: "index.html".to_string(),
        }),
        backend: None,
        fullstack: None,
        dependencies: DependencyConfig {
            install_command: "".to_string(),
            package_manager: PackageManager::Npm,
            install_timeout_secs: 0,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    // Rust Backend
    configs.insert(RuntimeType::RustBackend, ProjectRuntimeConfig {
        runtime_type: RuntimeType::RustBackend,
        frontend: None,
        backend: Some(BackendRuntime {
            framework: BackendFramework::Axum,
            port: 3000,
            host: "0.0.0.0".to_string(),
            dev_command: "cargo run".to_string(),
            build_command: "cargo build --release".to_string(),
            start_command: Some("./target/release/app".to_string()),
            binary_name: Some("app".to_string()),
            entry_file: "src/main.rs".to_string(),
            api_base: "/api".to_string(),
        }),
        fullstack: None,
        dependencies: DependencyConfig {
            install_command: "cargo build".to_string(),
            package_manager: PackageManager::Cargo,
            install_timeout_secs: 600,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    // Fullstack React + Rust
    configs.insert(RuntimeType::FullstackReactRust, ProjectRuntimeConfig {
        runtime_type: RuntimeType::FullstackReactRust,
        frontend: Some(FrontendRuntime {
            framework: FrontendFramework::React,
            dev_port: 5173,
            dev_host: "localhost".to_string(),
            dev_command: "bun run dev".to_string(),
            build_command: "bun run build".to_string(),
            output_dir: "dist".to_string(),
            has_hmr: true,
            proxy_config: Some(ProxyConfig {
                enabled: true,
                target: "http://localhost:3000".to_string(),
                change_origin: true,
                ws: true,
            }),
            entry_file: "index.html".to_string(),
        }),
        backend: Some(BackendRuntime {
            framework: BackendFramework::Axum,
            port: 3000,
            host: "0.0.0.0".to_string(),
            dev_command: "cargo run".to_string(),
            build_command: "cargo build --release".to_string(),
            start_command: None,
            binary_name: None,
            entry_file: "src/main.rs".to_string(),
            api_base: "/api".to_string(),
        }),
        fullstack: Some(FullstackRuntime {
            frontend_dev_command: "bun run dev".to_string(),
            backend_dev_command: "cargo run".to_string(),
            frontend_port: 5173,
            backend_port: 3000,
            concurrent: true,
            proxy: Some(ProxyConfig {
                enabled: true,
                target: "http://localhost:3000".to_string(),
                change_origin: true,
                ws: true,
            }),
        }),
        dependencies: DependencyConfig {
            install_command: "bun install && cargo build".to_string(),
            package_manager: PackageManager::Bun,
            install_timeout_secs: 600,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    // Node Express
    configs.insert(RuntimeType::NodeExpress, ProjectRuntimeConfig {
        runtime_type: RuntimeType::NodeExpress,
        frontend: None,
        backend: Some(BackendRuntime {
            framework: BackendFramework::Express,
            port: 3000,
            host: "0.0.0.0".to_string(),
            dev_command: "bun run dev".to_string(),
            build_command: "".to_string(),
            start_command: Some("bun run start".to_string()),
            binary_name: None,
            entry_file: "src/index.js".to_string(),
            api_base: "/api".to_string(),
        }),
        fullstack: None,
        dependencies: DependencyConfig {
            install_command: "bun install".to_string(),
            package_manager: PackageManager::Bun,
            install_timeout_secs: 180,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    // Python FastAPI
    configs.insert(RuntimeType::PythonFastapi, ProjectRuntimeConfig {
        runtime_type: RuntimeType::PythonFastapi,
        frontend: None,
        backend: Some(BackendRuntime {
            framework: BackendFramework::FastApi,
            port: 8000,
            host: "0.0.0.0".to_string(),
            dev_command: "uvicorn main:app --reload".to_string(),
            build_command: "".to_string(),
            start_command: Some("uvicorn main:app --host 0.0.0.0 --port 8000".to_string()),
            binary_name: None,
            entry_file: "main.py".to_string(),
            api_base: "/api".to_string(),
        }),
        fullstack: None,
        dependencies: DependencyConfig {
            install_command: "pip install -r requirements.txt".to_string(),
            package_manager: PackageManager::Pip,
            install_timeout_secs: 300,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    // Tauri + React
    configs.insert(RuntimeType::TauriReact, ProjectRuntimeConfig {
        runtime_type: RuntimeType::TauriReact,
        frontend: Some(FrontendRuntime {
            framework: FrontendFramework::React,
            dev_port: 5173,
            dev_host: "localhost".to_string(),
            dev_command: "bun run tauri dev".to_string(),
            build_command: "bun run tauri build".to_string(),
            output_dir: "src-tauri/target/release".to_string(),
            has_hmr: true,
            proxy_config: None,
            entry_file: "index.html".to_string(),
        }),
        backend: None,
        fullstack: None,
        dependencies: DependencyConfig {
            install_command: "bun install && cargo build".to_string(),
            package_manager: PackageManager::Bun,
            install_timeout_secs: 600,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    // Node Tool
    configs.insert(RuntimeType::NodeTool, ProjectRuntimeConfig {
        runtime_type: RuntimeType::NodeTool,
        frontend: None,
        backend: None,
        fullstack: None,
        dependencies: DependencyConfig {
            install_command: "bun install".to_string(),
            package_manager: PackageManager::Bun,
            install_timeout_secs: 180,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    // Rust CLI
    configs.insert(RuntimeType::RustCli, ProjectRuntimeConfig {
        runtime_type: RuntimeType::RustCli,
        frontend: None,
        backend: None,
        fullstack: None,
        dependencies: DependencyConfig {
            install_command: "cargo build --release".to_string(),
            package_manager: PackageManager::Cargo,
            install_timeout_secs: 600,
        },
        project_root: None,
        security_warnings: None,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        raw_analysis: None,
    });
    
    configs
}

/// Get preset config for a runtime type
pub fn get_preset_config(runtime_type: &RuntimeType) -> Option<ProjectRuntimeConfig> {
    get_preset_configs().get(runtime_type).cloned()
}
