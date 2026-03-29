# Tools Domain

## Tool Categories

| Category | File | Key Tools |
|----------|------|-----------|
| File | `file_tools.rs` | ReadFile, WriteFile, ListFiles |
| Data | `data_tools.rs` | CreateRequirement, AddFeature, GetRequirements, CreateTask |
| HITL | `hitl_tools.rs` | ReviewWithFeedback, ProvideFeedback |
| Memory | `memory_tools.rs` | QueryMemory, SaveInsight, PromoteToDecision |
| Artifact | `artifact_tools.rs` | SaveIdea, SavePrdDoc, SaveDesignDoc |
| Load | `load_artifacts.rs` | LoadIdea, LoadPrdDoc, LoadFeedbackHistory |
| Control | `control_tools.rs`, `goto_stage_tool.rs` | GotoStage |
| Validation | `validation_tools.rs` | CheckFeatureCoverage, CheckTaskDependencies |
| Test/Lint | `test_lint_tools.rs` | RunCommand, CheckTests, CheckLint |
| Knowledge | `knowledge_tools.rs` | LoadBaseKnowledge, SaveKnowledgeSnapshot |
| PM | `pm_tools.rs` | PMGotoStage, PMCreateIteration, PMRespond |
| Deploy | `deployment_tools.rs` | CopyWorkspaceToProject |
| Import | `legacy_project_analyzer_tools.rs` | ImportProject |

## Security

- All file ops validated within workspace
- Dangerous commands blocked

## Location

`crates/cowork-core/src/tools/`