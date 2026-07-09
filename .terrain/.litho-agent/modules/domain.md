# domain 模块深度报告

## 这个模块在做什么

Domain 是 Cowork Forge 的"骨架"——它定义了系统里最重要的几个概念：项目（Project）、迭代（Iteration）、记忆（Memory）。就像盖房子先要画蓝图，这些核心实体是所有其他模块的操作基础。特别值得一提的是"迭代"（Iteration）的概念——它不是普通意义上的"重复"，而是 Cowork Forge 的核心创新：每个迭代都是一个独立的开发周期，可以"继承"前一个迭代的代码或知识。

## 核心功能点

1. **Project 实体**——根聚合，管理项目名称、迭代列表、当前迭代 ID。支持添加迭代、设置当前迭代、获取最新完成的迭代。代码位置：`crates/cowork-core/src/domain/project.rs:6-50`
2. **Iteration 实体**——核心聚合，代表一个完整的开发周期。支持 Genesis（首次）和 Evolution（演化）两种创建方式，管理状态流转（Draft→Running→Paused→Completed→Failed）和阶段进度。代码位置：`crates/cowork-core/src/domain/iteration.rs:8-100`
3. **InheritanceMode 继承模式**——定义了迭代间的三种继承策略：None（全新开始）、Full（完整复制代码+制品）、Partial（只复制制品，代码重新生成）。代码位置：`crates/cowork-core/src/domain/iteration.rs`
4. **ProjectMemory 记忆系统**——跨迭代的知识累积，包括决策（Decisions）、模式（Patterns）、项目上下文（Context）。支持按关键词查询和历史知识快照管理。代码位置：`crates/cowork-core/src/domain/memory.rs:7-80`
5. **Artifacts 制品集合**——每次迭代的产出物容器，包括 idea.md、prd.md、design.md、plan.md、delivery_report.md 等。代码位置：`crates/cowork-core/src/domain/iteration.rs`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `Project` | `crates/cowork-core/src/domain/project.rs:6` | 项目根实体，管理名称、迭代清单和状态 |
| `Iteration` | `crates/cowork-core/src/domain/iteration.rs:8` | 开发周期实体，管理状态、阶段进度、制品 |
| `InheritanceMode` | `crates/cowork-core/src/domain/iteration.rs` | 迭代继承策略枚举（None/Full/Partial） |
| `Artifacts` | `crates/cowork-core/src/domain/iteration.rs` | 迭代产出物容器 |
| `ProjectMemory` | `crates/cowork-core/src/domain/memory.rs:7` | 项目级跨迭代记忆系统 |
| `IterationKnowledge` | `crates/cowork-core/src/domain/memory.rs:83` | 单次迭代的知识快照 |
| `Decision` | `crates/cowork-core/src/domain/memory.rs` | 项目的关键决策记录 |
| `Pattern` | `crates/cowork-core/src/domain/memory.rs` | 项目中发现的可复用模式 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| persistence | 被依赖 | 持久化模块负责保存和加载 Project/Iteration/Memory |
| pipeline | 被依赖 | Pipeline 读取和更新 Iteration 状态 |
| agents | 被依赖 | Agent 需要访问 Project 和 Iteration 上下文数据 |
| tools | 间接 | 数据工具通过 domain 类型操作迭代数据 |

## 跨模块协作场景

**在迭代执行过程中**：Pipeline 读取 Project 获取当前迭代 ID → 从 Persistence 加载 Iteration → 执行阶段并更新 Iteration 状态 → 保存 Iteration 到 Persistence → 每次迭代完成后 Knowledge Generation Agent 提取关键决策和模式 → 保存到 ProjectMemory。
