use crate::AppState;
use crate::TauriBackend;
use cowork_core::persistence::IterationStore;
use cowork_core::pipeline::IterationExecutor;
use cowork_core::persistence::ProjectStore;
use cowork_core::llm::{load_config, create_llm_client};
use std::sync::Arc;
use tauri::{Emitter, Manager, State, Window};
use adk_core::{Content, Part, LlmRequest};
use std::collections::HashMap;

const PM_AGENT_SYSTEM: &str = r#"
你是一个项目经理助手，帮助用户管理迭代项目。

## 你的能力

你可以使用以下工具：

### goto_stage
跳转到指定阶段重新执行项目。

**参数**:
- target_stage: 目标阶段名称

**可选值**: idea, prd, design, plan, coding, check, delivery

**使用场景**:
- 用户要求修改代码 → target_stage: "coding"
- 用户要求重新检查项目 → target_stage: "check"
- 用户要求修改设计 → target_stage: "design"
- 用户要求修改计划 → target_stage: "plan"
- 用户要求修改需求 → target_stage: "prd"

### create_iteration
创建新的迭代来实现新功能或进行重大变更。

**参数**:
- title: 新迭代的标题（简洁概括）
- description: 新迭代的详细描述
- inheritance: 继承模式（可选，默认 "partial"）
  - "partial": 继承代码，重新生成文档（推荐用于添加新功能）
  - "full": 继承所有代码和文档
  - "none": 完全从头开始

**使用场景**:
- 用户想要添加全新的功能模块
- 用户想要做重大架构变更
- 用户明确表示要"新做一个"或"创建新项目"

### 示例对话

用户: "帮我修改一下代码"
助手: [调用 goto_stage 工具，参数 target_stage="coding"]

用户: "我想重新检查一下项目"
助手: [调用 goto_stage 工具，参数 target_stage="check"]

用户: "我想添加一个用户登录功能"
助手: [调用 create_iteration 工具，参数 title="用户登录功能", description="实现用户登录认证系统，包括登录页面和后端验证"]

用户: "项目标题是什么？"
助手: 直接回答用户问题，不需要调用工具

## 规则

1. 当用户要求修改现有功能时，使用 goto_stage 工具
2. 当用户要求添加新功能或做大改动时，使用 create_iteration 工具
3. 当用户只是提问或讨论时，直接回答，不调用工具
4. 始终用中文回复
5. 调用工具前不需要询问确认，系统会自动处理
"#;

fn get_tools() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "goto_stage",
            "description": "跳转到指定阶段重新执行项目。当用户要求修改代码、重新检查、重新设计等时使用此工具。",
            "parameters": {
                "type": "object",
                "properties": {
                    "target_stage": {
                        "type": "string",
                        "enum": ["idea", "prd", "design", "plan", "coding", "check", "delivery"],
                        "description": "目标阶段名称。coding=修改代码, check=重新检查, design=修改设计, plan=修改计划, prd=修改需求, idea=修改想法, delivery=重新交付"
                    }
                },
                "required": ["target_stage"]
            }
        }),
        serde_json::json!({
            "name": "create_iteration",
            "description": "创建新的迭代来实现新功能或进行重大变更。当用户想要添加新功能模块或做重大改动时使用此工具。",
            "parameters": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "新迭代的标题，简洁概括用户需求"
                    },
                    "description": {
                        "type": "string",
                        "description": "新迭代的详细描述，包括用户想要实现的具体功能"
                    },
                    "inheritance": {
                        "type": "string",
                        "enum": ["none", "partial", "full"],
                        "description": "继承模式：partial=继承代码重新生成文档（推荐），full=继承所有，none=完全从头开始"
                    }
                },
                "required": ["title", "description"]
            }
        })
    ]
}

#[tauri::command]
pub async fn pm_send_message(
    iteration_id: String,
    message: String,
    history: Vec<serde_json::Value>,
    window: Window,
) -> Result<serde_json::Value, String> {
    let store = IterationStore::new();
    let iteration = store.load(&iteration_id).map_err(|e| format!("Failed to load iteration: {}", e))?;
    
    let is_first_message = history.is_empty();
    
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
    
    let config = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let client = create_llm_client(&config.llm).map_err(|e| format!("Failed to create LLM client: {}", e))?;
    
    // Build conversation
    let mut contents = vec![Content {
        role: "user".to_string(),
        parts: vec![Part::Text { 
            text: format!(
                "{}\n\n当前迭代信息:\n- ID: {}\n- 标题: {}\n- 描述: {}\n- 状态: {:?}\n- 当前阶段: {:?}",
                PM_AGENT_SYSTEM,
                iteration_id, 
                iteration.title, 
                iteration.description,
                iteration.status,
                iteration.current_stage
            )
        }],
    }];

    // Add history
    for h in history {
        if let Some(role) = h.get("type").and_then(|t| t.as_str()) {
            let content = h.get("content").and_then(|c| c.as_str()).unwrap_or("");
            let content = if role == "user" {
                Content {
                    role: "user".to_string(),
                    parts: vec![Part::Text { text: content.to_string() }],
                }
            } else {
                Content {
                    role: "model".to_string(),
                    parts: vec![Part::Text { text: content.to_string() }],
                }
            };
            contents.push(content);
        }
    }

    // Add current message
    contents.push(Content {
        role: "user".to_string(),
        parts: vec![Part::Text { text: message.clone() }],
    });

    // Create request with tools
    let tools_map: HashMap<String, serde_json::Value> = vec![(
        "tools".to_string(),
        serde_json::json!(get_tools())
    )].into_iter().collect();

    let req = LlmRequest {
        model: config.llm.model_name.clone(),
        contents,
        config: None,
        tools: tools_map,
    };

    let mut stream = client.generate_content(req, true).await.map_err(|e| format!("Failed to generate content: {}", e))?;
    
    use futures::StreamExt;
    
    let mut response_text = String::new();
    let mut function_calls: Vec<(String, serde_json::Value)> = Vec::new();
    let mut is_first_chunk = true;
    
    // Stream LLM response in real-time, similar to ChatGPT
    while let Some(chunk) = stream.next().await {
        if let Ok(r) = chunk {
            if let Some(c) = r.content {
                for p in c.parts.iter() {
                    match p {
                        Part::Text { text } => {
                            response_text.push_str(text);
                            // Emit each chunk in real-time for streaming display
                            let _ = window.emit("agent_streaming", serde_json::json!({
                                "content": text,
                                "agent_name": "PM Agent",
                                "is_thinking": false,
                                "is_first": is_first_chunk,
                                "is_last": false
                            }));
                            is_first_chunk = false;
                        }
                        Part::FunctionCall { name, args, .. } => {
                            function_calls.push((name.clone(), args.clone()));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    // Send streaming end signal
    if !response_text.is_empty() || !is_first_chunk {
        let _ = window.emit("agent_streaming", serde_json::json!({
            "content": "",
            "agent_name": "PM Agent",
            "is_thinking": false,
            "is_first": false,
            "is_last": true
        }));
    }

    // Process function calls - these add actions to the streaming message
    for (func_name, args) in &function_calls {
        if func_name == "goto_stage" {
            if let Some(target) = args.get("target_stage").and_then(|t| t.as_str()) {
                let stage_names = {
                    let mut map = HashMap::new();
                    map.insert("idea", "想法阶段");
                    map.insert("prd", "需求分析阶段");
                    map.insert("design", "设计阶段");
                    map.insert("plan", "计划阶段");
                    map.insert("coding", "编码阶段");
                    map.insert("check", "检查阶段");
                    map.insert("delivery", "交付阶段");
                    map
                };
                
                let stage_name = stage_names.get(target).unwrap_or(&target);
                
                // Append action prompt to streaming message
                let action_prompt = format!("\n\n点击下方按钮确认跳转到 **{}**：", stage_name);
                let _ = window.emit("agent_streaming", serde_json::json!({
                    "content": action_prompt,
                    "agent_name": "PM Agent",
                    "is_thinking": false,
                    "is_first": false,
                    "is_last": false
                }));
                
                // Send actions to be appended
                let _ = window.emit("pm_actions", serde_json::json!({
                    "actions": [{ 
                        "action_type": "pm_goto_stage", 
                        "target_stage": target, 
                        "label": format!("🔄 跳转到 {}", stage_name) 
                    }]
                }));
                
                // Send stream end signal
                let _ = window.emit("agent_streaming", serde_json::json!({
                    "content": "",
                    "agent_name": "PM Agent",
                    "is_thinking": false,
                    "is_first": false,
                    "is_last": true
                }));
                
                return Ok(serde_json::json!({
                    "agent_message": response_text,
                    "actions": [{ 
                        "action_type": "pm_goto_stage", 
                        "target_stage": target, 
                        "label": format!("🔄 跳转到 {}", stage_name) 
                    }],
                    "needs_restart": false
                }));
            }
        }
        
        if func_name == "create_iteration" {
            if let (Some(title), Some(description)) = (
                args.get("title").and_then(|t| t.as_str()),
                args.get("description").and_then(|d| d.as_str())
            ) {
                let inheritance = args.get("inheritance")
                    .and_then(|i| i.as_str())
                    .unwrap_or("partial");
                
                // Create the new iteration
                let project_store = ProjectStore::new();
                let project = project_store.load().map_err(|e| format!("Failed to load project: {}", e))?
                    .ok_or_else(|| "Project not initialized".to_string())?;
                
                let inheritance_mode = match inheritance {
                    "none" => cowork_core::domain::InheritanceMode::None,
                    "full" => cowork_core::domain::InheritanceMode::Full,
                    _ => cowork_core::domain::InheritanceMode::Partial,
                };
                
                let new_iteration = cowork_core::domain::Iteration::create_evolution(
                    &project,
                    title.to_string(),
                    description.to_string(),
                    iteration_id.clone(),
                    inheritance_mode,
                );
                
                let new_iteration_id = new_iteration.id.clone();
                let new_iteration_title = new_iteration.title.clone();
                
                let iteration_store = IterationStore::new();
                iteration_store.save(&new_iteration).map_err(|e| format!("Failed to save iteration: {}", e))?;
                
                // Update project
                let mut project = project;
                project_store.add_iteration(&mut project, new_iteration.to_summary())
                    .map_err(|e| format!("Failed to update project: {}", e))?;
                
                // Emit iteration_created event to notify frontend
                let _ = window.emit("iteration_created", &new_iteration_id);
                
                // Append action prompt to streaming message
                let action_prompt = format!("\n\n我已经创建了新迭代 **{}**。\n\n点击下方按钮启动新迭代：", title);
                let _ = window.emit("agent_streaming", serde_json::json!({
                    "content": action_prompt,
                    "agent_name": "PM Agent",
                    "is_thinking": false,
                    "is_first": false,
                    "is_last": false
                }));
                
                // Send actions to be appended
                let _ = window.emit("pm_actions", serde_json::json!({
                    "actions": [{ 
                        "action_type": "pm_create_iteration", 
                        "iteration_id": new_iteration_id,
                        "title": new_iteration_title,
                        "label": "🚀 启动新迭代" 
                    }]
                }));
                
                // Send stream end signal
                let _ = window.emit("agent_streaming", serde_json::json!({
                    "content": "",
                    "agent_name": "PM Agent",
                    "is_thinking": false,
                    "is_first": false,
                    "is_last": true
                }));
                
                return Ok(serde_json::json!({
                    "agent_message": response_text,
                    "actions": [{ 
                        "action_type": "pm_create_iteration", 
                        "iteration_id": new_iteration_id,
                        "title": new_iteration_title,
                        "label": "🚀 启动新迭代" 
                    }],
                    "needs_restart": false
                }));
            }
        }
    }

    // No function call - just signal stream complete
    // The content was already streamed via agent_streaming events
    if response_text.is_empty() { 
        // Send a fallback message if no content was streamed
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
    
    Ok(serde_json::json!({
        "agent_message": if response_text.is_empty() { "抱歉，我没有理解你的请求。".to_string() } else { response_text.clone() },
        "actions": [],
        "needs_restart": false
    }))
}

#[tauri::command]
pub async fn pm_restart_iteration(
    iteration_id: String, 
    target_stage: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = IterationStore::new();
    let mut iter = store.load(&iteration_id).map_err(|e| e.to_string())?;

    iter.current_stage = Some(target_stage.clone());
    iter.status = cowork_core::domain::IterationStatus::Running;

    store.save(&iter).map_err(|e| e.to_string())?;

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
        println!("[PM] Starting goto_stage for iteration: {} from stage: {}", iteration_id_clone, target_stage);
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
