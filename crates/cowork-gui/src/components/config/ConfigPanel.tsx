import React, { useEffect, useState } from 'react';
import {
  Tabs,
  Card,
  Button,
  Space,
  Spin,
  Empty,
  message,
  Modal,
  Typography,
  Tag,
  Popconfirm,
} from 'antd';
import {
  AppstoreOutlined,
  RobotOutlined,
  ApiOutlined,
  ThunderboltOutlined,
  PlusOutlined,
  ReloadOutlined,
  UndoOutlined,
} from '@ant-design/icons';
import { useConfigStore } from '../../stores/configStore';
import FlowConfigPanel from './FlowConfigPanel';
import AgentConfigForm from './AgentConfigForm';
import SkillManager from './SkillManager';
import IntegrationConfig from './IntegrationConfig';

const { Title, Text } = Typography;

type ConfigTab = 'flows' | 'agents' | 'skills' | 'integrations';

const ConfigPanel: React.FC = () => {
  const {
    loading,
    error,
    loadConfigs,
    resetConfigs,
    agents,
    flows,
    skills,
    integrations,
  } = useConfigStore();

  const [activeTab, setActiveTab] = useState<ConfigTab>('flows');
  const [resetting, setResetting] = useState(false);

  useEffect(() => {
    loadConfigs();
  }, [loadConfigs]);

  useEffect(() => {
    if (error) {
      message.error(error);
    }
  }, [error]);

  const handleRefresh = () => {
    loadConfigs();
  };

  const handleReset = async () => {
    setResetting(true);
    try {
      await resetConfigs();
      message.success('Configurations reset to defaults');
    } catch (err) {
      message.error('Failed to reset configurations');
    } finally {
      setResetting(false);
    }
  };

  const tabItems = [
    {
      key: 'flows',
      label: (
        <span>
          <AppstoreOutlined />
          Flows ({Object.keys(flows).length})
        </span>
      ),
      children: <FlowConfigPanel />,
    },
    {
      key: 'agents',
      label: (
        <span>
          <RobotOutlined />
          Agents ({Object.keys(agents).length})
        </span>
      ),
      children: <AgentConfigForm />,
    },
    {
      key: 'skills',
      label: (
        <span>
          <ThunderboltOutlined />
          Skills ({Object.keys(skills).length})
        </span>
      ),
      children: <SkillManager />,
    },
    {
      key: 'integrations',
      label: (
        <span>
          <ApiOutlined />
          Integrations ({Object.keys(integrations).length})
        </span>
      ),
      children: <IntegrationConfig />,
    },
  ];

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
        <Spin size="large" />
      </div>
    );
  }

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '16px', borderBottom: '1px solid #f0f0f0', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <Title level={4} style={{ margin: 0 }}>Configuration Management</Title>
          <Text type="secondary">Manage flows, agents, skills, and integrations</Text>
        </div>
        <Space>
          <Button icon={<ReloadOutlined />} onClick={handleRefresh}>
            Refresh
          </Button>
          <Popconfirm
            title="Reset to defaults?"
            description="This will restore all configurations to built-in defaults. Your custom configurations will be removed."
            onConfirm={handleReset}
            okText="Reset"
            cancelText="Cancel"
            okButtonProps={{ danger: true }}
          >
            <Button 
              icon={<UndoOutlined />} 
              danger
              loading={resetting}
            >
              Reset to Defaults
            </Button>
          </Popconfirm>
        </Space>
      </div>
      <div style={{ flex: 1, overflow: 'auto' }}>
        <Tabs
          activeKey={activeTab}
          onChange={(key) => setActiveTab(key as ConfigTab)}
          items={tabItems}
          style={{ padding: '0 16px' }}
          size="large"
        />
      </div>
    </div>
  );
};

export default ConfigPanel;
