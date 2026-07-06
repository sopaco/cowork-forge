import { create } from 'zustand';
import API from '../api';

export interface ThinkingMessage {
  type: 'thinking';
  content: string;
  agentName: string;
  stageName?: string;
  isStreaming?: boolean;
  isExpanded: boolean;
  timestamp: string;
}

export interface ChatMessage {
  type: string;
  content: string;
  agentName?: string;
  stageName?: string;
  level?: string;
  isStreaming?: boolean;
  isExpanded?: boolean;
  timestamp: string;
  toolName?: string;
  arguments?: Record<string, unknown>;
  result?: string;
  success?: boolean;
  actions?: PMAction[];
}

export interface UserMessage {
  type: 'user';
  content: string;
  timestamp: string;
}

export interface PMAgentMessage {
  type: 'pm_agent';
  content: string;
  actions?: PMAction[];
  timestamp: string;
}

export interface PMAction {
  action_type: 'pm_goto_stage' | 'pm_create_iteration' | 'pm_start_app' | 'pm_open_folder' | 'pm_view_knowledge' | 'pm_view_artifacts' | 'pm_view_code';
  target_stage?: string;
  iteration_id?: string;
  title?: string;
  description?: string;
  label: string;
}

export interface InputRequest {
  requestId: string;
  prompt: string;
  options: InputOption[];
  isArtifactConfirmation?: boolean;
  artifactType?: string;
  isFeedbackMode?: boolean;
  feedbackPrompt?: string;
}

export interface InputOption {
  id: string;
  label: string;
  description?: string;
}

export type ChatMode = 'disabled' | 'pipeline' | 'pm_agent';

interface AgentState {
  messages: ChatMessage[];
  pmMessages: (UserMessage | PMAgentMessage)[];
  isProcessing: boolean;
  currentAgent: string | null;
  currentStage: string | null;
  inputRequest: InputRequest | null;
  chatMode: ChatMode;
  pmProcessing: boolean;

  addMessage: (message: ChatMessage) => void;
  setMessages: (messages: ChatMessage[] | ((prev: ChatMessage[]) => ChatMessage[])) => void;
  /** 流式高效 patch：直接修改末尾消息的 content，避免 O(n) 数组拷贝 */
  appendLastMessageContent: (chunk: string, opts?: { isLast?: boolean; agentName?: string; msgType?: string }) => void;
  clearMessages: () => void;

  addPMMessage: (message: UserMessage | PMAgentMessage) => void;
  setPMMessages: (messages: (UserMessage | PMAgentMessage)[] | ((prev: (UserMessage | PMAgentMessage)[]) => (UserMessage | PMAgentMessage)[])) => void;
  /** 流式高效 patch：直接修改末尾 PM 消息的 content */
  appendLastPMMessageContent: (chunk: string, opts?: { isLast?: boolean }) => void;
  clearPMMessages: () => void;

  setProcessing: (isProcessing: boolean) => void;
  setCurrentAgent: (agent: string | null) => void;
  setCurrentStage: (stage: string | null) => void;
  setInputRequest: (request: InputRequest | null) => void;
  setChatMode: (mode: ChatMode) => void;
  setPmProcessing: (processing: boolean) => void;

  submitInput: (response: string, responseType: string) => Promise<void>;
  sendPMMessage: (iterationId: string, message: string) => Promise<void>;
  loadPMWelcomeMessage: (iterationId: string) => Promise<void>;
}

export const useAgentStore = create<AgentState>((set, get) => ({
  messages: [],
  pmMessages: [],
  isProcessing: false,
  currentAgent: null,
  currentStage: null,
  inputRequest: null,
  chatMode: 'disabled',
  pmProcessing: false,

  addMessage: (message) => {
    set((state) => ({ messages: [...state.messages, message] }));
  },

  setMessages: (messagesOrUpdater) => {
    if (typeof messagesOrUpdater === 'function') {
      set((state) => ({ messages: messagesOrUpdater(state.messages) }));
    } else {
      set({ messages: messagesOrUpdater });
    }
  },

  /**
   * 流式高效 patch：直接修改末尾消息的 content。
   * 比函数式 setState + slice+spread 更快：
   * - 复用同一数组外壳（仅替换末尾元素）
   * - 末尾消息也只新建一个对象，content 字符串拼接
   * 配合 useAppEvents 的 raf 节流使用，每帧只触发一次渲染。
   */
  appendLastMessageContent: (chunk, opts = {}) => {
    const { isLast, agentName, msgType } = opts;
    set((state) => {
      const prev = state.messages;
      if (prev.length === 0) {
        // 没有上一条，新建一条
        return {
          messages: [{
            type: msgType || 'agent',
            content: chunk,
            agentName: agentName || 'AI Agent',
            isStreaming: !isLast,
            isExpanded: false,
            timestamp: new Date().toISOString(),
          } as ChatMessage],
        };
      }
      const last = prev[prev.length - 1];
      const lastType = (last as { type?: string }).type;
      const lastAgent = (last as { agentName?: string }).agentName;
      // 末尾不是同类型同 agent 的流式消息，需要 push 新条目
      const sameKind = lastType === (msgType || 'agent') && lastAgent === (agentName || 'AI Agent') && (last as { isStreaming?: boolean }).isStreaming;
      if (!sameKind) {
        return {
          messages: [...prev, {
            type: msgType || 'agent',
            content: chunk,
            agentName: agentName || 'AI Agent',
            isStreaming: !isLast,
            isExpanded: false,
            timestamp: new Date().toISOString(),
          } as ChatMessage],
        };
      }
      // 同条流式：替换末尾元素（数组外壳仍新建以触发 React 重新渲染，但避免 O(n) slice）
      const next = prev.slice();
      next[next.length - 1] = {
        ...last,
        content: (last as { content: string }).content + chunk,
        isStreaming: !isLast,
      } as ChatMessage;
      return { messages: next };
    });
  },

  clearMessages: () => {
    set({ messages: [] });
  },

  addPMMessage: (message) => {
    set((state) => ({ pmMessages: [...state.pmMessages, message] }));
  },

  setPMMessages: (messagesOrUpdater) => {
    if (typeof messagesOrUpdater === 'function') {
      set((state) => ({ pmMessages: messagesOrUpdater(state.pmMessages) }));
    } else {
      set({ pmMessages: messagesOrUpdater });
    }
  },

  appendLastPMMessageContent: (chunk, opts = {}) => {
    const { isLast } = opts;
    set((state) => {
      const prev = state.pmMessages;
      if (prev.length === 0) {
        return {
          pmMessages: [{
            type: 'pm_agent' as const,
            content: chunk,
            isStreaming: !isLast,
            timestamp: new Date().toISOString(),
          } as PMAgentMessage & { isStreaming?: boolean }],
        };
      }
      const last = prev[prev.length - 1] as PMAgentMessage & { isStreaming?: boolean };
      if (last.type !== 'pm_agent' || !last.isStreaming) {
        return {
          pmMessages: [...prev, {
            type: 'pm_agent' as const,
            content: chunk,
            isStreaming: !isLast,
            timestamp: new Date().toISOString(),
          } as PMAgentMessage & { isStreaming?: boolean }],
        };
      }
      const next = prev.slice();
      next[next.length - 1] = {
        ...last,
        content: last.content + chunk,
        isStreaming: !isLast,
      } as PMAgentMessage & { isStreaming?: boolean };
      return { pmMessages: next };
    });
  },

  clearPMMessages: () => {
    set({ pmMessages: [] });
  },

  setProcessing: (isProcessing) => {
    set({ isProcessing });
  },

  setCurrentAgent: (agent) => {
    set({ currentAgent: agent });
  },

  setCurrentStage: (stage) => {
    set({ currentStage: stage });
  },

  setInputRequest: (request) => {
    set({ inputRequest: request });
  },

  setChatMode: (mode) => {
    set({ chatMode: mode });
  },

  setPmProcessing: (processing) => {
    set({ pmProcessing: processing });
  },

  submitInput: async (response, responseType) => {
    const { inputRequest } = get();
    if (!inputRequest) return;

    await API.input.submit(inputRequest.requestId, response, responseType);
    set({ inputRequest: null });
  },

  sendPMMessage: async (iterationId, message) => {
    const { pmMessages } = get();

    const userMsg: UserMessage = {
      type: 'user',
      content: message,
      timestamp: new Date().toISOString(),
    };

    // Add user message and set processing state
    set({ pmProcessing: true, pmMessages: [...pmMessages, userMsg] });

    try {
      // Call API - the response will be streamed via agent_streaming events
      const response = await API.pm.sendMessage(
        iterationId,
        message,
        [...pmMessages, userMsg]
      ) as { agent_message?: string; actions?: PMAction[] };

      // PM actions 由 `pm_actions` Tauri 事件统一附加，不要在这里用 response.actions 直接附加：
      // sendMessage 的 await 可能在流式事件全部到达前 resolve，导致 actions 被错误挂到较早的消息上。
      // 统一收尾：把末尾 PM 消息的 isStreaming 关掉
      set((state) => {
        const msgs = [...state.pmMessages];
        for (let i = msgs.length - 1; i >= 0; i--) {
          if (msgs[i].type === 'pm_agent') {
            const last = msgs[i] as PMAgentMessage & { isStreaming?: boolean };
            if (last.isStreaming) {
              msgs[i] = { ...last, isStreaming: false } as PMAgentMessage;
            }
            break;
          }
        }
        return { pmMessages: msgs, pmProcessing: false };
      });
    } catch (error) {
      set({ pmProcessing: false });
      throw error;
    }
  },

  loadPMWelcomeMessage: async (iterationId: string) => {
    try {
      const response = await API.pm.getWelcome(iterationId);

      if (response) {
        const agentMsg: PMAgentMessage = {
          type: 'pm_agent',
          content: response.agent_message,
          actions: response.actions,
          timestamp: new Date().toISOString(),
        };

        set({ pmMessages: [agentMsg] });
      }
    } catch (error) {
      // 静默失败，不打扰用户
    }
  },
}));
