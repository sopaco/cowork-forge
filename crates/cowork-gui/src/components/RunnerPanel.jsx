import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Spin, Button, Space, Tag, Input, Select, Checkbox, Card } from 'antd';
import { PlayCircleOutlined, StopOutlined, CopyOutlined, ClearOutlined, SearchOutlined, EyeOutlined, ReloadOutlined, AppstoreOutlined } from '@ant-design/icons';
import { showError, showSuccess } from '../utils/errorHandler.jsx';

const { TextArea } = Input;

// 过滤 ANSI 转义序列（彩色日志）
const stripAnsi = (text) => {
  // ANSI 转义序列正则: \x1b[...m
  return text.replace(/\x1b\[[0-9;]*m/g, '');
};

const RunnerPanel = ({ iterationId }) => {
  const [logs, setLogs] = useState([]);
  const [isRunning, setIsRunning] = useState(false);
  const [loading, setLoading] = useState(false);
  const [searchText, setSearchText] = useState('');
  const [filterType, setFilterType] = useState('all');
  const [autoScroll, setAutoScroll] = useState(true);
  const [maxLogs, setMaxLogs] = useState(5000);
  
  // Tab state
  const [activeTab, setActiveTab] = useState('run');
  const [projectRuntimeInfo, setProjectRuntimeInfo] = useState(null);
  
  const logsEndRef = useRef(null);
  const listenersRegistered = useRef(false);
  const isVisibleRef = useRef(true);

  // Check project status and load runtime info
  useEffect(() => {
    if (isVisibleRef.current && iterationId) {
      checkProjectStatus();
      loadProjectRuntimeInfo();
    }
  }, [iterationId]);

  useEffect(() => {
    isVisibleRef.current = true;
    checkProjectStatus();
    loadProjectRuntimeInfo();

    return () => {
      isVisibleRef.current = false;
    };
  }, [iterationId]);

  const checkProjectStatus = async () => {
    try {
      const running = await invoke('check_project_status', { iterationId });
      setIsRunning(running);
    } catch (error) {
      console.error('Failed to check project status:', error);
    }
  };

  const loadProjectRuntimeInfo = async () => {
    try {
      const info = await invoke('get_project_runtime_info', { iterationId });
      setProjectRuntimeInfo(info);
    } catch (error) {
      console.error('Failed to load project runtime info:', error);
      setProjectRuntimeInfo(null);
    }
  };

  // Auto-scroll
  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollTop = logsEndRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  // Event listeners
  useEffect(() => {
    if (listenersRegistered.current) {
      return;
    }
    listenersRegistered.current = true;

    const cleanupFunctions = [];

    listen('project_log', (event) => {
      const { iteration_id: logIterationId, session_id: logSessionId, stream, content } = event.payload;
      const targetId = logIterationId || logSessionId;
      if (targetId !== iterationId) {
        return;
      }

      // 过滤 ANSI 转义序列
      const cleanContent = stripAnsi(content);
      
      setLogs(prev => {
        const newLogs = [...prev, { type: stream, content: cleanContent, timestamp: new Date() }];
        return newLogs.slice(-maxLogs);
      });
    }).then(unlisten => cleanupFunctions.push(unlisten));

    listen('project_stopped', (event) => {
      const { iteration_id: stoppedIterationId, session_id: stoppedSessionId } = event.payload;
      const targetId = stoppedIterationId || stoppedSessionId;
      if (targetId === iterationId) {
        setIsRunning(false);
      }
    }).then(unlisten => cleanupFunctions.push(unlisten));

    return () => {
      cleanupFunctions.forEach(unlisten => {
        try { unlisten(); } catch (e) {}
      });
      listenersRegistered.current = false;
    };
  }, [iterationId, maxLogs]);

  const tryExecute = async (fn, errorMsg) => {
    try {
      return await fn();
    } catch (err) {
      console.error(errorMsg, err);
      showError(`${errorMsg}: ${err}`);
      return null;
    }
  };

  const startProject = async () => {
    setLoading(true);
    setLogs(prev => [...prev, { type: 'system', content: '> Starting project...\n', timestamp: new Date() }]);
    
    const result = await tryExecute(async () => {
      return await invoke('start_iteration_project', { iterationId });
    }, 'Failed to start project');
    
    setLoading(false);
    
    if (result) {
      setIsRunning(true);
      
      // Display startup info based on project type
      if (result.is_fullstack) {
        // Fullstack project - show both frontend and backend PIDs
        setLogs(prev => [
          ...prev, 
          { type: 'system', content: `> Fullstack project started\n`, timestamp: new Date() },
          { type: 'system', content: `> Frontend PID: ${result.frontend_pid} | URL: ${result.frontend_url}\n`, timestamp: new Date() },
          { type: 'system', content: `> Backend PID: ${result.backend_pid} | URL: ${result.backend_url}\n`, timestamp: new Date() },
        ]);
      } else if (result.process_id) {
        // Single process project
        setLogs(prev => [...prev, { type: 'system', content: `> Project started (PID: ${result.process_id})\n`, timestamp: new Date() }]);
      } else {
        // Built-in static server
        setLogs(prev => [...prev, { type: 'system', content: `> Project started: ${result.command}\n`, timestamp: new Date() }]);
      }
      
      // Refresh runtime info after starting
      loadProjectRuntimeInfo();
    }
  };

  const stopProject = async () => {
    setLoading(true);
    setLogs(prev => [...prev, { type: 'system', content: '> Stopping project...\n', timestamp: new Date() }]);
    
    const success = await tryExecute(async () => {
      await invoke('stop_iteration_project', { iterationId });
      return true;
    }, 'Failed to stop project');
    
    setLoading(false);
    
    if (success) {
      setIsRunning(false);
      setLogs(prev => [...prev, { type: 'system', content: '> Project stopped\n', timestamp: new Date() }]);
    }
  };

  const clearLogs = () => {
    setLogs([]);
  };

  const copyLogs = () => {
    const logsText = logs.map(log => log.content).join('');
    navigator.clipboard.writeText(logsText);
  };

  const refreshPreview = () => {
    const iframe = document.querySelector('.preview-iframe');
    if (iframe) {
      iframe.src = iframe.src;
    }
  };

  // Filter logs
  const filteredLogs = logs.filter(log => {
    if (filterType !== 'all' && log.type !== filterType) {
      return false;
    }
    if (searchText && !log.content.toLowerCase().includes(searchText.toLowerCase())) {
      return false;
    }
    return true;
  });

  const hasFrontend = projectRuntimeInfo?.has_frontend === true;
  const hasBackend = projectRuntimeInfo?.has_backend === true;
  const isFullstack = hasFrontend && hasBackend;
  const previewUrl = projectRuntimeInfo?.preview_url;
  const frontendPort = projectRuntimeInfo?.frontend_port;
  const backendPort = projectRuntimeInfo?.backend_port;

  // Render Run Tab (运行程序)
  const renderRunTab = () => (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Filter Bar */}
      <div style={{ 
        padding: '8px 12px', 
        borderBottom: '1px solid #f0f0f0', 
        display: 'flex', 
        gap: '10px', 
        alignItems: 'center', 
        background: '#fafafa',
        flexShrink: 0
      }}>
        <Input
          placeholder="Search logs..."
          prefix={<SearchOutlined />}
          value={searchText}
          onChange={(e) => setSearchText(e.target.value)}
          style={{ width: '180px' }}
          allowClear
          size="small"
        />
        <Select
          value={filterType}
          onChange={setFilterType}
          style={{ width: '100px' }}
          size="small"
        >
          <Select.Option value="all">All</Select.Option>
          <Select.Option value="stdout">Stdout</Select.Option>
          <Select.Option value="stderr">Stderr</Select.Option>
          <Select.Option value="system">System</Select.Option>
        </Select>
        <Checkbox checked={autoScroll} onChange={(e) => setAutoScroll(e.target.checked)} size="small">
          Auto-scroll
        </Checkbox>
        <span style={{ color: '#888', fontSize: '12px', marginLeft: 'auto' }}>
          {filteredLogs.length}/{logs.length} lines
        </span>
      </div>

      {/* Logs Display */}
      <div
        ref={logsEndRef}
        style={{
          flex: 1,
          backgroundColor: '#f5f5f5',
          color: '#333',
          fontFamily: 'Consolas, "Courier New", monospace',
          fontSize: '13px',
          padding: '10px',
          overflow: 'auto',
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-all',
          border: '1px solid #d9d9d9',
          margin: '8px',
          borderRadius: '4px',
        }}
      >
        {logs.length === 0 ? (
          <div style={{ color: '#999', textAlign: 'center', marginTop: '50px' }}>
            Click "Start" to run your project
          </div>
        ) : filteredLogs.length === 0 ? (
          <div style={{ color: '#999', textAlign: 'center', marginTop: '50px' }}>
            No matching logs
          </div>
        ) : (
          filteredLogs.map((log, index) => (
            <div key={index} style={{ 
              color: log.type === 'stderr' ? '#cf1322' : 
                    log.type === 'system' ? '#389e0d' : '#333',
              marginBottom: '2px'
            }}>
              {log.content}
            </div>
          ))
        )}
      </div>
    </div>
  );

  // Render Preview Tab (页面预览)
  const renderPreviewTab = () => (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', backgroundColor: '#f5f5f5', height: '100%', overflow: 'hidden' }}>
      {isRunning && previewUrl ? (
        <>
          {/* Fullstack Info Banner */}
          {isFullstack && (
            <div style={{ 
              padding: '8px 12px',
              backgroundColor: '#e6f7ff',
              borderBottom: '1px solid #91d5ff',
              display: 'flex',
              alignItems: 'center',
              gap: '16px',
              flexShrink: 0
            }}>
              <span style={{ fontSize: 12, fontWeight: 500, color: '#0050b3' }}>
                🔗 Fullstack Mode
              </span>
              <span style={{ fontSize: 12, color: '#666' }}>
                Frontend: <a href={previewUrl} target="_blank" rel="noopener noreferrer" style={{ color: '#1890ff' }}>{previewUrl}</a>
              </span>
              <span style={{ fontSize: 12, color: '#666' }}>
                Backend API: <a href={`http://localhost:${backendPort}`} target="_blank" rel="noopener noreferrer" style={{ color: '#722ed1' }}>http://localhost:{backendPort}</a>
              </span>
            </div>
          )}
          
          {/* Preview Toolbar */}
          <div style={{ 
            padding: '8px 12px',
            backgroundColor: '#ffffff',
            borderBottom: '1px solid #d9d9d9',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            flexShrink: 0
          }}>
            <span style={{ fontSize: 12, color: '#666', fontWeight: 500 }}>URL:</span>
            <span style={{ fontSize: 13, color: '#1890ff', fontFamily: 'monospace', flex: 1 }}>{previewUrl}</span>
            <Button 
              icon={<ReloadOutlined />} 
              size="small" 
              onClick={refreshPreview}
            >
              Refresh
            </Button>
          </div>
          
          {/* Preview iframe */}
          <iframe
            src={previewUrl}
            className="preview-iframe"
            style={{ flex: 1, width: '100%', border: 'none', backgroundColor: '#ffffff', margin: '8px', borderRadius: '4px' }}
            title="Preview"
            sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
          />
        </>
      ) : (
        <div style={{ 
          flex: 1,
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center', 
          flexDirection: 'column',
          gap: 16,
          backgroundColor: '#ffffff',
          margin: '8px',
          borderRadius: '4px',
          border: '1px solid #d9d9d9'
        }}>
          <EyeOutlined style={{ fontSize: 64, color: '#d9d9d9' }} />
          <div style={{ textAlign: 'center', color: '#666' }}>
            <div style={{ fontSize: 14, marginBottom: 8 }}>
              {!hasFrontend ? 'No frontend project detected' : 'Please start the project first'}
            </div>
            <div style={{ fontSize: 12, color: '#999' }}>
              {!isRunning && hasFrontend && 'Click "Start" to run the project for preview'}
            </div>
          </div>
        </div>
      )}
    </div>
  );

  return (
    <Card 
      className="runner-panel" 
      style={{ height: '100%', display: 'flex', flexDirection: 'column', margin: '8px', borderRadius: '8px' }}
      bodyStyle={{ flex: 1, display: 'flex', flexDirection: 'column', padding: 0, overflow: 'hidden' }}
    >
      {/* Header */}
      <div style={{ 
        padding: '12px 16px', 
        borderBottom: '1px solid #f0f0f0', 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center',
        background: '#ffffff'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ fontSize: '16px', fontWeight: 500 }}>🚀 Run Center</span>
        </div>
        
        <Space>
          {!isRunning ? (
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              onClick={startProject}
              loading={loading}
            >
              Start
            </Button>
          ) : (
            <Button danger icon={<StopOutlined />} onClick={stopProject} loading={loading}>
              Stop
            </Button>
          )}
        </Space>
      </div>

      {/* Tab Bar */}
      <div style={{ 
        background: '#fafafa', 
        borderBottom: '1px solid #f0f0f0',
        display: 'flex',
        padding: '0 16px',
        gap: '4px'
      }}>
        <button
          onClick={() => setActiveTab('run')}
          style={{
            background: activeTab === 'run' ? '#ffffff' : 'transparent',
            border: 'none',
            borderBottom: activeTab === 'run' ? '2px solid #1890ff' : '2px solid transparent',
            color: activeTab === 'run' ? '#1890ff' : '#666',
            padding: '10px 16px',
            cursor: 'pointer',
            fontSize: '13px',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            transition: 'all 0.2s'
          }}
        >
          <AppstoreOutlined /> Run Program
        </button>
        <button
          onClick={() => hasFrontend && isRunning && setActiveTab('preview')}
          disabled={!hasFrontend || !isRunning}
          style={{
            background: activeTab === 'preview' ? '#ffffff' : 'transparent',
            border: 'none',
            borderBottom: activeTab === 'preview' ? '2px solid #1890ff' : '2px solid transparent',
            color: !hasFrontend ? '#ccc' : !isRunning ? '#ccc' : activeTab === 'preview' ? '#1890ff' : '#666',
            padding: '10px 16px',
            cursor: (!hasFrontend || !isRunning) ? 'not-allowed' : 'pointer',
            fontSize: '13px',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            opacity: (!hasFrontend || !isRunning) ? 0.6 : 1,
            transition: 'all 0.2s'
          }}
        >
          <EyeOutlined /> Page Preview
          {!hasFrontend && <span style={{ fontSize: '11px', marginLeft: '4px', color: '#ccc' }}>(N/A)</span>}
          {hasFrontend && !isRunning && <span style={{ fontSize: '11px', marginLeft: '4px', color: '#ccc' }}>(Stopped)</span>}
        </button>
      </div>

      {/* Tab Content */}
      <div style={{ flex: 1, overflow: 'hidden', background: '#ffffff' }}>
        {activeTab === 'run' ? renderRunTab() : renderPreviewTab()}
      </div>

      {/* Footer */}
      <div style={{ 
        padding: '8px 16px', 
        borderTop: '1px solid #f0f0f0', 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center', 
        background: '#fafafa',
        flexShrink: 0
      }}>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <Tag color={isRunning ? 'success' : 'default'}>
            {isRunning ? 'Running' : 'Stopped'}
          </Tag>
          {isFullstack ? (
            <>
              <Tag color="blue">Frontend:{frontendPort}</Tag>
              <Tag color="purple">Backend:{backendPort}</Tag>
            </>
          ) : (
            <>
              {hasFrontend && (
                <Tag color="blue">Frontend</Tag>
              )}
              {hasBackend && (
                <Tag color="purple">Backend</Tag>
              )}
            </>
          )}
        </div>
        
        {/* Quick actions */}
        {activeTab === 'run' && (
          <Space size="small">
            <Button size="small" icon={<ClearOutlined />} onClick={clearLogs} disabled={logs.length === 0}>
              Clear
            </Button>
            <Button size="small" icon={<CopyOutlined />} onClick={copyLogs} disabled={logs.length === 0}>
              Copy
            </Button>
          </Space>
        )}
      </div>
    </Card>
  );
};

export default RunnerPanel;