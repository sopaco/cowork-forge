export interface Project {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  current_iteration_id: string | null;
  metadata?: Record<string, unknown>;
}

export interface ProjectInfo {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  current_iteration_id: string | null;
}

export type IterationStatus = 'Draft' | 'Running' | 'Paused' | 'Completed' | 'Failed';
export type InheritanceMode = 'none' | 'full' | 'partial';

export interface Iteration {
  id: string;
  number: number;
  title: string;
  description: string;
  base_iteration_id: string | null;
  inheritance: InheritanceMode;
  status: IterationStatus;
  started_at: string | null;
  completed_at: string | null;
  current_stage: string | null;
  completed_stages: string[];
  artifacts: Record<string, string>;
  created_at: string;
}

export interface IterationInfo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: IterationStatus;
  current_stage: string | null;
  created_at: string;
}

export interface CreateIterationRequest {
  title: string;
  description: string;
  base_iteration_id: string | null;
  inheritance: InheritanceMode;
}

export const STAGES = [
  { key: 'idea', label: 'Idea', color: '#1890ff' },
  { key: 'prd', label: 'PRD', color: '#52c41a' },
  { key: 'design', label: 'Design', color: '#722ed1' },
  { key: 'plan', label: 'Plan', color: '#fa8c16' },
  { key: 'coding', label: 'Coding', color: '#13c2c2' },
  { key: 'check', label: 'Check', color: '#eb2f96' },
  { key: 'delivery', label: 'Delivery', color: '#52c41a' },
] as const;

export type StageKey = typeof STAGES[number]['key'];

export interface Stage {
  key: StageKey;
  label: string;
  color: string;
}
