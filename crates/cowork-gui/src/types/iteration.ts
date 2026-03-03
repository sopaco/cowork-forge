/**
 * Iteration related types
 * Centralized type definitions for iterations
 */

export interface IterationInfo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: IterationStatus;
  current_stage: string | null;
  created_at: string;
  completed_at?: string;
  completed_stages: string[];
  base_iteration_id?: string;
  inheritance?: InheritanceMode;
}

export type IterationStatus = 'Draft' | 'Running' | 'Paused' | 'Completed' | 'Failed';
export type InheritanceMode = 'full' | 'partial' | 'none';

export interface CreateIterationRequest {
  title: string;
  description: string;
  base_iteration_id?: string | null;
  inheritance?: InheritanceMode;
}

export interface StageDef {
  key: string;
  label: string;
  color: string;
}
