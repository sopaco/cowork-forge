import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Spin, Alert, Button, Space } from 'antd';
import { ReloadOutlined, FullscreenOutlined, StopOutlined } from '@ant-design/icons';

const PreviewPanel = ({ sessionId }) => {
  const [previewUrl, setPreviewUrl] = useState(null);
  const [isRunning, setIsRunning] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const startPreview = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke('start_preview', { sessionId });
      setPreviewUrl(result.url);
      setIsRunning(true);
    } catch (err) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const stopPreview = async () => {
    setLoading(true);
    try {
      await invoke('stop_preview', { sessionId });
      setPreviewUrl(null);
      setIsRunning(false);
    } catch (err) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const refreshPreview = () => {
    if (previewUrl) {
      const iframe = document.querySelector('.preview-frame');
      if (iframe) {
        iframe.src = iframe.src;
      }
    }
  };

  return (
    <div className="preview-panel" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '10px', borderBottom: '1px solid #303030', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h3>🌐 Preview</h3>
        <Space>
          {!isRunning ? (
            <Button
              type="primary"
              icon={<span>▶</span>}
              onClick={startPreview}
              loading={loading}
            >
              Start Preview
            </Button>
          ) : (
            <>
              <Button icon={<ReloadOutlined />} onClick={refreshPreview}>
                Refresh
              </Button>
              <Button icon={<FullscreenOutlined />} onClick={() => alert('Fullscreen feature coming soon')}>
                Fullscreen
              </Button>
              <Button danger icon={<StopOutlined />} onClick={stopPreview} loading={loading}>
                Stop
              </Button>
            </>
          )}
        </Space>
      </div>

      {error && (
        <Alert
          message="Preview Error"
          description={error}
          type="error"
          showIcon
          closable
          style={{ margin: '10px' }}
        />
      )}

      {isRunning && previewUrl ? (
        <iframe
          src={previewUrl}
          className="preview-frame"
          style={{
            flex: 1,
            width: '100%',
            border: 'none',
            background: '#fff',
          }}
          title="Preview"
        />
      ) : (
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#888', flexDirection: 'column', gap: '10px' }}>
          {loading ? (
            <>
              <Spin />
              <div>Starting preview...</div>
            </>
          ) : 'Click "Start Preview" to preview your application'}
        </div>
      )}
    </div>
  );
};

export default PreviewPanel;