#[cfg(test)]
mod tests {
    use crate::data::*;
    use crate::storage::*;
    use std::env;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Serialize tests that mutate current_dir
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_env() -> (TempDir, PathBuf) {
        let original_dir = env::current_dir().unwrap();
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();
        (temp_dir, original_dir)
    }

    fn cleanup_test_env(original_dir: PathBuf) {
        let _ = env::set_current_dir(original_dir);
    }

    #[test]
    fn test_get_cowork_dir_creates_structure() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (_temp, original_dir) = setup_test_env();

        let dir = get_cowork_dir().unwrap();
        assert!(dir.exists());
        assert!(dir.join("sessions").exists());

        cleanup_test_env(original_dir);
    }

    #[test]
    fn test_save_and_load_requirements_session_scoped() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (_temp, original_dir) = setup_test_env();

        get_cowork_dir().unwrap();
        let session_id = "session-test";
        get_session_dir(session_id).unwrap();

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

        save_requirements(session_id, &reqs).unwrap();
        let loaded = load_requirements(session_id).unwrap();

        assert_eq!(loaded.requirements.len(), 1);
        assert_eq!(loaded.requirements[0].id, "REQ-001");

        cleanup_test_env(original_dir);
    }

    #[test]
    fn test_init_session_from_base_copies_state() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (_temp, original_dir) = setup_test_env();

        get_cowork_dir().unwrap();

        let base = "session-base";
        let new = "session-new";
        get_session_dir(base).unwrap();

        let mut reqs = Requirements::new();
        reqs.requirements.push(Requirement {
            id: "REQ-001".to_string(),
            title: "Base Requirement".to_string(),
            description: "Base description".to_string(),
            priority: Priority::High,
            category: RequirementCategory::Functional,
            acceptance_criteria: vec!["Criterion".to_string()],
            related_features: vec![],
        });
        save_requirements(base, &reqs).unwrap();

        init_session_from_base(new, base).unwrap();

        let loaded = load_requirements(new).unwrap();
        assert_eq!(loaded.requirements.len(), 1);
        assert_eq!(loaded.requirements[0].title, "Base Requirement");

        cleanup_test_env(original_dir);
    }

    #[test]
    fn test_cowork_dir_exists() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (_temp, original_dir) = setup_test_env();

        assert!(!cowork_dir_exists());
        get_cowork_dir().unwrap();
        assert!(cowork_dir_exists());

        cleanup_test_env(original_dir);
    }
}
