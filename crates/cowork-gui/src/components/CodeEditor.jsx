import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Editor from '@monaco-editor/react';
import { Tabs, Spin, Alert, Empty, message } from 'antd';
import { FolderOutlined, FileOutlined, ReloadOutlined } from '@ant-design/icons';

const CodeEditor = ({ sessionId }) => {
  const [fileTree, setFileTree] = useState(null);
  const [openFiles, setOpenFiles] = useState([]);
  const [activeFile, setActiveFile] = useState(null);
  const [fileContents, setFileContents] = useState({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (sessionId) {
      loadFileTree();
    }
  }, [sessionId]);

  const loadFileTree = async () => {
    setLoading(true);
    setError(null);
    try {
      const tree = await invoke('get_file_tree', { sessionId });
      setFileTree(tree);
    } catch (err) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const loadFileContent = async (filePath) => {
    try {
      const content = await invoke('read_file_content', { sessionId, filePath });
      setFileContents(prev => ({ ...prev, [filePath]: content }));
      return content;
    } catch (err) {
      message.error(`Failed to load file: ${err}`);
      return null;
    }
  };

  const saveFileContent = async (filePath, content) => {
    try {
      await invoke('save_file_content', { sessionId, filePath, content });
      message.success('File saved successfully');
    } catch (err) {
      message.error(`Failed to save file: ${err}`);
    }
  };

  const handleFileSelect = async (filePath) => {
    if (!openFiles.includes(filePath)) {
      setOpenFiles(prev => [...prev, filePath]);
      await loadFileContent(filePath);
    }
    setActiveFile(filePath);
  };

  const handleToggleFolder = (path) => {
    const toggleNode = (node) => {
      if (node.path === path) {
        return { ...node, is_expanded: !node.is_expanded };
      }
      if (node.children) {
        return {
          ...node,
          children: node.children.map(toggleNode)
        };
      }
      return node;
    };
    setFileTree(toggleNode(fileTree));
  };

  const handleCloseFile = (targetKey) => {
    const newOpenFiles = openFiles.filter(key => key !== targetKey);
    setOpenFiles(newOpenFiles);
    if (activeFile === targetKey) {
      setActiveFile(newOpenFiles[newOpenFiles.length - 1] || null);
    }
  };

  const handleEditorChange = (value) => {
    if (activeFile) {
      setFileContents(prev => ({ ...prev, [activeFile]: value }));
    }
  };

  const getLanguageFromPath = (filePath) => {
    const ext = filePath.split('.').pop().toLowerCase();
    const langMap = {
      'rs': 'rust',
      'js': 'javascript',
      'jsx': 'javascript',
      'ts': 'typescript',
      'tsx': 'typescript',
      'py': 'python',
      'html': 'html',
      'css': 'css',
      'json': 'json',
      'md': 'markdown',
      'toml': 'toml',
      'yaml': 'yaml',
      'yml': 'yaml',
    };
    return langMap[ext] || 'plaintext';
  };

  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '40px' }}>
        <Spin size="large" tip="Loading files..." />
      </div>
    );
  }

  if (error) {
    return (
      <Alert
        message="Error loading files"
        description={error}
        type="error"
        showIcon
        action={
          <button onClick={loadFileTree}>Retry</button>
        }
      />
    );
  }

  const tabItems = openFiles.map(filePath => ({
    key: filePath,
    label: <span><FileOutlined /> {filePath.split('/').pop()}</span>,
    closable: true,
    children: (
      <div style={{ height: '100%' }}>
        <Editor
          height="100%"
          language={getLanguageFromPath(filePath)}
          value={fileContents[filePath] || ''}
          onChange={(value) => handleEditorChange(value)}
          theme="vs-dark"
          options={{
            minimap: { enabled: true },
            fontSize: 14,
            lineNumbers: 'on',
            renderWhitespace: 'selection',
            scrollBeyondLastLine: false,
            automaticLayout: true,
          }}
          saveViewState={true}
          onMount={(editor) => {
            editor.addCommand(
              0,
              () => {
                if (activeFile) {
                  saveFileContent(activeFile, editor.getValue());
                }
              },
              'save'
            );
          }}
        />
      </div>
    ),
  }));

  return (
    <div className="code-editor-container" style={{ display: 'flex', height: '100%' }}>
      {/* File Tree */}
      <div style={{ 
        width: '250px', 
        borderRight: '1px solid var(--border-color)', 
        overflow: 'auto',
        background: 'var(--bg-container)'
      }}>
        <div style={{ padding: '10px', borderBottom: '1px solid var(--border-color)' }}>
          <h3 style={{ color: 'var(--text-primary)', margin: 0, display: 'flex', alignItems: 'center', gap: '8px' }}>
            <FolderOutlined /> Files
            <button
              onClick={loadFileTree}
              style={{ float: 'right', border: 'none', background: 'none', color: 'var(--primary)', cursor: 'pointer' }}
            >
              <ReloadOutlined />
            </button>
          </h3>
        </div>
        {renderFileTree(fileTree, handleFileSelect, handleToggleFolder, 0)}
      </div>

      {/* Editor */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <Tabs
          type="editable-card"
          activeKey={activeFile}
          onChange={handleFileSelect}
          onEdit={(targetKey, action) => {
            if (action === 'remove') {
              handleCloseFile(targetKey);
            }
          }}
          hideAdd
          items={tabItems}
          style={{ height: '100%' }}
        />
      </div>
    </div>
  );
};

const renderFileTree = (node, onSelect, onToggle, depth) => {
  if (!node) return null;

  const paddingLeft = depth * 20;

  return (
    <div key={node.path}>
      <div
        style={{
          padding: '8px 10px',
          cursor: 'pointer',
          paddingLeft: `${paddingLeft + 10}px`,
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--text-primary)',
        }}
        onClick={() => {
          if (node.is_dir) {
            onToggle(node.path);
          } else {
            onSelect(node.path);
          }
        }}
      >
        {node.is_dir ? (
          <FolderOutlined style={{ color: 'var(--primary)' }} />
        ) : (
          <FileOutlined style={{ color: 'var(--text-secondary)' }} />
        )}
        <span>{node.name}</span>
      </div>
      {node.children && node.is_expanded && (
        <div>
          {node.children.map(child => renderFileTree(child, onSelect, onToggle, depth + 1))}
        </div>
      )}
    </div>
  );
};

export default CodeEditor;