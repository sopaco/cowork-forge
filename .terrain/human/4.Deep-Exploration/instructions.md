# Instructions 领域

**模块路径**：`crates/cowork-core/src/instructions/`
**生成日期**：2026-07-05

---

## 概述

Instructions 是 Cowork Forge 的"岗位说明书"集合——它包含所有 Agent 角色的提示词（Prompt）。每个 Agent 在创建时加载对应的指令，这些指令定义了 Agent 的角色定位、行为规则和产出要求。

---

## 核心功能点

1. **阶段 Actor 指令**——Idea、PRD、Design、Plan、Coding 的生成指令
2. **阶段 Critic 指令**——PRD、Design、Plan、Coding 的评审和反馈指令
3. **Check/Delivery 指令**——验证和交付阶段的专属指令
4. **PM Agent 指令**——交付后 PM Agent 的交互指令

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `IDEA_AGENT_INSTRUCTION` | `crates/cowork-core/src/instructions/idea.rs` | Idea Agent 提示词 |
| `PRD_ACTOR_INSTRUCTION` | `crates/cowork-core/src/instructions/prd.rs` | PRD Actor 生成指令 |
| `PRD_CRITIC_INSTRUCTION` | `crates/cowork-core/src/instructions/prd.rs` | PRD Critic 评审指令 |
| `CODING_ACTOR_INSTRUCTION` | `crates/cowork-core/src/instructions/coding.rs` | Coding Actor 编码指令 |
| `PROJECT_MANAGER_AGENT_INSTRUCTION` | `crates/cowork-core/src/instructions/project_manager.rs` | PM Agent 指令 |

---

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| agents | 被依赖 | Agent 工厂引用指令常量构建 Agent |
