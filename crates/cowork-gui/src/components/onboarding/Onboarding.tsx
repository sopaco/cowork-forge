import React, { useState, useEffect } from 'react';
import { Modal, Steps, Button, Input, Form, Alert, Space, Typography, Card, Result } from 'antd';
import {
  RocketOutlined,
  ApiOutlined,
  FolderOutlined,
  CheckCircleOutlined,
  LoadingOutlined,
} from '@ant-design/icons';
import API from '../../api';
import { showError, showSuccess } from '../../utils/errorHandler';

const { Text, Paragraph, Title } = Typography;

interface OnboardingProps {
  visible: boolean;
  onComplete: () => void;
}

type StepStatus = 'wait' | 'process' | 'finish' | 'error';

export const Onboarding: React.FC<OnboardingProps> = ({ visible, onComplete }) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [config, setConfig] = useState({
    api_base_url: 'https://api.openai.com/v1',
    api_key: '',
    model_name: 'gpt-4',
  });
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null);
  const [workspacePath, setWorkspacePath] = useState('');

  useEffect(() => {
    if (visible) {
      API.config.hasValid().then((hasConfig) => {
        if (hasConfig) {
          API.config.get().then((cfg) => {
            setConfig({
              api_base_url: cfg.llm.api_base_url,
              api_key: cfg.llm.api_key,
              model_name: cfg.llm.model_name,
            });
          });
        }
      });
    }
  }, [visible]);

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    
    try {
      await API.config.save({
        llm: {
          api_base_url: config.api_base_url,
          api_key: config.api_key,
          model_name: config.model_name,
        },
      });
      
      const result = await API.config.testConnection();
      setTestResult(result.success ? 'success' : 'error');
      
      if (result.success) {
        showSuccess('连接成功！');
      } else {
        showError(result.message);
      }
    } catch (error) {
      setTestResult('error');
      showError(error);
    } finally {
      setTesting(false);
    }
  };

  const handleSelectFolder = async () => {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      setWorkspacePath(selected as string);
    }
  };

  const handleComplete = async () => {
    if (workspacePath) {
      try {
        await API.workspace.set(workspacePath);
      } catch (error) {
        showError(error);
        return;
      }
    }
    showSuccess('设置完成！');
    onComplete();
  };

  const steps = [
    {
      title: '欢迎',
      icon: <RocketOutlined />,
      content: (
        <div style={{ textAlign: 'center', padding: '24px 0' }}>
          <RocketOutlined style={{ fontSize: 64, color: '#1890ff', marginBottom: 24 }} />
          <Title level={3}>欢迎使用 Cowork Forge</Title>
          <Paragraph type="secondary" style={{ fontSize: 16 }}>
            AI 驱动的端到端开发工具
          </Paragraph>
          <Paragraph>
            接下来将引导您完成基本配置，让您快速开始创建项目。
          </Paragraph>
          <Space direction="vertical" size="small" style={{ marginTop: 24 }}>
            <Text type="secondary">• 配置 AI 模型 API</Text>
            <Text type="secondary">• 选择工作目录</Text>
            <Text type="secondary">• 开始创建您的第一个项目</Text>
          </Space>
        </div>
      ),
    },
    {
      title: '配置 AI',
      icon: <ApiOutlined />,
      content: (
        <div style={{ padding: '24px 0' }}>
          <Alert
            message="需要配置 AI 模型"
            description="Cowork Forge 需要连接 OpenAI 兼容的 AI 模型来生成代码和文档。"
            type="info"
            showIcon
            style={{ marginBottom: 24 }}
          />
          
          <Form layout="vertical">
            <Form.Item label="API Base URL" required>
              <Input
                value={config.api_base_url}
                onChange={(e) => setConfig({ ...config, api_base_url: e.target.value })}
                placeholder="https://api.openai.com/v1"
              />
            </Form.Item>
            
            <Form.Item label="API Key" required>
              <Input.Password
                value={config.api_key}
                onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
                placeholder="sk-..."
              />
            </Form.Item>
            
            <Form.Item label="模型名称">
              <Input
                value={config.model_name}
                onChange={(e) => setConfig({ ...config, model_name: e.target.value })}
                placeholder="gpt-4"
              />
            </Form.Item>
            
            <Form.Item>
              <Button
                type="primary"
                onClick={handleTestConnection}
                loading={testing}
                icon={testResult === 'success' ? <CheckCircleOutlined /> : undefined}
                disabled={!config.api_key || !config.api_base_url}
              >
                {testing ? '测试连接中...' : testResult === 'success' ? '连接成功' : '测试连接'}
              </Button>
              {testResult === 'error' && (
                <Text type="danger" style={{ marginLeft: 12 }}>
                  连接失败，请检查配置
                </Text>
              )}
            </Form.Item>
          </Form>
        </div>
      ),
    },
    {
      title: '工作目录',
      icon: <FolderOutlined />,
      content: (
        <div style={{ padding: '24px 0' }}>
          <Paragraph>
            选择一个目录作为您的工作空间，项目将在此创建。
          </Paragraph>
          
          <Card style={{ marginBottom: 16 }}>
            <Space direction="vertical" style={{ width: '100%' }}>
              <Text strong>当前选择：</Text>
              <Text code style={{ wordBreak: 'break-all' }}>
                {workspacePath || '（未选择）'}
              </Text>
              <Button icon={<FolderOutlined />} onClick={handleSelectFolder}>
                选择目录
              </Button>
            </Space>
          </Card>
          
          <Alert
            message="可以稍后设置"
            description="您也可以跳过此步骤，在项目面板中选择或创建目录。"
            type="info"
            showIcon
          />
        </div>
      ),
    },
    {
      title: '完成',
      icon: <CheckCircleOutlined />,
      content: (
        <Result
          status="success"
          title="设置完成！"
          subTitle="您已准备好开始使用 Cowork Forge 创建项目。"
          extra={[
            <Paragraph key="tips" type="secondary">
              提示：您可以随时在设置页面修改配置。
            </Paragraph>,
          ]}
        />
      ),
    },
  ];

  const canProceed = () => {
    switch (currentStep) {
      case 1:
        return testResult === 'success';
      case 2:
        return true;
      default:
        return true;
    }
  };

  return (
    <Modal
      open={visible}
      closable={false}
      footer={null}
      width={600}
      centered
      maskClosable={false}
    >
      <Steps
        current={currentStep}
        items={steps.map((s) => ({ title: s.title, icon: s.icon }))}
        style={{ marginBottom: 24 }}
      />
      
      <div style={{ minHeight: 300 }}>
        {steps[currentStep].content}
      </div>
      
      <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 24 }}>
        <Button
          onClick={() => setCurrentStep(currentStep - 1)}
          disabled={currentStep === 0}
        >
          上一步
        </Button>
        
        {currentStep < steps.length - 1 ? (
          <Button
            type="primary"
            onClick={() => setCurrentStep(currentStep + 1)}
            disabled={!canProceed()}
          >
            下一步
          </Button>
        ) : (
          <Button type="primary" onClick={handleComplete}>
            开始使用
          </Button>
        )}
      </div>
    </Modal>
  );
};
