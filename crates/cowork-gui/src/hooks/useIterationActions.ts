import { useCallback } from 'react';
import { App as AntApp } from 'antd';
import { useProjectStore, useAgentStore, useUIStore } from '../stores';
import API from '../api';

/**
 * Hook for handling iteration-related actions
 * Extracts iteration action logic from App.tsx
 */
export function useIterationActions() {
	const { message } = AntApp.useApp();

	// Project store
	const { currentIteration, setCurrentIteration, setIsExecuting } = useProjectStore();

	// Agent store
	const { setProcessing } = useAgentStore();

	// UI store
	const { activeView, setActiveView } = useUIStore();

	/**
	 * Handle selecting an iteration
	 * Directly fetches iteration data via API without depending on local iterations array
	 */
	const handleSelectIteration = useCallback(
		async (iterationId: string) => {
			try {
				const { currentIteration, isExecuting } = useProjectStore.getState();
				const fullIteration = await API.iteration.get(iterationId);
				
				if (isExecuting && currentIteration?.id === iterationId) {
					setCurrentIteration({ ...fullIteration, status: currentIteration.status });
				} else {
					setCurrentIteration(fullIteration);
				}
				setActiveView('chat');
			} catch (error) {
				console.error('Failed to load iteration:', error);
				message.error('Failed to load iteration: ' + error);
			}
		},
		[setCurrentIteration, setActiveView, message]
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
	 * Handle opening iteration folder
	 */
	const handleOpenIterationFolder = useCallback(async (iterationId: string) => {
		try {
			await API.util.openInFileManager(iterationId);
		} catch (error) {
			message.error('Failed to open iteration folder: ' + error);
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
		handleOpenIterationFolder,
		handleCommandSelect
	};
}
