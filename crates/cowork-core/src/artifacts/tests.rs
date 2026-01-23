#[cfg(test)]
mod tests {
    use crate::artifacts::*;

    #[test]
    fn test_artifact_envelope_creation() {
        let idea_spec = IdeaSpec {
            bg: "Test background".to_string(),
            g: vec!["Goal 1".to_string()],
            ng: vec!["Non-goal 1".to_string()],
            c: vec!["Constraint 1".to_string()],
            sc: vec!["Success 1".to_string()],
            r: vec!["Risk 1".to_string()],
            q: vec!["Question 1".to_string()],
        };

        let envelope = ArtifactEnvelope::new("test-session".to_string(), Stage::IdeaIntake, idea_spec);

        assert_eq!(envelope.meta.session_id, "test-session");
        assert_eq!(envelope.meta.stage, Stage::IdeaIntake);
        assert_eq!(envelope.meta.v, 1);
        assert_eq!(envelope.data.bg, "Test background");
    }

    #[test]
    fn test_artifact_envelope_with_summary() {
        let idea_spec = IdeaSpec {
            bg: "Test".to_string(),
            g: vec![],
            ng: vec![],
            c: vec![],
            sc: vec![],
            r: vec![],
            q: vec![],
        };

        let envelope = ArtifactEnvelope::new("test-session".to_string(), Stage::IdeaIntake, idea_spec)
            .with_summary(vec!["Summary line 1".to_string(), "Summary line 2".to_string()]);

        assert_eq!(envelope.summary.len(), 2);
        assert_eq!(envelope.summary[0], "Summary line 1");
    }

    #[test]
    fn test_artifact_envelope_with_prev() {
        let idea_spec = IdeaSpec {
            bg: "Test".to_string(),
            g: vec![],
            ng: vec![],
            c: vec![],
            sc: vec![],
            r: vec![],
            q: vec![],
        };

        let envelope = ArtifactEnvelope::new("test-session".to_string(), Stage::IdeaIntake, idea_spec)
            .with_prev(vec!["prev-artifact-1".to_string()]);

        assert_eq!(envelope.links.prev.len(), 1);
        assert_eq!(envelope.links.prev[0], "prev-artifact-1");
    }

    #[test]
    fn test_stage_as_str() {
        assert_eq!(Stage::IdeaIntake.as_str(), "idea_intake");
        assert_eq!(Stage::Requirements.as_str(), "requirements");
        assert_eq!(Stage::Design.as_str(), "design");
        assert_eq!(Stage::Plan.as_str(), "plan");
        assert_eq!(Stage::Coding.as_str(), "coding");
        assert_eq!(Stage::Check.as_str(), "check");
        assert_eq!(Stage::Feedback.as_str(), "feedback");
        assert_eq!(Stage::Delivery.as_str(), "delivery");
    }

    #[test]
    fn test_stage_all() {
        let all_stages = Stage::all();
        assert_eq!(all_stages.len(), 8);
        assert_eq!(all_stages[0], Stage::IdeaIntake);
        assert_eq!(all_stages[7], Stage::Delivery);
    }

    #[test]
    fn test_prd_serialization() {
        let prd = PRD {
            scope: Scope {
                g: vec!["Goal 1".to_string()],
                ng: vec!["Non-goal 1".to_string()],
            },
            reqs: vec![
                Requirement {
                    id: "REQ-001".to_string(),
                    pri: Priority::P0,
                    req_type: RequirementType::Func,
                    desc: "Test requirement".to_string(),
                    deps: vec![],
                    ac: vec!["AC1".to_string()],
                }
            ],
            cons: vec![],
            hitl: vec![],
        };

        let json = serde_json::to_string(&prd).unwrap();
        let deserialized: PRD = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.reqs.len(), 1);
        assert_eq!(deserialized.reqs[0].id, "REQ-001");
        assert_eq!(deserialized.reqs[0].pri, Priority::P0);
    }

    #[test]
    fn test_design_doc_structure() {
        let design = DesignDoc {
            cli: CliDesign {
                modes: vec!["interactive".to_string()],
                hitl_flow: vec!["flow1".to_string()],
            },
            wf: Workflow {
                stages: vec!["stage1".to_string()],
                transitions: vec!["s1->s2".to_string()],
            },
            arch: Architecture {
                layers: vec!["layer1".to_string()],
                comps: vec!["comp1".to_string()],
            },
            io: IoConfig {
                artifact_dir: ".output".to_string(),
                formats: vec!["json".to_string()],
            },
        };

        assert_eq!(design.cli.modes.len(), 1);
        assert_eq!(design.wf.stages.len(), 1);
        assert_eq!(design.arch.layers.len(), 1);
        assert_eq!(design.io.artifact_dir, ".output");
    }

    #[test]
    fn test_plan_structure() {
        let plan = Plan {
            c4: C4Design {
                context: vec!["ctx1".to_string()],
                containers: vec!["container1".to_string()],
                components: vec!["comp1".to_string()],
                code: vec!["code1".to_string()],
            },
            tasks: vec![
                Task {
                    id: "TASK-001".to_string(),
                    pri: Priority::P0,
                    desc: "Test task".to_string(),
                    deps: vec![],
                    out: vec!["output1".to_string()],
                }
            ],
            milestones: vec![
                Milestone {
                    id: "M1".to_string(),
                    desc: "Milestone 1".to_string(),
                    done_when: vec!["criteria1".to_string()],
                }
            ],
            todo_list: None,
        };

        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.milestones.len(), 1);
        assert_eq!(plan.c4.context.len(), 1);
    }

    #[test]
    fn test_priority_serialization() {
        let p0 = Priority::P0;
        let p1 = Priority::P1;
        let p2 = Priority::P2;

        let p0_json = serde_json::to_string(&p0).unwrap();
        let p1_json = serde_json::to_string(&p1).unwrap();
        let p2_json = serde_json::to_string(&p2).unwrap();

        assert_eq!(p0_json, "\"p0\"");
        assert_eq!(p1_json, "\"p1\"");
        assert_eq!(p2_json, "\"p2\"");
    }

    #[test]
    fn test_requirement_type_serialization() {
        let func = RequirementType::Func;
        let nfr = RequirementType::Nfr;
        let constraint = RequirementType::Constraint;

        let func_json = serde_json::to_string(&func).unwrap();
        let nfr_json = serde_json::to_string(&nfr).unwrap();
        let constraint_json = serde_json::to_string(&constraint).unwrap();

        assert_eq!(func_json, "\"func\"");
        assert_eq!(nfr_json, "\"nfr\"");
        assert_eq!(constraint_json, "\"constraint\"");
    }
}
