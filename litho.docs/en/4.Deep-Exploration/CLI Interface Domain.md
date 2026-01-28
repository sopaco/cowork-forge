# CLI Interface Domain Technical Documentation

## Overview

The **CLI Interface Domain** serves as the primary entry point and user interaction layer for Cowork Forge, an AI-powered software development system. This domain implements a comprehensive command-line interface that orchestrates the complete software development lifecycle through specialized AI agents.

## Architecture Position

**Domain Type**: Presentation Layer  
**Role**: User-facing interface and application orchestrator  
**Dependencies**: Pipeline Orchestration Domain, Configuration Management  
**Dependents**: Users (Software Developers, Project Managers, Startup Founders)

## Key Components

### 1. Command Line Parser (`main.rs`)

The CLI parser utilizes the `clap` framework to define and process user commands with the following structure:

```rust
[derive(Parser)]
#[command(name = "cowork")]
#[command(about = "AI-powered software development system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file (default: config.toml)
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable LLM streaming output
    #[arg(short, long, global = true)]
    stream: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new project
    New { idea: String },
    
    /// Resume an existing project
    Resume,
    
    /// Modify existing project from a stage
    Modify { from: String },
    
    /// Show project status
    Status,
    
    /// Initialize config file
    Init,
}
```

### 2. Application Orchestrator

The orchestrator handles the complete command execution workflow:

- **Configuration Loading**: Loads model configuration from file or environment variables
- **Logging Setup**: Configures tracing with appropriate verbosity levels
- **Command Dispatching**: Routes commands to appropriate handler functions
- **Error Handling**: Provides user-friendly error messages and recovery guidance

## Technical Implementation

### Dependencies and Integration

```toml
[dependencies]
cowork-core = { path = "../cowork-core" }  # Core business logic
adk-runner = "0.2.1"                      # AI agent execution framework
clap = { workspace = true }               # Command-line parsing
tokio = { workspace = true }              # Async runtime
tracing = { workspace = true }            # Structured logging
```

### Async Execution Model

The CLI utilizes Tokio for asynchronous execution, enabling non-blocking AI agent operations and real-time streaming:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    // Async command execution
    match cli.command {
        Commands::New { idea } => cmd_new(idea, &config, enable_stream).await?,
        // ... other commands
    }
    Ok(())
}
```

## Core Command Handlers

### 1. New Project Creation (`cmd_new`)

**Purpose**: Initialize a new software development project from an idea

**Workflow**:
1. Validate project directory doesn't exist
2. Create full development pipeline
3. Execute pipeline with user-provided idea
4. Handle real-time LLM streaming if enabled

```rust
async fn cmd_new(idea: String, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    if cowork_dir_exists() {
        error!(".cowork directory already exists");
        anyhow::bail!("Project already initialized");
    }
    
    let pipeline = create_cowork_pipeline(config)?;
    execute_pipeline(pipeline, &idea, enable_stream).await?;
}
```

### 2. Project Resumption (`cmd_resume`)

**Purpose**: Continue development from the last completed stage

**Workflow**:
1. Validate project directory exists
2. Create pipeline that skips completed stages
3. Resume execution from detected checkpoint
4. Maintain session continuity

### 3. Stage Modification (`cmd_modify`)

**Purpose**: Restart development from a specific stage

**Workflow**:
1. Validate target stage parameter (prd, design, plan, coding, check, delivery)
2. Create partial pipeline starting from specified stage
3. Execute modified workflow
4. Preserve artifacts from previous stages

### 4. Pipeline Execution Engine (`execute_pipeline`)

**Purpose**: Execute AI agent pipelines with session management

**Key Features**:
- Session creation and management using ADK framework
- Real-time LLM output streaming
- Error handling and recovery mechanisms
- State persistence for workflow continuity

```rust
async fn execute_pipeline(pipeline: Arc<dyn adk_core::Agent>, input: &str, enable_stream: bool) -> Result<()> {
    let session_service = Arc::new(InMemorySessionService::new());
    let session = session_service.create(CreateRequest {
        app_name: "cowork-forge".to_string(),
        user_id: "cowork-user".to_string(),
        session_id: None,
        state: HashMap::new(),
    }).await?;
    
    // Execute with streaming support
    // ...
}
```

## Configuration Management

### Configuration Sources
- **File-based**: Load from `config.toml` (default)
- **Environment Variables**: Fallback when file not found
- **Command-line Override**: Custom config path via `--config` flag

### Configuration Loading Logic
```rust
fn load_config(path: &str) -> Result<ModelConfig> {
    if Path::new(path).exists() {
        info!("Loading configuration from {}", path);
        ModelConfig::from_file(path)
    } else {
        info!("Loading from environment variables");
        ModelConfig::from_env()
    }
}
```

## Logging and Output Management

### Multi-level Logging System
- **Normal Mode**: Filters out verbose ADK internals
- **Verbose Mode**: Shows detailed debug information including ADK logs
- **Stream Output**: Real-time LLM thinking process display

```rust
let log_filter = if cli.verbose {
    "debug".to_string()  // Full debug logging
} else {
    "info,adk_agent=warn,adk_core=warn,adk_runner=warn".to_string()
};
```

### Output Stream Separation
- **STDERR**: Log messages and system information
- **STDOUT**: User-facing content and LLM responses

## Error Handling Strategy

### Validation Checks
- Project directory existence validation
- Configuration file accessibility
- Stage parameter validation
- Session creation error handling

### User-Friendly Error Messages
- Clear guidance for common issues
- Actionable recovery suggestions
- Context-aware error reporting

## Integration Patterns

### 1. Pipeline Orchestration Integration
The CLI delegates workflow execution to the Pipeline Orchestration Domain:
- Creates appropriate pipeline types (full, resume, partial)
- Passes user input and configuration
- Handles pipeline execution results

### 2. Session Management
Utilizes ADK session services for:
- Workflow state persistence
- Agent context management
- User session isolation

### 3. Real-time Communication
Supports LLM streaming for enhanced user experience:
- Live agent thinking process display
- Progress indication during long operations
- Interactive feedback mechanisms

## Usage Patterns

### Typical Workflow
```bash
# Initialize configuration
cowork init

# Start new project
cowork new "Build a task management application"

# Check project status
cowork status

# Resume interrupted project
cowork resume

# Modify from specific stage
cowork modify --from design
```

### Advanced Options
```bash
# Verbose logging with streaming
cowork --verbose --stream new "Project idea"

# Custom configuration
cowork --config custom.toml new "Project idea"
```

## Best Practices

### 1. Command Design
- Clear, intuitive command names
- Consistent parameter patterns
- Comprehensive help documentation

### 2. User Experience
- Progress indicators for long operations
- Clear success/failure messages
- Recovery guidance for common errors

### 3. Performance Considerations
- Async execution for non-blocking operations
- Efficient session management
- Optimal logging verbosity levels

The CLI Interface Domain successfully bridges user interaction with the powerful AI-powered development engine, providing an intuitive interface for managing complex software development workflows while maintaining technical robustness and user-friendly operation.