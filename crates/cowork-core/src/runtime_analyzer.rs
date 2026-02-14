// Runtime Analyzer Module
// Uses LLM to analyze project structure and generate runtime configuration

use crate::llm::config::LlmConfig;
use crate::project_runtime::{
    BackendFramework, BackendRuntime, DependencyConfig, FrontendFramework, FrontendRuntime,
    FullstackRuntime, PackageManager, ProjectRuntimeConfig, ProxyConfig, RuntimeType,
};
use crate::runtime_security::RuntimeSecurityChecker;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Project information collected for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub file_tree: String,
    pub package_json: Option<serde_json::Value>,
    pub cargo_toml: Option<String>,
    pub vite_config: Option<String>,
    pub detected_files: Vec<String>,
}

/// Analysis prompt template
const ANALYSIS_PROMPT: &str = r#"# 项目运行时分析

你是一个专业的软件工程师。请分析以下项目结构，推理出标准化的运行配置。

## ⚠️ 安全约束（必须遵守）

### ✅ 允许的操作
- 分析 package.json, Cargo.toml, vite.config.js 等配置文件
- 读取项目目录下的文件
- 生成标准开发命令（install, dev, build, test, lint, start）

### ❌ 禁止的操作
- 任何修改系统文件的命令
- 任何删除项目外文件的命令
- 任何下载并执行脚本的命令
- 任何格式化或磁盘操作命令
- 任何部署或发布命令

### 命令示例
✅ 正确: "npm install", "cargo run", "npm run dev", "uvicorn main:app --reload"
❌ 错误: "curl xxx | sh", "rm -rf /", "npm run eject", "npm publish"

## 项目文件结构
{file_tree}

## 关键配置文件

### package.json {package_json_exists}
{package_json_content}

### Cargo.toml {cargo_toml_exists}
{cargo_toml_content}

### vite.config.js {vite_config_exists}
{vite_config_content}

## 输出格式要求

请输出以下 JSON 格式的配置，不要任何其他内容：

```json
{
  "runtime_type": "react_vite | rust_backend | fullstack_react_rust | node_express | python_fastapi | vanilla_html | node_tool | rust_cli | tauri_react",
  "frontend": {
    "framework": "react | vue | solid | svelte | vanilla",
    "dev_port": 5173,
    "dev_host": "localhost",
    "dev_command": "bun run dev",
    "build_command": "bun run build",
    "output_dir": "dist",
    "has_hmr": true,
    "entry_file": "index.html",
    "proxy_config": {
      "enabled": true,
      "target": "http://localhost:3000",
      "change_origin": true,
      "ws": true
    }
  },
  "backend": {
    "framework": "axum | actix_web | express | fastify | nest | fastapi | flask | django",
    "port": 3000,
    "host": "0.0.0.0",
    "dev_command": "cargo run",
    "build_command": "cargo build --release",
    "start_command": "./target/release/app",
    "entry_file": "src/main.rs",
    "api_base": "/api"
  },
  "fullstack": {
    "frontend_dev_command": "bun run dev",
    "backend_dev_command": "cargo run",
    "frontend_port": 5173,
    "backend_port": 3000,
    "concurrent": true,
    "proxy": {
      "enabled": true,
      "target": "http://localhost:3000",
      "change_origin": true,
      "ws": true
    }
  },
  "dependencies": {
    "install_command": "bun install",
    "package_manager": "bun",
    "install_timeout_secs": 300
  },
  "security_notes": "安全分析说明"
}
```

请只输出 JSON，不要任何其他内容。如果无法确定配置，请返回 {{"runtime_type": "unknown"}}"#;

/// Runtime analyzer that uses LLM to analyze projects
pub struct RuntimeAnalyzer {
    llm_config: Option<LlmConfig>,
}

impl RuntimeAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self { llm_config: None }
    }

    /// Create with LLM config
    pub fn with_llm_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    /// Analyze project and generate runtime configuration
    pub async fn analyze(&self, workspace: &Path) -> Result<ProjectRuntimeConfig, String> {
        // 1. Collect project information
        let project_info = self.collect_project_info(workspace)?;

        // 2. Try LLM-based analysis first
        if let Some(config) = &self.llm_config {
            match self.analyze_with_llm(&project_info, config).await {
                Ok(config) => {
                    // 3. Validate with security checker
                    let checker =
                        RuntimeSecurityChecker::new().with_project_root(workspace.to_path_buf());
                    let security_result = checker.check_config(&config);

                    if !security_result.is_safe {
                        return Err(format!(
                            "安全检查失败: {}",
                            security_result.errors.join("; ")
                        ));
                    }

                    let mut config = config;
                    if !security_result.warnings.is_empty() {
                        config.security_warnings = Some(security_result.warnings);
                    }

                    return Ok(config);
                }
                Err(e) => {
                    tracing::warn!("LLM analysis failed: {}, falling back to heuristic", e);
                }
            }
        }

        // 4. Fallback to heuristic analysis
        self.analyze_with_heuristic(&project_info, workspace)
    }

    /// Collect project information for analysis
    fn collect_project_info(&self, workspace: &Path) -> Result<ProjectInfo, String> {
        let mut detected_files = Vec::new();
        let mut file_tree = String::new();

        // Collect file tree
        let walker = walkdir::WalkDir::new(workspace)
            .max_depth(4)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Skip hidden and build directories
                !name.starts_with('.')
                    && name != "node_modules"
                    && name != "target"
                    && name != "dist"
                    && name != "build"
                    && name != "__pycache__"
            });

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative = path.strip_prefix(workspace).unwrap_or(path);
            let relative_str = relative.to_string_lossy().to_string();

            if !relative_str.is_empty() && !relative_str.starts_with('.') {
                detected_files.push(relative_str.clone());

                let indent = relative_str.matches('/').count() + relative_str.matches('\\').count();
                let prefix = "  ".repeat(indent);
                let marker = if path.is_dir() { "📁 " } else { "📄 " };
                file_tree.push_str(&format!("{}{}{}\n", prefix, marker, relative_str));
            }
        }

        // Read package.json
        let package_json = self.read_json_file(&workspace.join("package.json"))?;

        // Read Cargo.toml
        let cargo_toml = fs::read_to_string(workspace.join("Cargo.toml")).ok();

        // Read vite.config
        let vite_config = self
            .read_config_file(&workspace.join("vite.config.js"))
            .or_else(|| self.read_config_file(&workspace.join("vite.config.ts")))
            .or_else(|| self.read_config_file(&workspace.join("vite.config.mjs")));

        Ok(ProjectInfo {
            file_tree,
            package_json,
            cargo_toml,
            vite_config,
            detected_files,
        })
    }

    /// Read and parse JSON file
    fn read_json_file(&self, path: &Path) -> Result<Option<serde_json::Value>, String> {
        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(json) => Ok(Some(json)),
                Err(e) => {
                    tracing::warn!("Failed to parse JSON {}: {}", path.display(), e);
                    Ok(None)
                }
            },
            Err(_) => Ok(None),
        }
    }

    /// Read config file content
    fn read_config_file(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    /// Analyze with LLM
    async fn analyze_with_llm(
        &self,
        project_info: &ProjectInfo,
        config: &LlmConfig,
    ) -> Result<ProjectRuntimeConfig, String> {
        // Build prompt
        let prompt = Self::build_prompt(project_info);

        // Call LLM
        let client = reqwest::Client::new();
        let response = client
            .post(&config.api_base_url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": config.model_name,
                "messages": [
                    {"role": "system", "content": "You are a software project analyzer. Output ONLY JSON, no other text."},
                    {"role": "user", "content": prompt}
                ],
                "temperature": 0.1,
                "max_tokens": 2000
            }))
            .send()
            .await
            .map_err(|e| format!("LLM request failed: {}", e))?;

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

        let content = response_json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or("Invalid LLM response format")?;

        // Parse JSON from response (might be wrapped in markdown)
        let json_str = content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse LLM output: {}\nOutput: {}", e, json_str))?;

        // Convert to ProjectRuntimeConfig
        Self::parse_llm_response(&parsed)
    }

    /// Build analysis prompt
    fn build_prompt(info: &ProjectInfo) -> String {
        let package_json_content = info
            .package_json
            .as_ref()
            .map(|j| serde_json::to_string_pretty(j).unwrap_or_default())
            .unwrap_or_else(|| "N/A".to_string());

        let package_json_exists = if info.package_json.is_some() {
            "(存在)"
        } else {
            "(不存在)"
        };

        let cargo_toml_content = info.cargo_toml.as_deref().unwrap_or("N/A");
        let cargo_toml_exists = if info.cargo_toml.is_some() {
            "(存在)"
        } else {
            "(不存在)"
        };

        let vite_config_content = info.vite_config.as_deref().unwrap_or("N/A");
        let vite_config_exists = if info.vite_config.is_some() {
            "(存在)"
        } else {
            "(不存在)"
        };

        ANALYSIS_PROMPT
            .replace("{file_tree}", &info.file_tree)
            .replace("{package_json_exists}", package_json_exists)
            .replace("{package_json_content}", &package_json_content)
            .replace("{cargo_toml_exists}", cargo_toml_exists)
            .replace("{cargo_toml_content}", cargo_toml_content)
            .replace("{vite_config_exists}", vite_config_exists)
            .replace("{vite_config_content}", vite_config_content)
    }

    /// Parse LLM response to ProjectRuntimeConfig
    fn parse_llm_response(json: &serde_json::Value) -> Result<ProjectRuntimeConfig, String> {
        let runtime_type_str = json
            .get("runtime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let runtime_type = match runtime_type_str {
            "react_vite" | "react" => RuntimeType::ReactVite,
            "react_cra" => RuntimeType::ReactCra,
            "vue_vite" => RuntimeType::VueVite,
            "vue" => RuntimeType::VueCli,
            "solid" => RuntimeType::SolidVite,
            "svelte" => RuntimeType::SvelteVite,
            "vanilla_html" | "vanilla" | "html" => RuntimeType::VanillaHtml,
            "rust_backend" | "rust" => RuntimeType::RustBackend,
            "node_express" | "express" => RuntimeType::NodeExpress,
            "node_fastify" | "fastify" => RuntimeType::NodeFastify,
            "node_nest" | "nest" => RuntimeType::NodeNest,
            "python_fastapi" | "fastapi" => RuntimeType::PythonFastapi,
            "python_flask" | "flask" => RuntimeType::PythonFlask,
            "python_django" | "django" => RuntimeType::PythonDjango,
            "fullstack_react_rust" => RuntimeType::FullstackReactRust,
            "fullstack_react_node" => RuntimeType::FullstackReactNode,
            "fullstack_vanilla_rust" => RuntimeType::FullstackVanillaRust,
            "fullstack_vanilla_node" => RuntimeType::FullstackVanillaNode,
            "tauri_react" => RuntimeType::TauriReact,
            "tauri" => RuntimeType::TauriVanilla,
            "electron_react" | "electron" => RuntimeType::ElectronReact,
            "node_tool" => RuntimeType::NodeTool,
            "rust_cli" => RuntimeType::RustCli,
            _ => RuntimeType::Unknown,
        };

        // Parse frontend config
        let frontend = json.get("frontend").and_then(|f| {
            Some(FrontendRuntime {
                framework: match f
                    .get("framework")
                    .and_then(|v| v.as_str())
                    .unwrap_or("vanilla")
                {
                    "react" => FrontendFramework::React,
                    "vue" => FrontendFramework::Vue,
                    "solid" => FrontendFramework::Solid,
                    "svelte" => FrontendFramework::Svelte,
                    _ => FrontendFramework::Vanilla,
                },
                dev_port: f.get("dev_port").and_then(|v| v.as_u64()).unwrap_or(5173) as u16,
                dev_host: f
                    .get("dev_host")
                    .and_then(|v| v.as_str())
                    .unwrap_or("localhost")
                    .to_string(),
                dev_command: f
                    .get("dev_command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("npm run dev")
                    .to_string(),
                build_command: f
                    .get("build_command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("npm run build")
                    .to_string(),
                output_dir: f
                    .get("output_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dist")
                    .to_string(),
                has_hmr: f.get("has_hmr").and_then(|v| v.as_bool()).unwrap_or(true),
                entry_file: f
                    .get("entry_file")
                    .and_then(|v| v.as_str())
                    .unwrap_or("index.html")
                    .to_string(),
                proxy_config: f.get("proxy_config").and_then(|p| {
                    Some(ProxyConfig {
                        enabled: p.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
                        target: p
                            .get("target")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        change_origin: p
                            .get("change_origin")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                        ws: p.get("ws").and_then(|v| v.as_bool()).unwrap_or(true),
                    })
                }),
            })
        });

        // Parse backend config
        let backend = json.get("backend").and_then(|b| {
            Some(BackendRuntime {
                framework: match b
                    .get("framework")
                    .and_then(|v| v.as_str())
                    .unwrap_or("express")
                {
                    "axum" => BackendFramework::Axum,
                    "actix_web" | "actix" => BackendFramework::ActixWeb,
                    "warp" => BackendFramework::Warp,
                    "rocket" => BackendFramework::Rocket,
                    "express" => BackendFramework::Express,
                    "fastify" => BackendFramework::Fastify,
                    "nest" => BackendFramework::Nest,
                    "fastapi" => BackendFramework::FastApi,
                    "flask" => BackendFramework::Flask,
                    "django" => BackendFramework::Django,
                    _ => BackendFramework::Express,
                },
                port: b.get("port").and_then(|v| v.as_u64()).unwrap_or(3000) as u16,
                host: b
                    .get("host")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0.0")
                    .to_string(),
                dev_command: b
                    .get("dev_command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("cargo run")
                    .to_string(),
                build_command: b
                    .get("build_command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                start_command: b
                    .get("start_command")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                binary_name: b
                    .get("binary_name")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                entry_file: b
                    .get("entry_file")
                    .and_then(|v| v.as_str())
                    .unwrap_or("src/main.rs")
                    .to_string(),
                api_base: b
                    .get("api_base")
                    .and_then(|v| v.as_str())
                    .unwrap_or("/api")
                    .to_string(),
            })
        });

        // Parse fullstack config
        let fullstack = json.get("fullstack").and_then(|fs| {
            Some(FullstackRuntime {
                frontend_dev_command: fs
                    .get("frontend_dev_command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("npm run dev")
                    .to_string(),
                backend_dev_command: fs
                    .get("backend_dev_command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("cargo run")
                    .to_string(),
                frontend_port: fs
                    .get("frontend_port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5173) as u16,
                backend_port: fs
                    .get("backend_port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3000) as u16,
                concurrent: fs
                    .get("concurrent")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                proxy: fs.get("proxy").and_then(|p| {
                    Some(ProxyConfig {
                        enabled: p.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                        target: p
                            .get("target")
                            .and_then(|v| v.as_str())
                            .unwrap_or("http://localhost:3000")
                            .to_string(),
                        change_origin: p
                            .get("change_origin")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                        ws: p.get("ws").and_then(|v| v.as_bool()).unwrap_or(true),
                    })
                }),
            })
        });

        // Parse dependencies
        let dependencies = json
            .get("dependencies")
            .and_then(|d| {
                Some(DependencyConfig {
                    install_command: d
                        .get("install_command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("npm install")
                        .to_string(),
                    package_manager: match d
                        .get("package_manager")
                        .and_then(|v| v.as_str())
                        .unwrap_or("npm")
                    {
                        "bun" => PackageManager::Bun,
                        "yarn" => PackageManager::Yarn,
                        "pnpm" => PackageManager::Pnpm,
                        "cargo" => PackageManager::Cargo,
                        "pip" => PackageManager::Pip,
                        "uv" => PackageManager::Uv,
                        _ => PackageManager::Npm,
                    },
                    install_timeout_secs: d
                        .get("install_timeout_secs")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(300),
                })
            })
            .unwrap_or_default();

        Ok(ProjectRuntimeConfig {
            runtime_type,
            frontend,
            backend,
            fullstack,
            dependencies,
            project_root: None,
            security_warnings: None,
            analyzed_at: chrono::Utc::now().to_rfc3339(),
            raw_analysis: None,
        })
    }

    /// Fallback heuristic analysis when LLM is not available
    fn analyze_with_heuristic(
        &self,
        info: &ProjectInfo,
        workspace: &Path,
    ) -> Result<ProjectRuntimeConfig, String> {
        let has_package_json = info.package_json.is_some();
        let has_cargo_toml = info.cargo_toml.is_some();

        // Check for Tauri
        let has_tauri = info
            .detected_files
            .iter()
            .any(|f| f.contains("src-tauri/tauri.conf.json"));

        // Determine runtime type based on file presence
        let runtime_type = if has_package_json && has_cargo_toml {
            // Fullstack: check if frontend is React
            if let Some(pkg) = &info.package_json {
                let deps = pkg.get("dependencies").or(pkg.get("devDependencies"));
                if deps.and_then(|d| d.get("react")).is_some() {
                    RuntimeType::FullstackReactRust
                } else {
                    RuntimeType::FullstackVanillaRust
                }
            } else {
                RuntimeType::FullstackReactRust
            }
        } else if has_package_json {
            // Node.js project - check framework
            if let Some(pkg) = &info.package_json {
                let deps = pkg.get("dependencies").or(pkg.get("devDependencies"));

                if has_tauri || info.detected_files.iter().any(|f| f.contains("src-tauri")) {
                    RuntimeType::TauriReact
                } else if deps.and_then(|d| d.get("react")).is_some() {
                    RuntimeType::ReactVite
                } else if deps.and_then(|d| d.get("vue")).is_some() {
                    RuntimeType::VueVite
                } else if deps.and_then(|d| d.get("@nestjs/core")).is_some() {
                    RuntimeType::NodeNest
                } else if deps.and_then(|d| d.get("express")).is_some() {
                    RuntimeType::NodeExpress
                } else if deps.and_then(|d| d.get("fastify")).is_some() {
                    RuntimeType::NodeFastify
                } else {
                    RuntimeType::NodeTool
                }
            } else {
                RuntimeType::NodeTool
            }
        } else if has_cargo_toml {
            // Rust project
            if has_tauri {
                RuntimeType::TauriVanilla
            } else if let Some(toml) = &info.cargo_toml {
                if toml.contains("[[bin]]") || toml.contains("[[lib]]") {
                    // Check for web framework
                    if toml.contains("axum")
                        || toml.contains("actix-web")
                        || toml.contains("warp")
                        || toml.contains("rocket")
                    {
                        RuntimeType::RustBackend
                    } else {
                        RuntimeType::RustCli
                    }
                } else {
                    RuntimeType::RustCli
                }
            } else {
                RuntimeType::RustCli
            }
        } else if info
            .detected_files
            .iter()
            .any(|f| f.ends_with("index.html"))
        {
            RuntimeType::VanillaHtml
        } else if info
            .detected_files
            .iter()
            .any(|f| f.ends_with("main.py") || f.ends_with("app.py"))
        {
            // Check for Python framework
            if let Some(_pkg_json) = &info.package_json {
                // It's a Python project with some JS tooling
                RuntimeType::NodeTool
            } else {
                RuntimeType::PythonFastapi // Default to FastAPI
            }
        } else {
            RuntimeType::Unknown
        };

        // Get preset config
        let mut config =
            crate::project_runtime::get_preset_config(&runtime_type).unwrap_or_else(|| {
                ProjectRuntimeConfig {
                    runtime_type: RuntimeType::Unknown,
                    ..Default::default()
                }
            });

        config.project_root = Some(workspace.to_path_buf());

        // Run security check
        let checker = RuntimeSecurityChecker::new().with_project_root(workspace.to_path_buf());
        let result = checker.check_config(&config);

        if !result.is_safe {
            return Err(format!("安全检查失败: {}", result.errors.join("; ")));
        }

        if !result.warnings.is_empty() {
            config.security_warnings = Some(result.warnings);
        }

        Ok(config)
    }
}

impl Default for RuntimeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Save runtime config to file
pub fn save_runtime_config(workspace: &Path, config: &ProjectRuntimeConfig) -> Result<(), String> {
    let config_dir = workspace.join(".cowork-v2");
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let config_file = config_dir.join("runtime_config.json");
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_file, json).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Load runtime config from file
pub fn load_runtime_config(workspace: &Path) -> Result<ProjectRuntimeConfig, String> {
    let config_file = workspace.join(".cowork-v2").join("runtime_config.json");

    let content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config: ProjectRuntimeConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    config.project_root = Some(workspace.to_path_buf());

    Ok(config)
}

/// Check if runtime config exists
pub fn has_runtime_config(workspace: &Path) -> bool {
    workspace
        .join(".cowork-v2")
        .join("runtime_config.json")
        .exists()
}
