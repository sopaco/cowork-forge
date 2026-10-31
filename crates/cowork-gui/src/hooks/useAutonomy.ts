import { useEffect } from 'react';
import { App as AntApp } from 'antd';
import {
	ensureAutonomyEventsBound,
	selectAutonomyRunning,
	useAutonomyStore
} from '../stores/autonomyStore';

/**
 * 自主模式状态与操作。
 *
 * 状态本体在全局 store（`stores/autonomyStore.ts`），这里只做两件事：
 * 1. 把 antd 的 message 注入 store（store 里不能调 hook）
 * 2. 订阅应用级事件 + 调研中轮询兜底
 *
 * 之所以不放组件 useState：面板挂在迭代页，切 tab 会卸载组件，
 * 组件内状态随之消失——用户会看到"调研跑完了但建议列表空了"。
 */
export function useAutonomy() {
	const { message } = AntApp.useApp();

	const status = useAutonomyStore(s => s.status);
	const ideas = useAutonomyStore(s => s.ideas);
	const cycles = useAutonomyStore(s => s.cycles);
	const running = useAutonomyStore(selectAutonomyRunning);
	const steps = useAutonomyStore(s => s.steps);
	const lastNote = useAutonomyStore(s => s.lastNote);

	// 注入提示函数
	useEffect(() => {
		useAutonomyStore.getState().setNotify((kind, text) => {
			if (kind === 'success') message.success(text);
			else if (kind === 'info') message.info(text);
			else if (kind === 'warning') message.warning(text);
			else message.error(text);
		});
	}, [message]);

	// 事件绑定（应用级一次性）+ 拉取最新状态
	useEffect(() => {
		ensureAutonomyEventsBound();
		useAutonomyStore.getState().refresh();
	}, []);

	// 轮询后端状态：事件在极端情况下可能丢失，轮询兜底。
	// 运行中 2s 一次（进度要跟手），空闲 15s 一次（能发现后台触发的调研，
	// 否则宿主自己跑的那一轮在界面上是隐形的）。
	useEffect(() => {
		const interval = running ? 2000 : 15000;
		const timer = setInterval(() => {
			useAutonomyStore.getState().refreshStatus();
		}, interval);
		return () => clearInterval(timer);
	}, [running]);

	return {
		status,
		ideas,
		cycles,
		running,
		steps,
		lastNote,
		refresh: useAutonomyStore(s => s.refresh),
		runCycle: useAutonomyStore(s => s.runCycle),
		decide: useAutonomyStore(s => s.decide),
		undo: useAutonomyStore(s => s.undo),
		propose: useAutonomyStore(s => s.propose),
		loadIdea: useAutonomyStore(s => s.loadIdea),
		setEnabled: useAutonomyStore(s => s.setEnabled),
		resetTrip: useAutonomyStore(s => s.resetTrip),
		updateConfig: useAutonomyStore(s => s.updateConfig)
	};
}

export default useAutonomy;
