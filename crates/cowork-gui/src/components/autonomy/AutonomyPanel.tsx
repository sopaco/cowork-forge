import React, { useMemo, useState } from 'react';
import {
	Alert,
	Badge,
	Button,
	Card,
	Empty,
	Form,
	Input,
	Modal,
	Select,
	Space,
	Switch,
	Tag,
	Tooltip,
	Typography
} from 'antd';
import {
	CheckCircleFilled,
	LoadingOutlined,
	MinusCircleOutlined,
	PlusCircleOutlined
} from '@ant-design/icons';
import { useAutonomy } from '../../hooks/useAutonomy';
import IdeaDetailDrawer from './IdeaDetailDrawer';
import type { IdeaScope, IdeaStatus, IdeaSummary } from '../../types/autonomy';

const { Text, Paragraph } = Typography;

const SCOPE_LABEL: Record<IdeaScope, string> = {
	feature: '新增功能',
	refactor: '重构',
	fix: '修复',
	docs: '文档',
	research: '调研'
};

const EFFORT_LABEL: Record<string, string> = {
	s: '小',
	m: '中',
	l: '大',
	xl: '特大'
};

const STATUS_LABEL: Record<IdeaStatus, string> = {
	draft: '草稿',
	proposed: '待决策',
	accepted: '已采纳',
	rejected: '已驳回',
	rewritten: '已改写',
	deferred: '已置后',
	superseded: '已被取代',
	converted: '已转迭代'
};

const CREDIBILITY_LABEL: Record<string, string> = {
	high: '可复现',
	mid: '有依据',
	low: '待核实'
};

const PRIORITY_LABEL: Record<string, string> = {
	high: '高',
	medium: '中',
	low: '低'
};

/** 工作量档位 → 粗略人档，仅用于路线图上的相对比较 */
const EFFORT_WEIGHT: Record<string, number> = { s: 1, m: 2, l: 3, xl: 5 };

/** 后端阶段 → 人话（用于"卡在哪一步"的说明，不笼统写"正在调研"） */
const STAGE_HINT: Record<string, string> = {
	researching: '正在收集证据',
	proposing: '正在生成提案'
};

/** 距某个 ISO 时间点过去了多少分钟（无法解析时返回 null）。 */
function minutesSince(iso: string | null | undefined): number | null {
	if (!iso) return null;
	const ts = Date.parse(iso);
	if (Number.isNaN(ts)) return null;
	return Math.max(0, Math.floor((Date.now() - ts) / 60000));
}

const STATUS_BADGE: Record<string, 'default' | 'processing' | 'success' | 'warning' | 'error'> = {
	disabled: 'default',
	idle: 'success',
	researching: 'processing',
	proposing: 'processing',
	awaiting_decision: 'processing',
	executing: 'processing',
	paused: 'warning',
	tripped: 'error'
};

const STATUS_TEXT: Record<string, string> = {
	disabled: '自主模式未开启',
	idle: '空闲待触发',
	researching: '正在调研',
	proposing: '正在整理建议',
	awaiting_decision: '等待你决策',
	executing: '正在执行',
	paused: '已暂停',
	tripped: '已熔断'
};

interface AutonomyPanelProps {
	/** 紧凑模式：只显示状态胶囊与立即调研（用于顶栏） */
	compact?: boolean;
}

/**
 * 自主模式面板。
 *
 * 设计约束（§11）：
 * - 状态常驻可见，但不抢主视线
 * - 建议卡片三层信息密度：一行结论 + 一条证据位置 + 一行操作
 * - 决策一步完成，且可撤销
 */
const AutonomyPanel: React.FC<AutonomyPanelProps> = ({ compact = false }) => {
	const {
		status,
		ideas,
		running,
		steps,
		lastNote,
		runCycle,
		decide,
		propose,
		setEnabled,
		resetTrip
	} = useAutonomy();

	const [detailId, setDetailId] = useState<string | null>(null);
	const [userIdeaOpen, setUserIdeaOpen] = useState(false);
	const [userForm] = Form.useForm();

	const openIdeas = useMemo(
		() => ideas.filter(i => i.status === 'proposed' || i.status === 'rewritten'),
		[ideas]
	);
	const decidedIdeas = useMemo(
		() => ideas.filter(i => !['proposed', 'rewritten', 'draft'].includes(i.status)),
		[ideas]
	);

	/** 路线图：按影响范围归类未决建议（从真实建议派生，不是另写的文档） */
	const roadmap = useMemo(() => {
		const groups = new Map<IdeaScope, { items: IdeaSummary[]; effort: number }>();
		for (const idea of openIdeas) {
			const entry = groups.get(idea.scope) ?? { items: [], effort: 0 };
			entry.items.push(idea);
			entry.effort += EFFORT_WEIGHT[idea.estimated_effort] ?? 0;
			groups.set(idea.scope, entry);
		}
		return (Object.keys(SCOPE_LABEL) as IdeaScope[])
			.filter(scope => groups.has(scope))
			.map(scope => ({ scope, ...groups.get(scope)! }));
	}, [openIdeas]);

	const enabled = status?.config.enabled ?? false;
	const tripped = status?.state.status === 'tripped';

	/** 触发条件的人话说明 */
	const triggerText = useMemo(() => {
		if (!status) return '';
		const { triggers } = status.config;
		const parts: string[] = [];
		if (triggers.on_iteration_completed) parts.push('迭代跑完后');
		if (triggers.on_schedule)
			parts.push(`每 ${triggers.schedule_interval_mins} 分钟`);
		if (triggers.on_idle) parts.push('空闲时');
		if (parts.length === 0) return '只在你点「立即调研」时跑';
		return parts.join('、') + '自动调研';
	}, [status]);

	/**
	 * 运行中的说明按**真实阶段**生成。
	 *
	 * 原来的实现只有一句硬编码的"正在调研"：卡在生成提案时用户看到的
	 * 和卡在收集证据时一模一样，没法判断是慢还是死了。这里给出阶段名、
	 * 当前步骤名，以及"多久没进展"——排查时这一行就是全部依据。
	 *
	 * 拆成两句：`runningHint` 用于还没有任何步骤时（刷新后/刚起步），
	 * `stallHint` 用于步骤列表已渲染但长时间没有心跳时——否则最常见的
	 * "卡在生成提案"场景下这条提示根本不会显示。
	 */
	const { runningHint, stallHint } = useMemo(() => {
		if (!running) return { runningHint: '', stallHint: '' };
		const phase = status?.state.status;
		const active = status?.state.active_cycle ?? null;
		const currentStep =
			steps.find(step => step.status === 'running') ??
			active?.steps.find(step => step.status === 'running');
		const stage = STAGE_HINT[phase ?? ''] ?? '正在调研';
		const stepText = currentStep ? `（${currentStep.name}）` : '';
		const idleMinutes = minutesSince(active?.heartbeat_at ?? active?.started_at);
		// 阈值对齐单次模型调用上限（10 分钟）：慢模型思考十分钟是正常的，
		// 低于这个量级提示"可能卡住"只会制造误判。
		const stalled = idleMinutes !== null && idleMinutes >= 10;
		const stall = stalled
			? `已经 ${idleMinutes} 分钟没有进展，很可能卡在模型调用上；超过 30 分钟会自动中止并回到空闲。`
			: '';
		return {
			runningHint: stalled
				? `${stage}${stepText}${stall}`
				: `${stage}${stepText}，切到别的页面也不会中断。`,
			stallHint: stall
		};
	}, [running, status, steps]);

	const handleAccept = (idea: IdeaSummary) => {
		Modal.confirm({
			title: '采纳这条建议？',
			content: `会新建一个迭代「${idea.title}」，并把建议内容作为迭代的起点。执行仍需要你在迭代页点开始。`,
			okText: '采纳并创建迭代',
			cancelText: '再想想',
			onOk: async () => {
				await decide(idea.id, 'accept', '');
			}
		});
	};

	if (compact) {
		return (
			<Space size={8} align="center">
				<Badge status={STATUS_BADGE[status?.state.status ?? 'disabled']} />
				<Text style={{ fontSize: 12 }}>{STATUS_TEXT[status?.state.status ?? 'disabled']}</Text>
				<Switch
					size="small"
					checked={enabled}
					disabled={tripped}
				 onChange={checked => setEnabled(checked)}
				/>
				<Button size="small" type="primary" loading={running} onClick={runCycle}>
					立即调研
				</Button>
			</Space>
		);
	}

	return (
		<Card
			size="small"
			title={
				<Space size={8} align="center">
					<Badge status={STATUS_BADGE[status?.state.status ?? 'disabled']} />
					<span>{STATUS_TEXT[status?.state.status ?? 'disabled']}</span>
				</Space>
			}
			extra={
				<Space size={8}>
					<Button size="small" onClick={() => setUserIdeaOpen(true)}>
						我也有想法
					</Button>
					<Switch
						checked={enabled}
						disabled={tripped}
						onChange={checked => setEnabled(checked)}
					/>
					<Button type="primary" size="small" loading={running} onClick={runCycle}>
						立即调研
					</Button>
				</Space>
			}
			styles={{ body: { paddingTop: 12 } }}
		>
			<Text type="secondary" style={{ fontSize: 12 }}>
				{triggerText}
				{status?.budget
					? ` · 今日已调研 ${status.budget.daily_cycles_used}/${status.budget.daily_cycle_limit} 次`
					: ''}
			</Text>

			{tripped && (
				<Alert
					style={{ marginTop: 12 }}
					type="error"
					showIcon
					message={status?.state.tripped_reason ?? '自主循环已熔断'}
					description="确认后点「恢复」重新开始，避免继续刷屏。"
					action={
						<Button size="small" onClick={resetTrip}>
							恢复
						</Button>
					}
				/>
			)}

			{running && steps.length > 0 && (
				<div style={{ marginTop: 12 }}>
					{steps.map(step => (
						<div key={step.name} style={{ marginBottom: 4, fontSize: 12 }}>
							{step.status === 'done' ? (
								<CheckCircleFilled style={{ color: 'var(--success)', marginRight: 6 }} />
							) : step.status === 'running' ? (
								<LoadingOutlined style={{ marginRight: 6 }} />
							) : (
								<MinusCircleOutlined style={{ color: 'var(--text-tertiary)', marginRight: 6 }} />
							)}
							<Text>{step.name}</Text>
							{step.detail && (
								<Text type="secondary" style={{ marginLeft: 6 }}>
									{step.detail}
								</Text>
							)}
						</div>
					))}
					{stallHint && (
						<Text type="warning" style={{ fontSize: 12, display: 'block', marginTop: 4 }}>
							{stallHint}
						</Text>
					)}
				</div>
			)}

			{running && steps.length === 0 && (
				<Text type="secondary" style={{ fontSize: 12, display: 'block', marginTop: 8 }}>
					{runningHint}
				</Text>
			)}

			{!running && lastNote && (
				<Text type="secondary" style={{ fontSize: 12, display: 'block', marginTop: 8 }}>
					{lastNote}
				</Text>
			)}

			<div style={{ marginTop: 12 }}>
				{openIdeas.length === 0 ? (
					<Empty
						image={Empty.PRESENTED_IMAGE_SIMPLE}
						description="还没有待决策的建议。点「立即调研」让它找找项目里还缺什么。"
					/>
				) : (
					openIdeas.map(idea => (
						<Card
							key={idea.id}
							size="small"
							variant="borderless"
							style={{
								background: 'var(--bg-container)',
								marginBottom: 8,
								border: '1px solid var(--border-light)'
							}}
							onClick={() => setDetailId(idea.id)}
						>
							<div style={{ fontWeight: 500, marginBottom: 4 }}>{idea.title}</div>
							<Paragraph
								type="secondary"
								style={{ fontSize: 12, marginBottom: 6 }}
								ellipsis={{ rows: 1 }}
							>
								{idea.problem}
							</Paragraph>
							<Space size={6} wrap style={{ marginBottom: 8 }}>
								<Tag>{SCOPE_LABEL[idea.scope]}</Tag>
								<Tag>工作量 {EFFORT_LABEL[idea.estimated_effort]}</Tag>
								<Tag>优先级 {PRIORITY_LABEL[idea.priority]}</Tag>
								<Tooltip title={`证据 ${idea.evidence_count} 条，${CREDIBILITY_LABEL[idea.credibility]}`}>
									<Tag color={idea.credibility === 'high' ? 'green' : 'default'}>
										{CREDIBILITY_LABEL[idea.credibility]}
									</Tag>
								</Tooltip>
								{idea.conflicts_with.length > 0 && <Tag color="orange">与 {idea.conflicts_with.length} 条冲突</Tag>}
							</Space>
							{idea.evidence_hint && (
								<div
									style={{
										fontSize: 11,
										color: 'var(--text-tertiary)',
										fontFamily: 'var(--font-mono)',
										marginBottom: 8,
										overflow: 'hidden',
										textOverflow: 'ellipsis',
										whiteSpace: 'nowrap'
									}}
								>
									{idea.evidence_hint}
								</div>
							)}
							<Space size={6} onClick={e => e.stopPropagation()}>
								<Button
									size="small"
									type="primary"
									icon={<PlusCircleOutlined />}
									onClick={() => handleAccept(idea)}
								>
									采纳
								</Button>
								<Button size="small" onClick={() => decide(idea.id, 'reject', '')}>
									驳回
								</Button>
								<Button size="small" onClick={() => setDetailId(idea.id)}>
									改写 / 详情
								</Button>
							</Space>
						</Card>
					))
				)}
			</div>

			{openIdeas.length > 0 && (
				<div style={{ marginTop: 12 }}>
					<Text type="secondary" style={{ fontSize: 12 }}>
						路线图（按影响范围归类）
					</Text>
					{roadmap.map(group => (
						<div key={group.scope} style={{ marginTop: 6, fontSize: 12 }}>
							<Space size={6} align="center">
								<Tag>{SCOPE_LABEL[group.scope]}</Tag>
								<Text type="secondary">
									{group.items.length} 条 · 预计 {group.effort} 人档
								</Text>
							</Space>
							<div style={{ marginTop: 2, color: 'var(--text-secondary)' }}>
								{group.items.map(item => item.title).join('、')}
							</div>
						</div>
					))}
				</div>
			)}

			{decidedIdeas.length > 0 && (
				<div style={{ marginTop: 12 }}>
					<Text type="secondary" style={{ fontSize: 12 }}>
						已处理 {decidedIdeas.length} 条
					</Text>
					<div style={{ marginTop: 6 }}>
						{decidedIdeas.slice(0, 6).map(idea => (
							<div
								key={idea.id}
								style={{
									fontSize: 12,
									padding: '4px 0',
									color: 'var(--text-secondary)',
									cursor: 'pointer'
								}}
								onClick={() => setDetailId(idea.id)}
							>
								<Tag style={{ marginRight: 6 }}>{STATUS_LABEL[idea.status]}</Tag>
								{idea.title}
							</div>
						))}
					</div>
				</div>
			)}

			<IdeaDetailDrawer ideaId={detailId} onClose={() => setDetailId(null)} />

			<Modal
				title="记一个想法"
				open={userIdeaOpen}
				onCancel={() => setUserIdeaOpen(false)}
				onOk={async () => {
					const values = await userForm.validateFields();
					const result = await propose({
						title: values.title,
						problem: values.problem,
						hypothesis: values.hypothesis,
						scope: values.scope,
						estimated_effort: values.estimated_effort,
						priority: values.priority,
						expected_benefit: values.expected_benefit ?? ''
					});
					if (result) {
						setUserIdeaOpen(false);
						userForm.resetFields();
					}
				}}
				width={520}
			>
				<Form
					form={userForm}
					layout="vertical"
					size="small"
					initialValues={{ scope: 'feature', estimated_effort: 'm', priority: 'medium' }}
				>
					<Form.Item
						name="title"
						label="标题"
						rules={[{ required: true, message: '写清要做什么改动' }]}
					>
						<Input placeholder="动词短语，描述具体改动" />
					</Form.Item>
					<Form.Item
						name="problem"
						label="现状痛点"
						rules={[{ required: true, message: '说清现在哪里别扭' }]}
					>
						<Input.TextArea rows={2} placeholder="一句话说清可观察的现象" />
					</Form.Item>
					<Form.Item
						name="hypothesis"
						label="验证假设"
						rules={[{ required: true, message: '写下做完之后会怎样' }]}
					>
						<Input.TextArea rows={2} placeholder="做了 X → Y 会变成 Z" />
					</Form.Item>
					<Space size={12} style={{ display: 'flex' }}>
						<Form.Item name="scope" label="范围">
							<Select
								style={{ width: 120 }}
								options={(Object.keys(SCOPE_LABEL) as IdeaScope[]).map(key => ({
									value: key,
									label: SCOPE_LABEL[key]
								}))}
							/>
						</Form.Item>
						<Form.Item name="estimated_effort" label="工作量">
							<Select
								style={{ width: 100 }}
								options={[
									{ value: 's', label: '小' },
									{ value: 'm', label: '中' },
									{ value: 'l', label: '大' },
									{ value: 'xl', label: '特大' }
								]}
							/>
						</Form.Item>
						<Form.Item name="priority" label="优先级">
							<Select
								style={{ width: 100 }}
								options={[
									{ value: 'high', label: '高' },
									{ value: 'medium', label: '中' },
									{ value: 'low', label: '低' }
								]}
							/>
						</Form.Item>
					</Space>
					<Form.Item name="expected_benefit" label="预期收益">
						<Input placeholder="能量化就量化，不确定就写「不确定」" />
					</Form.Item>
					<Text type="secondary" style={{ fontSize: 12 }}>
						没有证据的建议会先留在池子里，不会直接进入决策列表。
					</Text>
				</Form>
			</Modal>
		</Card>
	);
};

export default AutonomyPanel;
