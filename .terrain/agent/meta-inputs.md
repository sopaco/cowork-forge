# Developer Meta Inputs

Compiled from `terrain-meta.json` before Agent context generation.

## Private knowledge (.terrain\knowledge\00-glossary.md)

_Source: `.terrain/knowledge/00-glossary.md`_



## Private knowledge (.terrain\knowledge\adk-rust.md)

_Source: `.terrain/knowledge/adk-rust.md` (truncated)_

# adk-rust 框架速查

> Agent Development Kit for Rust — LLM Agent 编排框架 (github.com/zavora-ai/adk-rust)

## 核心类型

| 类型 | 用途 |
|------|------|
| `Agent` (trait) | 可执行 agent 单元，`run(ctx) -> Result<RunResult>` |
| `LlmAgentBuilder` | 构建 LLM 驱动的 agent（system prompt + tools + model） |
| `LoopAgent` | 循环编排器，按顺序执行 agents 数组，支持多轮迭代 |
| `Tool` (trait) | 工具 trait：`name/description/parameters_schema/execute(ctx, args)` |
| `ToolContext` (Arc<dyn>) | 工具执行上下文（LLM 调用、session 操作、action 设置） |
| `EventActions` | 工具返回后的控制指令：`escalate` / `exit_loop` / `goto_stage` |
| `IncludeContents` | 子 agent 可见会话历史模式 |
| `Session` | 对话会话，存储 messages/state，agent 间共享 |
| `ExitLoopTool` | 内置工具：调用后设置 `actions.exit_loop = true` |

## IncludeContents 模式

```rust
IncludeContents::None     // 子 agent 看不到父/前序 agent 的历史（默认）
IncludeContents::Default  // 共享会话历史（actor-critic 循环必须用此模式！）
```

**易错**：Actor-Critic 循环中 Critic 能看到 Actor 输出的前提是 `include_contents(IncludeContents::Default)`。设为 `None` 时 Critic 无法审查 Actor 的产出。

## LoopAgent 工作流 (Actor-Critic)

```
LoopAgent::new("name", vec![actor_agent, critic_agent])
          .with_max_iterations(N)
```

执行流程：
1. **Actor** 先运行（产出 artifact/内容），消息写入共享 Session
2. **Critic** 接着运行（读取 Session 中 Actor 的输出 + 历史），审查后：
   - 通过 → Critic 调用 `exit_loop` 工具，循环终止
   - 不通过 → Critic 调用 `provide_feedback` 写入反馈，继续下一轮
3. Actor 读取 feedback 后修改产出，Critic 再审，直到 exit_loop 或 max_iterations

**关键**：LoopAgent 中所有 agents **共享同一个 Session**。Actor 和 Critic 通过 Session 消息传递上下文，不是通过函数参数。

## EventActions 使用

```rust
// 在 Tool::execute 中设置 action
let mut actions = EventActions::default();
actions.escalate = true;   // HITL: 暂停执行，等待用户输入
ctx.set_actions(actions);  // 必须调用 set_actions 才生效！
```

| 字段 | 作用 |
|------|------|
| `escalate` | 升级到人工，暂停当前 agent 等待用户交互（HITL 工具用） |
| `exit_loop` | 退出 LoopAgent 循环（ExitLoopTool 自动设置，无需手动） |

**易错**：创建 `EventActions` 后**必须调用 `ctx.set_actions(actions)`**，仅创建 struct 不设置不会生效。

## Tool trait 实现

```rust
#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "做什么用" }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": { "param": {"type": "string"} },
            "required": ["param"]
        }))
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let param = args.get("param").and_then(|v| v.as_str())
            .ok_or_else(|| adk_core::AdkError::tool("param required"))?;
        Ok(json!({"result": "ok"}))
    }
}
```

**要点**：
- 参数校验失败返回 `Err(adk_core::AdkError::tool(...))`
- 成功返回 `Ok(json!(...))`
- 不要 panic/unwrap，用 `?` 传播错误
- 工具注册到 agent 后才能被 LLM 调用

## LlmAgentBuilder 构建

```rust
LlmAgentBuilder::new("agent_id", llm)  // llm: Arc<dyn Llm>
    .system_prompt("你是...")
    .with_tool(Arc::new(MyTool))       // 可多次调用添加工具
    .include_contents(IncludeContents::Default)  // 共享历史
    .temperature(0.3)
    .build()
```

**易错**：
- 忘记添加工具 → LLM 无法调用该工具
- `include_contents` 默认是 `None`，actor-critic 场景必须显式设为 `Default`
- system_prompt 中必须**列出可用工具名和用途**，否则 LLM 不知道何时调用

## 常见陷阱 (Pitfalls)

1. **Actor-Critic 看不到彼此输出**：检查 `include_contents` 是否为 `Default`
2. **exit_loop 不生效**：Critic 的工具列表中必须注册 `ExitLoopTool`（`"exit_loop"`）
3. **EventActions 不生效**：必须调用 `ctx.set_actions(actions)`，不是仅创建 struct
4. **工具永远不被调用**：检查 ① 是否注册到 agent ② prompt 中是否提及该工具
5. **Critic 无法读取 Actor 产物**：必须通过 Session 中的消息历史（`include_contents=Default`）或通过专门的 load 工具（如 `load_prd_doc`）从磁盘读取
6. **HITL 工具绕过 Intera

…

