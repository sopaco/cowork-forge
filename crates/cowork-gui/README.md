# Cowork Forge GUI

Desktop GUI for Cowork Forge built with Tauri v2 and React.

## Prerequisites

- Rust (nightly or stable)
- Bun runtime
- System dependencies for Tauri (see https://tauri.app/start/prerequisites/)

## Development

```bash
cd crates/cowork-gui
bun install
bun run dev
```

## Build

```bash
cd crates/cowork-gui
bun run build
```

## Features

- 🚀 Create new projects with AI
- 📊 Manage development sessions
- 🔄 Apply incremental updates
- 📈 Real-time event streaming
- 🎨 Modern React-based UI

## Architecture

- **Frontend**: React + Vite
- **Backend**: Rust (Tauri v2)
- **Core**: cowork-core library (shared with CLI)
- **Interaction**: InteractiveBackend abstraction
- **Events**: EventBus for real-time updates