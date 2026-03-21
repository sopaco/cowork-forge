// Project Analyzer - Analyzes existing projects to extract structure and metadata

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Represents a detected technology in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTechnology {
    pub name: String,
    pub version: Option<String>,
    pub category: TechCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TechCategory {
    Frontend,
    Backend,
    Database,
    BuildTool,
    Test,
    Lint,
    Container,
    Other,
}

/// Analysis result of an existing project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    /// Project root path
    pub project_path: String,
    /// Project name (derived from directory name or package.json)
    pub name: String,
    /// Detected technologies
    pub technologies: Vec<DetectedTechnology>,
    /// Project structure
    pub structure: ProjectStructure,
    /// Documentation files found
    pub documentation: Vec<DocumentationFile>,
    /// Architecture hints
    pub architecture_hints: ArchitectureHints,
}

/// Project directory structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStructure {
    pub root_files: Vec<String>,
    pub directories: Vec<DirectoryInfo>,
    pub entry_points: Vec<EntryPoint>,
}

/// Information about a directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub purpose: Option<String>,
    pub file_count: usize,
}

/// Main entry points detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub path: String,
    pub file_type: EntryPointType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryPointType {
    Frontend,
    Backend,
    CLI,
    Config,
}

/// Documentation file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationFile {
    pub path: String,
    pub title: Option<String>,
    pub file_type: DocType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocType {
    Readme,
    Changelog,
    Contributing,
    License,
    API,
    Architecture,
    Other,
}

/// Hints about the architecture
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArchitectureHints {
    pub pattern: Option<String>,
    pub layers: Vec<String>,
    pub is_monolithic: bool,
    pub is_distributed: bool,
}

/// Analyze an existing project
pub fn analyze_project(project_path: &Path) -> Result<ProjectAnalysis, String> {
    if !project_path.exists() {
        return Err(format!("Project path does not exist: {:?}", project_path));
    }

    let project_path_str = project_path.to_string_lossy().to_string();
    let name = derive_project_name(project_path);
    let technologies = detect_technologies(project_path);
    let structure = analyze_structure(project_path);
    let documentation = find_documentation(project_path);
    let architecture_hints = infer_architecture(&structure, &technologies);

    Ok(ProjectAnalysis {
        project_path: project_path_str,
        name,
        technologies,
        structure,
        documentation,
        architecture_hints,
    })
}

/// Derive project name from directory or package files
fn derive_project_name(project_path: &Path) -> String {
    // Try package.json first
    let pkg_json = project_path.join("package.json");
    if pkg_json.exists() {
        if let Ok(content) = fs::read_to_string(&pkg_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                    return name.to_string();
                }
            }
        }
    }

    // Try Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = fs::read_to_string(&cargo_toml) {
            for line in content.lines() {
                if line.trim().starts_with("name") && line.contains('=') {
                    let parts: Vec<&str> = line.split('=').collect();
                    if parts.len() >= 2 {
                        let name = parts[1].trim().trim_matches('"').trim_matches('\'');
                        return name.to_string();
                    }
                }
            }
        }
    }

    // Fall back to directory name
    project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_project")
        .to_string()
}

/// Detect technologies from configuration files
fn detect_technologies(project_path: &Path) -> Vec<DetectedTechnology> {
    let mut technologies = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Helper to add technology without duplicates
    let add_tech = |techs: &mut Vec<_>, seen: &mut HashSet<String>, name: &str, category: TechCategory| {
        if seen.insert(name.to_string()) {
            techs.push(DetectedTechnology {
                name: name.to_string(),
                version: None,
                category,
            });
        }
    };

    // package.json
    let pkg_json = project_path.join("package.json");
    if pkg_json.exists() {
        if let Ok(content) = fs::read_to_string(&pkg_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // Process dependencies
                if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                    for (key, _value) in deps {
                        // Frontend frameworks
                        if key == "react" {
                            add_tech(&mut technologies, &mut seen, "React", TechCategory::Frontend);
                        } else if key == "vue" {
                            add_tech(&mut technologies, &mut seen, "Vue", TechCategory::Frontend);
                        } else if key == "@angular/core" {
                            add_tech(&mut technologies, &mut seen, "Angular", TechCategory::Frontend);
                        } else if key == "svelte" {
                            add_tech(&mut technologies, &mut seen, "Svelte", TechCategory::Frontend);
                        }
                        // Backend frameworks
                        else if key == "express" {
                            add_tech(&mut technologies, &mut seen, "Express", TechCategory::Backend);
                        } else if key == "fastify" {
                            add_tech(&mut technologies, &mut seen, "Fastify", TechCategory::Backend);
                        } else if key == "@nestjs/core" {
                            add_tech(&mut technologies, &mut seen, "NestJS", TechCategory::Backend);
                        } else if key == "koa" {
                            add_tech(&mut technologies, &mut seen, "Koa", TechCategory::Backend);
                        }
                        // Databases
                        else if key == "mongoose" || key == "prisma" || key.contains("sql") {
                            add_tech(&mut technologies, &mut seen, "Database", TechCategory::Database);
                        }
                        // Build tools
                        else if key == "vite" || key == "webpack" || key == "esbuild" {
                            add_tech(&mut technologies, &mut seen, "Build Tool", TechCategory::BuildTool);
                        }
                    }
                }

                // Process devDependencies
                if let Some(dev_deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
                    for (key, _value) in dev_deps {
                        // Frontend frameworks (dev dependencies)
                        if key == "react" {
                            add_tech(&mut technologies, &mut seen, "React", TechCategory::Frontend);
                        } else if key == "vue" {
                            add_tech(&mut technologies, &mut seen, "Vue", TechCategory::Frontend);
                        }
                        // Build tools (common in devDependencies)
                        else if key == "vite" || key == "webpack" || key == "esbuild" || key == "@vitejs/plugin-react" {
                            add_tech(&mut technologies, &mut seen, "Build Tool", TechCategory::BuildTool);
                        }
                        // Testing frameworks
                        else if key == "jest" || key == "vitest" || key == "playwright" {
                            add_tech(&mut technologies, &mut seen, "Test", TechCategory::Test);
                        }
                        // Linting
                        else if key == "eslint" || key == "prettier" {
                            add_tech(&mut technologies, &mut seen, "Lint", TechCategory::Lint);
                        }
                    }
                }
            }
        }
    }

    // Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = fs::read_to_string(&cargo_toml) {
            if content.contains("axum") {
                add_tech(&mut technologies, &mut seen, "Axum", TechCategory::Backend);
            }
            if content.contains("actix-web") {
                add_tech(&mut technologies, &mut seen, "Actix-Web", TechCategory::Backend);
            }
            if content.contains("serde") {
                add_tech(&mut technologies, &mut seen, "Serde", TechCategory::Backend);
            }
            if content.contains("tokio") {
                add_tech(&mut technologies, &mut seen, "Tokio", TechCategory::Backend);
            }
            if content.contains("leptos") || content.contains("yew") || content.contains("dodrio") {
                add_tech(&mut technologies, &mut seen, "WebAssembly", TechCategory::Frontend);
            }
        }
    }

    // Python
    let requirements = project_path.join("requirements.txt");
    if requirements.exists() {
        if let Ok(content) = fs::read_to_string(&requirements) {
            for line in content.lines() {
                let dep = line.split('=').next().unwrap_or(line).trim().to_lowercase();
                if dep.starts_with("django") {
                    add_tech(&mut technologies, &mut seen, "Django", TechCategory::Backend);
                } else if dep.starts_with("flask") {
                    add_tech(&mut technologies, &mut seen, "Flask", TechCategory::Backend);
                } else if dep.starts_with("fastapi") {
                    add_tech(&mut technologies, &mut seen, "FastAPI", TechCategory::Backend);
                } else if dep.starts_with("sqlalchemy") {
                    add_tech(&mut technologies, &mut seen, "SQLAlchemy", TechCategory::Database);
                }
            }
        }
    }

    // Docker
    if project_path.join("Dockerfile").exists() {
        add_tech(&mut technologies, &mut seen, "Docker", TechCategory::Container);
    }
    if project_path.join("docker-compose.yml").exists() || project_path.join("docker-compose.yaml").exists() {
        add_tech(&mut technologies, &mut seen, "Docker Compose", TechCategory::Container);
    }

    technologies
}

/// Analyze project directory structure
fn analyze_structure(project_path: &Path) -> ProjectStructure {
    let mut structure = ProjectStructure::default();

    // Scan root directory
    if let Ok(entries) = fs::read_dir(project_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();

            if path.is_dir() {
                // Count files in subdirectory
                let file_count = fs::read_dir(&path)
                    .map(|e| e.filter_map(|x| x.ok()).count())
                    .unwrap_or(0);

                let purpose = infer_directory_purpose(&name);
                structure.directories.push(DirectoryInfo {
                    path: name.clone(),
                    purpose,
                    file_count,
                });
            } else {
                structure.root_files.push(name);
            }
        }
    }

    // Find entry points
    let entry_patterns = [
        ("index.html", EntryPointType::Frontend),
        ("main.js", EntryPointType::Frontend),
        ("main.ts", EntryPointType::Frontend),
        ("main.tsx", EntryPointType::Frontend),
        ("main.rs", EntryPointType::Backend),
        ("lib.rs", EntryPointType::Backend),
        ("app.py", EntryPointType::Backend),
        ("main.py", EntryPointType::Backend),
        ("index.js", EntryPointType::Frontend),
        ("App.tsx", EntryPointType::Frontend),
    ];

    for (filename, entry_type) in &entry_patterns {
        let entry_path = project_path.join(filename);
        if entry_path.exists() {
            structure.entry_points.push(EntryPoint {
                path: filename.to_string(),
                file_type: entry_type.clone(),
                description: format!("Main entry point for {:?}", entry_type),
            });
        }
    }

    structure
}

/// Infer directory purpose from name
fn infer_directory_purpose(name: &str) -> Option<String> {
    let purposes = HashMap::from([
        ("src", "Source code"),
        ("lib", "Library code"),
        ("tests", "Test files"),
        ("test", "Test files"),
        ("docs", "Documentation"),
        ("doc", "Documentation"),
        ("examples", "Example code"),
        ("example", "Example code"),
        ("scripts", "Build/utility scripts"),
        ("config", "Configuration files"),
        ("configs", "Configuration files"),
        ("public", "Static assets"),
        ("assets", "Static assets"),
        ("dist", "Build output"),
        ("build", "Build output"),
        ("target", "Build output (Rust)"),
        ("node_modules", "Dependencies"),
        ("__pycache__", "Python cache"),
    ]);

    purposes.get(name).map(|s| s.to_string())
}

/// Find documentation files
fn find_documentation(project_path: &Path) -> Vec<DocumentationFile> {
    let mut docs = Vec::new();

    let doc_files = [
        ("README.md", DocType::Readme),
        ("CHANGELOG.md", DocType::Changelog),
        ("CONTRIBUTING.md", DocType::Contributing),
        ("LICENSE", DocType::License),
        ("LICENSE.md", DocType::License),
        ("API.md", DocType::API),
        ("ARCHITECTURE.md", DocType::Architecture),
    ];

    for (filename, doc_type) in &doc_files {
        let path = project_path.join(filename);
        if path.exists() {
            let title = extract_title_from_markdown(&path);
            docs.push(DocumentationFile {
                path: filename.to_string(),
                title,
                file_type: doc_type.clone(),
            });
        }
    }

    docs
}

/// Extract title from markdown file
fn extract_title_from_markdown(path: &Path) -> Option<String> {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return Some(trimmed[2..].trim().to_string());
            }
        }
    }
    None
}

/// Infer architecture from structure and technologies
fn infer_architecture(structure: &ProjectStructure, technologies: &[DetectedTechnology]) -> ArchitectureHints {
    let mut hints = ArchitectureHints::default();

    // Check for frontend/backend split
    let has_frontend = technologies.iter().any(|t| t.category == TechCategory::Frontend);
    let has_backend = technologies.iter().any(|t| t.category == TechCategory::Backend);
    let has_database = technologies.iter().any(|t| t.category == TechCategory::Database);

    if has_frontend && has_backend {
        hints.pattern = Some("Full-stack".to_string());
        hints.layers.push("Frontend".to_string());
        hints.layers.push("Backend".to_string());
        if has_database {
            hints.layers.push("Database".to_string());
        }
    } else if has_frontend {
        hints.pattern = Some("Single-page Application".to_string());
        hints.layers.push("Frontend".to_string());
    } else if has_backend {
        hints.pattern = Some("Backend API".to_string());
        hints.layers.push("Backend".to_string());
        if has_database {
            hints.layers.push("Database".to_string());
        }
    }

    // Check for microservices indicators
    let has_docker = technologies.iter().any(|t| t.name == "Docker" || t.name == "Docker Compose");
    if has_docker && structure.directories.iter().any(|d| d.path == "services" || d.path == "microservices") {
        hints.pattern = Some("Microservices".to_string());
        hints.is_distributed = true;
    }

    // Check if monolithic
    if hints.layers.len() >= 2 && !hints.is_distributed {
        hints.is_monolithic = true;
    }

    hints
}
