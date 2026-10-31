/**
 * 自主模式（Autonomous Mode）类型定义
 * 与 cowork-core 的 autonomy / domain::idea serde 输出逐字对齐。
 */

export type AutonomyStatus =
	| 'disabled'
	| 'idle'
	| 'researching'
	| 'proposing'
	| 'awaiting_decision'
	| 'executing'
	| 'paused'
	| 'tripped';

/** 进行中的调研周期：跨页面/跨刷新展示"现在卡在哪一步" */
export interface ActiveCycle {
	cycle_id: string;
	trigger: CycleTrigger;
	started_at: string;
	heartbeat_at: string;
	steps: CycleStep[];
}

export interface AutonomyState {
	status: AutonomyStatus;
	cycle_count: number;
	last_cycle_at: string | null;
	tripped_reason: string | null;
	consecutive_empty_cycles: number;
	iteration_create_failures: number;
	/** 进行中的调研（无调研时为 null） */
	active_cycle?: ActiveCycle | null;
}

export interface AutonomyTriggers {
	on_iteration_completed: boolean;
	on_idle: boolean;
	on_schedule: boolean;
	schedule_interval_mins: number;
}

export interface AutonomyResearch {
	local: boolean;
	memory: boolean;
	feedback: boolean;
	external_mcp: 'as_needed' | 'off';
}

export interface AutonomyBudgetConfig {
	max_llm_calls_per_cycle: number;
	max_cycles_per_day: number;
	max_mcp_calls_per_cycle: number;
}

export interface AutonomyConfig {
	enabled: boolean;
	triggers: AutonomyTriggers;
	research: AutonomyResearch;
	budget: AutonomyBudgetConfig;
	auto_create_iteration: boolean;
}

export interface BudgetStatus {
	llm_used: number;
	llm_limit: number;
	mcp_used: number;
	mcp_limit: number;
	daily_cycles_used: number;
	daily_cycle_limit: number;
	day: string;
}

export interface AutonomyStatusPayload {
	state: AutonomyState;
	config: AutonomyConfig;
	host_running: boolean;
	host_paused: boolean;
	budget: BudgetStatus;
}

export type IdeaScope = 'feature' | 'refactor' | 'fix' | 'docs' | 'research';
export type Effort = 's' | 'm' | 'l' | 'xl';
export type IdeaPriority = 'high' | 'medium' | 'low';
export type IdeaStatus =
	| 'draft'
	| 'proposed'
	| 'accepted'
	| 'rejected'
	| 'rewritten'
	| 'deferred'
	| 'superseded'
	| 'converted';
export type Credibility = 'low' | 'mid' | 'high';

export interface FileRefEvidence {
	kind: 'file_ref';
	path: string;
	line_range: string | null;
	excerpt: string;
}

export interface IterationRefEvidence {
	kind: 'iteration_ref';
	id: string;
	excerpt: string;
}

export interface FeedbackRefEvidence {
	kind: 'feedback_ref';
	stage: string;
	timestamp: string;
	excerpt: string;
}

export interface ExternalRefEvidence {
	kind: 'external_ref';
	url: string;
	source: string;
	accessed_at: string;
	excerpt: string;
}

export type IdeaEvidence =
	| FileRefEvidence
	| IterationRefEvidence
	| FeedbackRefEvidence
	| ExternalRefEvidence;

export interface IdeaRewrite {
	title: string;
	problem: string;
	hypothesis: string;
	scope: IdeaScope;
}

export interface IdeaDecision {
	action: 'accept' | 'reject' | 'rewrite' | 'defer' | 'supersede';
	by: string;
	comment: string;
	rewritten: IdeaRewrite | null;
	decided_at: string;
	deferred_until: string | null;
}

export interface IdeaSummary {
	id: string;
	title: string;
	problem: string;
	hypothesis: string;
	scope: IdeaScope;
	estimated_effort: Effort;
	priority: IdeaPriority;
	status: IdeaStatus;
	source: 'research' | 'user_input' | 'pm_derived';
	credibility: Credibility;
	evidence_count: number;
	evidence_hint: string | null;
	expected_benefit: string;
	derived_iteration_id: string | null;
	deferred_until: string | null;
	conflicts_with: string[];
	updated_at: string;
}

/** 建议详情（core 的 Idea 聚合：含完整证据与最近一次决策） */
export interface Idea
	extends Omit<IdeaSummary, 'credibility' | 'evidence_count' | 'evidence_hint'> {
	evidence: IdeaEvidence[];
	fingerprint: string;
	created_at: string;
	decision: IdeaDecision | null;
	superseded_by: string | null;
	original: IdeaRewrite | null;
}

export interface DecisionLogEntry {
	entry_id: string;
	ts: string;
	idea_id: string;
	action: string;
	by: string;
	comment: string;
	resulting_iteration_id: string | null;
	reversed_by: string | null;
}

export type CycleTrigger = 'manual' | 'iteration_completed' | 'schedule' | 'idle';
export type CycleOutcome = 'completed' | 'empty' | 'skipped' | 'failed' | 'tripped';
export type StepStatus = 'pending' | 'running' | 'done' | 'skipped' | 'failed';

export interface CycleStep {
	name: string;
	status: StepStatus;
	detail: string | null;
	elapsed_ms: number | null;
}

export interface CycleSnapshot {
	cycle_id: string;
	trigger: CycleTrigger;
	started_at: string;
	finished_at: string | null;
	outcome: CycleOutcome;
	llm_calls: number;
	mcp_calls: number;
	degraded: boolean;
	degraded_reason: string | null;
	proposed_idea_ids: string[];
	drafted_idea_ids: string[];
	merged_count: number;
	discarded: string[];
	steps: CycleStep[];
	note: string | null;
	mcp_servers: string[];
	evidence_digest: string;
}

export interface DecisionOutcome {
	idea: IdeaSummary;
	iteration: AutonomyIteration | null;
	audit_entry_id: string;
	auto_start_recommended: boolean;
}

export interface AutonomyIteration {
	id: string;
	number: number;
	title: string;
	description: string;
	base_iteration_id: string | null;
	inheritance: string;
	status: string;
	started_at: string;
	completed_at: string | null;
	current_stage: string | null;
	completed_stages: string[];
	artifacts: Record<string, string>;
	source_idea_id: string | null;
}
