import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Spin, Button, Space, Tag, Input, Select, Checkbox } from 'antd';
import { PlayCircleOutlined, StopOutlined, CopyOutlined, ClearOutlined, SearchOutlined } from '@ant-design/icons';
import { showError, showSuccess } from '../utils/errorHandler.jsx';

const { TextArea } = Input;

const RunnerPanel = ({ sessionId }) => {
  const [logs, setLogs] = useState([]);
  const [isRunning, setIsRunning] = useState(false);
  const [loading, setLoading] = useState(false);
  const [searchText, setSearchText] = useState('');
  const [filterType, setFilterType] = useState('all'); // all, stdout, stderr
  const [autoScroll, setAutoScroll] = useState(true);
  const [maxLogs, setMaxLogs] = useState(5000);
  const logsEndRef = useRef(null);
  const logsContainerRef = useRef(null);
  const listenersRegistered = useRef(false);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollTop = logsEndRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  // Register event listeners
  useEffect(() => {
    if (listenersRegistered.current) {
      return;
    }
    listenersRegistered.current = true;

    const cleanupFunctions = [];

    // Listen for project log events
    listen('project_log', (event) => {
      const { session_id: logSessionId, stream, content } = event.payload;
      
      // Only process logs for current session
      if (logSessionId !== sessionId) {
        return;
      }

      setLogs(prev => {
        // Keep only last maxLogs entries to prevent memory issues
        const newLogs = [...prev, { type: stream, content, timestamp: new Date() }];
        return newLogs.slice(-maxLogs);
      });
    }).then(unlisten => cleanupFunctions.push(unlisten));

    // Listen for project stopped events
    listen('project_stopped', (event) => {
      const { session_id: stoppedSessionId } = event.payload;
      if (stoppedSessionId === sessionId) {
        setIsRunning(false);
      }
    }).then(unlisten => cleanupFunctions.push(unlisten));

    return () => {
      cleanupFunctions.forEach(unlisten => {
        try { unlisten(); } catch (e) {}
      });
      listenersRegistered.current = false;
    };
  }, [sessionId, maxLogs]);

  const startProject = async () => {
    setLoading(true);
    setLogs(prev => [...prev, { type: 'system', content: '> Starting project...\n', timestamp: new Date() }]);
    
    const result = await tryExecute(async () => {
      return await invoke('start_project', { sessionId });
    }, 'Failed to start project');
    
    setLoading(false);
    
    if (result) {
      setIsRunning(true);
      setLogs(prev => [...prev, { type: 'system', content: `> Project started (PID: ${result.process_id})\n`, timestamp: new Date() }]);
    }
  };

  const stopProject = async () => {
    setLoading(true);
    setLogs(prev => [...prev, { type: 'system', content: '> Stopping project...\n', timestamp: new Date() }]);
    
    const success = await tryExecute(async () => {
      await invoke('stop_project', { sessionId });
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

  // Filter logs based on search text and type
  const filteredLogs = logs.filter(log => {
    // Filter by type
    if (filterType !== 'all' && log.type !== filterType) {
      return false;
    }
    
    // Filter by search text
    if (searchText && !log.content.toLowerCase().includes(searchText.toLowerCase())) {
      return false;
    }
    
    return true;
  });

  return (
    <div className="runner-panel" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <div style={{ padding: '10px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h3>🚀 Run Project</h3>
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
          <Button icon={<ClearOutlined />} onClick={clearLogs} disabled={logs.length === 0}>
            Clear
          </Button>
          <Button icon={<CopyOutlined />} onClick={copyLogs} disabled={logs.length === 0}>
            Copy
          </Button>
        </Space>
      </div>

      {/* Filter Bar */}
      <div style={{ padding: '8px 10px', borderBottom: '1px solid #303030', display: 'flex', gap: '10px', alignItems: 'center', background: '#1a1a1a' }}>
        <Input
          placeholder="Search logs..."
          prefix={<SearchOutlined />}
          value={searchText}
          onChange={(e) => setSearchText(e.target.value)}
          style={{ width: '200px' }}
          allowClear
        />
        <Select
          value={filterType}
          onChange={setFilterType}
          style={{ width: '120px' }}
        >
          <Select.Option value="all">All Logs</Select.Option>
          <Select.Option value="stdout">Stdout</Select.Option>
          <Select.Option value="stderr">Stderr</Select.Option>
          <Select.Option value="system">System</Select.Option>
        </Select>
        <Checkbox checked={autoScroll} onChange={(e) => setAutoScroll(e.target.checked)}>
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
          backgroundColor: '#1e1e1e',
          color: '#d4d4d4',
          fontFamily: 'Consolas, "Courier New", monospace',
          fontSize: '13px',
          padding: '10px',
          overflow: 'auto',
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-all',
        }}
      >
        {logs.length === 0 ? (
          <div style={{ color: '#888', textAlign: 'center', marginTop: '50px' }}>
            Click "Start" to run your project
          </div>
        ) : filteredLogs.length === 0 ? (
          <div style={{ color: '#888', textAlign: 'center', marginTop: '50px' }}>
            No logs match the current filter
          </div>
        ) : (
          filteredLogs.map((log, index) => (
            <div key={index} style={{ 
              color: log.type === 'stderr' ? '#ff6b6b' : 
                    log.type === 'system' ? '#52c41a' : '#d4d4d4',
              marginBottom: '2px'
            }}>
              {log.content}
            </div>
          ))
        )}
      </div>

      {/* Footer */}
      <div style={{ padding: '5px 10px', borderTop: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#1a1a1a' }}>
        <Tag color={isRunning ? 'green' : 'default'}>
          Status: {isRunning ? 'Running' : 'Stopped'}
        </Tag>
        <span style={{ color: '#888', fontSize: '12px' }}>
          {logs.length} total lines
        </span>
      </div>
    </div>
  );
};

export default RunnerPanel;