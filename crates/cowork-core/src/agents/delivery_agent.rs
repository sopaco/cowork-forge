use anyhow::Result;
use std::sync::Arc;

use crate::artifacts::*;
use crate::memory::ArtifactStore;
use crate::config::LlmConfig;

/// Delivery Agent - 生成最终交付报告
pub struct DeliveryAgent {
    store: Arc<ArtifactStore>,
}

impl DeliveryAgent {
    pub fn new(_llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        tracing::info!("Creating Delivery Agent");
        
        Ok(Self {
            store,
        })
    }

    pub async fn execute(
        &self,
        session_id: &str,
        check_artifact: &CheckReportArtifact,
        _idea_artifact: &IdeaSpecArtifact,
    ) -> Result<DeliveryReportArtifact> {
        tracing::info!("DeliveryAgent: generating delivery report for session {}", session_id);

        // TODO: Implement comprehensive delivery report generation
        // For now, create a placeholder report
        
        let delivery_report = DeliveryReport {
            cap: vec![
                "Core functionality implemented".to_string(),
                "Basic error handling in place".to_string(),
            ],
            howto: vec![
                "Run: cargo run".to_string(),
                "Build: cargo build --release".to_string(),
            ],
            limits: vec![
                "Full workflow not yet complete".to_string(),
                "Limited test coverage".to_string(),
            ],
            acceptance: vec![
                format!("Checks run: {}", check_artifact.data.checks.len()),
                format!("Issues found: {}", check_artifact.data.issues.len()),
            ],
        };

        let summary = vec![
            format!("Capabilities: {}", delivery_report.cap.len()),
            format!("Usage steps: {}", delivery_report.howto.len()),
            format!("Known limits: {}", delivery_report.limits.len()),
        ];

        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Delivery, delivery_report)
            .with_summary(summary)
            .with_prev(vec![check_artifact.meta.artifact_id.clone()]);

        self.store.put(session_id, Stage::Delivery, &artifact)?;

        tracing::info!("Delivery report saved successfully");

        Ok(artifact)
    }
}
