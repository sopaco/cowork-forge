import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';
import autonomyApi from '../api/autonomy';
import type {
	AutonomyConfig,
	AutonomyStatusPayload,
	CycleSnapshot,
	CycleStep,
	DecisionLogEntry,
	DecisionOutcome,
	Idea,
	IdeaRewrite,
	IdeaScope,
	IdeaSummary,
	Effort,
	IdeaPriority
} from '../types/autonomy';

/** 提示消息类型 */
export type NotifyKind = 'success' | 'info' | 'warning' | 'error';
export type Notify = (kind: NotifyKind, text: string) => void;

export interface IdeaDetail {
	idea: Idea;
	timeline: DecisionLogEntry[];
}

interface AutonomyStoreState {
	status: AutonomyStatusPayload | null;
	ideas: IdeaSummary[];
	cycles: CycleSnapshot[];
	/** 本客户端发起的调研是否仍在等待结果 */
	localRunning: boolean;
	/** 当前调研步骤（收集证据 / 生成提案 / 质量门槛） */
	steps: CycleStep[];
	/** 最近一轮的人话结论 */
	lastNote: string | null;
	/** 由 UI 注入的提示函数（store 不能用 hook） */
	notify: Notify;

	setNotify: (notify: Notify) => void;
	refresh: () => Promise<void>;
	refreshStatus: () => Promise<void>;
	refreshIdeas: () => Promise<void>;
	refreshCycles: () => Promise<void>;
	runCycle: () => Promise<void>;
	decide: (
		ideaId: string,
		action: 'accept' | 'reject' | 'rewrite' | 'defer',
		comment: string,
		rewrite?: IdeaRewrite,
		deferredUntil?: string
	) => Promise<DecisionOutcome | null>;
	undo: (ideaId: string) => Promise<void>;
	propose: (input: {
		title: string;
		problem: string;
		hypothesis: string;
		scope: IdeaScope;
		estimated_effort: Effort;
		priority: IdeaPriority;
		expected_benefit?: string;
	}) => Promise<IdeaSummary | null>;
	loadIdea: (ideaId: string) => Promise<IdeaDetail | null>;
	setEnabled: (enabled: boolean) => Promise<void>;
	resetTrip: () => Promise<void>;
	updateConfig: (config: AutonomyConfig) => Promise<void>;
}

/**
 * 自主模式状态放在**全局 store** 而不是面板组件的 useState：
 * 面板挂在迭代页上，切到别的 tab 会卸载组件，组件内状态随之消失——
 * 用户会看到"调研跑完了但建议列表空了"。
 *
 * 事件订阅同样是应用级常驻（模块级一次性绑定），保证后台跑完的调研
 * 在用户切走时也能把结果写进 store。
 */
export const useAutonomyStore = create<AutonomyStoreState>((set, get) => ({
	status: null,
	ideas: [],
	cycles: [],
	localRunning: false,
	steps: [],
	lastNote: null,
	notify: () => {},

	setNotify: notify => set({ notify }),

	refreshStatus: async () => {
		try {
			set({ status: await autonomyApi.getStatus() });
		} catch (error) {
			console.warn('[autonomy] 读取状态失败', error);
		}
	},

	refreshIdeas: async () => {
		try {
			set({ ideas: await autonomyApi.listIdeas() });
		} catch (error) {
			console.warn('[autonomy] 读取建议失败', error);
		}
	},

	refreshCycles: async () => {
		try {
			set({ cycles: await autonomyApi.listCycles() });
		} catch (error) {
			console.warn('[autonomy] 读取调研历史失败', error);
		}
	},

	refresh: async () => {
		await Promise.all([
			get().refreshStatus(),
			get().refreshIdeas(),
			get().refreshCycles()
		]);
	},

	runCycle: async () => {
		const { notify } = get();
		set({ localRunning: true, steps: [] });
		try {
			const result = await autonomyApi.runCycle();
			const report = result.report;
			set({ localRunning: false, lastNote: report.snapshot.note });
			if (report.skipped) {
				notify('info', report.snapshot.note ?? '这轮调研被跳过了');
			} else if (report.published.length > 0) {
				notify('success', report.snapshot.note ?? `找到 ${report.published.length} 条建议`);
			} else {
				notify('info', report.snapshot.note ?? '这轮没有新发现');
			}
			await get().refresh();
		} catch (error) {
			set({ localRunning: false });
			notify('error', '调研失败：' + error);
		}
	},

	decide: async (ideaId, action, comment, rewrite, deferredUntil) => {
		const { notify } = get();
		try {
			const outcome = await autonomyApi.decide(ideaId, action, comment, rewrite, deferredUntil);
			const labels: Record<string, string> = {
				accept: '已采纳，迭代已创建',
				reject: '已驳回',
				rewrite: '已改写',
				defer: '已置后'
			};
			notify('success', labels[action] ?? '已记录');
			await get().refresh();
			return outcome;
		} catch (error) {
			notify('error', '操作失败：' + error);
			return null;
		}
	},

	undo: async ideaId => {
		const { notify } = get();
		try {
			await autonomyApi.undo(ideaId);
			notify('success', '已撤销');
			await get().refresh();
		} catch (error) {
			notify('error', '撤销失败：' + error);
		}
	},

	propose: async input => {
		const { notify } = get();
		try {
			const idea = await autonomyApi.propose(input);
			if (idea.status === 'draft') {
				notify('warning', '这条建议没通过质量门槛，先留在池子里。');
			} else {
				notify('success', '建议已记下');
			}
			await get().refresh();
			return idea;
		} catch (error) {
			notify('error', '保存失败：' + error);
			return null;
		}
	},

	loadIdea: async ideaId => {
		try {
			return await autonomyApi.getIdea(ideaId);
		} catch (error) {
			get().notify('error', '读取详情失败：' + error);
			return null;
		}
	},

	setEnabled: async enabled => {
		const { notify } = get();
		try {
			const status = enabled ? await autonomyApi.start() : await autonomyApi.stop();
			set({ status });
		} catch (error) {
			notify('error', '切换失败：' + error);
		}
	},

	resetTrip: async () => {
		const { notify } = get();
		try {
			set({ status: await autonomyApi.resetTrip() });
			notify('success', '自主循环已恢复');
		} catch (error) {
			notify('error', '恢复失败：' + error);
		}
	},

	updateConfig: async config => {
		const { notify } = get();
		try {
			set({ status: await autonomyApi.updateConfig(config) });
			notify('success', '配置已保存');
		} catch (error) {
			notify('error', '保存失败：' + error);
		}
	}
}));

/**
 * 是否正在调研。
 *
 * 两个来源取或：本客户端发起的调用（localRunning）+ 后端状态机
 * （researching / proposing）。后者保证"切走再回来"时依然知道在跑。
 */
export const selectAutonomyRunning = (state: AutonomyStoreState): boolean =>
	state.localRunning ||
	state.status?.state.status === 'researching' ||
	state.status?.state.status === 'proposing';

/** 后端状态机是否停在"等决策"。 */
export const selectAwaitingDecision = (state: AutonomyStoreState): boolean =>
	state.status?.state.status === 'awaiting_decision';

// ============================================================================
// 应用级事件绑定（模块级一次性，永久存活）
// ============================================================================

let eventsBound = false;

/** 绑定自主模式相关的 Tauri 事件。重复调用只会生效一次。 */
export function ensureAutonomyEventsBound(): void {
	if (eventsBound) return;
	eventsBound = true;

	const bind = <T,>(event: string, handler: (payload: T) => void) => {
		listen<T>(event, e => handler(e.payload)).catch(error => {
			console.warn(`[autonomy] 监听 ${event} 失败`, error);
		});
	};

	// 调研步骤进度
	bind<{ cycle_id: string; step: CycleStep }>('autonomy_cycle_step', payload => {
		useAutonomyStore.setState(prev => {
			const steps = prev.steps.filter(s => s.name !== payload.step.name);
			return { steps: [...steps, payload.step] };
		});
	});

	// 调研结束（周期快照）
	bind<{ snapshot: CycleSnapshot }>('autonomy_cycle_snapshot', payload => {
		useAutonomyStore.setState(prev => ({
			lastNote: payload.snapshot.note,
			cycles: [...prev.cycles.filter(c => c.cycle_id !== payload.snapshot.cycle_id), payload.snapshot]
		}));
		useAutonomyStore.getState().refresh();
	});

	// 一轮调研的完整报告
	bind<{
		report: { snapshot: CycleSnapshot; published: IdeaSummary[]; skipped: string | null };
		trigger: string;
	}>('autonomy_cycle_report', payload => {
		useAutonomyStore.setState({ lastNote: payload.report.snapshot.note });
		useAutonomyStore.getState().refresh();
	});

	// 建议列表变化
	bind('autonomy_ideas_changed', () => {
		useAutonomyStore.getState().refreshIdeas();
	});

	// 状态机变化
	bind('autonomy_state_changed', () => {
		useAutonomyStore.getState().refreshStatus();
	});
}
