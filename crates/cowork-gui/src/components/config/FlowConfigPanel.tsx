import React, { useState, useMemo } from 'react';
import {
  List,
  Button,
  Space,
  Typography,
  Tag,
  Modal,
  Form,
  Input,
  Select,
  Switch,
  message,
  Popconfirm,
  Empty,
  Drawer,
  Descriptions,
  Divider,
  Alert,
  Collapse,
  Transfer,
  Tooltip,
} from 'antd';
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ExportOutlined,
  ImportOutlined,
  PlayCircleOutlined,
  LockOutlined,
  MenuOutlined,
  CheckCircleOutlined,
  StarOutlined,
  StarFilled,
} from '@ant-design/icons';
import { useConfigStore } from '../../stores/configStore';
import type { FlowDefinition, StageReference } from '../../types/config';

const { Title, Text } = Typography;
const { TextArea } = Input;
const { Panel } = Collapse;

// Available stages for selection
const AVAILABLE_STAGES = [
  { key: 'idea', title: 'Idea', description: 'Capture and refine the project idea' },
  { key: 'prd', title: 'PRD', description: 'Product Requirements Document' },
  { key: 'design', title: 'Design', description: 'System design and architecture' },
  { key: 'plan', title: 'Plan', description: 'Implementation planning' },
  { key: 'coding', title: 'Coding', description: 'Code implementation' },
  { key: 'check', title: 'Check', description: 'Quality validation' },
  { key: 'delivery', title: 'Delivery', description: 'Deployment and delivery' },
];

const FlowConfigPanel: React.FC = () => {
  const {
    flows,
    stages,
    selectedFlow,
    selectFlow,
    saveFlow,
    deleteFlow,
    validateFlow,
    exportConfig,
    importConfig,
    default_flow_id,
    setDefaultFlow,
  } = useConfigStore();

  const [editModalVisible, setEditModalVisible] = useState(false);
  const [editingFlow, setEditingFlow] = useState<FlowDefinition | null>(null);
  const [detailDrawerVisible, setDetailDrawerVisible] = useState(false);
  const [importModalVisible, setImportModalVisible] = useState(false);
  const [importJson, setImportJson] = useState('');
  const [selectedStages, setSelectedStages] = useState<string[]>([]);
  const [form] = Form.useForm();

  // Separate flows into built-in and custom
  const { builtinFlows, customFlows } = useMemo(() => {
    const builtin: FlowDefinition[] = [];
    const custom: FlowDefinition[] = [];
    
    Object.values(flows).forEach(flow => {
      if (flow.is_builtin) {
        builtin.push(flow);
      } else {
        custom.push(flow);
      }
    });
    
    // Sort by name
    builtin.sort((a, b) => a.name.localeCompare(b.name));
    custom.sort((a, b) => a.name.localeCompare(b.name));
    
    return { builtinFlows: builtin, customFlows: custom };
  }, [flows]);

  const handleCreate = () => {
    setEditingFlow(null);
    form.resetFields();
    // Default to all stages
    const defaultStages = AVAILABLE_STAGES.map(s => s.key);
    setSelectedStages(defaultStages);
    form.setFieldsValue({
      id: `flow-${Date.now()}`,
      name: '',
      description: '',
      version: '1.0.0',
      config: {
        stop_on_failure: true,
        save_state_on_interrupt: true,
        memory_scope: 'merged',
        inheritance: {
          default_mode: 'partial',
          stage_mapping: {
            none: 'idea',
            partial: 'plan',
            full: 'idea',
          },
        },
      },
    });
    setEditModalVisible(true);
  };

  const handleEdit = (flow: FlowDefinition) => {
    if (flow.is_builtin) {
      message.warning('Built-in flows cannot be edited. Create a new flow instead.');
      return;
    }
    setEditingFlow(flow);
    form.setFieldsValue(flow);
    setSelectedStages(flow.stages.map(s => s.stage_id));
    setEditModalVisible(true);
    selectFlow(flow.id);
  };

  const handleView = (flow: FlowDefinition) => {
    selectFlow(flow.id);
    setDetailDrawerVisible(true);
  };

  const handleDelete = async (id: string) => {
    const flow = flows[id];
    if (flow?.is_builtin) {
      message.error('Cannot delete built-in flow');
      return;
    }
    try {
      await deleteFlow(id);
      message.success('Flow deleted successfully');
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to delete flow';
      message.error(errorMessage);
    }
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      
      // Build stages array from selected stages
      const flowStages: StageReference[] = selectedStages.map((stageId, index) => ({
        stage_id: stageId,
        alias: undefined,
        overrides: {
          needs_confirmation: undefined,
          hooks: [],
          timeout_secs: undefined,
          skip: false,
        },
        condition: undefined,
        on_success: index < selectedStages.length - 1 ? selectedStages[index + 1] : undefined,
        on_failure: undefined,
      }));

      const flow: FlowDefinition = {
        ...editingFlow,
        ...values,
        stages: flowStages,
        start_stage: selectedStages[0] || undefined,
      };
      
      const validation = await validateFlow(flow);
      if (!validation.valid) {
        const errors = validation.issues.map(i => i.message).join(', ');
        message.error(`Validation failed: ${errors}`);
        return;
      }

      await saveFlow(flow);
      message.success('Flow saved successfully');
      setEditModalVisible(false);
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to save flow';
      message.error(errorMessage);
    }
  };

  const handleExport = async (id: string) => {
    try {
      const json = await exportConfig('flow', id);
      navigator.clipboard.writeText(json);
      message.success('Flow exported to clipboard');
    } catch (error) {
      message.error('Failed to export flow');
    }
  };

  const handleImport = async () => {
    try {
      await importConfig('flow', importJson);
      message.success('Flow imported successfully');
      setImportModalVisible(false);
      setImportJson('');
    } catch (error) {
      message.error('Failed to import flow');
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await setDefaultFlow(id);
      message.success('Default flow updated');
    } catch (error) {
      message.error('Failed to set default flow');
    }
  };

  const selectedFlowData = selectedFlow ? flows[selectedFlow] : null;

  const getStageTypeColor = (stageId: string) => {
    const colors: Record<string, string> = {
      idea: 'blue',
      prd: 'green',
      design: 'purple',
      plan: 'orange',
      coding: 'cyan',
      check: 'gold',
      delivery: 'lime',
    };
    return colors[stageId] || 'default';
  };

  const getStageTitle = (stageId: string) => {
    const stage = AVAILABLE_STAGES.find(s => s.key === stageId);
    return stage?.title || stageId;
  };

  const renderFlowItem = (flow: FlowDefinition) => {
    const isBuiltin = flow.is_builtin;
    const isDefault = default_flow_id === flow.id;
    
    const actions = [
      <Tooltip key="default" title={isDefault ? 'This is the default flow' : 'Set as default flow'}>
        <Button
          type="link"
          size="small"
          icon={isDefault ? <StarFilled style={{ color: '#faad14' }} /> : <StarOutlined />}
          onClick={() => !isDefault && handleSetDefault(flow.id)}
          disabled={isDefault}
        >
          {isDefault ? 'Default' : 'Set Default'}
        </Button>
      </Tooltip>,
      <Button
        key="view"
        type="link"
        size="small"
        icon={<PlayCircleOutlined />}
        onClick={() => handleView(flow)}
      >
        View
      </Button>,
      <Button
        key="export"
        type="link"
        size="small"
        icon={<ExportOutlined />}
        onClick={() => handleExport(flow.id)}
      >
        Export
      </Button>,
    ];

    // Only show Edit and Delete for custom flows
    if (!isBuiltin) {
      actions.push(
        <Button
          key="edit"
          type="link"
          size="small"
          icon={<EditOutlined />}
          onClick={() => handleEdit(flow)}
        >
          Edit
        </Button>,
        <Popconfirm
          key="delete"
          title="Delete this flow?"
          onConfirm={() => handleDelete(flow.id)}
        >
          <Button type="link" size="small" danger icon={<DeleteOutlined />}>
            Delete
          </Button>
        </Popconfirm>
      );
    }

    return (
      <List.Item actions={actions}>
        <List.Item.Meta
          title={
            <Space>
              {flow.name}
              {flow.version && <Tag>{flow.version}</Tag>}
              {isDefault && (
                <Tag color="gold" icon={<StarFilled />}>
                  Default
                </Tag>
              )}
              {isBuiltin && (
                <Tag color="blue" icon={<LockOutlined />}>
                  Built-in
                </Tag>
              )}
            </Space>
          }
          description={
            <Space direction="vertical" size="small">
              <Text type="secondary">{flow.description || 'No description'}</Text>
              <Space size={4}>
                {flow.stages.map((s, i) => (
                  <Tag key={i} color={getStageTypeColor(s.stage_id)}>
                    {s.alias || s.stage_id}
                  </Tag>
                ))}
              </Space>
            </Space>
          }
        />
      </List.Item>
    );
  };

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Space>
          <Title level={5} style={{ margin: 0 }}>Flow Definitions</Title>
          {default_flow_id && flows[default_flow_id] && (
            <Tag color="gold" icon={<StarFilled />}>
              Active: {flows[default_flow_id].name}
            </Tag>
          )}
        </Space>
        <Space>
          <Button icon={<ImportOutlined />} onClick={() => setImportModalVisible(true)}>
            Import
          </Button>
          <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
            New Flow
          </Button>
        </Space>
      </div>

      {Object.keys(flows).length === 0 ? (
        <Empty description="No flows defined" style={{ marginTop: '40px' }} />
      ) : (
        <div style={{ flex: 1, overflow: 'auto', padding: '0 16px' }}>
          <Collapse defaultActiveKey={['builtin', 'custom']} ghost>
            {builtinFlows.length > 0 && (
              <Panel 
                header={
                  <Space>
                    <LockOutlined />
                    <Text strong>Preset Flows ({builtinFlows.length})</Text>
                    <Text type="secondary">- Read only</Text>
                  </Space>
                } 
                key="builtin"
              >
                <Alert
                  message="Preset flows are built-in configurations that cannot be modified. You can view or export them, or create a new custom flow based on them."
                  type="info"
                  showIcon
                  style={{ marginBottom: 12 }}
                />
                <List
                  dataSource={builtinFlows}
                  renderItem={renderFlowItem}
                />
              </Panel>
            )}
            
            {customFlows.length > 0 && (
              <Panel 
                header={
                  <Space>
                    <EditOutlined />
                    <Text strong>Custom Flows ({customFlows.length})</Text>
                  </Space>
                } 
                key="custom"
              >
                <List
                  dataSource={customFlows}
                  renderItem={renderFlowItem}
                />
              </Panel>
            )}
            
            {builtinFlows.length === 0 && customFlows.length === 0 && Object.keys(flows).length > 0 && (
              <Alert
                message="Flows loaded but type is unknown. Try refreshing the configuration."
                type="warning"
                showIcon
              />
            )}
          </Collapse>
        </div>
      )}

      {/* Edit Modal */}
      <Modal
        title={editingFlow ? 'Edit Flow' : 'Create Flow'}
        open={editModalVisible}
        onCancel={() => setEditModalVisible(false)}
        onOk={handleSave}
        width={700}
        okText="Save"
      >
        <Form form={form} layout="vertical">
          <Form.Item name="id" label="ID" rules={[{ required: true }]}>
            <Input disabled={!!editingFlow} />
          </Form.Item>
          <Form.Item name="name" label="Name" rules={[{ required: true }]}>
            <Input placeholder="e.g., My Custom Flow" />
          </Form.Item>
          <Form.Item name="description" label="Description">
            <TextArea rows={2} placeholder="Brief description of this flow" />
          </Form.Item>
          <Form.Item name="version" label="Version">
            <Input placeholder="e.g., 1.0.0" />
          </Form.Item>
          
          <Divider>Stage Sequence</Divider>
          <Alert
            message="Select and order the stages for this flow. Stages will execute in the selected order."
            type="info"
            showIcon
            style={{ marginBottom: 16 }}
          />
          
          <Form.Item label="Stages">
            <Transfer
              dataSource={AVAILABLE_STAGES}
              titles={['Available Stages', 'Selected Stages (Execution Order)']}
              targetKeys={selectedStages}
              onChange={(newTargetKeys) => setSelectedStages(newTargetKeys as string[])}
              render={item => (
                <Space>
                  <Tag color={getStageTypeColor(item.key)}>{item.title}</Tag>
                  <Text type="secondary" style={{ fontSize: 12 }}>{item.description}</Text>
                </Space>
              )}
              listStyle={{ width: 280, height: 300 }}
              showSearch
              filterOption={(input, item) =>
                item.title.toLowerCase().includes(input.toLowerCase()) ||
                item.description.toLowerCase().includes(input.toLowerCase())
              }
            />
          </Form.Item>
          
          <Divider>Configuration</Divider>
          <Form.Item name={['config', 'stop_on_failure']} label="Stop on Failure" valuePropName="checked" tooltip="Stop the entire flow when a stage fails">
            <Switch />
          </Form.Item>
          <Form.Item name={['config', 'save_state_on_interrupt']} label="Save State on Interrupt" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item name={['config', 'memory_scope']} label="Memory Scope">
            <Select>
              <Select.Option value="project">Project (shared across iterations)</Select.Option>
              <Select.Option value="iteration">Iteration (isolated)</Select.Option>
              <Select.Option value="merged">Merged (project + iteration)</Select.Option>
            </Select>
          </Form.Item>
          <Form.Item name={['config', 'inheritance', 'default_mode']} label="Default Inheritance Mode">
            <Select>
              <Select.Option value="none">None (fresh start)</Select.Option>
              <Select.Option value="partial">Partial (code only)</Select.Option>
              <Select.Option value="full">Full (code + artifacts)</Select.Option>
            </Select>
          </Form.Item>
        </Form>
      </Modal>

      {/* Detail Drawer */}
      <Drawer
        title={
          <Space>
            {selectedFlowData?.name || 'Flow Details'}
            {selectedFlowData?.is_builtin && <Tag color="blue" icon={<LockOutlined />}>Built-in</Tag>}
          </Space>
        }
        placement="right"
        width={500}
        onClose={() => setDetailDrawerVisible(false)}
        open={detailDrawerVisible}
      >
        {selectedFlowData && (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="ID">{selectedFlowData.id}</Descriptions.Item>
              <Descriptions.Item label="Version">{selectedFlowData.version || '-'}</Descriptions.Item>
              <Descriptions.Item label="Description">
                {selectedFlowData.description || '-'}
              </Descriptions.Item>
              <Descriptions.Item label="Start Stage">
                {selectedFlowData.start_stage || 'First stage'}
              </Descriptions.Item>
              <Descriptions.Item label="Type">
                {selectedFlowData.is_builtin ? (
                  <Tag color="blue" icon={<LockOutlined />}>Built-in Preset</Tag>
                ) : (
                  <Tag color="green">Custom</Tag>
                )}
              </Descriptions.Item>
            </Descriptions>

            <Title level={5}>Stage Sequence</Title>
            <List
              size="small"
              dataSource={selectedFlowData.stages}
              renderItem={(stage: StageReference, index: number) => (
                <List.Item>
                  <Space>
                    <Tag>{index + 1}</Tag>
                    <Tag color={getStageTypeColor(stage.stage_id)}>
                      {stage.alias || getStageTitle(stage.stage_id)}
                    </Tag>
                    {stage.overrides.skip && <Tag color="red">Skipped</Tag>}
                    {stage.condition && <Text type="secondary">if: {stage.condition}</Text>}
                  </Space>
                </List.Item>
              )}
            />

            <Title level={5}>Configuration</Title>
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="Stop on Failure">
                {selectedFlowData.config.stop_on_failure ? 'Yes' : 'No'}
              </Descriptions.Item>
              <Descriptions.Item label="Memory Scope">
                {selectedFlowData.config.memory_scope}
              </Descriptions.Item>
              <Descriptions.Item label="Inheritance Mode">
                {selectedFlowData.config.inheritance.default_mode}
              </Descriptions.Item>
            </Descriptions>
          </Space>
        )}
      </Drawer>

      {/* Import Modal */}
      <Modal
        title="Import Flow"
        open={importModalVisible}
        onCancel={() => setImportModalVisible(false)}
        onOk={handleImport}
        okText="Import"
      >
        <TextArea
          rows={10}
          placeholder="Paste flow JSON here..."
          value={importJson}
          onChange={(e) => setImportJson(e.target.value)}
        />
      </Modal>
    </div>
  );
};

export default FlowConfigPanel;