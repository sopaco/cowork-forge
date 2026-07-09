# instructions 模块深度报告

## 这个模块在做什么

Instructions 是 Cowork Forge 的"岗位说明书"集合——它包含了所有 Agent 角色的提示词（Prompt）。每个 Agent 在创建时都会加载对应角色的指令，这些指令定义了 Agent 的角色定位、行为规则、可用工具和目标输出。

## 核心功能点

1. **阶段 Actor 指令**——为 Idea、PRD、Design、Plan、Coding 阶段的 Actor Agent 提供生成指令。代码位置：`crates/cowork-core/src/instructions/idea.rs` 到 `coding.rs`
2. **阶段 Critic 指令**——为 PRD、Design、Plan、Coding 阶段的 Critic Agent 提供评审和反馈指令。
3. **Check/Delivery 指令**——验证和交付阶段的专属指令。
4. **PM Agent 指令**——项目交付后 PM Agent 的交互指令。代码位置：`crates/cowork-core/src/instructions/project_manager.rs`
5. **Summary/KG 指令**——迭代摘要和知识生成 Agent 的指令。代码位置：`crates/cowork-core/src/instructions/summary.rs`、`knowledge_gen.rs`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `IDEA_AGENT_INSTRUCTION` | `crates/cowork-core/src/instructions/idea.rs` | Idea Agent 的提示词 |
| `PRD_ACTOR_INSTRUCTION` | `crates/cowork-core/src/instructions/prd.rs` | PRD Actor 的生成指令 |
| `PRD_CRITIC_INSTRUCTION` | `crates/cowork-core/src/instructions/prd.rs` | PRD Critic 的评审指令 |
| `CODING_ACTOR_INSTRUCTION` | `crates/cowork-core/src/instructions/coding.rs` | Coding Actor 的编码指令 |
| `PROJECT_MANAGER_AGENT_INSTRUCTION` | `crates/cowork-core/src/instructions/project_manager.rs` | PM Agent 的交互指令 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| agents | 被依赖 | Agent 工厂引用指令常量构建 Agent |

## 跨模块协作场景

**在 Agent 创建时**：Agent 工厂调用 `create_prd_loop()` → 函数内引用 `PRD_ACTOR_INSTRUCTION` 和 `PRD_CRITIC_INSTRUCTION` → 将指令注入到 `LlmAgentBuilder` → Agent 执行时按指令规定的方式工作。
