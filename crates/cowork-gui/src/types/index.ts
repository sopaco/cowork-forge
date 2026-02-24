// Re-export all types
export * from './project';
export * from './registry';
export * from './chat';
export * from './artifacts';

// App-level types
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

export interface Template {
  id: string;
  name: string;
  description: string;
  language: string;
  framework?: string;
  created_at: string;
}
