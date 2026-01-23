# 重构架构设计文档

## 概述

本次重构将 **Cowork-rs** 从单体架构重构为基于 **StageAgent** 接口的模块化架构。

## 重构目标

1. **解耦阶段逻辑**：每个阶段的逻辑独立封装，不再耦合在 Orchestrator 中
2. **统一执行流程**：所有阶段遵循相同的执行模式（检查 → 执行 → HITL → 保存）
3. **提高可维护性**：新增阶段只需实现 `StageAgent` trait
4. **保持向后兼容**：现有 Agent 可以通过包装器适配新接口

## 核心组件

### 1. StageAgent Trait

```rust
#[async_trait]
pub trait StageAgent: Send + Sync {
    fn stage(&self) -> Stage;
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult>;
    fn dependencies(&self) -> Vec<Stage> { Vec::new() }
    fn requires_hitl_review(&self) -> bool { true }
    fn description(&self) -> &str { "No description" }
}
```

**职责**：
- 定义阶段的核心逻辑
- 声明依赖关系
- 控制是否需要 HITL 审查

### 2. StageAgentContext

```rust
pub struct StageAgentContext {
    pub session_id: String,
    pub store: Arc<ArtifactStore>,
    pub hitl: Arc<HitlController>,
    pub user_input: Option<String>,
}
```

**职责**：
- 提供 Agent 执行所需的所有上下文
- 提供辅助方法（如 `load_artifact`）

### 3. StageExecutor

```rust
pub struct StageExecutor {
    store: Arc<ArtifactStore>,
    hitl: Arc<HitlController>,
}

impl StageExecutor {
    pub async fn execute_stage<A: StageAgent>(
        &self,
        agent: &A,
        session_id: &str,
        meta: &mut SessionMeta,
        skip_if_completed: bool,
    ) -> Result<StageExecutionResult>
}
```

**职责**：
- 执行统一的阶段流程：
  1. 检查是否已完成
  2. 打印阶段信息
  3. 标记进行中
  4. 创建上下文并调用 Agent
  5. HITL 审查
  6. 标记完成/失败

### 4. Orchestrator（重构后）

```rust
impl Orchestrator {
    pub async fn run_workflow_from_stage(...) -> Result<()> {
        let executor = StageExecutor::new(self.store.clone(), hitl);
        
        // Stage 1: IDEA Intake
        let idea_agent = IdeaIntakeStageAgent::new(...)?;
        executor.execute_stage(&idea_agent, session_id, &mut meta, true).await?;
        
        // Stage 2: PRD
        let prd_agent = PrdStageAgent::new(...)?;
        executor.execute_stage(&prd_agent, session_id, &mut meta, true).await?;
        
        // ... 其他阶段
    }
}
```

**变化**：
- 从 800+ 行减少到 ~100 行
- 每个阶段只需 2-3 行代码
- 所有公共逻辑委托给 StageExecutor

## 迁移策略

### 阶段 1：创建基础设施（已完成）
- ✅ 创建 `StageAgent` trait
- ✅ 创建 `StageAgentContext`
- ✅ 创建 `StageExecutor`

### 阶段 2：包装现有 Agent
对于每个现有 Agent（如 `IdeaIntakeAgent`）：

```rust
// 1. 保留原有实现（向后兼容）
pub struct IdeaIntakeAgent { ... }

impl IdeaIntakeAgent {
    pub async fn execute(...) -> Result<Artifact> { ... }  // 原有方法保留
}

// 2. 创建 StageAgent 包装器
pub struct IdeaIntakeStageAgent {
    inner: IdeaIntakeAgent,
}

#[async_trait]
impl StageAgent for IdeaIntakeStageAgent {
    fn stage(&self) -> Stage { Stage::IdeaIntake }
    
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult> {
        // 调用原有 inner.execute()
        // 处理 HITL
        // 返回 StageAgentResult
    }
}
```

**需要包装的 Agent**：
- [x] IdeaIntakeAgent
- [ ] PrdAgent
- [ ] DesignAgent
- [ ] PlanAgent
- [ ] CodePlanner
- [ ] CheckAgent
- [ ] FeedbackAgent
- [ ] DeliveryAgent

### 阶段 3：重构 Orchestrator
- [ ] 使用 StageExecutor 替代内联逻辑
- [ ] 简化 `run_workflow_from_stage`
- [ ] 保持 Feedback Loop 逻辑（特殊处理）

### 阶段 4：测试和验证
- [ ] 单元测试：每个 StageAgent 的 execute 方法
- [ ] 集成测试：完整工作流
- [ ] 回归测试：确保原有功能正常

## 优势

### 1. 代码量减少
- **Before**: Orchestrator 800+ 行
- **After**: Orchestrator ~100 行 + 每个 Agent 包装器 ~50 行

### 2. 可维护性提升
```rust
// Before: 新增阶段需要修改 Orchestrator
impl Orchestrator {
    pub async fn run_workflow(...) {
        // ... 800 行代码中找位置插入
        let new_stage_artifact = if ... { ... } else { ... };  // 50行
    }
}

// After: 新增阶段只需实现 trait
pub struct NewStageAgent { ... }

#[async_trait]
impl StageAgent for NewStageAgent {
    fn stage(&self) -> Stage { Stage::NewStage }
    async fn execute(&self, ctx: &StageAgentContext) -> Result<...> { ... }
}

// 在 Orchestrator 中调用
executor.execute_stage(&new_stage_agent, session_id, &mut meta, true).await?;
```

### 3. 测试性提升
```rust
#[tokio::test]
async fn test_idea_intake_stage() {
    let agent = IdeaIntakeStageAgent::new(...)?;
    let context = StageAgentContext::new(...);
    
    let result = agent.execute(&context).await?;
    
    assert_eq!(result.stage, Stage::IdeaIntake);
    assert!(result.verified);
}
```

### 4. 并行执行基础
```rust
// 未来可以并行执行独立阶段
let (design_result, plan_result) = tokio::join!(
    executor.execute_stage(&design_agent, ...),
    executor.execute_stage(&plan_agent, ...),  // 如果不依赖 design
);
```

## 兼容性保证

1. **原有 Agent 保留**：`IdeaIntakeAgent::execute()` 等方法仍然存在
2. **公共 API 不变**：`Orchestrator::run_full_workflow()` 签名不变
3. **渐进式迁移**：可以逐步迁移每个 Agent，不需要一次性重构

## 下一步

1. 完成所有 Agent 的包装器实现
2. 重构 Orchestrator 使用 StageExecutor
3. 添加单元测试
4. 更新文档和示例

## 附录：完整示例

参见 `crates/cowork-core/src/agents/idea_intake_stage.rs`
