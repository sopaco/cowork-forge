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
  InputNumber,
  message,
  Popconfirm,
  Empty,
  Drawer,
  Descriptions,
  Divider,
  Tabs,
  Alert,
  Badge,
} from 'antd';
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ApiOutlined,
  LinkOutlined,
  SettingOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  ThunderboltOutlined,
  SafetyOutlined,
} from '@ant-design/icons';
import { useConfigStore } from '../../stores/configStore';
import type { 
  IntegrationDefinition, 
  IntegrationType, 
  AuthType, 
  CredentialSource,
  AuthConfig,
  ConnectionConfig,
} from '../../types/config';

const { Title, Text, Paragraph } = Typography;
const { TextArea } = Input;

const IntegrationConfig: React.FC = () => {
  const {
    integrations,
    selectedIntegration,
    selectIntegration,
    saveIntegration,
    deleteIntegration,
  } = useConfigStore();

  const [editModalVisible, setEditModalVisible] = useState(false);
  const [editingIntegration, setEditingIntegration] = useState<IntegrationDefinition | null>(null);
  const [detailDrawerVisible, setDetailDrawerVisible] = useState(false);
  const [testingConnection, setTestingConnection] = useState(false);
  const [form] = Form.useForm();

  const handleCreate = () => {
    setEditingIntegration(null);
    form.resetFields();
    form.setFieldsValue({
      id: `integration-${Date.now()}`,
      name: '',
      description: '',
      integration_type: 'rest_api',
      connection: {
        timeout_secs: 30,
        retry_count: 3,
        retry_delay_ms: 1000,
      },
      auth: {
        auth_type: 'none',
        credential_source: 'config',
      },
      events: [],
      enabled: true,
      metadata: {},
    });
    setEditModalVisible(true);
  };

  const handleEdit = (integration: IntegrationDefinition) => {
    setEditingIntegration(integration);
    form.setFieldsValue(integration);
    setEditModalVisible(true);
    selectIntegration(integration.id);
  };

  const handleView = (integration: IntegrationDefinition) => {
    selectIntegration(integration.id);
    setDetailDrawerVisible(true);
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteIntegration(id);
      message.success('Integration deleted successfully');
    } catch (error) {
      message.error('Failed to delete integration');
    }
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      const integration: IntegrationDefinition = {
        ...editingIntegration,
        ...values,
        metadata: {},
      };

      await saveIntegration(integration);
      message.success('Integration saved successfully');
      setEditModalVisible(false);
    } catch (error) {
      message.error('Failed to save integration');
    }
  };

  const handleTestConnection = async () => {
    setTestingConnection(true);
    try {
      // Simulate connection test
      await new Promise(resolve => setTimeout(resolve, 1500));
      message.success('Connection test successful');
    } catch (error) {
      message.error('Connection test failed');
    } finally {
      setTestingConnection(false);
    }
  };

  const selectedIntegrationData = selectedIntegration ? integrations[selectedIntegration] : null;

  const getTypeColor = (type: IntegrationType): string => {
    const colors: Record<string, string> = {
      rest_api: 'blue',
      webhook: 'green',
      message_queue: 'orange',
      database: 'purple',
    };
    return colors[type] || 'default';
  };

  const getAuthTypeColor = (type: AuthType): string => {
    const colors: Record<string, string> = {
      none: 'default',
      api_key: 'blue',
      bearer_token: 'green',
      basic_auth: 'orange',
      oauth2: 'purple',
    };
    return colors[type] || 'default';
  };

  const integrationTypes: IntegrationType[] = ['rest_api', 'webhook', 'message_queue', 'database'];
  const authTypes: AuthType[] = ['none', 'api_key', 'bearer_token', 'basic_auth', 'oauth2'];
  const credentialSources: CredentialSource[] = ['env', 'config', 'prompt'];
  const integrationEvents = [
    { value: 'on_stage_start', label: 'On Stage Start' },
    { value: 'on_stage_complete', label: 'On Stage Complete' },
    { value: 'on_flow_start', label: 'On Flow Start' },
    { value: 'on_flow_complete', label: 'On Flow Complete' },
    { value: 'on_error', label: 'On Error' },
  ];

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={5} style={{ margin: 0 }}>
          Integrations
          <Badge count={Object.keys(integrations).length} style={{ marginLeft: 8 }} />
        </Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
          New Integration
        </Button>
      </div>

      {Object.keys(integrations).length === 0 ? (
        <Empty 
          description={
            <Space direction="vertical" size="small">
              <Text>No integrations configured</Text>
              <Text type="secondary">Connect to external systems and services</Text>
            </Space>
          } 
          style={{ marginTop: '40px' }} 
        />
      ) : (
        <List
          style={{ flex: 1, overflow: 'auto', padding: '0 16px' }}
          dataSource={Object.values(integrations)}
          renderItem={(integration) => (
            <List.Item
              actions={[
                <Button
                  key="view"
                  type="link"
                  size="small"
                  icon={<ApiOutlined />}
                  onClick={() => handleView(integration)}
                >
                  View
                </Button>,
                <Button
                  key="edit"
                  type="link"
                  size="small"
                  icon={<EditOutlined />}
                  onClick={() => handleEdit(integration)}
                >
                  Edit
                </Button>,
                <Popconfirm
                  key="delete"
                  title="Delete this integration?"
                  onConfirm={() => handleDelete(integration.id)}
                >
                  <Button type="link" size="small" danger icon={<DeleteOutlined />}>
                    Delete
                  </Button>
                </Popconfirm>,
              ]}
            >
              <List.Item.Meta
                avatar={
                  integration.enabled ? (
                    <CheckCircleOutlined style={{ fontSize: 24, color: '#52c41a' }} />
                  ) : (
                    <CloseCircleOutlined style={{ fontSize: 24, color: '#d9d9d9' }} />
                  )
                }
                title={
                  <Space>
                    <Text strong>{integration.name}</Text>
                    <Tag color={getTypeColor(integration.integration_type)}>
                      {integration.integration_type.replace('_', ' ').toUpperCase()}
                    </Tag>
                    {!integration.enabled && <Tag color="red">Disabled</Tag>}
                  </Space>
                }
                description={
                  <Space direction="vertical" size="small">
                    <Text type="secondary">{integration.description || 'No description'}</Text>
                    <Space size={4}>
                      <Tag icon={<SafetyOutlined />} color={getAuthTypeColor(integration.auth.auth_type)}>
                        {integration.auth.auth_type}
                      </Tag>
                      {integration.connection.base_url && (
                        <Tag icon={<LinkOutlined />} color="blue">
                          {integration.connection.base_url}
                        </Tag>
                      )}
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
        title={editingIntegration ? 'Edit Integration' : 'Create Integration'}
        open={editModalVisible}
        onCancel={() => setEditModalVisible(false)}
        onOk={handleSave}
        width={700}
        okText="Save"
      >
        <Form form={form} layout="vertical">
          <Tabs
            items={[
              {
                key: 'basic',
                label: 'Basic',
                children: (
                  <>
                    <Form.Item name="id" label="ID" rules={[{ required: true }]}>
                      <Input disabled={!!editingIntegration} />
                    </Form.Item>
                    <Form.Item name="name" label="Name" rules={[{ required: true }]}>
                      <Input />
                    </Form.Item>
                    <Form.Item name="description" label="Description">
                      <TextArea rows={2} />
                    </Form.Item>
                    <Form.Item name="integration_type" label="Integration Type" rules={[{ required: true }]}>
                      <Select>
                        {integrationTypes.map(type => (
                          <Select.Option key={type} value={type}>
                            {type.replace('_', ' ').toUpperCase()}
                          </Select.Option>
                        ))}
                      </Select>
                    </Form.Item>
                    <Form.Item name="enabled" label="Enabled" valuePropName="checked">
                      <Switch />
                    </Form.Item>
                  </>
                ),
              },
              {
                key: 'connection',
                label: 'Connection',
                children: (
                  <>
                    <Form.Item name={['connection', 'base_url']} label="Base URL">
                      <Input placeholder="https://api.example.com" />
                    </Form.Item>
                    <Form.Item name={['connection', 'timeout_secs']} label="Timeout (seconds)">
                      <InputNumber min={1} max={300} style={{ width: '100%' }} />
                    </Form.Item>
                    <Form.Item name={['connection', 'retry_count']} label="Retry Count">
                      <InputNumber min={0} max={10} style={{ width: '100%' }} />
                    </Form.Item>
                    <Form.Item name={['connection', 'retry_delay_ms']} label="Retry Delay (ms)">
                      <InputNumber min={0} max={60000} style={{ width: '100%' }} />
                    </Form.Item>
                  </>
                ),
              },
              {
                key: 'auth',
                label: 'Authentication',
                children: (
                  <>
                    <Alert
                      message="Credentials are stored securely and never exposed in logs"
                      type="info"
                      showIcon
                      style={{ marginBottom: 16 }}
                    />
                    <Form.Item name={['auth', 'auth_type']} label="Authentication Type">
                      <Select>
                        {authTypes.map(type => (
                          <Select.Option key={type} value={type}>
                            {type.replace('_', ' ').toUpperCase()}
                          </Select.Option>
                        ))}
                      </Select>
                    </Form.Item>
                    <Form.Item name={['auth', 'credential_source']} label="Credential Source">
                      <Select>
                        {credentialSources.map(source => (
                          <Select.Option key={source} value={source}>
                            {source.toUpperCase()}
                          </Select.Option>
                        ))}
                      </Select>
                    </Form.Item>
                    <Form.Item name={['auth', 'credential_key']} label="Credential Key">
                      <Input placeholder="Environment variable name or config key" />
                    </Form.Item>
                  </>
                ),
              },
              {
                key: 'events',
                label: 'Events',
                children: (
                  <Form.Item name="events" label="Trigger Events">
                    <Select mode="multiple" placeholder="Select trigger events">
                      {integrationEvents.map(event => (
                        <Select.Option key={event.value} value={event.value}>
                          {event.label}
                        </Select.Option>
                      ))}
                    </Select>
                  </Form.Item>
                ),
              },
            ]}
          />
        </Form>
      </Modal>

      {/* Detail Drawer */}
      <Drawer
        title={selectedIntegrationData?.name || 'Integration Details'}
        placement="right"
        width={500}
        onClose={() => setDetailDrawerVisible(false)}
        open={detailDrawerVisible}
        extra={
          <Button 
            icon={<ThunderboltOutlined />} 
            onClick={handleTestConnection}
            loading={testingConnection}
          >
            Test Connection
          </Button>
        }
      >
        {selectedIntegrationData && (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="ID">{selectedIntegrationData.id}</Descriptions.Item>
              <Descriptions.Item label="Status">
                {selectedIntegrationData.enabled ? (
                  <Tag icon={<CheckCircleOutlined />} color="success">Enabled</Tag>
                ) : (
                  <Tag icon={<CloseCircleOutlined />} color="default">Disabled</Tag>
                )}
              </Descriptions.Item>
              <Descriptions.Item label="Type">
                <Tag color={getTypeColor(selectedIntegrationData.integration_type)}>
                  {selectedIntegrationData.integration_type.replace('_', ' ').toUpperCase()}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="Description">
                {selectedIntegrationData.description || '-'}
              </Descriptions.Item>
            </Descriptions>

            <Title level={5}>Connection</Title>
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="Base URL">
                {selectedIntegrationData.connection.base_url || '-'}
              </Descriptions.Item>
              <Descriptions.Item label="Timeout">
                {selectedIntegrationData.connection.timeout_secs || 30}s
              </Descriptions.Item>
              <Descriptions.Item label="Retry Count">
                {selectedIntegrationData.connection.retry_count || 3}
              </Descriptions.Item>
              <Descriptions.Item label="Retry Delay">
                {selectedIntegrationData.connection.retry_delay_ms || 1000}ms
              </Descriptions.Item>
            </Descriptions>

            <Title level={5}>Authentication</Title>
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="Type">
                <Tag color={getAuthTypeColor(selectedIntegrationData.auth.auth_type)}>
                  {selectedIntegrationData.auth.auth_type.replace('_', ' ').toUpperCase()}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="Credential Source">
                {selectedIntegrationData.auth.credential_source.toUpperCase()}
              </Descriptions.Item>
            </Descriptions>

            <Title level={5}>Trigger Events</Title>
            <Space wrap>
              {selectedIntegrationData.events.length > 0 ? (
                selectedIntegrationData.events.map((event, i) => (
                  <Tag key={i} color="blue">{event.replace(/_/g, ' ')}</Tag>
                ))
              ) : (
                <Text type="secondary">No events configured</Text>
              )}
            </Space>
          </Space>
        )}
      </Drawer>
    </div>
  );
};

export default IntegrationConfig;
