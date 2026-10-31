# Tasks — 自主模式 Phase 0 前置改造

> 执行顺序：T2 → T3 → T1 → T4 → V（依赖：T4 的 autonomy_store 复用 T2 helper；V 贯穿）。每项完成即跑 `cargo check -p cowork-core` 快验，最后统一全量验证。

## T2 存储层 schema_version 一次定型
- [x] 2.1 新建 `crates/cowork-core/src/persistence/schema.rs`：`SCHEMA_VERSION=1`、`to_versioned_json`、`from_versioned_json`（缺版本/错版本/非对象 → 可读 `Err`，无迁移）+ 4 项单测。
- [x] 2.2 `persistence/mod.rs` 注册模块并 re-export。
- [x] 2.3 `project_store.rs` / `iteration_store.rs` / `memory_store.rs` / `iteration_data.rs` 全部 load/save 改走版本化 helper；`iteration_store.rs::load_all` 坏文件由静默跳过改为显式报错（带文件路径）。
- [x] 2.4 `data/models.rs` 删除 5 个模型的 `schema_version: String` 字段及构造赋值。
- **验收**：单测全绿；`grep -rn "serde_json::from_str\|to_string_pretty" crates/cowork-core/src/persistence` 中 JSON 文档读写均经 helper（memory 默认落盘也覆盖）。

## T3 执行互斥 ExecutionGuard
- [x] 3.1 新建 `crates/cowork-core/src/execution_guard.rs`：`ExecutionGuard::try_acquire`/`Drop`、`is_execution_in_progress`、`current_execution`；poison 容错锁；冲突错误含 holder+iteration_id + 3 项单测（含多线程 barrier）。
- [x] 3.2 `pipeline/executor/mod.rs::execute` 首行 acquire（holder=`"pipeline"`，覆盖 continue/retry）。
- [x] 3.3 `tools/pm_tools.rs`：`PMGotoStageTool`/`PMCreateIterationTool` acquire（holder=`"pm_agent"`），冲突返回工具 JSON error，不覆盖 `set_iteration_id`。
- [x] 3.4 触碰的三处 `.lock().unwrap()`（iteration_data.rs / pipeline/mod.rs / llm/mod.rs）改 poison 容错。
- **验收**：并发单测（第二执行被拒且信息可读、释放后可重入）；GUI/CLI 路径无需改动即受保护。

## T1 PM 提示词工具名与语言一致性
- [x] 1.1 `instructions/project_manager.rs`：`respond_to_user`→`pm_respond`、`goto_stage`→`pm_goto_stage`、`create_iteration`→`pm_create_iteration`（§2 操作标题、§5 示例、工具使用规则三处）；L157 语言条目改为遵循文末语言指令；新增工具名一致性单测。
- [x] 1.2 `agents/mod.rs`：拼接 prompt 的英文模板/标签译为中文（逻辑不动）。
- **验收**：单测断言指令不含 `respond_to_user`/裸 `goto_stage`/`create_iteration`，含三个 `pm_*` 注册名；pm_agent.json 与 agent_factory 注册名已核对一致（无需改动）。

## T4 GUI 自主循环宿主骨架 + AutonomyConfig 最小加载
- [ ] 4.1 core `autonomy/`：`config.rs`（§4.1 schema + Default）、`state.rs`（AutonomyStatus 全集 + AutonomyState）、`controller.rs`（enable/pause/resume/stop/status 状态机 + 可读非法迁移错误）+ 迁移/往返单测。
- [ ] 4.2 `persistence/autonomy_store.rs`：config.json/state.json 版本化读写，缺失→Default；`init_project_structure` 创建 `autonomy/` 并写初始 config/state（仅缺失时写）。
- [ ] 4.3 GUI `autonomy_host.rs`：常驻 tokio task 生命周期（start/stop/pause/resume、watch 停止信号、30s tick）；tick 暂停跳过、执行互斥让路（`is_execution_in_progress`）；循环体留 Phase 1 接入注释。
- [ ] 4.4 GUI `commands/autonomy.rs` + `commands/mod.rs`（lazy_static AUTONOMY_HOST）+ `lib.rs` 注册 5 命令：`autonomy_start/stop/pause/resume/get_status`；变更后 emit `autonomy_state_changed`。
- **验收**：命令层仅转发 core API + 事件推送，无业务逻辑；`cargo check -p cowork-gui` 通过。

## V 全量验证（真实执行，结论入交付摘要）
- [ ] 5.1 `cargo check --workspace`
- [ ] 5.2 `cargo test --workspace`（新增测试全部通过）
- [ ] 5.3 `cargo clippy --workspace`（无新增 warning）
- [ ] 5.4 中文变更摘要（结论先行：每项改动/文件/验证真实结论/Phase 1 遗留）

## 约束核对（交付前逐项确认）
- [ ] 无 git commit；未动工作区既有改动（.terrain/.meta/freshness.json、autonomous-mode-design.md、spec.md、tasks.md 除外）。
- [ ] 新代码无 `unwrap()`（锁用 poison 容错）、统一 `anyhow::Result`、未硬编码 stage ID、未绕过限流/路径校验、core 无 Tauri/clap/dialoguer 新增依赖。
