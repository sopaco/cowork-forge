/**
 * Agent and Chat related types
 * Centralized type definitions for agent interactions
 */

export interface ThinkingMessage {
  type: 'thinking';
  content: string;
  agentName: string;
  stageName?: string;
  isStreaming?: boolean;
  isExpanded: boolean;
  timestamp: string;
}

export interface AgentMessage {
  type: 'agent';
  content: string;
  agentName: string;
  stageName?: string;
  level?: string;
  isStreaming?: boolean;
  timestamp: string;
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

export interface ToolCallMessage {
  type: 'tool_call';
  toolName: string;
  arguments: Record<string, unknown>;
  agentName: string;
  timestamp: string;
}

export interface ToolResultMessage {
  type: 'tool_result';
  toolName: string;
  result: string;
  success: boolean;
  agentName: string;
  timestamp: string;
}

export type ChatMessage =
  | ThinkingMessage
  | AgentMessage
  | UserMessage
  | PMAgentMessage
  | ToolCallMessage
  | ToolResultMessage;

export type ChatMode = 'disabled' | 'pipeline' | 'pm_agent';

export interface PMAction {
  action_type:
    | 'pm_goto_stage'
    | 'pm_create_iteration'
    | 'pm_start_app'
    | 'pm_open_folder'
    | 'pm_view_knowledge'
    | 'pm_view_artifacts'
    | 'pm_view_code';
  target_stage?: string;
  iteration_id?: string;
  title?: string;
  description?: string;
  label: string;
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
