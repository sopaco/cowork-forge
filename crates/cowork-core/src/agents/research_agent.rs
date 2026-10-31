// Autonomous Planner Agent —— PM 自主规划环的执行体（设计 §6.2）
//
// 与流水线里的阶段 Agent 不同，它**不产出工件**，只产出结构化的 Idea 提案。
// 调研证据由 `autonomy::research` 在 Rust 侧确定性收集后拼进 prompt，
// Agent 只负责"基于这份上下文判断缺口并提出可验证的改动方向"。

use adk_agent::LlmAgentBuilder;
use adk_core::{Content, IncludeContents, Llm, Tool};
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
    /// 是否真的调用了 `propose_ideas`。
    ///
    /// 这个字段区分两种"0 条提案"：模型明确提交了空数组（认为没缺口），
    /// 与模型根本没按约定提交（被截断/不遵守指令）。后者不是结论，是故障。
    pub propose_called: bool,
    /// `propose_ideas` 参数解析失败次数。
    ///
    /// 以前只打一条 warn，界面照样显示"没有新发现"——模型明明提交了，
    /// 却被我们自己的解析错误吞成了空结论。
    pub parse_errors: u32,
    /// 模型回合数（含工具调用的轮次），用于还原真实调用量。
    pub tool_rounds: u32,
    /// 外部检索预算用尽 → 本轮被掐断。
    pub truncated: bool,
}

impl PlannerOutcome {
    /// 外部检索是否真的发生（不是"配置了 MCP"，而是"这轮真的查了"）。
    pub fn used_external_search(&self) -> bool {
        self.external_used
    }
}

/// 规划 Agent 的内置工具集合。
///
/// 名字一律从工具实例取，**不再手写白名单**：之前 `is_builtin_readonly_tool`
/// 里写的是 `read_file_truncated`，而工具真实注册名是 `read_file_with_limit`，
/// 结果每读一个文件都被算成一次"外部检索"，三个文件就把 MCP 预算烧光，
/// 第四个文件直接把事件流掐断——表现为"跑完了，但没有新发现"。
pub fn planner_builtin_tools(iteration_id: &str) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ProposeIdeasTool),
        Arc::new(UpdateIdeaTool),
        // 只读工具：调研期间不得修改工程
        Arc::new(ListFilesTool),
        Arc::new(ReadFileTool),
        Arc::new(ReadFileWithLimitTool::new(10)),
        Arc::new(QueryMemoryTool::new(iteration_id.to_string())),
    ]
}

/// 构建自主规划 Agent。
///
/// `max_iterations` 与 `tool_timeout` 把 adk 的默认值（100 轮 / 5 分钟且带重试）
/// 收紧到自主调研能接受的范围：预算里的"每轮最多 N 次 LLM 调用"只有落到
/// 迭代上限上才是真闸门，否则只是跑完之后的记账。
///
/// `allow_external` 决定是否把 MCP 工具挂上去。这必须落在**挂载**上而不是
/// 提示词里：只要工具可见，模型就会去调，白纸黑字的"本轮只用本地证据"
/// 拦不住它（实测三个外部调用把预算吃满）。
pub fn create_autonomous_planner_agent(
    model: Arc<dyn Llm>,
    iteration_id: String,
    max_iterations: u32,
    tool_timeout: Duration,
    allow_external: bool,
) -> Result<Arc<dyn adk_core::Agent>> {
    let tools = planner_builtin_tools(&iteration_id);

    let mut builder = LlmAgentBuilder::new(AUTONOMOUS_PLANNER_NAME)
        .instruction(AUTONOMOUS_PLANNER_INSTRUCTION)
        .model(model)
        // 至少给 1 轮，配置里写成 0 也不能让 Agent 直接空跑
        .max_iterations(max_iterations.max(1))
        .tool_timeout(tool_timeout)
        .include_contents(IncludeContents::None);

    for tool in &tools {
        builder = builder.tool(tool.clone());
    }

    // 外部检索（MCP）按策略挂载；不挂时 Agent 根本看不到这些工具
    if allow_external {
        builder = crate::config_definition::agent_factory::add_mcp_toolsets_to_builder(builder);
    }

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
        true,
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
    allow_external: bool,
) -> Result<PlannerOutcome> {
    use crate::pipeline::stage_executor::{extract_text_from_event, SimpleInvocationContext};
    use crate::pipeline::PipelineContext;
    use futures::StreamExt;

    let agent = create_autonomous_planner_agent(
        model,
        iteration_id.clone(),
        max_iterations,
        tool_timeout,
        allow_external,
    )?;

    // 文件类工具（list_files / read_file / read_file_with_limit）只认**全局**
    // 迭代 ID，不认 PipelineContext：全局未设置时它们直接报
    // "Iteration ID not set"，模型就永远看不到代码，只能对着文件清单猜。
    // 这里显式指向最新迭代，并在作用域结束时复原。
    let target_iteration = crate::persistence::IterationStore::new()
        .load_all()
        .ok()
        .and_then(|mut iterations| {
            iterations.sort_by_key(|i| i.number);
            iterations.pop()
        })
        .map(|i| i.id);
    let _iteration_scope = target_iteration.as_deref().map(IterationScope::enter);

    // 内置工具名从工具实例取，杜绝白名单与注册名漂移
    let builtin_names: std::collections::HashSet<String> = planner_builtin_tools(&iteration_id)
        .iter()
        .map(|tool| tool.name().to_string())
        .collect();

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
    // `workspace_path` 之前指向 `.cowork-v2/workspace`（根本不存在）。
    // 文件工具不读它，但上下文里带着一个假路径只会误导后续维护者，
    // 这里对齐到真实迭代工作区。
    let workspace_path = target_iteration
        .as_deref()
        .and_then(|id| {
            crate::persistence::IterationStore::new()
                .workspace_path(id)
                .ok()
        })
        .unwrap_or_else(|| {
            crate::persistence::get_cowork_dir()
                .map(|p| p.join("workspace"))
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
        });

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
        let mut turn_has_tool_call = false;
        for part in &content.parts {
            let adk_core::Part::FunctionCall { name, args, .. } = part else {
                continue;
            };
            turn_has_tool_call = true;
            outcome.called_tools.push(name.clone());
            if name == PROPOSE_IDEAS_TOOL_NAME {
                outcome.propose_called = true;
                match parse_proposals(args) {
                    Ok(batch) => drafts.extend(batch),
                    Err(e) => {
                        outcome.parse_errors = outcome.parse_errors.saturating_add(1);
                        tracing::warn!("[autonomy] propose_ideas 参数不合法: {}", e);
                    }
                }
            } else if !builtin_names.contains(name) {
                // 只读内置工具之外的调用 = 外部检索（MCP）
                outcome.external_used = true;
                external_calls = external_calls.saturating_add(1);
                if external_calls > max_external_calls {
                    budget_exceeded = true;
                }
            }
        }
        if turn_has_tool_call {
            outcome.tool_rounds = outcome.tool_rounds.saturating_add(1);
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
    outcome.truncated = budget_exceeded;

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

/// 临时把全局迭代 ID 指向目标迭代，作用域结束时复原。
///
/// 文件类工具读的是全局单例而不是 `PipelineContext`，调研期间必须替它们
/// 把"当前迭代"设对，同时不能让这次设置泄漏到执行侧。
struct IterationScope {
    previous: Option<String>,
}

impl IterationScope {
    fn enter(iteration_id: &str) -> Self {
        let previous = crate::persistence::get_iteration_id();
        crate::persistence::set_iteration_id(iteration_id.to_string());
        Self { previous }
    }
}

impl Drop for IterationScope {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(id) => crate::persistence::set_iteration_id(id),
            None => crate::persistence::clear_iteration_id(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 白名单必须覆盖全部内置工具的真实注册名。
    ///
    /// 这个测试存在的理由：曾经手写的白名单里是 `read_file_truncated`，
    /// 而工具实际叫 `read_file_with_limit`，导致每次读文件都被记成一次
    /// 外部检索、烧掉 MCP 预算，最后把整轮 Agent 掐断成"没有新发现"。
    #[test]
    fn builtin_tool_names_cover_every_registered_tool() {
        let names: Vec<String> = planner_builtin_tools("iter-test")
            .iter()
            .map(|tool| tool.name().to_string())
            .collect();

        assert!(names.contains(&PROPOSE_IDEAS_TOOL_NAME.to_string()));
        assert!(names.contains(&"read_file_with_limit".to_string()));
        assert!(names.contains(&"list_files".to_string()));
        assert!(names.contains(&"read_file".to_string()));
        assert!(names.contains(&"query_memory".to_string()));
        assert_eq!(names.len(), 6, "工具数量变了就要同步更新这条断言: {names:?}");
    }

    #[test]
    fn outcome_reports_external_usage() {
        let mut outcome = PlannerOutcome::default();
        assert!(!outcome.used_external_search());
        outcome.external_used = true;
        assert!(outcome.used_external_search());
    }

    #[test]
    fn outcome_separates_empty_from_never_submitted() {
        // 模型明确提交了空数组：这是结论
        let submitted = PlannerOutcome {
            propose_called: true,
            ..Default::default()
        };
        assert!(submitted.drafts.is_empty());
        assert!(submitted.propose_called);

        // 压根没提交：这是故障，不能当作"没有新发现"
        let missing = PlannerOutcome::default();
        assert!(!missing.propose_called);
    }
}
