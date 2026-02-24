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
  inputRequest: InputRequest | null;
  chatMode: ChatMode;
  pmProcessing: boolean;
  
  addMessage: (message: ChatMessage) => void;
  setMessages: (messages: ChatMessage[] | ((prev: ChatMessage[]) => ChatMessage[])) => void;
  clearMessages: () => void;
  
  addPMMessage: (message: UserMessage | PMAgentMessage) => void;
  setPMMessages: (messages: (UserMessage | PMAgentMessage)[] | ((prev: (UserMessage | PMAgentMessage)[]) => (UserMessage | PMAgentMessage)[])) => void;
  clearPMMessages: () => void;
  
  setProcessing: (isProcessing: boolean) => void;
  setCurrentAgent: (agent: string | null) => void;
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
  
  setPMMessages: (messagesOrUpdater) => {
    if (typeof messagesOrUpdater === 'function') {
      set((state) => ({ pmMessages: messagesOrUpdater(state.pmMessages) }));
    } else {
      set({ pmMessages: messagesOrUpdater });
    }
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
    const { pmMessages } = get();
    
    console.log('[PM] Sending message:', { iterationId, message, pmMessagesLength: pmMessages.length });
    
    const userMsg: UserMessage = {
      type: 'user',
      content: message,
      timestamp: new Date().toISOString(),
    };
    
    // Add user message and set processing state
    set({ pmProcessing: true, pmMessages: [...pmMessages, userMsg] });
    
    try {
      console.log('[PM] Calling API...');
      // Call API - the response will be streamed via agent_streaming events
      // We don't need to wait for the full response or add the message here
      const response = await API.pm.sendMessage(
        iterationId, 
        message, 
        [...pmMessages, userMsg]
      ) as { agent_message?: string; actions?: PMAction[] };
      
      console.log('[PM] API response:', response);
      
      // If there are actions from the response, find the last pm_agent message and add actions to it
      // Note: The streaming message is added by the event listener, not here
      if (response.actions && response.actions.length > 0) {
        set((state) => {
          const msgs = [...state.pmMessages];
          // Find the last pm_agent message (not the user message)
          let lastPmAgentIdx = -1;
          for (let i = msgs.length - 1; i >= 0; i--) {
            if (msgs[i].type === 'pm_agent') {
              lastPmAgentIdx = i;
              break;
            }
          }
          
          if (lastPmAgentIdx >= 0) {
            const lastMsg = msgs[lastPmAgentIdx] as PMAgentMessage;
            // Only add actions if not already present
            if (!lastMsg.actions || lastMsg.actions.length === 0) {
              msgs[lastPmAgentIdx] = { ...lastMsg, actions: response.actions };
            }
          }
          return { pmMessages: msgs, pmProcessing: false };
        });
      } else {
        set({ pmProcessing: false });
      }
    } catch (error) {
      console.error('PM Agent error:', error);
      set({ pmProcessing: false });
      // Optionally add error message
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
      console.error('Failed to load welcome message:', error);
    }
  },
}));
