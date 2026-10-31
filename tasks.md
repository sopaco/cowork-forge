# Tasks — PM 自主规划环（Autonomous Mode）

> 依据：`autonomous-mode-design.md` §12 分阶段路线。执行顺序：Phase 0 → Phase 1 → Phase 2 → Phase 3。
> 每项完成即跑 `cargo check -p cowork-core` 快验，最后统一全量验证。

## Phase 0 —— 前置改造（阻塞项）

- [x] 0.1 PM 提示词工具名对齐（`respond_to_user` → `pm_respond` 等）+ 语言一致性
- [x] 0.2 InheritanceMode 解析收敛为 core 单函数（`domain::InheritanceMode::parse`）
- [x] 0.3 执行互斥 `ExecutionGuard`（pipeline + PM 工具 + 自主创建迭代）
- [x] 0.4 `Idea` / `IdeaDecision` / `IdeaStore` 落地，schema 一次定型
- [x] 0.5 `AutonomyConfig` / `AutonomyState` 聚合与 `autonomy/` 存储

## Phase 1 —— MVP（最小可验证闭环）

- [x] 1.1 `domain/idea.rs`：Idea 聚合 + 质量门槛 + 去重指纹 + 可信度
- [x] 1.2 `persistence/idea_store.rs`：一 idea 一文件 + `decisions.log.jsonl` 追加审计
- [x] 1.3 `autonomy/budget.rs`：`BudgetGuard`（单轮）+ `DailyBudget`（每日，跨天归零）
- [x] 1.4 `autonomy/research.rs`：确定性证据收集（记忆/特性/反馈/工作区/TODO），带上限
- [x] 1.5 `instructions/autonomous_planner.rs` + `agents/research_agent.rs`：规划 Agent
- [x] 1.6 `tools/idea_tools.rs`：`propose_ideas` / `update_idea`（只校验，不落库）
- [x] 1.7 `autonomy/controller.rs::run_cycle`：前置检查 → 证据 → 提案 → 门槛/去重/限量 → 落库
- [x] 1.8 决策门：`decide_idea`（采纳/驳回/改写/置后）+ `undo_decision` + `create_iteration_from_idea`
- [x] 1.9 `Iteration.source_idea_id` 回填
- [x] 1.10 GUI：`autonomy_run_cycle` + 建议列表/详情/决策命令 + 事件推送
- [x] 1.11 React：`AutonomyPanel` + `IdeaDetailDrawer`，挂载在迭代页

## Phase 2 —— 事件驱动 + 体验完善

- [x] 2.1 迭代完成事件触发（`notify_iteration_finished` → 宿主 `request_cycle`）
- [x] 2.2 外部检索按需 + 降级（无 MCP 服务 → `degraded` + 本地证据继续）
- [x] 2.3 评分与冲突标记（`conflicts_with` 双向标出，界面提示二选一）
- [x] 2.4 熔断完整实现（连续空转 / 单轮重复提案 / 连续创建迭代失败）+ 人工恢复
- [x] 2.5 提示词长度预算（`to_prompt_with_budget` + 超限截断单测）
- [x] 2.6 通知分级：静默更新 + 步骤进度事件（不做假百分比进度条）

## Phase 3 —— 完整形态

- [x] 3.1 定时触发（宿主 30s tick + `schedule_interval_mins`）
- [x] 3.2 空闲触发（`on_idle`，与定时共用间隔判定）
- [x] 3.3 路线图视图（从真实建议派生归类，非独立路线图产物）
- [x] 3.4 低风险自动启动（`auto_create_iteration` + 文档/调研范围，默认关，双门禁）

## V 全量验证（真实执行）

- [x] V.1 `cargo check --workspace` —— 0 error
- [x] V.2 `cargo test --workspace` —— 129 项全绿
- [x] V.3 `cargo clippy --workspace` —— 新增代码 0 warning
- [x] V.4 `npx tsc --noEmit` —— 0 error
- [x] V.5 `npm run build` —— 构建通过

## 遗留（明确不做）

- CLI 接入（设计裁定：仅 GUI 交付）
- `propose_roadmap` 独立路线图产物
- 自动采纳（D6=A：本期不自动采纳）
