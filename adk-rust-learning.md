# ADK-Rust 学习指南

本指南基于 deepwiki 提供的 adk-rust 实际使用方法和源码，重点介绍 tools、loop、典型 agent 模式等高级抽象的 API 和示例代码。

## 目录

1. [概述](#概述)
2. [Tools（工具）](#tools工具)
3. [Agent 模式](#agent-模式)
4. [Loop（循环）](#loop循环)
5. [会话状态管理](#会话状态管理)
6. [高级功能和配置](#高级功能和配置)
7. [错误处理和回调](#错误处理和回调)
8. [流式处理](#流式处理)

## 概述

ADK-Rust 是一个生产就绪的 Rust 实现的 Google Agent Development Kit (ADK)，用于构建高性能、内存安全的 AI 代理系统，支持流式响应、工作流编排和可扩展的工具集成。

主要特点：
- 高性能、内存安全的 AI 代理系统
- 流式响应和工作流编排
- 可扩展的工具集成
- 支持多种 agent 模式（顺序、并行、循环）

## Tools（工具）

### 1. 实现 Tool trait

最基础的创建工具方式是实现 `Tool` trait：

```rust
use adk_core::{Tool, ToolContext, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

struct WeatherTool {
    api_key: String,
}

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }
    
    fn description(&self) -> &str {
        "Get current weather for a city. Use this when the user asks about weather conditions."
    }
    
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "City name (e.g., 'London', 'New York')"
                },
                "units": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature units"
                }
            },
            "required": ["city"]
        }))
    }
    
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let city = args["city"].as_str().unwrap_or("Unknown");
        let units = args["units"].as_str().unwrap_or("celsius");
        
        // 调用天气 API...
        
        Ok(json!({
            "city": city,
            "temperature": 22,
            "units": units,
            "condition": "sunny"
        }))
    }
}
```

### 2. 使用 FunctionTool

更简单的方式是使用 `FunctionTool`，它允许您将异步函数包装为工具：

```rust
let weather_tool = FunctionTool::new(
    "get_weather",                              // 工具名称（LLM 使用）
    "Get the current weather for a city",       // 描述（帮助 LLM 决定何时使用）
    |_ctx, args| async move {                   // 处理函数
        let city = args.get("city")             // 从 JSON 中提取参数
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(json!({ "city": city, "temperature": "22°C" }))  // 返回 JSON 结果
    },
);
```

### 3. 将工具添加到 Agent

创建工具后，可以通过 `LlmAgentBuilder` 将其添加到 agent：

```rust
let agent = LlmAgentBuilder::new("weather_assistant")
    .description("A helpful assistant with weather abilities")
    .instruction("You are a helpful assistant. Use the weather tool for weather questions.")
    .model(Arc::new(model))
    .tool(Arc::new(weather_tool))
    .build()?;
```

### 4. 工具遥测

可以在工具实现中添加自定义遥测：

```rust
use adk_rust::prelude::*;
use adk_telemetry::{info, instrument, tool_execute_span};
use serde_json::{json, Value};

#[instrument(skip(ctx))]
async fn weather_tool_impl(
    ctx: Arc<dyn ToolContext>,
    args: Value,
) -> Result<Value> {
    let span = tool_execute_span("weather_tool");
    let _enter = span.enter();
    
    let location = args["location"].as_str().unwrap_or("unknown");
    info!(location = location, "Fetching weather data");
    
    // 工具逻辑
    let result = json!({
        "temperature": 72,
        "condition": "sunny"
    });
    
    info!(location = location, "Weather data retrieved");
    Ok(result)
}
```

## Agent 模式

### 1. 基础 LlmAgent

使用 `LlmAgentBuilder` 创建基础 agent：

```rust
use adk_agent::LlmAgentBuilder;
use adk_model::GeminiModel;
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GOOGLE_API_KEY")?;
    let model = GeminiModel::new(&api_key, "gemini-2.5-flash")?;
    
    let agent = LlmAgentBuilder::new("assistant")
        .description("A helpful AI assistant")
        .instruction("You are a friendly assistant. Be helpful and concise.")
        .model(Arc::new(model))
        .build()?;
    
    println!("Agent '{}' ready!", agent.name());
    Ok(())
}
```

### 2. 多工具 Agent

创建具有多个工具的 agent：

```rust
// Weather tool
let weather = FunctionTool::new(
    "get_weather",
    "Get weather for a city. Parameters: city (string)",
    |_ctx, args| async move {
        let city = args.get("city").and_then(|v| v.as_str()).unwrap_or("unknown");
        Ok(json!({
            "city": city,
            "temperature": "22°C",
            "humidity": "65%",
            "condition": "partly cloudy"
        }))
    },
);

// Calculator tool
let calc = FunctionTool::new(
    "calculate",
    "Math operations. Parameters: expression (string like '2 + 2')",
    |_ctx, args| async move {
        let expr = args.get("expression").and_then(|v| v.as_str()).unwrap_or("0");
        Ok(json!({ "expression": expr, "result": "computed" }))
    },
);

// Build the agent with tools
let agent = LlmAgentBuilder::new("assistant")
    .description("A helpful assistant with weather and calculation abilities")
    .instruction("You are a helpful assistant. Use the weather tool for weather questions. Use the calculator for math. Be concise and friendly.")
    .model(Arc::new(model))
    .tool(Arc::new(weather))
    .tool(Arc::new(calc))
    .output_key("last_response")  // 保存响应到会话状态
    .max_iterations(10)           // 限制 LLM 轮次
    .build()?;
```

### 3. 全局指令

可以使用 `global_instruction` 设置适用于所有 agents 的全局指令：

```rust
let agent = LlmAgentBuilder::new("assistant")
    .description("A helpful assistant")
    .global_instruction(
        "You are a professional assistant for Acme Corp. 
         Always maintain a friendly but professional tone. 
         Our company values are: customer-first, innovation, and integrity."
    )
    .instruction("Help users with their questions and tasks.")
    .model(model.clone())
    .build()?;
```

### 4. 顺序和并行工作流

创建多 agent 组合：

```rust
use adk_agent::{SequentialAgent, ParallelAgent, LoopAgent};
use std::sync::Arc;

// Sequential: A -> B -> C
let pipeline = SequentialAgent::new("pipeline", vec![
    agent_a.clone(),
    agent_b.clone(),
    agent_c.clone(),
]);

// Parallel: A, B, C simultaneously
let team = ParallelAgent::new("team", vec![
    analyst_a.clone(),
    analyst_b.clone(),
]);

// Loop: repeat until exit or max iterations
let iterator = LoopAgent::new("iterator", vec![worker.clone()])
    .with_max_iterations(10);
```

### 5. 研究管道示例

创建一个研究管道：研究 → 分析 → 总结：

```rust
// Step 1: Research agent gathers information
let researcher = LlmAgentBuilder::new("researcher")
    .instruction("Research the given topic. List 3-5 key facts or points. Be factual and concise.")
    .model(model.clone())
    .output_key("research")  // 保存输出到状态
    .build()?;

// Step 2: Analyzer agent identifies patterns
let analyzer = LlmAgentBuilder::new("analyzer")
    .instruction("Based on the research above, identify 2-3 key insights or patterns. What's the bigger picture?")
    .model(model.clone())
    .output_key("analysis")
    .build()?;

// Step 3: Summarizer creates final output
let summarizer = LlmAgentBuilder::new("summarizer")
    .instruction("Create a brief executive summary combining the research and analysis. Keep it under 100 words.")
    .model(model.clone())
    .build()?;

// Create the sequential pipeline
let pipeline = SequentialAgent::new(
    "research_pipeline",
    vec![Arc::new(researcher), Arc::new(analyzer), Arc::new(summarizer)],
).with_description("Research → Analyze → Summarize");
```

## Loop（循环）

### 1. LoopAgent 基本用法

`LoopAgent` 重复执行一组 agents，直到满足退出条件或达到最大迭代次数：

```rust
let loop_agent = LoopAgent::new("name", vec![agent1, agent2])
    .with_max_iterations(5)     // 安全限制（推荐）
    .with_description("Optional description")
    .before_callback(callback)
    .after_callback(callback)
```

### 2. 迭代改进循环

创建一个迭代改进循环，使用批评者和改进者不断优化内容：

```rust
// Critic agent evaluates content
let critic = LlmAgentBuilder::new("critic")
    .instruction("Review the content for quality. Score it 1-10 and list specific improvements needed. Be constructive but critical.")
    .model(model.clone())
    .build()?;

// Refiner agent improves based on critique
let refiner = LlmAgentBuilder::new("refiner")
    .instruction("Apply the critique to improve the content. If the score is 8 or higher, call exit_loop to finish. Otherwise, provide an improved version.")
    .model(model.clone())
    .tool(Arc::new(ExitLoopTool::new()))  // 可以退出循环
    .build()?;

// Create inner sequential: critic → refiner
let critique_refine = SequentialAgent::new(
    "critique_refine_step",
    vec![Arc::new(critic), Arc::new(refiner)],
);

// Wrap in loop with max 3 iterations
let iterative_improver = LoopAgent::new(
    "iterative_improver",
    vec![Arc::new(critique_refine)],
).with_max_iterations(3)
 .with_description("Critique-refine loop (max 3 passes)");
```

### 3. 复杂工作流组合

创建一个复杂工作流：并行分析 → 合成 → 质量循环：

```rust
// 1. Parallel analysis from multiple perspectives
let parallel_analysis = ParallelAgent::new(
    "multi_analysis",
    vec![Arc::new(tech_analyst), Arc::new(biz_analyst)],
);

// 2. Synthesize the parallel results
let synthesizer = LlmAgentBuilder::new("synthesizer")
    .instruction("Combine all analyses into a unified recommendation.")
    .model(model.clone())
    .build()?;

// 3. Quality loop: critique and refine
let quality_loop = LoopAgent::new(
    "quality_check",
    vec![Arc::new(critic), Arc::new(refiner)],
).with_max_iterations(2);

// Final pipeline: parallel → synthesize → quality loop
let full_pipeline = SequentialAgent::new(
    "full_analysis_pipeline",
    vec![
        Arc::new(parallel_analysis),
        Arc::new(synthesizer),
        Arc::new(quality_loop),
    ],
);
```

## 会话状态管理

### 1. 状态范围和前缀

状态键使用前缀来控制范围和持久性：

| 前缀 | 范围 | 持久性 | 使用场景 |
|------|------|--------|----------|
| `user:` | 用户级别 | 跨所有会话 | 用户偏好、设置 |
| `app:` | 应用程序级别 | 应用程序范围 | 共享配置 |
| `temp:` | 轮次级别 | 每轮清空 | 临时计算数据 |
| (无) | 会话级别 | 仅此会话 | 对话上下文 |

### 2. 状态管理示例

```rust
// In a callback or tool
let state = ctx.session().state();

// User preference (persists across sessions)
state.set("user:theme".into(), json!("dark"));

// Session-specific data
state.set("current_topic".into(), json!("weather"));

// Temporary data (cleared after this turn)
state.set("temp:step_count".into(), json!(1));

// Read values
if let Some(theme) = state.get("user:theme") {
    println!("Theme: {}", theme);
}
```

### 3. 在 Agent 中保存输出

使用 `output_key` 方法保存 agent 的响应到会话状态：

```rust
let agent = LlmAgentBuilder::new("summarizer")
    .instruction("Summarize the provided text.")
    .model(model.clone())
    .output_key("summary")  // 响应保存到 state["summary"]
    .build()?;
```

## 高级功能和配置

### 1. 限制 LLM 轮次

使用 `max_iterations` 控制 agent 可以进行的最大 LLM 调用次数：

```rust
let agent = LlmAgentBuilder::new("bounded_agent")
    .model(Arc::new(model))
    .instruction("You are a helpful assistant.")
    .tool(Arc::new(my_tool))
    .max_iterations(10)  // 10 次 LLM 调用后停止
    .build()?;
```

### 2. 动态指令提供者

使用 `instruction_provider` 根据上下文提供动态指令：

```rust
let agent = LlmAgentBuilder::new("contextual_agent")
    .model(Arc::new(model))
    .instruction_provider(Box::new(|ctx| {
        Box::pin(async move {
            // 根据上下文动态生成指令
            let user_preference = ctx.session().state()
                .get("user:communication_style")
                .and_then(|v| v.as_str())
                .unwrap_or("professional");
            
            Ok(format!("You are a helpful assistant. Communicate in a {} style.", user_preference))
        })
    }))
    .build()?;
```

### 3. 子代理注册

使用 `sub_agent` 注册子代理来处理特定任务或代理交接：

```rust
let specialized = LlmAgentBuilder::new("specialized")
    .instruction("Handle specialized tasks.")
    .model(model.clone())
    .build()?;

let coordinator = LlmAgentBuilder::new("coordinator")
    .description("Coordinates tasks and delegates when necessary")
    .instruction("You are a coordinator. Delegate specialized tasks to the specialized agent.")
    .model(model.clone())
    .sub_agent(Arc::new(specialized))
    .build()?;
```

## 错误处理和回调

### 1. 添加前后回调

添加 `before_agent` 和 `after_agent` 回调：

```rust
let agent = LlmAgentBuilder::new("my_agent")
    .model(model)
    .instruction("You are a helpful assistant.")
    // Add before_agent callback
    .before_callback(Box::new(|ctx| {
        Box::pin(async move {
            println!("Agent starting: {}", ctx.agent_name());
            Ok(None) // 继续执行
        })
    }))
    // Add after_agent callback
    .after_callback(Box::new(|ctx| {
        Box::pin(async move {
            println!("Agent completed: {}", ctx.agent_name());
            Ok(None) // 保持原始结果
        })
    }))
    .build()?;
```

### 2. 错误处理

使用回调进行验证和错误处理：

```rust
let agent = LlmAgentBuilder::new("error_handling_agent")
    .model(model)
    .before_callback(Box::new(|ctx| {
        Box::pin(async move {
            // 验证关键条件
            if ctx.user_id().is_empty() {
                return Err(AdkError::Agent("User ID is required".to_string()));
            }
            Ok(None)
        })
    }))
    .build()?;
```

### 3. 工具回调

添加 `before_tool` 和 `after_tool` 回调：

```rust
let agent = LlmAgentBuilder::new("tool_monitoring_agent")
    .model(model)
    .before_tool_callback(Box::new(|ctx, tool_name| {
        Box::pin(async move {
            println!("Starting tool: {}", tool_name);
            Ok(())
        })
    }))
    .after_tool_callback(Box::new(|ctx, tool_name, result| {
        Box::pin(async move {
            match result {
                Ok(_) => println!("Tool {} completed successfully", tool_name),
                Err(e) => println!("Tool {} failed: {}", tool_name, e),
            }
            Ok(())
        })
    }))
    .build()?;
```

## 流式处理

### 1. 使用 SSE 运行 Agent

使用 Server-Sent Events (SSE) 流式执行 agent：

```http
POST /api/run_sse

{
  "appName": "my_agent",
  "userId": "user123",
  "sessionId": "session456",
  "newMessage": {
    "role": "user",
    "parts": [
      {
        "text": "What is the capital of France?"
      }
    ]
  },
  "streaming": true
}
```

### 2. SSE 响应示例

```json
{
  "id": "evt_123",
  "timestamp": 1234567890,
  "author": "my_agent",
  "content": {
    "role": "model",
    "parts": [
      {
        "text": "The capital of France is Paris."
      }
    ]
  },
  "actions": {},
  "llm_response": {
    "content": {
      "role": "model",
      "parts": [
        {
          "text": "The capital of France is Paris."
        }
      ]
    }
  }
}
```

### 3. 使用 Python 请求客户端

```python
import requests
import json

def run_agent(message):
    url = 'http://localhost:8080/api/run_sse'
    payload = {
        'appName': 'my_agent',
        'userId': 'user123',
        'sessionId': 'session456',
        'newMessage': {
            'role': 'user',
            'parts': [{'text': message}]
        },
        'streaming': True
    }
    
    response = requests.post(url, json=payload, stream=True)
    
    for line in response.iter_lines():
        if line:
            line_str = line.decode('utf-8')
            if line_str.startswith('data: '):
                event = json.loads(line_str[6:])
                print('Event:', event)

run_agent('What is the capital of France?')
```

### 4. 使用 JavaScript/TypeScript Fetch API

```javascript
async function runAgent(message) {
  const response = await fetch('http://localhost:8080/api/run_sse', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      appName: 'my_agent',
      userId: 'user123',
      sessionId: 'session456',
      newMessage: {
        role: 'user',
        parts: [{ text: message }]
      },
      streaming: true
    })
  });

  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    
    const chunk = decoder.decode(value);
    const lines = chunk.split('\n');
    
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const event = JSON.parse(line.slice(6));
        console.log('Event:', event);
      }
    }
  }
}
```

## 最佳实践

1. **工具设计**:
   - 提供清晰的名称和描述，帮助 LLM 理解何时使用工具
   - 定义明确的 JSON 模式和必需参数
   - 在工具实现中添加遥测和日志记录

2. **Agent 构建**:
   - 使用 `max_iterations` 设置合理的迭代限制，防止无限循环
   - 提供清晰具体的指令，帮助 agent 理解何时调用 `exit_loop`
   - 使用 `output_key` 保存关键的中间结果到会话状态

3. **工作流组合**:
   - 使用 `SequentialAgent` 构建线性处理流程
   - 使用 `ParallelAgent` 执行并发分析任务
   - 使用 `LoopAgent` 实现迭代改进或质量控制循环

4. **状态管理**:
   - 使用适当的状态前缀（`user:`、`app:`、`temp:`）管理不同范围的持久性
   - 在状态间传递关键信息和中间结果
   - 定期清理不再需要的临时数据

5. **错误处理**:
   - 在关键路径上使用前后回调进行验证
   - 实现适当的错误恢复机制
   - 添加足够的监控和日志记录

ADK-Rust 提供了一个强大且灵活的框架，可用于构建各种复杂的 AI 代理系统，从简单的任务助手到多阶段的工作流程引擎。