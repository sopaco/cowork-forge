import React, { useState, useEffect } from 'react';
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
  Slider,
  message,
  Popconfirm,
  Empty,
  Drawer,
  Descriptions,
  Divider,
  Tabs,
  InputNumber,
  Alert,
  Tooltip,
} from 'antd';
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ExportOutlined,
  ImportOutlined,
  RobotOutlined,
  ToolOutlined,
  CodeOutlined,
  SettingOutlined,
  InfoCircleOutlined,
  FolderOpenOutlined,
} from '@ant-design/icons';
import { useConfigStore } from '../../stores/configStore';
import type { AgentDefinition, ToolReference, AgentType, ModelConfig, BuiltinInstruction, InstructionType } from '../../types/config';
import { open } from '@tauri-apps/plugin-dialog';

const { Title, Text, Paragraph } = Typography;
const { TextArea } = Input;

const AgentConfigForm: React.FC = () => {
  const {
    agents,
    skills,
    selectedAgent,
    selectAgent,
    saveAgent,
    deleteAgent,
    validateAgent,
    exportConfig,
    importConfig,
    getBuiltinInstructions,
  } = useConfigStore();

  const [editModalVisible, setEditModalVisible] = useState(false);
  const [editingAgent, setEditingAgent] = useState<AgentDefinition | null>(null);
  const [detailDrawerVisible, setDetailDrawerVisible] = useState(false);
  const [importModalVisible, setImportModalVisible] = useState(false);
  const [importJson, setImportJson] = useState('');
  const [form] = Form.useForm();
  
  // Instruction related state
  const [builtinInstructions, setBuiltinInstructions] = useState<BuiltinInstruction[]>([]);
  const [instructionType, setInstructionType] = useState<InstructionType>('builtin');
  const [selectedBuiltinId, setSelectedBuiltinId] = useState<string>('');
  const [instructionFilePath, setInstructionFilePath] = useState<string>('');
  const [instructionInlineContent, setInstructionInlineContent] = useState<string>('');
  
  // Load builtin instructions on mount
  useEffect(() => {
    getBuiltinInstructions().then(setBuiltinInstructions);
  }, [getBuiltinInstructions]);

  const handleCreate = () => {
    setEditingAgent(null);
    form.resetFields();
    form.setFieldsValue({
      id: `agent-${Date.now()}`,
      name: '',
      description: '',
      agent_type: 'simple',
      instruction: '',
      tools: [],
      skills: [],
      model: {
        temperature: 0.7,
      },
      include_contents: 'none',
      tags: [],
    });
    // Reset instruction state
    setInstructionType('builtin');
    setSelectedBuiltinId('');
    setInstructionFilePath('');
    setInstructionInlineContent('');
    setEditModalVisible(true);
  };

  // Parse instruction string to determine type and extract content
  const parseInstruction = (instruction: string): { type: InstructionType; builtinId: string; filePath: string; content: string } => {
    if (instruction.startsWith('builtin://')) {
      return {
        type: 'builtin',
        builtinId: instruction.substring('builtin://'.length),
        filePath: '',
        content: '',
      };
    } else if (instruction.startsWith('file://')) {
      return {
        type: 'file',
        builtinId: '',
        filePath: instruction.substring('file://'.length),
        content: '',
      };
    } else if (instruction.startsWith('inline://')) {
      return {
        type: 'inline',
        builtinId: '',
        filePath: '',
        content: instruction.substring('inline://'.length),
      };
    } else {
      // Treat as builtin name or inline content
      // Check if it matches a builtin id
      const matchingBuiltin = builtinInstructions.find(bi => bi.id === instruction);
      if (matchingBuiltin) {
        return {
          type: 'builtin',
          builtinId: instruction,
          filePath: '',
          content: '',
        };
      }
      // Otherwise treat as inline
      return {
        type: 'inline',
        builtinId: '',
        filePath: '',
        content: instruction,
      };
    }
  };

  const handleEdit = (agent: AgentDefinition) => {
    setEditingAgent(agent);
    // Convert ToolReference[] to string[] for Select component
    const toolIds = agent.tools.map(t => t.tool_id);
    form.setFieldsValue({
      ...agent,
      agent_type: typeof agent.agent_type === 'string' ? agent.agent_type : 'loop',
      tools: toolIds,
    });
    
    // Parse and set instruction state
    const parsed = parseInstruction(agent.instruction);
    setInstructionType(parsed.type);
    setSelectedBuiltinId(parsed.builtinId);
    setInstructionFilePath(parsed.filePath);
    
    // For inline, use the content directly; for builtin being converted to inline, use builtin content
    if (parsed.type === 'inline' && parsed.content) {
      setInstructionInlineContent(parsed.content);
    } else if (parsed.type === 'builtin' && parsed.builtinId) {
      const builtin = builtinInstructions.find(bi => bi.id === parsed.builtinId);
      setInstructionInlineContent(builtin?.content || '');
    } else {
      setInstructionInlineContent(parsed.content);
    }
    
    setEditModalVisible(true);
    selectAgent(agent.id);
  };

  const handleView = (agent: AgentDefinition) => {
    selectAgent(agent.id);
    setDetailDrawerVisible(true);
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteAgent(id);
      message.success('Agent deleted successfully');
    } catch (error) {
      message.error('Failed to delete agent');
    }
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      
      // Convert string[] to ToolReference[] for backend
      const tools: ToolReference[] = (values.tools || []).map((toolId: string) => ({
        tool_id: toolId,
      }));
      
      // Build instruction based on type
      let instruction = '';
      switch (instructionType) {
        case 'builtin':
          instruction = `builtin://${selectedBuiltinId}`;
          break;
        case 'file':
          instruction = `file://${instructionFilePath}`;
          break;
        case 'inline':
          instruction = `inline://${instructionInlineContent}`;
          break;
      }
      
      const agent: AgentDefinition = {
        ...editingAgent,
        ...values,
        instruction,
        tools,
        metadata: {},
      };

      const validation = await validateAgent(agent);
      if (!validation.valid) {
        const errors = validation.issues.map(i => i.message).join(', ');
        message.error(`Validation failed: ${errors}`);
        return;
      }

      await saveAgent(agent);
      message.success('Agent saved successfully');
      setEditModalVisible(false);
    } catch (error) {
      message.error('Failed to save agent');
    }
  };

  const handleExport = async (id: string) => {
    try {
      const json = await exportConfig('agent', id);
      navigator.clipboard.writeText(json);
      message.success('Agent exported to clipboard');
    } catch (error) {
      message.error('Failed to export agent');
    }
  };

  const handleImport = async () => {
    try {
      await importConfig('agent', importJson);
      message.success('Agent imported successfully');
      setImportModalVisible(false);
      setImportJson('');
    } catch (error) {
      message.error('Failed to import agent');
    }
  };

  const selectedAgentData = selectedAgent ? agents[selectedAgent] : null;

  const getAgentTypeTag = (type: AgentType) => {
    if (typeof type === 'string') {
      return <Tag color="blue">{type}</Tag>;
    }
    return <Tag color="purple">loop ({(type as { loop: { max_iterations?: number } }).loop?.max_iterations || 'unlimited'})</Tag>;
  };

  // Handle file selection for instruction file path
  const handleSelectInstructionFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Markdown', extensions: ['md', 'txt'] }],
      });
      if (selected && typeof selected === 'string') {
        setInstructionFilePath(selected);
      }
    } catch (error) {
      console.error('Failed to select file:', error);
    }
  };

  // Handle builtin selection - update inline content preview
  const handleBuiltinChange = (builtinId: string) => {
    setSelectedBuiltinId(builtinId);
    const builtin = builtinInstructions.find(bi => bi.id === builtinId);
    if (builtin) {
      setInstructionInlineContent(builtin.content);
    }
  };

  // Handle instruction type change
  const handleInstructionTypeChange = (type: InstructionType) => {
    setInstructionType(type);
    // When switching to inline, pre-fill with selected builtin content if available
    if (type === 'inline' && selectedBuiltinId) {
      const builtin = builtinInstructions.find(bi => bi.id === selectedBuiltinId);
      if (builtin) {
        setInstructionInlineContent(builtin.content);
      }
    }
  };

  const availableTools = [
    'read_file', 'write_file', 'edit_file', 'delete_file',
    'execute_command', 'search_file', 'search_content',
    'list_dir', 'create_dir', 'move_file', 'copy_file',
    'query_memory', 'save_memory', 'web_search', 'web_fetch',
  ];

  const availableSkills = Object.keys(skills);

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={5} style={{ margin: 0 }}>Agent Definitions</Title>
        <Space>
          <Button icon={<ImportOutlined />} onClick={() => setImportModalVisible(true)}>
            Import
          </Button>
          <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
            New Agent
          </Button>
        </Space>
      </div>

      {Object.keys(agents).length === 0 ? (
        <Empty description="No agents defined" style={{ marginTop: '40px' }} />
      ) : (
        <List
          style={{ flex: 1, overflow: 'auto', padding: '0 16px' }}
          dataSource={Object.values(agents).sort((a, b) => a.name.localeCompare(b.name))}
          renderItem={(agent) => (
            <List.Item
              actions={[
                <Button
                  key="view"
                  type="link"
                  size="small"
                  icon={<RobotOutlined />}
                  onClick={() => handleView(agent)}
                >
                  View
                </Button>,
                <Button
                  key="edit"
                  type="link"
                  size="small"
                  icon={<EditOutlined />}
                  onClick={() => handleEdit(agent)}
                >
                  Edit
                </Button>,
                <Button
                  key="export"
                  type="link"
                  size="small"
                  icon={<ExportOutlined />}
                  onClick={() => handleExport(agent.id)}
                >
                  Export
                </Button>,
                <Popconfirm
                  key="delete"
                  title="Delete this agent?"
                  onConfirm={() => handleDelete(agent.id)}
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
                    {agent.name}
                    {agent.version && <Tag>{agent.version}</Tag>}
                    {getAgentTypeTag(agent.agent_type)}
                  </Space>
                }
                description={
                  <Space direction="vertical" size="small">
                    <Text type="secondary">{agent.description || 'No description'}</Text>
                    <Space size={4}>
                      {agent.tools.slice(0, 5).map((t, i) => (
                        <Tag key={i} color="blue" style={{ fontSize: '11px' }}>{t.tool_id}</Tag>
                      ))}
                      {agent.tools.length > 5 && <Tag>+{agent.tools.length - 5}</Tag>}
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
        title={editingAgent ? 'Edit Agent' : 'Create Agent'}
        open={editModalVisible}
        onCancel={() => setEditModalVisible(false)}
        onOk={handleSave}
        width={800}
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
                      <Input disabled={!!editingAgent} />
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
                    <Form.Item name="agent_type" label="Agent Type">
                      <Select>
                        <Select.Option value="simple">Simple (Single Execution)</Select.Option>
                        <Select.Option value="loop">Loop (Actor-Critic)</Select.Option>
                      </Select>
                    </Form.Item>
                    <Form.Item 
                      name={['model', 'temperature']} 
                      label={
                        <span>
                          Temperature{' '}
                          <Tooltip title="Controls randomness. Lower = more deterministic, Higher = more creative">
                            <InfoCircleOutlined />
                          </Tooltip>
                        </span>
                      }
                    >
                      <Slider min={0} max={2} step={0.1} />
                    </Form.Item>
                    <Form.Item name={['model', 'max_tokens']} label="Max Tokens">
                      <InputNumber min={100} max={32000} style={{ width: '100%' }} />
                    </Form.Item>
                  </>
                ),
              },
              {
                key: 'instruction',
                label: 'Instruction',
                children: (
                  <div style={{ maxWidth: 700 }}>
                    <Alert
                      message="Select an instruction type to define how the agent should behave"
                      type="info"
                      showIcon
                      style={{ marginBottom: 16 }}
                    />
                    
                    <Form.Item label="Instruction Type">
                      <Select 
                        value={instructionType} 
                        onChange={handleInstructionTypeChange}
                        style={{ width: '100%' }}
                      >
                        <Select.Option value="builtin">
                          <Space>
                            <Tag color="blue">Built-in</Tag>
                            <span>Use a predefined instruction template</span>
                          </Space>
                        </Select.Option>
                        <Select.Option value="file">
                          <Space>
                            <Tag color="green">File</Tag>
                            <span>Load instruction from a file</span>
                          </Space>
                        </Select.Option>
                        <Select.Option value="inline">
                          <Space>
                            <Tag color="purple">Inline</Tag>
                            <span>Write custom instruction content</span>
                          </Space>
                        </Select.Option>
                      </Select>
                    </Form.Item>
                    
                    {instructionType === 'builtin' && (
                      <>
                        <Form.Item 
                          label="Built-in Instruction"
                          required
                          validateStatus={!selectedBuiltinId && 'error'}
                          help={!selectedBuiltinId && 'Please select a built-in instruction'}
                        >
                          <Select
                            value={selectedBuiltinId || undefined}
                            onChange={handleBuiltinChange}
                            placeholder="Select a built-in instruction..."
                            showSearch
                            optionFilterProp="label"
                            style={{ width: '100%' }}
                            listHeight={300}
                          >
                            {(builtinInstructions || []).map(bi => (
                              <Select.Option 
                                key={bi.id} 
                                value={bi.id}
                                label={bi.name}
                              >
                                <div style={{ padding: '4px 0' }}>
                                  <Text strong>{bi.name}</Text>
                                  <br />
                                  <Text type="secondary" style={{ fontSize: 12 }}>
                                    {bi.description}
                                  </Text>
                                </div>
                              </Select.Option>
                            ))}
                          </Select>
                        </Form.Item>
                        
                        {selectedBuiltinId && (
                          <div style={{ marginBottom: 16 }}>
                            <Text type="secondary" style={{ fontSize: 12, marginBottom: 8, display: 'block' }}>
                              Instruction Preview:
                            </Text>
                            <div
                              style={{
                                background: '#fafafa',
                                border: '1px solid #d9d9d9',
                                borderRadius: 6,
                                padding: 12,
                                maxHeight: 200,
                                overflow: 'auto',
                                fontSize: 12,
                                whiteSpace: 'pre-wrap',
                                fontFamily: 'monospace',
                              }}
                            >
                              {(builtinInstructions || []).find(bi => bi.id === selectedBuiltinId)?.content || ''}
                            </div>
                          </div>
                        )}
                      </>
                    )}
                    
                    {instructionType === 'file' && (
                      <>
                        <Form.Item 
                          label="Instruction File"
                          required
                          validateStatus={!instructionFilePath && 'error'}
                          help={!instructionFilePath && 'Please select or enter a file path'}
                        >
                          <Space.Compact style={{ width: '100%' }}>
                            <Input
                              value={instructionFilePath}
                              onChange={(e) => setInstructionFilePath(e.target.value)}
                              placeholder="./prompts/my_instruction.md"
                              style={{ flex: 1 }}
                            />
                            <Button 
                              icon={<FolderOpenOutlined />} 
                              onClick={handleSelectInstructionFile}
                            >
                              Browse
                            </Button>
                          </Space.Compact>
                        </Form.Item>
                        <Text type="secondary" style={{ fontSize: 12 }}>
                          Select a markdown (.md) or text (.txt) file containing the instruction
                        </Text>
                      </>
                    )}
                    
                    {instructionType === 'inline' && (
                      <>
                        <Form.Item 
                          label="Instruction Content"
                          required
                          validateStatus={!instructionInlineContent?.trim() && 'error'}
                          help={!instructionInlineContent?.trim() && 'Please enter instruction content'}
                        >
                          <TextArea
                            rows={12}
                            value={instructionInlineContent}
                            onChange={(e) => setInstructionInlineContent(e.target.value)}
                            placeholder="Enter your custom instruction here..."
                            style={{ fontFamily: 'monospace' }}
                          />
                        </Form.Item>
                        <Space>
                          <Text type="secondary" style={{ fontSize: 12 }}>
                            Tip: Select a built-in instruction first, then switch to Inline mode to customize it.
                          </Text>
                        </Space>
                      </>
                    )}
                  </div>
                ),
              },
              {
                key: 'tools',
                label: 'Tools',
                children: (
                  <>
                    <Form.Item name="tools" label="Available Tools">
                      <Select mode="multiple" placeholder="Select tools">
                        {availableTools.map(tool => (
                          <Select.Option key={tool} value={tool}>{tool}</Select.Option>
                        ))}
                      </Select>
                    </Form.Item>
                    <Form.Item name="skills" label="Skills">
                      <Select mode="multiple" placeholder="Select skills">
                        {availableSkills.map(skill => (
                          <Select.Option key={skill} value={skill}>{skill}</Select.Option>
                        ))}
                      </Select>
                    </Form.Item>
                  </>
                ),
              },
              {
                key: 'advanced',
                label: 'Advanced',
                children: (
                  <>
                    <Form.Item name="include_contents" label="Include Contents Mode">
                      <Select>
                        <Select.Option value="none">None</Select.Option>
                        <Select.Option value="all">All</Select.Option>
                        <Select.Option value="selected">Selected</Select.Option>
                      </Select>
                    </Form.Item>
                    <Form.Item name="tags" label="Tags">
                      <Select mode="tags" placeholder="Add tags" />
                    </Form.Item>
                  </>
                ),
              },
            ]}
          />
        </Form>
      </Modal>

      {/* Detail Drawer */}
      <Drawer
        title={selectedAgentData?.name || 'Agent Details'}
        placement="right"
        width={500}
        onClose={() => setDetailDrawerVisible(false)}
        open={detailDrawerVisible}
      >
        {selectedAgentData && (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="ID">{selectedAgentData.id}</Descriptions.Item>
              <Descriptions.Item label="Version">{selectedAgentData.version || '-'}</Descriptions.Item>
              <Descriptions.Item label="Type">
                {getAgentTypeTag(selectedAgentData.agent_type)}
              </Descriptions.Item>
              <Descriptions.Item label="Description">
                {selectedAgentData.description || '-'}
              </Descriptions.Item>
            </Descriptions>

            <Title level={5}>Model Configuration</Title>
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="Model ID">
                {selectedAgentData.model.model_id || 'Default'}
              </Descriptions.Item>
              <Descriptions.Item label="Temperature">
                {selectedAgentData.model.temperature ?? 0.7}
              </Descriptions.Item>
              <Descriptions.Item label="Max Tokens">
                {selectedAgentData.model.max_tokens || 'Default'}
              </Descriptions.Item>
            </Descriptions>

            <Title level={5}>Tools ({selectedAgentData.tools?.length || 0})</Title>
            <Space wrap>
              {selectedAgentData.tools?.map((tool, i) => (
                <Tag key={i} color="blue">{tool.tool_id}</Tag>
              ))}
            </Space>

            {(selectedAgentData.skills?.length || 0) > 0 && (
              <>
                <Title level={5}>Skills ({selectedAgentData.skills?.length || 0})</Title>
                <Space wrap>
                  {selectedAgentData.skills?.map((skill, i) => (
                    <Tag key={i} color="purple">{skill}</Tag>
                  ))}
                </Space>
              </>
            )}

            <Title level={5}>Instruction</Title>
            <Paragraph
              ellipsis={{ rows: 5, expandable: true, symbol: 'more' }}
              style={{ background: '#f5f5f5', padding: 8, borderRadius: 4 }}
            >
              {selectedAgentData.instruction}
            </Paragraph>
          </Space>
        )}
      </Drawer>

      {/* Import Modal */}
      <Modal
        title="Import Agent"
        open={importModalVisible}
        onCancel={() => setImportModalVisible(false)}
        onOk={handleImport}
        okText="Import"
      >
        <TextArea
          rows={10}
          placeholder="Paste agent JSON here..."
          value={importJson}
          onChange={(e) => setImportJson(e.target.value)}
        />
      </Modal>
    </div>
  );
};

export default AgentConfigForm;
