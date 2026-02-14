# 🖥️ Cowork Forge GUI

<p align="center">
  <strong>Your Personal AI Development Team - Visualized</strong>
</p>

<p align="center">
  <a href="../../README.md">Core Repository</a> | <a href="../../README_zh.md">中文文档</a>
</p>

Cowork Forge GUI is the visual command center for your AI development team. Built with **Tauri v2** and **React**, it transforms the abstract process of AI code generation into a tangible, interactive, and manageable workflow.

Instead of staring at a terminal, watch your **Product Manager**, **Architect**, and **Engineer** agents collaborate in real-time to build your software.

---

## ✨ Key Features & Visual Walkthrough

Experience the full lifecycle of software development through our intuitive interface.

| Feature & Description | Visual Preview |
| :--- | :--- |
| **🚀 Project Dashboard**<br><br>The central hub for all your development initiatives. View active projects, check their status (Running, Paused, Completed), and quickly launch new ones from a clean, modern interface. | ![Project Dashboard](./assets/dashboard_preview.png)<br>*(Placeholder: Screenshot of the main dashboard showing a grid/list of projects with status indicators and a 'New Project' button)* |
| **💬 Interactive Agent Chat**<br><br>Collaborate directly with your AI team. Discuss requirements with the Product Manager, review architecture with the Architect, or give feedback to the Engineer. The chat interface keeps the context of your entire project history. | ![Agent Chat Interface](./assets/chat_preview.png)<br>*(Placeholder: Screenshot of the chat interface showing a conversation between the user and the 'Idea Agent' or 'PRD Agent', with message bubbles)* |
| **📄 Live Artifact Preview**<br><br>Watch documents being written in real-time. View formatted **PRDs**, **System Designs**, and **Implementation Plans** as they are generated. Support for Markdown rendering allows for clear, structured documentation reviews. | ![Artifact Viewer](./assets/artifact_preview.png)<br>*(Placeholder: Screenshot showing a split pane with chat on one side and a rendered Markdown document (like a PRD) on the other)* |
| **⚡ Real-time Code Execution**<br><br>Monitor the **Coding Agent** as it writes files, runs builds, and executes tests. The built-in terminal view shows you exactly what commands are being run and their output, ensuring transparency and control. | ![Terminal & Execution](./assets/execution_preview.png)<br>*(Placeholder: Screenshot showing the 'Coding' stage active, with a terminal window at the bottom displaying build logs or test results)* |
| **🌐 Built-in App Preview**<br><br>See the result instantly. Cowork Forge GUI includes a web view to preview your generated web applications immediately after the build completes, without leaving the tool. | ![App Preview](./assets/app_preview.png)<br>*(Placeholder: Screenshot showing a generated web application running inside the GUI's preview pane)* |
| **📝 Task & Todo Management**<br><br>Keep track of progress. The system automatically generates and maintains a Todo list for the current iteration, checking off items as the agents complete them. | ![Todo List](./assets/todo_preview.png)<br>*(Placeholder: Screenshot of the sidebar or panel showing a checklist of tasks for the current stage)* |

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

---

## 🛠️ Technical Architecture

The GUI is built on a robust, high-performance stack designed for local-first AI development:

*   **Frontend**: **React** + **Vite** + **Ant Design** for a responsive and familiar UX.
*   **Backend**: **Rust** (via **Tauri v2**) for native performance, system access, and security.
*   **Core Integration**: Directly embeds `cowork-core` to run the AI pipeline locally without external dependencies (except LLM APIs).
*   **State Management**: Real-time event streaming connects the Rust core logic with the React frontend.

---

## 🚀 Getting Started

### Prerequisites

*   **Rust** (Stable or Nightly)
*   **Node.js** (or Bun/pnpm)
*   **System Dependencies** for Tauri (Linux/macOS/Windows specific libs)

### Installation & Run

```bash
# Navigate to the GUI directory
cd crates/cowork-gui

# Install frontend dependencies
npm install  # or 'bun install'

# Run in development mode (with hot reload)
npm run tauri:dev  # or 'cargo tauri dev'
```

### Building for Production

```bash
# Build a standalone executable/installer
npm run tauri:build # or 'cargo tauri build'
```

The output binary will be located in `src-tauri/target/release/bundle/`.

---

<p align="center">
  Built with ❤️ by the Cowork Forge Team
</p>
