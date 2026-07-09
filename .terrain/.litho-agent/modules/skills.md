# skills 模块深度报告

## 这个模块在做什么

Skills 模块是 Cowork Forge 的"技能培训中心"——它实现了 agentskills.io 标准，允许通过 Skill 包向 Agent 注入特定领域的知识、工具和提示词。比如，如果项目需要构建 React 前端，可以加载一个"React 开发技能包"，让 Coding Agent 自动获得 React 相关的最佳实践和工具。

## 核心功能点

1. **Skill 管理**——`SkillManager` 管理 Skill 文档的加载、索引和查询。代码位置：`crates/cowork-core/src/skills/manager.rs`
2. **Skill 注入**——`SkillInjector` 负责将匹配的 Skill 注入到 Agent 的指令中。代码位置：`crates/cowork-core/src/skills/mod.rs`
3. **Skill 发现**——支持从 `.skills/` 目录自动发现 Skill 文件。代码位置：`crates/cowork-core/src/skills/mod.rs`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `SkillManager` | `crates/cowork-core/src/skills/manager.rs` | Skill 生命周期管理（加载、索引、查询） |
| `SkillInjector` | `crates/cowork-core/src/skills/mod.rs` | Skill 注入器，将匹配的 Skill 插入 Agent 指令 |
| `SkillDocument` | `crates/cowork-core/src/skills/mod.rs` | Skill 文档的标准化表示 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| agents | 被依赖 | Agent 创建时可以注入匹配的 Skill |
| config_definition | 被依赖 | Skill 配置可以在 ConfigRegistry 中注册 |

## 跨模块协作场景

**在 Agent 创建过程中**：Agent Factory 创建 Agent 时 → 调用 SkillManager 查询匹配的 Skill → 调用 SkillInjector 将 Skill 内容注入 Agent 指令 → Agent 执行时使用注入的知识和工具。
