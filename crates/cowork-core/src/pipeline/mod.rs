// Main pipeline - Cowork Forge workflow

use crate::agents::*;
use crate::llm::*;
use adk_agent::SequentialAgent;
use adk_core::Agent;
use anyhow::Result;
use std::sync::Arc;

/// Create the main Cowork Forge pipeline
/// 
/// This assembles all agents into a sequential workflow:
/// 1. IdeaAgent - Capture user's idea
/// 2. PRD Loop - Requirements + Features (Actor-Critic)
/// 3. Design Loop - Architecture (Actor-Critic)
/// 4. Plan Loop - Implementation plan (Actor-Critic)
/// 5. Coding Loop - Code implementation (Actor-Critic)
/// 6. Check Agent - Quality assurance
/// 7. Delivery Agent - Final report
pub fn create_cowork_pipeline(config: &ModelConfig) -> Result<Arc<dyn Agent>> {
    // Create LLM client
    let llm = create_llm_client(&config.llm)?;

    // Create all agents
    let idea_agent = create_idea_agent(llm.clone())?;
    let prd_loop = create_prd_loop(llm.clone())?;
    let design_loop = create_design_loop(llm.clone())?;
    let plan_loop = create_plan_loop(llm.clone())?;
    let coding_loop = create_coding_loop(llm.clone())?;
    let check_agent = create_check_agent(llm.clone())?;
    let delivery_agent = create_delivery_agent(llm)?;

    // Assemble into SequentialAgent
    let pipeline = SequentialAgent::new(
        "cowork_forge_pipeline",
        vec![
            idea_agent,
            prd_loop as Arc<dyn Agent>,  // Cast LoopAgent to Agent
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
/// This function intelligently determines which stage to resume from
/// by checking what data files already exist.
pub fn create_resume_pipeline(config: &ModelConfig) -> Result<Arc<dyn Agent>> {
    use crate::storage::*;
    use std::path::Path;
    
    let llm = create_llm_client(&config.llm)?;

    // Determine which stage to start from based on existing data
    let start_stage = if Path::new(".cowork/artifacts/delivery_report.md").exists() {
        // Everything is done
        anyhow::bail!("Project already completed. Check .cowork/artifacts/delivery_report.md");
    } else if Path::new(".cowork/data/plan.json").exists() 
            && Path::new(".cowork/data/design.json").exists() 
            && Path::new(".cowork/data/requirements.json").exists() {
        // PRD, Design, Plan exist → Resume from Coding
        "coding"
    } else if Path::new(".cowork/data/design.json").exists() 
            && Path::new(".cowork/data/requirements.json").exists() {
        // PRD, Design exist → Resume from Plan
        "plan"
    } else if Path::new(".cowork/data/requirements.json").exists() {
        // PRD exists → Resume from Design
        "design"
    } else {
        // Nothing exists or only idea.md → Start from PRD
        "prd"
    };

    println!("📍 Resuming from: {} stage", start_stage);

    // Use create_partial_pipeline to start from the determined stage
    create_partial_pipeline(config, start_stage)
}

/// Create a partial pipeline starting from a specific stage
/// 
/// Useful for:
/// - Modifying requirements (start from PRD)
/// - Redesigning architecture (start from Design)
/// - Replanning (start from Plan)
/// - Recoding (start from Coding)
pub fn create_partial_pipeline(
    config: &ModelConfig,
    start_stage: &str,
) -> Result<Arc<dyn Agent>> {
    let llm = create_llm_client(&config.llm)?;

    let agents: Vec<Arc<dyn Agent>> = match start_stage {
        "prd" => {
            vec![
                create_prd_loop(llm.clone())? as Arc<dyn Agent>,
                create_design_loop(llm.clone())? as Arc<dyn Agent>,
                create_plan_loop(llm.clone())? as Arc<dyn Agent>,
                create_coding_loop(llm.clone())? as Arc<dyn Agent>,
                create_check_agent(llm.clone())?,
                create_delivery_agent(llm)?,
            ]
        }
        "design" => {
            vec![
                create_design_loop(llm.clone())? as Arc<dyn Agent>,
                create_plan_loop(llm.clone())? as Arc<dyn Agent>,
                create_coding_loop(llm.clone())? as Arc<dyn Agent>,
                create_check_agent(llm.clone())?,
                create_delivery_agent(llm)?,
            ]
        }
        "plan" => {
            vec![
                create_plan_loop(llm.clone())? as Arc<dyn Agent>,
                create_coding_loop(llm.clone())? as Arc<dyn Agent>,
                create_check_agent(llm.clone())?,
                create_delivery_agent(llm)?,
            ]
        }
        "coding" => {
            vec![
                create_coding_loop(llm.clone())? as Arc<dyn Agent>,
                create_check_agent(llm.clone())?,
                create_delivery_agent(llm)?,
            ]
        }
        "check" => {
            vec![
                create_check_agent(llm.clone())?,
                create_delivery_agent(llm)?,
            ]
        }
        "delivery" => {
            vec![create_delivery_agent(llm)?]
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
