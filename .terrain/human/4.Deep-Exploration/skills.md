# Skills 领域

**模块路径**：`crates/cowork-core/src/skills/`
**生成日期**：2026-07-05

---

## 概述

Skills 模块实现了 agentskills.io 标准，允许通过 Skill 包向 Agent 注入特定领域的知识、工具和提示词。比如加载"React 开发技能包"，Coding Agent 就能自动获得 React 最佳实践。

---

## 核心功能点

1. **Skill 管理**——`SkillManager` 加载、索引和查询 Skill 文档。`crates/cowork-core/src/skills/manager.rs`
2. **Skill 注入**——`SkillInjector` 将匹配的 Skill 注入 Agent 指令。`crates/cowork-core/src/skills/mod.rs`
3. **Skill 发现**——从 `.skills/` 目录自动发现 Skill 文件。

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `SkillManager` | `crates/cowork-core/src/skills/manager.rs` | Skill 生命周期管理 |
| `SkillInjector` | `crates/cowork-core/src/skills/mod.rs` | Skill 注入 Agent 指令 |
| `SkillDocument` | `crates/cowork-core/src/skills/mod.rs` | Skill 标准化表示 |
