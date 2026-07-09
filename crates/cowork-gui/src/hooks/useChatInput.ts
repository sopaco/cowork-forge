import { useCallback } from 'react';
import { App as AntApp } from 'antd';
import { useAgentStore, useUIStore, useProjectStore } from '../stores';
import { useShallow } from 'zustand/react/shallow';
import type { ChatMessage, ThinkingMessage, InputOption } from '../stores';

/**
 * Hook for handling chat input and user interactions
 * Extracts chat input logic from App.tsx
 */
export function useChatInput() {
	const { message } = AntApp.useApp();

	// Project store: selector + useShallow
	const updateCurrentIterationStatus = useProjectStore(s => s.updateCurrentIterationStatus);

	// Agent store: 多字段 selector + useShallow
	const agentState = useAgentStore(
		useShallow(s => ({
			inputRequest: s.inputRequest,
			addMessage: s.addMessage,
			setMessages: s.setMessages,
			setInputRequest: s.setInputRequest,
			submitInput: s.submitInput,
		}))
	);
	const { inputRequest, addMessage, setMessages, setInputRequest, submitInput } = agentState;

	// UI store
	const uiActions = useUIStore(
		useShallow(s => ({
			setActiveView: s.setActiveView,
			setActiveArtifactTab: s.setActiveArtifactTab,
			triggerCodeRefresh: s.triggerCodeRefresh,
			triggerArtifactsRefresh: s.triggerArtifactsRefresh,
		}))
	);
	const { setActiveView, setActiveArtifactTab, triggerCodeRefresh, triggerArtifactsRefresh } = uiActions;

	/**
	 * Handle sending user message
	 */
	const handleSendUserMessage = useCallback(
		async (userInput: string, setUserInput: (input: string) => void) => {
			if (!userInput.trim()) return;
			const msgContent = userInput;
			addMessage({
				type: 'user',
				content: msgContent,
				timestamp: new Date().toISOString()
			} as ChatMessage);

			if (inputRequest) {
				await submitInput(msgContent, 'text');
			} else {
				message.info('Waiting for the agent to request input. Your message was not sent.');
			}
			setUserInput('');
		},
		[inputRequest, addMessage, submitInput, message]
	);

	/**
	 * Handle selecting an option from input request
	 */
	const handleSelectOption = useCallback(
		async (option: InputOption, userInput: string, setUserInput: (input: string) => void) => {
			if (!inputRequest) return;

			if (option.id === 'view_artifact' && inputRequest.isArtifactConfirmation) {
				const artifactTypeToTab: Record<string, string> = {
					idea: 'idea',
					requirements: 'requirements',
					design: 'design',
					plan: 'plan',
					code: 'code'
				};
				const targetTab = artifactTypeToTab[inputRequest.artifactType || ''] || 'idea';
				setActiveArtifactTab(targetTab);

				if (inputRequest.artifactType === 'code') {
					setActiveView('code');
					triggerCodeRefresh();
				} else {
					setActiveView('artifacts');
					triggerArtifactsRefresh();
				}
				return;
			}

			if (option.id === 'feedback' && inputRequest.isArtifactConfirmation) {
				setInputRequest({
					...inputRequest,
					isFeedbackMode: true,
					feedbackPrompt: 'Please enter your feedback:'
				});
				setUserInput('');
				return;
			}

			addMessage({
				type: 'user',
				content: option.label,
				timestamp: new Date().toISOString()
			} as ChatMessage);
			await submitInput(option.id, 'selection');

			// When user confirms with "yes", update iteration status to Running
			if (option.id === 'yes') {
				updateCurrentIterationStatus('Running');
			}

			setUserInput('');
		},
		[
			inputRequest,
			addMessage,
			submitInput,
			setActiveView,
			setActiveArtifactTab,
			triggerCodeRefresh,
			triggerArtifactsRefresh,
			setInputRequest,
			updateCurrentIterationStatus
		]
	);

	/**
	 * Handle submitting feedback
	 */
	const handleSubmitFeedback = useCallback(
		async (userInput: string, setUserInput: (input: string) => void, updateCurrentIterationStatus: (status: string) => void) => {
			if (!inputRequest || !userInput.trim()) return;
			const feedback = userInput.trim();
			addMessage({
				type: 'agent',
				content: '📝 Feedback received. Regenerating...',
				agentName: 'System',
				timestamp: new Date().toISOString()
			} as ChatMessage);
			addMessage({
				type: 'user',
				content: `💬 Feedback:\n${feedback}`,
				timestamp: new Date().toISOString()
			} as ChatMessage);
			await submitInput(feedback, 'text');
			updateCurrentIterationStatus('Running');
			setUserInput('');
		},
		[inputRequest, addMessage, submitInput]
	);

	/**
	 * Handle toggling thinking message expansion
	 */
	const handleToggleThinking = useCallback(
		(index: number) => {
			setMessages((prev) =>
				prev.map((m, i) =>
					i === index && m.type === 'thinking'
						? ({ ...m, isExpanded: !(m as ThinkingMessage).isExpanded } as ChatMessage)
						: m
				)
			);
		},
		[setMessages]
	);

	/**
	 * Handle canceling feedback mode
	 */
	const handleCancelFeedback = useCallback(() => {
		if (inputRequest) {
			setInputRequest({ ...inputRequest, isFeedbackMode: false });
		}
	}, [inputRequest, setInputRequest]);

	return {
		inputRequest,
		handleSendUserMessage,
		handleSelectOption,
		handleSubmitFeedback,
		handleToggleThinking,
		handleCancelFeedback
	};
}
