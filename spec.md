# Spec — PM 自主规划环（Autonomous Mode）实现规格

> 状态：已实现（2026-10-31）｜ 依据：`autonomous-mode-design.md` §4–§12 ｜ 语言：Rust（cowork-core / cowork-gui src-tauri）+ TypeScript（React）
>
> 本文档描述**已落地**的实现。设计文档仍是方案来源，本文件是实现现状与验收口径。

## 0. 范围

在现有 7 阶段流水线**上方**新增「PM 自主规划环」：PM Agent 周期性调研项目 → 产出结构化 **Idea 提案** → 人在**决策门**采纳/驳回/改写/置后 → 被采纳的 Idea 复用现有 `Iteration` + `IterationExecutor` 进入既有流水线。**不新增第 8 个 stage，不改动 7 阶段本身。**

交付面：仅 GUI（`cowork-cli` 不新增命令）。核心业务逻辑全部在 `cowork-core`，GUI 侧只做薄壳转发 + 事件推送。

## 1. 领域层

| 事项 | 落点 | 要点 |
|------|------|------|
| Idea 聚合 | 新 `cowork-core/src/domain/idea.rs` | `Idea` / `IdeaDraft` / `IdeaSummary` / `IdeaEvidence` / `IdeaDecision` / `IdeaRewrite`；`IdeaScope`、`Effort`、`IdeaStatus`、`IdeaSource`、`Credibility`、`DecisionAction` 枚举 |
| 质量门槛 | `Idea::quality_issues()` | ① problem/hypothesis 非空且 hypothesis 可验证 ② **至少 1 条本地证据**（外部 URL 不能作唯一证据）③ scope 不越权（禁止绕过限流/路径校验/沙箱）④ effort 必填 |
| 去重指纹 | `compute_fingerprint()` | title+problem 规范化后 FNV-1a 64（自实现，跨进程确定）；同指纹合并、证据未变则状态抑制不重提 |
| 证据可信度 | `Evidence::credibility()` | 文件引用=High，迭代/反馈=Mid，外部=Low |

**`Iteration` 增加 `source_idea_id: Option<String>`**（采纳建议创建的迭代回填来源）。

## 2. 持久化层

| 文件 | 内容 | 说明 |
|------|------|------|
| `.cowork-v2/ideas/{idea_id}.json` | `Idea` 聚合 | 一 idea 一文件，版本化读写 |
| `.cowork-v2/autonomy/decisions.log.jsonl` | 决策审计 | **追加式**；撤销也追加一条 `reversed_by` 记录，绝不改写历史 |
| `.cowork-v2/autonomy/config.json` | `AutonomyConfig` | 自主模式唯一配置源 |
| `.cowork-v2/autonomy/state.json` | `AutonomyState` | 状态机 + 熔断计数 |
| `.cowork-v2/autonomy/budget.json` | `DailyBudget` | 每日周期数，跨天自动归零 |
| `.cowork-v2/autonomy/cycles/{cycle_id}.json` | `CycleSnapshot` | 调研轮次证据快照（增量调研依据） |

全部走 `persistence::schema` 的 `to_versioned_json` / `from_versioned_json`（`schema_version` 一次定型，无迁移垫片）。**被跳过的轮次不落盘**（宿主 30s 一次 tick，落盘会淹没历史）。

## 3. 自主循环（`cowork-core/src/autonomy/`）

```
controller.rs  AutonomyController —— 业务控制器（唯一决策落库实现）
cycle.rs       CycleSnapshot / CycleStep / CycleTrigger / CycleOutcome
budget.rs      BudgetGuard（单轮 LLM/MCP 记账）+ DailyBudget（每日周期）
research.rs    证据收集（Rust 侧确定性，不调 LLM）
config.rs      AutonomyConfig（schema 一次定型）
state.rs       AutonomyState + AutonomyStatus
```

**一轮 cycle 的流程**：

0. **状态机迁移**：`Idle → Researching`（落盘）`→ Proposing` → 结束按结果落 `AwaitingDecision`（有新建议）/ `Idle`（无新建议）/ `Tripped`（熔断）。调研中途崩溃残留的 `Researching`/`Proposing` 会在**进程启动时**由 `recover_stale_state()` 复位为 Idle（否则重入保护会让自主循环永久死锁）。`Researching`/`Proposing` 期间拒绝重入（`SkipReason::AlreadyRunning`）。
1. **前置检查** `should_trigger()`：熔断 → 暂停 → 已在跑 → 未启用 → 触发源未开 → 迭代执行中（让路）→ 当日预算耗尽 → 未到间隔。任一命中即跳过并返回可读原因（`SkipReason`）。
2. **收集证据** `collect_research()`：项目记忆（已知问题/近期决策）、未完成特性、阶段反馈、工作区文件清单与 TODO/FIXME 标记。全部带上限（文件 60、扫描 40、已知问题 12、反馈 12、TODO 20…）。
3. **组装 prompt**：`ResearchContext::to_prompt_with_budget()`，**硬性字符预算**（默认 12000），超限的条目丢弃并标注「…（其余 N 条从略）」。
4. **生成提案**：调用 `agents::run_autonomous_planner()`（adk-rust Agent，提示词 `instructions/autonomous_planner.rs`，工具 `propose_ideas` / `update_idea` + 只读工具 + 按需 MCP）。
5. **质量门槛 → 去重 → 状态抑制 → 限量 → 落库**：每轮最多发布 3 条（`MAX_PROPOSALS_PER_CYCLE`），超出的存为 `Draft` 池；同指纹合并；证据未变的被驳回建议不再重提。
6. **更新状态 + 熔断判定** + 落 cycle 快照 + 每日预算记账。

**熔断条件**（设计 §8-6）：

| 条件 | 阈值 | 动作 |
|------|------|------|
| 连续空转 | ≥3 轮 | Tripped |
| 单轮重复提案 | ≥3 条 | Tripped |
| 连续创建迭代失败 | ≥3 次 | Tripped |

Tripped 后需人工点「恢复」（`reset_trip()`）才继续。

**外部检索**（设计 §5.4）：仅当本地三处（已知问题/未完成特性/反馈/TODO）都查不到且策略为 `AsNeeded` 时才允许；无 MCP 服务时标记 `degraded` 并用本地证据继续，绝不因外查失败中断本轮。

## 4. 决策门

四类人工决策，**core 单点实现**（GUI 与 Agent 工具都调用同一套方法）：

| 动作 | 结果 | 审计 |
|------|------|------|
| 采纳 Accept | 创建迭代（`source_idea_id` 回填）+ 状态 `Converted` | 记录 `resulting_iteration_id` |
| 驳回 Reject | 状态 `Rejected` | 记录 |
| 改写 Rewrite | 保留原文（`original`）+ 状态 `Rewritten`（仍可继续决策） | 记录改写内容 |
| 置后 Defer | 状态 `Deferred` + `deferred_until` | 记录 |

**撤销**：`undo_decision()` 追加一条 `reversed_by` 记录，建议回到未决；已创建的迭代保留不删。

**转迭代规则**：有已完成的基线迭代 → `create_evolution` + `inheritance=Partial`（显式指定，不依赖 `Default`）；没有 → `create_genesis`（`inheritance=None`）。

**InheritanceMode 解析收敛**（设计 Phase 0.2）：`domain::InheritanceMode::parse()` 是唯一实现，GUI `iteration_commands.rs` 与 core `pm_tools.rs` 都改走它（此前两份 match 且默认值相反）。

## 5. GUI 层

**Tauri 命令**（`commands/autonomy.rs`，薄壳）：`autonomy_start/stop/pause/resume/get_status/reset_trip/update_config/run_cycle/list_ideas/get_idea/decide_idea/undo_decision/propose_idea/get_budget/list_cycles`。

**常驻宿主**（`autonomy_host.rs`）：30s tick；事件触发（迭代完成）优先，其次定时触发，再次空闲触发；迭代执行中让路；进度经 `CycleProgressSink` 推成 Tauri 事件。

**事件**：`autonomy_cycle_step` / `autonomy_cycle_snapshot` / `autonomy_cycle_report` / `autonomy_ideas_changed` / `autonomy_state_changed`。

**React**（`src/components/autonomy/`）：
- `AutonomyPanel` —— 状态胶囊 + 开关 + 「立即调研」+ 步骤进度 + 建议卡片（一行结论 / 一条证据位置 / 一行操作）+ 路线图归类 + 「我也有想法」入口。挂在迭代页顶部（导航融合，不新建孤立页面）。
- `IdeaDetailDrawer` —— 问题/假设/预期收益/全部证据原文/改写表单/决策历史时间线 + 四类决策 + 撤销。

**状态常驻**（`src/stores/autonomyStore.ts`）：自主模式的 status / ideas / cycles / 步骤进度放在**全局 zustand store**，Tauri 事件订阅是**应用级一次性绑定**（模块级 flag，永久存活）。原因：面板挂在迭代页，切 tab 会卸载组件 —— 组件内 state 会跟着消失，用户会看到"调研跑完了但建议列表空了"。`running` 取两个来源的或：本客户端发起的调用 + 后端状态机（researching/proposing）。调研中另有 2s 轮询兜底，防止事件丢失。

## 6. 验收口径

```bash
cargo check --workspace      # 0 error
cargo test --workspace       # 129 项全绿
cargo clippy --workspace     # 新增代码 0 warning
cd crates/cowork-gui && npx tsc --noEmit && npm run build
```

新增测试：idea 聚合 8 项、idea_store 3 项、autonomy_store 增量、budget 3 项、cycle 3 项、research 5 项、controller 11 项、planner 指令 3 项、idea_tools 4 项、execution_guard 3 项、iteration 继承解析 2 项。

## 7. 明确不做

- **CLI 接入**：CLI 不新增命令、不参与验收（设计 D 范围裁定）。
- **`propose_roadmap` 独立路线图产物**：面板里的「路线图」是从真实建议派生出的归类视图，不是 Agent 生成的独立路线图文档。
- **自动采纳**：本期不自动采纳（D6=A）；`auto_create_iteration` 只控制低风险范围是否**顺手启动**已创建的迭代，且采纳动作本身始终需要人点。
