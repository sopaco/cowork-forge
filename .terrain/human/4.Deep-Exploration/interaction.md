# Interaction 领域

**模块路径**：`crates/cowork-core/src/interaction/`
**生成日期**：2026-07-05

---

## 概述

Interaction 是 Cowork Forge 的"翻译官"——它定义了内核与用户的沟通协议。系统内核只需要调用 `show_message()`、`request_input()` 等方法，具体的展示形式（命令行打印还是图形弹窗）由实现者决定。这让同一套内核代码既能服务 CLI 也能服务 GUI。

---

## 核心功能点

1. **InteractiveBackend trait**——用户交互的完整接口定义（消息、输入、进度、流式）。`crates/cowork-core/src/interaction/mod.rs:108-160`
2. **CliBackend 实现**——基于 dialoguer + console。`crates/cowork-core/src/interaction/cli.rs`
3. **TauriBackend 实现**——基于 Tauri 事件系统。`crates/cowork-core/src/interaction/tauri.rs`

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `InteractiveBackend` trait | `crates/cowork-core/src/interaction/mod.rs:109` | 用户交互统一接口 |
| `CliBackend` | `crates/cowork-core/src/interaction/cli.rs` | CLI 模式交互实现 |
| `TauriBackend` | `crates/cowork-core/src/interaction/tauri.rs` | Tauri GUI 模式实现 |
| `MessageContext` | `crates/cowork-core/src/interaction/mod.rs:51` | 消息上下文（Agent 名、消息类型、阶段） |

---

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| pipeline | 被依赖 | Pipeline 通过 InteractiveBackend 与用户交互 |
| cowork-cli | 实现 | CLI 创建 CliBackend |
| cowork-gui | 实现 | GUI 创建 TauriBackend |

---

## 跨模块协作场景

**在阶段执行中**：`IterationExecutor` → `interaction.show_message()` 显示阶段开始 → Agent 通过 `send_streaming()` 推送实时输出 → 关键决策点通过 `request_input()` 请求确认 → 确认通过后继续。
