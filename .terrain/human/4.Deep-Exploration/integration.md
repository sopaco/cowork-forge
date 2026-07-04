# Integration 领域

**模块路径**：`crates/cowork-core/src/integration/`
**生成日期**：2026-07-05

---

## 概述

Integration 是 Cowork Forge 的"API 网关"——它允许在流水线执行过程中触发外部系统调用。比如 Delivery 阶段完成后自动触发部署 Webhook，或将 PRD 同步到需求管理工具。

---

## 核心功能点

1. **Hook 管理器**——`HookManager` 在阶段完成/失败等点触发回调。`crates/cowork-core/src/integration/hooks.rs`
2. **REST Adapter**——`RestAdapter` 实现 REST API 调用（POST + 认证）。`crates/cowork-core/src/integration/rest_adapter.rs`
3. **Integration 定义**——配置集成类型、连接参数、认证方式和触发事件（在 `config_definition` 模块中）

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `HookManager` | `crates/cowork-core/src/integration/hooks.rs` | 执行钩子管理 |
| `RestAdapter` | `crates/cowork-core/src/integration/rest_adapter.rs` | REST API 调用适配器 |
| `IntegrationAdapter` trait | `crates/cowork-core/src/integration/adapters.rs` | 集成适配器统一接口 |
