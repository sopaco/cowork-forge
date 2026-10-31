// Autonomous Planner Agent —— PM 自主规划环的执行体（设计 §6.2）
//
// 与流水线里的阶段 Agent 不同，它**不产出工件**，只产出结构化的 Idea 提案。
// 调研证据由 `autonomy::research` 在 Rust 侧确定性收集后拼进 prompt，
// Agent 只负责"基于这份上下文判断缺口并提出可验证的改动方向"。

use adk_agent::LlmAgentBuilder;
use adk_core::{Content, IncludeContents, Llm};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

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
    /// 外部检索实际调用次数（超预算会提前掐断本轮）。
    pub external_calls: u32,
}

impl PlannerOutcome {
    /// 外部检索是否真的发生（不是"配置了 MCP"，而是"这轮真的查了"）。
    pub fn used_external_search(&self) -> bool {
        self.external_used
    }
}

/// 构建自主规划 Agent。
///
/// `max_iterations` 与 `tool_timeout` 把 adk 的默认值（100 轮 / 5 分钟且带重试）
/// 收紧到自主调研能接受的范围：预算里的"每轮最多 N 次 LLM 调用"只有落到
/// 迭代上限上才是真闸门，否则只是跑完之后的记账。
pub fn create_autonomous_planner_agent(
    model: Arc<dyn Llm>,
    iteration_id: String,
    max_iterations: u32,
    tool_timeout: Duration,
) -> Result<Arc<dyn adk_core::Agent>> {
    let mut builder = LlmAgentBuilder::new(AUTONOMOUS_PLANNER_NAME)
        .instruction(AUTONOMOUS_PLANNER_INSTRUCTION)
        .model(model)
        // 至少给 1 轮，配置里写成 0 也不能让 Agent 直接空跑
        .max_iterations(max_iterations.max(1))
        .tool_timeout(tool_timeout)
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
pub async fn run_autonomous_planner_with_limits(
    model: Arc<dyn Llm>,
    prompt: String,
    iteration_id: String,
    max_iterations: u32,
    tool_timeout: Duration,
) -> Result<PlannerOutcome> {
    run_autonomous_planner_bounded(
        model,
        prompt,
        iteration_id,
        max_iterations,
        tool_timeout,
        u32::MAX,
    )
    .await
}

/// 与 [`run_autonomous_planner_with_limits`] 相同，额外限制外部检索调用次数。
///
/// 超限时**掐断**本轮（丢弃 Agent 的事件流即取消其执行），而不是跑完再记账：
/// 预算只有能在过程中拦住才有意义。
pub async fn run_autonomous_planner_bounded(
    model: Arc<dyn Llm>,
    prompt: String,
    iteration_id: String,
    max_iterations: u32,
    tool_timeout: Duration,
    max_external_calls: u32,
) -> Result<PlannerOutcome> {
    use crate::pipeline::stage_executor::{extract_text_from_event, SimpleInvocationContext};
    use crate::pipeline::PipelineContext;
    use futures::StreamExt;

    let agent =
        create_autonomous_planner_agent(model, iteration_id.clone(), max_iterations, tool_timeout)?;

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
    let mut external_calls: u32 = 0;
    let mut budget_exceeded = false;
    // 事件流里的错误不能静默吞掉：模型调用失败时 adk 会吐一条 Err 并结束流，
    // 吞掉它会让本轮看起来像"跑完了但没发现"，掩盖真正的原因。
    let mut stream_error: Option<String> = None;

    while let Some(result) = stream.next().await {
        let event = match result {
            Ok(event) => event,
            Err(e) => {
                if stream_error.is_none() {
                    stream_error = Some(e.to_string());
                }
                continue;
            }
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
                external_calls = external_calls.saturating_add(1);
                if external_calls > max_external_calls {
                    budget_exceeded = true;
                }
            }
        }

        if budget_exceeded {
            // 丢弃事件流 = 取消本轮 Agent 执行
            tracing::warn!(
                "[autonomy] 外部检索已用满 {} 次预算，提前结束本轮提案",
                max_external_calls
            );
            break;
        }
    }

    outcome.external_calls = external_calls;

    // 一条提案都没拿到 + 流里出现过错误 ⇒ 这是失败，不是"没有新发现"
    if drafts.is_empty()
        && let Some(err) = stream_error
    {
        return Err(anyhow::anyhow!("规划 Agent 执行中断：{}", err));
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
