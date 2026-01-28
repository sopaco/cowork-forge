use anyhow::Result;
use std::sync::Arc;

use crate::artifacts::Stage;
use crate::memory::ArtifactStore;
use crate::hitl::HitlController;
use crate::orchestrator::{SessionMeta, StageStatus};
use super::{StageAgent, StageAgentContext, StageAgentResult};

/// 统一的阶段执行器
/// 负责执行阶段的通用流程：检查 → 执行 → HITL → 保存 → 标记完成
pub struct StageExecutor {
    store: Arc<ArtifactStore>,
    hitl: Arc<HitlController>,
}

impl StageExecutor {
    pub fn new(store: Arc<ArtifactStore>, hitl: Arc<HitlController>) -> Self {
        Self { store, hitl }
    }
    
    /// 执行单个阶段
    /// 
    /// # 参数
    /// - `agent`: 实现了 StageAgent trait 的 Agent
    /// - `session_id`: 会话 ID
    /// - `meta`: 可变的会话元信息，用于更新阶段状态
    /// - `skip_if_completed`: 如果为 true，已完成的阶段会被跳过
    /// 
    /// # 返回
    /// - `Ok(StageExecutionResult)`: 执行结果，包含是否跳过、artifact_id 等
    pub async fn execute_stage<A: StageAgent>(
        &self,
        agent: &A,
        session_id: &str,
        meta: &mut SessionMeta,
        skip_if_completed: bool,
    ) -> Result<StageExecutionResult> {
        let stage = agent.stage();
        
        // 1. 检查是否已完成
        if skip_if_completed && self.is_stage_completed(meta, stage) {
            let artifact_id = self.get_completed_artifact_id(meta, stage)?;
            tracing::info!("Stage {:?} already completed, skipping", stage);
            return Ok(StageExecutionResult::skipped(stage, artifact_id));
        }
        
        // 2. 打印阶段开始信息
        self.print_stage_header(stage);
        
        // 3. 标记为进行中
        self.mark_stage_in_progress(meta, stage)?;
        
        // 4. 创建上下文
        let context = StageAgentContext::new(
            session_id.to_string(),
            self.store.clone(),
            self.hitl.clone(),
        );
        
        // 5. 执行 Agent
        tracing::info!("Executing agent for stage {:?}", stage);
        let result = match agent.execute(&context).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Agent execution failed for stage {:?}: {}", stage, e);
                self.mark_stage_failed(meta, stage, e.to_string(), true)?;
                return Err(e);
            }
        };
        
        // 6. 打印执行结果摘要
        self.print_stage_summary(stage, &result);
        
        // 7. HITL 审查（如果需要）
        let should_continue = if agent.requires_hitl_review() {
            self.hitl.confirm(&format!("继续到下一阶段？"))?
        } else {
            true
        };
        
        if !should_continue {
            tracing::info!("User cancelled at stage {:?}", stage);
            return Err(anyhow::anyhow!("User cancelled workflow at stage {:?}", stage));
        }
        
        // 8. 标记为完成
        self.mark_stage_completed(meta, stage, result.artifact_id.clone(), result.verified)?;
        
        Ok(StageExecutionResult {
            stage,
            artifact_id: result.artifact_id,
            verified: result.verified,
            skipped: false,
            summary: result.summary,
        })
    }
    
    /// 检查阶段是否已完成
    fn is_stage_completed(&self, meta: &SessionMeta, stage: Stage) -> bool {
        matches!(
            meta.stage_status.get(&stage),
            Some(StageStatus::Completed { .. })
        )
    }
    
    /// 获取已完成阶段的 artifact_id
    fn get_completed_artifact_id(&self, meta: &SessionMeta, stage: Stage) -> Result<String> {
        match meta.stage_status.get(&stage) {
            Some(StageStatus::Completed { artifact_id, .. }) => Ok(artifact_id.clone()),
            _ => Err(anyhow::anyhow!("Stage {:?} not completed", stage)),
        }
    }
    
    /// 标记阶段为进行中
    fn mark_stage_in_progress(&self, meta: &mut SessionMeta, stage: Stage) -> Result<()> {
        meta.stage_status.insert(
            stage,
            StageStatus::InProgress {
                started_at: chrono::Utc::now(),
            }
        );
        meta.current_stage = Some(stage);
        self.save_session_meta(meta)?;
        Ok(())
    }
    
    /// 标记阶段为完成
    fn mark_stage_completed(
        &self,
        meta: &mut SessionMeta,
        stage: Stage,
        artifact_id: String,
        verified: bool
    ) -> Result<()> {
        meta.stage_status.insert(
            stage,
            StageStatus::Completed {
                artifact_id,
                completed_at: chrono::Utc::now(),
                verified,
            }
        );
        self.save_session_meta(meta)?;
        Ok(())
    }
    
    /// 标记阶段为失败
    fn mark_stage_failed(
        &self,
        meta: &mut SessionMeta,
        stage: Stage,
        error: String,
        can_retry: bool
    ) -> Result<()> {
        meta.stage_status.insert(
            stage,
            StageStatus::Failed {
                error,
                failed_at: chrono::Utc::now(),
                can_retry,
            }
        );
        self.save_session_meta(meta)?;
        Ok(())
    }
    
    /// 保存 session meta
    fn save_session_meta(&self, meta: &SessionMeta) -> Result<()> {
        use std::fs;
        use std::path::PathBuf;

        let session_dir = PathBuf::from(".cowork").join(&meta.session_id);
        fs::create_dir_all(&session_dir)?;

        let meta_path = session_dir.join("meta.json");
        let content = serde_json::to_string_pretty(meta)?;
        fs::write(&meta_path, content)?;

        Ok(())
    }
    
    /// 打印阶段标题
    fn print_stage_header(&self, stage: Stage) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║   Stage: {:28} ║", format!("{:?}", stage));
        println!("╚═══════════════════════════════════════╝\n");
    }
    
    /// 打印阶段摘要
    fn print_stage_summary(&self, stage: Stage, result: &StageAgentResult) {
        println!("\n✅ Stage {:?} completed!", stage);
        if !result.summary.is_empty() {
            println!("Summary:");
            for line in &result.summary {
                println!("  {}", line);
            }
        }
        println!();
    }
}

/// 阶段执行结果
pub struct StageExecutionResult {
    pub stage: Stage,
    pub artifact_id: String,
    pub verified: bool,
    pub skipped: bool,
    pub summary: Vec<String>,
}

impl StageExecutionResult {
    pub fn skipped(stage: Stage, artifact_id: String) -> Self {
        Self {
            stage,
            artifact_id,
            verified: true,
            skipped: true,
            summary: Vec::new(),
        }
    }
}
