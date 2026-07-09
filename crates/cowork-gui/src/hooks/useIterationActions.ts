import { useCallback } from 'react';
import { App as AntApp } from 'antd';
import { useProjectStore, useAgentStore, useUIStore } from '../stores';
import { useShallow } from 'zustand/react/shallow';
import API from '../api';

/**
 * Hook for handling iteration-related actions
 * Extracts iteration action logic from App.tsx
 */
export function useIterationActions() {
	const { message } = AntApp.useApp();

	// Project store: selector + useShallow
	const projectActions = useProjectStore(
		useShallow(s => ({ currentIteration: s.currentIteration, setCurrentIteration: s.setCurrentIteration, setIsExecuting: s.setIsExecuting }))
	);
	const { currentIteration, setCurrentIteration, setIsExecuting } = projectActions;

	// Agent store
	const agentActions = useAgentStore(
		useShallow(s => ({
			setProcessing: s.setProcessing,
			setPmProcessing: s.setPmProcessing,
		}))
	);
	const { setProcessing, setPmProcessing } = agentActions;

	// UI store
	const uiState = useUIStore(
		useShallow(s => ({ activeView: s.activeView, setActiveView: s.setActiveView }))
	);
	const { activeView, setActiveView } = uiState;

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

				// Release builds launched from Finder may leave stale processing flags
				// when no pipeline is actually running for this iteration.
				const status = fullIteration.status.toLowerCase();
				if (!isExecuting && status !== 'running') {
					setProcessing(false);
					setPmProcessing(false);
				}

				setActiveView('chat');
			} catch (error) {
				console.error('Failed to load iteration:', error);
				message.error('Failed to load iteration: ' + error);
			}
		},
		[setCurrentIteration, setActiveView, setProcessing, setPmProcessing, message]
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
