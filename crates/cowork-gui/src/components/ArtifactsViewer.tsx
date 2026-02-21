import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';
import JsonView from 'react-json-view';
import { Tabs, Spin, Alert, Empty, Button, Space, Tooltip, message } from 'antd';
import { FileTextOutlined, ProjectOutlined, DatabaseOutlined, BuildOutlined, CheckCircleOutlined, FileMarkdownOutlined, FolderOpenOutlined, ReloadOutlined } from '@ant-design/icons';
import 'highlight.js/styles/atom-one-dark.css';

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

  const items = [];

  if (artifacts.idea) {
    items.push({
      key: 'idea',
      label: <span><FileTextOutlined /> Idea</span>,
      children: (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Idea Document</span>
            <Tooltip title="Open artifacts folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, overflow: 'auto' }}>
            <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight, rehypeRaw]}>{artifacts.idea}</ReactMarkdown>
          </div>
        </div>
      ),
    });
  }

  if (artifacts.requirements) {
    items.push({
      key: 'requirements',
      label: <span><ProjectOutlined /> Requirements</span>,
      children: (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Requirements Document</span>
            <Tooltip title="Open artifacts folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, overflow: 'auto' }}>
            <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight, rehypeRaw]}>{artifacts.requirements}</ReactMarkdown>
          </div>
        </div>
      ),
    });
  }

  if (artifacts.design_raw || artifacts.design) {
    const designViewMode = viewModes['design'] || 'doc';
    const designContent = artifacts.design_raw || (typeof artifacts.design === 'string' ? artifacts.design : JSON.stringify(artifacts.design, null, 2));
    items.push({
      key: 'design',
      label: <span><BuildOutlined /> Design</span>,
      children: (
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
          <div className="artifact-content" style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
            {artifacts.design_raw || designViewMode === 'doc' ? (
              <div className="markdown-content">
                <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight, rehypeRaw]}>{designContent}</ReactMarkdown>
              </div>
            ) : (
              <div style={{ overflow: 'auto', maxHeight: '100%' }}>
                <JsonView src={artifacts.design as object} theme="monokai" displayObjectSize={false} enableClipboard={false} indentWidth={2} collapsed={false} quotesOnKeys={false} sortKeys={false} />
              </div>
            )}
          </div>
        </div>
      ),
    });
  }

  if (artifacts.plan_raw || artifacts.plan) {
    const planViewMode = viewModes['plan'] || 'doc';
    const planContent = artifacts.plan_raw || (typeof artifacts.plan === 'string' ? artifacts.plan : JSON.stringify(artifacts.plan, null, 2));
    items.push({
      key: 'plan',
      label: <span><CheckCircleOutlined /> Plan</span>,
      children: (
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
          <div className="artifact-content" style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
            {artifacts.plan_raw || planViewMode === 'doc' ? (
              <div className="markdown-content">
                <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight, rehypeRaw]}>{planContent}</ReactMarkdown>
              </div>
            ) : (
              <div style={{ overflow: 'auto', maxHeight: '100%' }}>
                <JsonView src={artifacts.plan as object} theme="monokai" displayObjectSize={false} enableClipboard={false} indentWidth={2} collapsed={false} quotesOnKeys={false} sortKeys={false} />
              </div>
            )}
          </div>
        </div>
      ),
    });
  }

  if (artifacts.code_files && artifacts.code_files.length > 0) {
    items.push({
      key: 'code',
      label: <span><FileTextOutlined /> Code Files</span>,
      children: (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Code Files ({artifacts.code_files.length})</span>
            <Tooltip title="Open workspace folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenWorkspaceFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content" style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
            <div style={{ overflow: 'auto', maxHeight: '100%' }}>
              <JsonView src={artifacts.code_files as unknown as object} theme="monokai" displayObjectSize={false} enableClipboard={false} indentWidth={2} collapsed={false} quotesOnKeys={false} sortKeys={false} />
            </div>
          </div>
        </div>
      ),
    });
  }

  if (artifacts.check_report) {
    items.push({
      key: 'check_report',
      label: <span><CheckCircleOutlined /> Check Report</span>,
      children: (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Check Report</span>
            <Tooltip title="Open artifacts folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, overflow: 'auto' }}>
            <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight, rehypeRaw]}>{artifacts.check_report}</ReactMarkdown>
          </div>
        </div>
      ),
    });
  }

  if (artifacts.delivery_report) {
    items.push({
      key: 'delivery_report',
      label: <span><CheckCircleOutlined /> Delivery Report</span>,
      children: (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '10px 20px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1f1f1f', flexShrink: 0 }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>Delivery Report</span>
            <Tooltip title="Open artifacts folder">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={handleOpenArtifactsFolder}>Open Folder</Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, overflow: 'auto' }}>
            <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight, rehypeRaw]}>{artifacts.delivery_report}</ReactMarkdown>
          </div>
        </div>
      ),
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
      <Tabs activeKey={activeTab} onChange={handleTabChange} type="card" size="large" items={items} style={{ height: '100%' }} className="artifacts-tabs" />
    </div>
  );
};

export default ArtifactsViewer;
