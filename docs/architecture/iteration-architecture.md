# 迭代架构

## 概述

Iteration（迭代）是 Cowork Forge 的核心概念，代表一个完整的软件开发周期。每个迭代都是独立的、可管理的单元，包含完整的开发生命周期和独立的制品。通过迭代机制，Cowork Forge 将复杂的软件开发任务分解为可追踪、可管理、可演进的单元。

### 迭代的价值

1. **可管理性**：将大型开发任务分解为小的、可管理的单元
2. **可继承性**：新迭代可以继承现有迭代的制品和代码
3. **可演进性**：支持功能增强、Bug 修复和架构重构
4. **可追溯性**：完整的版本历史和制品管理
5. **可回滚性**：可以回退到任意历史迭代

### 迭代的核心组件

每个迭代包含以下核心组件：

- **迭代元数据**：迭代的基本信息和状态
- **制品**：各阶段生成的文档和交付物
- **结构化数据**：需求、设计、计划等结构化信息
- **工作空间**：代码生成的临时目录
- **会话数据**：元数据和反馈历史
- **记忆系统**：迭代级的学习和决策

## 迭代类型

Cowork Forge 支持两种迭代类型：

### Genesis（起源迭代）

**定义**：从零开始的全新迭代

**特点**：
- 不继承任何现有迭代
- 创建全新的制品和代码
- 从 Idea 阶段开始
- 适用于新项目创建

**使用场景**：
- 创建新项目
- 重新开始项目开发
- 完全重构项目

### Evolution（演化迭代）

**定义**：基于现有迭代创建的迭代

**特点**：
- 继承基础迭代的制品和代码
- 可以从任意阶段开始
- 支持增量更新
- 保留历史上下文

**使用场景**：
- 功能增强
- Bug 修复
- 架构调整
- 代码优化

## 迭代生命周期

### 状态机

```
┌─────────┐
│  Draft  │  草稿状态
└────┬────┘
     │ start()
     ↓
┌─────────┐
│ Running │  运行中
└────┬────┘
     │ pause()
     ↓
┌─────────┐
│ Paused  │  已暂停
└────┬────┘
     │ resume()
     ↓
┌─────────┐
│ Running │  运行中
└────┬────┘
     │
     ├─→ complete() ─→ ┌────────────┐
     │                  │ Completed  │  已完成
     │                  └────────────┘
     │
     └─→ fail() ──────→ ┌────────────┐
                        │   Failed    │  失败
                        └────────────┘
```

### 状态说明

| 状态 | 说明 | 可执行操作 |
|------|------|-----------|
| **Draft** | 草稿状态，尚未开始执行 | `start()` |
| **Running** | 运行中，正在执行阶段 | `pause()`、`fail()`、`complete()` |
| **Paused** | 已暂停，等待继续 | `resume()` |
| **Completed** | 已完成，所有阶段执行成功 | 无（终端状态） |
| **Failed** | 失败，执行过程中出现错误 | 无（终端状态） |

### 状态转换规则

1. **Draft → Running**：调用 `start()` 开始执行
2. **Running → Paused**：调用 `pause()` 暂停执行
3. **Paused → Running**：调用 `resume()` 继续执行
4. **Running → Completed**：所有阶段成功完成
5. **Running → Failed**：执行过程中出现致命错误

## 迭代数据结构

### Iteration 实体

```rust
pub struct Iteration {
    pub id: String,                    // 迭代唯一标识
    pub project_id: String,            // 所属项目 ID
    pub name: String,                  // 迭代名称
    pub description: String,           // 迭代描述
    pub iteration_type: IterationType, // 迭代类型（Genesis/Evolution）
    pub base_iteration_id: Option<String>, // 基础迭代 ID（Evolution 类型）
    pub inheritance_mode: InheritanceMode, // 继承模式
    pub current_stage: Stage,          // 当前阶段
    pub status: IterationStatus,       // 迭代状态
    pub created_at: DateTime<Utc>,     // 创建时间
    pub updated_at: DateTime<Utc>,     // 更新时间
    pub started_at: Option<DateTime<Utc>>,   // 开始时间
    pub completed_at: Option<DateTime<Utc>>, // 完成时间
    pub change_description: String,    // 变更描述
}
```

### IterationType

```rust
pub enum IterationType {
    Genesis,   // 起源迭代
    Evolution, // 演化迭代
}
```

### InheritanceMode

```rust
pub enum InheritanceMode {
    None,     // 不继承，全新开始
    Full,     // 完全继承，复制所有 workspace 文件
    Partial,  // 部分继承，只复制 artifacts 和配置
}
```

### Stage

```rust
pub enum Stage {
    Idea,     // 创意阶段
    Prd,      // 需求阶段
    Design,   // 设计阶段
    Plan,     // 计划阶段
    Coding,   // 编码阶段
    Check,    // 检查阶段
    Delivery, // 交付阶段
}
```

### IterationStatus

```rust
pub enum IterationStatus {
    Draft,     // 草稿
    Running,   // 运行中
    Paused,    // 已暂停
    Completed, // 已完成
    Failed,    // 失败
}
```

## 迭代目录结构

### 完整目录结构

```
.cowork-v2/
└── iterations/
    └── {iteration_id}/           # 迭代 ID，如 "iter-2-1770536303"
        ├── iteration.json        # 迭代元数据
        ├── artifacts/            # 迭代制品（文档）
        │   ├── idea.md          # 创意文档
        │   ├── prd.md           # 产品需求文档
        │   ├── design.md        # 设计文档
        │   ├── plan.md          # 实施计划
        │   └── delivery.md      # 交付报告
        ├── data/                # 结构化数据
        │   ├── requirements.json      # 需求列表
        │   ├── feature_list.json      # 功能列表
        │   ├── design_spec.json       # 设计规范
        │   ├── implementation_plan.json # 实施计划
        │   └── code_metadata.json     # 代码元数据
        ├── session/              # 会话数据
        │   ├── meta.json        # 会话元数据
        │   └── feedback.json    # 反馈历史
        ├── workspace/           # 代码工作空间
        │   ├── src/            # 源代码
        │   ├── components/     # 组件
        │   ├── tests/          # 测试
        │   ├── docs/           # 文档
        │   └── config/         # 配置
        └── logs/               # 日志文件
            ├── idea.log
            ├── prd.log
            ├── design.log
            ├── plan.log
            ├── coding.log
            ├── check.log
            └── delivery.log
```

### 目录说明

| 目录 | 内容 | 用途 |
|------|------|------|
| **iteration.json** | 迭代元数据 | 存储迭代的基本信息、状态、配置 |
| **artifacts/** | 迭代制品 | 存储各阶段生成的文档 |
| **data/** | 结构化数据 | 存储需求、设计、计划等结构化信息 |
| **session/** | 会话数据 | 存储会话元数据和反馈历史 |
| **workspace/** | 工作空间 | 代码生成的临时目录 |
| **logs/** | 日志文件 | 存储各阶段的执行日志 |

## 继承机制

### 继承流程

```
┌─────────────────┐
│ 基础迭代 N-1    │
│  (已完成)       │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│ 选择继承模式    │
└────────┬────────┘
         │
         ├─→ None ──────────────┐
         │                      │ 创建新目录
         ├─→ Full ──────────────┤
         │                      │ 复制 workspace/
         └─→ Partial ───────────┤
                                │ 复制 artifacts/ 和 data/
                                ↓
                        ┌─────────────────┐
                        │  新迭代 N       │
                        │  (从指定阶段开始)│
                        └─────────────────┘
```

### 继承模式详解

#### None 模式（不继承）

**适用场景**：
- 完全重新开始
- 需要彻底重构
- 清理历史代码

**继承内容**：
- 无任何继承
- 创建全新的迭代目录
- 从 Idea 阶段开始

**示例**：
```rust
let iteration = IterationBuilder::new("New Project")
    .iteration_type(IterationType::Genesis)
    .inheritance_mode(InheritanceMode::None)
    .build()?;
```

#### Full 模式（完全继承）

**适用场景**：
- 小幅功能增强
- Bug 修复
- 代码优化

**继承内容**：
- 复制 `workspace/` 目录下的所有文件
- 复制 `artifacts/` 目录下的所有文档
- 复制 `data/` 目录下的所有结构化数据
- 可以从任意阶段开始

**示例**：
```rust
let iteration = IterationBuilder::new("Fix Login Bug")
    .iteration_type(IterationType::Evolution)
    .base_iteration_id("iter-1-1234567890")
    .inheritance_mode(InheritanceMode::Full)
    .start_stage(Stage::Coding)
    .build()?;
```

#### Partial 模式（部分继承）

**适用场景**：
- 大幅功能增强
- 架构调整
- 需要重新生成代码

**继承内容**：
- 复制 `artifacts/` 目录下的所有文档
- 复制 `data/` 目录下的所有结构化数据
- **不复制** `workspace/` 目录
- 通常从 Coding 或后续阶段开始

**示例**：
```rust
let iteration = IterationBuilder::new("Add User Authentication")
    .iteration_type(IterationType::Evolution)
    .base_iteration_id("iter-1-1234567890")
    .inheritance_mode(InheritanceMode::Partial)
    .start_stage(Stage::Design)
    .build()?;
```

### 继承实现

```rust
impl IterationStore {
    pub fn create_evolution_iteration(
        &self,
        base_iteration_id: &str,
        inheritance_mode: InheritanceMode,
        change_description: &str,
    ) -> Result<Iteration, CoworkError> {
        // 1. 创建新迭代目录
        let new_iteration_id = self.generate_iteration_id()?;
        let new_iteration_dir = self.iterations_dir.join(&new_iteration_id);
        fs::create_dir_all(&new_iteration_dir)?;

        // 2. 根据继承模式复制内容
        match inheritance_mode {
            InheritanceMode::None => {
                // 不继承任何内容
            }
            InheritanceMode::Full => {
                // 复制 workspace
                self.copy_workspace(&base_iteration_id, &new_iteration_id)?;
                // 复制 artifacts 和 data
                self.copy_artifacts_and_data(&base_iteration_id, &new_iteration_id)?;
            }
            InheritanceMode::Partial => {
                // 只复制 artifacts 和 data
                self.copy_artifacts_and_data(&base_iteration_id, &new_iteration_id)?;
            }
        }

        // 3. 创建新迭代记录
        let iteration = Iteration {
            id: new_iteration_id.clone(),
            iteration_type: IterationType::Evolution,
            base_iteration_id: Some(base_iteration_id.to_string()),
            inheritance_mode,
            change_description: change_description.to_string(),
            // ... 其他字段
        };

        Ok(iteration)
    }
}
```

## 迭代间协作

### 制品传递

迭代之间通过制品实现协作和上下文传递：

```
迭代 N-1 制品
    ↓
    ├─→ idea.md ─────────────────────┐
    ├─→ prd.md ──────────────────────┤
    ├─→ design.md ───────────────────┤
    ├─→ plan.md ─────────────────────┤
    ├─→ requirements.json ───────────┤
    ├─→ design_spec.json ────────────┤
    └─→ implementation_plan.json ────┤
                                   ↓
                            ┌──────────────┐
                            │  迭代 N      │
                            │  (继承制品)  │
                            └──────────────┘
```

### 增量更新

演化迭代支持增量更新，只修改受影响的部分：

1. **增量需求更新**：只修改需要变更的需求
2. **增量设计更新**：只调整受影响的组件
3. **增量计划更新**：只更新相关的任务
4. **增量代码生成**：只生成和修改必要的代码

### 反馈循环

迭代之间通过反馈机制实现协作：

1. **Check Agent 反馈**：发现问题后通过 `goto_stage()` 返回 Coding 阶段
2. **HITL 反馈**：用户通过 Pipeline 层提供反馈
3. **迭代间反馈**：从历史迭代中学习，避免重复错误

## 迭代管理

### 创建迭代

```rust
// 创建 Genesis 迭代
let iteration = iteration_store.create_genesis_iteration(
    "My First Project",
    "A web application for task management"
)?;

// 创建 Evolution 迭代
let iteration = iteration_store.create_evolution_iteration(
    "iter-1-1234567890",
    InheritanceMode::Partial,
    "Add user authentication feature"
)?;
```

### 启动迭代

```rust
// 从指定阶段启动
iteration_executor.start_iteration(
    &iteration_id,
    Some(Stage::Design)  // 从 Design 阶段开始
).await?;
```

### 暂停迭代

```rust
iteration_executor.pause_iteration(&iteration_id).await?;
```

### 继续迭代

```rust
iteration_executor.resume_iteration(&iteration_id).await?;
```

### 查询迭代状态

```rust
let iteration = iteration_store.load(&iteration_id)?;
println!("Current stage: {:?}", iteration.current_stage);
println!("Status: {:?}", iteration.status);
```

### 列出项目迭代

```rust
let iterations = iteration_store.list_by_project(&project_id)?;
for iteration in iterations {
    println!("{}: {} ({})", iteration.id, iteration.name, iteration.status);
}
```

## 迭代最佳实践

### 命名规范

- 使用清晰、描述性的名称
- 包含迭代的主要目标
- 示例："Add User Authentication"、"Fix Login Bug"、"Optimize Database Queries"

### 描述规范

- 详细描述变更内容
- 说明变更原因和目标
- 列出预期的影响

### 继承模式选择

| 场景 | 推荐模式 | 原因 |
|------|---------|------|
| 全新项目 | None | 从零开始 |
| Bug 修复 | Full | 保留现有代码 |
| 小功能增强 | Full | 继承现有代码 |
| 大功能增强 | Partial | 重新生成代码 |
| 架构调整 | Partial | 需要重新设计 |
| 性能优化 | Full | 基于现有代码优化 |

### 迭代粒度

- 保持迭代粒度适中
- 每个迭代专注于一个主要目标
- 避免在一个迭代中完成过多任务

### 版本管理

- 定期创建里程碑迭代
- 为重要版本打标签
- 保留关键迭代的完整备份

## 相关文档

- [架构概览](./overview.md)
- [Agent 系统](./agent-system.md)
- [Pipeline 流程](./pipeline.md)
- [领域模型](../development/domain.md)