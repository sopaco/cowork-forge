// Import Configuration - Configuration for importing existing projects

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for importing an existing project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    /// Path to the project to import
    pub project_path: PathBuf,
    /// Options for artifact generation
    pub artifact_options: ArtifactOptions,
    /// Whether to create .cowork-v2 directory
    pub initialize_cowork_dir: bool,
    /// Project name (if different from directory name)
    pub project_name: Option<String>,
}

impl ImportConfig {
    /// Create a new import configuration
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            artifact_options: ArtifactOptions::default(),
            initialize_cowork_dir: true,
            project_name: None,
        }
    }

    /// Set artifact options
    pub fn with_artifact_options(mut self, options: ArtifactOptions) -> Self {
        self.artifact_options = options;
        self
    }

    /// Disable .cowork-v2 initialization
    pub fn skip_cowork_init(mut self) -> Self {
        self.initialize_cowork_dir = false;
        self
    }

    /// Set custom project name
    pub fn with_project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }
}

/// Options for which artifacts to generate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactOptions {
    /// Generate idea.md
    pub generate_idea: bool,
    /// Generate prd.md
    pub generate_prd: bool,
    /// Generate design.md
    pub generate_design: bool,
    /// Generate plan.md
    pub generate_plan: bool,
    /// Scan README.md for content
    pub scan_readme: bool,
    /// Scan docs/ directory
    pub scan_docs: bool,
    /// Scan code comments
    pub scan_comments: bool,
}

impl Default for ArtifactOptions {
    fn default() -> Self {
        Self {
            generate_idea: true,
            generate_prd: true,
            generate_design: true,
            generate_plan: true,
            scan_readme: true,
            scan_docs: true,
            scan_comments: false,
        }
    }
}

/// Result of importing a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Whether import was successful
    pub success: bool,
    /// Project name
    pub project_name: String,
    /// Path to the project
    pub project_path: PathBuf,
    /// Artifacts that were generated
    pub generated_artifacts: Vec<String>,
    /// Detected technologies
    pub detected_technologies: Vec<String>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ImportResult {
    /// Create a successful import result
    pub fn success(
        project_name: String,
        project_path: PathBuf,
        artifacts: Vec<String>,
        technologies: Vec<String>,
    ) -> Self {
        Self {
            success: true,
            project_name,
            project_path,
            generated_artifacts: artifacts,
            detected_technologies: technologies,
            error: None,
        }
    }

    /// Create a failed import result
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            project_name: String::new(),
            project_path: PathBuf::new(),
            generated_artifacts: Vec::new(),
            detected_technologies: Vec::new(),
            error: Some(message.into()),
        }
    }
}

/// Preview of what will be imported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    /// Project name
    pub name: String,
    /// Project path
    pub path: PathBuf,
    /// Detected technologies
    pub technologies: Vec<String>,
    /// Files that will be scanned
    pub files_to_scan: Vec<String>,
    /// Artifacts that will be generated
    pub artifacts_to_generate: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

impl ImportPreview {
    /// Create a preview from an existing path
    pub fn from_path(path: &PathBuf) -> Self {
        use crate::importer::project_analyzer::analyze_project;
        
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        match analyze_project(path) {
            Ok(analysis) => {
                let technologies = analysis.technologies
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();

                let files_to_scan: Vec<String> = std::fs::read_dir(path)
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                let artifacts_to_generate = vec![
                    "idea.md".to_string(),
                    "prd.md".to_string(),
                    "design.md".to_string(),
                    "plan.md".to_string(),
                ];

                let warnings = if analysis.documentation.is_empty() {
                    vec!["No README.md found - limited documentation available".to_string()]
                } else {
                    Vec::new()
                };

                Self {
                    name,
                    path: path.clone(),
                    technologies,
                    files_to_scan,
                    artifacts_to_generate,
                    warnings,
                }
            }
            Err(e) => Self {
                name,
                path: path.clone(),
                technologies: Vec::new(),
                files_to_scan: Vec::new(),
                artifacts_to_generate: Vec::new(),
                warnings: vec![format!("Analysis error: {}", e)],
            },
        }
    }
}
