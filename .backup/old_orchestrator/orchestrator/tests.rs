#[cfg(test)]
mod tests {
    use crate::orchestrator::*;
    use crate::memory::ArtifactStore;
    use crate::artifacts::*;
    use tempfile::TempDir;

    fn create_test_orchestrator() -> (TempDir, Orchestrator) {
        let temp_dir = TempDir::new().unwrap();
        let store = ArtifactStore::new(temp_dir.path().to_str().unwrap());
        (temp_dir, Orchestrator::new(store))
    }

    #[test]
    fn test_create_session() {
        let (_temp_dir, orchestrator) = create_test_orchestrator();
        
        let session_id = orchestrator.create_session().unwrap();
        assert!(!session_id.is_empty());
        
        // Verify meta.json exists
        let meta = orchestrator.load_session_meta(&session_id).unwrap();
        assert_eq!(meta.session_id, session_id);
        assert_eq!(meta.current_stage, None);
        assert!(meta.stage_status.is_empty());
    }

    #[test]
    fn test_save_and_load_session_meta() {
        let (_temp_dir, orchestrator) = create_test_orchestrator();
        
        let session_id = orchestrator.create_session().unwrap();
        
        // Modify meta
        let mut meta = orchestrator.load_session_meta(&session_id).unwrap();
        meta.current_stage = Some(Stage::Requirements);
        meta.stage_status.insert(
            Stage::IdeaIntake,
            StageStatus::Completed {
                artifact_id: "test-artifact-1".to_string(),
                completed_at: chrono::Utc::now(),
                verified: true,
            }
        );
        meta.stage_status.insert(
            Stage::Requirements,
            StageStatus::Completed {
                artifact_id: "test-artifact-2".to_string(),
                completed_at: chrono::Utc::now(),
                verified: true,
            }
        );
        
        orchestrator.save_session_meta(&meta).unwrap();
        
        // Reload and verify
        let loaded_meta = orchestrator.load_session_meta(&session_id).unwrap();
        assert_eq!(loaded_meta.current_stage, Some(Stage::Requirements));
        assert_eq!(loaded_meta.stage_status.len(), 2);
        assert!(matches!(
            loaded_meta.stage_status.get(&Stage::IdeaIntake),
            Some(StageStatus::Completed { verified: true, .. })
        ));
        assert!(matches!(
            loaded_meta.stage_status.get(&Stage::Requirements),
            Some(StageStatus::Completed { verified: true, .. })
        ));
    }

    #[test]
    fn test_load_artifact() {
        let (_temp_dir, orchestrator) = create_test_orchestrator();
        
        let session_id = orchestrator.create_session().unwrap();
        
        // Create and save an artifact
        let idea_spec = crate::artifacts::IdeaSpec {
            bg: "Test background".to_string(),
            g: vec!["Goal 1".to_string()],
            ng: vec![],
            c: vec![],
            sc: vec![],
            r: vec![],
            q: vec![],
        };
        
        let artifact = crate::artifacts::ArtifactEnvelope::new(
            session_id.clone(),
            Stage::IdeaIntake,
            idea_spec,
        );
        
        orchestrator.store.put(&session_id, Stage::IdeaIntake, &artifact).unwrap();
        
        // Load it back
        let loaded: crate::artifacts::IdeaSpecArtifact = 
            orchestrator.load_artifact(&session_id, Stage::IdeaIntake).unwrap();
        
        assert_eq!(loaded.data.bg, "Test background");
        assert_eq!(loaded.data.g.len(), 1);
    }

    #[test]
    fn test_session_exists() {
        let (_temp_dir, orchestrator) = create_test_orchestrator();
        
        let session_id = orchestrator.create_session().unwrap();
        
        // After create_session, meta.json exists
        let meta = orchestrator.load_session_meta(&session_id);
        assert!(meta.is_ok());
        
        // Store's session_exists checks for artifacts dir
        // Save an artifact to create the artifacts directory
        let idea_spec = IdeaSpec {
            bg: "Test".to_string(),
            g: vec![],
            ng: vec![],
            c: vec![],
            sc: vec![],
            r: vec![],
            q: vec![],
        };
        
        let artifact = ArtifactEnvelope::new(
            session_id.clone(),
            Stage::IdeaIntake,
            idea_spec,
        );
        
        orchestrator.store.put(&session_id, Stage::IdeaIntake, &artifact).unwrap();
        
        assert!(orchestrator.store.session_exists(&session_id));
        assert!(!orchestrator.store.session_exists("non-existent-session"));
    }

    #[test]
    fn test_list_artifacts() {
        let (_temp_dir, orchestrator) = create_test_orchestrator();
        
        let session_id = orchestrator.create_session().unwrap();
        
        // Create multiple artifacts
        let idea_spec = crate::artifacts::IdeaSpec {
            bg: "Test".to_string(),
            g: vec![],
            ng: vec![],
            c: vec![],
            sc: vec![],
            r: vec![],
            q: vec![],
        };
        
        let artifact1 = crate::artifacts::ArtifactEnvelope::new(
            session_id.clone(),
            Stage::IdeaIntake,
            idea_spec.clone(),
        );
        
        orchestrator.store.put(&session_id, Stage::IdeaIntake, &artifact1).unwrap();
        
        let prd = crate::artifacts::PRD {
            scope: crate::artifacts::Scope {
                g: vec![],
                ng: vec![],
            },
            reqs: vec![],
            cons: vec![],
            hitl: vec![],
        };
        
        let artifact2 = crate::artifacts::ArtifactEnvelope::new(
            session_id.clone(),
            Stage::Requirements,
            prd,
        );
        
        orchestrator.store.put(&session_id, Stage::Requirements, &artifact2).unwrap();
        
        // List artifacts
        let artifacts = orchestrator.list_artifacts(&session_id).unwrap();
        assert_eq!(artifacts.len(), 2);
        
        let stages: Vec<Stage> = artifacts.iter().map(|a| a.stage).collect();
        assert!(stages.contains(&Stage::IdeaIntake));
        assert!(stages.contains(&Stage::Requirements));
    }
}
