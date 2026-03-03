import { useCallback } from 'react';
import { App as AntApp, Modal } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { useProjectStore, useAgentStore, useUIStore } from '../stores';
import API from '../api';
import type { ChatMessage, PMAction, PMAgentMessage } from '../stores';

/**
 * Hook for handling PM Agent actions and message processing
 * Extracts PM Agent logic from App.tsx
 */
export function usePMAgent() {
	const { message } = AntApp.useApp();

	// Project store
	const { currentIteration, loadProject, setCurrentIteration } = useProjectStore();

	// Agent store
	const { setMessages, clearPMMessages, setPmProcessing, sendPMMessage } = useAgentStore();

	// UI store
	const { setActiveView, setActiveArtifactTab } = useUIStore();

	/**
	 * Build feedback text from PM Chat user messages
	 */
	const buildPMFeedback = useCallback(
		(msgs: (ChatMessage & { type: 'user' | 'pm_agent' })[], targetStage: string): string => {
			console.log(
				'[PM] buildPMFeedback called with',
				msgs.length,
				'messages, targetStage:',
				targetStage
			);

			const userMessages = msgs
				.filter((msg) => msg.type === 'user')
				.map((msg) => (msg as { content: string }).content)
				.filter((content) => content && content.trim());

			if (userMessages.length === 0) return '';

			const result = userMessages.join('\n\n');
			console.log('[PM] buildPMFeedback result length:', result.length);
			return result;
		},
		[]
	);

	/**
	 * Handle PM Agent actions
	 */
	const handlePMAction = useCallback(
		async (action: PMAction, pmMessages: (ChatMessage & { type: 'user' | 'pm_agent' })[]) => {
			if (!currentIteration) return;

			switch (action.action_type) {
				case 'pm_start_app':
					setActiveView('run');
					message.info('Starting application...');
					try {
						const isRunning = await invoke<boolean>('check_project_status', {
							iterationId: currentIteration.id
						});
						if (isRunning) {
							message.info('Application is already running');
							return;
						}
					} catch {}

					try {
						await invoke('start_iteration_project', { iterationId: currentIteration.id });
					} catch (err) {
						message.error('Failed to start app: ' + err);
					}
					break;

				case 'pm_open_folder':
					try {
						await invoke('open_in_file_manager', { path: `workspace_${currentIteration.id}` });
					} catch (err) {
						message.error('Failed to open folder: ' + err);
					}
					break;

				case 'pm_view_knowledge':
					setActiveView('project-knowledge');
					break;

				case 'pm_view_artifacts':
					setActiveView('artifacts');
					setActiveArtifactTab('design');
					break;

				case 'pm_view_code':
					setActiveView('code');
					break;

				case 'pm_goto_stage':
					if (action.target_stage) {
						console.log('[PM] pm_goto_stage action received, target_stage:', action.target_stage);
						const feedbackText = buildPMFeedback(pmMessages, action.target_stage);
						console.log(
							'[PM] buildPMFeedback result:',
							feedbackText
								? `${feedbackText.length} chars: ${feedbackText.substring(0, 100)}...`
								: 'empty'
						);
						Modal.confirm({
							title: 'Confirm Stage Return',
							content: `Return to ${action.target_stage} stage? Your PM Chat conversation will be passed as feedback to the agent.`,
							onOk: async () => {
								try {
									console.log(
										'[PM] User confirmed, calling API.pm.restart with feedback:',
										feedbackText ? `${feedbackText.length} chars` : 'none'
									);
									clearPMMessages();
									await API.pm.restart(
										currentIteration.id,
										action.target_stage!,
										feedbackText.length > 0 ? feedbackText : undefined
									);
									message.success(`Restarted from ${action.target_stage}`);
									loadProject();
								} catch (err) {
									message.error('Failed: ' + err);
								}
							}
						});
					}
					break;

				case 'pm_create_iteration':
					if (action.iteration_id) {
						Modal.confirm({
							title: '启动新迭代',
							content: `是否启动新迭代「${action.title || 'Untitled'}」？`,
							onOk: async () => {
								try {
									clearPMMessages();
									setMessages([]);
									await loadProject();
									const newIteration = await API.iteration.get(action.iteration_id!);
									setCurrentIteration(newIteration);
									await API.iteration.execute(action.iteration_id!);
									message.success('新迭代已启动');
									setActiveView('chat');
								} catch (err) {
									message.error('启动失败: ' + err);
								}
							}
						});
					}
					break;

				default:
					console.log('Unknown PM action:', action);
			}
		},
		[
			currentIteration,
			loadProject,
			setActiveView,
			setActiveArtifactTab,
			clearPMMessages,
			setMessages,
			setCurrentIteration,
			buildPMFeedback,
			message
		]
	);

	/**
	 * Send a PM Agent message
	 */
	const handlePMSendMessage = useCallback(
		async (userInput: string, setUserInput: (input: string) => void) => {
			console.log(
				'[usePMAgent] handlePMSendMessage called, userInput:',
				userInput,
				'currentIteration:',
				currentIteration?.id
			);
			if (!userInput.trim() || !currentIteration) {
				console.log('[usePMAgent] handlePMSendMessage early return: no input or no iteration');
				return;
			}
			const userMessage = userInput.trim();
			setUserInput('');
			setPmProcessing(true);

			try {
				await sendPMMessage(currentIteration.id, userMessage);
			} catch (error) {
				message.error('Failed to process message: ' + error);
			} finally {
				setPmProcessing(false);
			}
		},
		[currentIteration, sendPMMessage, setPmProcessing, message]
	);

	return {
		buildPMFeedback,
		handlePMAction,
		handlePMSendMessage
	};
}
