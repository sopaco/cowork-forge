// Artifact Generator - Generates Artifacts from project analysis

use crate::importer::project_analyzer::{ProjectAnalysis, DocType};
use serde::{Deserialize, Serialize};

/// Generated artifact content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    pub filename: String,
    pub content: String,
    pub artifact_type: ArtifactType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactType {
    Idea,
    PRD,
    Design,
    Plan,
}

/// Options for artifact generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactGenerationOptions {
    pub generate_idea: bool,
    pub generate_prd: bool,
    pub generate_design: bool,
    pub generate_plan: bool,
    pub scan_readme: bool,
    pub scan_docs: bool,
}

impl Default for ArtifactGenerationOptions {
    fn default() -> Self {
        Self {
            generate_idea: true,
            generate_prd: true,
            generate_design: true,
            generate_plan: true,
            scan_readme: true,
            scan_docs: true,
        }
    }
}

/// Generate artifacts from project analysis
pub fn generate_artifacts(
    analysis: &ProjectAnalysis,
    options: &ArtifactGenerationOptions,
) -> Vec<GeneratedArtifact> {
    let mut artifacts = Vec::new();

    if options.generate_idea {
        artifacts.push(generate_idea(analysis));
    }

    if options.generate_prd {
        artifacts.push(generate_prd(analysis));
    }

    if options.generate_design {
        artifacts.push(generate_design(analysis));
    }

    if options.generate_plan {
        artifacts.push(generate_plan(analysis));
    }

    artifacts
}

/// Generate idea.md from analysis
fn generate_idea(analysis: &ProjectAnalysis) -> GeneratedArtifact {
    use std::path::Path;
    
    // Try to get more context from README
    let readme_content = analysis.documentation.iter()
        .find(|d| d.file_type == DocType::Readme)
        .and_then(|d| {
            // Build full path from project path and documentation file path
            let readme_path = Path::new(&analysis.project_path).join(&d.path);
            std::fs::read_to_string(readme_path).ok()
        });

    let tech_list = if analysis.technologies.is_empty() {
        "Not specifically detected (generic project)".to_string()
    } else {
        analysis.technologies.iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    
    // Try to read package.json or Cargo.toml for dependencies
    let deps_info = read_dependencies_info(&analysis.project_path);

    let content = format!(r#"# Project Idea

## Project Overview
This is an existing project named **{}** that has been imported into Cowork Forge.

## Background
This project was originally created outside of Cowork Forge and has been analyzed to extract its structure and purpose.

{}

## Target Users
Users who work with the {} technology stack.

## Key Features
Based on the project analysis, the following key aspects are present:

{}

## Technical Stack
| Component | Technology |
|-----------|------------|
{}

{}

## Project Structure

### Root Files
{}

### Directories
{}

### Entry Points
{}

## Architecture
{}

## Next Steps
This idea will be passed to the PRD team for requirement analysis. Consider enhancing this document with:
- Specific business goals
- Target user personas  
- Key performance metrics
"#,
        analysis.name,
        if let Some(ref readme) = readme_content {
            let preview = readme.lines().take(50).collect::<Vec<_>>().join("\n");
            if preview.len() > 100 {
                format!("\n### README Content Preview\n```\n{}...\n```\n", 
                    &preview[..preview.len().min(2000)])
            } else {
                format!("\n### README Content\n```\n{}\n```\n", preview)
            }
        } else {
            "".to_string()
        },
        tech_list,
        format_features(analysis),
        if analysis.technologies.is_empty() {
            "| Not Detected | Generic Project |".to_string()
        } else {
            format_tech_stack(analysis)
        },
        deps_info,
        if analysis.structure.root_files.is_empty() {
            "None".to_string()
        } else {
            analysis.structure.root_files.iter()
                .map(|f| format!("- `{}`", f))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if analysis.structure.directories.is_empty() {
            "None".to_string()
        } else {
            analysis.structure.directories.iter()
                .map(|d| format!("- `{}/` ({} files) - {}", 
                    d.path, 
                    d.file_count, 
                    d.purpose.as_deref().unwrap_or("unknown purpose")))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if analysis.structure.entry_points.is_empty() {
            "No explicit entry points detected".to_string()
        } else {
            format_entry_points(analysis)
        },
        format_architecture(analysis),
    );

    GeneratedArtifact {
        filename: "idea.md".to_string(),
        content,
        artifact_type: ArtifactType::Idea,
    }
}

/// Read dependencies information from package.json or Cargo.toml
fn read_dependencies_info(project_path: &str) -> String {
    use std::path::Path;
    let path = Path::new(project_path);
    
    // Try package.json
    let pkg_json = path.join("package.json");
    if pkg_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut deps = Vec::new();
                
                if let Some(deps_obj) = json.get("dependencies").and_then(|d| d.as_object()) {
                    for (name, version) in deps_obj.iter().take(20) {
                        let ver = version.as_str().unwrap_or("*");
                        deps.push(format!("- {} @ {}", name, ver));
                    }
                }
                
                if let Some(dev_deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
                    deps.push("\n**Dev Dependencies:**".to_string());
                    for (name, version) in dev_deps.iter().take(10) {
                        let ver = version.as_str().unwrap_or("*");
                        deps.push(format!("- {} @ {}", name, ver));
                    }
                }
                
                if !deps.is_empty() {
                    return format!("### Dependencies (from package.json)\n{}\n", deps.join("\n"));
                }
            }
        }
    }
    
    // Try Cargo.toml
    let cargo_toml = path.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            let mut deps = Vec::new();
            let mut in_deps = false;
            
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "[dependencies]" {
                    in_deps = true;
                    continue;
                }
                if trimmed.starts_with('[') && trimmed != "[dependencies]" {
                    in_deps = false;
                }
                if in_deps && trimmed.contains('=') && !trimmed.starts_with('#') {
                    deps.push(format!("- {}", trimmed));
                    if deps.len() > 20 {
                        break;
                    }
                }
            }
            
            if !deps.is_empty() {
                return format!("### Dependencies (from Cargo.toml)\n{}\n", deps.join("\n"));
            }
        }
    }
    
    String::new()
}

/// Generate prd.md from analysis
fn generate_prd(analysis: &ProjectAnalysis) -> GeneratedArtifact {
    let content = format!(r#"# Product Requirements Document

## Project Overview
**Project Name:** {}
**Type:** Existing Project (Imported)

## Functional Requirements

### Core Features

Based on the project structure and technology stack, the following features are identified:

{}

### User Interactions

The project supports the following types of interactions based on its architecture:

- **Frontend:** {}
- **Backend:** {}

### Data Models

The project uses the following data management approach:
- **Database:** {}

## Non-Functional Requirements

### Performance
- The project should maintain responsive performance

### Security
- Security considerations should be reviewed based on the technology stack

### Scalability
- The architecture supports: {}

## Technical Constraints

- Technology constraints from detected stack: {}
- Dependencies: {}

## Open Questions

1. What are the specific business requirements this project addresses?
2. Are there any existing issues or bugs that need attention?
3. What is the deployment environment?
"#,
        analysis.name,
        format_feature_list(analysis),
        if analysis.technologies.iter().any(|t| t.category == crate::importer::project_analyzer::TechCategory::Frontend) { "Web interface" } else { "N/A" },
        if analysis.technologies.iter().any(|t| t.category == crate::importer::project_analyzer::TechCategory::Backend) { "API endpoints" } else { "N/A" },
        if analysis.technologies.iter().any(|t| t.category == crate::importer::project_analyzer::TechCategory::Database) { "Database integration detected" } else { "No database detected" },
        if analysis.architecture_hints.is_distributed { "Distributed/Microservices architecture" } else if analysis.architecture_hints.is_monolithic { "Monolithic architecture" } else { "To be determined" },
        analysis.technologies.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", "),
        "See requirements.txt, Cargo.toml, or package.json for specific dependencies",
    );

    GeneratedArtifact {
        filename: "prd.md".to_string(),
        content,
        artifact_type: ArtifactType::PRD,
    }
}

/// Generate design.md from analysis
fn generate_design(analysis: &ProjectAnalysis) -> GeneratedArtifact {
    let content = format!(r#"# Technical Design

## Architecture Overview

{}

## Technology Stack

| Component | Technology | Version/Notes |
|-----------|------------|---------------|
{}
{}

## Directory Structure

```
project/
{}
```

## Key Modules

### Entry Points
{}

### Configuration Files
{}

## Architecture Pattern

The project appears to follow a **{}** architecture.

## Layer Breakdown

{}
"#,
        analysis.architecture_hints.pattern.as_deref().unwrap_or("Standard project architecture"),
        format_tech_stack_table(analysis),
        if analysis.architecture_hints.is_distributed { "| Distributed | Yes | Multiple services |" } else { "" },
        format_directory_tree(&analysis.structure.directories),
        format_entry_points_detail(analysis),
        format_config_files(&analysis.structure.root_files),
        analysis.architecture_hints.pattern.as_deref().unwrap_or("standard"),
        format_layers(&analysis.architecture_hints.layers),
    );

    GeneratedArtifact {
        filename: "design.md".to_string(),
        content,
        artifact_type: ArtifactType::Design,
    }
}

/// Generate plan.md from analysis
fn generate_plan(analysis: &ProjectAnalysis) -> GeneratedArtifact {
    let content = format!(r#"# Implementation Plan

## Project: {}

This is an existing project that has been imported into Cowork Forge. The following plan outlines potential improvements and next steps.

## Phase 1: Project Assessment
- [ ] Review existing code structure
- [ ] Verify build process works correctly
- [ ] Run existing tests (if available)
- [ ] Identify any immediate issues

## Phase 2: Documentation
- [ ] Update README with current project status
- [ ] Document architecture decisions
- [ ] Review and update API documentation (if applicable)

## Phase 3: Testing & Quality
- [ ] Add unit tests if missing
- [ ] Set up CI/CD if not present
- [ ] Review code quality and style

## Phase 4: Feature Development
- [ ] Prioritize features based on business needs
- [ ] Implement new features using Cowork Forge workflow

## Phase 5: Deployment
- [ ] Review deployment configuration
- [ ] Set up production environment
- [ ] Document deployment process

## Technology Migration (Optional)

If modernization is needed:
- [ ] Evaluate technology upgrades
- [ ] Plan migration path
- [ ] Execute phased migration
"#,
        analysis.name,
    );

    GeneratedArtifact {
        filename: "plan.md".to_string(),
        content,
        artifact_type: ArtifactType::Plan,
    }
}

// Helper functions

fn format_features(analysis: &ProjectAnalysis) -> String {
    let mut features = Vec::new();
    
    // Generate features based on technologies
    for tech in &analysis.technologies {
        match tech.category {
            crate::importer::project_analyzer::TechCategory::Frontend => {
                features.push(format!("- Frontend development with {}", tech.name));
            }
            crate::importer::project_analyzer::TechCategory::Backend => {
                features.push(format!("- Backend API development with {}", tech.name));
            }
            crate::importer::project_analyzer::TechCategory::Database => {
                features.push("- Data persistence layer".to_string());
            }
            _ => {}
        }
    }

    if features.is_empty() {
        features.push("- Project features to be documented".to_string());
    }

    features.join("\n")
}

fn format_tech_stack(analysis: &ProjectAnalysis) -> String {
    analysis.technologies
        .iter()
        .map(|t| format!("| {} | {} |", t.name, format!("{:?}", t.category).to_lowercase()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_tech_stack_table(analysis: &ProjectAnalysis) -> String {
    analysis.technologies
        .iter()
        .map(|t| format!("| {} | {:?} | - |", t.name, t.category))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_entry_points(analysis: &ProjectAnalysis) -> String {
    if analysis.structure.entry_points.is_empty() {
        "No explicit entry points detected".to_string()
    } else {
        analysis.structure.entry_points
            .iter()
            .map(|e| format!("- {} ({:?})", e.path, e.file_type))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn format_entry_points_detail(analysis: &ProjectAnalysis) -> String {
    if analysis.structure.entry_points.is_empty() {
        "No entry points identified".to_string()
    } else {
        analysis.structure.entry_points
            .iter()
            .map(|e| format!("### {} ({:?})\nLocation: `{}`\n", e.path, e.file_type, e.path))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn format_architecture(analysis: &ProjectAnalysis) -> String {
    if let Some(pattern) = &analysis.architecture_hints.pattern {
        format!("Architecture Pattern: {}", pattern)
    } else {
        "Architecture pattern to be determined".to_string()
    }
}

fn format_feature_list(analysis: &ProjectAnalysis) -> String {
    let mut features = Vec::new();
    
    // Add feature entries based on structure
    if !analysis.structure.entry_points.iter().any(|e| matches!(e.file_type, crate::importer::project_analyzer::EntryPointType::Frontend)) {
        features.push("1. Frontend interface implementation".to_string());
    }
    if !analysis.structure.entry_points.iter().any(|e| matches!(e.file_type, crate::importer::project_analyzer::EntryPointType::Backend)) {
        features.push("2. Backend API implementation".to_string());
    }
    
    if features.is_empty() {
        features.push("1. Feature 1 (to be documented)".to_string());
        features.push("2. Feature 2 (to be documented)".to_string());
    }
    
    features.join("\n")
}

fn format_config_files(root_files: &[String]) -> String {
    let config_extensions = ["json", "toml", "yaml", "yml", "env", "config"];
    root_files
        .iter()
        .filter(|f| {
            f.contains('.') && config_extensions.iter().any(|ext| f.ends_with(&format!(".{}", ext)))
        })
        .map(|f| format!("- `{}`", f))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_directory_tree(directories: &[crate::importer::project_analyzer::DirectoryInfo]) -> String {
    directories
        .iter()
        .map(|d| format!("├── {} ({} files)", d.path, d.file_count))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_layers(layers: &[String]) -> String {
    layers
        .iter()
        .enumerate()
        .map(|(i, l)| format!("{}. {}", i + 1, l))
        .collect::<Vec<_>>()
        .join("\n")
}
