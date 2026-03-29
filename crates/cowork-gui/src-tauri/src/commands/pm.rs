// PM Agent Commands - Delegates to Core layer's PM Agent implementation
// This file is now a thin adapter that calls cowork-core's PM Agent

use crate::AppState;
use crate::TauriBackend;
use cowork_core::persistence::IterationStore;
use cowork_core::pipeline::IterationExecutor;
use cowork_core::persistence::ProjectStore;
use cowork_core::llm::{load_config, create_llm_client};
use cowork_core::{PMAgentStreamCallback, PMAgentAction, execute_pm_agent_message_streaming};
use std::sync::Arc;
use tauri::{Emitter, Manager, State, Window};
use async_trait::async_trait;

// ============================================================================
// Streaming Callback - Bridges Core layer events to Tauri events
// ============================================================================

struct TauriStreamCallback {
    window: tauri::AppHandle,
}

#[async_trait]
impl PMAgentStreamCallback for TauriStreamCallback {
    async fn on_text_chunk(&self, text: &str, is_first: bool, is_last: bool) {
        let _ = self.window.emit("agent_streaming", serde_json::json!({
            "content": text,
            "agent_name": "PM Agent",
            "is_thinking": false,
            "is_first": is_first,
            "is_last": is_last
        }));
    }

    async fn on_tool_call(&self, tool_name: &str, args: &serde_json::Value) {
        println!("[PM GUI] Tool called: {} with args: {:?}", tool_name, args);
    }
}

// ============================================================================
// PM Send Message - Main entry point for PM Agent chat
// ============================================================================

#[tauri::command]
pub async fn pm_send_message(
    iteration_id: String,
    message: String,
    history: Vec<serde_json::Value>,
    window: Window,
) -> Result<serde_json::Value, String> {
    eprintln!("[PM] pm_send_message called: iteration_id={}, message={}, history_len={}", 
        iteration_id, message, history.len());
    
    let store = IterationStore::new();
    let iteration = store.load(&iteration_id).map_err(|e| format!("Failed to load iteration: {}", e))?;
    
    let is_first_message = history.is_empty();
    
    // Handle welcome message for first interaction
    if is_first_message {
        let welcome_msg = format!(
            "👋 你好！我是项目经理助手。\n\n项目 **{}** 已经完成开发！\n\n接下来你可以：",
            iteration.title
        );
        
        let actions = vec![
            serde_json::json!({ "action_type": "pm_start_app", "label": "🚀 启动应用" }),
            serde_json::json!({ "action_type": "pm_open_folder", "label": "📁 打开项目文件夹" }),
            serde_json::json!({ "action_type": "pm_view_artifacts", "label": "📄 查看设计文档" }),
            serde_json::json!({ "action_type": "pm_view_code", "label": "💻 查看源代码" }),
            serde_json::json!({ "action_type": "pm_view_knowledge", "label": "📚 查看项目知识库" }),
        ];
        
        let result = serde_json::json!({
            "agent_message": welcome_msg,
            "actions": actions,
            "needs_restart": false
        });
        
        let _ = window.emit("pm_message", &result);
        return Ok(result);
    }
    
    // Load config and create LLM client
    let config = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let model = create_llm_client(&config.llm).map_err(|e| format!("Failed to create LLM client: {}", e))?;
    
    // Create streaming callback
    let callback = Arc::new(TauriStreamCallback {
        window: window.app_handle().clone(),
    });
    
    // Execute PM Agent via Core layer
    let result = execute_pm_agent_message_streaming(
        model,
        iteration_id.clone(),
        message,
        history,
        Some(callback),
    )
    .await
    .map_err(|e| format!("PM Agent execution failed: {}", e))?;
    
    // Convert PMAgentAction to frontend-friendly format
    let actions: Vec<serde_json::Value> = result.actions.iter().map(|action| {
        match action {
            PMAgentAction::GotoStage { target_stage, reason: _ } => {
                let stage_names = get_stage_names();
                let stage_name = stage_names.get(target_stage.as_str())
                    .map(|s| *s)
                    .unwrap_or(target_stage.as_str());
                
                // Send action prompt via streaming
                let _ = window.emit("agent_streaming", serde_json::json!({
                    "content": format!("\n\n点击下方按钮确认跳转到 **{}**：", stage_name),
                    "agent_name": "PM Agent",
                    "is_thinking": false,
                    "is_first": false,
                    "is_last": false
                }));
                
                serde_json::json!({
                    "action_type": "pm_goto_stage",
                    "target_stage": target_stage,
                    "label": format!("🔄 跳转到 {}", stage_name)
                })
            }
            PMAgentAction::CreateIteration { iteration_id, title, description: _, inheritance: _ } => {
                // Emit iteration_created event
                let _ = window.emit("iteration_created", iteration_id);
                
                // Send action prompt via streaming
                let _ = window.emit("agent_streaming", serde_json::json!({
                    "content": format!("\n\n我已经创建了新迭代 **{}**。\n\n点击下方按钮启动新迭代：", title),
                    "agent_name": "PM Agent",
                    "is_thinking": false,
                    "is_first": false,
                    "is_last": false
                }));
                
                serde_json::json!({
                    "action_type": "pm_create_iteration",
                    "iteration_id": iteration_id,
                    "title": title,
                    "label": "🚀 启动新迭代"
                })
            }
        }
    }).collect();
    
    // Send stream end signal
    let _ = window.emit("agent_streaming", serde_json::json!({
        "content": "",
        "agent_name": "PM Agent",
        "is_thinking": false,
        "is_first": false,
        "is_last": true
    }));
    
    // If no content was generated, send a fallback
    if result.message.is_empty() {
        let fallback = "抱歉，我没有理解你的请求。你可以尝试告诉我想做什么，比如「帮我修改代码」或「重新检查项目」。";
        let _ = window.emit("agent_streaming", serde_json::json!({
            "content": fallback,
            "agent_name": "PM Agent",
            "is_thinking": false,
            "is_first": true,
            "is_last": false
        }));
        let _ = window.emit("agent_streaming", serde_json::json!({
            "content": "",
            "agent_name": "PM Agent",
            "is_thinking": false,
            "is_first": false,
            "is_last": true
        }));
    }
    
    // Send actions if any
    if !actions.is_empty() {
        let _ = window.emit("pm_actions", serde_json::json!({
            "actions": actions
        }));
    }
    
    Ok(serde_json::json!({
        "agent_message": if result.message.is_empty() { "抱歉，我没有理解你的请求。".to_string() } else { result.message.clone() },
        "actions": actions,
        "needs_restart": false
    }))
}

// ============================================================================
// PM Restart Iteration - Restart pipeline from a specific stage
// ============================================================================

#[tauri::command]
pub async fn pm_restart_iteration(
    iteration_id: String, 
    target_stage: String,
    feedback: Option<String>,
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = IterationStore::new();
    let mut iter = store.load(&iteration_id).map_err(|e| e.to_string())?;

    iter.current_stage = Some(target_stage.clone());
    iter.status = cowork_core::domain::IterationStatus::Running;

    store.save(&iter).map_err(|e| e.to_string())?;

    // Save feedback to storage using the existing feedback mechanism
    // This allows the coding stage to read it via load_feedback_history
    if let Some(ref fb) = feedback {
        // Set iteration ID for storage operations
        cowork_core::persistence::set_iteration_id(iteration_id.clone());
        println!("[PM] Set iteration_id for storage: {}", iteration_id);
        
        // Debug: print the storage path
        if let Ok(iter_dir) = cowork_core::persistence::get_iteration_dir() {
            println!("[PM] Storage iteration dir: {}", iter_dir.display());
        }
        
        let feedback_entry = cowork_core::data::Feedback {
            stage: "pm_agent".to_string(),
            feedback_type: cowork_core::data::FeedbackType::QualityIssue,
            severity: cowork_core::data::Severity::Major,
            details: fb.clone(),
            suggested_fix: Some(format!("Restart from {} stage via PM Agent", target_stage)),
            timestamp: chrono::Utc::now(),
        };
        
        if let Err(e) = cowork_core::persistence::append_feedback(&feedback_entry) {
            eprintln!("[PM] Warning: Failed to save feedback: {}", e);
        } else {
            println!("[PM] Saved feedback to storage ({} chars): {}", fb.len(), fb.chars().take(50).collect::<String>());
        }
    } else {
        println!("[PM] No feedback provided, skipping storage save");
    }

    // Load project
    let project_store = ProjectStore::new();
    let mut project = project_store.load().map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not initialized".to_string())?;

    // Create interaction backend
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.pending_requests.clone(),
    ));

    let executor = IterationExecutor::new(interaction);

    // Emit started event
    let _ = window.emit("iteration_started", iteration_id.clone());

    // Execute in background
    let window_clone = window.app_handle().clone();
    let iteration_id_clone = iteration_id.clone();

    tokio::spawn(async move {
        println!("[PM] Starting goto_stage for iteration: {} from stage: {}", 
            iteration_id_clone, target_stage);
        // Use regular execute() - feedback is now in storage, coding stage will read it
        match executor.execute(&mut project, &iteration_id_clone, Some(target_stage), None).await {
            Ok(_) => {
                println!("[PM] goto_stage completed successfully");
                let _ = window_clone.emit("iteration_completed", iteration_id_clone);
            }
            Err(e) => {
                println!("[PM] goto_stage failed: {}", e);
                let _ = window_clone.emit("iteration_failed", (iteration_id_clone, e.to_string()));
            }
        }
    });

    Ok(())
}

// ============================================================================
// PM Get Iteration Context
// ============================================================================

#[tauri::command]
pub async fn pm_get_iteration_context(iteration_id: String) -> Result<serde_json::Value, String> {
    let store = IterationStore::new();
    let iter = store.load(&iteration_id).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "id": iter.id,
        "title": iter.title,
        "description": iter.description,
        "status": format!("{:?}", iter.status),
        "current_stage": iter.current_stage,
        "completed_stages": iter.completed_stages,
    }))
}

// ============================================================================
// PM Get Welcome Message
// ============================================================================

#[tauri::command]
pub async fn pm_get_welcome_message(iteration_id: String) -> Result<serde_json::Value, String> {
    let store = IterationStore::new();
    let iteration = store.load(&iteration_id).map_err(|e| format!("Failed to load iteration: {}", e))?;
    
    let welcome_msg = format!(
        "👋 你好！我是项目经理助手。\n\n项目 **{}** 已经完成开发！\n\n接下来你可以：",
        iteration.title
    );
    
    let actions = vec![
        serde_json::json!({ "action_type": "pm_start_app", "label": "🚀 启动应用" }),
        serde_json::json!({ "action_type": "pm_open_folder", "label": "📁 打开项目文件夹" }),
        serde_json::json!({ "action_type": "pm_view_artifacts", "label": "📄 查看设计文档" }),
        serde_json::json!({ "action_type": "pm_view_code", "label": "💻 查看源代码" }),
        serde_json::json!({ "action_type": "pm_view_knowledge", "label": "📚 查看项目知识库" }),
    ];
    
    Ok(serde_json::json!({
        "agent_message": welcome_msg,
        "actions": actions
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_stage_names() -> std::collections::HashMap<&'static str, &'static str> {
    let mut map = std::collections::HashMap::new();
    map.insert("idea", "想法阶段");
    map.insert("prd", "需求分析阶段");
    map.insert("design", "设计阶段");
    map.insert("plan", "计划阶段");
    map.insert("coding", "编码阶段");
    map.insert("check", "检查阶段");
    map.insert("delivery", "交付阶段");
    map
}
