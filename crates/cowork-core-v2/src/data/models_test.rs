#[cfg(test)]
mod tests {
    use crate::storage::*;
    use crate::data::*;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id("REQ", 0);
        assert_eq!(id1, "REQ-001");

        let id2 = generate_id("FEAT", 9);
        assert_eq!(id2, "FEAT-010");

        let id3 = generate_id("TASK", 99);
        assert_eq!(id3, "TASK-100");
    }

    #[test]
    fn test_requirements_new() {
        let reqs = Requirements::new();
        assert_eq!(reqs.schema_version, "1.0");
        assert_eq!(reqs.requirements.len(), 0);
    }

    #[test]
    fn test_feature_list_new() {
        let features = FeatureList::new();
        assert_eq!(features.features.len(), 0);
    }

    #[test]
    fn test_design_spec_new() {
        let design = DesignSpec::new();
        assert_eq!(design.architecture.components.len(), 0);
    }

    #[test]
    fn test_implementation_plan_new() {
        let plan = ImplementationPlan::new();
        assert_eq!(plan.tasks.len(), 0);
        assert_eq!(plan.milestones.len(), 0);
    }

    #[test]
    fn test_feedback_history_new() {
        let history = FeedbackHistory::new();
        assert_eq!(history.feedbacks.len(), 0);
    }
}
