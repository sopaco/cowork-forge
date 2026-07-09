# tools 模块深度报告

## 这个模块在做什么

Tools 模块是 Cowork Forge 的"工具箱"——它提供了 30 多个 ADK 标准工具，Agent 在"工作"时就用这些工具来操作文件、执行命令、读写数据、验证结果、与用户交互。没有这些工具，Agent 就像工人没有扳手和螺丝刀，只能看不能干。

## 核心功能点

1. **文件操作工具**——`ReadFileTool`、`WriteFileTool`、`ListFilesTool` 等，支持安全的工作区边界文件操作。代码位置：`crates/cowork-core/src/tools/file_tools.rs`
2. **人类在环工具**——`ReviewWithFeedbackContentTool`、`ProvideFeedbackTool` 等，在关键决策点暂停流水线请求人类确认。代码位置：`crates/cowork-core/src/tools/hitl_tools.rs`、`hitl_content_tools.rs`
3. **数据 CRUD 工具**——`CreateRequirementTool`、`CreateTaskTool`、`GetRequirementsTool`、`GetDesignTool`、`GetPlanTool` 等，管理迭代数据。代码位置：`crates/cowork-core/src/tools/data_tools.rs`
4. **验证工具**——`CheckFeatureCoverageTool`、`CheckTaskDependenciesTool`、`CheckDataFormatTool`、`CheckTestsTool`、`CheckLintTool`，质量把关。代码位置：`crates/cowork-core/src/tools/validation_tools.rs`
5. **Memory 工具**——`QueryMemoryTool`、`SaveInsightTool`、`SaveIssueTool`、`SaveLearningTool`、`PromoteToDecisionTool`、`PromoteToPatternTool`，跨迭代知识管理。代码位置：`crates/cowork-core/src/tools/memory_tools.rs`
6. **PM 工具**——`PMGotoStageTool`、`PMCreateIterationTool`、`PMRespondTool`、`PMSaveDecisionTool`，PM Agent 专用。代码位置：`crates/cowork-core/src/tools/pm_tools.rs`
7. **工具通知系统**——全局回调机制，实时广播工具调用到 GUI。代码位置：`crates/cowork-core/src/tools/mod.rs:36-104`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `ReadFileTool` | `crates/cowork-core/src/tools/file_tools.rs` | 读取工作区文件内容 |
| `WriteFileTool` | `crates/cowork-core/src/tools/file_tools.rs` | 写入文件到工作区 |
| `ExecuteShellCommandTool` | `crates/cowork-core/src/tools/test_lint_tools.rs` | 执行 Shell 命令（构建/测试等） |
| `QueryMemoryTool` | `crates/cowork-core/src/tools/memory_tools.rs` | 查询项目记忆 |
| `ReviewWithFeedbackContentTool` | `crates/cowork-core/src/tools/hitl_content_tools.rs` | 请求人类对内容给出反馈 |
| `CheckTestsTool` | `crates/cowork-core/src/tools/validation_tools.rs` | 检查测试是否通过 |
| `PMGotoStageTool` | `crates/cowork-core/src/tools/pm_tools.rs` | PM Agent 跳转到指定阶段 |
| `ToolNotifyFn` | `crates/cowork-core/src/tools/mod.rs:36` | 工具通知回调类型，GUI 实时显示工具调用 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| domain | 依赖 | 操作 Iteration 的制品和任务数据 |
| persistence | 依赖 | 保存和加载数据 |
| llm | 间接依赖 | 部分工具间接需要 LLM 功能 |

## 跨模块协作场景

**在 Coding 阶段**：Coding Actor 调用 `ReadFileTool` 读取现有代码 → `WriteFileTool` 写入新代码 → `ExecuteShellCommandTool` 运行构建和测试 → Coding Critic 调用 `ReadFileTool` 审查代码质量 → `ProvideFeedbackTool` 给出改进建议。

## 性能考量

文件操作使用 Tokio 异步封装，不会阻塞主线程。命令执行有超时控制，防止长时间运行的构建任务耗尽资源。工具通知系统设计为回调模式而非轮询，对性能影响极小。
