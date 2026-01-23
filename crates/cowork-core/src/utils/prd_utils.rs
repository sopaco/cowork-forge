use crate::artifacts::*;

/// 从 PRD Artifact 中提取摘要（用于 WatchDog）
pub fn extract_prd_summary(prd_artifact: &PRDArtifact) -> String {
    let prd = &prd_artifact.data;
    
    let mut summary_parts = vec![];
    
    // 项目范围
    if !prd.scope.g.is_empty() {
        summary_parts.push("**Goals**:".to_string());
        for goal in prd.scope.g.iter().take(3) {
            summary_parts.push(format!("- {}", goal));
        }
    }
    
    // 需求摘要（取前 5 个）
    if !prd.reqs.is_empty() {
        summary_parts.push("\n**Requirements**:".to_string());
        for req in prd.reqs.iter().take(5) {
            summary_parts.push(format!("- {}: {}", req.id, req.desc));
        }
        
        if prd.reqs.len() > 5 {
            summary_parts.push(format!("... and {} more requirements", prd.reqs.len() - 5));
        }
    }
    
    summary_parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prd_summary() {
        let prd_artifact = ArtifactEnvelope {
            meta: ArtifactMeta {
                session_id: "session_001".to_string(),
                artifact_id: "prd_001".to_string(),
                stage: Stage::Requirements,
                v: 1,
                ts: chrono::Utc::now(),
            },
            summary: vec![],
            links: ArtifactLinks { prev: vec![] },
            data: PRD {
                scope: Scope {
                    g: vec!["Create a todo app".to_string()],
                    ng: vec![],
                },
                reqs: vec![
                    Requirement {
                        id: "REQ-001".to_string(),
                        desc: "User can create todos".to_string(),
                        pri: Priority::P0,
                        req_type: RequirementType::Func,
                        deps: vec![],
                        ac: vec![],
                    },
                    Requirement {
                        id: "REQ-002".to_string(),
                        desc: "User can delete todos".to_string(),
                        pri: Priority::P0,
                        req_type: RequirementType::Func,
                        deps: vec![],
                        ac: vec![],
                    },
                ],
                cons: vec![],
                hitl: vec![],
            },
        };

        let summary = extract_prd_summary(&prd_artifact);
        
        assert!(summary.contains("Goals"));
        assert!(summary.contains("REQ-001"));
    }
}
