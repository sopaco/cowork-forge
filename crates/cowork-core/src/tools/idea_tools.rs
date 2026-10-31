// Idea 工具 —— PM 自主规划者的提案与提案维护（设计 §3 / §6.2）
//
// 职责边界：`propose_ideas` 只负责**结构化校验与回执**，不落库。
// 质量门槛、去重、限量、落库全部由 `AutonomyController::run_cycle` 单点完成，
// 避免出现"工具写一份、controller 再写一份"的双实现。

use adk_core::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::domain::idea::IdeaDraft;

use super::get_required_string_param;

/// 提案工具名（提示词与此逐字一致）。
pub const PROPOSE_IDEAS_TOOL_NAME: &str = "propose_ideas";
pub const UPDATE_IDEA_TOOL_NAME: &str = "update_idea";

/// 从工具参数解析提案批次。
pub fn parse_proposals(args: &Value) -> Result<Vec<IdeaDraft>, String> {
    let ideas = args
        .get("ideas")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "propose_ideas 需要 ideas 数组参数".to_string())?;

    let mut drafts = Vec::new();
    for (index, item) in ideas.iter().enumerate() {
        let draft: IdeaDraft = serde_json::from_value(item.clone())
            .map_err(|e| format!("第 {} 条提案字段不合法: {}", index + 1, e))?;
        drafts.push(draft);
    }
    Ok(drafts)
}

// ============================================================================
// Propose Ideas Tool - 提交结构化提案批次
// ============================================================================

pub struct ProposeIdeasTool;

#[async_trait]
impl Tool for ProposeIdeasTool {
    fn name(&self) -> &str {
        PROPOSE_IDEAS_TOOL_NAME
    }

    fn description(&self) -> &str {
        "Submit structured iteration proposals discovered during autonomous research. \
         Call exactly once at the end of a research cycle. Each proposal must carry at least \
         one piece of local evidence (file/iteration/feedback); an external URL alone is not enough."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "ideas": {
                    "type": "array",
                    "description": "Proposals ordered by value, at most 3. Empty array when nothing is worth proposing.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string", "description": "Verb phrase naming the concrete change"},
                            "problem": {"type": "string", "description": "Observed pain point in one sentence"},
                            "hypothesis": {"type": "string", "description": "Verifiable 'do X so Y becomes Z' statement"},
                            "scope": {"type": "string", "enum": ["feature", "refactor", "fix", "docs", "research"]},
                            "estimated_effort": {"type": "string", "enum": ["s", "m", "l", "xl"]},
                            "priority": {"type": "string", "enum": ["high", "medium", "low"]},
                            "expected_benefit": {"type": "string", "description": "Quantified benefit, or '不确定' when unknown"},
                            "conflicts_with": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "IDs of existing ideas this one conflicts with"
                            },
                            "evidence": {
                                "type": "array",
                                "minItems": 1,
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "kind": {"type": "string", "enum": ["file_ref", "iteration_ref", "feedback_ref", "external_ref"]},
                                        "path": {"type": "string"},
                                        "line_range": {"type": "string"},
                                        "id": {"type": "string"},
                                        "stage": {"type": "string"},
                                        "timestamp": {"type": "string"},
                                        "url": {"type": "string"},
                                        "source": {"type": "string"},
                                        "accessed_at": {"type": "string"},
                                        "excerpt": {"type": "string", "description": "Verbatim excerpt, not a paraphrase"}
                                    },
                                    "required": ["kind", "excerpt"]
                                }
                            }
                        },
                        "required": ["title", "problem", "hypothesis", "scope", "estimated_effort", "priority", "evidence"]
                    }
                }
            },
            "required": ["ideas"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        match parse_proposals(&args) {
            Ok(drafts) => Ok(json!({
                "status": "success",
                "received": drafts.len(),
                "titles": drafts.iter().map(|d| d.title.clone()).collect::<Vec<_>>(),
                "message": "提案已收到，将经过质量门槛与去重后发布。"
            })),
            Err(message) => Ok(json!({
                "status": "error",
                "message": message
            })),
        }
    }
}

// ============================================================================
// Update Idea Tool - 维护既有提案（取代关系）
// ============================================================================

pub struct UpdateIdeaTool;

#[async_trait]
impl Tool for UpdateIdeaTool {
    fn name(&self) -> &str {
        UPDATE_IDEA_TOOL_NAME
    }

    fn description(&self) -> &str {
        "Mark an existing idea as superseded by a better one. Use when new evidence makes an \
         earlier proposal obsolete. Does not accept/reject ideas — that is the human's decision."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "idea_id": {"type": "string", "description": "Existing idea to update"},
                "superseded_by": {"type": "string", "description": "ID of the idea that replaces it"},
                "comment": {"type": "string", "description": "Why it is superseded"}
            },
            "required": ["idea_id", "superseded_by"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let idea_id = get_required_string_param(&args, "idea_id")?;
        let superseded_by = get_required_string_param(&args, "superseded_by")?;
        let comment = args
            .get("comment")
            .and_then(|v| v.as_str())
            .unwrap_or("新证据使原提案过时");

        let mut controller = crate::autonomy::AutonomyController::new()
            .map_err(|e| adk_core::AdkError::tool(e.to_string()))?;
        match controller.supersede_idea(idea_id, superseded_by, comment) {
            Ok(()) => Ok(json!({
                "status": "success",
                "message": format!("{} 已被 {} 取代", idea_id, superseded_by)
            })),
            Err(e) => Ok(json!({"status": "error", "message": e.to_string()})),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_proposals_reads_full_payload() {
        let args = json!({
            "ideas": [{
                "title": "补上重试幂等",
                "problem": "重试会重复生成文档",
                "hypothesis": "补判断后只补缺口",
                "scope": "fix",
                "estimated_effort": "m",
                "priority": "high",
                "expected_benefit": "不再重复",
                "conflicts_with": ["idea-demo-1"],
                "evidence": [
                    {"kind": "file_ref", "path": "src/main.rs", "line_range": "10-20", "excerpt": "fn retry() {}"},
                    {"kind": "external_ref", "url": "https://example.com", "source": "tavily",
                     "accessed_at": "2026-10-31T00:00:00Z", "excerpt": "advisory"}
                ]
            }]
        });

        let drafts = parse_proposals(&args).expect("parse");
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].evidence.len(), 2);
        assert_eq!(drafts[0].conflicts_with, vec!["idea-demo-1".to_string()]);
        assert_eq!(drafts[0].local_evidence_count_for_test(), 1);
    }

    #[test]
    fn parse_proposals_accepts_empty_batch() {
        assert!(parse_proposals(&json!({"ideas": []})).expect("empty").is_empty());
    }

    #[test]
    fn parse_proposals_rejects_malformed_item() {
        let err = parse_proposals(&json!({"ideas": [{"title": "缺字段"}]})).expect_err("must fail");
        assert!(err.contains("第 1 条提案"), "{}", err);
        assert!(parse_proposals(&json!({})).is_err());
    }

    #[test]
    fn tool_names_match_planner_instruction() {
        assert_eq!(PROPOSE_IDEAS_TOOL_NAME, "propose_ideas");
        assert_eq!(UPDATE_IDEA_TOOL_NAME, "update_idea");
        assert!(crate::instructions::AUTONOMOUS_PLANNER_INSTRUCTION.contains(PROPOSE_IDEAS_TOOL_NAME));
    }

    impl IdeaDraft {
        fn local_evidence_count_for_test(&self) -> usize {
            self.evidence.iter().filter(|e| e.is_local()).count()
        }
    }
}
