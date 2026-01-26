use anyhow::Result;
use async_trait::async_trait;
use adk_rust::prelude::*;
use adk_rust::model::{OpenAIClient, OpenAIConfig};
use adk_rust::runner::{Runner, RunnerConfig};
use adk_rust::session::{InMemorySessionService, CreateRequest, SessionService, GetRequest};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

use crate::artifacts::*;
use crate::memory::ArtifactStore;
use crate::config::LlmConfig;
use crate::agents::{StageAgent, StageAgentContext, StageAgentResult};

/// Plan Agent - 基于 Design 生成实施计划
pub struct PlanAgent {
    model: Arc<OpenAIClient>,
    store: Arc<ArtifactStore>,
}

impl PlanAgent {
    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        let config = OpenAIConfig::compatible(
            llm_config.api_key.clone(),
            llm_config.api_base_url.clone(),
            llm_config.model_name.clone(),
        );
        
        tracing::info!("Creating Plan Agent with OpenAI-compatible client )");
        
        let model = OpenAIClient::new(config)?;

        Ok(Self {
            model: Arc::new(model),
            store,
        })
    }

    async fn generate_plan(&self, session_id: &str, design_artifact: &DesignDocArtifact) -> Result<PlanArtifact> {
        tracing::info!("PlanAgent: generating implementation plan for session {}", session_id);

        let output_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "c4": {
                    "type": "object",
                    "properties": {
                        "context": {"type": "array", "items": {"type": "string"}},
                        "containers": {"type": "array", "items": {"type": "string"}},
                        "components": {"type": "array", "items": {"type": "string"}},
                        "code": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["context", "containers", "components", "code"]
                },
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "pri": {"type": "string", "enum": ["p0", "p1", "p2"]},
                            "desc": {"type": "string"},
                            "deps": {"type": "array", "items": {"type": "string"}},
                            "out": {"type": "array", "items": {"type": "string"}}
                        },
                        "required": ["id", "pri", "desc", "deps", "out"]
                    }
                },
                "milestones": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "desc": {"type": "string"},
                            "done_when": {"type": "array", "items": {"type": "string"}}
                        },
                        "required": ["id", "desc", "done_when"]
                    }
                },
                "todo_list": {
                    "type": "object",
                    "properties": {
                        "items": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string"},
                                    "description": {"type": "string"},
                                    "status": {"type": "string", "enum": ["pending", "in_progress", "completed", "blocked"]},
                                    "related_requirements": {"type": "array", "items": {"type": "string"}},
                                    "related_files": {"type": "array", "items": {"type": "string"}},
                                    "verification_method": {"type": "string"}
                                },
                                "required": ["id", "description", "status", "related_requirements", "related_files", "verification_method"]
                            }
                        }
                    },
                    "required": ["items"]
                }
            },
            "required": ["c4", "tasks", "milestones", "todo_list"]
        });

        let context = format!(
            r#"Based on the following Design Document, create an implementation plan.

**CLI Modes:**
{}

**Workflow Stages:**
{}

**Architecture Layers:**
{}

**Architecture Components:**
{}

Create a detailed C4 model and task breakdown."#,
            design_artifact.data.cli.modes.join(", "),
            design_artifact.data.wf.stages.join(" → "),
            design_artifact.data.arch.layers.join(", "),
            design_artifact.data.arch.comps.join("\n"),
        );

        let agent = Arc::new(
            LlmAgentBuilder::new("plan_generator")
                .description("Generate implementation plan from design document")
                .instruction(
                    r#"You are a technical planner. Create a SIMPLE and FOCUSED implementation plan.

**CRITICAL PRINCIPLE: Simplicity Over Complexity**
- Focus ONLY on core functionality required to meet user needs
- Avoid adding testing frameworks, CI/CD pipelines, monitoring unless explicitly requested
- Keep the tech stack minimal and straightforward
- Prioritize "working code" over "perfect code"
- TodoList should focus on essential implementation tasks only

**Required JSON Structure:**
{
  "c4": {
    "context": ["system context descriptions"],
    "containers": ["container (app/service/db) descriptions"],
    "components": ["component descriptions"],
    "code": ["key code structure descriptions"]
  },
  "tasks": [
    {
      "id": "TASK-001",
      "pri": "p0|p1|p2",
      "desc": "task description",
      "deps": ["TASK-XXX dependencies"],
      "out": ["expected outputs/deliverables"]
    }
  ],
  "milestones": [
    {
      "id": "M1",
      "desc": "milestone description",
      "done_when": ["completion criteria"]
    }
  ],
  "todo_list": {
    "items": [
      {
        "id": "TODO-001",
        "description": "Specific actionable task for CORE functionality only",
        "status": "pending",
        "related_requirements": ["REQ-001"],
        "related_files": ["path/to/file.ext"],
        "verification_method": "manual_test|code_review (avoid complex testing infrastructure)"
      }
    ]
  }
}

**TodoList Generation Guidelines:**
1. Break down ONLY essential tasks for core functionality
2. Each TodoItem should map to specific requirements (from PRD)
3. List expected files to be created/modified
4. Use SIMPLE verification methods (manual test, basic code review)
5. Do NOT add tasks for: unit testing frameworks, CI/CD setup, coverage tools, linting setup
6. All todos should start with status "pending"
7. Ensure todos are ordered by dependencies
8. Keep it minimal - only what's needed to make the project work

**Output Requirements:**
1. Respond with ONLY valid JSON
2. All arrays must be present (including todo_list)
3. Tasks and todos should be ordered by dependencies
4. Each milestone should have clear, testable criteria
5. C4 model should be comprehensive yet concise
6. TodoList should cover ALL major implementation work"#,
                )
                .model(self.model.clone())
                .output_schema(output_schema)
                .output_key("plan_raw")
                .build()?,
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let app_name = "Cowork Forge".to_string();
        let user_id = session_id.to_string();

        let _session = session_service
            .create(CreateRequest {
                app_name: app_name.clone(),
                user_id: user_id.clone(),
                session_id: Some(session_id.to_string()),
                state: HashMap::new(),
            })
            .await?;

        let runner = Runner::new(RunnerConfig {
            app_name: app_name.clone(),
            agent: agent.clone(),
            session_service: session_service.clone(),
            artifact_service: None,
            memory_service: None,
            run_config: None,
        })?;

        let input_content = Content::new("user").with_text(&context);

        tracing::info!("Invoking Plan generation agent...");

        let mut event_stream = runner
            .run(user_id.clone(), session_id.to_string(), input_content)
            .await?;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(_event) => {},
                Err(e) => {
                    tracing::error!("Error during plan generation: {}", e);
                    return Err(anyhow::anyhow!("Plan generation failed: {}", e));
                }
            }
        }

        tracing::info!("Plan generation complete");

        let updated_session = session_service
            .get(GetRequest {
                user_id: user_id.clone(),
                session_id: session_id.to_string(),
                app_name: app_name.clone(),
                after: None,
                num_recent_events: None,
            })
            .await?;

        let state = updated_session.state();
        let raw_output = state
            .get("plan_raw")
            .ok_or_else(|| anyhow::anyhow!("No output from Plan agent"))?;

        let plan: Plan = match raw_output {
            serde_json::Value::String(json_str) => {
                serde_json::from_str(json_str.as_str())?
            }
            value => {
                serde_json::from_value(value.clone())?
            }
        };

        tracing::info!("Successfully parsed Plan");

        let summary = vec![
            format!("C4 Context: {} items", plan.c4.context.len()),
            format!("Tasks: {} total", plan.tasks.len()),
            format!("Milestones: {}", plan.milestones.len()),
        ];

        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Plan, plan)
            .with_summary(summary)
            .with_prev(vec![design_artifact.meta.artifact_id.clone()]);

        self.store.put(session_id, Stage::Plan, &artifact)?;

        tracing::info!("Plan artifact saved successfully");

        Ok(artifact)
    }
}

#[async_trait]
impl StageAgent for PlanAgent {
    fn stage(&self) -> Stage {
        Stage::Plan
    }
    
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult> {
        // 1. 加载 Design artifact
        let design_artifact: DesignDocArtifact = context.load_artifact(Stage::Design)?;
        
        // 2. 生成实施计划
        let mut artifact = self.generate_plan(&context.session_id, &design_artifact).await?;
        
        // 3. HITL 审查和修改
        if let Some(modified_json) = context.hitl.review_and_edit_json("Plan", &artifact.data)? {
            let modified_data: Plan = serde_json::from_str(&modified_json)?;
            artifact.data = modified_data;
            context.store.put(&context.session_id, Stage::Plan, &artifact)?;
            println!("✅ Plan 已更新");
        }
        
        // 4. 返回结果
        let summary = vec![
            format!("C4 Context: {} items", artifact.data.c4.context.len()),
            format!("Tasks: {} total", artifact.data.tasks.len()),
            format!("Milestones: {}", artifact.data.milestones.len()),
            format!("TodoList: {} items", artifact.data.todo_list.as_ref().map(|t| t.items.len()).unwrap_or(0)),
        ];
        
        Ok(StageAgentResult::new(artifact.meta.artifact_id, Stage::Plan)
            .with_verified(true)
            .with_summary(summary))
    }
    
    fn dependencies(&self) -> Vec<Stage> {
        vec![Stage::Design]
    }
    
    fn requires_hitl_review(&self) -> bool {
        true
    }
    
    fn description(&self) -> &str {
        "基于技术设计文档生成实施计划"
    }
}

