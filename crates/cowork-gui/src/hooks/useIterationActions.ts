import { useCallback } from 'react';
import { App as AntApp } from 'antd';
import { useProjectStore, useAgentStore, useUIStore } from '../stores';
import API from '../api';
import type { Iteration } from '../stores';

/**
 * Hook for handling iteration-related actions
 * Extracts iteration action logic from App.tsx
 */
export function useIterationActions() {
	const { message } = AntApp.useApp();

	// Project store
	const { iterations, currentIteration, setCurrentIteration, setIsExecuting } = useProjectStore();

	// Agent store
	const { setProcessing } = useAgentStore();

	// UI store
	const { activeView, setActiveView } = useUIStore();

	/**
	 * Handle selecting an iteration
	 */
	const handleSelectIteration = useCallback(
		(iterationId: string) => {
			const iteration = iterations.find((i: Iteration) => i.id === iterationId);
			if (iteration) {
				const { currentIteration, isExecuting } = useProjectStore.getState();
				API.iteration.get(iterationId).then((full) => {
					if (isExecuting && currentIteration?.id === iterationId) {
						setCurrentIteration({ ...full, status: currentIteration.status });
					} else {
						setCurrentIteration(full);
					}
				});
				setActiveView('chat');
			}
		},
		[iterations, setCurrentIteration, setActiveView]
	);

	/**
	 * Handle executing an iteration
	 */
	const handleExecuteIteration = useCallback(async () => {
		if (!currentIteration) return;
		try {
			setProcessing(true);
			await API.iteration.execute(currentIteration.id);
			message.info('Iteration execution started');
		} catch (error) {
			message.error('Failed to execute iteration: ' + error);
			setProcessing(false);
		}
	}, [currentIteration, setProcessing, message]);

	/**
	 * Handle opening project folder
	 */
	const handleOpenProjectFolder = useCallback(async () => {
		try {
			await API.util.openInFileManager('.');
		} catch (error) {
			message.error('Failed to open project folder');
		}
	}, [message]);

	/**
	 * Handle command palette selection
	 */
	const handleCommandSelect = useCallback(
		(commandId: string) => {
			const viewMap: Record<string, string> = {
				'view-iterations': 'iterations',
				'view-chat': 'chat',
				'view-artifacts': 'artifacts',
				'view-code': 'code',
				'view-run': 'run',
				'view-memory': 'execution-memory',
				'view-projects': 'projects',
				'view-settings': 'settings'
			};
			if (viewMap[commandId]) {
				setActiveView(viewMap[commandId] as typeof activeView);
			}
		},
		[setActiveView]
	);

	return {
		handleSelectIteration,
		handleExecuteIteration,
		handleOpenProjectFolder,
		handleCommandSelect
	};
}
