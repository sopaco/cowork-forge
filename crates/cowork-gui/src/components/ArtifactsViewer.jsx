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

const ArtifactsViewer = ({ iterationId, activeTab: externalActiveTab, onTabChange, refreshTrigger }) => {
  const [artifacts, setArtifacts] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [activeTab, setActiveTab] = useState('idea');
  const [viewModes, setViewModes] = useState({});
  const [autoRefresh, setAutoRefresh] = useState(false);
  const isVisibleRef = useRef(true);
  const pollingIntervalRef = useRef(null);
  const lastLoadTimeRef = useRef(0);
  const prevRefreshTriggerRef = useRef(0);

  // Sync with external active tab when provided
  useEffect(() => {
    if (externalActiveTab) {
      setActiveTab(externalActiveTab);
    }
  }, [externalActiveTab]);

  // Load artifacts when iterationId changes
  useEffect(() => {
    if (iterationId) {
      loadArtifacts();
    }
  }, [iterationId]);

  // Track visibility and set up polling
  useEffect(() => {
    isVisibleRef.current = true;

    // Load artifacts when component becomes visible
    if (iterationId) {
      loadArtifacts();

      // Set up polling to refresh artifacts periodically
      if (autoRefresh) {
        pollingIntervalRef.current = setInterval(() => {
          if (isVisibleRef.current && iterationId) {
            const now = Date.now();
            // Only refresh if at least 3 seconds have passed since last load
            if (now - lastLoadTimeRef.current > 3000) {
              loadArtifacts();
            }
          }
        }, 5000); // Check every 5 seconds
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

  // Listen for refresh trigger changes (immediate refresh)
  useEffect(() => {
    if (refreshTrigger !== undefined && refreshTrigger !== prevRefreshTriggerRef.current) {
      prevRefreshTriggerRef.current = refreshTrigger;
      console.log('[ArtifactsViewer] Refresh trigger changed, reloading artifacts...');
      loadArtifacts();
    }
  }, [refreshTrigger]);

  const loadArtifacts = async () => {
    // Prevent duplicate requests within 1 second
    const now = Date.now();
    if (now - lastLoadTimeRef.current < 1000) {
      return;
    }
    lastLoadTimeRef.current = now;

    setLoading(true);
    setError(null);
    try {
      // Try new V2 API first, fall back to legacy API
      const data = await invoke('get_iteration_artifacts', { iterationId })
        .catch(() => invoke('get_session_artifacts', { sessionId: iterationId }));
      setArtifacts(data);
    } catch (err) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const toggleViewMode = (tabKey) => {
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
      // Pass workspace prefix to indicate it's a workspace path
      await invoke('open_in_file_manager', { path: `workspace_${iterationId}` });
    } catch (error) {
      console.error('Failed to open workspace folder:', error);
      message.error('Failed to open workspace folder');
    }
  };

  if (!artifacts) {
    return (
      <Empty
        description="No artifacts available"
        image={Empty.PRESENTED_IMAGE_SIMPLE}
      />
    );
  }

  const items = [];

  if (artifacts.idea) {
    items.push({
      key: 'idea',
      label: <span><FileTextOutlined /> Idea</span>,
      children: (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{ 
            padding: '10px 20px', 
            borderBottom: '1px solid #303030', 
            display: 'flex', 
            justifyContent: 'space-between', 
            alignItems: 'center',
            background: '#1f1f1f',
            flexShrink: 0
          }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>
              Idea Document
            </span>
            <Tooltip title="Open artifacts folder">
              <Button 
                size="small" 
                icon={<FolderOpenOutlined />}
                onClick={handleOpenArtifactsFolder}
              >
                Open Folder
              </Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, overflow: 'auto' }}>
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight, rehypeRaw]}
            >
              {artifacts.idea}
            </ReactMarkdown>
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
          <div style={{ 
            padding: '10px 20px', 
            borderBottom: '1px solid #303030', 
            display: 'flex', 
            justifyContent: 'space-between', 
            alignItems: 'center',
            background: '#1f1f1f',
            flexShrink: 0
          }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>
              Requirements Document
            </span>
            <Tooltip title="Open artifacts folder">
              <Button 
                size="small" 
                icon={<FolderOpenOutlined />}
                onClick={handleOpenArtifactsFolder}
              >
                Open Folder
              </Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, overflow: 'auto' }}>
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight, rehypeRaw]}
            >
              {artifacts.requirements}
            </ReactMarkdown>
          </div>
        </div>
      ),
    });
  }

  // Features tab - removed as it's no longer supported in V2

  if (artifacts.design_raw || artifacts.design) {
    const designViewMode = viewModes['design'] || 'doc';
    // If design is a string (markdown), use it directly; if it's an object, convert to markdown
    const designContent = artifacts.design_raw || 
      (typeof artifacts.design === 'string' ? artifacts.design : designToMarkdown(artifacts.design));
    items.push({
      key: 'design',
      label: <span><BuildOutlined /> Design</span>,
      children: (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{
            padding: '10px 20px',
            borderBottom: '1px solid #303030',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            background: '#1f1f1f',
            flexShrink: 0
          }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>
              Design Specification
            </span>
            <Space>
              <Tooltip title="Open artifacts folder">
                <Button 
                  size="small" 
                  icon={<FolderOpenOutlined />}
                  onClick={handleOpenArtifactsFolder}
                >
                  Open Folder
                </Button>
              </Tooltip>
              {!artifacts.design_raw && typeof artifacts.design === 'object' && (
                <>
                  <Button
                    size="small"
                    type={designViewMode === 'doc' ? 'primary' : 'default'}
                    icon={<FileMarkdownOutlined />}
                    onClick={() => toggleViewMode('design')}
                  >
                    Doc
                  </Button>
                  <Button
                    size="small"
                    type={designViewMode === 'json' ? 'primary' : 'default'}
                    onClick={() => toggleViewMode('design')}
                  >
                    JSON
                  </Button>
                </>
              )}
            </Space>
          </div>
          <div className="artifact-content" style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
            {artifacts.design_raw || designViewMode === 'doc' ? (
              <div className="markdown-content">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  rehypePlugins={[rehypeHighlight, rehypeRaw]}
                >
                  {designContent}
                </ReactMarkdown>
              </div>
            ) : (
              <div style={{ overflow: 'auto', maxHeight: '100%' }}>
                <JsonView
                  src={artifacts.design}
                  theme="dark"
                  displayObjectSize={false}
                  enableClipboard={false}
                  indentWidth={2}
                  collapsed={false}
                  quotesOnKeys={false}
                  sortKeys={false}
                />
              </div>
            )}
          </div>
        </div>
      ),
    });
  }

  if (artifacts.plan_raw || artifacts.plan) {
    const planViewMode = viewModes['plan'] || 'doc';
    // If plan is a string (markdown), use it directly; if it's an object, convert to markdown
    const planContent = artifacts.plan_raw || 
      (typeof artifacts.plan === 'string' ? artifacts.plan : planToMarkdown(artifacts.plan));
    items.push({
      key: 'plan',
      label: <span><CheckCircleOutlined /> Plan</span>,
      children: (
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <div style={{
            padding: '10px 20px',
            borderBottom: '1px solid #303030',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            background: '#1f1f1f',
            flexShrink: 0
          }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>
              Implementation Plan
            </span>
            <Space>
              <Tooltip title="Open artifacts folder">
                <Button 
                  size="small" 
                  icon={<FolderOpenOutlined />}
                  onClick={handleOpenArtifactsFolder}
                >
                  Open Folder
                </Button>
              </Tooltip>
              {!artifacts.plan_raw && typeof artifacts.plan === 'object' && (
                <>
                  <Button
                    size="small"
                    type={planViewMode === 'doc' ? 'primary' : 'default'}
                    icon={<FileMarkdownOutlined />}
                    onClick={() => toggleViewMode('plan')}
                  >
                    Doc
                  </Button>
                  <Button
                    size="small"
                    type={planViewMode === 'json' ? 'primary' : 'default'}
                    onClick={() => toggleViewMode('plan')}
                  >
                    JSON
                  </Button>
                </>
              )}
            </Space>
          </div>
          <div className="artifact-content" style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
            {artifacts.plan_raw || planViewMode === 'doc' ? (
              <div className="markdown-content">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  rehypePlugins={[rehypeHighlight, rehypeRaw]}
                >
                  {planContent}
                </ReactMarkdown>
              </div>
            ) : (
              <div style={{ overflow: 'auto', maxHeight: '100%' }}>
                <JsonView
                  src={artifacts.plan}
                  theme="dark"
                  displayObjectSize={false}
                  enableClipboard={false}
                  indentWidth={2}
                  collapsed={false}
                  quotesOnKeys={false}
                  sortKeys={false}
                />
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
          <div style={{
            padding: '10px 20px',
            borderBottom: '1px solid #303030',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            background: '#1f1f1f',
            flexShrink: 0
          }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>
              Code Files ({artifacts.code_files.length})
            </span>
            <Tooltip title="Open workspace folder">
              <Button 
                size="small" 
                icon={<FolderOpenOutlined />}
                onClick={handleOpenWorkspaceFolder}
              >
                Open Folder
              </Button>
            </Tooltip>
          </div>
          <div className="artifact-content" style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
            <div style={{ overflow: 'auto', maxHeight: '100%' }}>
              <JsonView
                src={artifacts.code_files}
                theme="dark"
                displayObjectSize={false}
                enableClipboard={false}
                indentWidth={2}
                collapsed={false}
                quotesOnKeys={false}
                sortKeys={false}
              />
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
          <div style={{ 
            padding: '10px 20px', 
            borderBottom: '1px solid #303030', 
            display: 'flex', 
            justifyContent: 'space-between', 
            alignItems: 'center',
            background: '#1f1f1f',
            flexShrink: 0
          }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>
              Check Report
            </span>
            <Tooltip title="Open artifacts folder">
              <Button 
                size="small" 
                icon={<FolderOpenOutlined />}
                onClick={handleOpenArtifactsFolder}
              >
                Open Folder
              </Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, overflow: 'auto' }}>
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight, rehypeRaw]}
            >
              {artifacts.check_report}
            </ReactMarkdown>
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
          <div style={{ 
            padding: '10px 20px', 
            borderBottom: '1px solid #303030', 
            display: 'flex', 
            justifyContent: 'space-between', 
            alignItems: 'center',
            background: '#1f1f1f',
            flexShrink: 0
          }}>
            <span style={{ fontWeight: 'bold', color: '#fff' }}>
              Delivery Report
            </span>
            <Tooltip title="Open artifacts folder">
              <Button 
                size="small" 
                icon={<FolderOpenOutlined />}
                onClick={handleOpenArtifactsFolder}
              >
                Open Folder
              </Button>
            </Tooltip>
          </div>
          <div className="artifact-content markdown-content" style={{ flex: 1, overflow: 'auto' }}>
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight, rehypeRaw]}
            >
              {artifacts.delivery_report}
            </ReactMarkdown>
          </div>
        </div>
      ),
    });
  }

  // Handle tab change
  const handleTabChange = (key) => {
    setActiveTab(key);
    if (onTabChange) {
      onTabChange(key);
    }
  };

  return (
    <div className="artifacts-viewer" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header with refresh controls */}
      <div style={{ 
        padding: '10px 20px', 
        borderBottom: '1px solid #e8e8e8', 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center',
        background: '#fafafa',
        flexShrink: 0
      }}>
        <span style={{ fontWeight: 'bold', color: '#333' }}>
          Artifacts
        </span>
        <Space>
          <Tooltip title={autoRefresh ? "Auto-refresh is ON" : "Auto-refresh is OFF"}>
            <Button
              size="small"
              type={autoRefresh ? "primary" : "default"}
              onClick={() => setAutoRefresh(!autoRefresh)}
            >
              Auto-refresh
            </Button>
          </Tooltip>
          <Button
            size="small"
            icon={<ReloadOutlined />}
            onClick={loadArtifacts}
            loading={loading}
          >
            Refresh
          </Button>
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
      />
    </div>
  );
};

export default ArtifactsViewer;