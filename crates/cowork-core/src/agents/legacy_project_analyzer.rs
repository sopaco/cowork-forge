// Legacy Project Analyzer Agent - Analyzes existing projects and generates Artifacts

use crate::instructions::LEGACY_PROJECT_ANALYZER_INSTRUCTION;
use crate::tools::{
    ScanProjectTool,
    DetectTechStackTool,
    ReadProjectFileTool,
    ListProjectDirectoryTool,
    SaveArtifactTool,
};
use adk_agent::LlmAgentBuilder;
use adk_core::{IncludeContents, Llm};
use anyhow::Result;
use std::sync::Arc;

// ============================================================================
// Legacy Project Analyzer Agent
// ============================================================================

/// Create a Legacy Project Analyzer agent
/// This agent is responsible for:
/// 1. Analyzing existing project structure
/// 2. Detecting technology stack
/// 3. Extracting information from documentation
/// 4. Generating Artifacts (idea.md, prd.md, design.md, plan.md)
pub fn create_legacy_project_analyzer(model: Arc<dyn Llm>) -> Result<Arc<dyn adk_core::Agent>> {
    let agent = LlmAgentBuilder::new("legacy_project_analyzer")
        .instruction(LEGACY_PROJECT_ANALYZER_INSTRUCTION)
        .model(model)
        // Project analysis tools
        .tool(Arc::new(ScanProjectTool))
        .tool(Arc::new(DetectTechStackTool))
        .tool(Arc::new(ReadProjectFileTool))
        .tool(Arc::new(ListProjectDirectoryTool))
        // Artifact generation tools
        .tool(Arc::new(SaveArtifactTool))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

/// Create a Legacy Project Analyzer agent with iteration context
/// This version supports the iteration-based workflow
pub fn create_legacy_project_analyzer_with_id(
    model: Arc<dyn Llm>,
    iteration_id: String,
) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace {ITERATION_ID} placeholder in instruction
    let instruction = LEGACY_PROJECT_ANALYZER_INSTRUCTION
        .replace("{ITERATION_ID}", &iteration_id);

    let agent = LlmAgentBuilder::new("legacy_project_analyzer")
        .instruction(&instruction)
        .model(model)
        // Project analysis tools
        .tool(Arc::new(ScanProjectTool))
        .tool(Arc::new(DetectTechStackTool))
        .tool(Arc::new(ReadProjectFileTool))
        .tool(Arc::new(ListProjectDirectoryTool))
        // Artifact generation tools
        .tool(Arc::new(SaveArtifactTool))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}

/// Create a Legacy Project Analyzer agent with project-specific context
/// This version replaces {project_path} and {artifact_options} placeholders
pub fn create_legacy_project_analyzer_with_context(
    model: Arc<dyn Llm>,
    project_path: String,
    artifact_options: String,
) -> Result<Arc<dyn adk_core::Agent>> {
    // Replace placeholders in instruction template
    let instruction = LEGACY_PROJECT_ANALYZER_INSTRUCTION
        .replace("{project_path}", &project_path)
        .replace("{artifact_options}", &artifact_options);

    let agent = LlmAgentBuilder::new("legacy_project_analyzer")
        .instruction(&instruction)
        .model(model)
        // Project analysis tools
        .tool(Arc::new(ScanProjectTool))
        .tool(Arc::new(DetectTechStackTool))
        .tool(Arc::new(ReadProjectFileTool))
        .tool(Arc::new(ListProjectDirectoryTool))
        // Artifact generation tools
        .tool(Arc::new(SaveArtifactTool))
        .include_contents(IncludeContents::None)
        .build()?;

    Ok(Arc::new(agent))
}
