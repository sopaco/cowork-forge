# Spec — 自主模式 Phase 0 前置改造

> 状态：待审批 ｜ 依据：autonomous-mode-design.md §11/§12 路线 Phase 0 + 2026-10-31 源码实读 ｜ 语言：Rust（cowork-core / cowork-gui src-tauri）

## 0. 目标与范围

只做四件事，**不含 Phase 1**（Idea 聚合、提案、调研流程一律不做）：

| # | 事项 | 主要落点 |
|---|------|---------|
| T1 | PM 提示词工具名与注册名对齐 + 语言一致性 | `cowork-core/src/instructions/`、`cowork-core/src/agents/mod.rs` |
| T2 | 存储层 `schema_version` 一次定型 + 读取时版本校验（无迁移垫片） | `cowork-core/src/persistence/`、`cowork-core/src/data/models.rs` |
| T3 | 全局单例执行互斥（同一时刻仅一个迭代执行，冲突报可读错误） | 新 `cowork-core/src/execution_guard.rs` + executor + PM 工具 |
| T4 | GUI 自主循环宿主骨架 + `AutonomyConfig` 最小加载 | 新 `cowork-core/src/autonomy/` + `cowork-gui/src-tauri/src/` |

**不做**：git commit；前端 React 改动（T4 仅 Tauri 后端，不触发 `npm run build`）；InheritanceMode 解析收敛（设计表 0.2，不在用户四项清单内）；Idea/IdeaStore（Phase 1）。

**约束**：hexagonal（core 不依赖 Tauri/clap/dialoguer 新增）、统一 `anyhow::Result`、生产代码禁 `unwrap()`、不硬编码 stage ID、不绕过限流/路径校验、不改动无关代码（工作区仅有 `.terrain/.meta/freshness.json` 与 `autonomous-mode-design.md` 为既有未提交内容，均不动）。

---

## 1. 已核实现状（改动依据）

- PM 注册工具名 7 个：`pm_goto_stage`/`pm_create_iteration`/`pm_respond`/`pm_save_decision`/`query_memory`/`list_files`/`read_file`（[agents/mod.rs L98-L119](crates/cowork-core/src/agents/mod.rs)），映射同名于 agent_factory.rs L475-478；`read_file_truncated` 亦有注册（agent_factory.rs L424）→ pm_agent.json 工具清单无失配。
- 提示词失配点（instructions/project_manager.rs）：L148 `respond_to_user`（应为 `pm_respond`）、L138 `## goto_stage`、L142 `## create_iteration`（应为 `pm_goto_stage`/`pm_create_iteration`）、L119 示例 `[调用 goto_stage 工具]`。语言混杂点：提示词正文中文，而 agents/mod.rs L283-305 拼接 prompt 为英文（"User message:"、"## Current Iteration Info:"、"Please analyze..." 等），且 L157 硬编码"使用中文"与 `config::get_language_instruction()`（config.rs L21，按 locale 下发语言指令）语义冲突。
- persistence 9 类 JSON 均裸 `serde_json::from_str/to_string_pretty` 读写；`IterationStore::load_all`（iteration_store.rs L72-77）静默跳过坏文件；data/models.rs 中 `Requirements/FeatureList/DesignSpec/ImplementationPlan/CodeMetadata` 各带 `schema_version: String = "1.0"` 字段但从无校验、无读取方（全仓 grep 确认仅 models.rs 自身引用）。
- 执行全局单例：`CURRENT_ITERATION_ID`（persistence/iteration_data.rs L12-30）、`GOTO_STAGE_SIGNAL`（pipeline/mod.rs L18-34）、`CURRENT_EXECUTION_LLM`（llm/mod.rs L12-27）；三者均 `.lock().unwrap()`（违反 AGENTS.md 禁 unwrap，本次触碰时一并修为 poison 容错）。执行唯一入口 `IterationExecutor::execute`（pipeline/executor/mod.rs L81-103）；`continue_iteration`/`retry_iteration`（L545/L585）末尾委托 `self.execute` → 在 `execute` 加锁即可全覆盖。`PMGotoStageTool::execute` 调 `set_iteration_id`（覆盖单例）、`PMCreateIterationTool::execute` 并发写 project.json。
- GUI：invoke_handler（lib.rs L834-941）注册命令；全局单例先例 commands/mod.rs `lazy_static`；事件先例 `window.emit("iteration_failed", ...)`（iteration_commands.rs L216-227）；TauriBackend 用 `app_handle.emit`。

---

## 2. 变更规格

### T2 存储层 schema_version（先做，被 T4 复用）

**新文件 `crates/cowork-core/src/persistence/schema.rs`**：
- `pub const SCHEMA_VERSION: u32 = 1;`
- `pub fn to_versioned_json<T: Serialize>(value: &T) -> anyhow::Result<String>`：序列化为 pretty JSON 对象并在**顶层**插入 `"schema_version": 1`（扁平信封，不包裹 `data` 字段，保持现有 JSON 风格）。
- `pub fn from_versioned_json<T: DeserializeOwned>(content: &str, source: &str) -> anyhow::Result<T>`：先解析为 `serde_json::Value`，校验顶层 `schema_version`：
  - 缺失 → `Err`：`"{source}: missing schema_version field (expected 1)"`；
  - 非当前版本（含非数字）→ `Err`：`"{source}: unsupported schema_version {v} (this build only supports 1); automatic migration is not supported"`；
  - 校验通过后 **remove** 该键再反序列化 payload。**不做任何迁移/降级/serde default**。
- 单测（同文件 `#[cfg(test)]`）：往返（to→from 相等）、缺版本报错、错版本报错且信息含 source 与版本号、非对象报错。

**修改（读写全部改走上述 helper）**：
- `persistence/mod.rs`：`pub mod schema;` + re-export；`init_project_structure` 末尾创建 `autonomy/` 目录并写入初始 `autonomy/config.json` + `autonomy/state.json`（若不存在才写，见 T4）。
- `persistence/project_store.rs`：`load/save` ↔ `project.json`。
- `persistence/iteration_store.rs`：`load/save` ↔ `iterations/{id}.json`；`load_all` **改为显式报错**（`?` + 文件路径 context），不再静默跳过坏文件。
- `persistence/memory_store.rs`：project/iteration memory 的 load/save 及两处默认 memory 落盘。
- `persistence/iteration_data.rs`：`requirements/feature_list/design_spec/implementation_plan/code_metadata/session meta/feedback` 共 7 组 load/save。
- `data/models.rs`：**删除** 5 个模型的 `schema_version: String` 字段及构造函数里的赋值（无读取方；保留会与顶层 `schema_version` 冲突，且属"两套版本语义"）。

行为变化（有意为之）：旧的无版本文件读取时报错而非兼容——符合"无旧版本兼容、明确报错"裁定。

### T3 执行互斥

**新文件 `crates/cowork-core/src/execution_guard.rs`**（lib.rs 注册 `pub mod execution_guard;`）：
- 静态 `EXECUTION_OWNER: Mutex<Option<ExecutionOwner{holder, iteration_id}>>`，锁获取统一 `unwrap_or_else(PoisonError::into_inner)`（线程/任务边界一致，panic 不致死锁）。
- `pub struct ExecutionGuard`（RAII）：`try_acquire(holder: &str, iteration_id: &str) -> anyhow::Result<Self>`；冲突时错误信息含在跑的 iteration 与 holder："execution already in progress: iteration '{id}' is being executed by '{holder}'; only one iteration may execute at a time…"；`Drop` 释放（仅当 owner 仍是自己）。
- `pub fn is_execution_in_progress() -> bool`、`pub fn current_execution() -> Option<(String, String)>`（供自主循环让路/状态查询）。
- 单测（模块内 `static TEST_LOCK` 串行化）：二次 acquire 报错且信息可读；drop 后可再 acquire；多线程 barrier 下恰好一个成功。

**修改**：
- `pipeline/executor/mod.rs` `IterationExecutor::execute`：函数体首行 `let _guard = ExecutionGuard::try_acquire("pipeline", iteration_id)?;`（覆盖 continue/retry）。
- `tools/pm_tools.rs`：`PMGotoStageTool::execute`（在 `set_iteration_id` 之前）与 `PMCreateIterationTool::execute` 开头 acquire（holder `"pm_agent"`，iteration_id 用 `self.current_iteration_id`）；冲突时返回工具 JSON `{"status":"error","message":...}`（与现有 invalid stage 的返回风格一致），**不覆盖单例状态**。
- `persistence/iteration_data.rs` / `pipeline/mod.rs` / `llm/mod.rs`：触碰到的 `.lock().unwrap()` 一并改 poison 容错（不改语义）。

### T1 PM 提示词修复

**`instructions/project_manager.rs`**（保持中文单一语言）：
- L148 `## respond_to_user` → `## pm_respond`；L138 `## goto_stage` → `## pm_goto_stage`；L142 `## create_iteration` → `## pm_create_iteration`；L119 示例 → `[调用 pm_goto_stage 工具]`。
- §2 操作标题绑定真实工具名：`### pm_goto_stage - 返回之前阶段`、`### pm_create_iteration - 创建新迭代`、`### pm_respond - 回答问题`、`### pm_respond - 请求澄清`（保留 answer_question/ask_clarification 作为用法说明词）。
- L157 `4. **使用中文**：…` → 改为遵循消息末尾附加的语言指令（消除与 `get_language_instruction()` 的冲突）；其余英文标识符（bug_fix 等枚举值、文件名）保留。
- 新增单测（同文件）：指令中不出现 `respond_to_user`/裸 `goto_stage`/裸 `create_iteration` 工具名；出现 `pm_respond`/`pm_goto_stage`/`pm_create_iteration`。

**`agents/mod.rs`**（execute_pm_agent_message_streaming 及 load_artifacts_summary_for_pm）：拼接 prompt 的英文模板/标签译为中文（"用户消息："、"## 当前迭代信息："、"请分析用户的请求…"、"## 历史决策："、"## 对话历史："、"## 项目文件："、角色标签"用户/助手"、"[已截断]"），消除中英混杂；不改逻辑。

### T4 GUI 自主循环宿主骨架

**core 侧（只放接口 + 领域类型 + 生命周期状态机，业务逻辑不进 GUI）**：

- 新 `crates/cowork-core/src/autonomy/mod.rs`（`pub mod config; pub mod state; pub mod controller;` + re-export；lib.rs 注册）。
- `autonomy/config.rs`：`AutonomyConfig { enabled: bool, triggers: AutonomyTriggers{on_iteration_completed, on_idle, on_schedule: bool, schedule_interval_mins: u64}, research: AutonomyResearchConfig{local, memory, feedback: bool, external_mcp: ExternalMcpPolicy(AsNeeded|Off)}, budget: AutonomyBudget{max_llm_calls_per_cycle: u32, max_cycles_per_day: u32, max_mcp_calls_per_cycle: u32}, auto_create_iteration: bool }`（即设计 §4.1 schema 一次定型，全部 serde derive，**无 serde default 垫片**）。`Default`：`enabled=false`、triggers 全 false、`schedule_interval_mins=60`、research local/memory/feedback=true + `external_mcp=AsNeeded`、budget `10/20/3`、`auto_create_iteration=false`（D2/D6）。
- `autonomy/state.rs`：`AutonomyStatus`（§4.2 全集枚举，snake_case）：`Disabled/Idle/Researching/Proposing/AwaitingDecision/Executing/Paused/Tripped`；`AutonomyState { status, cycle_count: u64, last_cycle_at: Option<DateTime<Utc>>, tripped_reason: Option<String> }`。
- `autonomy/controller.rs`：`AutonomyController` 生命周期接口（`anyhow::Result`）：`enable()`（→Idle，写 config.enabled=true）、`pause()`（Idle→Paused，否则可读错误）、`resume()`（Paused→Idle）、`stop()`（任意→Disabled，config.enabled=false）、`status()`；迁移校验错误信息含当前状态。本期不实现 run_cycle/decide_idea（Phase 1）。单测：迁移全链 + 非法迁移报错 + config/state 落盘往返（tempfile + `set_workspace_path`，模块内 TEST_LOCK 串行）。
- 新 `crates/cowork-core/src/persistence/autonomy_store.rs`：`load_config/save_config/load_state/save_state` ↔ `.cowork-v2/autonomy/{config,state}.json`，走 T2 版本化 helper；文件缺失→返回 `Default`（创建语义，非兼容垫片）；版本不明→报错。注册进 persistence/mod.rs。

**GUI 侧（Tauri 后端薄壳，无业务逻辑）**：

- 新 `crates/cowork-gui/src-tauri/src/autonomy_host.rs`：`AutonomyHost`（`Mutex<HostInner{ join_handle, stop_tx: watch::Sender<bool>, paused: Arc<AtomicBool> }>`）：`start(app_handle)`（已在运行→可读错误；`tokio::spawn` 常驻循环）、`stop()`（watch 通知退出 + 回收句柄）、`pause()/resume()`（AtomicBool）、`is_running()/is_paused()`。循环骨架：`tokio::time::interval(30s)` + `select!` 停止信号；每 tick：暂停则跳过；`cowork_core::execution_guard::is_execution_in_progress()` 为真则**让路**（debug 日志，D5 执行优先）；循环体留空并注释标注 Phase 1 接入点。
- 新 `crates/cowork-gui/src-tauri/src/commands/autonomy.rs`：Tauri 命令 `autonomy_start/autonomy_stop/autonomy_pause/autonomy_resume/autonomy_get_status`；每个变更命令先调 core `AutonomyController` 再驱动 `AUTONOMY_HOST`，成功后 `window.emit("autonomy_state_changed", state)`；`autonomy_get_status` 返回 `{ state, config, host_running, host_paused }`（serde struct）。冲突/非法迁移错误 `map_err(|e| e.to_string())` 可读返回。
- `commands/mod.rs`：`pub mod autonomy;` + `lazy_static! AUTONOMY_HOST`（沿用 PROJECT_RUNNER 模式）。
- `lib.rs`：`mod autonomy_host;`、use 列表加 `autonomy`、invoke_handler 注册 5 个命令。

---

## 3. 验证（必须真实执行，结果写入交付摘要）

```bash
cargo check --workspace          # 0 error / 0 warning 新增
cargo test --workspace           # 含新增单测全绿（至少 cowork-core；GUI 测试随 workspace 跑）
cargo clippy --workspace         # 0 warning（新代码不引入 lint）
```

新增测试清单：schema 4 项、execution_guard 3 项、autonomy controller/store ≥4 项、PM 指令工具名 1 项。前端零改动 → 不跑 `npm run build`。任何失败修到通过再交付。

## 4. 交付物

中文 Markdown 变更摘要（结论先行）：每项改了什么、涉及文件、三项验证的真实输出结论、遗留到 Phase 1 的事项。
