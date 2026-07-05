import { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Editor, { type OnMount } from '@monaco-editor/react';
import type { editor as MonacoEditor } from 'monaco-editor';
import { FixedSizeList as List } from 'react-window';
import { Tabs, Spin, Alert, Empty, Dropdown, Button, Space } from 'antd';
import { FolderOutlined, FileOutlined, ReloadOutlined, CaretRightOutlined, CaretDownOutlined, CodeOutlined, DownOutlined } from '@ant-design/icons';
import { showError, showSuccess, showWarning, tryExecute } from '../utils/errorHandler';

interface FileTreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileTreeNode[];
  is_expanded?: boolean;
  language?: string;
}

interface FlatFileTreeNode extends FileTreeNode {
  depth: number;
  key: string;
}

interface FileReadResult {
  content: string;
  is_partial: boolean;
  offset: number;
  total_size: number;
}

interface FormatResult {
  success: boolean;
  formatted_files: string[];
}

interface CodeEditorProps {
  iterationId: string;
  refreshTrigger?: number;
}

const CodeEditor: React.FC<CodeEditorProps> = ({ iterationId, refreshTrigger }) => {
  const [fileTree, setFileTree] = useState<FileTreeNode | null>(null);
  const [openFiles, setOpenFiles] = useState<string[]>([]);
  const [activeFile, setActiveFile] = useState<string | null>(null);
  const [fileContents, setFileContents] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [formatting, setFormatting] = useState(false);
  const prevRefreshTriggerRef = useRef(0);

  // ===== Monaco viewState 持久化（切 tab 不丢光标/折叠/scroll） =====
  const editorRef = useRef<MonacoEditor.IStandaloneCodeEditor | null>(null);
  const viewStatesRef = useRef<Record<string, MonacoEditor.ICodeEditorViewState | null>>({});
  const monacoRef = useRef<typeof import('monaco-editor') | null>(null);

  // ===== 文件树虚拟化 =====
  const fileTreeContainerRef = useRef<HTMLDivElement>(null);
  const [treeHeight, setTreeHeight] = useState(600);
  const fileTreeListRef = useRef<List>(null);

  useEffect(() => {
    if (!fileTreeContainerRef.current) return;
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) setTreeHeight(entry.contentRect.height);
    });
    observer.observe(fileTreeContainerRef.current);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (iterationId) {
      loadFileTree();
    }
  }, [iterationId]);

  useEffect(() => {
    if (refreshTrigger !== undefined && refreshTrigger !== prevRefreshTriggerRef.current) {
      prevRefreshTriggerRef.current = refreshTrigger;
      console.log('[CodeEditor] Refresh trigger changed, reloading file tree...');
      loadFileTree();
      openFiles.forEach(filePath => loadFileContent(filePath));
    }
  }, [refreshTrigger, openFiles]);

  const flatFileTree = useMemo((): FlatFileTreeNode[] => {
    if (!fileTree) return [];
    
    const flatten = (node: FileTreeNode, depth = 0, result: FlatFileTreeNode[] = []): FlatFileTreeNode[] => {
      if (!node) return result;
      
      result.push({ ...node, depth, key: node.path });
      
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
      const tree = await invoke<FileTreeNode>('get_iteration_file_tree', { iterationId })
        .catch(() => invoke<FileTreeNode>('get_file_tree', { sessionId: iterationId }));
      setFileTree(tree);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const formatAllCode = async () => {
    setFormatting(true);
    const result = await tryExecute(async () => {
      return await invoke<FormatResult>('format_code', { iterationId, filePath: null })
        .catch(() => invoke<FormatResult>('format_code', { sessionId: iterationId, filePath: null }));
    }, 'Failed to format code');
    
    setFormatting(false);
    
    if (result && result.success) {
      showSuccess(`Formatted ${result.formatted_files.length} file(s)`);
      for (const filePath of openFiles) {
        await loadFileContent(filePath);
      }
    }
  };

  const formatActiveFile = async () => {
    if (!activeFile) return;
    
    setFormatting(true);
    const result = await tryExecute(async () => {
      return await invoke<FormatResult>('format_code', { iterationId, filePath: activeFile })
        .catch(() => invoke<FormatResult>('format_code', { sessionId: iterationId, filePath: activeFile }));
    }, 'Failed to format file');
    
    setFormatting(false);
    
    if (result && result.success) {
      showSuccess(`Formatted ${result.formatted_files.length} file(s)`);
      await loadFileContent(activeFile);
    }
  };

  const loadFileContent = async (filePath: string) => {
    const result = await tryExecute(async () => {
      const res = await invoke<string | FileReadResult>('read_iteration_file', { 
        iterationId, filePath, offset: null, limit: null
      }).catch(() => invoke<string | FileReadResult>('read_file_content', { 
        sessionId: iterationId, filePath, offset: null, limit: null
      }));
      
      let content: string;
      if (typeof res === 'object' && 'content' in res) {
        content = res.content;
        if (res.is_partial) {
          showWarning(`Large file loaded partially (${res.offset / 1024}KB - ${(res.offset + content.length) / 1024}KB of ${res.total_size / 1024}KB)`);
        }
      } else {
        content = res;
      }
      
      setFileContents(prev => ({ ...prev, [filePath]: content }));
      return content;
    }, 'Failed to load file content');
    
    return result;
  };

  const saveFileContent = async (filePath: string, content: string) => {
    const success = await tryExecute(async () => {
      await invoke('save_iteration_file', { iterationId, filePath, content })
        .catch(() => invoke('save_file_content', { sessionId: iterationId, filePath, content }));
      return true;
    }, 'Failed to save file');
    
    if (success) {
      showSuccess('File saved successfully');
    }
  };

  const handleFileSelect = useCallback(async (filePath: string) => {
    if (!openFiles.includes(filePath)) {
      setOpenFiles(prev => [...prev, filePath]);
      await loadFileContent(filePath);
    }
    setActiveFile(filePath);
  }, [openFiles]);

  const handleToggleFolder = useCallback((path: string) => {
    const toggleNode = (node: FileTreeNode): FileTreeNode => {
      if (node.path === path) {
        return { ...node, is_expanded: !node.is_expanded };
      }
      if (node.children) {
        return { ...node, children: node.children.map(toggleNode) };
      }
      return node;
    };
    setFileTree(prev => prev ? toggleNode(prev) : null);
  }, []);

  const handleCloseFile = (targetKey: string) => {
    const newOpenFiles = openFiles.filter(key => key !== targetKey);
    setOpenFiles(newOpenFiles);
    if (activeFile === targetKey) {
      setActiveFile(newOpenFiles[newOpenFiles.length - 1] || null);
    }
  };

  const handleEditorChange = (value: string | undefined) => {
    if (activeFile && value !== undefined) {
      setFileContents(prev => ({ ...prev, [activeFile]: value }));
    }
  };

  // 切 tab 前保存旧 tab 的 viewState
  const handleFileSelectWithViewState = useCallback(async (filePath: string) => {
    if (editorRef.current && activeFile) {
      viewStatesRef.current[activeFile] = editorRef.current.saveViewState();
    }
    await handleFileSelect(filePath);
    // 切换后恢复新 tab 的 viewState
    if (editorRef.current && viewStatesRef.current[filePath]) {
      editorRef.current.restoreViewState(viewStatesRef.current[filePath]!);
    }
  }, [activeFile, handleFileSelect]);

  // Editor 挂载：恢复 viewState，注册保存快捷键
  const handleEditorMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;
    if (activeFile && viewStatesRef.current[activeFile]) {
      editor.restoreViewState(viewStatesRef.current[activeFile]!);
    }
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      if (activeFile) {
        saveFileContent(activeFile, editor.getValue());
      }
    });
  };

  const getLanguageFromPath = (filePath: string): string => {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    const langMap: Record<string, string> = {
      'rs': 'rust', 'js': 'javascript', 'jsx': 'javascript', 'ts': 'typescript',
      'tsx': 'typescript', 'py': 'python', 'html': 'html', 'css': 'css',
      'json': 'json', 'md': 'markdown', 'toml': 'toml', 'yaml': 'yaml', 'yml': 'yaml',
    };
    return langMap[ext] || 'plaintext';
  };

  // react-window 兼容的 Row 组件（接收 data prop）
  const FileTreeRow = useCallback(({ index, style, data }: { index: number; style: React.CSSProperties; data: FlatFileTreeNode[] }) => {
    const node = data[index];
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
  }, [handleToggleFolder, handleFileSelect]);

  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '40px' }}>
        <Spin size="large" />
        <div style={{ marginTop: '16px', color: '#999' }}>Loading files...</div>
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
        action={<button onClick={loadFileTree}>Retry</button>}
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
              onChange={handleEditorChange}
              theme="vs-dark"
              options={{
                minimap: { enabled: true },
                fontSize: 14,
                lineNumbers: 'on',
                renderWhitespace: 'selection',
                scrollBeyondLastLine: false,
                automaticLayout: true,
              }}
              onMount={handleEditorMount}
            />
          ) : null}
        </div>
      </div>
    ),
  }));

  return (
    <div className="code-editor-container" style={{ display: 'flex', height: '100%' }}>
      <div style={{ width: '250px', borderRight: '1px solid var(--border-color)', background: 'var(--bg-container)', display: 'flex', flexDirection: 'column' }}>
        <div style={{ padding: '10px', borderBottom: '1px solid var(--border-color)' }}>
          <h3 style={{ color: 'var(--text-primary)', margin: 0, display: 'flex', alignItems: 'center', gap: '8px' }}>
            <FolderOutlined /> Files
            <Dropdown menu={{
              items: [
                { key: 'format-all', label: <span><CodeOutlined /> Format All Files</span>, onClick: formatAllCode },
                { key: 'format-active', label: <span><CodeOutlined /> Format Active File</span>, onClick: formatActiveFile, disabled: !activeFile },
              ]
            }}>
              <Button size="small" icon={<CodeOutlined />} loading={formatting} disabled={!openFiles.length}>
                Format <DownOutlined />
              </Button>
            </Dropdown>
            <button onClick={loadFileTree} style={{ float: 'right', border: 'none', background: 'none', color: 'var(--primary)', cursor: 'pointer' }}>
              <ReloadOutlined />
            </button>
          </h3>
        </div>
        <div ref={fileTreeContainerRef} style={{ flex: 1, overflow: 'hidden' }}>
          <List
            ref={fileTreeListRef}
            height={treeHeight}
            itemCount={flatFileTree.length}
            itemSize={26}
            width="100%"
            itemData={flatFileTree}
            overscanCount={8}
          >
            {FileTreeRow}
          </List>
        </div>
      </div>

      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', minHeight: 0 }}>
        <Tabs
          type="editable-card"
          activeKey={activeFile}
          onChange={handleFileSelectWithViewState}
          onEdit={(targetKey, action) => {
            if (action === 'remove' && typeof targetKey === 'string') {
              handleCloseFile(targetKey);
            }
          }}
          hideAdd
          items={tabItems}
          animated={false}
          style={{ height: '100%', display: 'flex', flexDirection: 'column' }}
          tabBarStyle={{ margin: 0, background: 'var(--bg-container)' }}
          className="code-editor-tabs"
        />
        
        {openFiles.length === 0 && (
          <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--text-secondary)' }}>
            <Empty description="Select a file from the tree to start editing" />
          </div>
        )}
      </div>
    </div>
  );
};

export default CodeEditor;
