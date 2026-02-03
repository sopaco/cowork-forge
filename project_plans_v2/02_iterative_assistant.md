# 方案 02: 智能迭代调度 Agent

**版本**: 1.0  
**创建日期**: 2026-02-02  
**Phase**: Phase 0  
**工作量**: 32h  
**优先级**: P0 (必须)

---

## 📋 概述

### 1.1 需求背景

**当前问题**:
- New 创建的项目在 Completed 后无法继续聊天交互
- 用户需要手动选择 Resume/Modify 操作，学习成本高
- 项目完成后继续开发的流程不够自然
- 缺少智能 Agent 来判断用户意图并自动调度

**目标**:
- 项目 Completed 后，聊天框继续可用
- 引入智能 Agent（IterativeAssistant）自动调度修改模式
- 根据用户输入智能判断是否需要 Resume/Modify/New
- 提供更自然的持续开发体验

### 1.2 设计原则

- ✅ 用户体验优先
- ✅ 智能意图识别
- ✅ 透明决策过程
- ✅ 保持用户控制权

---

## 🧠 技术方案

### 2.1 IterativeAssistant Agent

```rust
// cowork-core/src/agents/iterative_assistant.rs

use anyhow::Result;
use adk_core::{Agent, InvocationContext, Event, EventStream};

/// 迭代助手 Agent - 负责智能调度修改模式
pub struct IterativeAssistant {
    llm: Arc<dyn Llm>,
}

impl IterativeAssistant {
    pub fn new(llm: Arc<dyn Llm>) -> Self {
        Self { llm }
    }
    
    /// 分析用户意图
    pub async fn analyze_user_intent(&self, 
        user_input: &str, 
        current_session_status: &SessionStatus,
        project_context: &ProjectContext,
    ) -> Result<IterationIntent> {
        let prompt = format!(
            r#"
你是一个项目迭代助手，负责分析用户的意图并决定下一步行动。

当前状态:
- Session 状态: {:?}
- 项目上下文: {}

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
"#,
            current_session_status,
            serde_json::to_string_pretty(project_context)?,
            user_input
        );
        
        let response = self.llm.generate(&prompt).await?;
        
        // 解析 LLM 响应
        let intent: IterationIntent = serde_json::from_str(&response)?;
        
        Ok(intent)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationIntent {
    pub intent_type: IterationType,
    pub confidence: f64,
    pub reasoning: String,
    pub suggested_action: SuggestedAction,
    pub related_features: Vec<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestedAction {
    Modify,
    Resume,
    New,
    AskClarification,
    Ignore,
}
```

### 2.2 修改建议生成

```rust
/// 生成修改建议
pub async fn generate_modify_suggestion(
    llm: Arc<dyn Llm>,
    project_context: &ProjectContext,
    user_input: &str,
) -> Result<ModifySuggestion> {
    let prompt = format!(
        r#"
用户请求对项目进行修改。

项目上下文:
{}

用户输入: {}

请分析这个修改请求，并提供详细的修改建议，包括:

1. 修改类型识别（功能新增/功能修改/Bug修复/重构）
2. 受影响模块分析
3. 实施计划
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
"#,
        serde_json::to_string_pretty(project_context)?,
        user_input
    );
    
    let response = llm.generate(&prompt).await?;
    let suggestion: ModifySuggestion = serde_json::from_str(&response)?;
    
    Ok(suggestion)
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_level: String,
    pub risks: Vec<String>,
    pub mitigation_strategies: Vec<String>,
}
```

### 2.3 持续聊天交互

```rust
/// 处理聊天消息
pub async fn handle_chat_message(
    message: &str,
    session_id: &str,
    llm: Arc<dyn Llm>,
    storage: Arc<dyn Storage>,
) -> Result<ChatResponse> {
    // 检查 Session 状态
    let session_record = storage.get_session_record(session_id)?;
    
    match session_record.status {
        SessionStatus::InProgress => {
            // Session 进行中，直接处理消息
            Ok(ChatResponse::DirectProcessing)
        }
        SessionStatus::Completed => {
            // Session 已完成，使用 IterativeAssistant
            let assistant = IterativeAssistant::new(llm);
            
            // 获取项目上下文
            let project_context = load_project_context(session_id, storage)?;
            
            // 分析用户意图
            let intent = assistant.analyze_user_intent(
                message,
                &SessionStatus::Completed,
                &project_context,
            ).await?;
            
            match intent.intent_type {
                IterationType::ContinueDevelopment |
                IterationType::StartNewFeature |
                IterationType::FixBug |
                IterationType::Refactor => {
                    // 生成修改建议
                    let suggestion = generate_modify_suggestion(
                        llm,
                        &project_context,
                        message,
                    ).await?;
                    
                    Ok(ChatResponse::SuggestModify {
                        intent,
                        suggestion,
                    })
                }
                IterationType::NewProject => {
                    Ok(ChatResponse::SuggestNewProject {
                        intent,
                    })
                }
                IterationType::Clarification => {
                    Ok(ChatResponse::AskClarification {
                        intent,
                        questions: vec![
                            "你想做什么？".to_string(),
                            "是修改现有项目还是创建新项目？".to_string(),
                        ],
                    })
                }
            }
        }
        SessionStatus::Failed => {
            // Session 失败，建议恢复
            Ok(ChatResponse::SuggestResume {
                session_id: session_id.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChatResponse {
    DirectProcessing,
    SuggestModify {
        intent: IterationIntent,
        suggestion: ModifySuggestion,
    },
    SuggestNewProject {
        intent: IterationIntent,
    },
    AskClarification {
        intent: IterationIntent,
        questions: Vec<String>,
    },
    SuggestResume {
        session_id: String,
    },
}
```

### 2.4 Tauri 命令

```rust
// 发送聊天消息
#[tauri::command]
async fn send_chat_message(
    session_id: String,
    message: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<ChatActionResult, String> {
    // 处理聊天消息
    let response = handle_chat_message(
        &message,
        &session_id,
        state.llm.clone(),
        state.storage.clone(),
    ).await?;
    
    match response {
        ChatResponse::DirectProcessing => {
            // 直接处理，启动新 Modify Session
            let new_session_id = create_modify_session(
                &session_id,
                &message,
                window.clone(),
                state.clone(),
            ).await?;
            
            Ok(ActionResult::DirectProcessing { new_session_id })
        }
        ChatResponse::SuggestModify { intent, suggestion } => {
            // 发送修改建议到前端
            window.emit("modify_suggested", serde_json::json!({
                "intent": intent,
                "suggestion": suggestion,
                "session_id": session_id,
            }))?;
            
            Ok(ActionResult::AwaitConfirmation {
                action_type: "modify",
                data: suggestion,
            })
        }
        ChatResponse::SuggestNewProject { intent } => {
            // 发送新项目建议
            window.emit("new_project_suggested", serde_json::json!({
                "intent": intent,
                "session_id": session_id,
            }))?;
            
            Ok(ActionResult::AwaitConfirmation {
                action_type: "new_project",
                data: intent,
            })
        }
        ChatResponse::AskClarification { intent, questions } => {
            // 发送澄清请求
            window.emit("clarification_requested", serde_json::json!({
                "intent": intent,
                "questions": questions,
                "session_id": session_id,
            }))?;
            
            Ok(ActionResult::AwaitClarification)
        }
        ChatResponse::SuggestResume { session_id } => {
            // 发送恢复建议
            window.emit("resume_suggested", serde_json::json!({
                "session_id": session_id,
            }))?;
            
            Ok(ActionResult::AwaitConfirmation {
                action_type: "resume",
                data: session_id,
            })
        }
    }
}

// 确认修改建议
#[tauri::command]
async fn confirm_modify(
    session_id: String,
    suggestion: ModifySuggestion,
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 创建 Modify Session
    let new_session_id = create_modify_session(
        &session_id,
        &suggestion.title,
        window.clone(),
        state.clone(),
    ).await?;
    
    Ok(new_session_id)
}
```

---

## 🎨 前端实现

### 3.1 聊天交互增强

```jsx
// 增强的聊天处理
const handleSendUserMessage = async () => {
  if (!userInput.trim()) return;
  
  setMessages(prev => [...prev, { type: 'user', content: userInput }]);
  
  if (inputRequest) {
    // HITL 交互
    await invoke('submit_input_response', { 
      requestId: inputRequest.requestId, 
      response: userInput, 
      responseType: 'text' 
    });
    setInputRequest(null);
  } else {
    // 普通聊天消息
    const response = await invoke('send_chat_message', {
      sessionId: currentSession,
      message: userInput,
    });
    
    handleChatResponse(response);
  }
  
  setUserInput('');
};

const handleChatResponse = (response) => {
  switch (response.type) {
    case 'direct_processing':
      // 直接处理中
      setIsProcessing(true);
      break;
      
    case 'await_confirmation':
      // 等待确认
      setShowConfirmationDialog(true);
      setConfirmationData(response.data);
      break;
      
    case 'await_clarification':
      // 需要澄清
      setShowClarificationDialog(true);
      setClarificationData(response.data);
      break;
  }
};
```

### 3.2 修改建议确认对话框

```jsx
const ModifySuggestionDialog = ({ visible, suggestion, onConfirm, onCancel }) => {
  return (
    <Modal
      title="修改建议"
      visible={visible}
      onOk={onConfirm}
      onCancel={onCancel}
      width={700}
      okText="开始修改"
      cancelText="取消"
    >
      {suggestion && (
        <div>
          <h3>{suggestion.title}</h3>
          
          <Divider />
          
          <h4>修改类型</h4>
          <Tag color="blue">{suggestion.modification_type}</Tag>
          
          <Divider />
          
          <h4>受影响模块</h4>
          <div>
            {suggestion.affected_modules.map(module => (
              <Tag key={module}>{module}</Tag>
            ))}
          </div>
          
          <Divider />
          
          <h4>实施计划</h4>
          <ol>
            {suggestion.implementation_plan.map((step, idx) => (
              <li key={idx}>{step}</li>
            ))}
          </ol>
          
          <Divider />
          
          <h4>风险评估</h4>
          <div>
            <p><strong>风险等级:</strong> <Tag color={getRiskColor(suggestion.risk_assessment.risk_level)}>
              {suggestion.risk_assessment.risk_level}
            </Tag></p>
            <p><strong>预估工作量:</strong> {suggestion.estimated_effort}</p>
            <p><strong>置信度:</strong> {(suggestion.confidence * 100).toFixed(0)}%</p>
            
            {suggestion.risk_assessment.risks.length > 0 && (
              <>
                <h5>风险:</h5>
                <ul>
                  {suggestion.risk_assessment.risks.map((risk, idx) => (
                    <li key={idx}>{risk}</li>
                  ))}
                </ul>
              </>
            )}
            
            {suggestion.risk_assessment.mitigation_strategies.length > 0 && (
              <>
                <h5>缓解策略:</h5>
                <ul>
                  {suggestion.risk_assessment.mitigation_strategies.map((strategy, idx) => (
                    <li key={idx}>{strategy}</li>
                  ))}
                </ul>
              </>
            )}
          </div>
        </div>
      )}
    </Modal>
  );
};
```

---

## 📅 实施计划

### 4.1 任务分解 (32h)

#### 后端实现 (20h)
- [ ] IterativeAssistant Agent (8h)
- [ ] 用户意图分析 (4h)
- [ ] 修改建议生成 (4h)
- [ ] 持续聊天交互 (4h)
- [ ] 项目上下文加载 (2h)
- [ ] Tauri 命令实现 (6h)
- [ ] 错误处理 (2h)

#### 前端实现 (10h)
- [ ] 聊天交互增强 (4h)
- [ ] 修改建议对话框 (3h)
- [ ] 确认流程 (2h)
- [ ] 状态提示 (1h)

#### 测试与优化 (2h)
- [ ] 意图识别准确率测试 (1h)
- [ ] 用户体验测试 (1h)

---

## 🎯 验收标准

### 功能验收
- ✅ 项目 Completed 后聊天框继续可用
- ✅ 能够智能识别用户意图（6种类型）
- ✅ 能够生成详细的修改建议
- ✅ 支持用户确认或拒绝建议
- ✅ 支持澄清请求

### 技术验收
- ✅ 意图识别准确率 ≥ 80%
- ✅ 修改建议置信度 ≥ 0.7
- ✅ 支持所有项目类型
- ✅ 错误处理完善

---

**文档版本**: 1.0  
**创建时间**: 2026-02-02  
**Phase**: Phase 0  
**工作量**: 32h