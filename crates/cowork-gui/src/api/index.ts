import { invoke } from '@tauri-apps/api/core';

// Types defined locally to avoid import issues
export interface ProjectInfo {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  current_iteration_id: string | null;
}

export interface IterationInfo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: string;
  current_stage: string | null;
  created_at: string;
}

export interface Iteration {
  id: string;
  number: number;
  title: string;
  description: string;
  base_iteration_id: string | null;
  inheritance: string;
  status: string;
  started_at: string | null;
  completed_at: string | null;
  current_stage: string | null;
  completed_stages: string[];
  artifacts: Record<string, string>;
  created_at: string;
}

export interface CreateIterationRequest {
  title: string;
  description: string;
  base_iteration_id: string | null;
  inheritance: string;
}

export interface FileTreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileTreeNode[];
  is_expanded?: boolean;
  language?: string;
}

export interface ArtifactsData {
  iteration_id: string;
  idea?: string;
  requirements?: string;
  design?: string;
  plan?: string;
  code_files?: FileInfo[];
  check_report?: string;
  delivery_report?: string;
}

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  language?: string;
  modified_at?: string;
}

export interface PreviewInfo {
  url: string;
  port: number;
  status: string;
  project_type: string;
}

export interface ProjectRuntimeInfo {
  has_frontend: boolean;
  has_backend: boolean;
  preview_url?: string;
  frontend_port?: number;
  backend_port?: number;
  start_command?: string;
}

export interface AppConfig {
  llm: {
    api_base_url: string;
    api_key: string;
    model_name: string;
    temperature?: number;
    max_tokens?: number;
  };
  embedding?: {
    api_base_url?: string;
    api_key?: string;
    model_name?: string;
  };
  external_coding_agent?: {
    enabled: boolean;
    command: string;
    type: string;
  };
}

export interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  technology_stack: string[];
  created_at: string;
  updated_at: string;
  is_built_in: boolean;
  files: TemplateFile[];
  config: TemplateConfig;
}

export interface TemplateFile {
  path: string;
  content: string;
  is_binary?: boolean;
}

export interface TemplateConfig {
  variables: TemplateVariable[];
  post_creation_commands: string[];
}

export interface TemplateVariable {
  name: string;
  description: string;
  default_value: string;
  required: boolean;
}

// API object
const API = {
  project: {
    get: (): Promise<ProjectInfo | null> => invoke('gui_get_project'),
    init: (name: string): Promise<ProjectInfo> => invoke('gui_init_project', { name }),
    delete: (confirm: boolean): Promise<void> => invoke('gui_delete_project', { confirm }),
    getKnowledge: (projectId: string) => invoke('gui_get_project_knowledge', { projectId }),
  },
  
  iteration: {
    list: (): Promise<IterationInfo[]> => invoke('gui_get_iterations'),
    get: (iterationId: string): Promise<Iteration> => invoke('gui_get_iteration', { iterationId }),
    create: (request: CreateIterationRequest): Promise<Iteration> => invoke('gui_create_iteration', { request }),
    execute: (iterationId: string): Promise<void> => invoke('gui_execute_iteration', { iterationId }),
    continue: (iterationId: string): Promise<void> => invoke('gui_continue_iteration', { iterationId }),
    retry: (iterationId: string): Promise<void> => invoke('gui_retry_iteration', { iterationId }),
    delete: (iterationId: string): Promise<void> => invoke('gui_delete_iteration', { iterationId }),
    regenerateKnowledge: (iterationId: string): Promise<void> => invoke('gui_regenerate_knowledge', { iterationId }),
  },
  
  artifacts: {
    get: (iterationId: string): Promise<ArtifactsData> => invoke('get_iteration_artifacts', { iterationId }),
    getFileTree: (iterationId: string): Promise<FileTreeNode> => invoke('get_iteration_file_tree', { iterationId }),
    readFile: (iterationId: string, filePath: string, offset?: number, limit?: number) => 
      invoke('read_iteration_file', { iterationId, filePath, offset, limit }),
    saveFile: (iterationId: string, filePath: string, content: string): Promise<void> => 
      invoke('save_iteration_file', { iterationId, filePath, content }),
    formatCode: (iterationId: string, filePath: string): Promise<string> => 
      invoke('format_code', { iterationId, filePath }),
  },
  
  runner: {
    start: (iterationId: string): Promise<void> => invoke('start_iteration_project', { iterationId }),
    stop: (iterationId: string): Promise<void> => invoke('stop_iteration_project', { iterationId }),
    status: (iterationId: string): Promise<boolean> => invoke('check_project_status', { iterationId }),
    runtimeInfo: (iterationId: string): Promise<ProjectRuntimeInfo> => 
      invoke('get_project_runtime_info', { iterationId }),
  },
  
  preview: {
    start: (iterationId: string): Promise<PreviewInfo> => invoke('start_iteration_preview', { iterationId }),
    stop: (iterationId: string): Promise<void> => invoke('stop_iteration_preview', { iterationId }),
    status: (iterationId: string): Promise<PreviewInfo | null> => invoke('check_preview_status', { iterationId }),
  },
  
  memory: {
    query: (iterationId: string, queryType: string, category?: string) => 
      invoke('query_memory_index', { iterationId, queryType, category }),
    load: (memoryId: string) => invoke('load_memory_detail', { memoryId }),
    save: (iterationId: string, contentType: string, category: string, content: string): Promise<void> => 
      invoke('save_session_memory', { iterationId, contentType, category, content }),
    promote: (memoryId: string, iterationId: string): Promise<void> => 
      invoke('promote_to_project_memory', { memoryId, iterationId }),
  },
  
  registry: {
    list: (options?: { status?: string; search?: string; limit?: number }) => 
      invoke('get_all_projects', options || {}),
    register: (workspacePath: string, name: string, description?: string) => 
      invoke('register_project', { workspacePath, name, description }),
    delete: (projectId: string, deleteFiles: boolean): Promise<void> => 
      invoke('delete_project', { projectId, deleteFiles }),
    open: (projectId: string): Promise<void> => invoke('open_project', { projectId }),
    openInCurrentWindow: (projectId: string): Promise<void> => 
      invoke('open_project_in_current_window', { projectId }),
  },
  
  workspace: {
    set: (workspacePath: string): Promise<void> => invoke('set_workspace', { workspacePath }),
    get: (): Promise<string | null> => invoke('get_workspace'),
    hasOpen: (): Promise<boolean> => invoke('has_open_project'),
  },
  
  pm: {
    sendMessage: (iterationId: string, message: string, history: unknown[]) => 
      invoke('pm_send_message', { iterationId, message, history }),
    restart: (iterationId: string, targetStage: string): Promise<void> => 
      invoke('pm_restart_iteration', { iterationId, targetStage }),
  },
  
  input: {
    submit: (requestId: string, response: string, responseType: string): Promise<void> => 
      invoke('submit_input_response', { requestId, response, responseType }),
  },
  
  config: {
    get: (): Promise<AppConfig> => invoke('get_app_config'),
    save: (config: AppConfig): Promise<void> => invoke('save_app_config', { config }),
    testConnection: (): Promise<{ success: boolean; message: string }> => invoke('test_llm_connection'),
    hasValid: (): Promise<boolean> => invoke('has_valid_config'),
  },
  
  template: {
    list: (): Promise<ProjectTemplate[]> => invoke('get_templates'),
    export: (sessionId: string, name: string, description: string, category: string): Promise<ProjectTemplate> => 
      invoke('export_template', { sessionId, name, description, category }),
    import: (templateData: string): Promise<ProjectTemplate> => 
      invoke('import_template', { templateData }),
    apply: (templateId: string, variables: Record<string, unknown>, targetDir: string): Promise<string[]> => 
      invoke('apply_template', { templateId, variables, targetDir }),
    delete: (templateId: string): Promise<void> => invoke('delete_template', { templateId }),
  },
  
  util: {
    openInFileManager: (path: string): Promise<void> => invoke('open_in_file_manager', { path }),
    getSystemLocale: (): Promise<string> => invoke('get_system_locale'),
  },
};

export default API;
