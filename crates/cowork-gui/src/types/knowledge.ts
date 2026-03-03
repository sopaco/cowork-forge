/**
 * Knowledge related types
 */

export interface Knowledge {
  iteration_id: string;
  title: string;
  idea_summary?: string;
  design_summary?: string;
  plan_summary?: string;
  code_structure?: string;
  created_at: string;
  tech_stack?: string[];
  key_decisions?: string[];
  key_patterns?: string[];
  known_issues?: string[];
}

export interface KnowledgeListResult {
  knowledge_list: Knowledge[];
}
