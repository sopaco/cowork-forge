# Cowork Forge - API Usage

## CLI Commands

### Core Commands
```bash
# Initialize new project
cargo run --package cowork-cli -- init <project-name>

# List all projects
cargo run --package cowork-cli -- list

# Show project status
cargo run --package cowork-cli -- status <project-id>

# Continue development iteration
cargo run --package cowork-cli -- continue <project-id>

# Delete project
cargo run --package cowork-cli -- delete <project-id>
```

### Knowledge Management
```bash
# Generate project knowledge
cargo run --package cowork-cli -- knowledge <project-id>

# Import external project
cargo run --package cowork-cli -- import <path>
```

### Configuration
```bash
# Show configuration
cargo run --package cowork-cli -- config show

# Update configuration
cargo run --package cowork-cli -- config set <key> <value>
```

## GUI Interface

### Development Mode
```bash
cd crates/cowork-gui
cargo tauri dev
```

### Build for Production
```bash
cd crates/cowork-gui
cargo tauri build
```

### Tauri Commands
- Project management commands
- Configuration commands
- Pipeline execution commands
- Real-time streaming events

## Agent System API

### Agent Types
```rust
// PM Agent - Requirements
pm_agent::generate_prd(idea: &str) -> Result<PrdDocument>

// Architect Agent - Design
architect_agent::create_design(prd: &PrdDocument) -> Result<DesignDocument>

// Engineer Agent - Code
engineer_agent::implement_code(design: &DesignDocument) -> Result<CodeArtifact>

// Check Agent - Validation
check_agent::validate(artifact: &CodeArtifact) -> Result<ValidationReport>

// Delivery Agent - Deployment
delivery_agent::package(artifact: &CodeArtifact) -> Result<PackageArtifact>
```

### Pipeline Execution
```rust
// Execute full pipeline
pipeline::execute(project: &mut Project) -> Result<Iteration>

// Execute single stage
pipeline::execute_stage(
    project: &mut Project,
    stage_id: &str
) -> Result<StageResult>
```

## Tool System API

### File Tools
```rust
// Read file within workspace
file_tools::read_file(path: &Path) -> Result<String>

// Write file within workspace
file_tools::write_file(path: &Path, content: &str) -> Result<()>

// List directory contents
file_tools::list_directory(path: &Path) -> Result<Vec<PathBuf>>
```

### Data Tools
```rust
// Load project data
data_tools::load_project(project_id: &str) -> Result<Project>

// Save project data
data_tools::save_project(project: &Project) -> Result<()>

// Update iteration
data_tools::update_iteration(
    project_id: &str,
    iteration: &Iteration
) -> Result<()>
```

### HITL Tools
```rust
// Request human input
hitl_tools::request_input(prompt: &str) -> Result<String>

// Request approval
hitl_tools::request_approval(
    proposal: &str
) -> Result<bool>

// Show progress
hitl_tools::show_progress(
    stage: &str,
    progress: f32
) -> Result<()>
```

## Configuration API

### Config Structure
```toml
# config.toml
[llm]
provider = "openai"
api_key = "your-api-key"
model = "gpt-4"

[pipeline]
max_iterations = 10
quality_threshold = 0.8

[workspace]
root_dir = "."
```

### Runtime Configuration
```rust
// Load configuration
config::load_config() -> Result<Config>

// Get specific setting
config::get_setting(key: &str) -> Result<Value>

// Update setting
config::update_setting(key: &str, value: Value) -> Result<()>
```

## Event System

### Tauri Events
- `pipeline-progress` - Pipeline execution updates
- `agent-output` - Agent generation output
- `stage-completed` - Stage completion notification
- `error occurred` - Error notifications

### Event Payload
```typescript
interface PipelineEvent {
  type: 'progress' | 'output' | 'completed' | 'error';
  project_id: string;
  stage?: string;
  agent?: string;
  content?: string;
  progress?: number;
  timestamp: string;
}
```

## Integration APIs

### ACP Protocol
```rust
// Connect to external agent
acp::connect(endpoint: &str) -> Result<AcpClient>

// Send task to external agent
acp::send_task(
    client: &AcpClient,
    task: Task
) -> Result<TaskResult>

// Receive results
acp::receive_results(
    client: &AcpClient
) -> Result<Vec<TaskResult>>
```

### Skill System
```rust
// Load skill
skills::load_skill(name: &str) -> Result<Skill>

// Execute skill
skills::execute_skill(
    skill: &Skill,
    context: Context
) -> Result<SkillResult>

// List available skills
skills::list_skills() -> Result<Vec<SkillInfo>>
```