# Tools Domain

## Responsibility
Provide 30+ ADK tools for agent operations: file I/O, data CRUD, HITL, memory, validation.

## Tool Categories

### File Tools
- **Location**: `tools/file_tools.rs`
- **Purpose**: File system operations within workspace

| Tool | Description |
|------|-------------|
| `ReadFileTool` | Read file contents |
| `WriteFileTool` | Write/create file |
| `ListFilesTool` | List directory contents |

### Data Tools
- **Location**: `tools/data_tools.rs`
- **Purpose**: CRUD operations for requirements, features, tasks, design

| Tool | Description |
|------|-------------|
| `CreateRequirementTool` | Create a requirement entry |
| `AddFeatureTool` | Add feature to requirement |
| `UpdateRequirementTool` | Update existing requirement |
| `UpdateFeatureTool` | Update feature details |
| `DeleteRequirementTool` | Delete requirement |
| `GetRequirementsTool` | Query all requirements |
| `CreateTaskTool` | Create implementation task |
| `UpdateTaskStatusTool` | Update task status |
| `UpdateFeatureStatusTool` | Update feature status |
| `GetPlanTool` | Get all tasks |
| `GetDesignTool` | Get design components |
| `CreateDesignComponentTool` | Create design component |

### HITL Tools
- **Location**: `tools/hitl_tools.rs`, `tools/hitl_content_tools.rs`
- **Purpose**: Human-in-the-loop interaction

| Tool | Description |
|------|-------------|
| `ReviewWithFeedbackContentTool` | Show content, get pass/feedback |
| `ReviewAndEditContentTool` | Open editor for content |
| `ProvideFeedbackTool` | Submit feedback to critic |

### Memory Tools
- **Location**: `tools/memory_tools.rs`
- **Purpose**: Knowledge management

| Tool | Description |
|------|-------------|
| `QueryMemoryTool` | Query project/iteration memory |
| `SaveInsightTool` | Save insight to memory |
| `SaveIssueTool` | Save issue to memory |
| `SaveLearningTool` | Save learning to memory |
| `PromoteToDecisionTool` | Promote insight to decision |
| `PromoteToPatternTool` | Promote learning to pattern |

### Validation Tools
- **Location**: `tools/validation_tools.rs`
- **Purpose**: Verify data integrity

| Tool | Description |
|------|-------------|
| `CheckFeatureCoverageTool` | Verify all features have design |
| `CheckTaskDependenciesTool` | Verify task dependencies valid |
| `CheckDataFormatTool` | Verify data format correct |

### Control Tools
- **Location**: `tools/control_tools.rs`, `tools/goto_stage_tool.rs`
- **Purpose**: Pipeline flow control

| Tool | Description |
|------|-------------|
| `GotoStageTool` | Jump to specific stage |

### Artifact Tools
- **Location**: `tools/artifact_tools.rs`
- **Purpose**: Save stage artifacts

| Tool | Description |
|------|-------------|
| `SaveIdeaTool` | Save idea.md |
| `SavePrdDocTool` | Save prd.md |
| `SaveDesignDocTool` | Save design.md |
| `SavePlanDocTool` | Save plan.md |
| `SaveCheckReportTool` | Save check_report.md |
| `SaveDeliveryReportTool` | Save delivery_report.md |

### Load Artifact Tools
- **Location**: `tools/load_artifacts.rs`
- **Purpose**: Load previous artifacts

| Tool | Description |
|------|-------------|
| `LoadIdeaTool` | Load idea.md |
| `LoadPrdDocTool` | Load prd.md |
| `LoadDesignDocTool` | Load design.md |
| `LoadPlanDocTool` | Load plan.md |
| `LoadFeedbackHistoryTool` | Load feedback history |

### Deployment Tools
- **Location**: `tools/deployment_tools.rs`
- **Purpose**: Deploy generated code

| Tool | Description |
|------|-------------|
| `CopyWorkspaceToProjectTool` | Copy workspace to project root |

### Test/Lint Tools
- **Location**: `tools/test_lint_tools.rs`
- **Purpose**: Run validation commands

| Tool | Description |
|------|-------------|
| `RunCommandTool` | Execute shell command |
| `CheckTestsTool` | Run tests |
| `CheckLintTool` | Run linter |

### Knowledge Tools
- **Location**: `tools/knowledge_tools.rs`
- **Purpose**: Knowledge generation

| Tool | Description |
|------|-------------|
| `LoadBaseKnowledgeTool` | Load base iteration knowledge |
| `SaveKnowledgeSnapshotTool` | Save iteration knowledge |
| `LoadDocumentSummaryTool` | Load document summaries |
| `ReadFileWithLimitTool` | Read file with call limit |

### PM Tools
- **Location**: `tools/pm_tools.rs`
- **Purpose**: Project Manager Agent operations

| Tool | Description |
|------|-------------|
| `PMGotoStageTool` | Navigate to stage |
| `PMCreateIterationTool` | Create new iteration |
| `PMRespondTool` | Respond to user |
| `PMSaveDecisionTool` | Save decision |

### Legacy Project Analyzer Tools
- **Location**: `tools/legacy_project_analyzer_tools.rs`
- **Purpose**: Import existing projects

| Tool | Description |
|------|-------------|
| `ImportProjectTool` | Analyze and import project |

## Tool Notification System

```rust
// Global callback for GUI to receive tool events
fn set_tool_notify_callback<F>(callback: F);
fn notify_tool_call(tool_name: &str, args: &Value);
fn notify_tool_result(tool_name: &str, result: &Result<Value, AdkError>);
```

## Tool Parameter Helpers

```rust
fn get_required_string_param<'a>(args: &'a Value, key: &str) -> Result<&'a str, AdkError>;
fn get_optional_string_param(args: &Value, key: &str) -> Option<String>;
fn get_required_array_param<'a>(args: &'a Value, key: &str) -> Result<&'a Vec<Value>, AdkError>;
```

## Security Constraints

1. **Path Validation**: All file operations validated within workspace
2. **Command Sanitization**: Dangerous commands blocked (rm -rf, sudo, etc.)
3. **Workspace Containment**: No access outside project directory

## Code Location

```
crates/cowork-core/src/tools/
├── mod.rs                        # Re-exports, notification system
├── file_tools.rs                 # File I/O
├── data_tools.rs                 # Requirements, features, tasks
├── hitl_tools.rs                 # Human interaction
├── hitl_content_tools.rs         # Content review
├── memory_tools.rs               # Knowledge management
├── validation_tools.rs           # Data validation
├── control_tools.rs              # Flow control
├── artifact_tools.rs             # Save artifacts
├── load_artifacts.rs             # Load artifacts
├── deployment_tools.rs           # Deploy code
├── test_lint_tools.rs            # Test/lint execution
├── knowledge_tools.rs            # Knowledge generation
├── pm_tools.rs                   # PM Agent tools
└── legacy_project_analyzer_tools.rs  # Import tools
```
