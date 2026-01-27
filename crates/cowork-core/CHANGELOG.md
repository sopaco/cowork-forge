# Changelog - Cowork Forge V2

All notable changes to the V2 version of Cowork Forge will be documented in this file.

## [Unreleased]

### Added - V2 Complete Rewrite
- **Complete architecture rewrite** using adk-rust framework 0.2.1
- **Actor-Critic Pattern**: Each stage has paired Actor and Critic agents in LoopAgent
- **Structured Data Model**: 5-layer JSON-based data flow (Requirements → Features → Design → Plan → Code)
- **21 Comprehensive Tools**:
  - Data tools (12): create_requirement, add_feature, create_design_component, create_task, etc.
  - Validation tools (3): check_data_format, check_feature_coverage, check_task_dependencies
  - File tools (3): read_file, write_file, list_files
  - Command tools (3): run_command, check_tests, check_lint
  - Control tools (3): provide_feedback, ask_user, exit_loop
  - Artifact tools (4): save_delivery_report, save_prd_doc, save_design_doc, load_feedback_history
  - Stage control (1): goto_stage for pipeline restart capability

### Agents
- **IdeaAgent**: Captures and documents user's initial project idea
- **PRD Loop**: PRD Actor + Critic for requirements and features (10 iterations max)
- **Design Loop**: Design Actor + Critic for architecture (10 iterations max)
- **Plan Loop**: Plan Actor + Critic for implementation planning (10 iterations max)
- **Coding Loop**: Coding Actor + Critic for code generation (20 iterations max)
- **Check Agent**: Final quality assurance with goto_stage capability
- **Delivery Agent**: Comprehensive delivery report generation

### Pipeline
- **SequentialAgent-based pipeline** connecting all 7 stages
- **Three pipeline modes**:
  - Full pipeline (new projects)
  - Resume pipeline (skip idea stage)
  - Partial pipeline (start from specific stage)

### CLI
- `cowork-v2 init` - Initialize configuration file
- `cowork-v2 new <idea>` - Start new project
- `cowork-v2 resume` - Resume existing project
- `cowork-v2 modify --from <stage>` - Modify from specific stage
- `cowork-v2 status` - Show project status

### Features
- **OpenAI-compatible API support** - Works with any OpenAI-compatible endpoint
- **DFS cycle detection** - Prevents circular task dependencies
- **Real-time event streaming** - Live progress updates during execution
- **Session management** - Track project state across runs
- **Feedback system** - Structured feedback loop between Actor and Critic
- **Stage restart** - goto_stage allows Check Agent to restart from earlier stages

### Testing
- Unit tests for data models
- Storage layer tests with tempfile
- Configuration parsing tests
- 12 passing tests total

### Documentation
- Comprehensive README with architecture details
- Detailed agent instruction prompts
- Example configuration file
- API documentation in code comments

## [0.1.0] - V1 (Legacy)
See main CHANGELOG.md for V1 history.
