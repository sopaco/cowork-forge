import React, { useState } from 'react';
import {
  List,
  Button,
  Space,
  Typography,
  Tag,
  Modal,
  Input,
  message,
  Popconfirm,
  Empty,
  Drawer,
  Descriptions,
  Divider,
  Card,
  Badge,
  Tooltip,
  Alert,
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  ThunderboltOutlined,
  FolderOpenOutlined,
  ToolOutlined,
  FileTextOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons';
import { useConfigStore } from '../../stores/configStore';
import type { SkillManifest, SkillCategory } from '../../types/config';

const { Title, Text, Paragraph } = Typography;

const SkillManager: React.FC = () => {
  const {
    skills,
    selectedSkill,
    selectSkill,
    installSkill,
    uninstallSkill,
  } = useConfigStore();

  const [installModalVisible, setInstallModalVisible] = useState(false);
  const [skillPath, setSkillPath] = useState('');
  const [detailDrawerVisible, setDetailDrawerVisible] = useState(false);
  const [installing, setInstalling] = useState(false);

  const handleInstall = async () => {
    if (!skillPath.trim()) {
      message.error('Please enter a skill path');
      return;
    }
    
    setInstalling(true);
    try {
      await installSkill(skillPath.trim());
      message.success('Skill installed successfully');
      setInstallModalVisible(false);
      setSkillPath('');
    } catch (error) {
      message.error('Failed to install skill');
    } finally {
      setInstalling(false);
    }
  };

  const handleView = (skill: SkillManifest) => {
    selectSkill(skill.id);
    setDetailDrawerVisible(true);
  };

  const handleUninstall = async (id: string) => {
    try {
      await uninstallSkill(id);
      message.success('Skill uninstalled successfully');
    } catch (error) {
      message.error('Failed to uninstall skill');
    }
  };

  const selectedSkillData = selectedSkill ? skills[selectedSkill] : null;

  const getCategoryColor = (category: SkillCategory): string => {
    const colors: Record<string, string> = {
      domain: 'blue',
      tool: 'green',
      integration: 'purple',
      template: 'orange',
    };
    return colors[category] || 'default';
  };

  const getCategoryLabel = (category: SkillCategory): string => {
    const labels: Record<string, string> = {
      domain: 'Domain Skill',
      tool: 'Tool Extension',
      integration: 'Integration',
      template: 'Template',
    };
    return labels[category] || category;
  };

  const groupedSkills = Object.values(skills).reduce((acc, skill) => {
    const cat = skill.category;
    if (!acc[cat]) acc[cat] = [];
    acc[cat].push(skill);
    return acc;
  }, {} as Record<string, SkillManifest[]>);

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={5} style={{ margin: 0 }}>
          Skill Manager
          <Badge count={Object.keys(skills).length} style={{ marginLeft: 8 }} />
        </Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={() => setInstallModalVisible(true)}>
          Install Skill
        </Button>
      </div>

      {Object.keys(skills).length === 0 ? (
        <Empty 
          description={
            <Space direction="vertical" size="small">
              <Text>No skills installed</Text>
              <Text type="secondary">Install skills to extend agent capabilities</Text>
            </Space>
          } 
          style={{ marginTop: '40px' }} 
        />
      ) : (
        <div style={{ flex: 1, overflow: 'auto', padding: '0 16px' }}>
          {Object.entries(groupedSkills).map(([category, categorySkills]) => (
            <Card
              key={category}
              title={
                <Space>
                  <Tag color={getCategoryColor(category as SkillCategory)}>
                    {getCategoryLabel(category as SkillCategory)}
                  </Tag>
                  <Text type="secondary">({categorySkills.length})</Text>
                </Space>
              }
              size="small"
              style={{ marginBottom: 16 }}
            >
              <List
                dataSource={categorySkills.sort((a, b) => a.name.localeCompare(b.name))}
                renderItem={(skill) => (
                  <List.Item
                    actions={[
                      <Button
                        key="view"
                        type="link"
                        size="small"
                        icon={<InfoCircleOutlined />}
                        onClick={() => handleView(skill)}
                      >
                        Details
                      </Button>,
                      <Popconfirm
                        key="uninstall"
                        title="Uninstall this skill?"
                        onConfirm={() => handleUninstall(skill.id)}
                      >
                        <Button type="link" size="small" danger icon={<DeleteOutlined />}>
                          Uninstall
                        </Button>
                      </Popconfirm>,
                    ]}
                  >
                    <List.Item.Meta
                      avatar={<ThunderboltOutlined style={{ fontSize: 24, color: '#1890ff' }} />}
                      title={
                        <Space>
                          <Text strong>{skill.name}</Text>
                          <Tag>v{skill.version}</Tag>
                          {skill.author && <Text type="secondary">by {skill.author}</Text>}
                        </Space>
                      }
                      description={
                        <Space direction="vertical" size="small">
                          <Text type="secondary">{skill.description}</Text>
                          <Space size={4}>
                            <Tooltip title="Tools provided">
                              <Tag icon={<ToolOutlined />} color="green">
                                {skill.tools.length} tools
                              </Tag>
                            </Tooltip>
                            <Tooltip title="Prompts included">
                              <Tag icon={<FileTextOutlined />} color="blue">
                                {skill.prompts.length} prompts
                              </Tag>
                            </Tooltip>
                          </Space>
                        </Space>
                      }
                    />
                  </List.Item>
                )}
              />
            </Card>
          ))}
        </div>
      )}

      {/* Install Modal */}
      <Modal
        title="Install Skill"
        open={installModalVisible}
        onCancel={() => setInstallModalVisible(false)}
        onOk={handleInstall}
        okText="Install"
        confirmLoading={installing}
      >
        <Space direction="vertical" style={{ width: '100%' }}>
          <Alert
            message="Enter the path to a skill directory containing a manifest.json file"
            type="info"
            showIcon
          />
          <Input
            placeholder="/path/to/skill-directory"
            value={skillPath}
            onChange={(e) => setSkillPath(e.target.value)}
            prefix={<FolderOpenOutlined />}
          />
          <Text type="secondary">
            Skills can be installed from local directories or Git repositories
          </Text>
        </Space>
      </Modal>

      {/* Detail Drawer */}
      <Drawer
        title={selectedSkillData?.name || 'Skill Details'}
        placement="right"
        width={600}
        onClose={() => setDetailDrawerVisible(false)}
        open={detailDrawerVisible}
      >
        {selectedSkillData && (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <Descriptions column={2} bordered size="small">
              <Descriptions.Item label="ID">{selectedSkillData.id}</Descriptions.Item>
              <Descriptions.Item label="Version">{selectedSkillData.version}</Descriptions.Item>
              <Descriptions.Item label="Category">
                <Tag color={getCategoryColor(selectedSkillData.category)}>
                  {getCategoryLabel(selectedSkillData.category)}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="Author">{selectedSkillData.author || '-'}</Descriptions.Item>
              <Descriptions.Item label="Description" span={2}>
                {selectedSkillData.description}
              </Descriptions.Item>
              {selectedSkillData.homepage && (
                <Descriptions.Item label="Homepage" span={2}>
                  <a href={selectedSkillData.homepage} target="_blank" rel="noopener noreferrer">
                    {selectedSkillData.homepage}
                  </a>
                </Descriptions.Item>
              )}
              {selectedSkillData.repository && (
                <Descriptions.Item label="Repository" span={2}>
                  <a href={selectedSkillData.repository} target="_blank" rel="noopener noreferrer">
                    {selectedSkillData.repository}
                  </a>
                </Descriptions.Item>
              )}
            </Descriptions>

            <Title level={5}>Keywords</Title>
            <Space wrap>
              {selectedSkillData.keywords.map((keyword, i) => (
                <Tag key={i}>{keyword}</Tag>
              ))}
            </Space>

            {selectedSkillData.dependencies.length > 0 && (
              <>
                <Title level={5}>Dependencies</Title>
                <Space wrap>
                  {selectedSkillData.dependencies.map((dep, i) => (
                    <Tag key={i} color="orange">{dep}</Tag>
                  ))}
                </Space>
              </>
            )}

            <Title level={5}>Tools Provided ({selectedSkillData.tools.length})</Title>
            {selectedSkillData.tools.length > 0 ? (
              <List
                size="small"
                bordered
                dataSource={selectedSkillData.tools}
                renderItem={(tool) => (
                  <List.Item>
                    <List.Item.Meta
                      title={tool.name}
                      description={tool.description}
                    />
                  </List.Item>
                )}
              />
            ) : (
              <Text type="secondary">No tools provided</Text>
            )}

            <Title level={5}>Prompts ({selectedSkillData.prompts.length})</Title>
            {selectedSkillData.prompts.length > 0 ? (
              <List
                size="small"
                bordered
                dataSource={selectedSkillData.prompts}
                renderItem={(prompt) => (
                  <List.Item>
                    <List.Item.Meta
                      title={
                        <Space>
                          {prompt.name}
                          <Tag>{prompt.prompt_type}</Tag>
                          {prompt.target_agent && <Tag color="blue">{prompt.target_agent}</Tag>}
                        </Space>
                      }
                      description={
                        <Paragraph ellipsis={{ rows: 2 }} style={{ marginBottom: 0 }}>
                          {prompt.content}
                        </Paragraph>
                      }
                    />
                  </List.Item>
                )}
              />
            ) : (
              <Text type="secondary">No prompts included</Text>
            )}

            <Title level={5}>Compatible Agents</Title>
            <Space wrap>
              {selectedSkillData.compatible_agents.length > 0 ? (
                selectedSkillData.compatible_agents.map((agent, i) => (
                  <Tag key={i} color="purple">{agent}</Tag>
                ))
              ) : (
                <Text type="secondary">All agents</Text>
              )}
            </Space>
          </Space>
        )}
      </Drawer>
    </div>
  );
};

export default SkillManager;
