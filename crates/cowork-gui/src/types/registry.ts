export interface ProjectRecord {
  id: string;
  workspace_path: string;
  name: string;
  description: string | null;
  status: 'active' | 'archived' | 'deleted';
  created_at: string;
  updated_at: string;
  last_opened_at: string | null;
}

export interface ProjectQueryOptions {
  status?: 'active' | 'archived' | 'deleted';
  search?: string;
  limit?: number;
}
