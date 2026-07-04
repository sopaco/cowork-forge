# Persistence 领域

**模块路径**：`crates/cowork-core/src/persistence/`
**生成日期**：2026-07-05

---

## 概述

Persistence 是 Cowork Forge 的"文件柜"。它把系统中的所有数据（项目信息、迭代快照、项目记忆）写到硬盘上的 JSON 文件中。选择 JSON 文件而非数据库，是因为桌面工具追求"开箱即用"——用户不需要安装和配置数据库。数据文件都在 `.cowork-v2/` 目录下，方便 Git 管理和备份。

---

## 核心功能点

1. **ProjectStore**——保存和加载项目根信息。`crates/cowork-core/src/persistence/project_store.rs`
2. **IterationStore**——保存和加载迭代快照（包含进度、制品）。`crates/cowork-core/src/persistence/iteration_store.rs`
3. **MemoryStore**——保存和加载项目记忆（决策、模式、知识）。`crates/cowork-core/src/persistence/memory_store.rs`
4. **工作区路径管理**——支持 GUI 设置全局路径，解决 macOS 路径问题。`crates/cowork-core/src/persistence/mod.rs:20-35`

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `ProjectStore` | `crates/cowork-core/src/persistence/project_store.rs` | 项目根信息 JSON 读写 |
| `IterationStore` | `crates/cowork-core/src/persistence/iteration_store.rs` | 迭代快照 JSON 管理 |
| `MemoryStore` | `crates/cowork-core/src/persistence/memory_store.rs` | 项目记忆 JSON 持久化 |

---

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| domain | 依赖 | 持久化 domain 类型（Project/Iteration/Memory） |
| pipeline | 被依赖 | Pipeline 通过 Store 保存/加载数据 |
| tools | 被依赖 | 数据工具通过 Store 操作数据 |

---

## 数据目录结构

```
.cowork-v2/
├── project.json
├── iterations/
│   └── {iteration_id}.json
├── memory/
│   ├── project/project_memory.json
│   └── iterations/{iteration_id}.json
└── workspace/{iteration_id}/
```
