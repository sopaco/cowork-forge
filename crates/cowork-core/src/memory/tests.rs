#[cfg(test)]
mod tests {
    use crate::memory::*;
    use crate::artifacts::*;
    use tempfile::TempDir;

    fn create_temp_store() -> (TempDir, ArtifactStore) {
        let temp_dir = TempDir::new().unwrap();
        let store = ArtifactStore::new(temp_dir.path().to_str().unwrap());
        (temp_dir, store)
    }

    #[test]
    fn test_artifact_store_creation() {
        let (_temp_dir, _store) = create_temp_store();
        // Store created successfully if we reach here
    }

    #[test]
    fn test_put_and_list_artifacts() {
        let (_temp_dir, store) = create_temp_store();
        
        let idea_spec = IdeaSpec {
            bg: "Test background".to_string(),
            g: vec!["Goal 1".to_string()],
            ng: vec![],
            c: vec![],
            sc: vec![],
            r: vec![],
            q: vec![],
        };

        let artifact = ArtifactEnvelope::new(
            "test-session".to_string(),
            Stage::IdeaIntake,
            idea_spec,
        );

        store.put("test-session", Stage::IdeaIntake, &artifact).unwrap();

        let artifacts = store.list("test-session").unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].stage, Stage::IdeaIntake);
    }

    #[test]
    fn test_get_artifact() {
        let (_temp_dir, store) = create_temp_store();
        
        let idea_spec = IdeaSpec {
            bg: "Test background".to_string(),
            g: vec!["Goal 1".to_string()],
            ng: vec![],
            c: vec![],
            sc: vec![],
            r: vec![],
            q: vec![],
        };

        let artifact = ArtifactEnvelope::new(
            "test-session".to_string(),
            Stage::IdeaIntake,
            idea_spec,
        );

        let artifact_id = artifact.meta.artifact_id.clone();

        store.put("test-session", Stage::IdeaIntake, &artifact).unwrap();

        let retrieved: IdeaSpecArtifact = 
            store.get("test-session", &artifact_id).unwrap();

        assert_eq!(retrieved.meta.artifact_id, artifact_id);
        assert_eq!(retrieved.data.bg, "Test background");
    }

    #[test]
    fn test_session_exists() {
        let (_temp_dir, store) = create_temp_store();
        
        assert!(!store.session_exists("non-existent-session"));
        
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
            "test-session".to_string(),
            Stage::IdeaIntake,
            idea_spec,
        );

        store.put("test-session", Stage::IdeaIntake, &artifact).unwrap();

        assert!(store.session_exists("test-session"));
    }

    #[test]
    fn test_multiple_artifacts_different_stages() {
        let (_temp_dir, store) = create_temp_store();
        
        // IdeaSpec
        let idea_spec = IdeaSpec {
            bg: "Test".to_string(),
            g: vec![],
            ng: vec![],
            c: vec![],
            sc: vec![],
            r: vec![],
            q: vec![],
        };
        let idea_artifact = ArtifactEnvelope::new(
            "test-session".to_string(),
            Stage::IdeaIntake,
            idea_spec,
        );
        store.put("test-session", Stage::IdeaIntake, &idea_artifact).unwrap();

        // PRD
        let prd = PRD {
            scope: Scope {
                g: vec![],
                ng: vec![],
            },
            reqs: vec![],
            cons: vec![],
            hitl: vec![],
        };
        let prd_artifact = ArtifactEnvelope::new(
            "test-session".to_string(),
            Stage::Requirements,
            prd,
        );
        store.put("test-session", Stage::Requirements, &prd_artifact).unwrap();

        let artifacts = store.list("test-session").unwrap();
        assert_eq!(artifacts.len(), 2);
        
        // Verify both stages are present
        let stages: Vec<Stage> = artifacts.iter().map(|a| a.stage).collect();
        assert!(stages.contains(&Stage::IdeaIntake));
        assert!(stages.contains(&Stage::Requirements));
    }

    #[test]
    fn test_artifact_json_serialization() {
        let (_temp_dir, store) = create_temp_store();
        
        let idea_spec = IdeaSpec {
            bg: "Serialization test".to_string(),
            g: vec!["G1".to_string(), "G2".to_string()],
            ng: vec!["NG1".to_string()],
            c: vec!["C1".to_string()],
            sc: vec!["SC1".to_string()],
            r: vec!["R1".to_string()],
            q: vec!["Q1".to_string()],
        };

        let artifact = ArtifactEnvelope::new(
            "test-session".to_string(),
            Stage::IdeaIntake,
            idea_spec,
        )
        .with_summary(vec!["Summary test".to_string()]);

        let artifact_id = artifact.meta.artifact_id.clone();
        
        store.put("test-session", Stage::IdeaIntake, &artifact).unwrap();

        let retrieved: IdeaSpecArtifact = 
            store.get("test-session", &artifact_id).unwrap();

        assert_eq!(retrieved.data.bg, "Serialization test");
        assert_eq!(retrieved.data.g.len(), 2);
        assert_eq!(retrieved.summary.len(), 1);
        assert_eq!(retrieved.summary[0], "Summary test");
    }
}
