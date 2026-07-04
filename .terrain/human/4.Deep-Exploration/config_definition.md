# ConfigDefinition 领域

**模块路径**：`crates/cowork-core/src/config_definition/`
**生成日期**：2026-07-05

---

## 概述

ConfigDefinition 是 Cowork Forge 的"规章制度手册"。它把原来硬编码在 Rust 代码里的 Agent 定义、阶段流程、集成配置等"内部规定"，变成了可以通过 JSON 文件随时修改的外部配置。以前想添加一个新 Agent 角色需要修改代码重新编译，现在只需要写一个配置文件就能注册。

这是从"固定框架"到"可配置平台"的关键架构转变。ConfigRegistry 就像"公司的规章制度登记处"——所有定义（Agent、Stage、Flow、Integration）都在这里注册、查询和管理。

---

## 核心功能点

1. **Agent 定义**——`AgentDefinition` 定义 Agent 的角色、指令来源、工具集和模型参数
2. **Stage 定义**——`StageDefinition` 配置阶段的执行方式（Simple/ActorCritic）、Hook 点和制品模板
3. **Flow 定义**——`FlowDefinition` 定义自定义流程的阶段组合和顺序
4. **ConfigRegistry**——全局注册表，提供查询、验证和生命周期管理
5. **Agent Factory**——`create_agent_for_stage()` 根据配置动态创建 Agent

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `AgentDefinition` | `crates/cowork-core/src/config_definition/agent_definition.rs` | 定义 Agent 角色、指令、工具和模型参数 |
| `StageDefinition` | `crates/cowork-core/src/config_definition/stage_definition.rs` | 定义阶段的执行方式和 Hook |
| `FlowDefinition` | `crates/cowork-core/src/config_definition/flow_definition.rs` | 定义自定义流程的阶段序列 |
| `ConfigRegistry` | `crates/cowork-core/src/config_definition/registry.rs:41` | 全局注册表，所有定义的管理中心 |
| `ConfigValidator` | `crates/cowork-core/src/config_definition/validator.rs` | 验证配置完整性 |

---

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| pipeline | 被依赖 | Pipeline 查询 Flow 和 Stage 配置 |
| agents | 被依赖 | Agent Factory 根据 AgentDefinition 构建 Agent |

---

## 跨模块协作场景

**在流水线执行中**：`IterationExecutor` → `get_stages_from_flow()` 查询 `ConfigRegistry` → `create_agent_for_stage()` 根据 `AgentDefinition` 创建 Agent → Agent 执行 → 检查 Hook 配置触发外部集成。
