import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';
import JsonView from 'react-json-view';
import { Tabs, Spin, Alert, Empty, Button } from 'antd';
import { FileTextOutlined, ProjectOutlined, DatabaseOutlined, BuildOutlined, CheckCircleOutlined } from '@ant-design/icons';
import 'highlight.js/styles/atom-one-dark.css';

const ArtifactsViewer = ({ sessionId }) => {
  const [artifacts, setArtifacts] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [activeTab, setActiveTab] = useState('idea');

  useEffect(() => {
    if (sessionId) {
      loadArtifacts();
    }
  }, [sessionId]);

  const loadArtifacts = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke('get_session_artifacts', { sessionId });
      setArtifacts(data);
    } catch (err) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '40px' }}>
        <Spin size="large" tip="Loading artifacts..." />
      </div>
    );
  }

  if (error) {
    return (
      <Alert
        message="Error loading artifacts"
        description={error}
        type="error"
        showIcon
        action={
          <Button size="small" onClick={loadArtifacts}>Retry</Button>
        }
      />
    );
  }

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
        <div className="artifact-content markdown-content">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            rehypePlugins={[rehypeHighlight, rehypeRaw]}
          >
            {artifacts.idea}
          </ReactMarkdown>
        </div>
      ),
    });
  }

  if (artifacts.requirements) {
    items.push({
      key: 'requirements',
      label: <span><ProjectOutlined /> Requirements</span>,
      children: (
        <div className="artifact-content">
          <h3>Requirements ({artifacts.requirements.requirements.length})</h3>
          <JsonView
            src={artifacts.requirements}
            theme="dark"
            displayObjectSize={false}
            enableClipboard={false}
            indentWidth={2}
            collapsed={false}
            quotesOnKeys={false}
            sortKeys={false}
          />
        </div>
      ),
    });
  }

  if (artifacts.features) {
    items.push({
      key: 'features',
      label: <span><DatabaseOutlined /> Features</span>,
      children: (
        <div className="artifact-content">
          <h3>Features ({artifacts.features.features.length})</h3>
          <JsonView
            src={artifacts.features}
            theme="dark"
            displayObjectSize={false}
            enableClipboard={false}
            indentWidth={2}
            collapsed={false}
            quotesOnKeys={false}
            sortKeys={false}
          />
        </div>
      ),
    });
  }

  if (artifacts.design) {
    items.push({
      key: 'design',
      label: <span><BuildOutlined /> Design</span>,
      children: (
        <div className="artifact-content">
          <h3>Design Specification</h3>
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
      ),
    });
  }

  if (artifacts.plan) {
    items.push({
      key: 'plan',
      label: <span><CheckCircleOutlined /> Plan</span>,
      children: (
        <div className="artifact-content">
          <h3>Implementation Plan</h3>
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
      ),
    });
  }

  if (artifacts.code_files && artifacts.code_files.length > 0) {
    items.push({
      key: 'code',
      label: <span><FileTextOutlined /> Code Files</span>,
      children: (
        <div className="artifact-content">
          <h3>Code Files ({artifacts.code_files.length})</h3>
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
      ),
    });
  }

  if (artifacts.delivery_report) {
    items.push({
      key: 'report',
      label: <span><CheckCircleOutlined /> Report</span>,
      children: (
        <div className="artifact-content markdown-content">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            rehypePlugins={[rehypeHighlight, rehypeRaw]}
          >
            {artifacts.delivery_report}
          </ReactMarkdown>
        </div>
      ),
    });
  }

  return (
    <div className="artifacts-viewer" style={{ height: '100%' }}>
      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
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