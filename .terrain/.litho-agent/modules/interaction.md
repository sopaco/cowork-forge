# interaction 模块深度报告

## 这个模块在做什么

Interaction 是 Cowork Forge 的"翻译官"——它定义了系统内核与用户之间的沟通协议。内核只需要调用 `show_message()`、`request_input()` 等方法，具体的展示形式（命令行打印还是图形弹窗）由实现者决定。这种设计让同一套内核代码既能服务 CLI，也能服务 GUI 界面。

## 核心功能点

1. **InteractiveBackend trait**——定义了所有用户交互的接口：消息展示、输入请求、进度通知、流式推送。代码位置：`crates/cowork-core/src/interaction/mod.rs:108-160`
2. **CliBackend 实现**——基于 dialoguer 和 console 的 CLI 交互实现。代码位置：`crates/cowork-core/src/interaction/cli.rs`
3. **TauriBackend 实现**——基于 Tauri 事件系统的 GUI 交互实现。代码位置：`crates/cowork-core/src/interaction/tauri.rs`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `InteractiveBackend` trait | `crates/cowork-core/src/interaction/mod.rs:109` | 定义用户交互的统一接口 |
| `CliBackend` | `crates/cowork-core/src/interaction/cli.rs` | CLI 模式的交互实现 |
| `TauriBackend` | `crates/cowork-core/src/interaction/tauri.rs` | Tauri GUI 模式的交互实现 |
| `MessageContext` | `crates/cowork-core/src/interaction/mod.rs:51` | 消息上下文，包含 Agent 名称、消息类型和阶段信息 |
| `ProgressInfo` | `crates/cowork-core/src/interaction/mod.rs:101` | 长时间运行任务的进度信息 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| pipeline | 被依赖 | Pipeline 通过 InteractiveBackend 与用户交互 |
| cowork-cli | 实现 | CLI 创建 CliBackend 实例注入 Pipeline |
| cowork-gui | 实现 | GUI 创建 TauriBackend 实例注入 Pipeline |

## 跨模块协作场景

**在阶段执行过程中**：`IterationExecutor` 调用 `interaction.show_message()` 显示阶段开始 → Agent 执行过程中通过 `interaction.send_streaming()` 推送实时输出 → 关键决策点通过 `interaction.request_input()` 请求用户确认 → "确认通过"后继续执行。
