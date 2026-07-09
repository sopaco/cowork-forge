# integration 模块深度报告

## 这个模块在做什么

Integration 是 Cowork Forge 的"API 网关"——它允许在流水线执行过程中触发外部系统调用。比如，当 Delivery 阶段完成后，自动触发一个 Webhook 通知部署平台开始部署，或者将 PRD 内容同步到需求管理工具。

## 核心功能点

1. **Hook 管理器**——`HookManager` 在特定执行点（阶段完成、失败等）触发预配置的回调。代码位置：`crates/cowork-core/src/integration/hooks.rs`
2. **REST Adapter**——`RestAdapter` 实现标准的 REST API 调用，支持 POST 请求和认证配置。代码位置：`crates/cowork-core/src/integration/rest_adapter.rs`
3. **Integration 定义**——配置集成类型、连接参数、认证方式和触发事件。代码位置：`crates/cowork-core/src/integration_definition.rs`（config_definition 模块）

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `HookManager` | `crates/cowork-core/src/integration/hooks.rs` | 管理执行钩子，在特定事件点触发外部操作 |
| `RestAdapter` | `crates/cowork-core/src/integration/rest_adapter.rs` | REST API 调用适配器 |
| `IntegrationAdapter` trait | `crates/cowork-core/src/integration/adapters.rs` | 各种集成适配器的统一接口 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| pipeline | 被依赖 | Pipeline 在阶段执行前后触发 Hook |
| config_definition | 依赖 | 从 ConfigRegistry 获取集成配置 |

## 跨模块协作场景

**在阶段完成时**：Pipeline 执行完一个阶段后 → 检查 ConfigRegistry 中是否有对应阶段的 Hook 配置 → 如果有，调用 HookManager 执行 → HookManager 根据配置调用 RestAdapter（或其他适配器）→ 外部系统收到通知 → Pipeline 继续执行。
