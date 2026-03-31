/**
 * Matches the Rust backend config_definition types
 */

// Agent Types
export type AgentType = "simple" | { loop: { max_iterations?: number } };

export interface ModelConfig {
  model_id?: string;
  temperature?: number;
  max_tokens?: number;
  top_p?: number;
}

export interface ToolReference {
  tool_id: string;
  config?: Record<string, unknown>;
}

export type IncludeContentsMode = "none" | "all" | { selected: string[] };

export interface AgentDefinition {
  id: string;
  name: string;
  description?: string;
  version?: string;
  agent_type: AgentType;
  instruction: string;
  tools: ToolReference[];
  skills: string[];
  model: ModelConfig;
  include_contents: IncludeContentsMode;
  tags: string[];
  metadata: Record<string, unknown>;
}

// Stage Types
export type StageType =
  | "idea"
  | "prd"
  | "design"
  | "plan"
  | "coding"
  | "check"
  | "delivery";

export type HookPoint =
  | "pre_execute"
  | "post_execute"
  | "pre_confirmation"
  | "post_confirmation"
  | "on_failure";

export interface HookConfig {
  integration_id: string;
  point: HookPoint;
  action: string;
  params?: Record<string, unknown>;
  blocking?: boolean;
  timeout_secs?: number;
  on_failure?: "ignore" | "warn" | "abort";
}

export interface ArtifactConfig {
  save_path: string;
  format: string;
  include_metadata?: boolean;
}

export interface StageRetryConfig {
  max_retries: number;
  backoff_ms?: number;
  retry_on?: string[];
}

export interface StageDefinition {
  id: string;
  name: string;
  description?: string;
  stage_type: StageType;
  agent_id: string;
  needs_confirmation?: boolean;
  confirmation_prompt?: string;
  hooks: HookConfig[];
  artifacts?: Record<string, ArtifactConfig>;
  retry?: StageRetryConfig;
  timeout_secs?: number;
  tags: string[];
}

// Flow Types
export type MemoryScope = "project" | "iteration" | "merged";

export type InheritanceMode = "none" | "partial" | "full";

export interface InheritanceConfig {
  default_mode: InheritanceMode;
  stage_mapping: Record<string, string>;
}

export interface FlowConfig {
  stop_on_failure: boolean;
  max_total_time_secs?: number;
  save_state_on_interrupt: boolean;
  memory_scope: MemoryScope;
  inheritance: InheritanceConfig;
}

export interface StageOverrides {
  needs_confirmation?: boolean;
  hooks: HookConfig[];
  timeout_secs?: number;
  skip: boolean;
}

export interface StageReference {
  stage_id: string;
  alias?: string;
  overrides: StageOverrides;
  condition?: string;
  on_success?: string;
  on_failure?: string;
}

export interface GlobalHookConfig {
  integration_id: string;
  points: HookPoint[];
  blocking: boolean;
  timeout_secs: number;
}

export interface FlowDefinition {
  id: string;
  name: string;
  description?: string;
  version?: string;
  stages: StageReference[];
  start_stage?: string;
  global_hooks: GlobalHookConfig[];
  config: FlowConfig;
  tags: string[];
  metadata: Record<string, unknown>;
  /** Whether this is a built-in preset configuration (read-only) */
  is_builtin?: boolean;
}

// Skill Types (agentskills.io standard)
export interface SkillInfo {
  id: string;
  name: string;
  description: string;
  tags: string[];
  body: string;
}

// Integration Types
export type IntegrationType =
  | "rest_api"
  | "webhook"
  | "message_queue"
  | "database";

export type AuthType =
  | "none"
  | "api_key"
  | "bearer_token"
  | "basic_auth"
  | "oauth2";

export type CredentialSource = "env" | "config" | "prompt";

export interface AuthConfig {
  auth_type: AuthType;
  credential_source: CredentialSource;
  credential_key?: string;
  additional_headers?: Record<string, string>;
}

export interface ConnectionConfig {
  base_url?: string;
  timeout_secs?: number;
  retry_count?: number;
  retry_delay_ms?: number;
}

export type IntegrationEvent =
  | "on_stage_start"
  | "on_stage_complete"
  | "on_flow_start"
  | "on_flow_complete"
  | "on_error";

export interface IntegrationDefinition {
  id: string;
  name: string;
  description?: string;
  integration_type: IntegrationType;
  connection: ConnectionConfig;
  auth: AuthConfig;
  events: IntegrationEvent[];
  enabled: boolean;
  metadata: Record<string, unknown>;
}

// Config Validation
export interface ValidationIssue {
  path: string;
  message: string;
  severity: "error" | "warning";
}

export interface ValidationResult {
  valid: boolean;
  issues: ValidationIssue[];
}

// Config Registry State
export interface ConfigRegistryState {
  agents: Record<string, AgentDefinition>;
  stages: Record<string, StageDefinition>;
  flows: Record<string, FlowDefinition>;
  skills: SkillInfo[];
  integrations: Record<string, IntegrationDefinition>;
  default_flow_id?: string;
}

// Builtin Instructions
export interface BuiltinInstruction {
  id: string;
  name: string;
  description: string;
  content: string;
}

// Instruction type for form
export type InstructionType = "builtin" | "file" | "inline";

// Tool Info for agent configuration
export interface ToolInfo {
  id: string;
  name: string;
  category: string;
  description: string;
}
