# Developer Meta Inputs

Compiled from `terrain-meta.json` before Agent context generation.

## Private knowledge (.terrain/knowledge/00-glossary.md)

_Source: `.terrain/knowledge/00-glossary.md`_



## Private knowledge (.terrain/knowledge/adk-rust.md)

_Source: `.terrain/knowledge/adk-rust.md` (truncated)_

# adk-rust 框架速查

> Agent Development Kit for Rust — LLM Agent 编排框架 (github.com/zavora-ai/adk-rust)

## 核心类型

| 类型 | 用途 |
|------|------|
| `Agent` (trait) | 可执行 agent 单元，`run(ctx) -> Result<RunResult>` |
| `LlmAgentBuilder` | 构建 LLM agent：`.instruction()` + `.tool()` + `.model()` |
| `LoopAgent` | 循环编排器，按顺序执行 agents 数组，支持多轮迭代 |
| `Tool` (trait) | 工具 trait：`name/description/parameters_schema/execute(ctx, args)` |
| `ToolContext` (Arc<dyn>) | 工具执行上下文（LLM 调用、session 操作、action 设置） |
| `EventActions` | 工具返回后的控制指令：`escalate` / `exit_loop` / `goto_stage` |
| `IncludeContents` | 子 agent 可见会话历史模式 |
| `Session` | 对话会话，存储 messages/state，agent 间共享 |
| `ExitLoopTool` | 内置工具：调用后设置 `actions.escalate = true`（注意：是 escalate 不是 exit_loop 字段！） |

## IncludeContents 模式

```rust
IncludeContents::None     // 子 agent 只看到自己的 instruction + 当前用户 turn（看不到前序/历史消息）
IncludeContents::Default  // 子 agent 看到共享 Session 的完整对话历史
```

**Actor-Critic 正确配置（易错！）**：
- **Actor** → `IncludeContents::Default`：Actor 需要看到前一轮 Critic 的文字反馈来修正产出
- **Critic** → `IncludeContents::None`：Critic **不需要**看 Actor 的对话历史！Critic 通过工具（`load_prd_doc`/`get_plan`/`list_files`等）从磁盘/persistence 加载 Actor 的 artifact 进行审查。设为 None 可避免将 Actor 的 system prompt + 完整工具调用链（可能 50K+ tokens）传给 Critic，节省约一半 token 成本。

**误区纠正**：旧说法"Critic 必须用 Default 才能看到 Actor 产出"是错误的。Critic 通过工具加载 artifact，不依赖对话历史。Default 仅用于 Actor 需要跨轮看到 Critic 反馈的场景。

## LoopAgent 工作流 (Actor-Critic)

```
LoopAgent::new("name", vec![actor_agent, critic_agent])
          .with_max_iterations(N)
```

执行流程：
1. LoopAgent 创建一个 `HistoryTrackingSession` 包裹父上下文
2. 每轮迭代依次执行 Actor → Critic，每个子 agent 的输出 event 都写入 HistoryTrackingSession
3. **Actor**（Default）：在第 2+ 轮迭代时能看到前一轮 Critic 的反馈文字，据此修正产出并保存 artifact
4. **Critic**（None）：每轮只看到自己的 instruction + 初始用户 prompt，通过工具加载最新 artifact 审查：
   - 通过 → Critic 调用 `exit_loop` 工具，循环终止（整个 LoopAgent 成功结束）
   - 小问题 → Critic 直接在文字回复中描述问题（不调用 provide_feedback），Actor 下轮可见
   - 大问题 → Critic 调用 `provide_feedback` 持久化反馈 + 退出循环，触发 Stage 级别重试
5. 达到 max_iterations 仍未 exit_loop → LoopAgent 正常结束，Stage executor 根据历史决定重试

**EventActions.escalate 的作用**：子 agent 工具中设置 `escalate=true` 会立即中断 LoopAgent 循环。`provide_feedback` 和 `exit_loop` 都会设置 escalate=true。区别是 provide_feedback 额外持久化了结构化反馈供 Stage executor 使用。

## EventActions 使用

```rust
// 在 Tool::execute 中设置 action
let mut actions = EventActions::default();
actions.escalate = true;   // 中断当前 LoopAgent/agent，回到上层
ctx.set_actions(actions);  // 必须调用 set_actions 才生效！
```

| 字段 | 作用 |
|------|------|
| `escalate` | 设置为 true 时立即退出 LoopAgent（provide_feedback/exit_loop 都用这个） |

**易错**：
- 创建 `EventActions` 后**必须调用 `ctx.set_actions(actions)`**
- `exit_loop` 字段在新版 adk-rust 中不是独立字段——ExitLoopTool 实际设置的是 `escalate=true`

## LlmAgentBuilder 构建

```rust
LlmAgentBuilder::new("agent_id")
    .instruction("你是...")           // 不是 system_prompt()！
    .model(model)                      // Arc<dyn Llm>
    .tool(Arc::new(MyTool))           // 不是 with_tool()！可多次调用
    .include_contents(IncludeContents::Default)
    .temperature(0.3)
    .build()
```

**易错**：
- 方法名是 `.instruction()` 不是 `.system_prompt()`，是 `.tool()` 不是 `.with_tool()`
- 忘记添加工具 → LLM 无法调用该工具
- `include_contents` 默认是 `None`
- instruction 中必须列出可用工具名和用途，否则 LLM 不知道何时调用

## Tool trait 实现

```rust
#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "做什么用" }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": { "param": 

…

