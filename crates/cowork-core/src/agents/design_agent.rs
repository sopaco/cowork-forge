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

/// Design Agent - 基于 PRD 生成技术设计文档
pub struct DesignAgent {
    model: Arc<OpenAIClient>,
    store: Arc<ArtifactStore>,
}

impl DesignAgent {
    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        let config = OpenAIConfig::compatible(
            llm_config.api_key.clone(),
            llm_config.api_base_url.clone(),
            llm_config.model_name.clone(),
        );
        
        tracing::info!("Creating Design Agent with OpenAI-compatible client )");
        
        let model = OpenAIClient::new(config)?;

        Ok(Self {
            model: Arc::new(model),
            store,
        })
    }

    async fn generate_design(&self, session_id: &str, prd_artifact: &PRDArtifact) -> Result<DesignDocArtifact> {
        tracing::info!("DesignAgent: generating design document for session {}", session_id);

        let output_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "cli": {
                    "type": "object",
                    "properties": {
                        "modes": {"type": "array", "items": {"type": "string"}},
                        "hitl_flow": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["modes", "hitl_flow"]
                },
                "wf": {
                    "type": "object",
                    "properties": {
                        "stages": {"type": "array", "items": {"type": "string"}},
                        "transitions": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["stages", "transitions"]
                },
                "arch": {
                    "type": "object",
                    "properties": {
                        "layers": {"type": "array", "items": {"type": "string"}},
                        "comps": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["layers", "comps"]
                },
                "io": {
                    "type": "object",
                    "properties": {
                        "artifact_dir": {"type": "string"},
                        "formats": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["artifact_dir", "formats"]
                }
            },
            "required": ["cli", "wf", "arch", "io"]
        });

        // Build context from PRD
        let req_summary: Vec<String> = prd_artifact.data.reqs.iter()
            .map(|r| format!("{} [{}]: {}", r.id, r.pri as u8, r.desc))
            .collect();

        let context = format!(
            r#"Based on the following PRD, create a technical design document.

**Scope:**
In-scope goals: {}
Out-of-scope: {}

**Requirements:**
{}

**Constraints:**
{}

Create a design that addresses all functional and non-functional requirements."#,
            prd_artifact.data.scope.g.join(", "),
            prd_artifact.data.scope.ng.join(", "),
            req_summary.join("\n"),
            prd_artifact.data.cons.iter().map(|c| c.desc.as_str()).collect::<Vec<_>>().join("\n"),
        );

        let agent = Arc::new(
            LlmAgentBuilder::new("design_generator")
                .description("Generate technical design document from PRD")
                .instruction(
                    r#"You are a technical architect. Create a SIMPLE and PRACTICAL design document.

**CRITICAL PRINCIPLE: Keep It Simple**
- Focus on core functionality ONLY
- Avoid unnecessary complexity
- Do NOT include testing frameworks, CI/CD, coverage tools unless explicitly required
- Use the simplest tech stack that meets requirements
- Prioritize clarity and maintainability over advanced features

**Required JSON Structure:**
{
  "cli": {
    "modes": ["interactive", "batch", "server"],
    "hitl_flow": ["description of human-in-the-loop interaction points"]
  },
  "wf": {
    "stages": ["stage1", "stage2", ...],
    "transitions": ["stage1 -> stage2: condition", ...]
  },
  "arch": {
    "layers": ["presentation", "business", "data", ...],
    "comps": ["component descriptions"]
  },
  "io": {
    "artifact_dir": "./.output",
    "formats": ["json", "markdown", ...]
  }
}

**Output Requirements:**
1. Respond with ONLY valid JSON
2. All arrays must be present (can be empty)
3. Design should be SIMPLE, practical and implementable
4. Avoid over-engineering - use minimal viable architecture
5. NO testing infrastructure unless explicitly requested
6. NO CI/CD, monitoring, logging frameworks unless required"#,
                )
                .model(self.model.clone())
                .output_schema(output_schema)
                .output_key("design_raw")
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

        tracing::info!("Invoking Design generation agent...");

        let mut event_stream = runner
            .run(user_id.clone(), session_id.to_string(), input_content)
            .await?;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(_event) => {},
                Err(e) => {
                    tracing::error!("Error during design generation: {}", e);
                    return Err(anyhow::anyhow!("Design generation failed: {}", e));
                }
            }
        }

        tracing::info!("Design generation complete");

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
            .get("design_raw")
            .ok_or_else(|| anyhow::anyhow!("No output from Design agent"))?;

        let design: DesignDoc = match raw_output {
            serde_json::Value::String(json_str) => {
                serde_json::from_str(json_str.as_str())?
            }
            value => {
                serde_json::from_value(value.clone())?
            }
        };

        tracing::info!("Successfully parsed DesignDoc");

        let summary = vec![
            format!("CLI modes: {}", design.cli.modes.len()),
            format!("Workflow stages: {}", design.wf.stages.len()),
            format!("Architecture components: {}", design.arch.comps.len()),
            format!("Output formats: {}", design.io.formats.join(", ")),
        ];

        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Design, design)
            .with_summary(summary)
            .with_prev(vec![prd_artifact.meta.artifact_id.clone()]);

        self.store.put(session_id, Stage::Design, &artifact)?;

        tracing::info!("Design artifact saved successfully");

        Ok(artifact)
    }
}

#[async_trait]
impl StageAgent for DesignAgent {
    fn stage(&self) -> Stage {
        Stage::Design
    }
    
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult> {
        // 1. 加载 PRD artifact
        let prd_artifact: PRDArtifact = context.load_artifact(Stage::Requirements)?;
        
        // 2. 生成设计文档
        let mut artifact = self.generate_design(&context.session_id, &prd_artifact).await?;
        
        // 3. HITL 审查和修改
        if let Some(modified_json) = context.hitl.review_and_edit_json("DesignDoc", &artifact.data)? {
            let modified_data: DesignDoc = serde_json::from_str(&modified_json)?;
            artifact.data = modified_data;
            context.store.put(&context.session_id, Stage::Design, &artifact)?;
            println!("✅ DesignDoc 已更新");
        }
        
        // 4. 返回结果
        let summary = vec![
            format!("CLI modes: {}", artifact.data.cli.modes.len()),
            format!("Workflow stages: {}", artifact.data.wf.stages.len()),
            format!("Architecture components: {}", artifact.data.arch.comps.len()),
            format!("Output formats: {}", artifact.data.io.formats.join(", ")),
        ];
        
        Ok(StageAgentResult::new(artifact.meta.artifact_id, Stage::Design)
            .with_verified(true)
            .with_summary(summary))
    }
    
    fn dependencies(&self) -> Vec<Stage> {
        vec![Stage::Requirements]
    }
    
    fn requires_hitl_review(&self) -> bool {
        true
    }
    
    fn description(&self) -> &str {
        "基于 PRD 生成技术设计文档"
    }
}

