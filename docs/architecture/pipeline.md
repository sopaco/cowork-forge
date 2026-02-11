# Pipeline 流程

## 概述

Pipeline 是 Cowork Forge 的核心执行引擎，负责协调整个开发流程的执行。它管理迭代的完整生命周期，编排各阶段的执行顺序，处理错误和重试，集成人机协作机制（HITL），并确保开发流程的顺利进行。

### Pipeline 的职责

1. **生命周期管理**：管理迭代的完整生命周期（启动、暂停、继续、完成）
2. **阶段编排**：按预定顺序执行各个阶段
3. **错误处理**：捕获和处理执行过程中的错误
4. **重试机制**：自动重试失败的操作
5. **反馈循环**：处理 Agent 和用户的反馈
6. **Artifacts 验证**：验证各阶段生成的制品
7. **HITL 集成**：集成人机协作机制

### Pipeline 组件

```
┌─────────────────────────────────────────────────────────┐
│                  IterationExecutor                      │
│              (迭代执行器 - 顶层协调器)                   │
└────────────────────┬────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────┐
│                    Pipeline                              │
│           (工作流编排 - 阶段序列管理)                    │
└────────────────────┬────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────┐
│                StageExecutor                            │
│           (阶段执行器 - 单阶段执行逻辑)                  │
└────────────────────┬────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────┐
│                Agent Runtime                            │
│           (Agent 运行时 - 基于 adk-rust)                 │
└─────────────────────────────────────────────────────────┘
```

## 执行流程

### 完整执行流程

```
1. 加载迭代和项目数据
   │
   ↓
2. 准备工作空间
   │
   ├─→ Genesis: 创建新目录
   └─→ Evolution: 根据继承模式复制内容
   │
   ↓
3. 创建执行上下文
   │
   ↓
4. 确定起始阶段
   │
   ├─→ Genesis: 从 Idea 开始
   └─→ Evolution: 从指定阶段开始
   │
   ↓
5. 按顺序执行阶段序列
   │
   ├─→ Idea
   ├─→ PRD
   ├─→ Design
   ├─→ Plan
   ├─→ Coding
   ├─→ Check
   └─→ Delivery
   │
   ↓
6. 每个阶段完成后更新迭代状态
   │
   ↓
7. 验证 Artifacts 是否生成
   │
   ├─→ 存在 → 继续
   └─→ 不存在 → 自动重试（最多 3 次）
   │
   ↓
8. 如果是关键阶段，触发 HITL 用户审查
   │
   ├─→ Idea
   ├─→ PRD
   ├─→ Design
   └─→ Plan
   │
   ↓
9. 处理用户反馈
   │
   ├─→ 通过 → 继续下一阶段
   └─→ 反馈 → goto_stage 当前阶段（带反馈）
   │
   ↓
10. 生成并保存制品
   │
   ↓
11. 最终完成迭代
```

### 阶段序列

| 序号 | 阶段 | Agent 类型 | 说明 |
|------|------|-----------|------|
| 1 | Idea | 单一 Agent | 理解创意，生成 idea.md |
| 2 | PRD | LoopAgent | 生成需求文档，创建结构化需求 |
| 3 | Design | LoopAgent | 设计系统架构，创建设计规范 |
| 4 | Plan | LoopAgent | 制定实施计划，分解任务 |
| 5 | Coding | LoopAgent | 生成代码，实现功能 |
| 6 | Check | 单一 Agent | 质量检查，验证功能完整性 |
| 7 | Delivery | 单一 Agent | 生成交付报告，部署代码 |

## IterationExecutor

### 职责

IterationExecutor 是 Pipeline 的顶层协调器，负责：

1. **迭代生命周期管理**：启动、暂停、继续、完成迭代
2. **工作空间准备**：根据继承模式准备工作空间
3. **上下文创建**：创建执行上下文
4. **阶段调度**：调度和执行各个阶段
5. **状态更新**：更新迭代状态
6. **错误恢复**：处理错误和恢复

### 核心方法

```rust
impl IterationExecutor {
    /// 启动迭代
    pub async fn start_iteration(
        &self,
        iteration_id: &str,
        start_stage: Option<Stage>,
    ) -> Result<(), CoworkError> {
        // 1. 加载迭代
        let iteration = self.iteration_store.load(iteration_id)?;

        // 2. 准备工作空间
        self.prepare_workspace(&iteration)?;

        // 3. 创建执行上下文
        let context = self.create_context(&iteration)?;

        // 4. 确定起始阶段
        let start_stage = start_stage.unwrap_or(Stage::Idea);

        // 5. 执行 Pipeline
        self.pipeline.execute(&context, start_stage).await?;

        // 6. 标记迭代为完成
        self.mark_iteration_completed(iteration_id)?;

        Ok(())
    }

    /// 暂停迭代
    pub async fn pause_iteration(&self, iteration_id: &str) -> Result<(), CoworkError> {
        self.iteration_store.update_status(iteration_id, IterationStatus::Paused)?;
        Ok(())
    }

    /// 继续迭代
    pub async fn resume_iteration(&self, iteration_id: &str) -> Result<(), CoworkError> {
        let iteration = self.iteration_store.load(iteration_id)?;
        let context = self.create_context(&iteration)?;
        self.pipeline.execute(&context, iteration.current_stage).await?;
        Ok(())
    }
}
```

## Pipeline

### 职责

Pipeline 是工作流编排器，负责：

1. **阶段序列管理**：管理阶段的执行顺序
2. **阶段执行**：调用 StageExecutor 执行各个阶段
3. **反馈处理**：处理 Agent 和用户的反馈
4. **Artifacts 验证**：验证各阶段生成的制品
5. **HITL 集成**：集成人机协作机制
6. **错误处理**：处理阶段执行错误

### 核心方法

```rust
impl Pipeline {
    /// 执行 Pipeline
    pub async fn execute(
        &self,
        context: &ExecutionContext,
        start_stage: Stage,
    ) -> Result<(), CoworkError> {
        // 获取阶段序列
        let stages = self.get_stage_sequence(start_stage);

        // 依次执行各个阶段
        for stage in stages {
            // 执行阶段
            let result = self.stage_executor.execute(context, stage).await?;

            // 验证 Artifacts
            if !self.verify_artifacts(context, &stage).await? {
                return Err(CoworkError::ArtifactsNotFound(stage));
            }

            // 处理反馈
            if let Some(feedback) = self.check_feedback(context, &stage).await? {
                // 用户提供了反馈，重新执行当前阶段
                self.stage_executor.execute_with_feedback(context, stage, feedback).await?;
            }

            // 触发 HITL（如果是关键阶段）
            if self.is_critical_stage(&stage) {
                self.trigger_hitl(context, &stage).await?;
            }
        }

        Ok(())
    }

    /// 获取阶段序列
    fn get_stage_sequence(&self, start_stage: Stage) -> Vec<Stage> {
        match start_stage {
            Stage::Idea => vec![
                Stage::Idea, Stage::Prd, Stage::Design,
                Stage::Plan, Stage::Coding, Stage::Check, Stage::Delivery,
            ],
            Stage::Prd => vec![
                Stage::Prd, Stage::Design, Stage::Plan,
                Stage::Coding, Stage::Check, Stage::Delivery,
            ],
            // ... 其他阶段
        }
    }

    /// 判断是否为关键阶段
    fn is_critical_stage(&self, stage: &Stage) -> bool {
        matches!(stage, Stage::Idea | Stage::Prd | Stage::Design | Stage::Plan)
    }
}
```

## StageExecutor

### 职责

StageExecutor 是单阶段执行器，负责：

1. **Agent 执行**：调用相应的 Agent 执行任务
2. **结果收集**：收集 Agent 的执行结果
3. **反馈处理**：处理反馈和重试
4. **错误处理**：处理执行错误

### 核心方法

```rust
impl StageExecutor {
    /// 执行阶段
    pub async fn execute(
        &self,
        context: &ExecutionContext,
        stage: Stage,
    ) -> Result<StageResult, CoworkError> {
        // 1. 获取 Agent
        let agent = self.get_agent(context, stage)?;

        // 2. 执行 Agent
        let result = agent.run(context).await?;

        // 3. 处理结果
        match result {
            AgentResult::Success(output) => {
                // 阶段成功
                Ok(StageResult::Success(Some(output)))
            }
            AgentResult::NeedsFeedback(feedback_request) => {
                // 需要反馈
                let feedback = self.request_feedback(context, feedback_request).await?;
                self.execute_with_feedback(context, stage, feedback).await
            }
            AgentResult::Error(error) => {
                // 执行错误
                Err(CoworkError::AgentExecution(error))
            }
        }
    }

    /// 带反馈执行
    pub async fn execute_with_feedback(
        &self,
        context: &ExecutionContext,
        stage: Stage,
        feedback: String,
    ) -> Result<StageResult, CoworkError> {
        // 1. 保存反馈到迭代
        self.save_feedback(context, &feedback)?;

        // 2. 获取 Agent
        let agent = self.get_agent(context, stage)?;

        // 3. 带反馈执行 Agent
        let result = agent.run_with_feedback(context, &feedback).await?;

        // 4. 处理结果
        match result {
            AgentResult::Success(output) => {
                Ok(StageResult::Success(Some(output)))
            }
            AgentResult::Error(error) => {
                Err(CoworkError::AgentExecution(error))
            }
            _ => Err(CoworkError::InvalidFeedback),
        }
    }
}
```

## 错误处理

### 错误类型

```rust
pub enum CoworkError {
    // 迭代错误
    IterationNotFound(String),
    InvalidIterationState(String),

    // Agent 错误
    AgentExecution(String),
    AgentTimeout,

    // 制品错误
    ArtifactsNotFound(Stage),
    InvalidArtifactFormat,

    // 反馈错误
    InvalidFeedback,
    FeedbackTimeout,

    // IO 错误
    IoError(std::io::Error),

    // 其他错误
    LlmError(String),
    ValidationError(String),
}
```

### 错误处理策略

| 错误类型 | 处理策略 | 说明 |
|---------|---------|------|
| **AgentExecution** | 重试 3 次 | Agent 执行失败，自动重试 |
| **AgentTimeout** | 增加超时时间，重试 1 次 | Agent 超时，增加超时时间后重试 |
| **ArtifactsNotFound** | 重试 3 次 | 制品未生成，自动重试 |
| **InvalidArtifactFormat** | 标记阶段失败 | 制品格式错误，标记阶段失败 |
| **InvalidFeedback** | 请求新的反馈 | 反馈无效，请求新的反馈 |
| **FeedbackTimeout** | 使用默认行为 | 反馈超时，使用默认行为 |
| **IoError** | 标记迭代失败 | IO 错误，标记迭代失败 |
| **LlmError** | 重试 3 次 | LLM 错误，自动重试 |

## 重试机制

### 重试策略

```rust
impl Pipeline {
    /// 带重试执行阶段
    async fn execute_with_retry(
        &self,
        context: &ExecutionContext,
        stage: Stage,
    ) -> Result<StageResult, CoworkError> {
        let max_retries = 3;
        let mut retry_count = 0;

        loop {
            match self.stage_executor.execute(context, stage).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    retry_count += 1;

                    if retry_count >= max_retries {
                        return Err(error);
                    }

                    // 等待后重试
                    tokio::time::sleep(Duration::from_secs(2 * retry_count as u64)).await;

                    // 记录重试
                    self.log_retry(context, &stage, retry_count, &error).await;
                }
            }
        }
    }
}
```

### 重试配置

| 操作 | 最大重试次数 | 重试间隔 | 说明 |
|------|-----------|---------|------|
| **Agent 执行** | 3 次 | 2s, 4s, 6s | 指数退避 |
| **Artifacts 验证** | 3 次 | 2s, 4s, 6s | 指数退避 |
| **LLM 调用** | 3 次 | 1s, 2s, 3s | 指数退避 |
| **文件操作** | 1 次 | - | 不重试 |
| **反馈请求** | 1 次 | - | 不重试 |

## 反馈循环

### 反馈来源

1. **Agent 反馈**：Agent 通过工具提供反馈
2. **Critic 反馈**：Critic 提供质量反馈
3. **HITL 反馈**：用户通过界面提供反馈
4. **系统反馈**：系统自动生成的反馈

### 反馈处理流程

```
Agent 执行
    ↓
生成结果
    ↓
Pipeline 验证 Artifacts
    ↓
    ├─→ 验证通过 → 继续
    └─→ 验证失败 → 自动重试
    ↓
触发 HITL（关键阶段）
    ↓
用户审查
    ↓
    ├─→ 通过 → 继续下一阶段
    └─→ 反馈 → goto_stage 当前阶段
    ↓
保存反馈
    ↓
重新执行阶段（带反馈）
    ↓
完成
```

### 反馈存储

```rust
pub struct Feedback {
    pub id: String,
    pub iteration_id: String,
    pub stage: Stage,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub source: FeedbackSource,
}

pub enum FeedbackSource {
    Agent,
    Critic,
    User,
    System,
}
```

## HITL 集成

### HITL 触发时机

HITL 在以下阶段触发：

1. **Idea** 阶段完成后
2. **PRD** 阶段完成后
3. **Design** 阶段完成后
4. **Plan** 阶段完成后

**不触发 HITL 的阶段**：
- Coding：由 Critic 处理质量验证
- Check：自动执行，无需人工干预
- Delivery：自动执行，无需人工干预

### HITL 流程

```rust
impl Pipeline {
    /// 触发 HITL
    async fn trigger_hitl(
        &self,
        context: &ExecutionContext,
        stage: &Stage,
    ) -> Result<Option<String>, CoworkError> {
        // 1. 加载 Artifacts
        let artifacts = self.load_artifacts(context, stage)?;

        // 2. 显示给用户
        self.interaction.show_artifacts(&artifacts).await;

        // 3. 请求用户反馈
        let feedback = self.interaction.request_feedback(
            &format!("Please review the {} stage output", stage)
        ).await?;

        // 4. 保存反馈
        if let Some(fb) = &feedback {
            self.save_feedback(context, fb).await?;
        }

        Ok(feedback)
    }
}
```

### HITL 配置

```rust
pub struct HitlConfig {
    pub enabled: bool,              // 是否启用 HITL
    pub timeout_seconds: u64,       // 反馈超时时间
    pub max_feedback_length: usize, // 最大反馈长度
    pub required_stages: Vec<Stage>, // 需要 HITL 的阶段
}

impl Default for HitlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 3000,    // 50 分钟
            max_feedback_length: 10000,
            required_stages: vec![
                Stage::Idea,
                Stage::Prd,
                Stage::Design,
                Stage::Plan,
            ],
        }
    }
}
```

## 自愈机制

### Check 阶段的自愈

Check Agent 发现问题时，可以通过 `goto_stage()` 返回 Coding 阶段修复：

```rust
impl CheckAgent {
    async fn execute(&self, context: &ExecutionContext) -> Result<AgentResult, CoworkError> {
        // 1. 检查功能覆盖度
        let coverage = self.check_feature_coverage(context).await?;
        if coverage < 0.8 {
            // 覆盖度不足，返回 Coding 阶段
            self.goto_stage(context, Stage::Coding, "Insufficient feature coverage").await?;
        }

        // 2. 检查测试通过率
        let test_result = self.check_tests(context).await?;
        if !test_result.passed {
            // 测试失败，返回 Coding 阶段
            self.goto_stage(context, Stage::Coding, "Tests failed").await?;
        }

        // 3. 其他检查...

        Ok(AgentResult::Success("All checks passed".to_string()))
    }
}
```

## 性能优化

### 并行执行

某些阶段可以并行执行：

```rust
impl Pipeline {
    /// 并行执行多个阶段
    async fn execute_parallel(
        &self,
        context: &ExecutionContext,
        stages: Vec<Stage>,
    ) -> Result<Vec<StageResult>, CoworkError> {
        let futures: Vec<_> = stages
            .into_iter()
            .map(|stage| self.stage_executor.execute(context, stage))
            .collect();

        let results = futures::future::try_join_all(futures).await?;
        Ok(results)
    }
}
```

### 缓存机制

缓存阶段结果，避免重复执行：

```rust
impl StageExecutor {
    /// 带缓存执行
    async fn execute_with_cache(
        &self,
        context: &ExecutionContext,
        stage: Stage,
    ) -> Result<StageResult, CoworkError> {
        // 1. 检查缓存
        if let Some(cached) = self.cache.get(&context.iteration_id, &stage) {
            return Ok(cached);
        }

        // 2. 执行阶段
        let result = self.execute(context, stage).await?;

        // 3. 缓存结果
        self.cache.set(&context.iteration_id, &stage, result.clone());

        Ok(result)
    }
}
```

## 监控和日志

### 执行日志

```rust
impl Pipeline {
    /// 记录阶段开始
    async fn log_stage_start(&self, context: &ExecutionContext, stage: &Stage) {
        log::info!(
            "Starting stage: {} for iteration: {}",
            stage,
            context.iteration_id
        );
    }

    /// 记录阶段完成
    async fn log_stage_complete(&self, context: &ExecutionContext, stage: &Stage, duration: Duration) {
        log::info!(
            "Completed stage: {} for iteration: {} in {:.2}s",
            stage,
            context.iteration_id,
            duration.as_secs_f64()
        );
    }

    /// 记录错误
    async fn log_error(&self, context: &ExecutionContext, stage: &Stage, error: &CoworkError) {
        log::error!(
            "Error in stage: {} for iteration: {}: {:?}",
            stage,
            context.iteration_id,
            error
        );
    }
}
```

## 相关文档

- [架构概览](./overview.md)
- [Agent 系统](./agent-system.md)
- [迭代架构](./iteration-architecture.md)
- [文件安全机制](./file-security.md)
- [Artifacts 验证](./artifacts-validation.md)
- [HITL 工作流](../features/hitl-workflow.md)
