# Tools Domain

## Tool Categories

| Category | File | Key Tools |
|----------|------|-----------|
| File | `file_tools.rs` | ReadFile, WriteFile, ListFiles |
| Data | `data_tools.rs` | CreateRequirement, AddFeature, GetRequirements, CreateTask |
| HITL | `hitl_tools.rs` | ReviewWithFeedback, ProvideFeedback |
| HITL Content | `hitl_content_tools.rs` | ReviewAndEditContentTool, ReviewAndEditFileTool |
| Test/Lint | `test_lint_tools.rs` | RunCommand, ExecuteShellCommand, CheckTests, CheckLint |
| Memory | `memory_tools.rs` | QueryMemory, SaveInsight, PromoteToDecision |
| Artifact | `artifact_tools.rs` | SaveIdea, SavePrdDoc, SaveDesignDoc |
| Load | `load_artifacts.rs` | LoadIdea, LoadPrdDoc, LoadFeedbackHistory |
| Control | `control_tools.rs`, `goto_stage_tool.rs` | GotoStage (backward only) |
| Validation | `validation_tools.rs` | CheckFeatureCoverage, CheckTaskDependencies |
| Knowledge | `knowledge_tools.rs` | LoadBaseKnowledge, SaveKnowledgeSnapshot |
| PM | `pm_tools.rs` | PMGotoStage, PMCreateIteration, PMRespond |
| Deploy | `deployment_tools.rs` | CopyWorkspaceToProject |
| Import | `legacy_project_analyzer_tools.rs` | ImportProject |
| MCP | `mcp_tools.rs` | MCP remote server integration (Model Context Protocol) |

## Tool Notification System

Tools broadcast calls and results to GUI via global notifier (`tools/mod.rs`):
- `notify_tool_call(tool_name, args)` - called before tool execution
- `notify_tool_result(tool_name, result)` - called after tool execution
- Registered by GUI backend at startup via `set_tool_notify_callback()`

## Parameter Helpers

Safe parameter extraction utilities in `tools/mod.rs`:
- `get_required_string_param(args, key)` - required string from JSON args
- `get_optional_string_param(args, key)` - optional string from JSON args
- `get_required_array_param(args, key)` - required array from JSON args

## Security

- All file ops validated within workspace
- Dangerous commands blocked
- Path traversal prevention (UNC normalization on Windows)

## Location

`crates/cowork-core/src/tools/`