# config_definition 模块深度报告

## 这个模块在做什么

ConfigDefinition 是 Cowork Forge 的"规章制度手册"——它把原来写在代码里的 Agent 定义、阶段流程、集成配置等"规定"，变成了可以随时修改的配置文件。以前想加一个新的 Agent 角色需要改 Rust 代码重新编译，现在只需要写一个 JSON 配置文件就能注册。

## 核心功能点

1. **Agent 定义**——`AgentDefinition` 结构体定义了 Agent 的角色、指令、工具集和模型参数，支持内置指令和文件指令两种方式。代码位置：`crates/cowork-core/src/config_definition/agent_definition.rs`
2. **Stage 定义**——`StageDefinition` 配置每个阶段的执行方式（Simple/ActorCritic）、Hook、制品模板和质量标准。代码位置：`crates/cowork-core/src/config_definition/stage_definition.rs`
3. **Flow 定义**——`FlowDefinition` 定义自定义开发流程的阶段组合和执行顺序，支持继承模式配置。代码位置：`crates/cowork-core/src/config_definition/flow_definition.rs`
4. **ConfigRegistry**——全局配置注册表，集中管理所有定义，提供查询、验证和生命周期管理。代码位置：`crates/cowork-core/src/config_definition/registry.rs:41-60`
5. **Agent Factory**——`create_agent_for_stage()` 和 `create_agent_from_config()` 根据配置定义动态创建 Agent 实例。代码位置：`crates/cowork-core/src/config_definition/agent_factory.rs`
6. **MCP 工具集初始化**——支持通过 Model Context Protocol 集成远程 MCP 服务器工具。代码位置：`crates/cowork-core/src/config_definition/agent_factory.rs`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `AgentDefinition` | `crates/cowork-core/src/config_definition/agent_definition.rs` | 定义 Agent 角色、指令、工具和模型参数 |
| `StageDefinition` | `crates/cowork-core/src/config_definition/stage_definition.rs` | 定义阶段的执行方式和 Hook |
| `FlowDefinition` | `crates/cowork-core/src/config_definition/flow_definition.rs` | 定义自定义开发流程的阶段序列 |
| `ConfigRegistry` | `crates/cowork-core/src/config_definition/registry.rs:41` | 全局配置注册表，管理的所有定义集合 |
| `ConfigValidator` | `crates/cowork-core/src/config_definition/validator.rs` | 验证配置定义的完整性和正确性 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| pipeline | 被依赖 | Pipeline 查询 Flow 和 Stage 配置 |
| agents | 被依赖 | Agent Factory 根据 AgentDefinition 构建 Agent |
| tools | 被依赖 | MCP 工具集工具注册 |

## 跨模块协作场景

**在流水线执行中**：`IterationExecutor` 调用 `get_stages_from_flow()` 从 `ConfigRegistry` 获取当前流程的阶段定义 → 对每个阶段调用 `create_agent_for_stage()` 根据 `AgentDefinition` 创建 Agent → Agent 执行过程中使用注册的工具集 → 执行结束后检查是否有 Hook 配置需要触发外部集成。
