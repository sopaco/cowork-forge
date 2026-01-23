use anyhow::Result;
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

/// IDEA Intake Agent - 将用户输入转换为结构化的 IdeaSpec
pub struct IdeaIntakeAgent {
    model: Arc<OpenAIClient>,
    store: Arc<ArtifactStore>,
}

impl IdeaIntakeAgent {
    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        // Create OpenAI-compatible client using the compatible() constructor
        // This sets the custom base_url for private deployment
        let config = OpenAIConfig::compatible(
            llm_config.api_key.clone(),
            llm_config.api_base_url.clone(),
            llm_config.model_name.clone(),
        );
        
        tracing::info!("Creating OpenAI-compatible client");
        tracing::info!("  API Base: {}", llm_config.api_base_url);
        tracing::info!("  Model: {}", llm_config.model_name);
        tracing::info!("  API Key: {}...", &llm_config.api_key[..10]);
        
        let model = OpenAIClient::new(config)?;

        Ok(Self {
            model: Arc::new(model),
            store,
        })
    }

    pub async fn execute(&self, session_id: &str, user_input: &str) -> Result<IdeaSpecArtifact> {
        tracing::info!("IdeaIntakeAgent: processing user input for session {}", session_id);

        // Define the output schema for IdeaSpec
        // Note: For OpenAI-compatible APIs that don't support response_format,
        // this schema is primarily used for documentation and potential guardrail validation.
        // The actual structure is enforced through the instruction prompt.
        let output_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "bg": {
                    "type": "string",
                    "description": "Background (1-2 sentences describing the context)"
                },
                "g": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Goals (list of project objectives)"
                },
                "ng": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Non-goals (what's explicitly out of scope)"
                },
                "c": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Constraints (technical/business limitations)"
                },
                "sc": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Success criteria (measurable outcomes)"
                },
                "r": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Risks (potential issues)"
                },
                "q": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Questions (unresolved points needing clarification)"
                }
            },
            "required": ["bg", "g", "ng", "c", "sc", "r", "q"]
        });

        // Build agent with output_schema and detailed instruction
        // Since the OpenAI-compatible API may not support response_format,
        // we provide explicit JSON structure in the instruction.
        let agent = Arc::new(
            LlmAgentBuilder::new("idea_intake")
                .description("Convert user IDEA into structured IdeaSpec")
                .instruction(
                    r#"You are an IDEA analyzer. Extract and structure the user's idea into a JSON object.

**Required JSON Structure:**
{
  "bg": "string - Background context in 1-2 sentences",
  "g": ["array of strings - Project goals/objectives"],
  "ng": ["array of strings - Non-goals (out of scope items)"],
  "c": ["array of strings - Constraints (technical/business limitations)"],
  "sc": ["array of strings - Success criteria (measurable outcomes)"],
  "r": ["array of strings - Risks (potential issues)"],
  "q": ["array of strings - Questions (unresolved points)"]
}

**Output Requirements:**
1. Respond with ONLY valid JSON (no markdown, no code blocks, no additional text)
2. All fields are required (use empty arrays if no items)
3. Be concise - use short phrases
4. Ensure all array items are non-empty strings

**Example:**
{
  "bg": "Build a landing page to showcase product features",
  "g": ["Attract potential customers", "Explain core value proposition"],
  "ng": ["E-commerce functionality", "User authentication"],
  "c": ["Static HTML only", "Load time < 3s"],
  "sc": ["Mobile responsive", "90+ Lighthouse score"],
  "r": ["Content may become outdated"],
  "q": ["What color scheme?", "Need multilingual support?"]
}"#,
                )
                .model(self.model.clone())
                .output_schema(output_schema)  // For documentation and future guardrail validation
                .output_key("idea_spec_raw")
                .build()?,
        );

        // Initialize session service and create a session
        let session_service = Arc::new(InMemorySessionService::new());
        let app_name = "cowork".to_string();
        let user_id = session_id.to_string();

        let session = session_service
            .create(CreateRequest {
                app_name: app_name.clone(),
                user_id: user_id.clone(),
                session_id: Some(session_id.to_string()),
                state: HashMap::new(),
            })
            .await?;

        tracing::debug!("Session created: {}", session.id());

        // Create the Runner with agent in config
        let runner = Runner::new(RunnerConfig {
            app_name: app_name.clone(),
            agent: agent.clone(),
            session_service: session_service.clone(),
            artifact_service: None,
            memory_service: None,
            run_config: None,
        })?;

        // Define the input content
        let input_content = Content::new("user").with_text(user_input);

        tracing::info!("Invoking LLM agent...");

        // Run the agent and consume event stream
        let mut event_stream = runner
            .run(user_id.clone(), session_id.to_string(), input_content)
            .await?;

        // Consume the event stream to ensure agent execution completes
        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(event) => {
                    tracing::debug!("Event received: {:?}", event);
                    // Optionally process LLM responses
                    if let Some(llm_response_content) = event.llm_response.content {
                        for part in llm_response_content.parts {
                            if let Some(text) = part.text() {
                                tracing::debug!("LLM output: {}", text);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error during agent execution: {}", e);
                    return Err(anyhow::anyhow!("Agent execution failed: {}", e));
                }
            }
        }

        tracing::info!("Agent execution complete");

        // Retrieve the session state and extract the structured data
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

        // Extract the output from session state
        let raw_output = state
            .get("idea_spec_raw")
            .ok_or_else(|| anyhow::anyhow!("No output from agent (key 'idea_spec_raw' not found)"))?;

        tracing::debug!("Raw output from session state: {}", raw_output);

        // Parse the JSON output into IdeaSpec
        // The LLM might return a JSON string or a JSON object
        let idea_spec: IdeaSpec = match raw_output {
            serde_json::Value::String(json_str) => {
                // If it's a string, parse it first
                tracing::debug!("Output is a JSON string, parsing...");
                serde_json::from_str(json_str.as_str())
                    .map_err(|e| anyhow::anyhow!("Failed to parse JSON string: {}", e))?
            }
            value => {
                // If it's already a structured value, deserialize directly
                tracing::debug!("Output is a structured JSON value");
                serde_json::from_value(value.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize JSON value: {}", e))?
            }
        };

        tracing::info!("Successfully parsed IdeaSpec");

        // Create artifact
        let summary = vec![
            format!("Background: {}", idea_spec.bg),
            format!("Goals: {}", idea_spec.g.len()),
            format!("Non-Goals: {}", idea_spec.ng.len()),
            format!("Constraints: {}", idea_spec.c.len()),
        ];

        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::IdeaIntake, idea_spec)
            .with_summary(summary);

        // Save to store
        self.store.put(session_id, Stage::IdeaIntake, &artifact)?;

        tracing::info!("IdeaSpec artifact saved successfully");

        Ok(artifact)
    }
}
