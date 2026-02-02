// Main pipeline - Cowork Forge workflow

use crate::agents::*;
use crate::agents::ResilientAgent;
use crate::llm::*;
use crate::interaction::InteractiveBackend;
use adk_core::{Agent, EventStream, InvocationContext, Result as AdkResult};
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;

/// StageExecutor - Executes stages sequentially without propagating escalate
/// 
/// Unlike SequentialAgent, this executor isolates each stage's escalate flag,
/// allowing LoopAgents to use ExitLoopTool without terminating the entire workflow.
pub struct StageExecutor {
    name: String,
    stages: Vec<(String, Arc<dyn Agent>)>,
}

impl StageExecutor {
    pub fn new(name: impl Into<String>, stages: Vec<(String, Arc<dyn Agent>)>) -> Self {
        Self {
            name: name.into(),
            stages,
        }
    }
}

#[async_trait]
impl Agent for StageExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Stage-based workflow executor"
    }

    fn sub_agents(&self) -> &[Arc<dyn Agent>] {
        &[] // Stages are not exposed as sub-agents
    }

    async fn run(&self, ctx: Arc<dyn InvocationContext>) -> AdkResult<EventStream> {
        let stages = self.stages.clone();
        
        let s: Pin<Box<dyn Stream<Item = AdkResult<adk_core::Event>> + Send>> = Box::pin(async_stream::stream! {
            for (stage_name, agent) in stages {
                println!("\n🔄 Starting stage: {}", stage_name);
                
                // Run the stage agent
                let mut stage_stream = agent.run(ctx.clone()).await?;
                
                // Forward all events from this stage
                while let Some(result) = stage_stream.next().await {
                    match result {
                        Ok(event) => {
                            // Append content to history for next stages
                            if let Some(ref content) = event.llm_response.content {
                                ctx.session().append_to_history(content.clone());
                            }
                            
                            // NOTE: We deliberately ignore event.actions.escalate here
                            // This allows LoopAgents to use ExitLoopTool without affecting other stages
                            
                            yield Ok(event);
                        }
                        Err(e) => {
                            // If a stage errors, stop the entire workflow
                            yield Err(e);
                            return;
                        }
                    }
                }
                
                println!("✅ Stage completed: {}", stage_name);
            }
            
            println!("\n🎉 All stages completed successfully");
        });

        Ok(s)
    }
}

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
pub fn create_cowork_pipeline(
    config: &ModelConfig,
    session_id: &str,
    interaction: Arc<dyn crate::interaction::InteractiveBackend>,
) -> Result<Arc<dyn Agent>> {
    // Create LLM client
    let llm = create_llm_client(&config.llm)?;

    // Create all agents with session context
    let idea_agent = create_idea_agent_with_interaction(
        llm.clone(), 
        session_id, 
        Some(interaction.clone())
    )?;
    let prd_loop = create_prd_loop(llm.clone(), session_id, interaction.clone())?;
    let design_loop = create_design_loop(llm.clone(), session_id, interaction.clone())?;
    let plan_loop = create_plan_loop(llm.clone(), session_id, interaction.clone())?;
    let coding_loop = create_coding_loop(llm.clone(), session_id, interaction.clone())?;
    let check_agent = create_check_agent(llm.clone(), session_id)?;
    let delivery_agent = create_delivery_agent(llm, session_id)?;

    // Assemble into StageExecutor (replaces SequentialAgent)
    // Each stage can now use ExitLoopTool without affecting other stages
    let pipeline = StageExecutor::new(
        "cowork_forge_pipeline",
        vec![
            ("idea".to_string(), idea_agent),
            ("prd".to_string(), prd_loop),
            ("design".to_string(), design_loop),
            ("plan".to_string(), plan_loop),
            ("coding".to_string(), coding_loop),
            ("check".to_string(), check_agent),
            ("delivery".to_string(), delivery_agent),
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
    interaction: Arc<dyn crate::interaction::InteractiveBackend>,
) -> Result<Arc<dyn Agent>> {
    use crate::storage::*;
    
    let _llm = create_llm_client(&config.llm)?;

    // Determine which stage to start from based on existing data files in base session
    let start_stage = if has_code_files(base_session_id)? {
        "check"
    } else if has_implementation_plan(base_session_id)?
        && has_design_spec(base_session_id)?
        && has_requirements(base_session_id)?
    {
        "coding"
    } else if has_design_spec(base_session_id)? && has_requirements(base_session_id)? {
        "plan"
    } else if has_requirements(base_session_id)? {
        "design"
    } else {
        "prd"
    };

    println!("📍 Resuming from: {} stage", start_stage);

    create_partial_pipeline(config, session_id, base_session_id, start_stage, interaction)
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
    interaction: Arc<dyn crate::interaction::InteractiveBackend>,
) -> Result<Arc<dyn Agent>> {
    let llm = create_llm_client(&config.llm)?;

    let stages: Vec<(String, Arc<dyn Agent>)> = match start_stage {
        "prd" => {
            vec![
                ("prd".to_string(), create_prd_loop(llm.clone(), session_id, interaction.clone())?),
                ("design".to_string(), create_design_loop(llm.clone(), session_id, interaction.clone())?),
                ("plan".to_string(), create_plan_loop(llm.clone(), session_id, interaction.clone())?),
                ("coding".to_string(), create_coding_loop(llm.clone(), session_id, interaction.clone())?),
                ("check".to_string(), create_check_agent(llm.clone(), session_id)?),
                ("delivery".to_string(), create_delivery_agent(llm, session_id)?),
            ]
        }
        "design" => {
            vec![
                ("design".to_string(), create_design_loop(llm.clone(), session_id, interaction.clone())?),
                ("plan".to_string(), create_plan_loop(llm.clone(), session_id, interaction.clone())?),
                ("coding".to_string(), create_coding_loop(llm.clone(), session_id, interaction.clone())?),
                ("check".to_string(), create_check_agent(llm.clone(), session_id)?),
                ("delivery".to_string(), create_delivery_agent(llm, session_id)?),
            ]
        }
        "plan" => {
            vec![
                ("plan".to_string(), create_plan_loop(llm.clone(), session_id, interaction.clone())?),
                ("coding".to_string(), create_coding_loop(llm.clone(), session_id, interaction.clone())?),
                ("check".to_string(), create_check_agent(llm.clone(), session_id)?),
                ("delivery".to_string(), create_delivery_agent(llm, session_id)?),
            ]
        }
        "coding" => {
            vec![
                ("coding".to_string(), create_coding_loop(llm.clone(), session_id, interaction.clone())?),
                ("check".to_string(), create_check_agent(llm.clone(), session_id)?),
                ("delivery".to_string(), create_delivery_agent(llm, session_id)?),
            ]
        }
        "check" => {
            vec![
                ("check".to_string(), create_check_agent(llm.clone(), session_id)?),
                ("delivery".to_string(), create_delivery_agent(llm, session_id)?),
            ]
        }
        "delivery" => {
            vec![("delivery".to_string(), create_delivery_agent(llm, session_id)?)]
        }
        _ => {
            anyhow::bail!("Unknown stage: {}. Valid stages: prd, design, plan, coding, check, delivery", start_stage)
        }
    };

    let pipeline = StageExecutor::new(
        format!("cowork_partial_pipeline_{}", start_stage),
        stages,
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
    interaction: Arc<dyn crate::interaction::InteractiveBackend>,
) -> Result<Arc<dyn Agent>> {
    let llm = create_llm_client(&config.llm)?;

    // Create modify pipeline with specialized agents
    let agents: Vec<Arc<dyn Agent>> = vec![
        create_change_triage_agent(llm.clone(), session_id, base_session_id)?,
        create_code_patch_agent(llm.clone(), session_id, base_session_id, interaction.clone())?,
        create_check_agent(llm.clone(), session_id)?,
        create_modify_delivery_agent(llm, session_id, base_session_id)?,
    ];

    let pipeline = StageExecutor::new(
        format!("cowork_modify_pipeline_{}", session_id),
        vec![
            ("triage".to_string(), agents[0].clone()),
            ("patch".to_string(), agents[1].clone()),
            ("check".to_string(), agents[2].clone()),
            ("delivery".to_string(), agents[3].clone()),
        ],
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
    interaction: Arc<dyn crate::interaction::InteractiveBackend>,
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
        .tool(Arc::new(DeleteFileTool))
        .tool(Arc::new(DeleteDirectoryTool))
        .tool(Arc::new(RunCommandTool))
        .tool(Arc::new(UpdateTaskStatusTool::new(session.clone())))
        .tool(Arc::new(UpdateFeatureStatusTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;
    
    // Wrap with ResilientAgent for error handling
    Ok(Arc::new(ResilientAgent::new(Arc::new(agent), interaction)))
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
