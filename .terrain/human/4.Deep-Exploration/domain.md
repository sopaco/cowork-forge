# Domain 领域

**模块路径**：`crates/cowork-core/src/domain/`
**生成日期**：2026-07-05

---

## 概述

Domain 是 Cowork Forge 的"骨架"——它定义了系统中最重要的几个概念：项目（Project）、迭代（Iteration）、记忆（Memory）。这些是所有其他模块操作的基础。特别值得一提的是**迭代继承**（InheritanceMode），这是 Cowork Forge 的核心创新：每个迭代都是一个独立的开发周期，可以"继承"前一个迭代的代码或知识，让增量开发变得自然。

---

## 核心功能点

1. **Project 实体**——根聚合，管理名称、迭代列表、状态。代码位置：`crates/cowork-core/src/domain/project.rs:6`
2. **Iteration 实体**——开发周期实体，Genesis/Evolution 创建、状态流转（Draft→Running→Paused→Completed→Failed）。代码位置：`crates/cowork-core/src/domain/iteration.rs:8`
3. **InheritanceMode**——三种继承策略：None（全新）、Full（完整复制）、Partial（只复制制品）。代码位置：`crates/cowork-core/src/domain/iteration.rs`
4. **ProjectMemory 记忆系统**——跨迭代知识累积（Decisions、Patterns、Context），支持关键词查询和快照管理。代码位置：`crates/cowork-core/src/domain/memory.rs:7`

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `Project` | `crates/cowork-core/src/domain/project.rs:6` | 项目根实体 |
| `Iteration` | `crates/cowork-core/src/domain/iteration.rs:8` | 开发周期实体 |
| `InheritanceMode` | `crates/cowork-core/src/domain/iteration.rs` | 迭代继承策略枚举 |
| `ProjectMemory` | `crates/cowork-core/src/domain/memory.rs:7` | 跨迭代记忆系统 |
| `IterationKnowledge` | `crates/cowork-core/src/domain/memory.rs:83` | 单次迭代知识快照 |

---

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| persistence | 被依赖 | 保存和加载 Project/Iteration/Memory |
| pipeline | 被依赖 | 读取和更新 Iteration 状态 |
| agents | 被依赖 | 需要访问 Project/Iteration 上下文 |
