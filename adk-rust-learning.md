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
    
    let location = args.get("location").and_then(|v| v.as_str()).unwrap_or("unknown");
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

### 5. 安全参数提取（Cowork Forge 最佳实践）

#### 问题：unwrap() 导致 panic

传统的工具实现使用 `unwrap()` 提取参数，当参数缺失时会导致整个应用崩溃：

```rust
// ❌ 危险：使用 unwrap()
async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
    let title = args["title"].as_str().unwrap();  // 如果参数缺失，panic！
    let content = args["content"].as_str().unwrap();  // 如果参数缺失，panic！
    // ...
}
```

**运行时错误**：
```
thread 'tokio-runtime-worker' panicked at crates\cowork-core\src\tools\hitl_content_tools.rs:156:48:
called `Option::unwrap()` on a `None` value
```

#### 解决方案：安全的参数提取函数

提供辅助函数来安全地提取参数：

```rust
// tools/mod.rs

/// 安全提取必需的字符串参数
pub fn get_required_string_param<'a>(
    args: &'a Value,
    key: &str,
) -> Result<&'a str, AdkError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| AdkError::Tool(format!("Missing required parameter: {}", key)))
}

/// 安全提取可选的字符串参数
pub fn get_optional_string_param(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// 安全提取必需的数组参数
pub fn get_required_array_param<'a>(
    args: &'a Value,
    key: &str,
) -> Result<&'a Vec<Value>, AdkError> {
    args.get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| AdkError::Tool(format!("Missing required parameter: {}", key)))
}
```

#### 正确的工具实现

```rust
// ✅ 安全：使用参数提取函数
pub struct SaveIdeaTool;

#[async_trait]
impl Tool for SaveIdeaTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // 安全提取必需参数
        let content = get_required_string_param(&args, "content")?;

        save_idea(content)
            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "message": "Idea document saved successfully",
            "file_path": "artifacts/idea.md"
        }))
    }
}
```

**运行时错误**（当参数缺失时）：
```
Error: Missing required parameter: content
```

#### 参数提取函数对比

| 函数 | 用途 | 缺失时行为 | 返回类型 |
|------|------|------------|----------|
| `get_required_string_param` | 提取必需字符串 | 返回错误 | `Result<&str, AdkError>` |
| `get_optional_string_param` | 提取可选字符串 | 返回 `None` | `Option<String>` |
| `get_required_array_param` | 提取必需数组 | 返回错误 | `Result<&Vec<Value>, AdkError>` |
| `get_optional_array_param` | 提取可选数组 | 返回空数组 | `Vec<Value>` |

#### 实际应用：修复 unwrap() 统计

在 Cowork Forge 项目中，我们修复了 33 个危险的 unwrap() 调用：

| 文件 | 修复数量 | 示例 |
|------|----------|------|
| hitl_content_tools.rs | 2 | `title`, `content` |
| validation_tools.rs | 1 | `data_type` |
| artifact_tools.rs | 5 | `content` (多个 Save 工具) |
| goto_stage_tool.rs | 2 | `stage`, `reason` |
| file_tools.rs | 3 | `path`, `content`, `command` |
| hitl_tools.rs | 4 | `file_path`, `title`, `path` |
| control_tools.rs | 5 | `feedback_type`, `severity`, `details` |
| data_tools.rs | 11 | `priority`, `category`, `title`, `description`, `feature_id`, `component_id` |

**结果**：
- ✅ 从 39 个潜在 panic 点减少到 6 个相对安全的点
- ✅ 所有缺失参数现在返回清晰的错误信息
- ✅ 系统稳定性显著提升

#### 工具实现检查清单

实现新工具时，请检查以下项目：

```rust
pub struct MyNewTool;

#[async_trait]
impl Tool for MyNewTool {
    fn name(&self) -> &str {
        "my_new_tool"
    }

    fn description(&self) -> &str {
        "Describe what this tool does in detail."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "required_param": {
                    "type": "string",
                    "description": "Description of required parameter"
                },
                "optional_param": {
                    "type": "string",
                    "description": "Description of optional parameter"
                }
            },
            "required": ["required_param"]  // 声明必需参数
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // ✅ 使用安全的参数提取
        let required = get_required_string_param(&args, "required_param")?;
        
        // ✅ 使用安全的可选参数提取
        let optional = get_optional_string_param(&args, "optional_param");
        
        // ✅ 处理逻辑
        // ...
        
        // ✅ 返回结构化结果
        Ok(json!({
            "status": "success",
            "result": "..."
        }))
    }
}
```

**检查清单**：
- [ ] 必需参数使用 `get_required_string_param` 或 `get_required_array_param`
- [ ] 可选参数使用 `get_optional_string_param` 或 `get_optional_array_param`
- [ ] 不使用 `unwrap()` 提取参数
- [ ] `parameters_schema` 中声明所有必需参数
- [ ] 返回清晰的错误信息（不要返回空错误）
- [ ] 返回结构化的 JSON 结果

#### 生命周期标注注意事项

当引用 `Value` 中的字符串时，需要添加生命周期标注：

```rust
// ❌ 错误：缺少生命周期标注
pub fn get_required_string_param(args: &Value, key: &str) -> Result<&str, AdkError> {
    // 编译错误：missing lifetime specifier
}

// ✅ 正确：添加生命周期标注
pub fn get_required_string_param<'a>(
    args: &'a Value,
    key: &str,
) -> Result<&'a str, AdkError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| AdkError::Tool(format!("Missing required parameter: {}", key)))
}
```

生命周期 `'a` 告诉编译器：返回的字符串引用的生命周期与输入 `args` 相同。

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

### 2. Actor-Critic Loop 模式（Cowork Forge 核心模式）

使用 LoopAgent 实现经典的 Actor-Critic 协作模式：

```rust
// 创建 Actor - 负责创建内容
let actor = LlmAgentBuilder::new("prd_actor")
    .instruction(PRD_ACTOR_INSTRUCTION)
    .model(model.clone())
    .tool(Arc::new(CreateRequirementTool))
    .tool(Arc::new(AddFeatureTool))
    .tool(Arc::new(GetRequirementsTool))
    .build()?;

// 创建 Critic - 负责审查质量
let critic = LlmAgentBuilder::new("prd_critic")
    .instruction(PRD_CRITIC_INSTRUCTION)
    .model(model)
    .tool(Arc::new(GetRequirementsTool))
    .tool(Arc::new(CheckFeatureCoverageTool))
    .build()?;

// 创建 LoopAgent - Actor 和 Critic 交替执行
let mut loop_agent = LoopAgent::new(
    "prd_loop",
    vec![Arc::new(actor), Arc::new(critic)],
);
// 使用 max_iterations=1 避免过度优化，保持节奏
loop_agent = loop_agent.with_max_iterations(1);

Ok(Arc::new(loop_agent))
```

**设计理念**：
- Actor：创建内容（需求、设计、计划、代码）
- Critic：审查质量、发现问题
- max_iterations=1：避免过度优化，让流程继续
- 每个阶段独立迭代，而非无限循环

### 3. 多工具 Agent

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

### 3. 顺序和并行工作流

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

### 4. 并行 Agent 的潜在应用

虽然 Cowork Forge 当前使用顺序执行，但可以考虑在以下场景使用 ParallelAgent：

**场景 1：并行分析**
```rust
// 从不同角度并行分析需求
let technical_analyst = create_agent("technical", ...);
let business_analyst = create_agent("business", ...);
let ux_analyst = create_agent("ux", ...);

let parallel_analysis = ParallelAgent::new(
    "parallel_requirements_analysis",
    vec![
        Arc::new(technical_analyst),
        Arc::new(business_analyst),
        Arc::new(ux_analyst),
    ],
);

let synthesizer = create_agent("synthesizer", ...);
let pipeline = SequentialAgent::new(
    "full_requirements_flow",
    vec![Arc::new(parallel_analysis), Arc::new(synthesizer)],
);
```

**场景 2：并行测试**
```rust
// 并行运行多种测试
let unit_tests = create_agent("unit_tests", ...);
let integration_tests = create_agent("integration_tests", ...);
let e2e_tests = create_agent("e2e_tests", ...);

let parallel_testing = ParallelAgent::new(
    "parallel_test_suite",
    vec![Arc::new(unit_tests), Arc::new(integration_tests), Arc::new(e2e_tests)],
);
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

## ⚠️ 重要：LoopAgent 与 SequentialAgent 的交互问题

### 问题
当 LoopAgent 作为 SequentialAgent 的子节点时，如果 LoopAgent 中的任何 agent 调用 `exit_loop()` 工具，**整个 SequentialAgent 都会终止**，而不仅仅是 LoopAgent。这是 adk-rust 的设计限制。

### 解决方案
不要使用 `exit_loop()` 工具。改用 `max_iterations` 控制循环：

```rust
// ❌ 错误：使用 exit_loop 会导致整个 pipeline 终止
let refiner = LlmAgentBuilder::new("refiner")
    .tool(Arc::new(ExitLoopTool::new()))
    .build()?;

// ✅ 正确：使用 max_iterations 控制循环
let loop_agent = LoopAgent::new("loop", vec![critic, refiner])
    .with_max_iterations(1);  // 让 LoopAgent 自然完成
```

### 适用场景
- **适用**：当 LoopAgent 是 SequentialAgent 的一部分时
- **适用**：当 LoopAgent 与其他 agent 需要顺序执行时
- **不适用**：如果 LoopAgent 是顶层 agent（不嵌套在 SequentialAgent 中），可以安全使用 exit_loop

### Cowork Forge 的实践
在 Cowork Forge 中，所有 LoopAgent 都使用 `max_iterations=1` 避免这个问题：

```rust
// PRD Loop - 只迭代一次
let mut loop_agent = LoopAgent::new("prd_loop", vec![prd_actor, prd_critic]);
loop_agent = loop_agent.with_max_iterations(1);

// Coding Loop - 更多迭代，但仍然不使用 exit_loop
let mut loop_agent = LoopAgent::new("coding_loop", vec![coding_actor, coding_critic]);
loop_agent = loop_agent.with_max_iterations(5);
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

## 数据传递模式对比

### 模式 1：工具主动获取（Cowork Forge 使用）

**方式**：通过工具获取结构化数据

```rust
// Agent 通过工具获取数据
let critic = LlmAgentBuilder::new("critic")
    .instruction("Review the requirements and design...")
    .tool(Arc::new(GetRequirementsTool))  // 主动获取需求
    .tool(Arc::new(GetDesignTool))      // 主动获取设计
    .tool(Arc::new(CheckFeatureCoverageTool))  // 验证覆盖范围
    .build()?;
```

**优点**：
- 数据结构化、类型安全
- 数据可追溯、可审计
- 支持跨 session 的数据访问

**缺点**：
- 需要额外的工具调用开销
- 需要定义数据结构

### 模式 2：output_key（状态传递）

**方式**：通过 output_key 保存输出，后续 agent 从状态读取

```rust
// Research agent 保存输出
let researcher = LlmAgentBuilder::new("researcher")
    .instruction("Research the given topic...")
    .model(model.clone())
    .output_key("research")  // 保存到 state["research"]
    .build()?;

// Analyzer agent 从状态读取
let analyzer = LlmAgentBuilder::new("analyzer")
    .instruction("Based on the research in state['research'], analyze...")
    .model(model.clone())
    .build()?;
```

**优点**：
- 简单直接
- 自动传递
- 无需额外工具

**缺点**：
- 数据非结构化（纯文本）
- 依赖 LLM 解析
- 难以追踪数据来源

**选择建议**：
- **结构化数据**（需求、设计、计划）→ 使用工具模式
- **简单文本内容**（摘要、反馈）→ 使用 output_key 模式

Cowork Forge 采用工具模式，因为需要：
1. 数据独立于对话历史
2. 支持跨 session 的数据访问
3. 结构化的数据验证和管理

## HITL（Human-in-the-Loop）工具

### 概念
HITL 工具允许 agent 在执行过程中请求人类反馈或编辑内容，实现"人在回路"的质量保证。

### Cowork Forge 的 HITL 工具演进

#### 两种 HITL 工具类型

**1. Content-based HITL 工具（推荐）**

基于内容的交互，不暴露文件路径，更安全、更灵活：

```rust
// ReviewAndEditContentTool - 让用户审查和编辑内容
let idea_agent = LlmAgentBuilder::new("idea_agent")
    .instruction(IDEA_AGENT_INSTRUCTION)
    .tool(Arc::new(SaveIdeaTool))
    .tool(Arc::new(ReviewAndEditContentTool))  // Content-based HITL
    .build()?;

// ReviewWithFeedbackContentTool - 支持三种反馈模式
let prd_actor = LlmAgentBuilder::new("prd_actor")
    .instruction(PRD_ACTOR_INSTRUCTION)
    .tool(Arc::new(SavePrdDocTool))
    .tool(Arc::new(ReviewWithFeedbackContentTool))  // Content-based HITL
    .tool(Arc::new(CreateRequirementTool))
    .build()?;
```

**三种反馈模式**：

1. **Edit 模式** - 输入 "edit" 或粘贴多行内容
   - 用户直接提供编辑后的内容
   - Agent 使用编辑后的内容

2. **Pass 模式** - 输入 "pass" 或选择"通过"
   - 跳过当前阶段
   - 使用原始内容

3. **Feedback 模式** - 输入文本建议
   - 提供具体的修改建议
   - Agent 根据建议修订内容

**使用流程示例**：
```
Agent: 生成需求大纲内容 → 调用 review_with_feedback_content(title="Review PRD", content=<draft>)
User: "需求太多，减少到5个核心需求"
Agent: 识别 action="feedback" → 修订内容 → 再次调用 review_with_feedback_content
User: "pass"
Agent: 使用修订后的内容创建正式需求 → 调用 save_prd_doc(content=<final_prd>)
```

**优点**：
- 安全：不暴露文件路径
- 灵活：可以在内存中操作，无需实际文件
- 解耦：Agent 只关心内容，不关心存储位置

**2. File-based HITL 工具（已废弃）**

基于文件的交互，需要文件路径：

```rust
// 已废弃 - 不推荐使用
let idea_agent = LlmAgentBuilder::new("idea_agent")
    .instruction(IDEA_AGENT_INSTRUCTION)
    .tool(Arc::new(WriteFileTool))
    .tool(Arc::new(ReviewAndEditFileTool))  // File-based（已废弃）
    .build()?;
```

**问题**：
- 需要通用的 WriteFileTool 权限
- 暴露文件路径给 Agent
- 违反权限最小化原则

#### Cowork Forge 的最佳实践

**Idea 阶段**：
```rust
// ✅ 正确：使用 Content-based HITL
let idea_agent = LlmAgentBuilder::new("idea_agent")
    .tool(Arc::new(SaveIdeaTool))
    .tool(Arc::new(ReviewAndEditContentTool))
    .build()?;

// ❌ 错误：使用 File-based HITL
let idea_agent = LlmAgentBuilder::new("idea_agent")
    .tool(Arc::new(WriteFileTool))
    .tool(Arc::new(ReviewAndEditFileTool))
    .build()?;
```

**PRD/Design/Plan 阶段**：
```rust
// ✅ 正确：使用 Content-based HITL
let prd_actor = LlmAgentBuilder::new("prd_actor")
    .tool(Arc::new(SavePrdDocTool))
    .tool(Arc::new(ReviewWithFeedbackContentTool))
    .tool(Arc::new(CreateRequirementTool))
    .build()?;

// ❌ 错误：使用 File-based HITL + WriteFileTool
let prd_actor = LlmAgentBuilder::new("prd_actor")
    .tool(Arc::new(WriteFileTool))  // 不应该有通用写权限
    .tool(Arc::new(ReviewWithFeedbackTool))
    .build()?;
```

**Coding 阶段**：
```rust
// Coding 阶段需要 ReadFileTool/WriteFileTool，但不使用 HITL
let coding_actor = LlmAgentBuilder::new("coding_actor")
    .tool(Arc::new(ReadFileTool))    // 需要：读取代码文件
    .tool(Arc::new(WriteFileTool))   // 需要：写入代码文件
    .tool(Arc::new(ListFilesTool))
    .tool(Arc::new(RunCommandTool))
    // 不使用 HITL 工具
    .build()?;
```

### HITL 的最佳实践

1. **时机选择**：在关键决策点使用 HITL
   - Idea 阶段：确认项目方向
   - PRD 阶段：审查需求完整性
   - Design 阶段：审查架构合理性
   - Plan 阶段：审查任务可行性

2. **频率控制**：避免过度打扰用户
   - 每个阶段最多 1-2 次 HITL 交互
   - Critic 评审后不再 HITL

3. **上下文提供**：给用户充分的信息
   - 预览内容（前 15 行）
   - 清晰的操作提示
   - 已知问题列表

4. **安全原则**：
   - 非编码阶段：使用 Content-based HITL
   - 不暴露文件路径
   - 使用专用的 Save 工具

## Event 流处理

### Event 流结构

`Agent::run()` 返回的是 `Stream<Item = Result<Event>>`：

```rust
let mut stream = agent.run(invocation_ctx).await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(event) => {
            // 处理不同类型的事件
        }
        Err(e) => {
            return Err(format!("流错误: {}", e));
        }
    }
}
```

### Event 类型处理

```rust
match &event {
    Event::Content(content) => {
        // 提取文本内容
        for part in &content.parts {
            if let Some(text) = part.text() {
                generated_text.push_str(text);
            }
        }
    }
    Event::ToolCall(call) => {
        println!("工具调用: {}({:?})", call.name, call.args);
    }
    Event::ToolResult(result) => {
        println!("工具结果: {:?}", result);
    }
    Event::Error(e) => {
        println!("错误: {}", e);
    }
}
```

### 完整示例

```rust
async fn execute_agent_with_context(
    agent: Arc<dyn Agent>,
    ctx: Arc<dyn InvocationContext>,
) -> Result<String> {
    let mut stream = agent.run(ctx).await?;
    let mut generated_text = String::new();
    let mut tool_calls = Vec::new();
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                match &event {
                    Event::Content(content) => {
                        for part in &content.parts {
                            if let Some(text) = part.text() {
                                generated_text.push_str(text);
                            }
                        }
                    }
                    Event::ToolCall(call) => {
                        tool_calls.push(call.clone());
                        println!("工具调用: {}", call.name);
                    }
                    Event::ToolResult(result) => {
                        println!("工具结果: {:?}", result);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                return Err(format!("流错误: {}", e));
            }
        }
    }
    
    Ok(generated_text)
}
```

## IncludeContents 控制上下文包含

### 概念
`IncludeContents` 控制 agent 执行时包含哪些历史对话内容，影响 token 消耗和上下文质量。

### 选项说明

| 值 | 说明 | 适用场景 | Token 消耗 |
|---|---|---|---|
| `IncludeContents::None` | 不包含历史 | 每次都是全新的上下文 | 最少 |
| `IncludeContents::LastN(n)` | 包含最近 n 条 | 需要短期上下文时 | 中等 |
| `IncludeContents::All` | 包含所有历史 | 需要完整对话历史 | 最多 |

### Cowork Forge 的选择

```rust
let agent = LlmAgentBuilder::new("prd_actor")
    .instruction(PRD_ACTOR_INSTRUCTION)
    .model(model)
    .tool(Arc::new(GetRequirementsTool))
    .include_contents(IncludeContents::None)  // 不包含历史
    .build()?;
```

**选择 IncludeContents::None 的原因**：
1. **独立任务**：每个阶段都是独立的任务（需求、设计、计划等）
2. **工具获取数据**：通过工具（GetRequirementsTool 等）获取数据，而非依赖对话历史
3. **节省 token**：不传递历史对话，显著降低 token 消耗
4. **可预测性**：每次执行都是干净的上下文，结果更可预测

### 何时使用 IncludeContents::All

```rust
// 需要完整对话历史的场景
let chat_agent = LlmAgentBuilder::new("chat_agent")
    .instruction("Continue our conversation...")
    .model(model)
    .include_contents(IncludeContents::All)  // 包含所有历史
    .build()?;
```

适用场景：
- 对话式 agent（需要记住之前的对话）
- 需要跨轮次的上下文积累
- 需要引用之前的回复

### 何时使用 IncludeContents::LastN

```rust
// 需要短期上下文的场景
let refinement_agent = LlmAgentBuilder::new("refinement_agent")
    .instruction("Refine based on recent discussion...")
    .model(model)
    .include_contents(IncludeContents::LastN(3))  // 最近 3 条
    .build()?;
```

适用场景：
- 需要短期上下文但不需要完整历史
- 节省 token 的同时保持一定的上下文
- 多轮对话的中间阶段

## 结构化数据持久化

### 概念
将数据独立于对话历史存储到文件系统，支持跨 session 访问和数据追踪。

### Cowork Forge 的实现

#### 1. 数据结构定义

```rust
// requirements.json
pub struct Requirements {
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub requirements: Vec<Requirement>,
}

// feature_list.json
pub struct FeatureList {
    pub schema_version: String,
    pub features: Vec<Feature>,
}

// design_spec.json
pub struct DesignSpec {
    pub schema_version: String,
    pub architecture: Architecture,
    pub technology_stack: TechnologyStack,
}

// implementation_plan.json
pub struct ImplementationPlan {
    pub schema_version: String,
    pub milestones: Vec<Milestone>,
    pub tasks: Vec<Task>,
}
```

#### 2. 持久化函数

```rust
// storage/mod.rs
pub fn load_requirements() -> Result<Requirements>;
pub fn save_requirements(reqs: &Requirements) -> Result<()>;
pub fn load_feature_list() -> Result<FeatureList>;
pub fn save_feature_list(features: &FeatureList) -> Result<()>;
pub fn load_design_spec() -> Result<DesignSpec>;
pub fn save_design_spec(design: &DesignSpec) -> Result<()>;
pub fn load_implementation_plan() -> Result<ImplementationPlan>;
pub fn save_implementation_plan(plan: &ImplementationPlan) -> Result<()>;
```

#### 3. 工具与持久化的集成

```rust
// CreateRequirementTool - 创建并保存需求
pub struct CreateRequirementTool;

#[async_trait]
impl Tool for CreateRequirementTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let mut reqs = load_requirements()?;
        
        let priority = match get_required_string_param(&args, "priority")?.as_str() {
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium,
        };
        
        let category = match get_required_string_param(&args, "category")?.as_str() {
            "functional" => RequirementCategory::Functional,
            "non_functional" => RequirementCategory::NonFunctional,
            _ => RequirementCategory::Functional,
        };
        
        let requirement = Requirement {
            id: generate_id("REQ", reqs.requirements.len()),
            title: get_required_string_param(&args, "title")?.to_string(),
            description: get_required_string_param(&args, "description")?.to_string(),
            priority,
            category,
            acceptance_criteria: get_required_array_param(&args, "acceptance_criteria")?
                .iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect(),
            related_features: vec![],
        };
        
        reqs.requirements.push(requirement);
        save_requirements(&reqs)?;  // 持久化到文件
        
        Ok(json!({"status": "success", "requirement_id": requirement.id}))
    }
}

// GetRequirementsTool - 从文件读取需求
pub struct GetRequirementsTool;

#[async_trait]
impl Tool for GetRequirementsTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> Result<Value> {
        let requirements = load_requirements()?;
        let features = load_feature_list()?;
        
        Ok(json!({
            "requirements": requirements.requirements,
            "features": features.features
        }))
    }
}
```

### 优点

1. **数据独立性**：数据不依赖对话历史，可独立访问
2. **可追溯性**：每次修改都有时间戳和版本信息
3. **跨 session 访问**：不同 session 可以访问同一项目的数据
4. **类型安全**：使用强类型结构，避免解析错误
5. **版本管理**：支持 schema_version 进行数据迁移

### 使用场景

```rust
// Stage 1: 创建需求
create_idea_agent() → 创建 idea.md
create_prd_loop() → 创建 requirements.json, feature_list.json

// Stage 2: 创建设计
create_design_loop() → 读取 requirements.json → 创建 design_spec.json

// Stage 3: 创建计划
create_plan_loop() → 读取 requirements.json, design_spec.json → 创建 implementation_plan.json

// Stage 4: 编码
create_coding_loop() → 读取 implementation_plan.json → 执行任务 → 更新 task status

// Stage 5: 检查
create_check_agent() → 读取所有数据 → 验证完整性

// Stage 6: 交付
create_delivery_agent() → 读取所有数据 → 生成报告
```

## 工具权限管理

### 核心原则

Cowork Forge 遵循**权限最小化**原则：每个 agent 只分配完成任务所需的最小工具集，避免不必要的权限。

### 工具权限分类

#### 1. 编码阶段（Coding & Check）

拥有完整的文件操作和命令执行权限：

```rust
// Coding Actor - 需要读写代码文件
let coding_actor = LlmAgentBuilder::new("coding_actor")
    .tool(Arc::new(ReadFileTool))      // 读取代码文件
    .tool(Arc::new(WriteFileTool))     // 写入代码文件
    .tool(Arc::new(ListFilesTool))     // 列出文件
    .tool(Arc::new(RunCommandTool))    // 运行测试/构建命令
    .tool(Arc::new(CheckTestsTool))    // 检查测试
    .tool(Arc::new(UpdateTaskStatusTool))  // 更新任务状态
    .build()?;

// Check Agent - 需要读取代码文件和运行命令
let check_agent = LlmAgentBuilder::new("check_agent")
    .tool(Arc::new(ReadFileTool))      // 读取代码文件
    .tool(Arc::new(ListFilesTool))     // 列出文件
    .tool(Arc::new(RunCommandTool))    // 运行测试/构建
    .tool(Arc::new(CheckTestsTool))    // 检查测试
    .tool(Arc::new(CheckLintTool))     // 检查代码质量
    .tool(Arc::new(GotoStageTool))     // 回退到之前阶段
    .build()?;
```

#### 2. 设计阶段（Idea、PRD、Design、Plan）

使用专用的 Artifact 工具，不提供通用文件操作权限：

```rust
// Idea Agent - 只能保存 idea.md
let idea_agent = LlmAgentBuilder::new("idea_agent")
    .tool(Arc::new(SaveIdeaTool))              // 保存 idea.md
    .tool(Arc::new(ReviewAndEditContentTool))  // 用户审查
    .build()?;

// PRD Actor - 只能保存 prd.md 和操作结构化数据
let prd_actor = LlmAgentBuilder::new("prd_actor")
    .tool(Arc::new(SavePrdDocTool))                    // 保存 prd.md
    .tool(Arc::new(ReviewWithFeedbackContentTool))     // 用户审查
    .tool(Arc::new(CreateRequirementTool))             // 创建需求
    .tool(Arc::new(AddFeatureTool))                    // 添加功能
    .tool(Arc::new(GetRequirementsTool))               // 读取需求数据
    .build()?;

// Design Actor - 只能保存 design.md
let design_actor = LlmAgentBuilder::new("design_actor")
    .tool(Arc::new(SaveDesignDocTool))                // 保存 design.md
    .tool(Arc::new(ReviewWithFeedbackContentTool))     // 用户审查
    .tool(Arc::new(CreateDesignComponentTool))        // 创建组件
    .tool(Arc::new(GetRequirementsTool))              // 读取需求
    .tool(Arc::new(GetDesignTool))                    // 读取设计
    .build()?;

// Plan Actor - 只能保存 plan.md
let plan_actor = LlmAgentBuilder::new("plan_actor")
    .tool(Arc::new(SavePlanDocTool))                  // 保存 plan.md
    .tool(Arc::new(ReviewWithFeedbackContentTool))     // 用户审查
    .tool(Arc::new(CreateTaskTool))                   // 创建任务
    .tool(Arc::new(GetRequirementsTool))              // 读取需求
    .tool(Arc::new(GetDesignTool))                    // 读取设计
    .tool(Arc::new(GetPlanTool))                      // 读取计划
    .build()?;
```

#### 3. 交付阶段（Delivery）

使用 Load 工具读取 artifacts，不提供通用读取权限：

```rust
// Delivery Agent - 只能加载 artifacts 和保存报告
let delivery_agent = LlmAgentBuilder::new("delivery_agent")
    .tool(Arc::new(LoadIdeaTool))          // 加载 idea.md
    .tool(Arc::new(LoadPrdDocTool))       // 加载 prd.md
    .tool(Arc::new(LoadDesignDocTool))    // 加载 design.md
    .tool(Arc::new(SaveDeliveryReportTool))  // 保存报告
    .tool(Arc::new(ListFilesTool))        // 验证项目文件
    .build()?;
```

### 专用工具实现

#### Save 工具（保存 Artifacts）

```rust
pub struct SavePrdDocTool;

#[async_trait]
impl Tool for SavePrdDocTool {
    fn name(&self) -> &str {
        "save_prd_doc"
    }

    fn description(&self) -> &str {
        "Save the PRD (Product Requirements Document) markdown file."
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let content = get_required_string_param(&args, "content")?;

        // 使用 artifact_path 确保路径限制在 artifacts 目录
        let path = artifact_path("prd.md")?;
        fs::write(&path, content)?;

        Ok(json!({
            "status": "success",
            "message": "PRD document saved successfully",
            "file_path": "artifacts/prd.md"  // 返回保存的路径
        }))
    }
}
```

#### Load 工具（加载 Artifacts）

```rust
pub struct LoadPrdDocTool;

#[async_trait]
impl Tool for LoadPrdDocTool {
    fn name(&self) -> &str {
        "load_prd_doc"
    }

    fn description(&self) -> &str {
        "Load the PRD (Product Requirements Document) markdown from the artifacts directory."
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {
        // 使用 artifact_path 确保路径限制在 artifacts 目录
        let path = artifact_path("prd.md")?;
        let content = fs::read_to_string(&path)?;

        Ok(json!({
            "status": "success",
            "content": content,
            "file_path": "artifacts/prd.md"
        }))
    }
}
```

### 权限矩阵

| 阶段 | ReadFile | WriteFile | LoadIdea | LoadPrd | LoadDesign | LoadPlan | SaveXxx | ListFiles | RunCommand | HITL |
|------|----------|-----------|----------|---------|------------|---------|---------|-----------|------------|------|
| Idea | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ save_idea | ❌ | ❌ | ✅ review_and_edit_content |
| PRD Actor | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ save_prd_doc | ❌ | ❌ | ✅ review_with_feedback_content |
| PRD Critic | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Design Actor | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ save_design_doc | ❌ | ❌ | ✅ review_with_feedback_content |
| Design Critic | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Plan Actor | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ save_plan_doc | ❌ | ❌ | ✅ review_with_feedback_content |
| Plan Critic | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Coding Actor | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ |
| Coding Critic | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ |
| Check Agent | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ |
| Delivery Agent | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ save_delivery_report | ✅ | ❌ | ❌ |

### 安全特性

#### 1. 路径限制

```rust
// storage/mod.rs
pub fn artifact_path(filename: &str) -> Result<PathBuf> {
    let iteration_dir = get_iteration_dir()?;
    let artifacts_dir = iteration_dir.join("artifacts");
    
    // 只允许在 artifacts 目录下操作
    let path = artifacts_dir.join(filename);
    
    // 验证路径没有遍历攻击
    let canonical = path.canonicalize()?;
    let artifacts_canonical = artifacts_dir.canonicalize()?;
    
    if !canonical.starts_with(&artifacts_canonical) {
        return Err(anyhow!("Path traversal detected"));
    }
    
    Ok(path)
}
```

#### 2. 命令白名单

```rust
// file_tools.rs
pub async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
    let command = get_required_string_param(&args, "command")?;
    
    // 拒绝危险命令
    let dangerous_patterns = ["rm -rf", "sudo", "format", "del /f", "format c:"];
    for pattern in dangerous_patterns {
        if command.to_lowercase().contains(pattern) {
            return Err(AdkError::Tool(format!("Dangerous command blocked: {}", command)));
        }
    }
    
    // 执行命令
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .timeout(Duration::from_secs(30))  // 超时控制
        .output()
        .await?;
    
    Ok(json!({
        "status": "success",
        "output": String::from_utf8_lossy(&output.stdout)
    }))
}
```

#### 3. Content-based HITL

使用基于内容的 HITL 工具，避免文件路径暴露：

```rust
// hitl_content_tools.rs
pub struct ReviewWithFeedbackContentTool;

#[async_trait]
impl Tool for ReviewWithFeedbackContentTool {
    fn name(&self) -> &str {
        "review_with_feedback_content"
    }

    fn description(&self) -> &str {
        "Review content and allow user to: edit, pass, or provide feedback."
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let title = get_required_string_param(&args, "title")?;
        let content = get_required_string_param(&args, "content")?;

        // 显示内容给用户
        interaction.show_message(MessageLevel::Info, format!("\n📝 {}\n{}", title, content)).await;

        // 获取用户输入
        let response = interaction.request_input(
            "Type 'edit' to open editor, 'pass' to continue, or provide feedback:",
            options,
            Some(content.to_string())
        ).await?;

        match response {
            InputResponse::Text(text) => {
                // 判断是编辑内容还是反馈
                if text.contains('\n') || text.len() > 100 {
                    Ok(json!({
                        "action": "edit",
                        "content": text
                    }))
                } else {
                    Ok(json!({
                        "action": "feedback",
                        "feedback": text,
                        "content": content
                    }))
                }
            }
            _ => Ok(json!({"action": "pass", "content": content}))
        }
    }
}
```

## 最佳实践

1. **工具设计**:
   - 提供清晰的名称和描述，帮助 LLM 理解何时使用工具
   - 定义明确的 JSON 模式和必需参数
   - 在工具实现中添加遥测和日志记录
   - **添加安全约束**：验证路径、拒绝绝对路径、防止路径遍历
   - **添加执行限制**：检测并拒绝阻塞命令

2. **Agent 构建**:
   - 使用 `max_iterations` 设置合理的迭代限制，防止无限循环
   - 提供清晰具体的指令，帮助 agent 理解何时调用 `exit_loop`
   - **避免在 SequentialAgent 中使用 exit_loop**：使用 max_iterations 替代
   - **使用 IncludeContents::None** 节省 token（除非需要对话历史）
   - 使用 `output_key` 保存关键的中间结果到会话状态

3. **工作流组合**:
   - 使用 `SequentialAgent` 构建线性处理流程
   - **考虑使用 ParallelAgent**：适用于并行分析和测试
   - 使用 `LoopAgent` 实现迭代改进或质量控制循环
   - **Actor-Critic Loop**：使用 max_iterations=1 避免过度优化

4. **状态管理**:
   - 使用适当的状态前缀（`user:`、`app:`、`temp:`）管理不同范围的持久性
   - **结构化数据持久化**：使用工具+文件系统，而非依赖对话历史
   - 在状态间传递关键信息和中间结果
   - 定期清理不再需要的临时数据

5. **错误处理**:
   - 在关键路径上使用前后回调进行验证
   - 实现适当的错误恢复机制
   - 添加足够的监控和日志记录

6. **HITL 设计**:
   - 在关键决策点使用 HITL 工具
   - 控制交互频率，避免过度打扰用户
   - 提供充分的上下文和清晰的提示
   - 支持多种反馈模式（edit、pass、text）

7. **数据传递**:
   - **结构化数据** → 使用工具模式（Tool trait）
   - **简单文本** → 使用 output_key 模式
   - **跨 session 数据** → 使用文件持久化

ADK-Rust 提供了一个强大且灵活的框架，可用于构建各种复杂的 AI 代理系统，从简单的任务助手到多阶段的工作流程引擎。Cowork Forge 的实践展示了如何将这些高级特性组合成一个完整的、生产就绪的开发系统。