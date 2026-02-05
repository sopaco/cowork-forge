import { useState, useEffect, useMemo, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Editor from '@monaco-editor/react';
import { Tabs, Spin, Alert, Empty, Dropdown, Button, Space } from 'antd';
import { FolderOutlined, FileOutlined, ReloadOutlined, CaretRightOutlined, CaretDownOutlined, CodeOutlined, DownOutlined } from '@ant-design/icons';
import { showError, showSuccess, tryExecute } from '../utils/errorHandler.jsx';

// Try to import react-window if available
let FixedSizeList = null;
try {
  FixedSizeList = require('react-window').FixedSizeList;
} catch (e) {
  console.warn('react-window not installed. Install it with: npm install react-window');
}

const CodeEditor = ({ iterationId }) => {
  const [fileTree, setFileTree] = useState(null);
  const [openFiles, setOpenFiles] = useState([]);
  const [activeFile, setActiveFile] = useState(null);
  const [fileContents, setFileContents] = useState({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [formatting, setFormatting] = useState(false);

  useEffect(() => {
    if (iterationId) {
      loadFileTree();
    }
  }, [iterationId]);

  // Flatten file tree for virtual scrolling
  const flatFileTree = useMemo(() => {
    if (!fileTree) return [];
    
    const flatten = (node, depth = 0, result = []) => {
      if (!node) return result;
      
      result.push({
        ...node,
        depth,
        key: node.path,
      });
      
      if (node.children && node.is_expanded) {
        node.children.forEach(child => flatten(child, depth + 1, result));
      }
      
      return result;
    };
    
    return flatten(fileTree);
  }, [fileTree]);

  const loadFileTree = async () => {
    setLoading(true);
    setError(null);
    try {
      // Try new V2 API first, fall back to legacy API
      const tree = await invoke('get_iteration_file_tree', { iterationId })
        .catch(() => invoke('get_file_tree', { sessionId: iterationId }));
      setFileTree(tree);
    } catch (err) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const formatAllCode = async () => {
    setFormatting(true);
    const result = await tryExecute(async () => {
      return await invoke('format_code', { iterationId, filePath: null })
        .catch(() => invoke('format_code', { sessionId: iterationId, filePath: null }));
    }, 'Failed to format code');
    
    setFormatting(false);
    
    if (result && result.success) {
      showSuccess(`Formatted ${result.formatted_files.length} file(s)`);
      // Reload all open files to show formatted content
      for (const filePath of openFiles) {
        await loadFileContent(filePath);
      }
    }
  };

  const formatActiveFile = async () => {
    if (!activeFile) return;
    
    setFormatting(true);
    const result = await tryExecute(async () => {
      return await invoke('format_code', { iterationId, filePath: activeFile })
        .catch(() => invoke('format_code', { sessionId: iterationId, filePath: activeFile }));
    }, 'Failed to format file');
    
    setFormatting(false);
    
    if (result && result.success) {
      showSuccess(`Formatted ${result.formatted_files.length} file(s)`);
      // Reload active file to show formatted content
      await loadFileContent(activeFile);
    }
  };

  const loadFileContent = async (filePath) => {
    const result = await tryExecute(async () => {
      const result = await invoke('read_iteration_file', { 
        iterationId, 
        filePath,
        offset: null,
        limit: null
      }).catch(() => invoke('read_file_content', { 
        sessionId: iterationId, 
        filePath,
        offset: null,
        limit: null
      }));
      
      // Handle both old format (string) and new format (FileReadResult)
      let content = result;
      if (typeof result === 'object' && result.content !== undefined) {
        content = result.content;
        if (result.is_partial) {
          showWarning(`Large file loaded partially (${result.offset / 1024}KB - ${(result.offset + content.length) / 1024}KB of ${result.total_size / 1024}KB)`);
        }
      }
      
      setFileContents(prev => ({ ...prev, [filePath]: content }));
      return content;
    }, 'Failed to load file content');
    
    return result;
  };

  const saveFileContent = async (filePath, content) => {
    const success = await tryExecute(async () => {
      await invoke('save_iteration_file', { iterationId, filePath, content })
        .catch(() => invoke('save_file_content', { sessionId: iterationId, filePath, content }));
      return true;
    }, 'Failed to save file');
    
    if (success) {
      showSuccess('File saved successfully');
    }
  };

  const handleFileSelect = useCallback(async (filePath) => {
    if (!openFiles.includes(filePath)) {
      setOpenFiles(prev => [...prev, filePath]);
      await loadFileContent(filePath);
    }
    setActiveFile(filePath);
  }, [openFiles]);

  const handleToggleFolder = useCallback((path) => {
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
  }, [fileTree]);

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

  // Render single file tree row for virtual scrolling
  const renderFileTreeRow = useCallback(({ index, style }) => {
    const node = flatFileTree[index];
    if (!node) return null;

    return (
      <div
        style={{
          ...style,
          paddingLeft: `${node.depth * 20 + 10}px`,
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          cursor: 'pointer',
          color: 'var(--text-primary)',
        }}
        onClick={() => {
          if (node.is_dir) {
            handleToggleFolder(node.path);
          } else {
            handleFileSelect(node.path);
          }
        }}
      >
        {node.is_dir ? (
          <>
            {node.is_expanded ? (
              <CaretDownOutlined style={{ fontSize: '12px', color: 'var(--text-secondary)' }} />
            ) : (
              <CaretRightOutlined style={{ fontSize: '12px', color: 'var(--text-secondary)' }} />
            )}
            <FolderOutlined style={{ color: 'var(--primary)' }} />
          </>
        ) : (
          <>
            <span style={{ width: '12px' }} />
            <FileOutlined style={{ color: 'var(--text-secondary)' }} />
          </>
        )}
        <span style={{ fontSize: '13px' }}>{node.name}</span>
      </div>
    );
  }, [flatFileTree, handleToggleFolder, handleFileSelect]);

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
      <div style={{ height: '100%', display: 'flex', flexDirection: 'column', position: 'relative' }}>
        <div style={{ flex: 1, overflow: 'hidden' }}>
          {activeFile === filePath ? (
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
          ) : null}
        </div>
      </div>
    ),
  }));

  return (
    <div className="code-editor-container" style={{ display: 'flex', height: '100%' }}>
      {/* File Tree */}
      <div style={{ 
        width: '250px', 
        borderRight: '1px solid var(--border-color)', 
        background: 'var(--bg-container)',
        display: 'flex',
        flexDirection: 'column'
      }}>
        <div style={{ padding: '10px', borderBottom: '1px solid var(--border-color)' }}>
          <h3 style={{ color: 'var(--text-primary)', margin: 0, display: 'flex', alignItems: 'center', gap: '8px' }}>
            <FolderOutlined /> Files
            <Dropdown menu={{
              items: [
                {
                  key: 'format-all',
                  label: <span><CodeOutlined /> Format All Files</span>,
                  onClick: formatAllCode,
                },
                {
                  key: 'format-active',
                  label: <span><CodeOutlined /> Format Active File</span>,
                  onClick: formatActiveFile,
                  disabled: !activeFile,
                },
              ]
            }}>
              <Button
                size="small"
                icon={<CodeOutlined />}
                loading={formatting}
                disabled={!openFiles.length}
              >
                Format <DownOutlined />
              </Button>
            </Dropdown>
            <button
              onClick={loadFileTree}
              style={{ float: 'right', border: 'none', background: 'none', color: 'var(--primary)', cursor: 'pointer' }}
            >
              <ReloadOutlined />
            </button>
          </h3>
        </div>
        {/* File Tree Content with Virtual Scrolling */}
        <div style={{ flex: 1, overflow: 'hidden' }}>
          {FixedSizeList && flatFileTree.length > 50 ? (
            // Use virtual scrolling for large file trees
            <FixedSizeList
              height={600}
              itemCount={flatFileTree.length}
              itemSize={32}
              width="100%"
            >
              {renderFileTreeRow}
            </FixedSizeList>
          ) : (
            // Fallback to regular rendering for small file trees
            <div style={{ overflow: 'auto', height: '100%' }}>
              {flatFileTree.map((node) => renderFileTreeRow({ index: flatFileTree.indexOf(node), style: {} }))}
            </div>
          )}
        </div>
      </div>

      {/* Editor */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', minHeight: 0 }}>
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
          animated={false}
          style={{ 
            height: '100%',
            display: 'flex',
            flexDirection: 'column'
          }}
          tabBarStyle={{ 
            margin: 0, 
            background: 'var(--bg-container)'
          }}
          cla