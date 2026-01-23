use anyhow::Result;
use std::sync::Arc;

use crate::artifacts::*;
use crate::memory::ArtifactStore;
use crate::config::LlmConfig;
use adk_rust::model::{OpenAIClient, OpenAIConfig};

/// Code Updater Agent - 增量修改现有代码
/// 
/// 核心功能：
/// 1. 分析需求变更，识别受影响的文件
/// 2. 生成增量修改计划（而非全量重新生成）
/// 3. 保护用户手动修改的代码
/// 4. 支持合并策略（覆盖/合并/保留）
#[allow(dead_code)]
pub struct CodeUpdater {
    model: Arc<OpenAIClient>,
    store: Arc<ArtifactStore>,
}

impl CodeUpdater {
    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        let config = OpenAIConfig::compatible(
            llm_config.api_key.clone(),
            llm_config.api_base_url.clone(),
            llm_config.model_name.clone(),
        );
        
        let client = OpenAIClient::new(config)?;
        
        Ok(Self {
            model: Arc::new(client),
            store,
        })
    }

    /// 分析需求变更，生成增量更新计划
    /// 
    /// # 参数
    /// - session_id: 会话 ID
    /// - old_prd: 旧版 PRD
    /// - new_prd: 新版 PRD
    /// - existing_code: 现有代码变更记录
    /// 
    /// # 返回
    /// - IncrementalUpdatePlan: 增量更新计划
    pub async fn analyze_changes(
        &self,
        session_id: &str,
        old_prd: &PRD,
        new_prd: &PRD,
        existing_code: &CodeChange,
    ) -> Result<IncrementalUpdatePlan> {
        tracing::info!("Analyzing requirement changes for session {}", session_id);
        
        // 1. 识别新增、修改、删除的需求
        let req_changes = self.diff_requirements(old_prd, new_prd);
        
        // 2. 基于 RequirementMapping 找到受影响的文件
        let affected_files = self.find_affected_files(&req_changes, existing_code);
        
        // 3. 生成修改策略
        let update_plan = IncrementalUpdatePlan {
            added_requirements: req_changes.added.clone(),
            modified_requirements: req_changes.modified.clone(),
            removed_requirements: req_changes.removed.clone(),
            affected_files,
            merge_strategy: MergeStrategy::Smart,  // 默认智能合并
        };
        
        tracing::info!(
            "Update plan: {} added, {} modified, {} removed requirements, {} affected files",
            update_plan.added_requirements.len(),
            update_plan.modified_requirements.len(),
            update_plan.removed_requirements.len(),
            update_plan.affected_files.len()
        );
        
        Ok(update_plan)
    }
    
    /// Diff 两个 PRD，识别变化
    fn diff_requirements(&self, old_prd: &PRD, new_prd: &PRD) -> RequirementChanges {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();
        
        // 识别新增和修改
        for new_req in &new_prd.reqs {
            match old_prd.reqs.iter().find(|r| r.id == new_req.id) {
                Some(old_req) => {
                    // 检查是否有修改
                    if old_req.desc != new_req.desc || old_req.pri != new_req.pri {
                        modified.push(new_req.clone());
                    }
                }
                None => {
                    // 新增需求
                    added.push(new_req.clone());
                }
            }
        }
        
        // 识别删除
        for old_req in &old_prd.reqs {
            if !new_prd.reqs.iter().any(|r| r.id == old_req.id) {
                removed.push(old_req.id.clone());
            }
        }
        
        RequirementChanges {
            added,
            modified,
            removed,
        }
    }
    
    /// 查找受影响的文件
    fn find_affected_files(
        &self,
        req_changes: &RequirementChanges,
        existing_code: &CodeChange,
    ) -> Vec<AffectedFile> {
        let mut affected = Vec::new();
        
        // 遍历所有需求映射
        for mapping in &existing_code.requirement_mapping {
            let mut impact = FileImpact::None;
            
            // 检查是否被删除
            if req_changes.removed.contains(&mapping.req_id) {
                impact = FileImpact::RequirementRemoved;
            }
            // 检查是否被修改
            else if req_changes.modified.iter().any(|r| r.id == mapping.req_id) {
                impact = FileImpact::RequirementModified;
            }
            
            if impact != FileImpact::None {
                for file_path in &mapping.files {
                    affected.push(AffectedFile {
                        path: file_path.clone(),
                        impact,
                        related_requirement: mapping.req_id.clone(),
                    });
                }
            }
        }
        
        // 新增需求需要创建新文件（暂时标记为 None，后续由 CodePlanner 决定）
        
        affected
    }
}

/// 需求变更记录
#[derive(Debug, Clone)]
pub struct RequirementChanges {
    pub added: Vec<Requirement>,
    pub modified: Vec<Requirement>,
    pub removed: Vec<String>,  // 需求 ID
}

/// 增量更新计划
#[derive(Debug, Clone)]
pub struct IncrementalUpdatePlan {
    pub added_requirements: Vec<Requirement>,
    pub modified_requirements: Vec<Requirement>,
    pub removed_requirements: Vec<String>,
    pub affected_files: Vec<AffectedFile>,
    pub merge_strategy: MergeStrategy,
}

/// 受影响的文件
#[derive(Debug, Clone)]
pub struct AffectedFile {
    pub path: String,
    pub impact: FileImpact,
    pub related_requirement: String,
}

/// 文件影响类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileImpact {
    None,
    RequirementModified,  // 需求修改
    RequirementRemoved,   // 需求删除
}

/// 合并策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// 覆盖（危险：丢失用户修改）
    Overwrite,
    /// 智能合并（保留用户修改，添加新功能）
    Smart,
    /// 保留原文件，生成 .new 文件
    KeepOriginal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_requirements() {
        let old_prd = PRD {
            scope: Scope {
                g: vec!["Test".to_string()],
                ng: vec![],
            },
            reqs: vec![
                Requirement {
                    id: "REQ-001".to_string(),
                    pri: Priority::P0,
                    req_type: RequirementType::Func,
                    desc: "Old description".to_string(),
                    deps: vec![],
                    ac: vec![],
                },
            ],
            cons: vec![],
            hitl: vec![],
        };
        
        let new_prd = PRD {
            scope: Scope {
                g: vec!["Test".to_string()],
                ng: vec![],
            },
            reqs: vec![
                Requirement {
                    id: "REQ-001".to_string(),
                    desc: "New description".to_string(),  // 修改
                    pri: Priority::P0,
                    req_type: RequirementType::Func,
                    deps: vec![],
                    ac: vec![],
                },
                Requirement {
                    id: "REQ-002".to_string(),  // 新增
                    desc: "New requirement".to_string(),
                    pri: Priority::P1,
                    req_type: RequirementType::Func,
                    deps: vec![],
                    ac: vec![],
                },
            ],
            cons: vec![],
            hitl: vec![],
        };
        
        // 创建临时存储和配置
        let store = Arc::new(ArtifactStore::new(".cowork_test"));
        let llm_config = LlmConfig {
            api_key: "test".to_string(),
            api_base_url: "http://test".to_string(),
            model_name: "test".to_string(),
        };
        
        let updater = CodeUpdater::new(&llm_config, store).unwrap();
        let changes = updater.diff_requirements(&old_prd, &new_prd);
        
        assert_eq!(changes.added.len(), 1);
        assert_eq!(changes.added[0].id, "REQ-002");
        
        assert_eq!(changes.modified.len(), 1);
        assert_eq!(changes.modified[0].id, "REQ-001");
        
        assert_eq!(changes.removed.len(), 0);
    }
}
