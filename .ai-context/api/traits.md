# API: Key Traits

## InteractiveBackend

**Location**: `crates/cowork-core/src/interaction/mod.rs`

```rust
#[async_trait]
pub trait InteractiveBackend: Send + Sync {
    async fn show_message(&self, level: MessageLevel, content: String);
    
    async fn show_message_with_context(
        &self,
        level: MessageLevel,
        content: String,
        context: MessageContext,
    );

    async fn send_streaming(&self, content: String, agent_name: &str, is_thinking: bool);

    async fn send_tool_call(&self, tool_name: &str, arguments: &Value, agent_name: &str);

    async fn send_tool_result(
        &self,
        tool_name: &str,
        result: &str,
        success: bool,
        agent_name: &str,
    );

    async fn request_input(
        &self,
        prompt: &str,
        options: Vec<InputOption>,
        initial_content: Option<String>,
    ) -> Result<InputResponse>;

    async fn show_progress(&self, task_id: String, progress: ProgressInfo);

    async fn submit_response(&self, request_id: String, response: String) -> Result<()>;
}
```

## Stage

**Location**: `crates/cowork-core/src/pipeline/mod.rs`

```rust
#[async_trait]
pub trait Stage: Send + Sync {
    fn name(&self) -> &str;
    
    fn description(&self) -> &str;

    fn needs_confirmation(&self) -> bool {
        false
    }

    async fn execute(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> StageResult;

    async fn execute_with_feedback(
        &self,
        ctx: &PipelineContext,
        interaction: Arc<dyn InteractiveBackend>,
        _feedback: &str,
    ) -> StageResult {
        self.execute(ctx, interaction).await
    }
}
```

## StageResult

**Location**: `crates/cowork-core/src/pipeline/mod.rs`

```rust
#[derive(Debug)]
pub enum StageResult {
    Success(Option<String>),      // Artifact path
    Failed(String),               // Error message
    Paused,                       // Waiting for human confirmation
    NeedsRevision(String),        // Needs revision with feedback
    GotoStage(String, String),    // (target_stage, reason)
}
```

## PipelineContext

**Location**: `crates/cowork-core/src/pipeline/mod.rs`

```rust
#[derive(Debug, Clone)]
pub struct PipelineContext {
    pub project: Project,
    pub iteration: Iteration,
    pub workspace_path: std::path::PathBuf,
}
```

## Tool (from adk-rust)

**Location**: `adk_tool::Tool`

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    
    async fn execute(&self, args: Value) -> Result<Value, AdkError>;
}
```

## Agent (from adk-rust)

**Location**: `adk_core::Agent`

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    
    async fn run(
        &self,
        context: &dyn InvocationContext,
        input: Vec<Part>,
    ) -> Result<Vec<Part>, AdkError>;
}
```

## Llm (from adk-rust)

**Location**: `adk_core::Llm`

```rust
#[async_trait]
pub trait Llm: Send + Sync {
    async fn generate(
        &self,
        messages: Vec<Content>,
    ) -> Result<Content, AdkError>;
    
    async fn generate_stream(
        &self,
        messages: Vec<Content>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Content, AdkError>> + Send>>, AdkError>;
}
```
