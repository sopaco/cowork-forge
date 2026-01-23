use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::artifacts::Stage;
use crate::memory::ArtifactStore;
use crate::agents::{
    IdeaIntakeAgent, PrdAgent, DesignAgent, PlanAgent, 
    CheckAgent, FeedbackAgent, DeliveryAgent,
    StageExecutor, CodingStageAgent
};
use crate::hitl::HitlController;
use crate::config::ModelConfig;

#[cfg(test)]
mod tests;

/// Stage 执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StageStatus {
    /// 未开始
    NotStarted,
    
    /// 执行中
    InProgress {
        started_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// 完成（可能有或没有验证）
    Completed {
        artifact_id: String,
        completed_at: chrono::DateTime<chrono::Utc>,
        verified: bool,  // 是否经过验证
    },
    
    /// 失败
    Failed {
        error: String,
        failed_at: chrono::DateTime<chrono::Utc>,
        can_retry: bool,
    },
}

/// Session 元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub current_stage: Option<Stage>,
    
    #[serde(default)]
    pub stage_status: HashMap<Stage, StageStatus>,  // 阶段状态
    
    // Feedback loop 控制
    #[serde(default)]
    pub feedback_iterations: usize,  // 当前 Feedback 迭代次数
    
    #[serde(default = "default_max_feedback_iterations")]
    pub max_feedback_iterations: usize,  // 最大 Feedback 迭代次数（默认 20）
    
    // 修改上下文：保存用户通过 modify 命令提交的修改意图
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modification_context: Option<String>,
}

fn default_max_feedback_iterations() -> usize {
    20
}

/// Orchestrator 负责驱动多阶段流程
pub struct Orchestrator {
    store: Arc<ArtifactStore>,
}

impl Orchestrator {
    pub fn new(store: ArtifactStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// 创建新 session
    pub fn create_session(&self) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let meta = SessionMeta {
            session_id: session_id.clone(),
            created_at: chrono::Utc::now(),
            current_stage: None,
            stage_status: HashMap::new(),
            feedback_iterations: 0,
            max_feedback_iterations: 20,
            modification_context: None,
        };

        self.save_session_meta(&meta)?;

        tracing::info!("Session created: {}", session_id);
        Ok(session_id)
    }

    /// 加载 session meta
    pub fn load_session_meta(&self, session_id: &str) -> Result<SessionMeta> {
        use std::fs;
        use std::path::PathBuf;

        let meta_path = PathBuf::from(".cowork")
            .join(session_id)
            .join("meta.json");

        let content = fs::read_to_string(&meta_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// 保存 session meta
    pub fn save_session_meta(&self, meta: &SessionMeta) -> Result<()> {
        use std::fs;
        use std::path::PathBuf;

        let session_dir = PathBuf::from(".cowork").join(&meta.session_id);
        fs::create_dir_all(&session_dir)?;

        let meta_path = session_dir.join("meta.json");
        let content = serde_json::to_string_pretty(meta)?;
        fs::write(&meta_path, content)?;

        Ok(())
    }

    /// 运行完整的 8 阶段工作流
    pub async fn run_full_workflow(&self, session_id: &str, model_config: &ModelConfig) -> Result<()> {
        self.run_workflow_from_stage(session_id, model_config, None).await
    }
    
    /// 检查阶段是否已完成（包括已验证和未验证）
    fn is_stage_completed(&self, meta: &SessionMeta, stage: Stage) -> bool {
        matches!(
            meta.stage_status.get(&stage),
            Some(StageStatus::Completed { .. })
        )
    }

    /// 从指定阶段开始运行工作流（用于恢复）
    /// 
    /// 使用新的 StageExecutor 架构，大幅简化代码
    pub async fn run_workflow_from_stage(
        &self,
        session_id: &str,
        model_config: &ModelConfig,
        resume_from: Option<Stage>,
    ) -> Result<()> {
        // 使用 Box::pin 包装递归调用
        Box::pin(self.run_workflow_from_stage_impl(session_id, model_config, resume_from)).await
    }

    /// 实际的工作流实现（内部方法）
    async fn run_workflow_from_stage_impl(
        &self,
        session_id: &str,
        model_config: &ModelConfig,
        resume_from: Option<Stage>,
    ) -> Result<()> {
        tracing::info!("Running workflow for session: {}, resume_from: {:?}", session_id, resume_from);

        let hitl = Arc::new(HitlController::new());
        let mut meta = self.load_session_meta(session_id)?;

        // 创建 StageExecutor
        let executor = StageExecutor::new(self.store.clone(), hitl.clone());

        // 确定起始阶段
        let start_stage = resume_from.unwrap_or(Stage::IdeaIntake);
        
        // 如果是恢复模式，显示已完成的阶段
        if resume_from.is_some() {
            self.print_resume_status(&meta, start_stage)?;
        }

        // ========================================
        // Stage 1: IDEA Intake
        // ========================================
        let idea_agent = IdeaIntakeAgent::new(&model_config.llm, self.store.clone())?;
        executor.execute_stage(&idea_agent, session_id, &mut meta, true).await?;

        // ========================================
        // Stage 2: PRD Generation
        // ========================================
        let prd_agent = PrdAgent::new(&model_config.llm, self.store.clone())?;
        executor.execute_stage(&prd_agent, session_id, &mut meta, true).await?;

        // ========================================
        // Stage 3: Design
        // ========================================
        let design_agent = DesignAgent::new(&model_config.llm, self.store.clone())?;
        executor.execute_stage(&design_agent, session_id, &mut meta, true).await?;

        // ========================================
        // Stage 4: Plan
        // ========================================
        let plan_agent = PlanAgent::new(&model_config.llm, self.store.clone())?;
        executor.execute_stage(&plan_agent, session_id, &mut meta, true).await?;

        // ========================================
        // Stage 5: Coding
        // ========================================
        let coding_agent = CodingStageAgent::new(&model_config.llm, self.store.clone())?;
        executor.execute_stage(&coding_agent, session_id, &mut meta, true).await?;

        // ========================================
        // Stage 6: Check
        // ========================================
        let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;
        let _check_result = executor.execute_stage(&check_agent, session_id, &mut meta, true).await?;

        // ========================================
        // Stage 7: Feedback Loop
        // ========================================
        // Feedback 是特殊的循环阶段，需要特殊处理
        loop {
            let feedback_agent = FeedbackAgent::new(&model_config.llm, self.store.clone())?;
            let _feedback_result = executor.execute_stage(&feedback_agent, session_id, &mut meta, false).await?;
            
            // 加载 Feedback artifact 查看是否需要迭代
            let feedback_artifact: crate::artifacts::FeedbackArtifact = 
                self.load_artifact(session_id, Stage::Feedback)?;
            
            // 如果没有需要修改或重跑的内容，结束循环
            if feedback_artifact.data.delta.is_empty() && feedback_artifact.data.rerun.is_empty() {
                println!("✓ 无需修改，Feedback 循环结束");
                break;
            }

            // 检查是否达到最大迭代次数
            if meta.feedback_iterations >= meta.max_feedback_iterations {
                println!("⚠️  已达到最大 Feedback 迭代次数 ({}次)", meta.max_feedback_iterations);
                break;
            }

            // 应用 delta 修改
            if !feedback_artifact.data.delta.is_empty() {
                println!("\n📝 应用 {} 项修改...", feedback_artifact.data.delta.len());
                self.apply_feedback_deltas(session_id, &feedback_artifact.data.delta)?;
            }
            
            // 处理需要重跑的阶段
            if !feedback_artifact.data.rerun.is_empty() {
                println!("\n🔄 需要重跑 {} 个阶段", feedback_artifact.data.rerun.len());
                
                // 找到最早需要重跑的阶段
                let earliest_rerun_stage = self.find_earliest_stage(&feedback_artifact.data.rerun);
                
                println!("从 {:?} 阶段开始重新执行", earliest_rerun_stage);
                
                // 清除该阶段及之后所有阶段的完成状态
                self.clear_stages_from(&mut meta, earliest_rerun_stage)?;
                
                // 增加迭代计数
                meta.feedback_iterations += 1;
                self.save_session_meta(&meta)?;
                
                // 递归重新执行工作流
                return Box::pin(self.run_workflow_from_stage_impl(session_id, model_config, Some(earliest_rerun_stage))).await;
            }
            
            // 没有重跑需求但有 delta，继续下一轮 feedback
            meta.feedback_iterations += 1;
            self.save_session_meta(&meta)?;
            
            println!("\n继续收集反馈（迭代 {}/{}）", meta.feedback_iterations, meta.max_feedback_iterations);
        }

        // ========================================
        // Stage 8: Delivery
        // ========================================
        let delivery_agent = DeliveryAgent::new(&model_config.llm, self.store.clone())?;
        executor.execute_stage(&delivery_agent, session_id, &mut meta, true).await?;

        println!("\n╔═══════════════════════════════════════╗");
        println!("║   🎉 工作流完成！                     ║");
        println!("╚═══════════════════════════════════════╝\n");
        println!("Session ID: {}", session_id);
        println!("Artifacts: .cowork/{}/artifacts/", session_id);

        Ok(())
    }

    /// 从文件系统加载指定阶段的 artifact
    fn load_artifact<T>(&self, session_id: &str, stage: Stage) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        use std::fs;

        let artifacts = self.store.list(session_id)?;
        
        // 找到该阶段的最新 artifact
        let artifact_meta = artifacts
            .iter()
            .filter(|a| a.stage == stage)
            .max_by_key(|a| &a.path_json)
            .ok_or_else(|| anyhow::anyhow!("No artifact found for stage {:?}", stage))?;

        let content = fs::read_to_string(&artifact_meta.path_json)?;
        let artifact: T = serde_json::from_str(&content)?;
        
        tracing::info!("Loaded artifact for stage {:?} from {}", stage, artifact_meta.path_json.display());
        
        Ok(artifact)
    }

    /// 恢复会话（从中断点继续）
    pub async fn resume_session(&self, session_id: &str, model_config: &ModelConfig) -> Result<()> {
        // 检查 session 是否存在
        if !self.store.session_exists(session_id) {
            return Err(anyhow::anyhow!("Session {} not found", session_id));
        }

        // 加载 session meta
        let meta = self.load_session_meta(session_id)?;
        
        // 确定下一个要执行的阶段
        let all_stages = Stage::all();
        let next_stage = all_stages
            .iter()
            .find(|s| !self.is_stage_completed(&meta, **s))
            .cloned();

        if let Some(stage) = next_stage {
            println!("\n📋 恢复会话: {}", session_id);
            println!("下一阶段: {:?}", stage);
            println!();
            
            self.run_workflow_from_stage(session_id, model_config, Some(stage)).await
        } else {
            println!("\n✅ 会话 {} 已全部完成", session_id);
            Ok(())
        }
    }

    /// 修改需求/设计并触发重新执行
    pub async fn modify_and_rerun(
        &self,
        session_id: &str,
        modification: &str,
        model_config: &ModelConfig,
    ) -> Result<()> {
        tracing::info!("modify_and_rerun: session={}, modification={}", session_id, modification);

        // 检查 session 是否存在
        if !self.store.session_exists(session_id) {
            return Err(anyhow::anyhow!("Session {} not found", session_id));
        }

        let mut meta = self.load_session_meta(session_id)?;

        // 检查是否超过最大迭代次数
        if meta.feedback_iterations >= meta.max_feedback_iterations {
            return Err(anyhow::anyhow!(
                "已达到最大 Feedback 迭代次数 ({})，无法继续修改",
                meta.max_feedback_iterations
            ));
        }

        // 保存修改上下文
        meta.modification_context = Some(modification.to_string());
        self.save_session_meta(&meta)?;
        
        println!("\n💾 保存修改上下文: {}", modification);
        println!("🤖 使用 FeedbackAgent 分析修改影响...");

        // 使用 FeedbackAgent 分析修改
        let feedback_agent = FeedbackAgent::new(&model_config.llm, self.store.clone())?;
        
        // 加载 CheckReport
        let check_artifact: crate::artifacts::CheckReportArtifact = 
            self.load_artifact(session_id, Stage::Check)?;
        
        // 调用 FeedbackAgent 分析修改
        let feedback_artifact = feedback_agent.analyze_feedback(
            session_id,
            &check_artifact,
            modification
        ).await?;
        
        println!("\n📋 分析结果:");
        println!("  Delta 修改: {} 项", feedback_artifact.data.delta.len());
        println!("  需要重跑: {} 个阶段", feedback_artifact.data.rerun.len());
        
        // 应用 delta 修改
        if !feedback_artifact.data.delta.is_empty() {
            println!("\n📝 应用修改...");
            self.apply_feedback_deltas(session_id, &feedback_artifact.data.delta)?;
        }
        
        // 找到需要重跑的最早阶段
        if !feedback_artifact.data.rerun.is_empty() {
            let earliest_stage = self.find_earliest_stage(&feedback_artifact.data.rerun);
            
            println!("\n🔄 从 {:?} 阶段开始重新执行", earliest_stage);
            
            // 清除该阶段及之后所有阶段的完成状态
            self.clear_stages_from(&mut meta, earliest_stage)?;
            
            // 增加迭代计数
            meta.feedback_iterations += 1;
            self.save_session_meta(&meta)?;
            
            // 重新执行工作流
            self.run_workflow_from_stage(session_id, model_config, Some(earliest_stage)).await
        } else {
            println!("\n✅ 修改已应用，无需重跑阶段");
            Ok(())
        }
    }

    /// 列出 session 的所有 artifacts
    pub fn list_artifacts(&self, session_id: &str) -> Result<Vec<crate::memory::ArtifactMeta>> {
        self.store.list(session_id)
    }

    /// 打印恢复模式的状态信息
    fn print_resume_status(&self, meta: &SessionMeta, start_stage: Stage) -> Result<()> {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║   🔄 恢复会话: {}  ", &meta.session_id[..8]);
        println!("╚═══════════════════════════════════════╝");
        
        // 验证前置阶段
        for stage in Stage::all() {
            if *stage == start_stage { break; }
            
            match meta.stage_status.get(stage) {
                Some(StageStatus::Completed { verified: true, artifact_id, .. }) => {
                    println!("✅ {} - 已完成并验证 (artifact: {})", stage.as_str(), &artifact_id[..8]);
                }
                Some(StageStatus::Completed { verified: false, artifact_id, .. }) => {
                    println!("⚠️  {} - 已完成但未验证 (artifact: {})", stage.as_str(), &artifact_id[..8]);
                }
                Some(StageStatus::Failed { error, can_retry, .. }) => {
                    println!("❌ {} - 失败: {}", stage.as_str(), error);
                    if *can_retry {
                        println!("   提示：可以重试此阶段");
                    }
                    return Err(anyhow::anyhow!("前置阶段 {} 失败，无法继续", stage.as_str()));
                }
                Some(StageStatus::InProgress { .. }) => {
                    println!("🔄 {} - 未完成（进行中）", stage.as_str());
                    return Err(anyhow::anyhow!("前置阶段 {} 未完成", stage.as_str()));
                }
                Some(StageStatus::NotStarted) | None => {
                    println!("❓ {} - 未开始", stage.as_str());
                    return Err(anyhow::anyhow!("前置阶段 {} 未完成", stage.as_str()));
                }
            }
        }
        
        println!("从阶段继续: {:?}", start_stage);
        println!();
        
        Ok(())
    }

    /// 应用 Feedback delta 修改
    /// 
    /// Delta 格式示例：
    /// - target_stage: Requirements
    ///   change: "添加用户登录功能"
    fn apply_feedback_deltas(&self, session_id: &str, deltas: &[crate::artifacts::Delta]) -> Result<()> {
        for delta in deltas {
            println!("  🔧 {}: {}", delta.target_stage.as_str(), delta.change);
            
            // 根据目标阶段，修改对应的 artifact
            match delta.target_stage {
                Stage::IdeaIntake => {
                    // 修改 IdeaSpec（一般不常见）
                    tracing::info!("Applying delta to IdeaSpec: {}", delta.change);
                }
                Stage::Requirements => {
                    // 修改 PRD
                    self.apply_delta_to_prd(session_id, &delta.change)?;
                }
                Stage::Design => {
                    // 修改 Design
                    self.apply_delta_to_design(session_id, &delta.change)?;
                }
                Stage::Plan => {
                    // 修改 Plan
                    self.apply_delta_to_plan(session_id, &delta.change)?;
                }
                _ => {
                    tracing::warn!("Delta target stage {:?} not supported yet", delta.target_stage);
                }
            }
        }
        
        Ok(())
    }

    /// 应用 delta 到 PRD
    fn apply_delta_to_prd(&self, session_id: &str, change: &str) -> Result<()> {
        // 加载现有 PRD
        let mut prd_artifact: crate::artifacts::PRDArtifact = 
            self.load_artifact(session_id, Stage::Requirements)?;
        
        // 简单实现：将变更添加到 hitl 问题中（标记为待处理）
        prd_artifact.data.hitl.push(crate::artifacts::HitlQuestion {
            id: format!("FEEDBACK-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
            q: format!("反馈修改: {}", change),
            opts: vec!["是".to_string(), "否".to_string()],
            def: "是".to_string(),
        });
        
        // 保存修改后的 PRD
        self.store.put(session_id, Stage::Requirements, &prd_artifact)?;
        
        tracing::info!("Applied delta to PRD: {}", change);
        Ok(())
    }

    /// 应用 delta 到 Design
    fn apply_delta_to_design(&self, session_id: &str, change: &str) -> Result<()> {
        let mut design_artifact: crate::artifacts::DesignDocArtifact = 
            self.load_artifact(session_id, Stage::Design)?;
        
        // 简单实现：添加变更说明到组件列表中
        design_artifact.data.arch.comps.push(format!("反馈修改: {}", change));
        
        self.store.put(session_id, Stage::Design, &design_artifact)?;
        
        tracing::info!("Applied delta to Design: {}", change);
        Ok(())
    }

    /// 应用 delta 到 Plan
    fn apply_delta_to_plan(&self, session_id: &str, change: &str) -> Result<()> {
        let mut plan_artifact: crate::artifacts::PlanArtifact = 
            self.load_artifact(session_id, Stage::Plan)?;
        
        // 简单实现：添加新任务
        plan_artifact.data.tasks.push(crate::artifacts::Task {
            id: format!("FEEDBACK-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
            pri: crate::artifacts::Priority::P1,
            desc: format!("反馈修改: {}", change),
            deps: vec![],
            out: vec![],
        });
        
        self.store.put(session_id, Stage::Plan, &plan_artifact)?;
        
        tracing::info!("Applied delta to Plan: {}", change);
        Ok(())
    }

    /// 找到需要重跑的最早阶段
    fn find_earliest_stage(&self, reruns: &[crate::artifacts::Rerun]) -> Stage {
        let all_stages = Stage::all();
        
        for stage in all_stages {
            if reruns.iter().any(|r| r.stage == *stage) {
                return *stage;
            }
        }
        
        // 默认从 Requirements 开始
        Stage::Requirements
    }

    /// 清除指定阶段及之后所有阶段的完成状态
    fn clear_stages_from(&self, meta: &mut SessionMeta, start_stage: Stage) -> Result<()> {
        let all_stages = Stage::all();
        let mut should_clear = false;
        
        for stage in all_stages {
            if *stage == start_stage {
                should_clear = true;
            }
            
            if should_clear {
                // 移除完成状态
                meta.stage_status.remove(stage);
                println!("  清除 {} 阶段状态", stage.as_str());
            }
        }
        
        self.save_session_meta(meta)?;
        Ok(())
    }
}
