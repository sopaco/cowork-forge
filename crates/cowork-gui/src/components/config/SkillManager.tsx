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
  InfoCircleOutlined,
  TagOutlined,
} from '@ant-design/icons';
import { useConfigStore } from '../../stores/configStore';
import type { SkillInfo } from '../../types/config';

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

  const handleView = (skill: SkillInfo) => {
    selectSkill(skill.name);
    setDetailDrawerVisible(true);
  };

  const handleUninstall = async (name: string) => {
    try {
      await uninstallSkill(name);
      message.success('Skill uninstalled successfully');
    } catch (error) {
      message.error('Failed to uninstall skill');
    }
  };

  const selectedSkillData = selectedSkill 
    ? skills.find(s => s.name === selectedSkill) 
    : null;

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={5} style={{ margin: 0 }}>
          Skill Manager
          <Badge count={skills.length} style={{ marginLeft: 8 }} />
        </Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={() => setInstallModalVisible(true)}>
          Install Skill
        </Button>
      </div>

      {skills.length === 0 ? (
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
          <Card size="small" style={{ marginBottom: 16 }}>
            <List
              dataSource={skills.sort((a, b) => a.name.localeCompare(b.name))}
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
                      onConfirm={() => handleUninstall(skill.name)}
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
                      </Space>
                    }
                    description={
                      <Space direction="vertical" size="small">
                        <Text type="secondary">{skill.description}</Text>
                        <Space size={4}>
                          {skill.tags.slice(0, 3).map((tag, i) => (
                            <Tag key={i} color="blue">{tag}</Tag>
                          ))}
                          {skill.tags.length > 3 && (
                            <Tag>+{skill.tags.length - 3}</Tag>
                          )}
                        </Space>
                      </Space>
                    }
                  />
                </List.Item>
              )}
            />
          </Card>
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
        <Space direction="vertical" style={{ width: '100%' }} size="middle">
          <Alert
            message="Enter the path to a skill directory containing a SKILL.md file"
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
            Skills follow the agentskills.io standard. Each skill directory should contain a SKILL.md file with YAML frontmatter.
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
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="ID">{selectedSkillData.id}</Descriptions.Item>
              <Descriptions.Item label="Name">{selectedSkillData.name}</Descriptions.Item>
              <Descriptions.Item label="Description">
                {selectedSkillData.description}
              </Descriptions.Item>
            </Descriptions>

            <Title level={5}>
              <TagOutlined style={{ marginRight: 8 }} />
              Tags
            </Title>
            <Space wrap>
              {selectedSkillData.tags.length > 0 ? (
                selectedSkillData.tags.map((tag, i) => (
                  <Tag key={i} color="blue">{tag}</Tag>
                ))
              ) : (
                <Text type="secondary">No tags</Text>
              )}
            </Space>

            <Title level={5}>Skill Instructions</Title>
            {selectedSkillData.body ? (
              <Card size="small" style={{ maxHeight: 400, overflow: 'auto' }}>
                <Paragraph style={{ whiteSpace: 'pre-wrap', marginBottom: 0 }}>
                  {selectedSkillData.body}
                </Paragraph>
              </Card>
            ) : (
              <Text type="secondary">No instructions defined</Text>
            )}
          </Space>
        )}
      </Drawer>
    </div>
  );
};

export default SkillManager;