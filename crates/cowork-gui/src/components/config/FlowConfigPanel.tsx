import React, { useState } from 'react';
import {
  Card,
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
} from 'antd';
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ExportOutlined,
  ImportOutlined,
  PlayCircleOutlined,
  SettingOutlined,
  OrderedListOutlined,
} from '@ant-design/icons';
import { useConfigStore } from '../../stores/configStore';
import type { FlowDefinition, StageReference } from '../../types/config';

const { Title, Text, Paragraph } = Typography;
const { TextArea } = Input;

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
  } = useConfigStore();

  const [editModalVisible, setEditModalVisible] = useState(false);
  const [editingFlow, setEditingFlow] = useState<FlowDefinition | null>(null);
  const [detailDrawerVisible, setDetailDrawerVisible] = useState(false);
  const [importModalVisible, setImportModalVisible] = useState(false);
  const [importJson, setImportJson] = useState('');
  const [form] = Form.useForm();

  const handleCreate = () => {
    setEditingFlow(null);
    form.resetFields();
    form.setFieldsValue({
      id: `flow-${Date.now()}`,
      name: '',
      description: '',
      stages: [],
      config: {
        stop_on_failure: true,
        save_state_on_interrupt: true,
        memory_scope: 'merged',
      },
    });
    setEditModalVisible(true);
  };

  const handleEdit = (flow: FlowDefinition) => {
    setEditingFlow(flow);
    form.setFieldsValue(flow);
    setEditModalVisible(true);
    selectFlow(flow.id);
  };

  const handleView = (flow: FlowDefinition) => {
    selectFlow(flow.id);
    setDetailDrawerVisible(true);
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteFlow(id);
      message.success('Flow deleted successfully');
    } catch (error) {
      message.error('Failed to delete flow');
    }
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      const flow: FlowDefinition = {
        ...editingFlow,
        ...values,
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
    } catch (error) {
      message.error('Failed to save flow');
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
    const stage = stages[stageId];
    return colors[stage?.stage_type || ''] || 'default';
  };

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={5} style={{ margin: 0 }}>Flow Definitions</Title>
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
        <List
          style={{ flex: 1, overflow: 'auto', padding: '0 16px' }}
          dataSource={Object.values(flows).sort((a, b) => a.name.localeCompare(b.name))}
          renderItem={(flow) => (
            <List.Item
              actions={[
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
                  key="edit"
                  type="link"
                  size="small"
                  icon={<EditOutlined />}
                  onClick={() => handleEdit(flow)}
                >
                  Edit
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
                <Popconfirm
                  key="delete"
                  title="Delete this flow?"
                  onConfirm={() => handleDelete(flow.id)}
                >
                  <Button type="link" size="small" danger icon={<DeleteOutlined />}>
                    Delete
                  </Button>
                </Popconfirm>,
              ]}
            >
              <List.Item.Meta
                title={
                  <Space>
                    {flow.name}
                    {flow.version && <Tag>{flow.version}</Tag>}
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
          )}
        />
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
            <Input />
          </Form.Item>
          <Form.Item name="description" label="Description">
            <TextArea rows={2} />
          </Form.Item>
          <Form.Item name="version" label="Version">
            <Input placeholder="e.g., 1.0.0" />
          </Form.Item>
          <Divider>Stages</Divider>
          <Alert
            message="Stage configuration is managed through the stage definitions"
            type="info"
            showIcon
            style={{ marginBottom: 16 }}
          />
          <Form.Item name={['config', 'stop_on_failure']} label="Stop on Failure" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item name={['config', 'save_state_on_interrupt']} label="Save State on Interrupt" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item name={['config', 'memory_scope']} label="Memory Scope">
            <Select>
              <Select.Option value="project">Project</Select.Option>
              <Select.Option value="iteration">Iteration</Select.Option>
              <Select.Option value="merged">Merged</Select.Option>
            </Select>
          </Form.Item>
        </Form>
      </Modal>

      {/* Detail Drawer */}
      <Drawer
        title={selectedFlowData?.name || 'Flow Details'}
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
                      {stage.alias || stage.stage_id}
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
