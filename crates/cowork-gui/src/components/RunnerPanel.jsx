import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Spin, Button, Space, Tag } from 'antd';
import { PlayCircleOutlined, StopOutlined, CopyOutlined } from '@ant-design/icons';

const RunnerPanel = ({ sessionId }) => {
  const [logs, setLogs] = useState([]);
  const [isRunning, setIsRunning] = useState(false);
  const [loading, setLoading] = useState(false);
  const logsEndRef = useRef(null);

  useEffect(() => {
    if (logsEndRef.current) {
      logsEndRef.current.scrollTop = logsEndRef.current.scrollHeight;
    }
  }, [logs]);

  const startProject = async () => {
    setLoading(true);
    setLogs(prev => [...prev, '> Starting project...\n']);
    try {
      const result = await invoke('start_project', { sessionId });
      setIsRunning(true);
      setLogs(prev => [...prev, `> Project started (PID: ${result.process_id})\n`]);
    } catch (err) {
      setLogs(prev => [...prev, `> Error: ${err}\n`]);
    } finally {
      setLoading(false);
    }
  };

  const stopProject = async () => {
    setLoading(true);
    setLogs(prev => [...prev, '> Stopping project...\n']);
    try {
      await invoke('stop_project', { sessionId });
      setIsRunning(false);
      setLogs(prev => [...prev, '> Project stopped\n']);
    } catch (err) {
      setLogs(prev => [...prev, `> Error: ${err}\n`]);
    } finally {
      setLoading(false);
    }
  };

  const copyLogs = () => {
    const logsText = logs.join('');
    navigator.clipboard.writeText(logsText);
  };

  return (
    <div className="runner-panel" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
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
          <Button icon={<CopyOutlined />} onClick={copyLogs}>
            Copy
          </Button>
        </Space>
      </div>

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
        ) : (
          logs.map((log, index) => (
            <div key={index}>{log}</div>
          ))
        )}
      </div>

      <div style={{ padding: '5px 10px', borderTop: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Tag color={isRunning ? 'green' : 'default'}>
          Status: {isRunning ? 'Running' : 'Stopped'}
        </Tag>
        <span style={{ color: '#888', fontSize: '12px' }}>
          {logs.length} lines
        </span>
      </div>
    </div>
  );
};

export default RunnerPanel;