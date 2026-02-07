import React, { useState, useEffect } from 'react';
import { Button, Spin, Alert } from 'antd';
import { PlayCircleOutlined, StopOutlined, ReloadOutlined, EyeOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import '../styles.css';

const PreviewPanel = ({ iterationId }) => {
  const [previewUrl, setPreviewUrl] = useState(null);
  const [isRunning, setIsRunning] = useState(false);
  const [loading, setLoading] = useState(false);

  // Check server status on mount
  useEffect(() => {
    checkPreviewStatus();
  }, [iterationId]);

  const checkPreviewStatus = async () => {
    try {
      const info = await invoke('check_preview_status', { iterationId });
      if (info) {
        setPreviewUrl(info.url);
        setIsRunning(info.status === 'Running');
      } else {
        setPreviewUrl(null);
        setIsRunning(false);
      }
    } catch (error) {
      console.error('Failed to check preview status:', error);
    }
  };

  const startPreview = async () => {
    setLoading(true);
    try {
      const result = await invoke('start_iteration_preview', { iterationId })
        .catch(() => invoke('start_preview', { sessionId: iterationId }));
      setPreviewUrl(result.url);
      setIsRunning(true);
    } catch (error) {
      console.error('Failed to start preview:', error);
      setIsRunning(false);
    } finally {
      setLoading(false);
    }
  };

  const stopPreview = async () => {
    setLoading(true);
    try {
      await invoke('stop_iteration_preview', { iterationId })
        .catch(() => invoke('stop_preview', { sessionId: iterationId }));
      setPreviewUrl(null);
      setIsRunning(false);
    } catch (error) {
      console.error('Failed to stop preview:', error);
    } finally {
      setLoading(false);
    }
  };

  const refreshPreview = () => {
    if (previewUrl) {
      const iframe = document.querySelector('.preview-frame');
      if (iframe) {
        iframe.src = iframe.src; // Force reload
      }
    }
  };

  return (
    <div className="preview-panel" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <h3>🌐 Preview</h3>
      
      <div style={{ marginBottom: 16, display: 'flex', gap: 8 }}>
        {!isRunning ? (
          <Button 
            type="primary" 
            icon={<PlayCircleOutlined />} 
            onClick={startPreview}
            loading={loading}
          >
            Start Preview
          </Button>
        ) : (
          <>
            <Button 
              icon={<ReloadOutlined />} 
              onClick={refreshPreview}
            >
              Refresh
            </Button>
            <Button 
              danger 
              icon={<StopOutlined />} 
              onClick={stopPreview}
              loading={loading}
            >
              Stop
            </Button>
          </>
        )}
      </div>

      {previewUrl && (
        <Alert
          message="Preview URL"
          description={
            <a href={previewUrl} target="_blank" rel="noopener noreferrer">
              {previewUrl}
            </a>
          }
          type="info"
          showIcon
          closable
          style={{ marginBottom: 16 }}
        />
      )}

      <div style={{ flex: 1, backgroundColor: '#f0f0f0', borderRadius: 4, overflow: 'hidden' }}>
        {isRunning && previewUrl ? (
          <iframe
            src={previewUrl}
            className="preview-frame"
            style={{ width: '100%', height: '100%', border: 'none' }}
            title="Preview"
            sandbox="allow-scripts allow-same-origin allow-forms"
          />
        ) : loading ? (
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center', 
            height: '100%',
            flexDirection: 'column',
            gap: 16
          }}>
            <Spin size="large" />
            <div>Starting preview...</div>
          </div>
        ) : (
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center', 
            height: '100%',
            color: '#999',
            textAlign: 'center',
            padding: 20
          }}>
            <div>
              <EyeOutlined style={{ fontSize: 48, marginBottom: 16, display: 'block' }} />
              <div>Click "Start Preview" to preview your application</div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default PreviewPanel;