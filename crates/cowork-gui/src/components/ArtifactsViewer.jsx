import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';
import JsonView from 'react-json-view';
import { Tabs, Spin, Alert, Empty, Button, Space, Tooltip, message } from 'antd';
import { FileTextOutlined, ProjectOutlined, DatabaseOutlined, BuildOutlined, CheckCircleOutlined, FileMarkdownOutlined, FolderOpenOutlined } from '@ant-design/icons';
import 'highlight.js/styles/atom-one-dark.css';

const ArtifactsViewer = ({ iterationId, activeTab: externalActiveTab, onTabChange }) => {
  const [artifacts, setArtifacts] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [activeTab, setActiveTab] = useState('idea');
  const [viewModes, setViewModes] = useState({});

  // Sync with external active tab when provided
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

  const loadArtifacts = async () => {
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

  const requirementsToMarkdown = (req) => {
    if (!req || !req.requirements) return '';
    let md = `# Requirements\n\n`;
    req.requirements.forEach((r, i) => {
      md += `## ${i + 1}. ${r.title}\n\n`;
      md += `**ID:** ${r.id}\n\n`;
      md += `**Description:** ${r.description}\n\n`;
      if (r.acceptance_criteria && r.acceptance_criteria.length > 0) {
        md += `### Acceptance Criteria\n\n`;
        r.acceptance_criteria.forEach((ac, j) => {
          md += `- ${ac}\n`;
        });
        md += `\n`;
      }
      if (r.notes) {
        md += `**Notes:** ${r.notes}\n\n`;
      }
      md += `---\n\n`;
    });
    return md;
  };

  const featuresToMarkdown = (feat) => {
    if (!feat || !feat.features) return '';
    let md = `# Features\n\n`;
    feat.features.forEach((f, i) => {
      const title = f.title || f.name || `Feature ${i + 1}`;
      md += `## ${i + 1}. ${title}\n\n`;
      if (f.id) {
        md += `**ID:** ${f.id}\n\n`;
      }
      if (f.description) {
        md += `**Description:** ${f.description}\n\n`;
      }
      if (f.scenarios && f.scenarios.length > 0) {
        md += `### Scenarios\n\n`;
        f.scenarios.forEach((sc, j) => {
          const scTitle = sc.title || `Scenario ${j + 1}`;
          md += `#### ${scTitle}\n\n`;
          if (sc.given) {
            md += `**Given:** ${sc.given}\n\n`;
          }
          if (sc.when) {
            md += `**When:** ${sc.when}\n\n`;
          }
          if (sc.then) {
            md += `**Then:** ${sc.then}\n\n`;
          }
        });
      }
      if (f.notes) {
        md += `**Notes:** ${f.notes}\n\n`;
      }
      md += `---\n\n`;
    });
    return md;
  };

  const designToMarkdown = (design) => {
    if (!design) return '';
    let md = `# Design Specification\n\n`;
    if (design.overview) {
      md += `## Overview\n\n`;
      if (typeof design.overview === 'string') {
        md += `${design.overview}\n\n`;
      } else if (typeof design.overview === 'object') {
        md += `${JSON.stringify(design.overview, null, 2)}\n\n`;
      }
    }
    if (design.architecture) {
      md += `## Architecture\n\n`;
      if (typeof design.architecture === 'string') {
        md += `${design.architecture}\n\n`;
      } else if (typeof design.architecture === 'object') {
        // Handle object format
        if (design.architecture.type) {
          md += `**Type:** ${design.architecture.type}\n\n`;
        }
        if (design.architecture.description) {
          md += `**Description:** ${design.architecture.description}\n\n`;
        }
        if (design.architecture.components && Array.isArray(design.architecture.components)) {
          md += `**Components:**\n\n`;
          design.architecture.components.forEach((comp, j) => {
            md += `- ${comp.name || comp.type}: ${comp.description || ''}\n`;
          });
          md += `\n`;
        }
        // If still need to show full object
        md += `\n\`\`\`\n${JSON.stringify(design.architecture, null, 2)}\n\`\`\`\n\n`;
      }
    }
    if (design.components && design.components.length > 0) {
      md += `## Components\n\n`;
      design.components.forEach((comp, i) => {
        const compName = comp.name || comp.type || `Component ${i + 1}`;
        md += `### ${i + 1}. ${compName}\n\n`;
        if (comp.description) {
          md += `**Description:** ${comp.description}\n\n`;
        }
        if (comp.responsibilities && comp.responsibilities.length > 0) {
          md += `**Responsibilities:**\n\n`;
          comp.responsibilities.forEach((resp) => {
            md += `- ${resp}\n`;
          });
          md += `\n`;
        }
      });
    }
    if (design.technology_stack) {
      md += `## Technology Stack\n\n`;
      if (typeof design.technology_stack === 'string') {
        md += `${design.technology_stack}\n\n`;
      } else if (typeof design.technology_stack === 'object') {
        // Handle object format
        if (design.technology_stack.frontend) {
          md += `**Frontend:** ${design.technology_stack.frontend}\n\n`;
        }
        if (design.technology_stack.backend) {
          md += `**Backend:** ${design.technology_stack.backend}\n\n`;
        }
        if (design.technology_stack.database) {
          md += `**Database:** ${design.technology_stack.database}\n\n`;
        }
        if (design.technology_stack.other) {
          md += `**Other:** ${design.technology_stack.other}\n\n`;
        }
        // Show all fields
        Object.entries(design.technology_stack).forEach(([key, value]) => {
          if (typeof value === 'string') {
            md += `**${key.charAt(0).toUpperCase() + key.slice(1)}:** ${value}\n\n`;
          }
        });
      }
    }
    return md;
  };

  const planToMarkdown = (plan) => {
    if (!plan || !plan.tasks) return '';
    let md = `# Implementation Plan\n\n`;
    if (plan.overview) {
      md += `## Overview\n\n${plan.overview}\n\n`;
    }
    if (plan.tasks && plan.tasks.length > 0) {
      md += `## Tasks\n\n`;
      plan.tasks.forEach((task, i) => {
        md += `### ${i + 1}. ${task.title}\n\n`;
        md += `**ID:** ${task.id}\n\n`;
        md += `**Description:** ${task.description}\n\n`;
        if (task.estimated_hours) {
          md += `**Estimated Hours:** ${task.estimated_hours}\n\n`;
        }
        if (task.dependencies && task.dependencies.length > 0) {
          md += `**Dependencies:** ${task.dependencies.join(', ')}\n\n`;
        }
        md += `---\n\n`;
      });
    }
    return md;
  };

  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '40px' }}>
        <Spin size="large" />
        <div style={{ marginTop: '16px', color: '#999' }}>Loading artifacts...</div>
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
    const reqViewMode = viewModes['requirements'] || 'doc';
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
              Requirements ({artifacts.requirements.requirements.length})
            </span>
            <Space>
              <Button 
                size="small" 
                type={reqViewMode === 'doc' ? 'primary' : 'default'}
                icon={<FileMarkdownOutlined />}
                onClick={() => toggleViewMode('requirements')}
              >
                Doc
              </Button>
              <Button 
                size="small" 
                type={reqViewMode === 'json' ? 'primary' : 'default'}
                onClick={() => toggleViewMode('requirements')}
              >
                JSON
              </Button>
            </Space>
          </div>
          <div className="artifact-content" style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
            {reqViewMode === 'doc' ? (
              <div className="markdown-content">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  rehypePlugins={[rehypeHighlight, rehypeRaw]}
                >
                  {requirementsToMarkdown(artifacts.requirements)}
                </ReactMarkdown>
              </div>
            ) : (
              <div style={{ overflow: 'auto', maxHeight: '100%' }}>
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
            )}
          </div>
        </div>
      ),
    });
  }

  if (artifacts.features) {
    const featViewMode = viewModes['features'] || 'doc';
    items.push({
      key: 'features',
      label: <span><DatabaseOutlined /> Features</span>,
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
              Features ({artifacts.features.features.length})
            </span>
            <Space>
              <Button 
                size="small" 
                type={featViewMode === 'doc' ? 'primary' : 'default'}
                icon={<FileMarkdownOutlined />}
                onClick={() => toggleViewMode('features')}
              >
                Doc
              </Button>
              <Button 
                size="small" 
                type={featViewMode === 'json' ? 'primary' : 'default'}
                onClick={() => toggleViewMode('features')}
              >
                JSON
              </Button>
            </Space>
          </div>
          <div className="artifact-content" style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
            {featViewMode === 'doc' ? (
              <div className="markdown-content">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  rehypePlugins={[rehypeHighlight, rehypeRaw]}
                >
                  {featuresToMarkdown(artifacts.features)}
                </ReactMarkdown>
              </div>
            ) : (
              <div style={{ overflow: 'auto', maxHeight: '100%' }}>
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
            )}
          </div>
        </div>
      ),
    });
  }

  if (artifacts.design_raw || artifacts.design) {
    const designViewMode = viewModes['design'] || 'doc';
    const designContent = artifacts.design_raw || designToMarkdown(artifacts.design);
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
              {!artifacts.design_raw && (
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
    const planContent = artifacts.plan_raw || planToMarkdown(artifacts.plan);
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
              {!artifacts.plan_raw && (
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
    <div className="artifacts-viewer" style={{ height: '100%' }}>
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