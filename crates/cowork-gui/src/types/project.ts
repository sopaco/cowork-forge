/**
 * Project related types
 * Centralized type definitions for projects
 */

export interface ProjectMetadata {
  session_count: number;
  technology_stack: string[];
}

export interface ProjectData {
  project_id: string;
  projectId?: string;
  name: string;
  description?: string;
  status: ProjectStatus;
  workspacePath?: string;
  workspace_path?: string;
  last_opened_at?: string;
  metadata: ProjectMetadata;
}

export type ProjectStatus = 'active' | 'archived' | 'deleted';

export interface ProjectInfo {
  id: string;
  name: string;
  current_iteration_id: string | null;
}

export interface CreateProjectRequest {
  path: string;
  name: string;
  description?: string | null;
}

export interface UpdateProjectRequest {
  projectId: string;
  name: string;
  description?: string | null;
  status?: string | null;
}

export interface CreateProjectResponse {
  project_id: string;
  created_dir: boolean;
}
