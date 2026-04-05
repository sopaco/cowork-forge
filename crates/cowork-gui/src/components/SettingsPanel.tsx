import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Form, Input, Button, Switch, Select, Card, App, Spin, Typography, Space, Tag, Divider } from "antd";
import { SaveOutlined, FolderOpenOutlined, ApiOutlined, RobotOutlined, CloudOutlined, CheckCircleOutlined, ExclamationCircleOutlined, SearchOutlined, BookOutlined } from "@ant-design/icons";

const { Title, Text, Paragraph } = Typography;
const { Password } = Input;

const AGENT_TYPES = [
  { value: "opencode", label: "OpenCode" },
  { value: "iflow", label: "iFlow" },
  { value: "codex", label: "Codex" },
  { value: "gemini", label: "Gemini CLI" },
  { value: "claude", label: "Claude CLI" },
];

const TRANSPORT_TYPES = [
  { value: "stdio", label: "Standard I/O" },
  { value: "websocket", label: "WebSocket" },
];

interface LLMConfig {
  api_base_url: string;
  api_key: string;
  model_name: string;
  temperature?: number;
  max_tokens?: number;
}

interface EmbeddingConfig {
  api_base_url?: string;
  api_key?: string;
  model_name?: string;
}

interface CodingAgentConfig {
  enabled: boolean;
  agent_type?: string;
  command?: string;
  args?: string[];
  transport?: string;
  workspace_path?: string;
}

interface AppConfig {
  llm: LLMConfig;
  embedding?: EmbeddingConfig;
  coding_agent?: CodingAgentConfig;
  mcp?: McpConfig;
}

interface McpConfig {
  tavily_api_key: string;
  deepwiki_enabled: boolean;
}

const SettingsPanel: React.FC = () => {
  const { message } = App.useApp();
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [configPath, setConfigPath] = useState("");
  const [hasConfig, setHasConfig] = useState(false);
  const [testingTavily, setTestingTavily] = useState(false);
  const [testingDeepWiki, setTestingDeepWiki] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const path = await invoke<string>("get_config_path");
      setConfigPath(path);

      const valid = await invoke<boolean>("has_valid_config");
      setHasConfig(valid);

      if (valid) {
        const config = await invoke<AppConfig>("get_app_config");
        form.setFieldsValue(config);
      } else {
        const defaultConfig = await invoke<AppConfig>("get_default_config");
        form.setFieldsValue(defaultConfig);
      }
    } catch (error) {
      message.error("Failed to load config: " + error);
      const defaultConfig = await invoke<AppConfig>("get_default_config");
      form.setFieldsValue(defaultConfig);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      setSaving(true);
      await invoke("save_app_config", { config: values });
      message.success("Configuration saved successfully");
      setHasConfig(true);
    } catch (error: unknown) {
      if ((error as { errorFields?: unknown }).errorFields) {
        message.error("Please fill in all required fields");
      } else {
        message.error("Failed to save config: " + error);
      }
    } finally {
      setSaving(false);
    }
  };

  const handleOpenFolder = async () => {
    try {
      await invoke("open_config_folder");
    } catch (error) {
      message.error("Failed to open folder: " + error);
    }
  };

  const handleTestTavily = async () => {
    try {
      const mcpConfig = form.getFieldValue("mcp");
      if (!mcpConfig?.tavily_api_key) {
        message.warning("Please input Tavily API key first");
        return;
      }

      setTestingTavily(true);
      await invoke("gui_test_tavily_connection", { apiKey: mcpConfig.tavily_api_key });
      message.success("Tavily MCP connection test successful!");
    } catch (error) {
      message.error("Tavily connection test failed: " + error);
    } finally {
      setTestingTavily(false);
    }
  };

  const handleTestDeepWiki = async () => {
    try {
      const mcpConfig = form.getFieldValue("mcp");
      if (!mcpConfig?.deepwiki_enabled) {
        message.warning("Please enable DeepWiki first");
        return;
      }

      setTestingDeepWiki(true);
      await invoke("gui_test_deepwiki_connection");
      message.success("DeepWiki MCP connection test successful!");
    } catch (error) {
      message.error("DeepWiki connection test failed: " + error);
    } finally {
      setTestingDeepWiki(false);
    }
  };

  const handleTestConnection = async () => {
    try {
      const llmConfig = form.getFieldValue("llm");
      if (!llmConfig?.api_base_url || !llmConfig?.api_key || !llmConfig?.model_name) {
        message.warning("Please fill in all LLM settings first");
        return;
      }

      setSaving(true);
      await invoke("test_llm_connection", { llmConfig });
      message.success("Connection test successful!");
    } catch (error) {
      message.error("Connection test failed: " + error);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "100%" }}>
        <Spin size="large" tip="Loading configuration...">
          <div style={{ padding: 50 }} />
        </Spin>
      </div>
    );
  }

  return (
    <div style={{ padding: "24px", maxWidth: "800px", margin: "0 auto" }}>
      <div style={{ marginBottom: "24px" }}>
        <Title level={3} style={{ marginBottom: "8px" }}>
          <ApiOutlined style={{ marginRight: "8px" }} />
          Settings
        </Title>
        <Text type="secondary">Configure your LLM and coding agent settings</Text>
        <Paragraph type="secondary" style={{ marginTop: "8px", marginBottom: 0 }}>
          Config file: <Text code>{configPath}</Text>
        </Paragraph>
      </div>

      {!hasConfig && (
        <Card style={{ marginBottom: "24px", borderColor: "#faad14" }}>
          <Space>
            <ExclamationCircleOutlined style={{ color: "#faad14" }} />
            <Text>No configuration found. Please configure your LLM settings below.</Text>
          </Space>
        </Card>
      )}

      <Form
        form={form}
        layout="vertical"
        initialValues={{
          coding_agent: { enabled: false, agent_type: "opencode", command: "bun", args: ["x", "opencode-ai", "acp"], transport: "stdio" },
          mcp: { tavily_api_key: "", deepwiki_enabled: false },
        }}
      >
        <Card title={<Space><CloudOutlined /><span>LLM Configuration</span><Tag color="red">Required</Tag></Space>} style={{ marginBottom: "24px" }}>
          <Form.Item name={["llm", "api_base_url"]} label="API Base URL" rules={[{ required: true, message: "Please input API base URL" }]}>
            <Input placeholder="https://api.openai.com/v1" />
          </Form.Item>
          <Form.Item name={["llm", "api_key"]} label="API Key" rules={[{ required: true, message: "Please input API key" }]}>
            <Password placeholder="sk-..." />
          </Form.Item>
          <Form.Item name={["llm", "model_name"]} label="Model Name" rules={[{ required: true, message: "Please input model name" }]}>
            <Input placeholder="gpt-4o" />
          </Form.Item>
          <Button icon={<CheckCircleOutlined />} onClick={handleTestConnection} loading={saving}>Test Connection</Button>
        </Card>

        <Card title={<Space><CloudOutlined /><span>Embedding Configuration</span><Tag>Optional</Tag></Space>} style={{ marginBottom: "24px" }}>
          <Form.Item name={["embedding", "api_base_url"]} label="API Base URL">
            <Input placeholder="https://api.openai.com/v1" />
          </Form.Item>
          <Form.Item name={["embedding", "api_key"]} label="API Key">
            <Password placeholder="sk-..." />
          </Form.Item>
          <Form.Item name={["embedding", "model_name"]} label="Model Name">
            <Input placeholder="text-embedding-3-small" />
          </Form.Item>
        </Card>

        <Card title={<Space><SearchOutlined /><span>MCP Services</span><Tag>Optional</Tag></Space>} style={{ marginBottom: "24px" }}>
          <Paragraph type="secondary" style={{ marginBottom: "16px" }}>
            Configure remote Model Context Protocol (MCP) services to extend agent capabilities.
          </Paragraph>
          
          <Form.Item name={["mcp", "tavily_api_key"]} label={
            <Space>
              <SearchOutlined />
              <span>Tavily API Key</span>
              <Tag color="blue">Web Search</Tag>
            </Space>
          }>
            <Password placeholder="tvly-dev-your-api-key" />
          </Form.Item>
          <Form.Item>
            <Button 
              icon={<CheckCircleOutlined />} 
              onClick={handleTestTavily} 
              loading={testingTavily}
              size="small"
            >
              Test Tavily Connection
            </Button>
          </Form.Item>

          <Divider style={{ margin: "16px 0" }} />

          <Form.Item name={["mcp", "deepwiki_enabled"]} label={
            <Space>
              <BookOutlined />
              <span>DeepWiki</span>
              <Tag color="purple">Code Docs</Tag>
            </Space>
          } valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item noStyle shouldUpdate={(prev, cur) => prev.mcp?.deepwiki_enabled !== cur.mcp?.deepwiki_enabled}>
            {({ getFieldValue }) =>
              getFieldValue(["mcp", "deepwiki_enabled"]) ? (
                <Form.Item>
                  <Button 
                    icon={<CheckCircleOutlined />} 
                    onClick={handleTestDeepWiki} 
                    loading={testingDeepWiki}
                    size="small"
                  >
                    Test DeepWiki Connection
                  </Button>
                </Form.Item>
              ) : null
            }
          </Form.Item>
        </Card>

        <Card title={<Space><RobotOutlined /><span>External Coding Agent</span><Tag>Optional</Tag></Space>} style={{ marginBottom: "24px" }}>
          <Form.Item name={["coding_agent", "enabled"]} label="Enable External Agent" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item noStyle shouldUpdate={(prev, cur) => prev.coding_agent?.enabled !== cur.coding_agent?.enabled}>
            {({ getFieldValue }) =>
              getFieldValue(["coding_agent", "enabled"]) ? (
                <>
                  <Form.Item name={["coding_agent", "agent_type"]} label="Agent Type">
                    <Select options={AGENT_TYPES} />
                  </Form.Item>
                  <Form.Item name={["coding_agent", "command"]} label="Command">
                    <Input placeholder="bun" />
                  </Form.Item>
                  <Form.Item name={["coding_agent", "args"]} label="Arguments">
                    <Select mode="tags" placeholder='["x", "opencode-ai", "acp"]' style={{ width: "100%" }} />
                  </Form.Item>
                  <Form.Item name={["coding_agent", "transport"]} label="Transport">
                    <Select options={TRANSPORT_TYPES} />
                  </Form.Item>
                  <Form.Item name={["coding_agent", "workspace_path"]} label="Workspace Path (optional)">
                    <Input placeholder="Leave empty to use current project" />
                  </Form.Item>
                </>
              ) : null
            }
          </Form.Item>
        </Card>

        <div style={{ display: "flex", gap: "12px" }}>
          <Button type="primary" icon={<SaveOutlined />} onClick={handleSave} loading={saving}>Save Configuration</Button>
          <Button icon={<FolderOpenOutlined />} onClick={handleOpenFolder}>Open Config Folder</Button>
        </div>
      </Form>
    </div>
  );
};

export default SettingsPanel;
