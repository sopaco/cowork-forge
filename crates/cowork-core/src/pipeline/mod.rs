// Main pipeline - Cowork Forge workflow

use crate::agents::*;
use crate::llm::*;
use adk_agent::SequentialAgent;
use adk_core::Agent;
use anyhow::Result;
use std::sync::Arc;

/// Create the main Cowork Forge pipeline for new projects
/// 
/// This assembles all agents into a sequential workflow:
/// 1. IdeaAgent - Capture user's idea
/// 2. PRD Loop - Requirements + Features (Actor-Critic)
/// 3. Design Loop - Architecture (Actor-Critic)
/// 4. Plan Loop - Implementation plan (Actor-Critic)
/// 5. Coding Loop - Code implementation (Actor-Critic)
/// 6. Check Agent - Quality assurance
/// 7. Delivery Agent - Final report
pub fn create_cowork_pipeline(config: &ModelConfig, session_id: &str) -> Result<Arc<dyn Agent>> {
    // Create LLM client
    let llm = create_llm_client(&config.llm)?;

    // Create all agents with session context
    let idea_agent = create_idea_agent(llm.clone(), session_id)?;
    let prd_loop = create_prd_loop(llm.clone(), session_id)?;
    let design_loop = create_design_loop(llm.clone(), session_id)?;
    let plan_loop = create_plan_loop(llm.clone(), session_id)?;
    let coding_loop = create_coding_loop(llm.clone(), session_id)?;
    let check_agent = create_check_agent(llm.clone(), session_id)?;
    let delivery_agent = create_delivery_agent(llm, session_id)?;

    // Assemble into SequentialAgent
    let pipeline = SequentialAgent::new(
        "cowork_forge_pipeline",
        vec![
            idea_agent,
            prd_loop as Arc<dyn Agent>,
            design_loop as Arc<dyn Agent>,
            plan_loop as Arc<dyn Agent>,
            coding_loop as Arc<dyn Agent>,
            check_agent,
            delivery_agent,
        ],
    );

    Ok(Arc::new(pipeline))
}

/// Create a resume pipeline (skip Idea stage and completed stages)
/// 
/// This function determines which stage to resume from by checking
/// what data files exist in the base session
pub fn create_resume_pipeline(
    config: &ModelConfig,
    session_id: &str,
    base_session_id: &str,
) -> Result<Arc<dyn Agent>> {
    use crate::storage::*;
    
    let _llm = create_llm_client(&config.llm)?;

    // Determine which stage to start from based on existing data files in base session
    // NOTE: load_* returns default empty structs when files don't exist, so we must check file existence.
    // IMPORTANT: Check from the most advanced stage to the earliest to resume from the furthest progress point.
    let start_stage = if has_code_files(base_session_id)? {
        // Code files exist → Coding is complete, resume from Check
        "check"
    } else if has_implementation_plan(base_session_id)?
        && has_design_spec(base_session_id)?
        && has_requirements(base_session_id)?
    {
        // PRD, Design, Plan exist (but no code files yet) → Resume from Coding
        "coding"
    } else if has_design_spec(base_session_id)? && has_requirements(base_session_id)? {
        // PRD, Design exist → Resume from Plan
        "plan"
    } else if has_requirements(base_session_id)? {
        // PRD exists → Resume from Design
        "design"
    } else {
        // Nothing exists or only idea.md → Start from PRD
        "prd"
    };

    println!("📍 Resuming from: {} stage", start_stage);

    // Use create_partial_pipeline to start from the determined stage
    create_partial_pipeline(config, session_id, base_session_id, start_stage)
}

/// Create a partial pipeline starting from a specific stage (for revert)
/// 
/// Useful for:
/// - Modifying requirements (start from PRD)
/// - Redesigning architecture (start from Design)
/// - Replanning (start from Plan)
/// - Recoding (start from Coding)
pub fn create_partial_pipeline(
    config: &ModelConfig,
    session_id: &str,
    _base_session_id: &str,
    start_stage: &str,
) -> Result<Arc<dyn Agent>> {
    let llm = create_llm_client(&config.llm)?;

    let agents: Vec<Arc<dyn Agent>> = match start_stage {
        "prd" => {
            vec![
                create_prd_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_design_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_plan_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_coding_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_check_agent(llm.clone(), session_id)?,
                create_delivery_agent(llm, session_id)?,
            ]
        }
        "design" => {
            vec![
                create_design_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_plan_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_coding_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_check_agent(llm.clone(), session_id)?,
                create_delivery_agent(llm, session_id)?,
            ]
        }
        "plan" => {
            vec![
                create_plan_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_coding_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_check_agent(llm.clone(), session_id)?,
                create_delivery_agent(llm, session_id)?,
            ]
        }
        "coding" => {
            vec![
                create_coding_loop(llm.clone(), session_id)? as Arc<dyn Agent>,
                create_check_agent(llm.clone(), session_id)?,
                create_delivery_agent(llm, session_id)?,
            ]
        }
        "check" => {
            vec![
                create_check_agent(llm.clone(), session_id)?,
                create_delivery_agent(llm, session_id)?,
            ]
        }
        "delivery" => {
            vec![create_delivery_agent(llm, session_id)?]
        }
        _ => {
            anyhow::bail!("Unknown stage: {}. Valid stages: prd, design, plan, coding, check, delivery", start_stage)
        }
    };

    let pipeline = SequentialAgent::new(
        format!("cowork_partial_pipeline_{}", start_stage),
        agents,
    );

    Ok(Arc::new(pipeline))
}

/// Create a modify pipeline for incremental changes
/// 
/// This is a new pipeline designed for incremental updates:
/// 1. Change Triage - Analyze the change and determine scope
/// 2. Artifact Patch - Update affected artifacts (PRD/Design/Plan as needed)
/// 3. Code Patch - Generate code changes (patches, not full rewrite)
/// 4. Check - Verify changes
/// 5. Delivery - Generate change report
pub fn create_modify_pipeline(
    config: &ModelConfig,
    session_id: &str,
    base_session_id: &str,
) -> Result<Arc<dyn Agent>> {
    let llm = create_llm_client(&config.llm)?;

    // Create modify pipeline with specialized agents
    let agents: Vec<Arc<dyn Agent>> = vec![
        create_change_triage_agent(llm.clone(), session_id, base_session_id)?,
        create_code_patch_agent(llm.clone(), session_id, base_session_id)?,
        create_check_agent(llm.clone(), session_id)?,
        create_modify_delivery_agent(llm, session_id, base_session_id)?,
    ];

    let pipeline = SequentialAgent::new(
        format!("cowork_modify_pipeline_{}", session_id),
        agents,
    );

    Ok(Arc::new(pipeline))
}

// Placeholder for new modify-specific agents
// These are now implemented below
fn create_change_triage_agent(
    llm: Arc<dyn adk_core::Llm>,
    session_id: &str,
    base_session_id: &str,
) -> Result<Arc<dyn Agent>> {
    use crate::instructions::CHANGE_TRIAGE_INSTRUCTION;
    use crate::tools::*;
    use adk_agent::LlmAgentBuilder;
    use adk_core::IncludeContents;
    
    let session = session_id.to_string();
    
    let agent = LlmAgentBuilder::new("change_triage_agent")
        .instruction(CHANGE_TRIAGE_INSTRUCTION)
        .model(llm)
        .tool(Arc::new(GetRequirementsTool::new(base_session_id.to_string())))
        .tool(Arc::new(GetDesignTool::new(base_session_id.to_string())))
        .tool(Arc::new(GetPlanTool::new(base_session_id.to_string())))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(LoadChangeRequestTool::new(session.clone())))
        .tool(Arc::new(SaveChangeRequestTool::new(session.clone())))
        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;
    
    Ok(Arc::new(agent))
}

fn create_code_patch_agent(
    llm: Arc<dyn adk_core::Llm>,
    session_id: &str,
    _base_session_id: &str,
) -> Result<Arc<dyn Agent>> {
    use crate::instructions::CODE_PATCH_INSTRUCTION;
    use crate::tools::*;
    use adk_agent::LlmAgentBuilder;
    use adk_core::IncludeContents;
    
    let session = session_id.to_string();
    
    let agent = LlmAgentBuilder::new("code_patch_agent")
        .instruction(CODE_PATCH_INSTRUCTION)
        .model(llm)
        .tool(Arc::new(LoadChangeRequestTool::new(session.clone())))
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(WriteFileTool))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(UpdateTaskStatusTool::new(session.clone())))
        .tool(Arc::new(UpdateFeatureStatusTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;
    
    Ok(Arc::new(agent))
}

fn create_modify_delivery_agent(
    llm: Arc<dyn adk_core::Llm>,
    session_id: &str,
    _base_session_id: &str,
) -> Result<Arc<dyn Agent>> {
    use crate::instructions::MODIFY_DELIVERY_INSTRUCTION;
    use crate::tools::*;
    use adk_agent::LlmAgentBuilder;
    use adk_core::IncludeContents;
    
    let session = session_id.to_string();
    
    let agent = LlmAgentBuilder::new("modify_delivery_agent")
        .instruction(MODIFY_DELIVERY_INSTRUCTION)
        .model(llm)
        .tool(Arc::new(LoadChangeRequestTool::new(session.clone())))
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(LoadFeedbackHistoryTool::new(session.clone())))
        .tool(Arc::new(SaveDeliveryReportTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;
    
    Ok(Arc::new(agent))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        // Test that we can create a config
        let config = ModelConfig {
            llm: LlmConfig {
                api_base_url: "http://localhost:8000/v1".to_string(),
                api_key: "test-key".to_string(),
                model_name: "gpt-4".to_string(),
            },
        };

        assert_eq!(config.llm.model_name, "gpt-4");
    }
}
