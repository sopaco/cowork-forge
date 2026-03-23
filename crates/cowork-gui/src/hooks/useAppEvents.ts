import { useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { App as AntApp } from 'antd';
import { useProjectStore, useAgentStore, useUIStore } from '../stores';
import API from '../api';
import type {
	ChatMessage,
	ThinkingMessage,
	InputOption,
	PMAction,
	PMAgentMessage
} from '../stores';

/**
 * Hook for handling all Tauri events in the application
 * Extracts ~350 lines of event listener logic from App.tsx
 */
export function useAppEvents(userInput: string, setUserInput: (input: string) => void) {
	const { message } = AntApp.useApp();
	const listenersRegistered = useRef(false);

	// Project store actions
	const {
		loadProject,
		loadIterations,
		setCurrentIteration,
		updateCurrentIterationStatus,
		setIsExecuting
	} = useProjectStore();

	// Agent store actions
	const {
		setMessages,
		clearMessages,
		setPMMessages,
		clearPMMessages,
		setProcessing,
		setCurrentAgent,
		setCurrentStage,
		setInputRequest,
		setPmProcessing,
		submitInput,
		loadPMWelcomeMessage
	} = useAgentStore();

	// UI store actions
	const {
		commandPaletteVisible,
		setActiveView,
		setCommandPaletteVisible,
		setActiveArtifactTab,
		triggerArtifactsRefresh,
		triggerCodeRefresh,
		triggerMemoryRefresh,
		triggerKnowledgeRefresh
	} = useUIStore();

	useEffect(() => {
		const setupListeners = async () => {
			if (listenersRegistered.current) return;
			listenersRegistered.current = true;

			// Register all listeners in parallel for faster startup
			const listenerPromises = [
				// Iteration lifecycle events
				listen('iteration_created', () => {
					loadProject();
					message.success('Iteration created');
				}),

				listen('iteration_started', (event) => {
					const iterationId = event.payload as string;
					setProcessing(true);
					setIsExecuting(true);
					updateCurrentIterationStatus('Running');
					setActiveView('chat');
					message.info('Iteration started');
				}),

				listen('iteration_continued', (event) => {
					const iterationId = event.payload as string;
					setProcessing(true);
					setIsExecuting(true);
					updateCurrentIterationStatus('Running');
					setActiveView('chat');
					message.info('Iteration continued');
				}),

				listen('iteration_retrying', (event) => {
					const iterationId = event.payload as string;
					setProcessing(true);
					setIsExecuting(true);
					updateCurrentIterationStatus('Running');
					setActiveView('chat');
					message.info('Retrying iteration...');
				}),

				listen('iteration_completed', (event) => {
					const iterationId = event.payload as string;
					setProcessing(false);
					setIsExecuting(false);
					setCurrentAgent(null);
					setCurrentStage(null);
					setInputRequest(null);
					updateCurrentIterationStatus('Completed');
					loadProject();
					triggerMemoryRefresh();
					triggerKnowledgeRefresh();
					clearPMMessages();
					setActiveView('chat');
					loadPMWelcomeMessage(iterationId);
					message.success('Iteration completed');
				}),

				listen('iteration_failed', (event) => {
					const [, error] = event.payload as [string, string];
					setProcessing(false);
					setIsExecuting(false);
					setCurrentAgent(null);
					setCurrentStage(null);
					setInputRequest(null);
					updateCurrentIterationStatus('Failed');
					loadProject();
					message.error('Iteration failed: ' + error);
				}),

				// Agent events (non-streaming)
				listen('agent_event', (event) => {
					const { content, agent_name, message_type, stage_name, level } = event.payload as {
						content?: string;
						agent_name?: string;
						message_type?: string;
						stage_name?: string;
						level?: string;
					};

					if (agent_name) setCurrentAgent(agent_name);
					if (stage_name) setCurrentStage(stage_name);
					if (!content) return;

					setMessages((prev) => {
						const lastMsg = prev[prev.length - 1];
						const isThinking = message_type === 'thinking';

						if (isThinking) {
							if (
								lastMsg?.type === 'thinking' &&
								(lastMsg as ThinkingMessage).isStreaming &&
								(lastMsg as ThinkingMessage).agentName === agent_name
							) {
								return [
									...prev.slice(0, -1),
									{
										...lastMsg,
										content: (lastMsg as ThinkingMessage).content + content
									} as ChatMessage
								];
							}
							return [
								...prev,
								{
									type: 'thinking',
									content,
									agentName: agent_name || 'AI Agent',
									stageName: stage_name,
									isStreaming: true,
									isExpanded: false,
									timestamp: new Date().toISOString()
								} as ThinkingMessage
							] as ChatMessage[];
						} else {
							if (
								lastMsg?.type === 'agent' &&
								(lastMsg as { isStreaming?: boolean }).isStreaming &&
								(lastMsg as { agentName?: string }).agentName === agent_name
							) {
								return [
									...prev.slice(0, -1),
									{
										...lastMsg,
										content: (lastMsg as { content: string }).content + content
									} as ChatMessage
								];
							}
							return [
								...prev,
								{
									type: 'agent',
									content,
									agentName: agent_name || 'AI Agent',
									stageName: stage_name,
									level,
									isStreaming: true,
									timestamp: new Date().toISOString()
								} as ChatMessage
							];
						}
					});
				}),

				// Agent streaming events
				listen('agent_streaming', (event) => {
					const { content, agent_name, is_thinking, is_first, is_last } = event.payload as {
						content?: string;
						agent_name?: string;
						is_thinking?: boolean;
						is_first?: boolean;
						is_last?: boolean;
					};

					// Handle PM Agent streaming messages separately
					if (agent_name === 'PM Agent') {
						if (is_last && !content) {
							setPMMessages((prev) => {
								const lastMsg = prev[prev.length - 1];
								if (lastMsg?.type === 'pm_agent') {
									return [...prev.slice(0, -1), { ...lastMsg } as PMAgentMessage];
								}
								return prev;
							});
							setPmProcessing(false);
							return;
						}

						if (!content) return;

						setPMMessages((prev) => {
							const lastMsg = prev[prev.length - 1];
							if (
								is_first ||
								!lastMsg ||
								lastMsg.type !== 'pm_agent' ||
								!(lastMsg as PMAgentMessage & { isStreaming?: boolean }).isStreaming
							) {
								return [
									...prev,
									{
										type: 'pm_agent' as const,
										content,
										isStreaming: !is_last,
										timestamp: new Date().toISOString()
									} as PMAgentMessage & { isStreaming?: boolean }
								];
							}
							return [
								...prev.slice(0, -1),
								{
									...lastMsg,
									content: (lastMsg as PMAgentMessage).content + content,
									isStreaming: !is_last
								} as PMAgentMessage & { isStreaming?: boolean }
							];
						});
						return;
					}

					// Handle Pipeline Agent streaming messages
					if (!content) return;
					const msgType = is_thinking ? 'thinking' : 'agent';

					setMessages((prev) => {
						const lastMsg = prev[prev.length - 1];
						if (
							lastMsg?.type === msgType &&
							(lastMsg as { isStreaming?: boolean }).isStreaming &&
							(lastMsg as { agentName?: string }).agentName === agent_name
						) {
							return [
								...prev.slice(0, -1),
								{
									...lastMsg,
									content: (lastMsg as { content: string }).content + content,
									isStreaming: !is_last
								} as ChatMessage
							];
						}
						return [
							...prev,
							{
								type: msgType,
								content,
								agentName: agent_name || 'AI Agent',
								isStreaming: !is_last,
								isExpanded: false,
								timestamp: new Date().toISOString()
							} as ChatMessage
						];
					});
				}),

				// Tool events
				listen('tool_call', (event) => {
					const { tool_name, arguments: args, agent_name } = event.payload as {
						tool_name: string;
						arguments: Record<string, unknown>;
						agent_name?: string;
					};
					setMessages((prev) => [
						...prev,
						{
							type: 'tool_call',
							toolName: tool_name,
							arguments: args,
							agentName: agent_name || 'AI Agent',
							timestamp: new Date().toISOString()
						} as ChatMessage
					]);
				}),

				listen('tool_result', (event) => {
					const { tool_name, result, success, agent_name } = event.payload as {
						tool_name: string;
						result: string;
						success: boolean;
						agent_name?: string;
					};
					setMessages((prev) => [
						...prev,
						{
							type: 'tool_result',
							toolName: tool_name,
							result,
							success,
							agentName: agent_name || 'AI Agent',
							timestamp: new Date().toISOString()
						} as ChatMessage
					]);
				}),

				// PM actions event
				listen('pm_actions', (event) => {
					const { actions } = event.payload as { actions: PMAction[] };
					setPMMessages((prev) => {
						const lastMsg = prev[prev.length - 1];
						if (lastMsg?.type === 'pm_agent') {
							return [
								...prev.slice(0, -1),
								{
									...lastMsg,
									actions: [...((lastMsg as PMAgentMessage).actions || []), ...actions]
								} as PMAgentMessage
							];
						}
						return prev;
					});
				}),

				// Input request event
				listen('input_request', async (event) => {
					const [requestId, prompt, options] = event.payload as [string, string, InputOption[]];
					updateCurrentIterationStatus('Paused');

					const artifactMatch = prompt.match(/\[ARTIFACT_TYPE:(\w+)\]$/);
					if (artifactMatch) {
						const artifactType = artifactMatch[1];
						const cleanPrompt = prompt.replace(/\[ARTIFACT_TYPE:\w+\]$/, '').trim();

						await loadIterations();
						const latestIterations = useProjectStore.getState().iterations;
						if (latestIterations && latestIterations.length > 0) {
							const latestIteration = latestIterations[latestIterations.length - 1];
							const fullIteration = await API.iteration.get(latestIteration.id);
							setCurrentIteration(fullIteration);
						}

						setInputRequest({
							requestId,
							prompt: cleanPrompt,
							options,
							isArtifactConfirmation: true,
							artifactType
						});
					} else {
						setInputRequest({ requestId, prompt, options });
					}
					setUserInput('');
				}),

				// Project events
				listen('project_loaded', () => {
					setProcessing(false);
					setCurrentAgent(null);
					setInputRequest(null);
					clearMessages();
					setCurrentIteration(null);
					setActiveView('iterations');
					loadProject();
					message.success('Project loaded');
				}),

				listen('project_initialized', () => {
					setProcessing(false);
					setCurrentAgent(null);
					setInputRequest(null);
					clearMessages();
					setCurrentIteration(null);
					setActiveView('iterations');
					loadProject();
					message.success('Project initialized');
				}),

				// Knowledge events
				listen('knowledge_regeneration_completed', () => {
					triggerKnowledgeRefresh();
					message.success('Knowledge updated');
				}),

				listen<[string, string]>('knowledge_regeneration_failed', (event) => {
					const [iterationId, error] = event.payload;
					console.error('[App] Knowledge regeneration failed:', iterationId, error);
					message.error('Knowledge generation failed: ' + error);
				}),
			];

			// Wait for all listeners to be registered in parallel
			await Promise.all(listenerPromises);
		};

		setupListeners();

		// Keyboard shortcuts
		const handleKeyDown = (e: KeyboardEvent) => {
			if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
				e.preventDefault();
				setCommandPaletteVisible(!commandPaletteVisible);
			}
		};

		window.addEventListener('keydown', handleKeyDown);
		return () => window.removeEventListener('keydown', handleKeyDown);
	}, []);

	return {
		// Expose any needed state or actions
	};
}
