/**
 * Centralized type exports
 * Single source of truth for all type definitions
 */

// Iteration types
export type {
  IterationInfo,
  IterationStatus,
  InheritanceMode,
  CreateIterationRequest,
  StageDef,
} from './iteration';

// Project types
export type {
  ProjectMetadata,
  ProjectData,
  ProjectStatus,
  ProjectInfo,
  CreateProjectRequest,
  UpdateProjectRequest,
  CreateProjectResponse,
} from './project';

// Agent and Chat types
export type {
  ThinkingMessage,
  AgentMessage,
  UserMessage,
  PMAgentMessage,
  ToolCallMessage,
  ToolResultMessage,
  ChatMessage,
  ChatMode,
  PMAction,
  InputOption,
  InputRequest,
} from './agent';

// Knowledge types
export type {
  Knowledge,
  KnowledgeListResult,
} from './knowledge';

// Autonomous mode types
export type {
	AutonomyStatus,
	AutonomyState,
	AutonomyTriggers,
	AutonomyResearch,
	AutonomyBudgetConfig,
	AutonomyConfig,
	AutonomyStatusPayload,
	BudgetStatus,
	IdeaScope,
	Effort,
	IdeaPriority,
	IdeaStatus,
	Credibility,
	FileRefEvidence,
	IterationRefEvidence,
	FeedbackRefEvidence,
	ExternalRefEvidence,
	IdeaEvidence,
	IdeaRewrite,
	IdeaDecision,
	IdeaSummary,
	Idea,
	DecisionLogEntry,
	CycleTrigger,
	CycleOutcome,
	StepStatus,
	CycleStep,
	CycleSnapshot,
	DecisionOutcome,
	AutonomyIteration,
} from './autonomy';

// Config types
export type {
	AgentType,
	ModelConfig,
  ToolReference,
  IncludeContentsMode,
  AgentDefinition,
  StageType,
  HookPoint,
  HookConfig,
  ArtifactConfig,
  StageRetryConfig,
  StageDefinition,
  MemoryScope,
  // InheritanceMode - already exported from iteration.ts
  InheritanceConfig,
  FlowConfig,
  StageOverrides,
  StageReference,
  GlobalHookConfig,
  FlowDefinition,
  SkillInfo,
  IntegrationType,
  AuthType,
  CredentialSource,
  AuthConfig,
  ConnectionConfig,
  IntegrationEvent,
  IntegrationDefinition,
  ValidationIssue,
  ValidationResult,
  ConfigRegistryState,
} from './config';
