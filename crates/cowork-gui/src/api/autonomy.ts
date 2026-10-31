import { invoke } from '@tauri-apps/api/core';
import type {
	AutonomyConfig,
	AutonomyStatusPayload,
	BudgetStatus,
	CycleSnapshot,
	DecisionLogEntry,
	DecisionOutcome,
	Idea,
	IdeaScope,
	IdeaSummary,
	IdeaRewrite,
	Effort,
	IdeaPriority
} from '../types/autonomy';

/** 自主模式 API —— 对应 Tauri 命令 autonomy::* */
export const autonomyApi = {
	/** 状态快照（状态机 + 配置 + 预算） */
	getStatus: () => invoke<AutonomyStatusPayload>('autonomy_get_status'),

	/** 开启自主循环 */
	start: () => invoke<AutonomyStatusPayload>('autonomy_start'),

	/** 停止自主循环 */
	stop: () => invoke<AutonomyStatusPayload>('autonomy_stop'),

	/** 暂停 */
	pause: () => invoke<AutonomyStatusPayload>('autonomy_pause'),

	/** 恢复 */
	resume: () => invoke<AutonomyStatusPayload>('autonomy_resume'),

	/** 熔断后的人工恢复 */
	resetTrip: () => invoke<AutonomyStatusPayload>('autonomy_reset_trip'),

	/** 更新配置 */
	updateConfig: (config: AutonomyConfig) =>
		invoke<AutonomyStatusPayload>('autonomy_update_config', { config }),

	/** 立即调研（跑一轮） */
	runCycle: () =>
		invoke<{ report: { snapshot: CycleSnapshot; published: IdeaSummary[]; skipped: string | null } }>(
			'autonomy_run_cycle'
		),

	/** 建议列表 */
	listIdeas: () => invoke<IdeaSummary[]>('autonomy_list_ideas'),

	/** 建议详情（含证据与决策历史） */
	getIdea: (ideaId: string) =>
		invoke<{ idea: Idea; timeline: DecisionLogEntry[] }>('autonomy_get_idea', { ideaId }),

	/** 决策：采纳 / 驳回 / 改写 / 置后 */
	decide: (
		ideaId: string,
		action: 'accept' | 'reject' | 'rewrite' | 'defer',
		comment: string,
		rewrite?: IdeaRewrite,
		deferredUntil?: string
	) =>
		invoke<DecisionOutcome>('autonomy_decide_idea', {
			request: {
				idea_id: ideaId,
				action,
				comment,
				rewrite: rewrite ?? null,
				deferred_until: deferredUntil ?? null
			}
		}),

	/** 撤销上一次决策 */
	undo: (ideaId: string) => invoke<IdeaSummary>('autonomy_undo_decision', { ideaId }),

	/** 用户主动输入一条建议 */
	propose: (input: {
		title: string;
		problem: string;
		hypothesis: string;
		scope: IdeaScope;
		estimated_effort: Effort;
		priority: IdeaPriority;
		expected_benefit?: string;
		evidence?: unknown[];
	}) =>
		invoke<IdeaSummary>('autonomy_propose_idea', {
			request: {
				title: input.title,
				problem: input.problem,
				hypothesis: input.hypothesis,
				scope: input.scope,
				estimated_effort: input.estimated_effort,
				priority: input.priority,
				expected_benefit: input.expected_benefit ?? '',
				evidence: input.evidence ?? []
			}
		}),

	/** 预算总览 */
	getBudget: () => invoke<BudgetStatus>('autonomy_get_budget'),

	/** 历史调研轮次 */
	listCycles: () => invoke<CycleSnapshot[]>('autonomy_list_cycles')
};

export default autonomyApi;
