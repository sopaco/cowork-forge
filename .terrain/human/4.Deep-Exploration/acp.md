# ACP 领域

**模块路径**：`crates/cowork-core/src/acp/`
**生成日期**：2026-07-05

---

## 概述

ACP 模块是 Cowork Forge 的"外交部门"——它负责通过 Agent Client Protocol 与系统外部的 AI Agent 通信。当内置的 Coding Agent 不够用时，可以调用外部编码 Agent（如 OpenCode、Gemini CLI、Claude CLI）来完成编码任务。

---

## 核心功能点

1. **ACP 客户端**——`AcpClient` 实现 ACP 协议，支持 stdio 和 WebSocket 传输。`crates/cowork-core/src/acp/client.rs`
2. **外部编码 Agent 集成**——`ExternalCodingAgent` 适配 OpenCode、iFlow、Codex、Gemini CLI、Claude CLI。`crates/cowork-core/src/agents/external_coding_agent.rs`

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `AcpClient` | `crates/cowork-core/src/acp/client.rs` | ACP 协议客户端 |
| `ExternalCodingAgent` | `crates/cowork-core/src/agents/external_coding_agent.rs` | 外部编码 Agent 适配器 |

---

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| tools | 被依赖 | ACP 工具调用外部 Agent |
