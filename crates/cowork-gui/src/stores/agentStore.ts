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
  action_type: 'pm_goto_stage' | 'pm_create_iteration';
  target_stage?: string;
  description?: string;
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
  inputRequest: InputRequest | null;
  chatMode: ChatMode;
  pmProcessing: boolean;
  
  addMessage: (message: ChatMessage) => void;
  setMessages: (messages: ChatMessage[] | ((prev: ChatMessage[]) => ChatMessage[])) => void;
  clearMessages: () => void;
  
  addPMMessage: (message: UserMessage | PMAgentMessage) => void;
  setPMMessages: (messages: (UserMessage | PMAgentMessage)[]) => void;
  clearPMMessages: () => void;
  
  setProcessing: (isProcessing: boolean) => void;
  setCurrentAgent: (agent: string | null) => void;
  setInputRequest: (request: InputRequest | null) => void;
  setChatMode: (mode: ChatMode) => void;
  setPmProcessing: (processing: boolean) => void;
  
  submitInput: (response: string, responseType: string) => Promise<void>;
  sendPMMessage: (iterationId: string, message: string) => Promise<void>;
}

export const useAgentStore = create<AgentState>((set, get) => ({
  messages: [],
  pmMessages: [],
  isProcessing: false,
  currentAgent: null,
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
  
  clearMessages: () => {
    set({ messages: [] });
  },
  
  addPMMessage: (message) => {
    set((state) => ({ pmMessages: [...state.pmMessages, message] }));
  },
  
  setPMMessages: (messages) => {
    set({ pmMessages: messages });
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
    console.log('[AgentStore] sendPMMessage called', { iterationId, message });
    const { pmMessages, pmProcessing } = get();
    console.log('[AgentStore] Current state:', { pmProcessing, pmMessagesCount: pmMessages.length });
    
    if (pmProcessing) {
      console.log('[AgentStore] Already processing - resetting state and continuing');
      set({ pmProcessing: false });
    }
    
    const userMsg: UserMessage = {
      type: 'user',
      content: message,
      timestamp: new Date().toISOString(),
    };
    
    console.log('[AgentStore] Setting state with user message');
    set({ pmProcessing: true, pmMessages: [...pmMessages, userMsg] });
    
    try {
      console.log('[AgentStore] Calling API.pm.sendMessage...');
      const response = await API.pm.sendMessage(
        iterationId, 
        message, 
        [...pmMessages, userMsg]
      ) as { agent_message: string; actions?: PMAction[] };
      
      console.log('[AgentStore] API response:', response);
      
      const agentMsg: PMAgentMessage = {
        type: 'pm_agent',
        content: response.agent_message,
        actions: response.actions,
        timestamp: new Date().toISOString(),
      };
      
      set((state) => ({
        pmMessages: [...state.pmMessages, agentMsg],
        pmProcessing: false,
      }));
      console.log('[AgentStore] Agent message added to state');
    } catch (error) {
      console.error('[AgentStore] PM Agent error:', error);
      set({ pmProcessing: false });
      throw error;
    }
  },
}));
