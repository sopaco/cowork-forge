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

/// Feedback Agent - 收集反馈并决定是否需要迭代
pub struct FeedbackAgent {
    model: Arc<OpenAIClient>,
    store: Arc<ArtifactStore>,
}

impl FeedbackAgent {
    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        let config = OpenAIConfig::compatible(
            llm_config.api_key.clone(),
            llm_config.api_base_url.clone(),
            llm_config.model_name.clone(),
        );
        
        tracing::info!("Creating Feedback Agent with OpenAI-compatible client )");
        
        let model = OpenAIClient::new(config)?;

        Ok(Self {
            model: Arc::new(model),
            store,
        })
    }

    pub async fn analyze_feedback(
        &self,
        session_id: &str,
        check_artifact: &CheckReportArtifact,
        user_feedback: &str,
    ) -> Result<FeedbackArtifact> {
        tracing::info!("FeedbackAgent: processing feedback for session {}", session_id);

        let output_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "delta": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "target_stage": {
                                "type": "string",
                                "enum": ["idea_intake", "requirements", "design", "plan", "coding", "check", "feedback", "delivery"]
                            },
                            "change": {"type": "string"}
                        },
                        "required": ["target_stage", "change"]
                    }
                },
                "rerun": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "stage": {
                                "type": "string",
                                "enum": ["idea_intake", "requirements", "design", "plan", "coding", "check", "feedback", "delivery"]
                            },
                            "reason": {"type": "string"}
                        },
                        "required": ["stage", "reason"]
                    }
                }
            },
            "required": ["delta", "rerun"]
        });

        let context = format!(
            r#"Based on the check report and user feedback, analyze what needs to be changed.

**Check Report Summary:**
Total checks: {}
Issues found: {}

**Issues:**
{}

**User Feedback:**
{}

Determine what changes are needed and which stages should be re-run."#,
            check_artifact.data.checks.len(),
            check_artifact.data.issues.len(),
            check_artifact.data.issues.iter()
                .map(|i| format!("[{}] {}: {}", i.sev, i.id, i.desc))
                .collect::<Vec<_>>()
                .join("\n"),
            user_feedback,
        );

        let agent = Arc::new(
            LlmAgentBuilder::new("feedback_analyzer")
                .description("Analyze feedback and determine necessary changes")
                .instruction(
                    r#"You are a project coordinator. Analyze feedback and determine next steps.

**IMPORTANT GUIDELINES:**

1. **Understand User Intent**:
   - If user mentions "页面" (page), "界面" (UI), "代码" (code), "功能" (feature) → likely needs Coding stage change
   - If user mentions "需求" (requirements), "功能点" (feature points) → likely needs Requirements stage change
   - If user mentions "技术方案" (tech solution), "架构" (architecture), "数据库" (database) → likely needs Design stage change
   - If user mentions "计划" (plan), "任务" (tasks) → likely needs Plan stage change

2. **Delta Generation Rules**:
   - `delta` describes WHAT to change in which stage
   - `target_stage` should match the stage that owns the artifact being modified
   - Be specific: "修改登录页面布局" not just "修改页面"

3. **Rerun Generation Rules**:
   - `rerun` specifies which stages need to be re-executed
   - **CRITICAL**: If delta targets Coding, you MUST include Coding in rerun list
   - **CRITICAL**: If delta targets Design, you MUST include Design in rerun list
   - Always cascade: Coding change → rerun [Coding, Check, Feedback]
   - Design change → rerun [Design, Plan, Coding, Check, Feedback]

4. **Common Patterns**:
   - "修改页面" → delta: Coding, rerun: [Coding, Check]
   - "改用 PostgreSQL" → delta: Design, rerun: [Design, Plan, Coding, Check]
   - "增加新需求" → delta: Requirements, rerun: [Requirements, Design, Plan, Coding, Check]

**Required JSON Structure:**
{
  "delta": [
    {
      "target_stage": "stage_name",
      "change": "description of what needs to change"
    }
  ],
  "rerun": [
    {
      "stage": "stage_to_rerun",
      "reason": "why it needs to be re-run"
    }
  ]
}

**Stage Names:**
- idea_intake, requirements, design, plan, coding, check, feedback, delivery

**Output Requirements:**
1. Respond with ONLY valid JSON
2. Arrays can be empty if no changes/reruns needed
3. Be specific about what needs to change
4. Provide clear reasons for re-runs
5. **ENSURE delta.target_stage matches the first stage in rerun list**"#,
                )
                .model(self.model.clone())
                .output_schema(output_schema)
                .output_key("feedback_raw")
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

        tracing::info!("Invoking Feedback analysis agent...");

        let mut event_stream = runner
            .run(user_id.clone(), session_id.to_string(), input_content)
            .await?;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(_event) => {},
                Err(e) => {
                    tracing::error!("Error during feedback analysis: {}", e);
                    return Err(anyhow::anyhow!("Feedback analysis failed: {}", e));
                }
            }
        }

        tracing::info!("Feedback analysis complete");

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
            .get("feedback_raw")
            .ok_or_else(|| anyhow::anyhow!("No output from Feedback agent"))?;

        let feedback: Feedback = match raw_output {
            serde_json::Value::String(json_str) => {
                serde_json::from_str(json_str.as_str())?
            }
            value => {
                serde_json::from_value(value.clone())?
            }
        };

        tracing::info!("Successfully parsed Feedback");

        let summary = vec![
            format!("Changes needed: {}", feedback.delta.len()),
            format!("Stages to rerun: {}", feedback.rerun.len()),
        ];

        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Feedback, feedback)
            .with_summary(summary)
            .with_prev(vec![check_artifact.meta.artifact_id.clone()]);

        self.store.put(session_id, Stage::Feedback, &artifact)?;

        tracing::info!("Feedback artifact saved successfully");

        Ok(artifact)
    }
}

#[async_trait]
impl StageAgent for FeedbackAgent {
    fn stage(&self) -> Stage {
        Stage::Feedback
    }
    
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult> {
        // 1. 加载 CheckReport
        let check_artifact: CheckReportArtifact = context.load_artifact(Stage::Check)?;
        
        // 2. 获取用户反馈
        let user_feedback = if let Some(ref input) = context.user_input {
            input.clone()
        } else {
            context.hitl.input("有反馈吗？（直接回车跳过）")?
        };
        
        // 如果没有反馈，返回空的 Feedback
        if user_feedback.trim().is_empty() {
            println!("✓ 用户满意，跳过 Feedback");
            
            let empty_feedback = Feedback {
                delta: vec![],
                rerun: vec![],
            };
            
            let artifact = ArtifactEnvelope::new(context.session_id.clone(), Stage::Feedback, empty_feedback)
                .with_summary(vec!["No feedback".to_string()])
                .with_prev(vec![check_artifact.meta.artifact_id.clone()]);
            
            context.store.put(&context.session_id, Stage::Feedback, &artifact)?;
            
            return Ok(StageAgentResult::new(artifact.meta.artifact_id, Stage::Feedback)
                .with_verified(true)
                .with_summary(vec!["No changes needed".to_string()]));
        }
        
        // 3. 分析反馈
        let artifact = self.analyze_feedback(&context.session_id, &check_artifact, &user_feedback).await?;
        
        // 4. 返回结果
        let summary = vec![
            format!("Changes needed: {}", artifact.data.delta.len()),
            format!("Stages to rerun: {}", artifact.data.rerun.len()),
        ];
        
        Ok(StageAgentResult::new(artifact.meta.artifact_id, Stage::Feedback)
            .with_verified(true)
            .with_summary(summary))
    }
    
    fn dependencies(&self) -> Vec<Stage> {
        vec![Stage::Check]
    }
    
    fn requires_hitl_review(&self) -> bool {
        false  // Feedback 阶段本身就是收集 HITL
    }
    
    fn description(&self) -> &str {
        "收集用户反馈并决定是否需要迭代"
    }
}

