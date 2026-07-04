# persistence 模块深度报告

## 这个模块在做什么

Persistence 是 Cowork Forge 的"文件柜"——它负责把系统中的所有数据（项目信息、迭代快照、项目记忆）写到硬盘上的 JSON 文件中。之所以不用数据库而用 JSON 文件，是因为桌面工具追求的是"开箱即用"——用户不需要安装和配置数据库就能用。

## 核心功能点

1. **ProjectStore**——保存和加载项目根信息（Project 实体）。代码位置：`crates/cowork-core/src/persistence/project_store.rs`
2. **IterationStore**——保存和加载每次迭代的完整快照（Iteration 实体），包括迭代历史、阶段进度、制品。代码位置：`crates/cowork-core/src/persistence/iteration_store.rs`
3. **MemoryStore**——保存和加载项目级记忆（ProjectMemory），包括决策、模式、迭代知识快照。代码位置：`crates/cowork-core/src/persistence/memory_store.rs`
4. **IterationData**——迭代数据的结构和序列化。代码位置：`crates/cowork-core/src/persistence/iteration_data.rs`
5. **工作区路径管理**——支持 GUI 模式下设置全局工作区路径，解决 macOS 应用包启动时的路径问题。代码位置：`crates/cowork-core/src/persistence/mod.rs:20-35`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `ProjectStore` | `crates/cowork-core/src/persistence/project_store.rs` | 项目根信息的 JSON 文件读写 |
| `IterationStore` | `crates/cowork-core/src/persistence/iteration_store.rs` | 迭代快照的 JSON 文件管理 |
| `MemoryStore` | `crates/cowork-core/src/persistence/memory_store.rs` | 项目记忆的 JSON 文件持久化 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| domain | 依赖 | 持久化 domain 类型（Project/Iteration/ProjectMemory） |
| pipeline | 被依赖 | Pipeline 通过 Store 保存和加载项目/迭代数据 |
| tools | 被依赖 | 数据工具通过 Store 操作迭代数据 |

## 跨模块协作场景

**在迭代创建和保存过程中**：`IterationExecutor.create_genesis_iteration()` 创建 Iteration 实体 → 调用 `IterationStore.save()` 写 JSON 文件 → 调用 `ProjectStore.add_iteration()` 更新项目信息。整个流程是"内存中构建对象 → 序列化 JSON → 写入文件"的三步模式。
