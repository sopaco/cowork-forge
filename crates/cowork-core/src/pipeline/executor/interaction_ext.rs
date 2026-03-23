// Interaction extension traits for iteration executor

use crate::interaction::{InputOption, InputResponse, InteractiveBackend};

/// Confirmation action for user interaction
#[derive(Debug, Clone)]
pub enum ConfirmationAction {
    Continue,                // User confirmed to continue
    ViewArtifact,            // User wants to view the artifact
    ProvideFeedback(String), // User provided feedback for revision
    Cancel,                  // User cancelled
}

/// Extension trait for InteractiveBackend to support confirmation dialogs
#[async_trait::async_trait]
pub trait InteractionExt {
    async fn request_confirmation(&self, prompt: &str) -> bool;
    async fn request_confirmation_with_artifact(&self, prompt: &str, artifact_type: &str) -> bool;
    async fn request_confirmation_with_feedback(
        &self,
        prompt: &str,
        artifact_type: &str,
    ) -> ConfirmationAction;
}

#[async_trait::async_trait]
impl InteractionExt for dyn InteractiveBackend {
    async fn request_confirmation(&self, prompt: &str) -> bool {
        let options = vec![
            InputOption {
                id: "yes".to_string(),
                label: "Continue".to_string(),
                description: Some("Proceed to next stage".to_string()),
            },
            InputOption {
                id: "no".to_string(),
                label: "Cancel".to_string(),
                description: Some("Stop the iteration".to_string()),
            },
        ];

        match self.request_input(prompt, options, None).await {
            Ok(InputResponse::Selection(id)) => id == "yes",
            _ => false,
        }
    }

    async fn request_confirmation_with_artifact(&self, prompt: &str, artifact_type: &str) -> bool {
        let options = vec![
            InputOption {
                id: "yes".to_string(),
                label: "Continue".to_string(),
                description: Some("Confirm and proceed to next stage".to_string()),
            },
            InputOption {
                id: "view_artifact".to_string(),
                label: "View Artifact".to_string(),
                description: Some(format!("Open {} tab to review", artifact_type)),
            },
            InputOption {
                id: "no".to_string(),
                label: "Cancel".to_string(),
                description: Some("Stop the iteration".to_string()),
            },
        ];

        let full_prompt = format!("{}\n[ARTIFACT_TYPE:{}]", prompt, artifact_type);

        match self.request_input(&full_prompt, options, None).await {
            Ok(InputResponse::Selection(id)) => match id.as_str() {
                "yes" => true,
                "view_artifact" => {
                    let _ = self
                        .show_message(
                            crate::interaction::MessageLevel::Info,
                            format!("[VIEW_ARTIFACT:{}]", artifact_type),
                        )
                        .await;
                    false
                }
                _ => false,
            },
            _ => false,
        }
    }

    async fn request_confirmation_with_feedback(
        &self,
        prompt: &str,
        artifact_type: &str,
    ) -> ConfirmationAction {
        let options = vec![
            InputOption {
                id: "yes".to_string(),
                label: "Continue".to_string(),
                description: Some("Confirm and proceed to next stage".to_string()),
            },
            InputOption {
                id: "view_artifact".to_string(),
                label: "View Artifact".to_string(),
                description: Some(format!("Open {} tab to review", artifact_type)),
            },
            InputOption {
                id: "feedback".to_string(),
                label: "Provide Feedback".to_string(),
                description: Some("Enter feedback to regenerate".to_string()),
            },
            InputOption {
                id: "no".to_string(),
                label: "Cancel".to_string(),
                description: Some("Stop the iteration".to_string()),
            },
        ];

        let full_prompt = format!("{}\n[ARTIFACT_TYPE:{}]", prompt, artifact_type);

        match self.request_input(&full_prompt, options, None).await {
            Ok(InputResponse::Selection(id)) => match id.as_str() {
                "yes" => ConfirmationAction::Continue,
                "view_artifact" => ConfirmationAction::ViewArtifact,
                "feedback" => {
                    let feedback_options = vec![InputOption {
                        id: "submit".to_string(),
                        label: "Submit Feedback".to_string(),
                        description: Some("Submit your feedback".to_string()),
                    }];

                    let feedback_prompt = "Please enter your feedback or suggestions for improvement:";

                    match self
                        .request_input(feedback_prompt, feedback_options, Some(String::new()))
                        .await
                    {
                        Ok(InputResponse::Text(feedback)) => {
                            ConfirmationAction::ProvideFeedback(feedback)
                        }
                        Ok(InputResponse::Selection(_)) => ConfirmationAction::ViewArtifact,
                        _ => ConfirmationAction::Cancel,
                    }
                }
                _ => ConfirmationAction::Cancel,
            },
            Ok(InputResponse::Text(feedback)) => ConfirmationAction::ProvideFeedback(feedback),
            _ => ConfirmationAction::Cancel,
        }
    }
}
