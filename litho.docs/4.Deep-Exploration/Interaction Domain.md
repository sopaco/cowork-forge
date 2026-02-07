# Interaction Domain Documentation

## Overview

The Interaction Domain in Cowork Forge serves as the critical abstraction layer between the core AI-powered development engine and user interfaces, enabling seamless Human-in-the-Loop (HITL) collaboration across both CLI and GUI environments. This domain implements the Strategy pattern to decouple business logic from presentation concerns, allowing the same core workflow to operate consistently whether users interact through a terminal or a graphical interface.

At its core, the Interaction Domain provides a unified interface for all user-facing operations—including message display, input collection, progress tracking, and response submission—through the `InteractiveBackend` trait. This design ensures that the pipeline executor, AI agents, and tool implementations can communicate with users without being coupled to any specific UI technology. The domain is implemented as a shared component (`cowork-core`) used by both the CLI interface (`cowork-cli`) and GUI interface (`cowork-gui`), guaranteeing behavioral consistency across platforms while allowing each to leverage platform-specific capabilities.

The domain is tightly integrated with the EventBus infrastructure, enabling real-time, event-driven communication between the engine and UI components. This integration allows the GUI to render live updates, progress indicators, and HITL prompts without requiring direct polling or blocking calls, creating a responsive and fluid user experience.

## Core Architecture

### InteractiveBackend Trait

The `InteractiveBackend` trait defines the contract that all UI implementations must satisfy, establishing a clean separation between the engine's business logic and the presentation layer. This trait is implemented by concrete backends for different interfaces, enabling polymorphic behavior through dependency injection.

```rust
#[async_trait]
pub trait InteractiveBackend: Send + Sync {
    async fn show_message(&self, level: MessageLevel, content: String);
    async fn request_input(&self, prompt: &str, options: Vec<InputOption>, initial_content: Option<String>) -> Result<InputResponse>;
    async fn show_progress(&self, task_id: String, progress: ProgressInfo);
    async fn submit_response(&self, request_id: String, response: String) -> Result<()>;
    fn event_bus(&self) -> Arc<EventBus>;
}
```

The trait exposes five essential methods:

1. **`show_message`**: Displays feedback to the user with semantic levels (Info, Success, Warning, Error, Debug) and corresponding emoji rendering.
2. **`request_input`**: Handles multi-modal user input—text entry, selection from options, or external editor invocation—with support for initial content pre-filling.
3. **`show_progress`**: Reports real-time progress for long-running operations with percentage, current/total counters, and descriptive messages.
4. **`submit_response`**: Enables asynchronous submission of HITL responses from GUI backends (e.g., modal dialog confirmations).
5. **`event_bus`**: Provides access to the shared event bus for subscribing to engine events.

All methods are `async` to accommodate non-blocking I/O operations, particularly important for GUI interactions that may involve network calls or UI thread coordination.

### Data Models

The domain defines several structured data types to ensure type-safe communication:

- **`MessageLevel`**: Enumerates semantic feedback levels with associated emoji representations (`ℹ️`, `✅`, `⚠️`, `❌`, `🔍`).
- **`InputOption`**: Represents selectable options with `id`, `label`, and optional `description` for rich CLI menus.
- **`InputResponse`**: Encapsulates user responses as `Text(String)`, `Selection(String)`, or `Cancel`, enabling the engine to distinguish between free-form feedback and menu selections.
- **`ProgressInfo`**: Tracks task progress with `current`, `total`, and `message` fields for dynamic progress rendering.

These types are `Serialize`/`Deserialize` enabled, facilitating serialization over IPC boundaries and future extensibility to remote UIs.

## Implementation Details

### CLI Backend (`CliBackend`)

The CLI backend provides a rich terminal-based interaction experience using the `dialoguer` crate for advanced input handling and UTF-8-compliant I/O.

#### Key Features:
- **Emoji-enhanced output**: Messages are rendered with semantic emojis for visual clarity.
- **Multi-mode input**: Supports both selection menus and text input with optional editor invocation.
- **Editor integration**: When users type `edit`, the system launches an external editor (e.g., VSCode) via `dialoguer::Editor`, preserving content across sessions.
- **Input fallbacks**: Users can type `pass` to skip input or `cancel` to abort operations.
- **Real-time progress**: Progress bars display percentage, current/total values, and descriptive messages.

```rust
async fn request_input(&self, prompt: &str, options: Vec<InputOption>, initial_content: Option<String>) -> Result<InputResponse> {
    if options.is_empty() {
        // Text input with editor option
        let input = read_line(prompt)?;
        if input.to_lowercase() == "edit" {
            let edited = Editor::new().edit(initial_content.as_deref())?;
            return Ok(InputResponse::Text(edited.unwrap_or_default()));
        }
        Ok(InputResponse::Text(input))
    } else {
        // Menu selection with fallback to text
        for (i, option) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, option.label);
        }
        let input = read_line("Your choice:")?;
        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= options.len() {
                return Ok(InputResponse::Selection(options[num - 1].id.clone()));
            }
        }
        // Fallback to text input
        Ok(InputResponse::Text(input))
    }
}
```

The CLI backend is fully synchronous in its interaction model, meaning user responses are received immediately within the execution flow, making it ideal for command-line workflows.

### GUI Backend (`TauriBackend`)

The Tauri backend is currently implemented as a placeholder within `cowork-core`, but its design anticipates full integration with the Tauri GUI application (`cowork-gui/src-tauri`). This architectural decision follows the principle of keeping core logic independent of UI implementation details.

#### Current Implementation:
- All methods output to console for debugging purposes.
- `submit_response` and `request_input` are stubbed, awaiting integration with Tauri's event system.
- The `event_bus()` method provides the same event bus instance used by the CLI, ensuring consistent event propagation.

#### Future Integration Plan:
- `request_input` will publish an `EngineEvent::HITLRequest` and await a response via a `oneshot` channel or WebSocket subscription.
- `show_message` and `show_progress` will emit Tauri events (`emit_all`) to trigger UI updates in the React frontend.
- `submit_response` will receive responses from the frontend via Tauri's `Emitter` system, completing the HITL loop.

This design enables a clean separation: the core engine remains unaware of whether it's interacting with a terminal or a web-based GUI, while the GUI layer handles platform-specific rendering and user interaction.

### Event Bus Integration

The Interaction Domain is deeply integrated with the `EventBus`, a publish-subscribe system built on `tokio::sync::broadcast::channel(1000)`.

#### Key Characteristics:
- **Broadcast Channel**: Supports up to 1,000 subscribers with non-blocking publishing.
- **Thread-safe**: All components (`Arc<EventBus>`) can be shared across threads.
- **Event Types**: Includes 14 distinct event variants covering:
  - Stage lifecycle (`StageStarted`, `StageCompleted`)
  - Agent activity (`AgentThinking`, `AgentOutput`)
  - Tool execution (`ToolStarted`, `ToolCompleted`)
  - File operations (`FileCreated`, `FileModified`)
  - Errors (`Error`)
  - HITL workflows (`HITLRequest`, `HITLResponse`)
  - Progress tracking (`Progress`)

#### Event Structure Example:
```rust
pub enum EngineEvent {
    HITLRequest {
        tool: String,
        content: String,
        options: Vec<String>,
        session_id: String,
    },
    HITLResponse {
        tool: String,
        response: String,
        session_id: String,
    },
    Progress {
        task_id: String,
        current: u32,
        total: u32,
        message: String,
    },
}
```

#### Usage Pattern:
```rust
// In pipeline stage:
event_bus.publish(EngineEvent::HITLRequest {
    tool: "ReviewAndEditFileTool".to_string(),
    content: file_content,
    options: vec!["approve".to_string(), "reject".to_string()],
    session_id: session_id.clone(),
});

// In GUI frontend:
let mut rx = event_bus.subscribe();
while let Ok(event) = rx.recv().await {
    match event {
        EngineEvent::HITLRequest { content, .. } => {
            show_modal_dialog(&content); // Show modal in React
        }
        EngineEvent::Progress { task_id, current, total, .. } => {
            update_progress_bar(task_id, current, total); // Update UI
        }
        _ => {}
    }
}
```

This event-driven model enables the GUI to remain responsive while the engine continues processing, and allows multiple UI components (e.g., status bar, progress panel, HITL modal) to independently subscribe to relevant events.

## Workflow Integration

The Interaction Domain is invoked at multiple critical points in the development workflow:

### 1. Development Iteration Process
During each stage of the pipeline (Idea → PRD → Design → Plan → Coding → Check → Delivery), AI agents may trigger HITL interactions:

- **PRD Stage**: `ReviewAndEditContentTool` calls `request_input` to prompt user review of generated requirements.
- **Coding Stage**: `ReviewAndEditFileTool` invokes the editor via `request_input` for code modifications.
- **Check Stage**: `ProvideFeedbackTool` uses `show_message` to display quality findings and `request_input` to collect user validation.

### 2. Change Request Analysis
When users initiate a modification (`cowork modify`), the system:
1. Captures natural language input via `request_input`.
2. Uses `show_message` to display scope analysis.
3. Publishes `HITLRequest` events to allow GUI users to approve or refine changes.

### 3. Real-Time Feedback Loop
The event bus enables a closed-loop feedback system:
- **Engine → UI**: Publishes `StageStarted`, `Progress`, `HITLRequest`.
- **UI → Engine**: Submits responses via `submit_response` (GUI) or direct `InputResponse` (CLI).
- **Engine → UI**: Confirms completion with `HITLResponse` and `StageCompleted`.

This ensures users are always aware of system state and can intervene at any point without disrupting the workflow.

## Architectural Advantages

### 1. Clean Separation of Concerns
By abstracting interaction through `InteractiveBackend`, the core engine remains agnostic to UI technology. This enables:
- Parallel development of CLI and GUI interfaces.
- Easy addition of new backends (e.g., web dashboard, mobile app).
- Independent testing of engine logic without UI dependencies.

### 2. Extensibility
New interaction patterns can be added without modifying the engine:
- Add `request_confirmation` method for multi-step approvals.
- Introduce `show_modal` for rich GUI dialogs.
- Support voice input via `SpeechInput` backend.

### 3. Resilience and Safety
- **Non-blocking I/O**: Async methods prevent UI freezes.
- **Error Handling**: All methods return `Result`, ensuring errors propagate appropriately.
- **UTF-8 Support**: CLI backend explicitly handles UTF-8 input/output, ensuring compatibility with international characters.

### 4. Cross-Platform Consistency
Despite different implementations, both CLI and GUI provide identical semantic interactions:
- Same input modes (`Text`, `Selection`, `Cancel`)
- Same event semantics (`HITLRequest` → `HITLResponse`)
- Same progress reporting format

This guarantees users experience consistent behavior regardless of interface, reducing cognitive load and training overhead.

## Integration Points

| Component | Interaction Type | Purpose |
|---------|------------------|---------|
| **Pipeline Executor** | Calls `InteractiveBackend` methods | Triggers user feedback during stage execution |
| **HITL Tools** | Uses `request_input` and `submit_response` | Enables document/code review workflows |
| **EventBus** | Publishes `EngineEvent` | Enables real-time UI updates |
| **Tauri GUI** | Subscribes to `EventBus` | Renders HITL modals, progress bars, and messages |
| **LLM Agents** | Indirectly via tools | Receive context from user feedback |

## Conclusion

The Interaction Domain is a foundational pillar of Cowork Forge’s architecture, enabling a seamless, consistent, and extensible Human-in-the-Loop experience across CLI and GUI interfaces. By leveraging Rust’s trait system, async/await concurrency, and event-driven communication, it achieves a rare balance of simplicity, robustness, and flexibility.

The domain’s design exemplifies clean architecture principles: it isolates UI concerns from business logic, promotes testability, and enables future innovation without architectural disruption. As the system evolves, new interaction modalities—such as voice commands, AI-assisted suggestions, or collaborative editing—can be seamlessly integrated by implementing the `InteractiveBackend` trait, ensuring Cowork Forge remains at the forefront of AI-assisted software development.