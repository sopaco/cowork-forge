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

/// PRD Agent - 基于 IdeaSpec 生成产品需求文档
pub struct PrdAgent {
    model: Arc<OpenAIClient>,
    store: Arc<ArtifactStore>,
}

impl PrdAgent {
    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        let config = OpenAIConfig::compatible(
            llm_config.api_key.clone(),
            llm_config.api_base_url.clone(),
            llm_config.model_name.clone(),
        );
        
        tracing::info!("Creating PRD Agent with OpenAI-compatible client )");
        
        let model = OpenAIClient::new(config)?;

        Ok(Self {
            model: Arc::new(model),
            store,
        })
    }

    async fn generate_prd(&self, session_id: &str, idea_artifact: &IdeaSpecArtifact) -> Result<PRDArtifact> {
        tracing::info!("PrdAgent: generating PRD for session {}", session_id);

        // Define output schema for PRD
        let output_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": "object",
                    "properties": {
                        "g": {"type": "array", "items": {"type": "string"}},
                        "ng": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["g", "ng"]
                },
                "reqs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "pri": {"type": "string", "enum": ["p0", "p1", "p2"]},
                            "type": {"type": "string", "enum": ["func", "nfr", "constraint"]},
                            "desc": {"type": "string"},
                            "deps": {"type": "array", "items": {"type": "string"}},
                            "ac": {"type": "array", "items": {"type": "string"}}
                        },
                        "required": ["id", "pri", "type", "desc", "deps", "ac"]
                    }
                },
                "cons": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "desc": {"type": "string"}
                        },
                        "required": ["id", "desc"]
                    }
                },
                "hitl": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "q": {"type": "string"},
                            "opts": {"type": "array", "items": {"type": "string"}},
                            "def": {"type": "string"}
                        },
                        "required": ["id", "q", "opts", "def"]
                    }
                }
            },
            "required": ["scope", "reqs", "cons", "hitl"]
        });

        // Build context from IdeaSpec
        let context = format!(
            r#"Based on the following IDEA specification, create a detailed Product Requirements Document (PRD).

**IDEA Background:**
{}

**Goals:**
{}

**Non-Goals:**
{}

**Constraints:**
{}

**Success Criteria:**
{}

**Risks:**
{}

**Questions:**
{}"#,
            idea_artifact.data.bg,
            idea_artifact.data.g.join("\n- "),
            idea_artifact.data.ng.join("\n- "),
            idea_artifact.data.c.join("\n- "),
            idea_artifact.data.sc.join("\n- "),
            idea_artifact.data.r.join("\n- "),
            idea_artifact.data.q.join("\n- "),
        );

        let agent = Arc::new(
            LlmAgentBuilder::new("prd_generator")
                .description("Generate Product Requirements Document from IdeaSpec")
                .instruction(
                    r#"You are a product manager. Create a structured PRD (Product Requirements Document) from the IDEA specification.

**Required JSON Structure:**
{
  "scope": {
    "g": ["array of in-scope goals"],
    "ng": ["array of out-of-scope items"]
  },
  "reqs": [
    {
      "id": "REQ-001",
      "pri": "p0|p1|p2",
      "type": "func|nfr|constraint",
      "desc": "requirement description",
      "deps": ["array of req IDs this depends on"],
      "ac": ["array of acceptance criteria"]
    }
  ],
  "cons": [
    {
      "id": "CON-001",
      "desc": "constraint description"
    }
  ],
  "hitl": [
    {
      "id": "HITL-001",
      "q": "question needing human input",
      "opts": ["option1", "option2"],
      "def": "default option"
    }
  ]
}

**Output Requirements:**
1. Respond with ONLY valid JSON (no markdown, no code blocks)
2. All arrays can be empty but must be present
3. Use clear, actionable language
4. Each requirement must have specific, testable acceptance criteria
5. Priority p0 = critical, p1 = important, p2 = nice-to-have
6. Include HITL questions for unclear decisions

Generate the PRD now based on the IDEA provided."#,
                )
                .model(self.model.clone())
                .output_schema(output_schema)
                .output_key("prd_raw")
                .build()?,
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let app_name = "cowork".to_string();
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

        tracing::info!("Invoking PRD generation agent...");

        let mut event_stream = runner
            .run(user_id.clone(), session_id.to_string(), input_content)
            .await?;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(event) => {
                    tracing::debug!("Event received: {:?}", event);
                }
                Err(e) => {
                    tracing::error!("Error during PRD generation: {}", e);
                    return Err(anyhow::anyhow!("PRD generation failed: {}", e));
                }
            }
        }

        tracing::info!("PRD generation complete");

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
            .get("prd_raw")
            .ok_or_else(|| anyhow::anyhow!("No output from PRD agent"))?;

        tracing::debug!("Raw PRD output: {}", raw_output);

        let prd: PRD = match raw_output {
            serde_json::Value::String(json_str) => {
                tracing::debug!("Output is a JSON string, parsing...");
                serde_json::from_str(json_str.as_str())
                    .map_err(|e| anyhow::anyhow!("Failed to parse PRD JSON: {}", e))?
            }
            value => {
                tracing::debug!("Output is a structured JSON value");
                serde_json::from_value(value.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize PRD: {}", e))?
            }
        };

        tracing::info!("Successfully parsed PRD");

        let summary = vec![
            format!("Scope: {} goals, {} non-goals", prd.scope.g.len(), prd.scope.ng.len()),
            format!("Requirements: {} total", prd.reqs.len()),
            format!("Constraints: {}", prd.cons.len()),
            format!("HITL Questions: {}", prd.hitl.len()),
        ];

        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Requirements, prd)
            .with_summary(summary)
            .with_prev(vec![idea_artifact.meta.artifact_id.clone()]);

        self.store.put(session_id, Stage::Requirements, &artifact)?;

        tracing::info!("PRD artifact saved successfully");

        Ok(artifact)
    }
}

#[async_trait]
impl StageAgent for PrdAgent {
    fn stage(&self) -> Stage {
        Stage::Requirements
    }
    
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult> {
        // 1. 加载 IdeaSpec artifact
        let idea_artifact: IdeaSpecArtifact = context.load_artifact(Stage::IdeaIntake)?;
        
        // 2. 生成 PRD
        let mut artifact = self.generate_prd(&context.session_id, &idea_artifact).await?;
        
        // 3. HITL 审查和修改
        if let Some(modified_json) = context.hitl.review_and_edit_json("PRD", &artifact.data)? {
            let modified_data: PRD = serde_json::from_str(&modified_json)?;
            artifact.data = modified_data;
            context.store.put(&context.session_id, Stage::Requirements, &artifact)?;
            println!("✅ PRD 已更新");
        }
        
        // 4. 返回结果
        let summary = vec![
            format!("Scope: {} goals, {} non-goals", artifact.data.scope.g.len(), artifact.data.scope.ng.len()),
            format!("Requirements: {} total", artifact.data.reqs.len()),
            format!("Constraints: {}", artifact.data.cons.len()),
            format!("HITL Questions: {}", artifact.data.hitl.len()),
        ];
        
        Ok(StageAgentResult::new(artifact.meta.artifact_id, Stage::Requirements)
            .with_verified(true)
            .with_summary(summary))
    }
    
    fn dependencies(&self) -> Vec<Stage> {
        vec![Stage::IdeaIntake]
    }
    
    fn requires_hitl_review(&self) -> bool {
        true
    }
    
    fn description(&self) -> &str {
        "基于 IdeaSpec 生成产品需求文档（PRD）"
    }
}

