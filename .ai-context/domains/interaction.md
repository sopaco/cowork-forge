# Interaction Domain

## Responsibility
Abstract user interaction between core engine and UI implementations (CLI/GUI).

## Core Trait

```rust
// Location: interaction/mod.rs
#[async_trait]
pub trait InteractiveBackend: Send + Sync {
    // Display methods
    async fn show_message(&self, level: MessageLevel, content: String);
    async fn show_message_with_context(&self, level: MessageLevel, content: String, context: MessageContext);
    async fn send_streaming(&self, content: String, agent_name: &str, is_thinking: bool);
    async fn send_tool_call(&self, tool_name: &str, arguments: &Value, agent_name: &str);
    async fn send_tool_result(&self, tool_name: &str, result: &str, success: bool, agent_name: &str);
    
    // HITL methods
    async fn request_input(&self, prompt: &str, options: Vec<InputOption>, initial_content: Option<String>) -> Result<InputResponse>;
    
    // Progress
    async fn show_progress(&self, task_id: String, progress: ProgressInfo);
    
    // Response submission (for GUI async HITL)
    async fn submit_response(&self, request_id: String, response: String) -> Result<()>;
}
```

## Supporting Types

```rust
enum MessageLevel {
    Info, Success, Warning, Error, Debug,
}

enum MessageType {
    Normal,
    Thinking,
    ToolCall { tool_name: String, arguments: Value },
    ToolResult { tool_name: String, success: bool },
    Streaming { is_first: bool, is_last: bool },
}

struct MessageContext {
    agent_name: String,
    message_type: MessageType,
    stage_name: Option<String>,
}

struct InputOption {
    id: String,
    label: String,
    description: Option<String>,
}

enum InputResponse {
    Text(String),
    Selection(String),
    Cancel,
}

struct ProgressInfo {
    current: u32,
    total: u32,
    message: String,
}
```

## Implementations

### CliBackend
- **Location**: `interaction/cli.rs`
- **Features**:
  - Terminal output with colored formatting
  - `dialoguer` for interactive prompts
  - External editor invocation for content review

### TauriBackend
- **Location**: `interaction/tauri.rs`
- **Features**:
  - Event-driven communication via Tauri IPC
  - `oneshot` channels for async HITL responses
  - Real-time streaming to React frontend

## HITL Flow

```
Stage Execution
    │
    ▼
Stage generates artifact
    │
    ▼
InteractiveBackend::request_input()
    │ Options: [Pass, View, Feedback, Cancel]
    │
    ├─► CLI: dialoguer::Select prompt
    │         External editor for content view
    │
    └─► GUI: Emit input_request event
              React shows modal
              User response via submit_response()
              oneshot::Receiver waits
    │
    ▼
InputResponse returned to Stage
```

## Event Types (GUI)

| Event | Purpose |
|-------|---------|
| `agent_event` | Agent messages, stage transitions |
| `agent_streaming` | Token-by-token LLM output |
| `tool_call` | Tool invocation notification |
| `tool_result` | Tool execution result |
| `input_request` | HITL modal trigger |
| `project_log` | Development server logs |

## Code Locations

```
crates/cowork-core/src/interaction/
├── mod.rs      # InteractiveBackend trait, types
├── cli.rs      # CliBackend implementation
└── tauri.rs    # TauriBackend implementation
```
