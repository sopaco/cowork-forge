// Autonomous Planner Agent —— PM 自主规划环的执行体（设计 §6.2）
//
// 与流水线里的阶段 Agent 不同，它**不产出工件**，只产出结构化的 Idea 提案。
// 调研证据由 `autonomy::research` 在 Rust 侧确定性收集后拼进 prompt，
// Agent 只负责"基于这份上下文判断缺口并提出可验证的改动方向"。

use adk_agent::LlmAgentBuilder;
use adk_core::{Content, IncludeContents, Llm};
use anyhow::Result;
use std::sync::Arc;

use crate::domain::idea::IdeaDraft;
use crate::instructions::{
    AUTONOMOUS_PLANNER_INSTRUCTION, AUTONOMOUS_PLANNER_NAME,
};
use crate::tools::idea_tools::{parse_proposals, PROPOSE_IDEAS_TOOL_NAME};
use crate::tools::{
    ListFilesTool, ProposeIdeasTool, QueryMemoryTool, ReadFileTool, ReadFileWithLimitTool,
    UpdateIdeaTool,
};

/// 规划 Agent 的执行结果。
#[derive(Debug, Clone, Default)]
pub struct PlannerOutcome {
    /// 解析出的提案批次（按 Agent 提交顺序）。
    pub drafts: Vec<IdeaDraft>,
    /// Agent 的自由文本（用于日志与降级诊断）。
    pub text: String,
    /// 调用过的工具名（用于统计外部检索是否真的发生）。
    pub called_tools: Vec<String>,
    /// 是否调用过外部检索类工具。
    pub external_used: bool,
}

impl PlannerOutcome {
    /// 外部检索是否真的发生（不是"配置了 MCP"，而是"这轮真的查了"）。
    pub fn used_external_search(&self) -> bool {
        self.external_used
    }
}

/// 构建自主规划 Agent。
pub fn create_autonomous_planner_agent(
    model: Arc<dyn Llm>,
    iteration_id: String,
) -> Result<Arc<dyn adk_core::Agent>> {
    let mut builder = LlmAgentBuilder::new(AUTONOMOUS_PLANNER_NAME)
        .instruction(AUTONOMOUS_PLANNER_INSTRUCTION)
        .model(model)
        .tool(Arc::new(ProposeIdeasTool))
        .tool(Arc::new(UpdateIdeaTool))
        // 只读工具：调研期间不得修改工程
        .tool(Arc::new(ListFilesTool))
        .tool(Arc::new(ReadFileTool))
        .tool(Arc::new(ReadFileWithLimitTool::new(10)))
        .tool(Arc::new(QueryMemoryTool::new(iteration_id)))
        .include_contents(IncludeContents::None);

    // 外部检索（MCP）按需挂载；未配置时 Agent 不会看到这些工具
    builder = crate::config_definition::agent_factory::add_mcp_toolsets_to_builder(builder);

    let agent = builder.build()?;
    Ok(Arc::new(agent))
}

/// 执行一次自主规划，返回提案批次。
///
/// `prompt` 由 `autonomy::research::ResearchContext::to_prompt()` 组装，
/// 这里只负责跑 Agent 并从工具调用里取回结构化提案。
pub async fn run_autonomous_planner(
    model: Arc<dyn Llm>,
    prompt: String,
    iteration_id: String,
) -> Result<PlannerOutcome> {
    use crate::pipeline::stage_executor::{extract_text_from_event, SimpleInvocationContext};
    use crate::pipeline::PipelineContext;
    use futures::StreamExt;

    let agent = create_autonomous_planner_agent(model, iteration_id.clone())?;

    // 规划 Agent 不产出工件，用一个**不落库**的临时迭代提供会话 id，
    // 避免污染真实迭代列表与全局当前迭代。
    let project = crate::persistence::ProjectStore::new()
        .load()?
        .unwrap_or_else(|| crate::domain::Project::new("workspace"));
    let scratch_iteration = crate::domain::Iteration::create_genesis(
        &project,
        "autonomous research".to_string(),
        String::new(),
    );
    let workspace_path = crate::persistence::get_cowork_dir()
        .map(|p| p.join("workspace"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let content = Content::new("user").with_text(prompt);
    let ctx = PipelineContext::new(project, scratch_iteration, workspace_path);
    let invocation_ctx = Arc::new(SimpleInvocationContext::new(&ctx, &content, agent.clone()));

    let mut stream = agent
        .run(invocation_ctx)
        .await
        .map_err(|e| anyhow::anyhow!("自主规划 Agent 执行失败: {}", e))?;

    let mut outcome = PlannerOutcome::default();
    let mut drafts: Vec<IdeaDraft> = Vec::new();

    while let Some(result) = stream.next().await {
        let Ok(event) = result else {
            continue;
        };

        if let Some(text) = extract_text_from_event(&event)
            && !text.trim().is_empty()
        {
            outcome.text.push_str(&text);
        }

        let Some(content) = event.content() else {
            continue;
        };
        for part in &content.parts {
            let adk_core::Part::FunctionCall { name, args, .. } = part else {
                continue;
            };
            outcome.called_tools.push(name.clone());
            if name == PROPOSE_IDEAS_TOOL_NAME {
                match parse_proposals(args) {
                    Ok(batch) => drafts.extend(batch),
                    Err(e) => {
                        tracing::warn!("[autonomy] propose_ideas 参数不合法: {}", e);
                    }
                }
            } else if !is_builtin_readonly_tool(name) {
                // 非内置只读工具 = 外部检索（MCP）
                outcome.external_used = true;
            }
        }
    }

    // 只保留最后一次提交（Agent 若重复调用，以最后一次为准）
    outcome.drafts = drafts;
    Ok(outcome)
}

fn is_builtin_readonly_tool(name: &str) -> bool {
    matches!(
        name,
        "propose_ideas"
            | "update_idea"
            | "list_files"
            | "read_file"
            | "read_file_truncated"
            | "query_memory"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_readonly_detection() {
        assert!(is_builtin_readonly_tool("list_files"));
        assert!(is_builtin_readonly_tool("propose_ideas"));
        assert!(!is_builtin_readonly_tool("tavily_search"));
        assert!(!is_builtin_readonly_tool("deepwiki_ask_question"));
    }

    #[test]
    fn outcome_reports_external_usage() {
        let mut outcome = PlannerOutcome::default();
        assert!(!outcome.used_external_search());
        outcome.external_used = true;
        assert!(outcome.used_external_search());
    }
}
