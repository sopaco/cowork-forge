import React, { useState, useEffect, useRef } from 'react';
import { Button, Spin, Alert, Space, Typography } from 'antd';
import { PlayCircleOutlined, StopOutlined, ReloadOutlined, EyeOutlined, GlobalOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import '../styles.css';

const { Text, Title } = Typography;

const PreviewPanel = ({ iterationId }) => {
  const [previewUrl, setPreviewUrl] = useState(null);
  const [isRunning, setIsRunning] = useState(false);
  const [loading, setLoading] = useState(false);
  const isVisibleRef = useRef(true);

  // Check server status when component mounts or becomes visible
  useEffect(() => {
    if (isVisibleRef.current) {
      checkPreviewStatus();
    }
  }, [iterationId]);

  // Track visibility and refresh when component becomes visible again
  useEffect(() => {
    isVisibleRef.current = true;
    
    // Check status when component becomes visible
    checkPreviewStatus();

    return () => {
      isVisibleRef.current = false;
    };
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
    <div className="preview-panel" style={{ 
      height: '100%', 
      display: 'flex', 
      flexDirection: 'column',
      backgroundColor: '#f8fafc',
      padding: 0
    }}>
      {/* Header */}
      <div style={{ 
        padding: '16px 20px',
        borderBottom: '1px solid #e2e8f0',
        backgroundColor: '#ffffff',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center'
      }}>
        <Space align="center">
          <GlobalOutlined style={{ fontSize: 20, color: '#2563eb' }} />
          <Title level={4} style={{ margin: 0, color: '#1e293b' }}>
            Preview
          </Title>
        </Space>
        
        <Space size="small">
          {!isRunning ? (
            <Button 
              type="primary" 
              icon={<PlayCircleOutlined />} 
              onClick={startPreview}
              loading={loading}
              size="middle"
            >
              Start
            </Button>
          ) : (
            <>
              <Button 
                icon={<ReloadOutlined />} 
                onClick={refreshPreview}
                size="middle"
              >
                Refresh
              </Button>
              <Button 
                danger 
                icon={<StopOutlined />} 
                onClick={stopPreview}
                loading={loading}
                size="middle"
              >
                Stop
              </Button>
            </>
          )}
        </Space>
      </div>

      {/* Preview URL Bar */}
      {previewUrl && (
        <div style={{ 
          padding: '12px 20px',
          backgroundColor: '#ffffff',
          borderBottom: '1px solid #e2e8f0'
        }}>
          <Space align="center" style={{ width: '100%' }}>
            <Text type="secondary" style={{ fontSize: 12, fontWeight: 500 }}>
              URL:
            </Text>
            <Text 
              copyable={{ text: previewUrl }} 
              style={{ 
                color: '#2563eb',
                fontSize: 13,
                fontFamily: 'monospace'
              }}
            >
              {previewUrl}
            </Text>
          </Space>
        </div>
      )}

      {/* Preview Content */}
      <div style={{ flex: 1, backgroundColor: '#f1f5f9', overflow: 'hidden' }}>
        {isRunning && previewUrl ? (
          <iframe
            src={previewUrl}
            className="preview-frame"
            style={{ width: '100%', height: '100%', border: 'none', backgroundColor: '#ffffff' }}
            title="Preview"
            sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
          />
        ) : loading ? (
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center', 
            height: '100%',
            flexDirection: 'column',
            gap: 16,
            backgroundColor: '#ffffff'
          }}>
            <Spin size="large" />
            <Text type="secondary">Starting preview server...</Text>
          </div>
        ) : (
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center', 
            height: '100%',
            backgroundColor: '#ffffff'
          }}>
            <div style={{ textAlign: 'center', color: '#94a3b8' }}>
              <EyeOutlined style={{ fontSize: 64, marginBottom: 16, display: 'block', opacity: 0.5 }} />
              <Text style={{ fontSize: 14, color: '#64748b' }}>
                Click "Start" to preview your application
              </Text>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default PreviewPanel;
