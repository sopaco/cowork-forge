export interface Artifact {
  name: string;
  path: string;
  content?: string;
  type: 'markdown' | 'json' | 'code' | 'other';
}

export interface ArtifactsData {
  idea?: string;
  requirements?: string;
  features?: string;
  design?: string;
  plan?: string;
  delivery?: string;
}

export interface FileTreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileTreeNode[];
}

export interface FileInfo {
  path: string;
  content: string;
  language?: string;
}

export interface MemoryItem {
  id: string;
  content_type: string;
  category: string;
  content: string;
  created_at: string;
  iteration_id?: string;
}

export interface Knowledge {
  tech_stack?: string[];
  key_decisions?: Array<{
    title: string;
    description: string;
    rationale: string;
  }>;
  key_patterns?: Array<{
    name: string;
    description: string;
    usage: string;
  }>;
  known_issues?: Array<{
    description: string;
    severity: string;
    workaround?: string;
  }>;
  summaries?: {
    idea?: string;
    prd?: string;
    design?: string;
    plan?: string;
  };
}

export interface RuntimeInfo {
  runtime_type: string;
  startup_command: string;
  port?: number;
  language: string;
}

export interface PreviewInfo {
  url: string;
  port: number;
  status: 'running' | 'stopped' | 'error';
}
