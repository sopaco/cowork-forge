#[cfg(test)]
mod tests {
    use crate::storage::*;
    use crate::data::*;
    use tempfile::TempDir;
    use std::env;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();
        temp_dir
    }

    #[test]
    fn test_get_cowork_dir_creates_structure() {
        let _temp = setup_test_env();
        
        let dir = get_cowork_dir().unwrap();
        assert!(dir.exists());
        assert!(dir.join("data").exists());
        assert!(dir.join("artifacts").exists());
        assert!(dir.join("session").exists());
        assert!(dir.join("logs").exists());
    }

    #[test]
    fn test_save_and_load_requirements() {
        let _temp = setup_test_env();
        get_cowork_dir().unwrap();

        let mut reqs = Requirements::new();
        reqs.requirements.push(Requirement {
            id: "REQ-001".to_string(),
            title: "Test Requirement".to_string(),
            description: "Test description".to_string(),
            priority: Priority::High,
            category: RequirementCategory::Functional,
            acceptance_criteria: vec!["Criterion 1".to_string()],
            related_features: vec![],
        });

        save_requirements(&reqs).unwrap();
        let loaded = load_requirements().unwrap();

        assert_eq!(loaded.requirements.len(), 1);
        assert_eq!(loaded.requirements[0].id, "REQ-001");
        assert_eq!(loaded.requirements[0].title, "Test Requirement");
    }

    #[test]
    fn test_save_and_load_feature_list() {
        let _temp = setup_test_env();
        get_cowork_dir().unwrap();

        let mut features = FeatureList::new();
        features.features.push(Feature {
            id: "FEAT-001".to_string(),
            name: "Test Feature".to_string(),
            description: "Test description".to_string(),
            requirement_ids: vec!["REQ-001".to_string()],
            status: FeatureStatus::Pending,
            assigned_to_tasks: vec![],
            completion_criteria: vec!["Done".to_string()],
            created_at: chrono::Utc::now(),
            completed_at: None,
            metadata: FeatureMetadata::default(),
        });

        save_feature_list(&features).unwrap();
        let loaded = load_feature_list().unwrap();

        assert_eq!(loaded.features.len(), 1);
        assert_eq!(loaded.features[0].id, "FEAT-001");
    }

    #[test]
    fn test_cowork_dir_exists() {
        let _temp = setup_test_env();
        
        assert!(!cowork_dir_exists());
        get_cowork_dir().unwrap();
        assert!(cowork_dir_exists());
    }
}
