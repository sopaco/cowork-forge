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
    setLogs(prev => [...prev, { type: 'system', content: '> 正在启动项目...\n', timestamp: new Date() }]);
    
    const result = await tryExecute(async () => {
      return await invoke('start_iteration_project', { iterationId });
    }, '启动项目失败');
    
    setLoading(false);
    
    if (result) {
      setIsRunning(true);
      setLogs(prev => [...prev, { type: 'system', content: `> 项目已启动 (PID: ${result.process_id})\n`, timestamp: new Date() }]);
      // Refresh runtime info after starting
      loadProjectRuntimeInfo();
    }
  };

  const stopProject = async () => {
    setLoading(true);
    setLogs(prev => [...prev, { type: 'system', content: '> 正在停止项目...\n', timestamp: new Date() }]);
    
    const success = await tryExecute(async () => {
      await invoke('stop_iteration_project', { iterationId });
      return true;
    }, '停止项目失败');
    
    setLoading(false);
    
    if (success) {
      setIsRunning(false);
      setLogs(prev => [...prev, { type: 'system', content: '> 项目已停止\n', timestamp: new Date() }]);
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
  const previewUrl = projectRuntimeInfo?.preview_url;

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
          placeholder="搜索日志..."
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
          <Select.Option value="all">全部</Select.Option>
          <Select.Option value="stdout">标准输出</Select.Option>
          <Select.Option value="stderr">错误输出</Select.Option>
          <Select.Option value="system">系统</Select.Option>
        </Select>
        <Checkbox checked={autoScroll} onChange={(e) => setAutoScroll(e.target.checked)} size="small">
          自动滚动
        </Checkbox>
        <span style={{ color: '#888', fontSize: '12px', marginLeft: 'auto' }}>
          {filteredLogs.length}/{logs.length} 行
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
            点击「启动」按钮运行项目
          </div>
        ) : filteredLogs.length === 0 ? (
          <div style={{ color: '#999', textAlign: 'center', marginTop: '50px' }}>
            没有匹配的日志
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
              刷新
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
              {!hasFrontend ? '未检测到前端项目' : '请先启动项目'}
            </div>
            <div style={{ fontSize: 12, color: '#999' }}>
              {!isRunning && hasFrontend && '点击「启动」按钮运行项目后即可预览'}
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
          <span style={{ fontSize: '16px', fontWeight: 500 }}>🚀 运行中心</span>
        </div>
        
        <Space>
          {!isRunning ? (
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              onClick={startProject}
              loading={loading}
            >
              启动
            </Button>
          ) : (
            <Button danger icon={<StopOutlined />} onClick={stopProject} loading={loading}>
              停止
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
          <AppstoreOutlined /> 运行程序
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
          <EyeOutlined /> 页面预览
          {!hasFrontend && <span style={{ fontSize: '11px', marginLeft: '4px', color: '#ccc' }}>(无可用)</span>}
          {hasFrontend && !isRunning && <span style={{ fontSize: '11px', marginLeft: '4px', color: '#ccc' }}>(未启动)</span>}
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
            {isRunning ? '运行中' : '已停止'}
          </Tag>
          {hasFrontend && (
            <Tag color="blue">前端</Tag>
          )}
          {projectRuntimeInfo?.has_backend && (
            <Tag color="purple">后端</Tag>
          )}
        </div>
        
        {/* Quick actions */}
        {activeTab === 'run' && (
          <Space size="small">
            <Button size="small" icon={<ClearOutlined />} onClick={clearLogs} disabled={logs.length === 0}>
              清空
            </Button>
            <Button size="small" icon={<CopyOutlined />} onClick={copyLogs} disabled={logs.length === 0}>
              复制
            </Button>
          </Space>
        )}
      </div>
    </Card>
  );
};

export default RunnerPanel;