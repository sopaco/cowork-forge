# Agent 系统

## 概述

Cowork Forge 的 Agent 系统基于 [adk-rust](https://adk-rust.com/) 框架构建，提供了一套专业化、可协作的 AI 智能体，模拟真实开发团队的角色分工。每个 Agent 都有明确的职责、专用的工具和优化的指令，能够独立完成特定阶段的开发任务。

### 设计理念

Agent 系统遵循以下设计原则：

1. **角色专业化**：每个 Agent 专注于特定领域，职责清晰
2. **权限最小化**：只提供完成任务所需的最小工具集
3. **质量保证**：复杂阶段采用 Actor-Critic 循环确保产出质量
4. **协作机制**：通过共享记忆和制品实现 Agent 间协作
5. **可扩展性**：基于 adk-rust 框架，易于扩展和定制

### Agent 类型

Cowork Forge 有 8 个专业智能体，分为两种类型：

| 类型 | Agent | 说明 |
|------|-------|------|
| **单一 Agent** | Idea Agent | 无 Critic，直接生成内容 |
| **单一 Agent** | Check Agent | 无 Critic，执行质量检查 |
| **单一 Agent** | Delivery Agent | 无 Critic，生成交付报告 |
| **LoopAgent** | PRD Loop | Actor + Critic，最多 1 轮迭代 |
| **LoopAgent** | Design Loop | Actor + Critic，最多 1 轮迭代 |
| **LoopAgent** | Plan Loop | Actor + Critic，最多 1 轮迭代 |
| **LoopAgent** | Coding Loop | Actor + Critic，最多 5 轮迭代 |

## adk-rust 框架

### 框架简介

adk-rust 是一个 Rust AI Agent 框架，提供了构建复杂 Agent 系统所需的核心功能：

- **LlmAgent**：单一智能体，支持工具调用和自定义指令
- **LoopAgent**：循环智能体，支持多轮迭代和多智能体协作
- **工具系统**：类型安全的工具接口，支持异步执行
- **上下文管理**：自动管理对话历史和上下文传递
- **错误处理**：完善的错误处理和重试机制

### 创建单一 Agent

```rust
use adk_core::agent::LlmAgentBuilder;
use std::sync::Arc;

let agent = LlmAgentBuilder::new("idea_agent")
    .instruction(IDEA_AGENT_INSTRUCTION)
    .model(model)
    .tool(Arc::new(SaveIdeaTool))
    .include_contents(IncludeContents::None)
    .build()?;
```

### 创建 LoopAgent

```rust
use adk_core::agent::LoopAgent;

let loop_agent = LoopAgent::new(
    "prd_loop",
    vec![Arc::new(prd_actor), Arc::new(prd_critic)]
).with_max_iterations(1);
```

**重要设计决策**：

- **不使用 SequentialAgent**：避免 ExitLoopTool 终止外层 SequentialAgent 的问题
- **max_iterations=1**：为未来调优准备，当前保持简洁
- **ExitLoopTool 配置**：支持未来扩展多轮迭代

## Agent 详细说明

### Idea Agent

**类型**：单一 Agent（无 Critic）

**职责**：理解用户创意，生成 idea.md 文档

**工作流程**：
1. 分析用户输入的创意描述和变更历史
2. 生成结构化的 idea.md 文档，包含：
   - 项目背景和目标
   - 核心功能概述
   - 目标用户群体
   - 技术方向和约束
3. 调用 `save_idea()` 保存最终文档
4. 完成（Pipeline 层验证 Artifacts 并触发 HITL 用户审查）

**工具清单**：
- `SaveIdeaTool`：保存 idea.md 到 artifacts 目录
- `QueryMemoryTool`（可选）：查询迭代记忆
- `SaveInsightTool`（可选）：保存洞见到记忆系统

**指令文件**：`crates/cowork-core/src/instructions/idea.rs`

### PRD Loop

**类型**：LoopAgent（Actor + Critic）

**职责**：生成产品需求文档，创建结构化需求

#### PRD Actor

**工作流程**：
1. 调用 `load_idea()` 读取 idea.md 获取项目背景
2. 分析需求，创建结构化需求列表（Requirements）
3. 为每个需求添加功能特性（Features）
4. 生成 PRD 文档（markdown 格式）
5. 调用 `save_prd_doc()` 保存最终文档

**工具清单**：
- `LoadIdeaTool`：读取 idea.md 文档
- `CreateRequirementTool`：创建需求条目
- `AddFeatureTool`：添加功能特性
- `UpdateRequirementTool`：更新需求（增量更新）
- `UpdateFeatureTool`：更新功能（增量更新）
- `DeleteRequirementTool`：删除需求（增量更新）
- `GetRequirementsTool`：读取需求列表
- `SavePrdDocTool`：保存 prd.md 文档

#### PRD Critic

**工作流程**：
1. 调用 `get_requirements()` 读取结构化需求
2. 调用 `load_idea()` 验证与创意文档的一致性
3. 检查需求完整性、优先级合理性、验收标准明确性
4. 提供反馈或通过验证

**工具清单**：
- `GetRequirementsTool`：读取结构化需求
- `LoadIdeaTool`：读取 idea.md 验证一致性
- `ProvideFeedbackTool`：提供反馈

**指令文件**：`crates/cowork-core/src/instructions/prd.rs`

### Design Loop

**类型**：LoopAgent（Actor + Critic）

**职责**：设计系统架构，创建设计规范

#### Design Actor

**工作流程**：
1. 调用 `get_requirements()` 读取需求列表
2. 调用 `load_prd_doc()` 读取 PRD 文档
3. 设计系统架构（技术栈选择、模块划分、接口设计）
4. 创建设计组件（DesignComponents）
5. 生成 Design 文档（markdown 格式）
6. 调用 `save_design_doc()` 保存最终文档

**工具清单**：
- `GetRequirementsTool`：读取需求列表
- `GetDesignTool`：读取设计规范
- `LoadPrdDocTool`：读取 PRD 文档
- `CreateDesignComponentTool`：创建设计组件
- `SaveDesignDocTool`：保存 design.md 文档

#### Design Critic

**工作流程**：
1. 调用 `get_requirements()` 读取需求
2. 调用 `get_design()` 读取设计规范
3. 调用 `load_design_doc()` 验证 markdown 文档
4. 检查功能覆盖度、架构合理性、接口完整性
5. 提供反馈或通过验证

**工具清单**：
- `GetRequirementsTool`：读取需求列表
- `GetDesignTool`：读取设计规范
- `LoadDesignDocTool`：读取 design.md 验证
- `CheckFeatureCoverageTool`：检查功能覆盖度
- `ProvideFeedbackTool`：提供反馈

**指令文件**：`crates/cowork-core/src/instructions/design.rs`

### Plan Loop

**类型**：LoopAgent（Actor + Critic）

**职责**：制定实施计划，分解任务

#### Plan Actor

**工作流程**：
1. 调用 `get_requirements()` 读取需求
2. 调用 `get_design()` 读取设计规范
3. 调用 `load_prd_doc()` 读取 PRD 文档
4. 调用 `load_design_doc()` 读取 Design 文档
5. 分解实施任务（Tasks），设置优先级和依赖关系
6. 制定里程碑（Milestones）
7. 生成 Plan 文档（markdown 格式）
8. 调用 `save_plan_doc()` 保存最终文档

**工具清单**：
- `GetRequirementsTool`：读取需求列表
- `GetDesignTool`：读取设计规范
- `GetPlanTool`：读取实施计划
- `LoadPrdDocTool`：读取 PRD 文档
- `LoadDesignDocTool`：读取 Design 文档
- `CreateTaskTool`：创建任务
- `SavePlanDocTool`：保存 plan.md 文档

#### Plan Critic

**工作流程**：
1. 调用 `get_plan()` 读取实施计划
2. 调用 `get_requirements()` 读取需求
3. 调用 `load_plan_doc()` 验证 markdown 文档
4. 检查任务依赖关系、实施可行性、时间估算
5. 提供反馈或通过验证

**工具清单**：
- `GetPlanTool`：读取实施计划
- `GetRequirementsTool`：读取需求列表
- `LoadPlanDocTool`：读取 plan.md 验证
- `CheckTaskDependenciesTool`：检查任务依赖
- `ProvideFeedbackTool`：提供反馈

**指令文件**：`crates/cowork-core/src/instructions/plan.rs`

### Coding Loop

**类型**：LoopAgent（Actor + Critic，最多 5 轮迭代）

**职责**：生成代码，实现功能

#### Coding Actor

**工作流程**：
1. 调用 `get_plan()` 读取任务列表（结构化数据，非 LoadPlanDoc）
2. 选择下一个待执行任务
3. 读取现有代码文件（ReadFile）
4. 生成/更新代码文件（WriteFile，在 workspace 内）
5. 列出工作空间文件（ListFiles）
6. 运行测试验证（RunCommand + CheckTests）
7. 更新任务状态（UpdateTaskStatus）
8. 更新功能状态（UpdateFeatureStatus）

**工具清单**：
- `GetPlanTool`：读取任务列表（结构化数据）
- `UpdateTaskStatusTool`：更新任务状态
- `UpdateFeatureStatusTool`：更新功能状态
- `ReadFileTool`：读取代码文件（从 workspace）
- `WriteFileTool`：写入代码文件（到 workspace）
- `ListFilesTool`：列出工作空间文件
- `RunCommandTool`：运行测试和构建命令（在 workspace）
- `CheckTestsTool`：检查测试通过率

#### Coding Critic

**工作流程**：
1. 调用 `get_plan()` 读取任务列表
2. 读取代码文件验证实现
3. 运行测试检查功能正确性
4. 提供代码审查反馈

**工具清单**：
- `GetPlanTool`：读取任务列表
- `ReadFileTool`：读取代码文件（从 workspace）
- `ListFilesTool`：列出工作空间文件
- `RunCommandTool`：运行测试验证（在 workspace）
- `ProvideFeedbackTool`：提供反馈

**指令文件**：`crates/cowork-core/src/instructions/coding.rs`

### Check Agent

**类型**：单一 Agent

**职责**：质量检查，验证功能完整性

**工作流程**：
1. 调用 `get_requirements()` 读取需求列表
2. 调用 `get_design()` 读取设计规范
3. 调用 `get_plan()` 读取任务列表
4. 检查数据格式完整性（CheckDataFormat）
5. 检查功能覆盖度（CheckFeatureCoverage）
6. 检查任务依赖关系（CheckTaskDependencies）
7. 读取代码文件和列出文件（从 workspace）
8. 运行测试（CheckTests）
9. 运行代码检查（CheckLint）
10. 综合评估：
    - 如果全部通过，进入 Delivery 阶段
    - 如果发现问题，调用 `goto_stage()` 返回 Coding 阶段修复

**工具清单**：
- `GetRequirementsTool`：读取需求列表
- `GetDesignTool`：读取设计规范
- `GetPlanTool`：读取任务列表
- `CheckDataFormatTool`：检查数据格式
- `CheckFeatureCoverageTool`：检查功能覆盖度
- `CheckTaskDependenciesTool`：检查任务依赖
- `RunCommandTool`：运行测试和构建
- `ReadFileTool`：读取代码文件（从 workspace）
- `ListFilesTool`：列出工作空间文件
- `CheckTestsTool`：检查测试通过率
- `CheckLintTool`：运行代码检查
- `ProvideFeedbackTool`：提供反馈
- `GotoStageTool`：返回之前阶段

**指令文件**：`crates/cowork-core/src/instructions/check.rs`

### Delivery Agent

**类型**：单一 Agent

**职责**：生成交付报告，部署代码

**工作流程**：
1. 调用 `get_requirements()` 读取需求列表
2. 调用 `get_design()` 读取设计规范
3. 调用 `get_plan()` 读取任务列表
4. 调用 `load_feedback_history()` 读取反馈历史
5. 调用 `load_idea()` 读取 idea.md
6. 调用 `load_prd_doc()` 读取 prd.md
7. 调用 `load_design_doc()` 读取 design.md
8. 列出项目文件验证交付物
9. 生成交付报告（delivery.md），包含：
    - 功能实现清单
    - 测试覆盖率
    - 已知问题和限制
    - 使用说明
10. 调用 `save_delivery_report()` 保存报告
11. 调用 `copy_workspace_to_project(confirm=true)` 部署代码到项目根目录

**工具清单**：
- `GetRequirementsTool`：读取需求列表
- `GetDesignTool`：读取设计规范
- `GetPlanTool`：读取任务列表
- `LoadFeedbackHistoryTool`：读取反馈历史
- `ListFilesTool`：列出项目文件
- `LoadIdeaTool`：读取 idea.md
- `LoadPrdDocTool`：读取 prd.md
- `LoadDesignDocTool`：读取 design.md
- `SaveDeliveryReportTool`：保存 delivery.md
- `CopyWorkspaceToProjectTool`：复制代码到项目根目录

**指令文件**：`crates/cowork-core/src/instructions/delivery.rs`

## Actor-Critic 循环机制

### 工作原理

Actor-Critic 循环是一种双智能体协作模式，通过 Actor 生成内容、Critic 验证质量的循环确保产出质量。

```
┌─────────────┐
│   Actor     │  生成内容
└──────┬──────┘
       │
       ↓
┌─────────────┐
│   Critic    │  验证质量
└──────┬──────┘
       │
       ↓
   通过？
   ├─ Yes → 完成
   └─ No  → 提供反馈 → Actor 改进
```

### 循环控制

| Agent 类型 | max_iterations | 说明 |
|-----------|----------------|------|
| PRD Loop | 1 | 为未来调优准备 |
| Design Loop | 1 | 为未来调优准备 |
| Plan Loop | 1 | 为未来调优准备 |
| Coding Loop | 5 | 允许多轮迭代完善代码 |

### Critic 验证逻辑

Critic 使用以下标准验证 Actor 的产出：

1. **完整性检查**：是否包含所有必需的组件
2. **一致性检查**：是否与前置阶段的内容一致
3. **质量检查**：是否符合最佳实践和质量标准
4. **可行性检查**：是否可实现和维护

### 反馈机制

Critic 可以通过以下方式提供反馈：

1. **直接反馈**：通过 `ProvideFeedbackTool` 提供文本反馈
2. **结构化反馈**：通过更新结构化数据（如 Requirements）提供反馈
3. **跳转反馈**：通过 `GotoStageTool` 跳转到之前阶段

## 工具配置

### 工具权限矩阵

| 阶段 | ReadFile | WriteFile | LoadIdea | LoadPrd | LoadDesign | LoadPlan | SaveXxx | ListFiles | RunCommand | GetRequirements | GetDesign | GetPlan | CreateXxx | UpdateXxx | CheckXxx | CopyWorkspace |
|------|----------|-----------|----------|---------|------------|---------|---------|-----------|------------|-----------------|-------------|----------|-----------|-----------|----------|----------------|
| Idea | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ save_idea | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| PRD Actor | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ save_prd_doc | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ create_requirement/add_feature | ✅ update_requirement/update_feature/delete_requirement | ❌ | ❌ |
| PRD Critic | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Design Actor | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ save_design_doc | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ create_design_component | ❌ | ❌ | ❌ |
| Design Critic | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ check_feature_coverage | ❌ |
| Plan Actor | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ save_plan_doc | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ create_task | ❌ | ❌ | ❌ |
| Plan Critic | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ check_task_dependencies | ❌ |
| Coding Actor | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ update_task/update_feature | ✅ check_tests | ❌ |
| Coding Critic | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Check Agent | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ check_data_format/check_feature_coverage/check_task_dependencies/check_tests/check_lint | ❌ |
| Delivery Agent | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ save_delivery_report | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ copy_workspace |

### 工具注册

所有工具在 `crates/cowork-core/src/agents/mod.rs` 中注册：

```rust
use crate::tools::*;

// Idea Agent
.tool(Arc::new(SaveIdeaTool))

// PRD Agent
.tool(Arc::new(LoadIdeaTool))
.tool(Arc::new(CreateRequirementTool))
.tool(Arc::new(AddFeatureTool))
.tool(Arc::new(SavePrdDocTool))

// Delivery Agent
.tool(Arc::new(CopyWorkspaceToProjectTool))
```

## 指令系统

### 指令文件结构

每个 Agent 都有对应的指令文件，位于 `crates/cowork-core/src/instructions/` 目录：

```
instructions/
├── idea.rs         # Idea Agent 指令
├── prd.rs          # PRD Agent 指令
├── design.rs       # Design Agent 指令
├── plan.rs         # Plan Agent 指令
├── coding.rs       # Coding Agent 指令
├── check.rs        # Check Agent 指令
└── delivery.rs     # Delivery Agent 指令
```

### 指令模板

指令模板包含以下部分：

1. **角色定义**：Agent 的身份和职责
2. **任务描述**：具体要完成的任务
3. **工作流程**：详细的执行步骤
4. **输出格式**：期望的输出格式
5. **工具使用说明**：如何使用可用工具
6. **质量标准**：产出的质量要求

### 指令最佳实践

- **清晰明确**：指令应该清晰、具体、无歧义
- **步骤化**：将复杂任务分解为简单步骤
- **示例驱动**：提供示例帮助 Agent 理解期望
- **强调工具使用**：明确指示 Agent 必须调用哪些工具
- **质量优先**：强调质量而非速度

## 协作机制

### 记忆共享

所有 Agent 通过共享的记忆系统获取上下文：

- **项目级记忆**：存储项目级别的决策和学习
- **迭代级记忆**：存储特定迭代的学习和决策

### 制品传递

Agent 通过读取和写入制品实现协作：

1. **Idea Agent** 生成 `idea.md`
2. **PRD Agent** 读取 `idea.md`，生成 `prd.md` 和 `requirements.json`
3. **Design Agent** 读取 `idea.md` 和 `prd.md`，生成 `design.md` 和 `design_spec.json`
4. **Plan Agent** 读取前置制品，生成 `plan.md` 和 `implementation_plan.json`
5. **Coding Agent** 读取结构化数据，在 workspace 生成代码
6. **Check Agent** 验证所有制品和代码
7. **Delivery Agent** 整合所有信息，生成交付报告

### 反馈循环

Agent 之间通过反馈机制实现协作：

1. **Actor-Critic 反馈**：Critic 向 Actor 提供质量反馈
2. **跨阶段反馈**：通过 `GotoStageTool` 跳转到之前阶段
3. **HITL 反馈**：用户通过 Pipeline 层提供反馈

## 相关文档

- [架构概览](./overview.md)
- [迭代架构](./iteration-architecture.md)
- [Pipeline 流程](./pipeline.md)
- [工具系统](../development/tools.md)