# Agents Domain

## Responsibility
Build and configure AI agents using adk-rust framework for each pipeline stage.

## Agent Types

### Simple Agents
Single LlmAgent for non-critical stages.

| Agent | Stage | Tools |
|-------|-------|-------|
| `idea_agent` | Idea | SaveIdeaTool, ReviewAndEditContentTool, QueryMemoryTool |
| `check_agent` | Check | Validation tools, GotoStageTool, SaveCheckReportTool |
| `delivery_agent` | Delivery | Load artifact tools, SaveDeliveryReportTool, CopyWorkspaceToProjectTool |

### Actor-Critic Loops
LoopAgent with Actor + Critic for critical stages.

| Loop | Stages | Max Iterations |
|------|--------|----------------|
| `prd_loop` | PRD Actor → PRD Critic | 1 |
| `design_loop` | Design Actor → Design Critic | 1 |
| `plan_loop` | Plan Actor → Plan Critic | 1 |
| `coding_loop` | Coding Actor → Coding Critic | 5 |

## Agent Builders

### Location: `agents/mod.rs`

```rust
// Simple agents
fn create_idea_agent(model: Arc<dyn Llm>) -> Result<Arc<dyn Agent>>;
fn create_idea_agent_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;

fn create_check_agent(model: Arc<dyn Llm>) -> Result<Arc<dyn Agent>>;
fn create_check_agent_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;

fn create_delivery_agent(model: Arc<dyn Llm>) -> Result<Arc<dyn Agent>>;
fn create_delivery_agent_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;

// Actor-Critic loops
fn create_prd_loop(model: Arc<dyn Llm>) -> Result<Arc<dyn Agent>>;
fn create_prd_loop_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;

fn create_design_loop(model: Arc<dyn Llm>) -> Result<Arc<dyn Agent>>;
fn create_design_loop_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;

fn create_plan_loop(model: Arc<dyn Llm>) -> Result<Arc<dyn Agent>>;
fn create_plan_loop_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;

fn create_coding_loop(model: Arc<dyn Llm>) -> Result<Arc<dyn Agent>>;
fn create_coding_loop_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;

// Knowledge agents
fn create_summary_agent(model: Arc<dyn Llm>, iteration_id: String, iteration_number: u32) -> Result<Arc<dyn Agent>>;
fn create_knowledge_generation_agent(model: Arc<dyn Llm>, iteration_id: String, iteration_number: u32, base_iteration_id: Option<String>) -> Result<Arc<dyn Agent>>;

// PM Agent
fn create_project_manager_agent(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;
```

## Tool Configuration per Agent

### PRD Actor
```
LoadFeedbackHistoryTool, LoadIdeaTool, CreateRequirementTool, AddFeatureTool,
UpdateRequirementTool, UpdateFeatureTool, DeleteRequirementTool, GetRequirementsTool,
SavePrdDocTool, QueryMemoryTool, SaveInsightTool
```

### PRD Critic
```
GetRequirementsTool, LoadIdeaTool, ProvideFeedbackTool, QueryMemoryTool, SaveIssueTool
```

### Design Actor
```
LoadFeedbackHistoryTool, GetRequirementsTool, GetDesignTool, LoadPrdDocTool,
CreateDesignComponentTool, SaveDesignDocTool, QueryMemoryTool, SaveInsightTool,
SaveIssueTool, SaveLearningTool
```

### Coding Actor
```
LoadFeedbackHistoryTool, GetPlanTool, UpdateTaskStatusTool, UpdateFeatureStatusTool,
ReadFileTool, WriteFileTool, ListFilesTool, RunCommandTool, CheckTestsTool,
QueryMemoryTool, SaveInsightTool, SaveIssueTool, SaveLearningTool
```

## External Coding Agent

```rust
// Location: agents/external_coding_agent.rs
pub struct ExternalCodingAgent {
    acp_client: AcpClient,
}

pub struct StreamingTask {
    // ACP task with streaming support
}

impl ExternalCodingAgent {
    pub async fn execute_coding_task(&self, task: StreamingTask) -> Result<TaskResult>;
}
```

## PM Agent

```rust
// Location: agents/mod.rs

#[derive(Debug, Clone)]
pub struct PMAgentResult {
    pub message: String,
    pub actions: Vec<PMAgentAction>,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone)]
pub enum PMAgentAction {
    GotoStage { target_stage: String, reason: String },
    CreateIteration { iteration_id: String, title: String, description: String, inheritance: String },
}

pub async fn execute_pm_agent_message_streaming(
    model: Arc<dyn Llm>,
    iteration_id: String,
    message: String,
    history: Vec<Value>,
    stream_callback: Option<Arc<dyn PMAgentStreamCallback>>,
) -> Result<PMAgentResult>;
```

## Legacy Project Analyzer

```rust
// Location: agents/legacy_project_analyzer.rs
pub fn create_legacy_project_analyzer(model: Arc<dyn Llm>) -> Result<Arc<dyn Agent>>;
pub fn create_legacy_project_analyzer_with_id(model: Arc<dyn Llm>, iteration_id: String) -> Result<Arc<dyn Agent>>;
pub fn create_legacy_project_analyzer_with_context(model: Arc<dyn Llm>, iteration_id: String, context: String) -> Result<Arc<dyn Agent>>;
```

## Critical Implementation Note

```rust
// IMPORTANT: LoopAgent + SequentialAgent Bug
// When a sub-agent calls exit_loop(), it terminates the ENTIRE SequentialAgent.
// SOLUTION: Use max_iterations=1 to let LoopAgent complete naturally.
let mut loop_agent = LoopAgent::new("prd_loop", vec![actor, critic]);
loop_agent = loop_agent.with_max_iterations(1);
```

## Code Locations

```
crates/cowork-core/src/agents/
├── mod.rs                      # All agent builders
├── external_coding_agent.rs    # ACP-based external agent
├── iterative_assistant.rs      # Legacy (deprecated)
└── legacy_project_analyzer.rs  # Import analyzer
```
