// Iterative Assistant Agent - Intelligent iteration scheduling
// Analyzes user intent and auto-schedules Modify mode for completed projects

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// Data Structures
// ============================================================================

/// Iteration intent types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IterationType {
    ContinueDevelopment,
    StartNewFeature,
    FixBug,
    Refactor,
    NewProject,
    Clarification,
}

/// Suggested action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestedAction {
    Modify,
    Resume,
    New,
    AskClarification,
    Ignore,
}

/// Iteration intent result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationIntent {
    pub intent_type: IterationType,
    pub confidence: f64,
    pub reasoning: String,
    pub suggested_action: SuggestedAction,
    pub related_features: Vec<String>,
}

/// Modify suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifySuggestion {
    pub modification_type: String,
    pub title: String,
    pub affected_modules: Vec<String>,
    pub implementation_plan: Vec<String>,
    pub risk_assessment: RiskAssessment,
    pub estimated_effort: String,
    pub confidence: f64,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_level: String,
    pub risks: Vec<String>,
    pub mitigation_strategies: Vec<String>,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub status: String,
    pub session_type: String,
    pub created_at: String,
}

/// Project context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_name: String,
    pub sessions: Vec<SessionInfo>,
    pub technology_stack: Vec<String>,
}

/// Session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
}

/// Chat response type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ChatResponse {
    DirectProcessing,
    AwaitConfirmation {
        action_type: String,
        data: ModifySuggestion,
    },
    AwaitClarification {
        intent: IterationIntent,
        questions: Vec<String>,
    },
    SuggestResume {
        session_id: String,
    },
}

// ============================================================================
// Iterative Assistant
// ============================================================================

pub struct IterativeAssistant {
    llm: Arc<dyn LlmProvider>,
}

pub trait LlmProvider: Send + Sync {
    fn generate(&self, prompt: &str) -> Result<String>;
}

impl IterativeAssistant {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
    
    /// Analyze user intent
    pub async fn analyze_user_intent(
        &self,
        user_input: &str,
        current_session_status: &SessionStatus,
        project_context: &ProjectContext,
    ) -> Result<IterationIntent> {
        let project_context_json = serde_json::to_string_pretty(project_context)?;
        
        let prompt = format!(
            r#"你是一个项目迭代助手，负责分析用户的意图并决定下一步行动。

当前状态:
- Session 状态: {}
- Session 类型: {}
- 创建时间: {}

项目上下文:
{}

用户输入: {}

请分析用户的意图，从以下选项中选择最合适的一个:

1. ContinueDevelopment - 继续开发当前项目（用户想要继续完善项目）
2. StartNewFeature - 开发新功能（用户想要添加新功能）
3. FixBug - 修复问题（用户报告了 bug）
4. Refactor - 重构代码（用户想要重构）
5. NewProject - 创建全新项目（用户想要开始新项目）
6. Clarification - 需要更多信息（用户意图不明确）

请以 JSON 格式返回你的分析结果:
{{
  "intent_type": "ContinueDevelopment",
  "confidence": 0.9,
  "reasoning": "用户说'继续优化首页性能'，这表明用户想要继续改进当前项目",
  "suggested_action": "modify",
  "related_features": ["首页", "性能优化"]
}}

注意：
- intent_type 必须是上面列出的6种之一
- confidence 是0到1之间的浮点数
- suggested_action 可以是 "modify", "resume", "new", "ask_clarification", "ignore"
- reasoning 简洁解释你的判断
- related_features 列出相关的功能或模块
"#,
            current_session_status.status,
            current_session_status.session_type,
            current_session_status.created_at,
            project_context_json,
            user_input
        );
        
        let response = self.llm.generate(&prompt)?;
        
        // Parse LLM response
        let intent: IterationIntent = serde_json::from_str(&response)
            .map_err(|e| anyhow::anyhow!("Failed to parse intent response: {}. Response was: {}", e, response))?;
        
        Ok(intent)
    }
    
    /// Generate modify suggestion
    pub async fn generate_modify_suggestion(
        &self,
        project_context: &ProjectContext,
        user_input: &str,
    ) -> Result<ModifySuggestion> {
        let project_context_json = serde_json::to_string_pretty(project_context)?;
        
        let prompt = format!(
            r#"用户请求对项目进行修改。

项目上下文:
{}

用户输入: {}

请分析这个修改请求，并提供详细的修改建议，包括:

1. 修改类型识别（feature_addition, feature_modification, bug_fix, refactor）
2. 受影响模块分析
3. 实施计划（分步骤）
4. 风险评估

请以 JSON 格式返回:
{{
  "modification_type": "feature_addition",
  "title": "添加用户评论功能",
  "affected_modules": ["frontend", "backend", "database"],
  "implementation_plan": [
    "1. 设计评论数据模型",
    "2. 实现后端 API",
    "3. 实现前端 UI",
    "4. 集成到现有系统"
  ],
  "risk_assessment": {{
    "risk_level": "medium",
    "risks": [
      "可能需要数据库迁移",
      "影响现有功能"
    ],
    "mitigation_strategies": [
      "使用数据库迁移工具",
      "充分测试现有功能"
    ]
  }},
  "estimated_effort": "2-3天",
  "confidence": 0.85
}}

注意：
- modification_type 可以是 "feature_addition", "feature_modification", "bug_fix", "refactor"
- risk_level 可以是 "low", "medium", "high", "critical"
- estimated_effort 使用描述性文本，如 "2-3天", "1周" 等
- confidence 是0到1之间的浮点数
"#,
            project_context_json,
            user_input
        );
        
        let response = self.llm.generate(&prompt)?;
        
        // Parse LLM response
        let suggestion: ModifySuggestion = serde_json::from_str(&response)
            .map_err(|e| anyhow::anyhow!("Failed to parse suggestion response: {}. Response was: {}", e, response))?;
        
        Ok(suggestion)
    }
    
    /// Generate clarification questions
    pub async fn generate_clarification_questions(
        &self,
        user_input: &str,
        project_context: &ProjectContext,
    ) -> Result<Vec<String>> {
        let project_context_json = serde_json::to_string_pretty(project_context)?;
        
        let prompt = format!(
            r#"用户的请求不够明确，需要更多信息。

项目上下文:
{}

用户输入: {}

请生成2-3个澄清问题，帮助更好地理解用户的需求。

请以 JSON 格式返回:
{{
  "questions": [
    "你想修改哪个功能？",
    "具体要实现什么效果？",
    "有没有参考的例子？"
  ]
}}

注意：
- 问题要简洁明了
- 问题要能帮助缩小需求范围
- 最多3个问题
"#,
            project_context_json,
            user_input
        );
        
        let response = self.llm.generate(&prompt)?;
        
        // Parse LLM response
        let result: serde_json::Value = serde_json::from_str(&response)?;
        
        let questions: Vec<String> = result["questions"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|q| q.as_str().map(|s| s.to_string()))
            .collect();
        
        Ok(questions)
    }
}

// ============================================================================
// Mock LLM Provider for testing
// ============================================================================

#[cfg(test)]
pub struct MockLlmProvider;

#[cfg(test)]
impl LlmProvider for MockLlmProvider {
    fn generate(&self, _prompt: &str) -> Result<String> {
        Ok(r#"{
  "intent_type": "ContinueDevelopment",
  "confidence": 0.9,
  "reasoning": "Mock response",
  "suggested_action": "modify",
  "related_features": []
}"#.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_iteration_intent_serialize() {
        let intent = IterationIntent {
            intent_type: IterationType::ContinueDevelopment,
            confidence: 0.9,
            reasoning: "Test".to_string(),
            suggested_action: SuggestedAction::Modify,
            related_features: vec!["test".to_string()],
        };
        
        let json = serde_json::to_string(&intent).unwrap();
        assert!(json.contains("continue_development"));
    }
    
    #[test]
    fn test_modify_suggestion_serialize() {
        let suggestion = ModifySuggestion {
            modification_type: "feature_addition".to_string(),
            title: "Test".to_string(),
            affected_modules: vec!["test".to_string()],
            implementation_plan: vec!["1".to_string()],
            risk_assessment: RiskAssessment {
                risk_level: "low".to_string(),
                risks: vec![],
                mitigation_strategies: vec![],
            },
            estimated_effort: "1天".to_string(),
            confidence: 0.8,
        };
        
        let json = serde_json::to_string(&suggestion).unwrap();
        assert!(json.contains("feature_addition"));
    }
}