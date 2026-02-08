# HITL 工作流

## 概述

HITL（Human-In-The-Loop，人在环路）是 Cowork Forge 的核心人机协作机制，允许用户在关键阶段审查 AI Agent 的输出，提供反馈，确保开发方向符合预期。HITL 在 Pipeline 层处理，避免与 Critic 判断冲突，提供更好的用户体验。

### HITL 的价值

1. **质量控制**：人工审查确保输出质量
2. **方向对齐**：确保 AI 理解正确，方向一致
3. **及时纠正**：在早期发现问题，避免后期返工
4. **知识传递**：用户通过反馈传递专业知识
5. **信任建立**：用户参与决策，增强对系统的信任

### HITL 特点

- **外层处理**：在 Pipeline 层触发，不干扰 Agent 内部逻辑
- **异步执行**：用户可以在方便时提供反馈
- **可选配置**：可以启用或禁用 HITL
- **反馈循环**：支持多次反馈和迭代
- **上下文保持**：反馈时保留完整的上下文信息

## HITL 触发时机

### 关键阶段

HITL 在以下阶段触发：

| 阶段 | 触发时机 | 审查内容 |
|------|---------|---------|
| **Idea** | Idea Agent 完成后 | idea.md - 项目创意和方向 |
| **PRD** | PRD Loop 完成后 | prd.md - 产品需求文档 |
| **Design** | Design Loop 完成后 | design.md - 系统设计文档 |
| **Plan** | Plan Loop 完成后 | plan.md - 实施计划 |

### 不触发 HITL 的阶段

以下阶段不触发 HITL，由系统自动处理：

| 阶段 | 原因 |
|------|------|
| **Coding** | 由 Critic 处理质量验证，代码由系统生成 |
| **Check** | 自动执行质量检查，无需人工干预 |
| **Delivery** | 自动生成交付报告，用户可在最后审查 |

## HITL 流程

### 完整流程

```
Agent + Critic 完成
    ↓
Pipeline 验证 Artifacts
    ↓
    ├─→ 验证失败 → 自动重试
    └─→ 验证成功 → 继续
    ↓
判断是否为关键阶段
    ↓
    ├─→ 不是 → 继续下一阶段
    └─→ 是 → 触发 HITL
    ↓
加载 Artifacts
    ↓
显示给用户
    ↓
请求用户反馈
    ↓
    ├─→ 用户跳过 → 继续下一阶段
    ├─→ 用户通过 → 继续下一阶段
    └─→ 用户反馈 → 处理反馈
        ↓
    保存反馈到迭代
        ↓
    重新执行当前阶段（带反馈）
        ↓
    再次验证 Artifacts
        ↓
    继续
```

### HITL 实现

```rust
impl Pipeline {
    /// 触发 HITL
    async fn trigger_hitl(
        &self,
        context: &ExecutionContext,
        stage: &Stage,
    ) -> Result<Option<String>, CoworkError> {
        // 1. 检查是否启用 HITL
        if !self.hitl_config.enabled {
            return Ok(None);
        }

        // 2. 检查阶段是否需要 HITL
        if !self.hitl_config.required_stages.contains(stage) {
            return Ok(None);
        }

        // 3. 加载 Artifacts
        let artifacts = self.load_artifacts(context, stage).await?;

        // 4. 显示给用户
        self.interaction.show_artifacts(&artifacts).await;

        // 5. 请求用户反馈
        let feedback = self.interaction.request_feedback(
            &format!("Please review the {} stage output. Options: 'approve', 'skip', or provide your feedback.", stage)
        ).await?;

        // 6. 处理反馈
        match feedback {
            None => {
                // 用户跳过
                Ok(None)
            }
            Some(fb) if fb.to_lowercase() == "approve" => {
                // 用户批准
                Ok(None)
            }
            Some(fb) if fb.to_lowercase() == "skip" => {
                // 用户跳过
                Ok(None)
            }
            Some(fb) => {
                // 用户提供了反馈
                self.save_feedback(context, &fb).await?;
                Ok(Some(fb))
            }
        }
    }
}
```

### Artifacts 加载

```rust
impl Pipeline {
    /// 加载 Artifacts
    async fn load_artifacts(
        &self,
        context: &ExecutionContext,
        stage: &Stage,
    ) -> Result<StageArtifacts, CoworkError> {
        let iteration_dir = context.workspace.parent().unwrap();
        let artifacts_dir = iteration_dir.join("artifacts");

        let artifacts = match stage {
            Stage::Idea => {
                let content = fs::read_to_string(artifacts_dir.join("idea.md"))?;
                StageArtifacts::Idea(content)
            }
            Stage::Prd => {
                let content = fs::read_to_string(artifacts_dir.join("prd.md"))?;
                StageArtifacts::Prd(content)
            }
            Stage::Design => {
                let content = fs::read_to_string(artifacts_dir.join("design.md"))?;
                StageArtifacts::Design(content)
            }
            Stage::Plan => {
                let content = fs::read_to_string(artifacts_dir.join("plan.md"))?;
                StageArtifacts::Plan(content)
            }
            _ => return Err(CoworkError::InvalidStage(format!("Stage {} does not support HITL", stage))),
        };

        Ok(artifacts)
    }
}

#[derive(Debug, Clone)]
pub enum StageArtifacts {
    Idea(String),
    Prd(String),
    Design(String),
    Plan(String),
}
```

## 反馈处理

### 反馈类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackType {
    Approve,    // 批准
    Skip,       // 跳过
    Correction, // 修正
    Enhancement, // 增强
    Question,   // 问题
}
```

### 反馈存储

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub id: String,                  // 反馈 ID
    pub iteration_id: String,        // 迭代 ID
    pub stage: Stage,                // 阶段
    pub feedback_type: FeedbackType, // 反馈类型
    pub content: String,             // 反馈内容
    pub timestamp: DateTime<Utc>,    // 时间戳
    pub resolved: bool,              // 是否已解决
}

impl Pipeline {
    /// 保存反馈
    async fn save_feedback(
        &self,
        context: &ExecutionContext,
        content: &str,
    ) -> Result<(), CoworkError> {
        let feedback = Feedback {
            id: format!("fb-{}", uuid::Uuid::new_v4()),
            iteration_id: context.iteration_id.clone(),
            stage: context.current_stage,
            feedback_type: FeedbackType::Correction,
            content: content.to_string(),
            timestamp: Utc::now(),
            resolved: false,
        };

        let iteration_store = IterationStore::new();
        iteration_store.save_feedback(&feedback)?;

        Ok(())
    }
}
```

### 反馈处理流程

```rust
impl Pipeline {
    /// 处理反馈
    async fn handle_feedback(
        &self,
        context: &ExecutionContext,
        stage: Stage,
        feedback: String,
    ) -> Result<StageResult, CoworkError> {
        // 1. 记录反馈
        self.interaction.show_message(
            MessageLevel::Info,
            format!("Processing feedback: {}", feedback)
        ).await;

        // 2. 重新执行阶段（带反馈）
        let result = self.stage_executor
            .execute_with_feedback(context, stage, feedback)
            .await?;

        // 3. 验证新的 Artifacts
        if !self.check_artifact_exists(&stage.to_string(), &context.workspace).await? {
            return Err(CoworkError::ArtifactsNotFound(
                "Artifacts still not generated after feedback".to_string()
            ));
        }

        // 4. 标记反馈为已解决
        self.mark_feedback_resolved(context, &stage).await?;

        Ok(result)
    }

    /// 标记反馈为已解决
    async fn mark_feedback_resolved(
        &self,
        context: &ExecutionContext,
        stage: &Stage,
    ) -> Result<(), CoworkError> {
        let iteration_store = IterationStore::new();
        iteration_store.mark_feedback_resolved(&context.iteration_id, stage)?;

        Ok(())
    }
}
```

## HITL 配置

### 配置结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlConfig {
    pub enabled: bool,              // 是否启用 HITL
    pub timeout_seconds: u64,       // 反馈超时时间
    pub max_feedback_length: usize, // 最大反馈长度
    pub required_stages: Vec<Stage>, // 需要 HITL 的阶段
    pub auto_approve: bool,         // 是否自动批准
    pub allow_skip: bool,           // 是否允许跳过
}

impl Default for HitlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 300,    // 5 分钟
            max_feedback_length: 10000,
            required_stages: vec![
                Stage::Idea,
                Stage::Prd,
                Stage::Design,
                Stage::Plan,
            ],
            auto_approve: false,
            allow_skip: true,
        }
    }
}
```

### 配置文件

```toml
[hitl]
enabled = true
timeout_seconds = 300
max_feedback_length = 10000
auto_approve = false
allow_skip = true

required_stages = [
    "Idea",
    "Prd",
    "Design",
    "Plan"
]
```

## 交互接口

### InteractiveBackend Trait

```rust
#[async_trait]
pub trait InteractiveBackend: Send + Sync {
    /// 显示消息
    async fn show_message(&self, level: MessageLevel, message: &str);

    /// 显示 Artifacts
    async fn show_artifacts(&self, artifacts: &StageArtifacts);

    /// 请求反馈
    async fn request_feedback(&self, prompt: &str) -> Result<Option<String>, CoworkError>;

    /// 显示进度
    async fn show_progress(&self, current: usize, total: usize, message: &str);
}

#[derive(Debug, Clone)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
}
```

### CLI 实现

```rust
pub struct CliBackend;

#[async_trait]
impl InteractiveBackend for CliBackend {
    async fn show_message(&self, level: MessageLevel, message: &str) {
        match level {
            MessageLevel::Info => println!("ℹ️  {}", message),
            MessageLevel::Success => println!("✅ {}", message),
            MessageLevel::Warning => println!("⚠️  {}", message),
            MessageLevel::Error => println!("❌ {}", message),
        }
    }

    async fn show_artifacts(&self, artifacts: &StageArtifacts) {
        println!("\n--- Artifacts ---");
        match artifacts {
            StageArtifacts::Idea(content) => println!("{}", content),
            StageArtifacts::Prd(content) => println!("{}", content),
            StageArtifacts::Design(content) => println!("{}", content),
            StageArtifacts::Plan(content) => println!("{}", content),
        }
        println!("--- End of Artifacts ---\n");
    }

    async fn request_feedback(&self, prompt: &str) -> Result<Option<String>, CoworkError> {
        println!("{}", prompt);
        println!("Enter your feedback (or 'approve'/'skip' to continue):");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() || input.eq_ignore_ascii_case("skip") {
            return Ok(None);
        }

        if input.eq_ignore_ascii_case("approve") {
            return Ok(None);
        }

        Ok(Some(input.to_string()))
    }
}
```

### GUI 实现

```rust
pub struct GuiBackend {
    tx: mpsc::Sender<GuiMessage>,
}

#[async_trait]
impl InteractiveBackend for GuiBackend {
    async fn show_message(&self, level: MessageLevel, message: &str) {
        let msg = GuiMessage::ShowMessage {
            level: level.clone(),
            message: message.to_string(),
        };
        self.tx.send(msg).await.unwrap();
    }

    async fn show_artifacts(&self, artifacts: &StageArtifacts) {
        let content = match artifacts {
            StageArtifacts::Idea(c) => c.clone(),
            StageArtifacts::Prd(c) => c.clone(),
            StageArtifacts::Design(c) => c.clone(),
            StageArtifacts::Plan(c) => c.clone(),
        };

        let msg = GuiMessage::ShowArtifacts { content };
        self.tx.send(msg).await.unwrap();
    }

    async fn request_feedback(&self, prompt: &str) -> Result<Option<String>, CoworkError> {
        let (rx, request_id) = {
            let request_id = uuid::Uuid::new_v4().to_string();
            let msg = GuiMessage::RequestFeedback {
                request_id: request_id.clone(),
                prompt: prompt.to_string(),
            };
            self.tx.send(msg).await.unwrap();

            let (tx, rx) = mpsc::channel(1);
            self.feedback_requests.lock().await.insert(request_id.clone(), tx);
            (rx, request_id)
        };

        let feedback = rx.recv().await.ok_or_else(|| {
            CoworkError::FeedbackTimeout("No feedback received".to_string())
        })?;

        self.feedback_requests.lock().await.remove(&request_id);

        if feedback.is_empty() || feedback.eq_ignore_ascii_case("skip") {
            return Ok(None);
        }

        if feedback.eq_ignore_ascii_case("approve") {
            return Ok(None);
        }

        Ok(Some(feedback))
    }
}
```

## HITL 最佳实践

### 1. 提供清晰的上下文

```rust
// ✅ 好的做法：提供清晰的上下文
let prompt = format!(
    "Please review the {} stage output.\n\nContext:\n- Project: {}\n- Iteration: {}\n- Stage: {}\n\nYour feedback:",
    stage, context.project_name, context.iteration_id, stage
);

// ❌ 不好的做法：上下文模糊
let prompt = "Please review the output:";
```

### 2. 设置合理的超时

```rust
// ✅ 好的做法：设置超时
let feedback = tokio::time::timeout(
    Duration::from_secs(self.hitl_config.timeout_seconds),
    self.interaction.request_feedback(prompt)
).await??;

// ❌ 不好的做法：无限等待
let feedback = self.interaction.request_feedback(prompt).await?;
```

### 3. 限制反馈长度

```rust
// ✅ 好的做法：限制反馈长度
if feedback.len() > self.hitl_config.max_feedback_length {
    return Err(CoworkError::InvalidFeedback(
        format!("Feedback too long (max {} characters)", self.hitl_config.max_feedback_length)
    ));
}

// ❌ 不好的做法：不限制长度
let feedback = feedback.trim();
```

### 4. 提供批准和跳过选项

```rust
// ✅ 好的做法：提供批准和跳过选项
let prompt = "Enter your feedback (or 'approve'/'skip' to continue):";

// ❌ 不好的做法：强制要求反馈
let prompt = "Enter your feedback (required):";
```

## 相关文档

- [架构概览](../architecture/overview.md)
- [Pipeline 流程](../architecture/pipeline.md)
- [Agent 系统](../architecture/agent-system.md)
- [迭代架构](../architecture/iteration-architecture.md)