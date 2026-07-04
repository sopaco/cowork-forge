# Tools 领域

**模块路径**：`crates/cowork-core/src/tools/`
**生成日期**：2026-07-05

---

## 概述

Tools 模块是 Cowork Forge 的"工具箱"。它提供了 30 多个 ADK 标准工具，Agent 在执行任务时通过它们来操作文件、执行命令、管理数据、验证结果、与用户交互。没有这些工具，Agent 就像没有扳手的工人——只能看不能干。

Tools 模块的设计精妙之处在于**工具通知系统**（`crates/cowork-core/src/tools/mod.rs:36-104`）：它提供了一个全局回调机制，可以在工具调用前后通知 GUI 界面。这意味着 GUI 用户可以看到 Agent "正在做什么"——读哪个文件、执行什么命令——让 Agent 的工作过程变得透明可见。

---

## 核心功能点

1. **文件操作工具**——`ReadFileTool`、`WriteFileTool`、`ListFilesTool`，安全的工作区边界文件操作
2. **人类在环工具**——`ReviewWithFeedbackContentTool`、`ProvideFeedbackTool`，在关键决策点暂停请求人类确认
3. **数据 CRUD 工具**——`CreateRequirementTool`、`CreateTaskTool`、`GetRequirementsTool`、`GetDesignTool`、`GetPlanTool` 等，管理迭代数据
4. **验证工具**——`CheckFeatureCoverageTool`、`CheckTaskDependenciesTool`、`CheckDataFormatTool`、`CheckTestsTool`、`CheckLintTool`
5. **Memory 工具**——`QueryMemoryTool`、`SaveInsightTool`、`SaveIssueTool`、`SaveLearningTool`、`PromoteToDecisionTool`、`PromoteToPatternTool`
6. **PM 工具**——`PMGotoStageTool`、`PMCreateIterationTool`、`PMRespondTool`、`PMSaveDecisionTool`

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `ReadFileTool` | `crates/cowork-core/src/tools/file_tools.rs` | 读取工作区文件内容 |
| `WriteFileTool` | `crates/cowork-core/src/tools/file_tools.rs` | 写入文件到工作区 |
| `ExecuteShellCommandTool` | `crates/cowork-core/src/tools/test_lint_tools.rs` | 执行 Shell 命令（构建/测试） |
| `QueryMemoryTool` | `crates/cowork-core/src/tools/memory_tools.rs` | 查询项目记忆 |
| `ReviewWithFeedbackContentTool` | `crates/cowork-core/src/tools/hitl_content_tools.rs` | 请求人类对内容给出反馈 |
| `CheckTestsTool` | `crates/cowork-core/src/tools/validation_tools.rs` | 检查测试是否通过 |
| `PMGotoStageTool` | `crates/cowork-core/src/tools/pm_tools.rs` | PM Agent 跳转到指定阶段 |
| `ToolNotifyFn` | `crates/cowork-core/src/tools/mod.rs:36` | 工具通知回调类型，GUI 实时显示 |

---

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| domain | 依赖 | 操作 Iteration 的制品和任务数据 |
| persistence | 依赖 | 保存和加载数据 |
| llm | 间接依赖 | 部分工具间接需要 LLM 功能 |

---

## 跨模块协作场景

**在 Coding 阶段**：Coding Actor 调用 `ReadFileTool` 读取现有代码 → `WriteFileTool` 写入新代码 → `ExecuteShellCommandTool` 运行构建和测试 → Coding Critic 调用 `ReadFileTool` 审查代码质量 → `ProvideFeedbackTool` 给出改进建议。

---

## 性能考量

文件操作使用 Tokio 异步封装，不阻塞主线程。命令执行有超时控制。工具通知系统设计为回调模式，对性能影响极小。
