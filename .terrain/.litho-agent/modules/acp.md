# acp 模块深度报告

## 这个模块在做什么

ACP 模块是 Cowork Forge 的"外交部门"——它负责与系统外部的 AI Agent 通信。当内置的 Coding Agent 不够用时，可以通过 ACP（Agent Client Protocol）协议调用外部的编码 Agent（如 OpenCode、Gemini CLI、Claude CLI 等）来完成编码任务。

## 核心功能点

1. **ACP 客户端**——实现 Agent Client Protocol，支持 stdio 和 WebSocket 两种传输方式。代码位置：`crates/cowork-core/src/acp/client.rs`
2. **外部编码 Agent 集成**——支持 OpenCode、iFlow、Codex、Gemini CLI、Claude CLI 等多种外部 Agent。代码位置：`crates/cowork-core/src/agents/external_coding_agent.rs`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `AcpClient` | `crates/cowork-core/src/acp/client.rs` | ACP 协议客户端，管理外部 Agent 连接 |
| `AcpTaskResult` | `crates/cowork-core/src/acp/mod.rs` | ACP 任务执行结果 |
| `ExternalCodingAgent` | `crates/cowork-core/src/agents/external_coding_agent.rs` | 外部编码 Agent 适配器 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| tools | 被依赖 | ACP 工具调用外部 Agent |

## 跨模块协作场景

**在编码阶段配置了外部 Agent 时**：Coding Stage 检查配置 → 如果有 `[coding_agent]` 配置 → 使用 `ExternalCodingAgent` 替代内置 Coding Loop → 通过 `AcpClient` 与外部 Agent 通信 → 外部 Agent 执行编码任务 → 结果返回给 Coding Stage。
