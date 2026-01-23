use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::artifacts::Stage;
use crate::memory::ArtifactStore;
use crate::hitl::HitlController;

/// 统一的阶段 Agent 接口
/// 所有阶段的 Agent 都应该实现这个 trait
#[async_trait]
pub trait StageAgent: Send + Sync {
    /// 该 Agent 负责的阶段
    fn stage(&self) -> Stage;
    
    /// 执行 Agent 的核心逻辑
    /// 
    /// # 参数
    /// - `context`: 执行上下文，包含 session_id、store、hitl 等
    /// 
    /// # 返回
    /// - `Ok(result)`: 成功执行，返回结果包含 artifact_id 等信息
    /// - `Err(e)`: 执行失败
    async fn execute(&self, context: &StageAgentContext) -> Result<StageAgentResult>;
    
    /// 可选：获取该阶段的依赖阶段
    fn dependencies(&self) -> Vec<Stage> {
        Vec::new()
    }
    
    /// 可选：是否需要 HITL 审查
    fn requires_hitl_review(&self) -> bool {
        true
    }
    
    /// 可选：获取 Agent 的描述
    fn description(&self) -> &str {
        "No description"
    }
}

/// Agent 执行上下文
/// 包含所有 Agent 执行时需要的共享资源
pub struct StageAgentContext {
    pub session_id: String,
    pub store: Arc<ArtifactStore>,
    pub hitl: Arc<HitlController>,
    /// 可选：用户提供的额外输入
    pub user_input: Option<String>,
}

impl StageAgentContext {
    pub fn new(
        session_id: String,
        store: Arc<ArtifactStore>,
        hitl: Arc<HitlController>,
    ) -> Self {
        Self {
            session_id,
            store,
            hitl,
            user_input: None,
        }
    }
    
    pub fn with_user_input(mut self, input: String) -> Self {
        self.user_input = Some(input);
        self
    }
    
    /// 从 store 加载指定阶段的 artifact
    pub fn load_artifact<T>(&self, stage: Stage) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        use std::fs;
        
        let artifacts = self.store.list(&self.session_id)?;
        
        let artifact_meta = artifacts
            .iter()
            .filter(|a| a.stage == stage)
            .max_by_key(|a| &a.path_json)
            .ok_or_else(|| anyhow::anyhow!("No artifact found for stage {:?}", stage))?;

        let content = fs::read_to_string(&artifact_meta.path_json)?;
        let artifact: T = serde_json::from_str(&content)?;
        
        Ok(artifact)
    }
}

/// Agent 执行结果
pub struct StageAgentResult {
    pub artifact_id: String,
    pub stage: Stage,
    pub verified: bool,
    pub summary: Vec<String>,
}

impl StageAgentResult {
    pub fn new(artifact_id: String, stage: Stage) -> Self {
        Self {
            artifact_id,
            stage,
            verified: true,
            summary: Vec::new(),
        }
    }
    
    pub fn with_verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
    
    pub fn with_summary(mut self, summary: Vec<String>) -> Self {
        self.summary = summary;
        self
    }
}
