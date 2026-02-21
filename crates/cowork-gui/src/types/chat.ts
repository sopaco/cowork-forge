export type MessageType = 
  | 'user' 
  | 'agent' 
  | 'thinking' 
  | 'tool_call' 
  | 'tool_result' 
  | 'pm_agent' 
  | 'error'
  | 'system';

export interface BaseMessage {
  type: MessageType;
  timestamp: string;
}

export interface UserMessage extends BaseMessage {
  type: 'user';
  content: string;
}

export interface AgentMessage extends BaseMessage {
  type: 'agent';
  content: string;
  agentName: string;
  stageName?: string;
  level?: string;
  isStreaming?: boolean;
}

export interface ThinkingMessage extends BaseMessage {
  type: 'thinking';
  content: string;
  agentName: string;
  stageName?: string;
  isStreaming?: boolean;
  isExpanded: boolean;
}

export interface ToolCallMessage extends BaseMessage {
  type: 'tool_call';
  toolName: string;
  arguments: Record<string, unknown>;
  agentName: string;
}

export interface ToolResultMessage extends BaseMessage {
  type: 'tool_result';
  toolName: string;
  result: string;
  success: boolean;
  agentName: string;
}

export interface PMAgentMessage extends BaseMessage {
  type: 'pm_agent';
  content: string;
  actions?: PMAction[];
}

export interface ErrorMessage extends BaseMessage {
  type: 'error';
  content: string;
}

export type ChatMessage = 
  | UserMessage 
  | AgentMessage 
  | ThinkingMessage 
  | ToolCallMessage 
  | ToolResultMessage 
  | PMAgentMessage 
  | ErrorMessage;

export interface PMAction {
  action_type: 'pm_goto_stage' | 'pm_create_iteration';
  target_stage?: string;
  description?: string;
}

export interface InputOption {
  id: string;
  label: string;
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

export type ChatMode = 'disabled' | 'pipeline' | 'pm_agent';
