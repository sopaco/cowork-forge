import React, { useCallback, useEffect, useState } from 'react';
import { App as AntApp, Button, Collapse, Descriptions, Divider, Drawer, Form, Input, Modal, Select, Space, Tag, Timeline, Typography } from 'antd';
import { useAutonomy } from '../../hooks/useAutonomy';
import API from '../../api';
import type { DecisionOutcome, IdeaScope, IdeaRewrite } from '../../types/autonomy';

const { Text, Paragraph } = Typography;

const SCOPE_LABEL: Record<IdeaScope, string> = {
	feature: '新增功能',
	refactor: '重构',
	fix: '修复',
	docs: '文档',
	research: '调研'
};

const ACTION_LABEL: Record<string, string> = {
	accept: '采纳',
	reject: '驳回',
	rewrite: '改写',
	defer: '置后',
	supersede: '被取代'
};

interface IdeaDetailDrawerProps {
	ideaId: string | null;
	onClose: () => void;
}

/**
 * 建议详情抽屉：问题 / 假设 / 预期收益 / 全部证据 / 决策历史，
 * 以及四类决策操作（一步完成，可撤销）。
 */
const IdeaDetailDrawer: React.FC<IdeaDetailDrawerProps> = ({ ideaId, onClose }) => {
	const { message } = AntApp.useApp();
	const { loadIdea, decide, undo } = useAutonomy();
	const [detail, setDetail] = useState<Awaited<ReturnType<typeof loadIdea>>>(null);
	const [rewriteOpen, setRewriteOpen] = useState(false);
	const [form] = Form.useForm();

	const reload = useCallback(async () => {
		if (!ideaId) {
			setDetail(null);
			return;
		}
		const result = await loadIdea(ideaId);
		setDetail(result);
		if (result?.idea) {
			form.setFieldsValue({
				title: result.idea.title,
				problem: result.idea.problem,
				hypothesis: result.idea.hypothesis,
				scope: result.idea.scope
			});
		}
	}, [ideaId, loadIdea, form]);

	useEffect(() => {
		setRewriteOpen(false);
		reload();
	}, [reload]);

	const idea = detail?.idea;
	const timeline = detail?.timeline ?? [];

	const handleDecide = async (
		action: 'accept' | 'reject' | 'rewrite' | 'defer',
		comment: string,
		rewrite?: IdeaRewrite,
		deferredUntil?: string
	): Promise<DecisionOutcome | null> => {
		if (!idea) return null;
		const outcome = await decide(idea.id, action, comment, rewrite, deferredUntil);
		await reload();
		return outcome;
	};

	const handleAccept = () => {
		if (!idea) return;
		Modal.confirm({
			title: '采纳这条建议？',
			content: `会新建一个迭代「${idea.title}」。执行仍需要你在迭代页点开始。`,
			okText: '采纳并创建迭代',
			cancelText: '再想想',
			onOk: async () => {
				const outcome = await handleDecide('accept', '');
				// 低风险范围 + 开启了自动执行：问一句要不要直接开跑（双门禁）
				if (outcome?.auto_start_recommended && outcome.iteration) {
					Modal.confirm({
						title: '现在就开始执行？',
						content: '这条建议属于低风险范围，可以直接开跑。',
						okText: '开始执行',
						cancelText: '先不跑',
						onOk: async () => {
							try {
								await API.iteration.execute(outcome.iteration!.id);
								message.success('迭代已开始');
							} catch (error) {
								message.error('启动失败：' + error);
							}
						}
					});
				}
			}
		});
	};

	const handleRewrite = async () => {
		const values = await form.validateFields();
		await handleDecide('rewrite', '按人工改写后的范围执行', {
			title: values.title,
			problem: values.problem,
			hypothesis: values.hypothesis,
			scope: values.scope
		});
		setRewriteOpen(false);
	};

	const canDecide = idea?.status === 'proposed' || idea?.status === 'rewritten';

	return (
		<Drawer
			title={idea?.title ?? '建议详情'}
			width={560}
			open={ideaId !== null}
			onClose={onClose}
			destroyOnHidden
			footer={
				idea && (
					<Space>
						{canDecide ? (
							<>
								<Button type="primary" onClick={handleAccept}>
									采纳
								</Button>
								<Button onClick={() => handleDecide('reject', '')}>驳回</Button>
								<Button onClick={() => setRewriteOpen(v => !v)}>
									{rewriteOpen ? '收起改写' : '改写'}
								</Button>
								<Button
									onClick={() =>
										handleDecide(
											'defer',
											'稍后再看',
											undefined,
											new Date(Date.now() + 7 * 86400_000).toISOString()
										)
									}
								>
									置后一周
								</Button>
							</>
						) : (
							<Text type="secondary">这条建议已经决策过了。</Text>
						)}
						{timeline.some(t => !t.reversed_by) && (
							<Button danger onClick={async () => {
								await undo(idea.id);
								await reload();
							}}>
								撤销上次决策
							</Button>
						)}
					</Space>
				)
			}
		>
			{!idea ? (
				<Text type="secondary">加载中…</Text>
			) : (
				<>
					<Descriptions size="small" column={2} style={{ marginBottom: 12 }}>
						<Descriptions.Item label="范围">{SCOPE_LABEL[idea.scope]}</Descriptions.Item>
						<Descriptions.Item label="工作量">{idea.estimated_effort}</Descriptions.Item>
						<Descriptions.Item label="优先级">{idea.priority}</Descriptions.Item>
						<Descriptions.Item label="来源">
							{idea.source === 'user_input' ? '你输入的' : '调研发现'}
						</Descriptions.Item>
					</Descriptions>

					<Paragraph strong style={{ marginBottom: 4 }}>
						现状痛点
					</Paragraph>
					<Paragraph style={{ marginBottom: 12 }}>{idea.problem}</Paragraph>

					<Paragraph strong style={{ marginBottom: 4 }}>
						验证假设
					</Paragraph>
					<Paragraph style={{ marginBottom: 12 }}>{idea.hypothesis}</Paragraph>

					{idea.expected_benefit && (
						<>
							<Paragraph strong style={{ marginBottom: 4 }}>
								预期收益
							</Paragraph>
							<Paragraph style={{ marginBottom: 12 }}>{idea.expected_benefit}</Paragraph>
						</>
					)}

					{idea.original && (
						<>
							<Paragraph strong style={{ marginBottom: 4 }}>
								改写前原文
							</Paragraph>
							<Paragraph type="secondary" style={{ marginBottom: 12 }}>
								{idea.original.title} — {idea.original.problem}
							</Paragraph>
						</>
					)}

					<Paragraph strong style={{ marginBottom: 6 }}>
						证据（{idea.evidence.length}）
					</Paragraph>
					<Collapse
						size="small"
						items={[
							{
								key: 'evidence',
								label: '展开看原文摘录',
								children: (
									<Space direction="vertical" style={{ width: '100%' }} size={8}>
										{idea.evidence.map((e, index) => (
											<div key={index}>
												<Tag>{e.kind.replace('_ref', '')}</Tag>
												<Text
													style={{
														fontSize: 11,
														fontFamily: 'var(--font-mono)',
														display: 'block',
														marginTop: 2
													}}
												>
													{'path' in e
														? `${e.path}${e.line_range ? ':' + e.line_range : ''}`
														: 'id' in e
															? e.id
															: 'url' in e
																? e.url
																: `${e.stage}`}
												</Text>
												<Paragraph
													type="secondary"
													style={{
														fontSize: 12,
														marginTop: 4,
														marginBottom: 0,
														whiteSpace: 'pre-wrap',
														fontFamily: 'var(--font-mono)'
													}}
												>
													{e.excerpt}
												</Paragraph>
											</div>
										))}
									</Space>
								)
							}
						]}
					/>

					{idea.conflicts_with.length > 0 && (
						<>
							<Divider style={{ margin: '12px 0' }} />
							<Text type="warning" style={{ fontSize: 12 }}>
								与另外 {idea.conflicts_with.length} 条建议互斥，建议二选一。
							</Text>
						</>
					)}

					{rewriteOpen && (
						<>
							<Divider style={{ margin: '12px 0' }} />
							<Form form={form} layout="vertical" size="small">
								<Form.Item
									name="title"
									label="标题"
									rules={[{ required: true, message: '标题不能为空' }]}
								>
									<Input placeholder="动词短语，描述具体改动" />
								</Form.Item>
								<Form.Item
									name="problem"
									label="现状痛点"
									rules={[{ required: true, message: '问题描述不能为空' }]}
								>
									<Input.TextArea rows={2} />
								</Form.Item>
								<Form.Item
									name="hypothesis"
									label="验证假设"
									rules={[{ required: true, message: '假设不能为空' }]}
								>
									<Input.TextArea rows={2} />
								</Form.Item>
								<Form.Item name="scope" label="范围">
									<Select
										options={(Object.keys(SCOPE_LABEL) as IdeaScope[]).map(key => ({
											value: key,
											label: SCOPE_LABEL[key]
										}))}
									/>
								</Form.Item>
								<Button type="primary" onClick={handleRewrite}>
									保存改写
								</Button>
							</Form>
						</>
					)}

					{timeline.length > 0 && (
						<>
							<Divider style={{ margin: '12px 0' }} />
							<Paragraph strong style={{ marginBottom: 8 }}>
								决策历史
							</Paragraph>
							<Timeline
								items={timeline.map(entry => ({
									color: entry.reversed_by ? 'gray' : 'blue',
									children: (
										<div style={{ fontSize: 12 }}>
											<Space size={6}>
												<Tag>{ACTION_LABEL[entry.action] ?? entry.action}</Tag>
												<Text type="secondary">
													{new Date(entry.ts).toLocaleString()}
												</Text>
												{entry.reversed_by && <Tag color="default">已撤销</Tag>}
											</Space>
											{entry.comment && (
												<div style={{ marginTop: 4 }}>{entry.comment}</div>
											)}
											{entry.resulting_iteration_id && (
												<div style={{ marginTop: 4, color: 'var(--text-tertiary)' }}>
													迭代 {entry.resulting_iteration_id}
												</div>
											)}
										</div>
									)
								}))}
							/>
						</>
					)}
				</>
			)}
		</Drawer>
	);
};

export default IdeaDetailDrawer;
