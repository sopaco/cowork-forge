import { useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { App as AntApp } from 'antd';
import { useProjectStore, useAgentStore, useUIStore } from '../stores';
import { useShallow } from 'zustand/react/shallow';
import API from '../api';
import type {
	ChatMessage,
	ThinkingMessage,
	InputOption,
	PMAction,
	PMAgentMessage
} from '../stores';

/**
 * Hook for handling all Tauri events in the application.
 *
 * 性能优化要点：
 * 1. 所有 store 通过 selector + useShallow 取多个 action，避免订阅整个 store
 * 2. 流式 agent_streaming 事件走 raf 节流（每帧合并一次），每秒 setState 从 ~30 次降到 ≤60 次
 * 3. 末尾消息 content 走 appendLastMessageContent / appendLastPMMessageContent 的 mutable patch
 * 4. 非流式事件（agent_event/tool_call/tool_result）保留原直写路径，因为它们频率低
 */
export function useAppEvents(userInput: string, setUserInput: (input: string) => void) {
	const { message } = AntApp.useApp();
	const listenersRegistered = useRef(false);

	// Project store actions: 多 action 通过 useShallow 一次取，避免重渲
	const projectActions = useProjectStore(
		useShallow(s => ({
			loadProject: s.loadProject,
			loadIterations: s.loadIterations,
			setCurrentIteration: s.setCurrentIteration,
			updateCurrentIterationStatus: s.updateCurrentIterationStatus,
			setIsExecuting: s.setIsExecuting,
		}))
	);

	// Agent store actions
	const agentActions = useAgentStore(
		useShallow(s => ({
			setMessages: s.setMessages,
			clearMessages: s.clearMessages,
			setPMMessages: s.setPMMessages,
			clearPMMessages: s.clearPMMessages,
			setProcessing: s.setProcessing,
			setCurrentAgent: s.setCurrentAgent,
			setCurrentStage: s.setCurrentStage,
			setInputRequest: s.setInputRequest,
			setPmProcessing: s.setPmProcessing,
			setPendingPMActions: s.setPendingPMActions,
			flushPendingPMActions: s.flushPendingPMActions,
			submitInput: s.submitInput,
			loadPMWelcomeMessage: s.loadPMWelcomeMessage,
			appendLastMessageContent: s.appendLastMessageContent,
			appendLastPMMessageContent: s.appendLastPMMessageContent,
			addMessage: s.addMessage,
		}))
	);

	// UI store actions + state
	const uiState = useUIStore(
		useShallow(s => ({
			commandPaletteVisible: s.commandPaletteVisible,
			setActiveView: s.setActiveView,
			setCommandPaletteVisible: s.setCommandPaletteVisible,
			setActiveArtifactTab: s.setActiveArtifactTab,
			triggerArtifactsRefresh: s.triggerArtifactsRefresh,
			triggerCodeRefresh: s.triggerCodeRefresh,
			triggerMemoryRefresh: s.triggerMemoryRefresh,
			triggerKnowledgeRefresh: s.triggerKnowledgeRefresh,
		}))
	);

	useEffect(() => {
		const setupListeners = async () => {
			if (listenersRegistered.current) return;
			listenersRegistered.current = true;

			// ===== 流式 chunk 缓冲 + raf 节流 =====
			// Pipeline 消息缓冲
			let pipelineBuffer = '';
			let pipelineLastMeta: { isLast?: boolean; agentName?: string; msgType?: string } | null = null;
			let pipelineRafId: number | null = null;
			// PM 消息缓冲
			let pmBuffer = '';
			let pmLastMeta: { isLast?: boolean } | null = null;
			let pmRafId: number | null = null;

			const flushPipeline = () => {
				pipelineRafId = null;
				if (!pipelineBuffer) {
					// 即使 buffer 空，isLast 也要落地（关闭流式标志）
					if (pipelineLastMeta?.isLast && pipelineLastMeta.agentName) {
						// 末尾已经是流式，再次 patch 一次 isLast=false 即可
						agentActions.appendLastMessageContent('', { isLast: true, agentName: pipelineLastMeta.agentName, msgType: pipelineLastMeta.msgType });
					}
					pipelineLastMeta = null;
					return;
				}
				agentActions.appendLastMessageContent(pipelineBuffer, {
					isLast: pipelineLastMeta?.isLast,
					agentName: pipelineLastMeta?.agentName,
					msgType: pipelineLastMeta?.msgType,
				});
				pipelineBuffer = '';
				pipelineLastMeta = null;
			};

			const flushPM = () => {
				pmRafId = null;
				if (!pmBuffer) {
					if (pmLastMeta?.isLast) {
						agentActions.appendLastPMMessageContent('', { isLast: true });
					}
					pmLastMeta = null;
					return;
				}
				agentActions.appendLastPMMessageContent(pmBuffer, { isLast: pmLastMeta?.isLast });
				pmBuffer = '';
				pmLastMeta = null;
			};

			const schedulePipelineFlush = () => {
				if (pipelineRafId == null) {
					pipelineRafId = requestAnimationFrame(flushPipeline);
				}
			};
			const schedulePMFlush = () => {
				if (pmRafId == null) {
					pmRafId = requestAnimationFrame(flushPM);
				}
			};

			const activateIterationForExecution = async (iterationId: string, status: string) => {
				agentActions.setProcessing(true);
				projectActions.setIsExecuting(true);

				try {
					const fullIteration = await API.iteration.get(iterationId);
					const { currentIteration } = useProjectStore.getState();
					if (currentIteration?.id !== iterationId) {
						agentActions.clearMessages();
					}
					projectActions.setCurrentIteration({ ...fullIteration, status });
				} catch (error) {
					console.error('[App] Failed to load iteration for execution:', error);
					projectActions.updateCurrentIterationStatus(status);
				}

				uiState.setActiveView('chat');
			};

			// Register all listeners in parallel for faster startup
			const listenerPromises = [
				// Iteration lifecycle events
				listen('iteration_created', () => {
					projectActions.loadProject();
					message.success('Iteration created');
				}),

				listen('iteration_started', async (event) => {
					const iterationId = event.payload as string;
					await activateIterationForExecution(iterationId, 'Running');
					message.info('Iteration started');
				}),

				listen('iteration_continued', async (event) => {
					const iterationId = event.payload as string;
					await activateIterationForExecution(iterationId, 'Running');
					message.info('Iteration continued');
				}),

				listen('iteration_retrying', async (event) => {
					const iterationId = event.payload as string;
					await activateIterationForExecution(iterationId, 'Running');
					message.info('Retrying iteration...');
				}),

				listen('iteration_completed', (event) => {
					const iterationId = event.payload as string;
					agentActions.setProcessing(false);
					projectActions.setIsExecuting(false);
					agentActions.setCurrentAgent(null);
					agentActions.setCurrentStage(null);
					agentActions.setInputRequest(null);
					projectActions.updateCurrentIterationStatus('Completed');
					projectActions.loadProject();
					uiState.triggerMemoryRefresh();
					uiState.triggerKnowledgeRefresh();
					agentActions.clearPMMessages();
					uiState.setActiveView('chat');
					agentActions.loadPMWelcomeMessage(iterationId);
					message.success('Iteration completed');
				}),

				listen('iteration_failed', (event) => {
					const [, error] = event.payload as [string, string];
					agentActions.setProcessing(false);
					projectActions.setIsExecuting(false);
					agentActions.setCurrentAgent(null);
					agentActions.setCurrentStage(null);
					agentActions.setInputRequest(null);
					projectActions.updateCurrentIterationStatus('Failed');
					projectActions.loadProject();
					message.error('Iteration failed: ' + error);
				}),

				// Agent events (non-streaming) — 用于 thinking 和系统级 agent 消息
				listen('agent_event', (event) => {
					const { content, agent_name, message_type, stage_name, level } = event.payload as {
						content?: string;
						agent_name?: string;
						message_type?: string;
						stage_name?: string;
						level?: string;
					};

					if (agent_name) agentActions.setCurrentAgent(agent_name);
					if (stage_name) agentActions.setCurrentStage(stage_name);
					if (!content) return;

					// thinking 和系统消息走原直写路径（频率低）
					agentActions.setMessages((prev) => {
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
							] as ChatMessage[];
						}
					});
				}),

				// Agent streaming events — 走 raf 节流 + mutable patch
				listen('agent_streaming', (event) => {
					const { content, agent_name, is_thinking, is_first, is_last } = event.payload as {
						content?: string;
						agent_name?: string;
						is_thinking?: boolean;
						is_first?: boolean;
						is_last?: boolean;
					};

					// PM Agent 流式
					if (agent_name === 'PM Agent') {
						if (is_last && !content) {
							// 收尾：先 flush 被 raf 节流的 PM buffer，确保提示文本等内容先落地
							flushPM();
							// 再把末尾 PM 消息的 isStreaming 关掉
							// 关键：必须显式置 isStreaming: false，否则 PM action 按钮永远不渲染
							agentActions.setPMMessages((prev) => {
								const lastMsg = prev[prev.length - 1];
								if (lastMsg?.type === 'pm_agent') {
									return [...prev.slice(0, -1), { ...lastMsg, isStreaming: false } as PMAgentMessage];
								}
								return prev;
							});
							// 流式结束后 flush pending 的 actions，确保它们挂到最后一条消息
							agentActions.flushPendingPMActions();
							agentActions.setPmProcessing(false);
							return;
						}

						if (!content) return;

						// is_first 时强制 flush 上一条（避免拼接串台）
						if (is_first && pmBuffer) {
							flushPM();
						}

						pmBuffer += content;
						pmLastMeta = { isLast: is_last };
						schedulePMFlush();
						return;
					}

					// Pipeline Agent 流式
					if (!content) return;
					const msgType = is_thinking ? 'thinking' : 'agent';

					// is_first 时强制 flush 上一条
					if (is_first && pipelineBuffer) {
						flushPipeline();
					}

					pipelineBuffer += content;
					pipelineLastMeta = { isLast: is_last, agentName: agent_name, msgType };
					schedulePipelineFlush();
				}),

				// Tool events
				listen('tool_call', (event) => {
					const { tool_name, arguments: args, agent_name } = event.payload as {
						tool_name: string;
						arguments: Record<string, unknown>;
						agent_name?: string;
					};
					// 收尾当前流式条目（避免 tool_call 挂在 streaming 消息后面）
					if (pipelineBuffer) flushPipeline();
					agentActions.addMessage({
						type: 'tool_call',
						toolName: tool_name,
						arguments: args,
						agentName: agent_name || 'AI Agent',
						timestamp: new Date().toISOString()
					} as ChatMessage);
				}),

				listen('tool_result', (event) => {
					const { tool_name, result, success, agent_name } = event.payload as {
						tool_name: string;
						result: string;
						success: boolean;
						agent_name?: string;
					};
					if (pipelineBuffer) flushPipeline();
					agentActions.addMessage({
						type: 'tool_result',
						toolName: tool_name,
						result,
						success,
						agentName: agent_name || 'AI Agent',
						timestamp: new Date().toISOString()
					} as ChatMessage);
				}),

				// PM actions event
				listen('pm_actions', (event) => {
					const { actions } = event.payload as { actions: PMAction[] };
					// setPendingPMActions 会在流式收尾后自动 flush。
					// 后端顺序为：提示文本流式 chunk → is_last → pm_actions → invoke 返回；
					// pm_actions 可能晚于 is_last 的 flush 到达，必须在 setPending 时再次 flush。
					agentActions.setPendingPMActions(actions);
				}),

				// Input request event
				listen('input_request', async (event) => {
					const [requestId, prompt, options] = event.payload as [string, string, InputOption[]];
					projectActions.updateCurrentIterationStatus('Paused');

					const artifactMatch = prompt.match(/\[ARTIFACT_TYPE:(\w+)\]$/);
					if (artifactMatch) {
						const artifactType = artifactMatch[1];
						const cleanPrompt = prompt.replace(/\[ARTIFACT_TYPE:\w+\]$/, '').trim();

						// Best-effort iteration refresh — must NOT block the confirmation
						try {
							await projectActions.loadIterations();
							const latestIterations = useProjectStore.getState().iterations;
							if (latestIterations && latestIterations.length > 0) {
								const latestIteration = latestIterations[latestIterations.length - 1];
								const fullIteration = await API.iteration.get(latestIteration.id);
								projectActions.setCurrentIteration(fullIteration);
							}
						} catch (err) {
							console.error('[App] Failed to refresh iteration before confirmation:', err);
						}

						agentActions.setInputRequest({
							requestId,
							prompt: cleanPrompt,
							options,
							isArtifactConfirmation: true,
							artifactType
						});
					} else {
						agentActions.setInputRequest({ requestId, prompt, options });
					}
					setUserInput('');
				}),

				// Project events
				listen('project_loaded', async () => {
					agentActions.setProcessing(false);
					agentActions.setCurrentAgent(null);
					agentActions.setInputRequest(null);
					agentActions.clearMessages();
					projectActions.setCurrentIteration(null);
					await projectActions.loadProject();
					uiState.setActiveView('iterations');
					message.success('Project loaded');
				}),

				listen('project_initialized', async () => {
					agentActions.setProcessing(false);
					agentActions.setCurrentAgent(null);
					agentActions.setInputRequest(null);
					agentActions.clearMessages();
					projectActions.setCurrentIteration(null);
					await projectActions.loadProject();
					uiState.setActiveView('iterations');
					message.success('Project initialized');
				}),

				// Knowledge events
				listen('knowledge_regeneration_completed', () => {
					uiState.triggerKnowledgeRefresh();
					message.success('Knowledge updated');
				}),

				listen<[string, string]>('knowledge_regeneration_failed', (event) => {
					const [iterationId, error] = event.payload;
					console.error('[App] Knowledge regeneration failed:', iterationId, error);
					message.error('Knowledge generation failed: ' + error);
				}),

				// Tray navigation event (emitted when user clicks "Settings" in tray menu)
				listen<string>('tray_navigate', (event) => {
					const target = event.payload;
					if (target === 'settings') {
						uiState.setActiveView('settings');
					}
				}),
			];

			// Wait for all listeners to be registered in parallel
			await Promise.all(listenerPromises);

			// Check if there's already an open project (e.g., when launched with --workspace)
			try {
				const hasOpenProject = await API.workspace.hasOpen();
				if (hasOpenProject) {
					await projectActions.loadProject();
					uiState.setActiveView('iterations');
				}
			} catch (error) {
				console.error('[App] Failed to check for open project:', error);
			}
		};

		setupListeners();

		// Keyboard shortcuts
		const handleKeyDown = (e: KeyboardEvent) => {
			if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
				e.preventDefault();
				uiState.setCommandPaletteVisible(!uiState.commandPaletteVisible);
			}
		};

		window.addEventListener('keydown', handleKeyDown);
		return () => window.removeEventListener('keydown', handleKeyDown);
	}, []);

	return {
		// Expose any needed state or actions
	};
}
