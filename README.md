<p align="center">
  <img height="200" src="./assets/blend_banner.png">
</p>

<h1 align="center">Cowork Forge</h1>

<p align="center">
    <a href="./README.md">English</a>
    |
    <a href="./README_zh.md">中文</a>
</p>

<p align="center">
    <strong>🤖 Full-Stack AI Development Team - Complete Collaboration from Idea to Delivery 🤖</strong>
</p>
<p align="center">Cowork Forge is not just a code generator, but a complete virtual development team. It includes professional roles like Product Manager, Architect, Project Manager, and Engineer, working together through AI agent collaboration to transform your ideas into production-ready software systems.</p>

<p align="center">
  <a href="https://github.com/sopaco/cowork-forge/tree/main/litho.docs/en"><img alt="Litho Docs" src="https://img.shields.io/badge/Litho-Docs-green?logo=Gitbook&color=%23008a60"/></a>
  <a href="https://github.com/sopaco/cowork-forge/tree/main/litho.docs/zh"><img alt="Litho Docs" src="https://img.shields.io/badge/Litho-中文-green?logo=Gitbook&color=%23008a60"/></a>
  <a href="https://github.com/sopaco/cowork-forge"><img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/sopaco/cowork-forge/rust.yml?label=Build"/></a>
  <a href="./LICENSE"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-blue.svg?label=LICENSE" /></a>
</p>

<hr />

# 👋 What is Cowork Forge?

<strong>Cowork Forge</strong> is a complete AI-powered development team system that simulates how real software teams collaborate. AI agents take on roles as Product Manager, Architect, Project Manager, and Engineer, working together to complete everything from ideation to delivery.

Unlike AI coding assistants that only generate code snippets, Cowork Forge provides end-to-end development lifecycle management:
- **Product Manager AI**: Transforms your ideas into detailed Product Requirements Documents (PRD)
- **Architect AI**: Designs complete technical architecture and system components
- **Project Manager AI**: Breaks down tasks, plans dependencies, and defines implementation paths
- **Engineer AI**: Implements code, ensures quality, and generates delivery reports

Each role uses Actor-Critic patterns for self-review and optimization, with human validation at critical decision points to ensure output quality and direction.

<p align="center">
  <strong>One person can have a complete development team - Cowork Forge makes AI agents work for you like a real team.</strong>
</p>

<div style="text-align: center; margin: 30px 0;">
  <table style="width: 100%; border-collapse: collapse; margin: 0 auto;">
    <tr>
      <th style="width: 50%; padding: 15px; background-color: #f8f9fa; border: 1px solid #e9ecef; text-align: center; font-weight: bold; color: #495057;">Traditional Development</th>
      <th style="width: 50%; padding: 15px; background-color: #f8f9fa; border: 1px solid #e9ecef; text-align: center; font-weight: bold; color: #495057;">Development with Cowork Forge</th>
    </tr>
    <tr>
      <td style="padding: 15px; border: 1px solid #e9ecef; vertical-align: top;">
        <p style="font-size: 14px; color: #6c757d; margin-bottom: 10px;"><strong>Requires Multiple Roles and Tools</strong></p>
        <ul style="font-size: 13px; color: #6c757d; line-height: 1.6;">
          <li>Need Product Manager to write PRD</li>
          <li>Need Architect to design technical solution</li>
          <li>Need Project Manager to break down tasks</li>
          <li>Need Engineer to write code</li>
          <li>High cost and long cycle for multi-role collaboration</li>
        </ul>
      </td>
      <td style="padding: 15px; border: 1px solid #e9ecef; vertical-align: top;">
        <p style="font-size: 14px; color: #6c757d; margin-bottom: 10px;"><strong>One System Covers All Roles</strong></p>
        <ul style="font-size: 13px; color: #6c757d; line-height: 1.6;">
          <li>AI Product Manager auto-generates professional PRD</li>
          <li>AI Architect designs complete technical solution</li>
          <li>AI Project Manager intelligently breaks down tasks</li>
          <li>AI Engineer implements high-quality code</li>
          <li>Full-process collaboration, human only validates key decisions</li>
        </ul>
      </td>
    </tr>
  </table>
</div>

<hr />

## ✨ Visual Walkthrough
> Cowork Forge is an intelligent development engine that can be called by various technology stacks (Python/Java/NodeJS) through FFI as the high-performance cowork-core library; easily integrated into workflows through cowork-cli; also includes a locally deployable GUI project that provides a friendly interactive interface. Using Cowork Forge GUI as an example, we can intuitively experience the complete lifecycle of software development through its interface.

|  |  |
| :--- | :--- |
| **📄 Live Artifact Preview**：Watch documents being written in real-time. View formatted **PRDs**, **System Designs**, and **Implementation Plans** as they are generated. Support for Markdown rendering allows for clear, structured documentation reviews. ![Artifact Viewer](./assets/snapshots/artifact_preview.png) | **💬 Interactive Agent Chat**：Collaborate directly with your AI team. Discuss requirements with the Product Manager, review architecture with the Architect, or give feedback to the Engineer. The chat interface keeps the context of your entire project history. ![Agent Chat Interface](./assets/snapshots/chat_preview.png) |
| **🌐 Built-in App Preview**：See the result instantly. Cowork Forge GUI includes a web view to preview your generated web applications immediately after the build completes, without leaving the tool. ![App Preview](./assets/snapshots/app_preview.png) | **📝 Task & Todo Management**：Keep track of progress. The system automatically generates and maintains a Todo list for the current iteration, checking off items as the agents complete them. ![Todo List](./assets/snapshots/todo_preview.png) |
| **🚀 Project Dashboard**：The central hub for all your development initiatives. View active projects, check their status (Running, Paused, Completed), and quickly launch new ones from a clean, modern interface.![Project Dashboard](./assets/snapshots/dashboard_preview.png) | **⚡ Real-time Code Execution**：Monitor the **Coding Agent** as it writes files, runs builds, and executes tests. The built-in terminal view shows you exactly what commands are being run and their output, ensuring transparency and control. ![Terminal & Execution](./assets/snapshots/execution_preview.png) |

---

## 🔄 Development Workflow

Cowork Forge GUI guides you through the standard **7-Stage Development Lifecycle**:

1.  **💡 Idea Intake**: Chat with the agent to define your concept.
2.  **📋 PRD Generation**: Review the generated Product Requirements Document.
3.  **🏗️ Architecture Design**: Approve the technical design and system boundaries.
4.  **📅 Implementation Planning**: See the task breakdown and timeline.
5.  **💻 Coding & Testing**: Watch code being written and tests passing.
6.  **✅ Quality Check**: Verify the implementation against requirements.
7.  **📦 Delivery**: Receive the final project report and artifacts.

# 🏆 Cowork Forge vs. Competitors

Cowork Forge stands out in the AI development tools landscape through its unique multi-agent architecture and comprehensive workflow coverage.

## Core Capabilities Comparison

| Capability | Cowork Forge | GitHub Copilot | Cursor AI | Aider |
|------------|------------------------|----------------|-----------|-------|
| **End-to-End Workflow** | ✅ Complete (Idea→Delivery) | ❌ Code completion only | ❌ Code editing focus | ❌ Code assistance only |
| **Multi-Agent Architecture** | ✅ 8 specialized agents | ❌ Single model | ❌ Single model | ❌ Single model |
| **PRD Generation** | ✅ Automated | ❌ N/A | ❌ N/A | ❌ N/A |
| **Technical Design** | ✅ C4 architecture docs | ❌ N/A | ❌ N/A | ❌ N/A |
| **Implementation Planning** | ✅ Task breakdown & milestones | ❌ N/A | ❌ N/A | ❌ N/A |
| **Incremental Updates** | ✅ Smart delta analysis | ❌ N/A | ❌ Limited | ❌ Limited |
| **Multi-Language Support** | ✅ Rust, Python, JS/TS | ✅ Many languages | ✅ Many languages | ✅ Many languages |
| **Human-in-the-Loop** | ✅ Critical decision points | ❌ N/A | ❌ Limited | ❌ Limited |
| **Automated Verification** | ✅ Build/test integration | ❌ N/A | ❌ N/A | ❌ N/A |
| **Safety Checks** | ✅ Multi-layer security | ❌ N/A | ❌ Basic | ❌ Basic |
| **Artifact Storage** | ✅ Versioned artifacts | ❌ N/A | ❌ N/A | ❌ N/A |
| **Open Source** | ✅ MIT License | ❌ Proprietary | ❌ Proprietary | ✅ MIT License |
| **Self-Hosted** | ✅ Local execution | ❌ Cloud only | ❌ Cloud only | ✅ Local execution |

## Key Differentiators

### 1. Complete Virtual Development Team

Unlike tools that only assist with writing code, Cowork Forge provides complete development team role coverage:
- **Product Manager Role**: Transforms vague ideas into structured Product Requirements Documents
- **Architect Role**: Designs complete technical architecture and system components
- **Project Manager Role**: Breaks down tasks, plans dependencies, and implementation paths
- **Engineer Role**: Implements code and performs quality verification

This full-role coverage ensures end-to-end continuity and professionalism from requirements analysis to code implementation.

### 2. Multi-Agent Collaboration
Cowork Forge's specialized agents work together like a real development team:
- <strong>Idea Agent</strong>: Captures and structures user requirements
- <strong>PRD Loop Agent</strong>: Generates comprehensive PRDs with actor-critic refinement
- <strong>Design Loop Agent</strong>: Creates technical architecture with actor-critic refinement
- <strong>Plan Loop Agent</strong>: Breaks down implementation tasks with actor-critic refinement
- <strong>Coding Loop Agent</strong>: Plans and executes code changes with actor-critic refinement
- <strong>Check Agent</strong>: Verifies code quality and completeness
- <strong>Delivery Agent</strong>: Generates comprehensive delivery reports
- <strong>Change Triage Agent</strong>: Analyzes and triages incremental change requests
- <strong>Code Patch Agent</strong>: Implements precise code patches for modifications
- <strong>Modify Delivery Agent</strong>: Generates modification delivery reports

### 3. Human-in-the-Loop Validation
Critical outputs require human confirmation before proceeding, ensuring:
- Accurate requirement capture
- Sound technical decisions
- Feasible implementation plans
- Safe code changes

This balance of automation and human control sets Cowork Forge apart from fully autonomous tools.

### 4. Incremental Code Updates
When requirements or designs change, Cowork Forge intelligently identifies affected files and updates only what's necessary—preserving your customizations and avoiding full regeneration.

### 5. Built-in Safety
Multi-layer security checks prevent:
- Dangerous command execution (rm -rf, sudo, etc.)
- Unauthorized file system access
- Malicious code injection
- Resource exhaustion

❤️ Like <strong>Cowork Forge</strong>? Star it 🌟 or [Sponsor Me](https://github.com/sponsors/sopaco)! ❤️

# 🌠 Features & Capabilities

- <strong>7-Stage Development Workflow:</strong> Comprehensive workflow covering Idea Intake → PRD Generation → Technical Design → Implementation Plan → Coding → Quality Check → Delivery, corresponding to the complete process of a real development team.
- <strong>Specialized AI Agents:</strong> Each stage handled by a dedicated agent with domain expertise. Four critical stages (PRD, Design, Plan, Coding) use actor-critic loops for iterative refinement, ensuring output quality meets professional standards.
- <strong>Intelligent Code Planning:</strong> Analyzes project structure, dependencies, and generates precise code change plans.
- <strong>Incremental Code Updates:</strong> Smart delta analysis updates only affected files, preserving existing modifications.
- <strong>Automated Quality Verification:</strong> Multi-language build/test integration with comprehensive error analysis and reporting.
- <strong>Human-in-the-Loop Validation:</strong> Critical outputs (PRD, design, plans) require human confirmation before proceeding.
- <strong>Artifact-Based Storage:</strong> Versioned storage of all stage outputs with JSON and Markdown formats.
- <strong>Todo List Management:</strong> Automatic task tracking with status inference and progress reporting.
- <strong>Multi-Language Project Support:</strong> Automatic detection and handling of Rust, Python, JavaScript/TypeScript projects.
- <strong>Security & Safety:</strong> Command validation, path access control, and watchdog monitoring for safe execution.

# 🏗️ Architecture

Cowork Forge is built as a Rust workspace with modular, domain-driven architecture based on the adk-rust framework:

```mermaid
graph TB
    subgraph "CLI Layer"
        CLI[cowork-cli]
    end
    
    subgraph "Core Library"
        CORE[cowork-core]
    end
    
    subgraph "Core Modules"
        AGENTS[Agents]
        PIPELINE[Pipeline]
        TOOLS[Tools]
        PERSISTENCE[Persistence]
        DOMAIN[Domain]
    end
    
    subgraph "ADK Framework"
        ADK[adk-rust 0.2.1]
        LLM[adk-model]
    end
    
    subgraph "Infrastructure"
        FS[File System]
        CONFIG[Config]
        INTERACTION[Interaction]
    end
    
    subgraph "External"
        OPENAI[OpenAI LLM]
        EMBEDDING[Embedding API]
    end
    
    CLI --> CORE
    CORE --> AGENTS
    CORE --> PIPELINE
    CORE --> TOOLS
    
    AGENTS --> ADK
    PIPELINE --> DOMAIN
    TOOLS --> PERSISTENCE
    
    ADK --> LLM
    LLM --> OPENAI
    LLM --> EMBEDDING
    
    PIPELINE --> INTERACTION
    TOOLS --> FS
    CORE --> CONFIG
```

## Key Components

### Rust Workspace Structure
The project is organized as a Rust workspace with multiple crates:
- `cowork-core`: Core library with domain logic, agents, and tools
- `cowork-cli`: Command-line interface for interacting with the system
- `cowork-gui`: Optional graphical user interface (Tauri-based)

### Iteration Architecture
Core concept that manages complete development cycles as independent, inheritable units:
- **Genesis Iterations**: Start new projects from scratch
- **Evolution Iterations**: Build upon existing iterations with inheritance modes
- **Inheritance Modes**: None (fresh start), Full (complete code copy), Partial (documents only)

### ADK Framework Integration
Built on the adk-rust framework (v0.2.1) providing:
- Agent orchestration and management
- LLM integration with OpenAI and custom providers
- Tool system for safe code operations
- Built-in iteration support

### AI Agents
Specialized agents using the adk-rust agent framework:
- Idea Agent: Structures initial concepts
- Loop Agents (PRD, Design, Plan, Coding): Actor-critic pattern for refinement
- Check Agent: Validates implementation
- Delivery Agent: Finalizes deliverables

### Tools Module
Secure tool execution with workspace validation:
- File operations within project boundaries
- Command execution with safety checks
- Interactive tools for human-in-the-loop validation

### Persistence Layer
Data management and storage:
- Iteration storage and retrieval
- Artifact management
- Configuration persistence

# 🧠 How It Works

Cowork Forge uses a sophisticated multi-stage workflow orchestrated by the `Orchestrator`:

```mermaid
sequenceDiagram
    participant User as User
    participant CLI as Cowork Forge CLI
    participant Orch as Orchestrator
    participant Agents as AI Agents
    participant LLM as OpenAI LLM
    participant FS as File System
    participant CMD as Command Line

    User->>CLI: Provide idea/requirement
    CLI->>Orch: Start new session
    Orch->>Agents: Execute IdeaIntakeAgent
    Agents->>LLM: Structure requirements
    LLM-->>Agents: Return IdeaSpec
    Agents->>User: HITL validation
    User-->>Agents: Confirm/edit
    
    loop For each stage
        Orch->>Agents: Execute next agent
        Agents->>LLM: Generate stage output
        LLM-->>Agents: Return results
        
        alt Critical stage
            Agents->>User: HITL validation
            User-->>Agents: Confirm/edit
        end
        
        alt Coding stage
            Agents->>FS: Read project files
            Agents->>LLM: Plan code changes
            LLM-->>Agents: Return code plan
            Agents->>User: HITL validation
            User-->>Agents: Confirm plan
            Agents->>FS: Write code changes
            Agents->>CMD: Run build/test
            CMD-->>Agents: Return results
        end
    end
    
    Orch->>Agents: Execute DeliveryAgent
    Agents->>User: Present delivery report
```

# 🖥 Getting Started

### Prerequisites
- [**Rust**](https://www.rust-lang.org) (edition 2024)
- [**LLM API Access**](https://platform.openai.com/) (OpenAI or compatible provider)
- Git and language-specific build tools (cargo, npm, pip, etc.)

### Installation

Build from source:

```sh
# Clone the repository
git clone https://github.com/sopaco/cowork-forge.git
cd cowork-forge

# Build the entire workspace
cargo build --release

# The CLI binary will be available at:
# target/release/cowork
```

### Configuration

Cowork Forge uses a `config.toml` file for configuration. Create one in your project directory or use `--config` to specify a path:

```toml
# LLM Configuration
[llm]
api_base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-api-key"
model_name = "gpt-4"

# Optional: Embedding Configuration
[embedding]
api_base_url = "https://your-embedding-api.com/v1"
api_key = "your-embedding-api-key"
model_name = "text-embedding-ada-002"
```

# 🚀 Usage

Cowork Forge offers two ways to interact with your AI development team: the Command Line Interface (CLI) and the Graphical User Interface (GUI).

## 🖥️ Cowork CLI

### Iteration Management

```sh
# Initialize a new project
cowork init --name "My Project"

# Create a new iteration (Genesis)
cowork iter --title "Build a REST API" --description "Task management API with authentication"

# Create an evolution iteration
cowork iter --title "Add user profiles" --base iter-1-1234567890 --inherit partial

# List all iterations
cowork list

# Show iteration details
cowork show iter-1-1234567890

# Continue a paused iteration
cowork continue iter-1-1234567890

# Check status
cowork status
```

### Iteration Workflow

When you start an iteration, Cowork Forge guides you through the 7-stage workflow:

1. **Idea**: Your concept is structured into a formal specification
2. **PRD**: Product Requirements Document with actor-critic refinement
3. **Technical Design**: Architecture design with component specifications and actor-critic refinement
4. **Implementation Plan**: Task breakdown with dependencies and actor-critic refinement
5. **Coding**: Code implementation with actor-critic refinement and human validation
6. **Quality Check**: Verification of feature coverage and code completeness
7. **Delivery**: Final delivery report with implementation summary

At each critical stage, you'll be prompted to review and confirm the output before proceeding.

### Example Session Flow

```sh
# Initialize a new project
$ cowork init --name "My File Converter"

# Create a new iteration
$ cowork iter --title "Build a CLI tool" --description "A command-line tool for converting files between formats"

[Pipeline] Starting Genesis iteration: iter-1-1770536303
[Iteration] Stage 1/7: Idea Agent
[Idea Agent] Processing requirement...
[Idea Agent] Generated IdeaSpec at: .cowork-v2/iterations/iter-1-1770536303/artifacts/idea.md

Review the specification and provide feedback (or 'continue' to proceed):

> continue

[Iteration] Stage 2/7: PRD Loop Agent
[PRD Agent] Generating Product Requirements Document...
[PRD Agent] Generated PRD with 12 requirements at: .cowork-v2/iterations/iter-1-1770536303/artifacts/prd.md

Review the PRD and provide feedback (or 'continue' to proceed):

> continue

[Iteration] Stage 3/7: Design Loop Agent
[Design Agent] Creating technical architecture...
[Design Agent] Generated design at: .cowork-v2/iterations/iter-1-1770536303/artifacts/design.md

Review the design and provide feedback (or 'continue' to proceed):

> continue

... (continues through all 7 stages)

[Iteration] Stage 7/7: Delivery Agent
[Delivery Agent] Generating delivery report...
[Delivery Agent] Iteration completed successfully at: .cowork-v2/iterations/iter-1-1770536303/artifacts/delivery.md

Summary:
- 12 requirements implemented
- 4 modules created
- 15 test cases added
- Build: PASSED
- Tests: 15/15 PASSED

[Pipeline] Iteration iter-1-1770536303 completed successfully
```

### Managing Iterations

```sh
# List all iterations
$ cowork list
ID                     Title                    Status    Created At
iter-1-1770536303      Build a CLI tool         Completed 2023-12-01 10:30
iter-2-1770537500      Add batch processing      Paused    2023-12-01 14:45

# View iteration details
$ cowork show iter-1-1770536303

# Continue a paused iteration
$ cowork continue iter-2-1770537500

# Delete an iteration
$ cowork delete iter-2-1770537500

# Create an evolution iteration (based on existing)
$ cowork iter --title "Add batch processing" --base iter-1-1770536303 --inherit partial
```

### Inheritance Modes

When creating evolution iterations, you can choose from three inheritance modes:

| Mode        | Description                           | Use Case                          |
|-------------|---------------------------------------|-----------------------------------|
| `none`      | Fresh start, no inheritance           | Complete rewrites, new projects   |
| `full`      | Copy workspace code + artifacts       | Bug fixes, small enhancements     |
| `partial`   | Copy artifacts only, regenerate code  | Large features, architecture change|

### Configuration Management

```sh
# Initialize configuration file
cowork init --name "My Project"

# Use custom configuration
cowork iter --title "Your idea" --config ./custom-config.toml

# List iterations with detailed status
cowork list --all

# Check current project status
cowork status
```

## 🎨 Cowork GUI

The Cowork GUI provides a rich visual experience for managing your projects, monitoring agent progress, and previewing results.

### Features
- **Visual Dashboard**: Overview of all your projects and iterations.
- **Real-time Monitoring**: Watch agents work in real-time with detailed logs and status updates.
- **Interactive Chat**: Communicate with agents directly through a chat interface.
- **Built-in Preview**: Preview your generated web applications directly within the app.

### Running the GUI

To run the GUI from source:

1. Ensure you have Node.js and Rust installed.
2. Navigate to the GUI directory:
   ```sh
   cd crates/cowork-gui
   ```
3. Install frontend dependencies:
   ```sh
   npm install
   # or
   bun install
   ```
4. Start the application:
   ```sh
   cargo tauri dev
   ```

# 🌐 The Cowork Forge Ecosystem

Cowork Forge is organized as a modular Rust workspace based on the adk-rust framework:

```mermaid
graph TD
    subgraph "Workspace"
        CLI["cowork-cli"]
        CORE["cowork-core"]
        GUI["cowork-gui (optional)"]
    end

    subgraph "Dependencies"
        ADK["adk-rust 0.2.1"]
        ADK_MODEL["adk-model 0.2.1"]
    end
    
    subgraph "External Services"
        LLM[("LLM API")]
        EMBEDDING[("Embedding API")]
        FS[("File System")]
    end

    %% Define Dependencies
    CLI --> CORE
    CORE --> ADK
    CORE --> ADK_MODEL
    
    ADK_MODEL --> LLM
    ADK_MODEL --> EMBEDDING
    CORE --> FS
```

- <strong>`cowork-core`</strong>: Core library containing domain logic, agent implementations, and iteration management.
- <strong>`cowork-cli`</strong>: Command-line interface for iteration management and project interaction.
- <strong>`cowork-gui`</strong>: Optional graphical user interface based on Tauri framework.

### Core Modules

<strong>cowork-core</strong> is organized into the following domain modules:

- <strong>`pipeline`</strong>: Iteration pipeline orchestration managing iteration lifecycle and stage execution.
- <strong>`agents`</strong>: Specialized AI agents built with adk-rust framework (Idea, PRD Loop, Design Loop, Plan Loop, Coding Loop, Check, Delivery).
- <strong>`instructions`</strong>: Prompt templates for each agent using adk-rust instruction system.
- <strong>`tools`</strong>: File operations and command execution with workspace validation and security checks.
- <strong>`llm`</strong>: LLM integration layer using adk-model for OpenAI and custom providers.
- <strong>`domain`</strong>: Core domain entities including Iteration, Project, and Stage definitions.
- <strong>`persistence`</strong>: Iteration storage and retrieval system with inheritance support.
- <strong>`tech_stack`</strong>: Technology stack detection and configuration management.


# 🔒 Security

Cowork Forge implements multiple layers of security:

1. **Command Validation**: Dangerous commands are blocked before execution
2. **Path Access Control**: Restricted access to sensitive system directories
3. **Build Tool Whitelist**: Only authorized development tools can be executed
4. **Timeout Controls**: Prevents resource exhaustion from long-running commands
5. **Watchdog Monitoring**: Detects and prevents agent deviation from objectives

# 🤝 Contributing

We welcome all forms of contributions! Report bugs or submit feature requests through [GitHub Issues](https://github.com/sopaco/cowork-forge/issues).

### Development Process
1. Fork this project
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Create a Pull Request

### Running Tests

```sh
# Run all tests
cargo test

# Run tests with coverage
cargo test --all-features

# Run specific module tests
cargo test -p cowork-core
```

# 📚 Documentation

Comprehensive documentation is available in the [docs](./docs/) directory:

- [Architecture Overview](./docs/architecture/overview.md) - System architecture and design principles
- [Iteration Architecture](./docs/architecture/iteration-architecture.md) - The core iteration system
- [Agent System](./docs/architecture/agent-system.md) - AI agent implementation details
- [Pipeline Workflow](./docs/architecture/pipeline.md) - Stage execution and management
- [Development Guide](./docs/development/) - Contributor resources and patterns

# 🪪 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

# 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Powered by [OpenAI](https://openai.com/) GPT models
- Inspired by modern software development practices and AI agent research

# 📬 Contact

- **GitHub**: [sopaco/cowork-forge](https://github.com/sopaco/cowork-forge)
- **Issues**: [GitHub Issues](https://github.com/sopaco/cowork-forge/issues)

---

**Transform your development workflow with Cowork Forge—the future of collaborative software development.** 🚀
