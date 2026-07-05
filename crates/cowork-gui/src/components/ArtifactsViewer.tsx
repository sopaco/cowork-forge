import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';
import { remarkPlugins, fullRehypePlugins } from '@/utils/markdown';
import { App, Tabs, Empty, Button, Space, Tooltip } from 'antd';
import { FileTextOutlined, ProjectOutlined, BuildOutlined, CheckCircleOutlined, FileMarkdownOutlined, FolderOpenOutlined, ReloadOutlined } from '@ant-design/icons';

// Native JSON renderer — avoids react-json-view's React 19 incompatibility (white-screen crash).
const renderJson = (data: unknown) => {
  let text: string;
  try {
    text = JSON.stringify(data, null, 2);
  } catch {
    text = String(data);
  }
  return (
    <pre className="artifact-json-view" style={{
      margin: 0,
      padding: '14px 16px',
      background: '#1e293b',
      color: '#e2e8f0',
      borderRadius: 6,
      fontSize: 13,
      lineHeight: 1.6,
      fontFamily: "'JetBrains Mono', 'Consolas', 'Monaco', monospace",
      overflow: 'auto',
      whiteSpace: 'pre-wrap',
      wordBreak: 'break-word',
    }}>{text}</pre>
  );
};

interface ArtifactsData {
  iteration_id?: string;
  idea?: string;
  requirements?: string;
  design?: unknown;
  design_raw?: string;
  plan?: unknown;
  plan_raw?: string;
  code_files?: FileInfo[];
  check_report?: string;
  delivery_report?: string;
}

interface FileInfo {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  language?: string;
  modified_at?: string;
}

interface ArtifactsViewerProps {
  iterationId: string;
  activeTab?: string;
  onTabChange?: (key: string) => void;
  refreshTrigger?: number;
}

const ArtifactsViewer: React.FC<ArtifactsViewerProps> = ({ iterationId, activeTab: externalActiveTab, onTabChange, refreshTrigger }) => {
  const { message } = App.useApp();
  const [artifacts, setArtifacts] = useState<ArtifactsData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('idea');
  const [viewModes, setViewModes] = useState<Record<string, 'json' | 'doc'>>({});
  const [autoRefresh, setAutoRefresh] = useState(false);
  const isVisibleRef = useRef(true);
  const pollingIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const lastLoadTimeRef = useRef(0);
  const prevRefreshTriggerRef = useRef(0);

  useEffect(() => {
    if (externalActiveTab) {
      setActiveTab(externalActiveTab);
    }
  }, [externalActiveTab]);

  useEffect(() => {
    if (iterationId) {
      loadArtifacts();
    }
  }, [iterationId]);

  useEffect(() => {
    isVisibleRef.current = true;

    if (iterationId) {
      loadArtifacts();

      if (autoRefresh) {
        pollingIntervalRef.current = setInterval(() => {
          if (isVisibleRef.current && iterationId) {
            const now = Date.now();
            if (now - lastLoadTimeRef.current > 3000) {
              loadArtifacts();
            }
          }
        }, 5000);
      }
    }

    return () => {
      isVisibleRef.current = false;
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
        pollingIntervalRef.current = null;
      }
    };
  }, [iterationId, autoRefresh]);

  useEffect(() => {
    if (refreshTrigger !== undefined && refreshTrigger !== prevRefreshTriggerRef.current) {
      prevRefreshTriggerRef.current = refreshTrigger;
      console.log('[ArtifactsViewer] Refresh trigger changed, reloading artifacts...');
      loadArtifacts();
    }
  }, [refreshTrigger]);

  const loadArtifacts = async () => {
    const now = Date.now();
    if (now - lastLoadTimeRef.current < 1000) {
      return;
    }
    lastLoadTimeRef.current = now;

    setLoading(true);
    setError(null);
    try {
      const data = await invoke<ArtifactsData>('get_iteration_artifacts', { iterationId })
        .catch(() => invoke<ArtifactsData>('get_session_artifacts', { sessionId: iterationId }));
      setArtifacts(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const toggleViewMode = (tabKey: string) => {
    setViewModes(prev => ({
      ...prev,
      [tabKey]: prev[tabKey] === 'json' ? 'doc' : 'json'
    }));
  };

  const handleOpenArtifactsFolder = async () => {
    try {
      await invoke('open_in_file_manager', { path: iterationId });
    } catch (error) {
      console.error('Failed to open artifacts folder:', error);
      message.error('Failed to open artifacts folder');
    }
  };

  const handleOpenWorkspaceFolder = async () => {
    try {
      await invoke('open_in_file_manager', { path: `workspace_${iterationId}` });
    } catch (error) {
      console.error('Failed to open workspace folder:', error);
      message.error('Failed to open workspace folder');
    }
  };

  if (!artifacts) {
    return <Empty description="No artifacts available" image={Empty.PRESENTED_IMAGE_SIMPLE} />;
  }

  // ====== tab 懒构造：只有 activeTab 的 children 才包含 ReactMarkdown 节点 ======
  // 其他 tab 的 children 设为 null，切到该 tab 时才重新渲染（Antd Tabs 默认会重新挂载 children）
  const items = [];
  const isActive = (key: string) => activeTab === key;

  if (artifacts.idea) {
    items.push({
      key: 'idea',
      label: <span><FileTextOutlined /> Idea</span>,
      children: isActive('idea') ? (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Idea Document</span>
            <Tooltip title="Open artifacts folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, minHeight: 0, overflow: 'auto' }}>
            <ReactMarkdown remarkPlugins={remarkPlugins} rehypePlugins={fullRehypePlugins}>{artifacts.idea}</ReactMarkdown>
          </div>
        </div>
      ) : null,
    });
  }

  if (artifacts.requirements) {
    items.push({
      key: 'requirements',
      label: <span><ProjectOutlined /> Requirements</span>,
      children: isActive('requirements') ? (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Requirements Document</span>
            <Tooltip title="Open artifacts folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, minHeight: 0, overflow: 'auto' }}>
            <ReactMarkdown remarkPlugins={remarkPlugins} rehypePlugins={fullRehypePlugins}>{artifacts.requirements}</ReactMarkdown>
          </div>
        </div>
      ) : null,
    });
  }

  if (artifacts.design_raw || artifacts.design) {
    const designViewMode = viewModes['design'] || 'doc';
    const designContent = artifacts.design_raw || (typeof artifacts.design === 'string' ? artifacts.design : JSON.stringify(artifacts.design, null, 2));
    items.push({
      key: 'design',
      label: <span><BuildOutlined /> Design</span>,
      children: isActive('design') ? (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Design Specification</span>
            <Space>
              <Tooltip title="Open artifacts folder">
                <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
              </Tooltip>
              {!artifacts.design_raw && typeof artifacts.design === 'object' && (
                <>
                  <Button size="small" type={designViewMode === 'doc' ? 'primary' : 'default'} icon={<FileMarkdownOutlined />} onClick={() => toggleViewMode('design')}>Doc</Button>
                  <Button size="small" type={designViewMode === 'json' ? 'primary' : 'default'} onClick={() => toggleViewMode('design')}>JSON</Button>
                </>
              )}
            </Space>
          </div>
          <div className="artifact-content" style={{ flex: 1, minHeight: 0, overflow: 'auto' }}>
            {artifacts.design_raw || designViewMode === 'doc' ? (
              <div className="markdown-content">
                <ReactMarkdown remarkPlugins={remarkPlugins} rehypePlugins={fullRehypePlugins}>{designContent}</ReactMarkdown>
              </div>
            ) : (
              renderJson(artifacts.design)
            )}
          </div>
        </div>
      ) : null,
    });
  }

  if (artifacts.plan_raw || artifacts.plan) {
    const planViewMode = viewModes['plan'] || 'doc';
    const planContent = artifacts.plan_raw || (typeof artifacts.plan === 'string' ? artifacts.plan : JSON.stringify(artifacts.plan, null, 2));
    items.push({
      key: 'plan',
      label: <span><CheckCircleOutlined /> Plan</span>,
      children: isActive('plan') ? (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Implementation Plan</span>
            <Space>
              <Tooltip title="Open artifacts folder">
                <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
              </Tooltip>
              {!artifacts.plan_raw && typeof artifacts.plan === 'object' && (
                <>
                  <Button size="small" type={planViewMode === 'doc' ? 'primary' : 'default'} icon={<FileMarkdownOutlined />} onClick={() => toggleViewMode('plan')}>Doc</Button>
                  <Button size="small" type={planViewMode === 'json' ? 'primary' : 'default'} onClick={() => toggleViewMode('plan')}>JSON</Button>
                </>
              )}
            </Space>
          </div>
          <div className="artifact-content" style={{ flex: 1, minHeight: 0, overflow: 'auto' }}>
            {artifacts.plan_raw || planViewMode === 'doc' ? (
              <div className="markdown-content">
                <ReactMarkdown remarkPlugins={remarkPlugins} rehypePlugins={fullRehypePlugins}>{planContent}</ReactMarkdown>
              </div>
            ) : (
              renderJson(artifacts.plan)
            )}
          </div>
        </div>
      ) : null,
    });
  }

  if (artifacts.code_files && artifacts.code_files.length > 0) {
    items.push({
      key: 'code',
      label: <span><FileTextOutlined /> Code Files</span>,
      children: isActive('code') ? (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Code Files ({artifacts.code_files.length})</span>
            <Tooltip title="Open workspace folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenWorkspaceFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content" style={{ flex: 1, minHeight: 0, overflow: 'auto' }}>
            {renderJson(artifacts.code_files)}
          </div>
        </div>
      ) : null,
    });
  }

  if (artifacts.check_report) {
    items.push({
      key: 'check_report',
      label: <span><CheckCircleOutlined /> Check Report</span>,
      children: isActive('check_report') ? (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Check Report</span>
            <Tooltip title="Open artifacts folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, minHeight: 0, overflow: 'auto' }}>
            <ReactMarkdown remarkPlugins={remarkPlugins} rehypePlugins={fullRehypePlugins}>{artifacts.check_report}</ReactMarkdown>
          </div>
        </div>
      ) : null,
    });
  }

  if (artifacts.delivery_report) {
    items.push({
      key: 'delivery_report',
      label: <span><CheckCircleOutlined /> Delivery Report</span>,
      children: isActive('delivery_report') ? (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Delivery Report</span>
            <Tooltip title="Open artifacts folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, minHeight: 0, overflow: 'auto' }}>
            <ReactMarkdown remarkPlugins={remarkPlugins} rehypePlugins={fullRehypePlugins}>{artifacts.delivery_report}</ReactMarkdown>
          </div>
        </div>
      ) : null,
    });
  }

  const handleTabChange = (key: string) => {
    setActiveTab(key);
    if (onTabChange) {
      onTabChange(key);
    }
  };

  return (
    <div className="artifacts-viewer" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '10px 20px', borderBottom: '1px solid #e8e8e8', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#fafafa', flexShrink: 0 }}>
        <span style={{ fontWeight: 'bold', color: '#333' }}>Artifacts</span>
        <Space>
          <Tooltip title={autoRefresh ? "Auto-refresh is ON" : "Auto-refresh is OFF"}>
            <Button size="small" type={autoRefresh ? "primary" : "default"} onClick={() => setAutoRefresh(!autoRefresh)}>Auto-refresh</Button>
          </Tooltip>
          <Button size="small" icon={<ReloadOutlined />} onClick={loadArtifacts} loading={loading}>Refresh</Button>
        </Space>
      </div>
      <Tabs
        activeKey={activeTab}
        onChange={handleTabChange}
        type="card"
        size="large"
        items={items}
        style={{ height: '100%' }}
        className="artifacts-tabs"
        destroyOnHidden
      />
    </div>
  );
};

export default ArtifactsViewer;
