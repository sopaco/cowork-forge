/**
 * Event names constants
 */

export const TAURI_EVENTS = {
  // Iteration events
  ITERATION_CREATED: 'iteration_created',
  ITERATION_STARTED: 'iteration_started',
  ITERATION_CONTINUED: 'iteration_continued',
  ITERATION_RETRYING: 'iteration_retrying',
  ITERATION_COMPLETED: 'iteration_completed',
  ITERATION_FAILED: 'iteration_failed',
  
  // Agent events
  AGENT_EVENT: 'agent_event',
  AGENT_STREAMING: 'agent_streaming',
  
  // Tool events
  TOOL_CALL: 'tool_call',
  TOOL_RESULT: 'tool_result',
  
  // PM events
  PM_ACTIONS: 'pm_actions',
  
  // Input events
  INPUT_REQUEST: 'input_request',
  
  // Project events
  PROJECT_LOADED: 'project_loaded',
  PROJECT_INITIALIZED: 'project_initialized',
  PROJECT_CREATED: 'project_created',
  
  // Knowledge events
  KNOWLEDGE_REGENERATION_COMPLETED: 'knowledge_regeneration_completed',
  KNOWLEDGE_REGENERATION_FAILED: 'knowledge_regeneration_failed',
} as const;

export type TauriEventName = typeof TAURI_EVENTS[keyof typeof TAURI_EVENTS];
