import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Layout, Menu, Button, Spin, Empty, Modal, Dropdown, message } from 'antd';
import { 
  FolderOutlined, 
  FileTextOutlined, 
  CodeOutlined, 
  EyeOutlined, 
  PlayCircleOutlined,
  EditOutlined,
  RollbackOutlined,
  DownOutlined,
  MessageOutlined,
  SettingOutlined
} from '@ant-design/icons';
import ArtifactsViewer from './components/ArtifactsViewer';
import CodeEditor from './components/CodeEditor';
import PreviewPanel from './components/PreviewPanel';
import RunnerPanel from './components/RunnerPanel';

const { Sider, Content, Header, Footer } = Layout;

function App() {
  const [sessions, setSessions] = useState([]);
  const [currentSession, setCurrentSession] = useState(null);
  const [projectIdea, setProjectIdea] = useState('');
  const [userInput, setUserInput] = useState('');
  const [inputRequest, setInputRequest] = useState(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [activeView, setActiveView] = useState('chat'); // chat, artifacts, code, preview, run
  const [messages, setMessages] = useState([]);
  const listenersRegistered = useRef(false);
  const messagesEndRef = useRef(null);
  const messagesContainerRef = useRef(null);
  const [isUserScrolled, setIsUserScrolled] = useState(false);
  const [loadingSessions, setLoadingSessions] = useState(true);
  const [showInactiveSessions, setShowInactiveSessions] = useState(true);
  const initialLoadRef = useRef(true);

  const loadSessions = async () => {
    const isInitialLoad = initialLoadRef.current;
    
    // 只在首次加载时显示loading状态
    if (isInitialLoad) {
      setLoadingSessions(true);
    }
    
    try {
      console.log('[App] Loading sessions...');
      const sessionsData = await invoke('get_sessions');
      console.log('[App] Loaded sessions:', sessionsData, 'Length:', Array.isArray(sessionsData) ? sessionsData.length : 'Not an array');
      setSessions(Array.isArray(sessionsData) ? sessionsData : []);
    } catch (error) {
      console.error('[App] Failed to load sessions:', error);
      console.error('[App] Error details:', JSON.stringify(error));
      // If project not initialized, show empty sessions
      setSessions([]);
    } finally {
      // 只在首次加载时关闭loading状态
      if (isInitialLoad) {
        setLoadingSessions(false);
        initialLoadRef.current = false;
      }
    }
  };

  useEffect(() => {
    console.log('[App] Component mounted, loading sessions...');
    loadSessions();
  }, []);

  // Load session artifacts when session changes
  useEffect(() => {
    if (currentSession) {
      console.log('[App] Session changed to:', currentSession);
      // Clear messages when switching to a different session
      // Note: Historical messages are not persisted, only real-time streaming is supported
      setMessages([]);
    }
  }, [currentSession]);

  useEffect(() => {
    if (listenersRegistered.current) {
      return;
    }
    listenersRegistered.current = true;

    const cleanupFunctions = [];

    listen('agent_event', (event) => {
      const { content, is_thinking } = event.payload;
      setIsProcessing(is_thinking || content.trim().length > 0);
      if (!is_thinking && content) {
        setMessages(prev => {
          const lastMsg = prev[prev.length - 1];
          if (lastMsg && lastMsg.type === 'agent' && lastMsg.isStreaming) {
            return [
              ...prev.slice(0, -1),
              { ...lastMsg, content: lastMsg.content + content, isStreaming: true }
            ];
          } else {
            return [
              ...prev,
              { type: 'agent', content, isStreaming: true, timestamp: new Date().toISOString() }
            ];
          }
        });
      }
    }).then(unlisten => cleanupFunctions.push(unlisten));

    listen('input_request', (event) => {
      const [requestId, prompt, options] = event.payload;
      setInputRequest({ requestId, prompt, options });
      setUserInput('');
    }).then(unlisten => cleanupFunctions.push(unlisten));

    listen('session_completed', (event) => {
      setIsProcessing(false);
      loadSessions();
    }).then(unlisten => cleanupFunctions.push(unlisten));

    listen('session_failed', (event) => {
      setIsProcessing(false);
      loadSessions();
    }).then(unlisten => cleanupFunctions.push(unlisten));

    return () => {
      cleanupFunctions.forEach(unlisten => {
        try { unlisten(); } catch (e) {}
      });
      listenersRegistered.current = false;
    };
  }, []);

  // Auto-scroll to bottom when switching to chat view
  useEffect(() => {
    if (activeView === 'chat' && messagesContainerRef.current) {
      messagesContainerRef.current.scrollTop = messagesContainerRef.current.scrollHeight;
    }
  }, [activeView]);

  // Auto-scroll to bottom when new messages arrive (if user hasn't scrolled up)
  useEffect(() => {
    if (!isUserScrolled && messagesContainerRef.current) {
      const container = messagesContainerRef.current;
      const isNearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 100;
      
      if (isNearBottom) {
        container.scrollTop = container.scrollHeight;
      }
    }
  }, [messages, isUserScrolled]);

  const handleScroll = (e) => {
    const container = e.target;
    const isNearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 50;
    setIsUserScrolled(!isNearBottom);
  };

  const handleCreateProject = async () => {
    if (!projectIdea.trim()) return;
    setIsProcessing(true);
    try {
      const sessionId = await invoke('create_project', { idea: projectIdea });
      setMessages(prev => [...prev, { type: 'user', content: projectIdea, timestamp: new Date().toISOString() }]);
      setProjectIdea('');
      setCurrentSession(sessionId);
      setTimeout(() => loadSessions(), 500);
    } catch (error) {
      alert('Failed to create project: ' + error);
      setIsProcessing(false);
    }
  };

  const handleSendUserMessage = async () => {
    if (!userInput.trim()) return;
    setMessages(prev => [...prev, { type: 'user', content: userInput, timestamp: new Date().toISOString() }]);
    if (inputRequest) {
      await invoke('submit_input_response', { requestId: inputRequest.requestId, response: userInput, responseType: 'text' });
      setInputRequest(null);
    }
    setUserInput('');
  };

  const handleSelectOption = async (option) => {
    if (!inputRequest) return;
    setMessages(prev => [...prev, { type: 'user', content: option.label, timestamp: new Date().toISOString() }]);
    await invoke('submit_input_response', { requestId: inputRequest.requestId, response: option.id, responseType: 'selection' });
    setInputRequest(null);
    setUserInput('');
  };

  const handleResumeSession = async (sessionId) => {
    const baseSession = sessions.find(s => s.id === sessionId);
    Modal.confirm({
      title: 'Resume Session',
      content: (
        <div>
          <p>You are about to resume from session:</p>
          <p style={{ fontStyle: 'italic', color: '#1890ff' }}>{baseSession?.description}</p>
          <p>This will <strong>create a new session</strong> that continues from this checkpoint.</p>
          <p>The new session will:</p>
          <ul style={{ marginLeft: '20px' }}>
            <li>Inherit all artifacts from this session</li>
            <li>Continue development from the last successful stage</li>
            <li>Generate new code and updates</li>
          </ul>
          <p>Do you want to continue?</p>
        </div>
      ),
      okText: 'Yes, Resume',
      cancelText: 'Cancel',
      onOk: async () => {
        setIsProcessing(true);
        try {
          console.log('[App] Resuming from session:', sessionId);
          const newSessionId = await invoke('resume_project', { baseSessionId: sessionId });
          console.log('[App] New session created:', newSessionId);
          setCurrentSession(newSessionId);
          setMessages([]);
          setActiveView('chat');
          // Add a system message explaining what happened
          setMessages([{ 
            type: 'system', 
            content: `Resumed from session ${sessionId.substring(0, 12)}... Creating new continuation...`, 
            timestamp: new Date().toISOString() 
          }]);
          setTimeout(() => loadSessions(), 500);
        } catch (error) {
          console.error('[App] Failed to resume session:', error);
          alert('Failed to resume session: ' + error);
          setIsProcessing(false);
        }
      }
    });
  };

  const handleModifyProject = async () => {
    const latestCompleted = sessions.find(s => s.status === 'Completed');
    if (!latestCompleted) {
      message.warning('No completed session found. Please complete a project first.');
      return;
    }
    
    let modifyIdea = '';
    Modal.confirm({
      title: 'Modify Project',
      width: 600,
      content: (
        <div>
          <p>Base Session:</p>
          <p style={{ fontStyle: 'italic', color: '#1890ff', marginBottom: '15px' }}>
            {latestCompleted.description}
          </p>
          <p>Changes:</p>
          <input
            type="text"
            placeholder="Describe what you want to change..."
            onChange={(e) => modifyIdea = e.target.value}
            autoFocus
            style={{ width: '100%', padding: '8px', marginBottom: '15px', background: '#1e1e1e', border: '1px solid #303030', color: '#fff', borderRadius: '4px' }}
          />
          <p style={{ fontSize: '12px', color: '#888' }}>
            This will create a new session with Type: Modify, based on {latestCompleted.id.substring(0, 12)}...
          </p>
        </div>
      ),
      okText: 'Modify',
      cancelText: 'Cancel',
      onOk: async () => {
        if (!modifyIdea.trim()) {
          message.warning('Please describe the changes you want to make.');
          return;
        }
        setIsProcessing(true);
        try {
          const newSessionId = await invoke('modify_project', { 
            baseSessionId: latestCompleted.id,
            idea: modifyIdea 
          });
          setCurrentSession(newSessionId);
          setMessages([]);
          setActiveView('chat');
          setMessages([{ 
            type: 'system', 
            content: `Modifying project based on session ${latestCompleted.id.substring(0, 12)}...`, 
            timestamp: new Date().toISOString() 
          }]);
          setTimeout(() => loadSessions(), 500);
        } catch (error) {
          message.error('Failed to modify project: ' + error);
          setIsProcessing(false);
        }
      }
    });
  };

  const handleRevertProject = async () => {
    const latestCompleted = sessions.find(s => s.status === 'Completed');
    if (!latestCompleted) {
      message.warning('No completed session found. Please complete a project first.');
      return;
    }
    
    let selectedStage = 'prd';
    Modal.confirm({
      title: 'Revert Project',
      width: 600,
      content: (
        <div>
          <p>Base Session:</p>
          <p style={{ fontStyle: 'italic', color: '#1890ff', marginBottom: '15px' }}>
            {latestCompleted.description}
          </p>
          <p>Revert to stage:</p>
          <select
            defaultValue="prd"
            onChange={(e) => selectedStage = e.target.value}
            style={{ width: '100%', padding: '8px', marginBottom: '15px', background: '#1e1e1e', border: '1px solid #303030', color: '#fff', borderRadius: '4px' }}
          >
            <option value="prd">prd (Requirements)</option>
            <option value="design">design (Design Specification)</option>
            <option value="plan">plan (Implementation Plan)</option>
            <option value="coding">coding (Code Generation)</option>
            <option value="check">check (Quality Check)</option>
            <option value="delivery">delivery (Final Delivery)</option>
            <option value="auto">auto (Use current stage)</option>
          </select>
          <p style={{ fontSize: '12px', color: '#888' }}>
            This will create a new session with Type: Revert, based on {latestCompleted.id.substring(0, 12)}...
          </p>
        </div>
      ),
      okText: 'Revert',
      cancelText: 'Cancel',
      onOk: async () => {
        setIsProcessing(true);
        try {
          const newSessionId = await invoke('revert_project', { 
            baseSessionId: latestCompleted.id,
            fromStage: selectedStage 
          });
          setCurrentSession(newSessionId);
          setMessages([]);
          setActiveView('chat');
          setMessages([{ 
            type: 'system', 
            content: `Reverting project from ${selectedStage} stage, based on session ${latestCompleted.id.substring(0, 12)}...`, 
            timestamp: new Date().toISOString() 
          }]);
          setTimeout(() => loadSessions(), 500);
        } catch (error) {
          message.error('Failed to revert project: ' + error);
          setIsProcessing(false);
        }
      }
    });
  };

  const handleModifySession = async (sessionId) => {
    const session = sessions.find(s => s.id === sessionId);
    if (!session) return;
    
    let modifyIdea = '';
    Modal.confirm({
      title: 'Modify Project',
      width: 600,
      content: (
        <div>
          <p>Base Session:</p>
          <p style={{ fontStyle: 'italic', color: '#1890ff', marginBottom: '15px' }}>
            {session.description}
          </p>
          <p>Changes:</p>
          <input
            type="text"
            placeholder="Describe what you want to change..."
            onChange={(e) => modifyIdea = e.target.value}
            autoFocus
            style={{ width: '100%', padding: '8px', marginBottom: '15px', background: '#1e1e1e', border: '1px solid #303030', color: '#fff', borderRadius: '4px' }}
          />
          <p style={{ fontSize: '12px', color: '#888' }}>
            This will create a new session with Type: Modify, based on {session.id.substring(0, 12)}...
          </p>
        </div>
      ),
      okText: 'Modify',
      cancelText: 'Cancel',
      onOk: async () => {
        if (!modifyIdea.trim()) {
          message.warning('Please describe the changes you want to make.');
          return;
        }
        setIsProcessing(true);
        try {
          const newSessionId = await invoke('modify_project', { 
            baseSessionId: session.id,
            idea: modifyIdea 
          });
          setCurrentSession(newSessionId);
          setMessages([]);
          setActiveView('chat');
          setMessages([{ 
            type: 'system', 
            content: `Modifying project based on session ${session.id.substring(0, 12)}...`, 
            timestamp: new Date().toISOString() 
          }]);
          setTimeout(() => loadSessions(), 500);
        } catch (error) {
          message.error('Failed to modify session: ' + error);
          setIsProcessing(false);
        }
      }
    });
  };

  const handleRevertSession = async (sessionId) => {
    const session = sessions.find(s => s.id === sessionId);
    if (!session) return;
    
    let selectedStage = 'prd';
    Modal.confirm({
      title: 'Revert Project',
      width: 600,
      content: (
        <div>
          <p>Base Session:</p>
          <p style={{ fontStyle: 'italic', color: '#1890ff', marginBottom: '15px' }}>
            {session.description}
          </p>
          <p>Revert to stage:</p>
          <select
            defaultValue="prd"
            onChange={(e) => selectedStage = e.target.value}
            style={{ width: '100%', padding: '8px', marginBottom: '15px', background: '#1e1e1e', border: '1px solid #303030', color: '#fff', borderRadius: '4px' }}
          >
            <option value="prd">prd (Requirements)</option>
            <option value="design">design (Design Specification)</option>
            <option value="plan">plan (Implementation Plan)</option>
            <option value="coding">coding (Code Generation)</option>
            <option value="check">check (Quality Check)</option>
            <option value="delivery">delivery (Final Delivery)</option>
            <option value="auto">auto (Use current stage)</option>
          </select>
          <p style={{ fontSize: '12px', color: '#888' }}>
            This will create a new session with Type: Revert, based on {session.id.substring(0, 12)}...
          </p>
        </div>
      ),
      okText: 'Revert',
      cancelText: 'Cancel',
      onOk: async () => {
        setIsProcessing(true);
        try {
          const newSessionId = await invoke('revert_project', { 
            baseSessionId: session.id,
            fromStage: selectedStage 
          });
          setCurrentSession(newSessionId);
          setMessages([]);
          setActiveView('chat');
          setMessages([{ 
            type: 'system', 
            content: `Reverting project from ${selectedStage} stage, based on session ${session.id.substring(0, 12)}...`, 
            timestamp: new Date().toISOString() 
          }]);
          setTimeout(() => loadSessions(), 500);
        } catch (error) {
          message.error('Failed to revert session: ' + error);
          setIsProcessing(false);
        }
      }
    });
  };

  const hasCompletedSession = (sessions) => {
    return sessions && sessions.some(s => s.status === 'Completed');
  };

  const getProjectStatus = (sessions) => {
    const inProgress = sessions.find(s => s.status === 'InProgress' || s.status === 'in_progress');
    if (inProgress) {
      return `Status: In Progress (${inProgress.id.substring(0, 20)}...) ⏳`;
    }
    const completed = sessions.filter(s => s.status === 'Completed' || s.status === 'completed');
    if (completed.length > 0) {
      return `Status: Ready ✓ (${completed.length} session${completed.length > 1 ? 's' : ''})`;
    }
    return 'Status: Not initialized';
  };

  const renderContent = () => {
    if (!currentSession) {
      return (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', flexDirection: 'column', color: '#888' }}>
          <Empty description="Select a session to view details" />
        </div>
      );
    }

    switch (activeView) {
      case 'chat':
        return (
          <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
            <div 
              ref={messagesContainerRef}
              style={{ flex: 1, overflow: 'auto', padding: '20px' }}
              onScroll={handleScroll}
            >
              {messages.map((msg, idx) => (
                <div key={idx} style={{ marginBottom: '20px', padding: '10px', background: msg.type === 'user' ? '#1890ff22' : msg.type === 'system' ? '#52c41a22' : '#262626', borderRadius: '8px' }}>
                  <div style={{ fontWeight: 'bold', marginBottom: '5px', color: msg.type === 'system' ? '#52c41a' : '#1890ff' }}>{msg.type}</div>
                  <div style={{ whiteSpace: 'pre-wrap' }}>{msg.content}</div>
                </div>
              ))}
              {isProcessing && (
                <div style={{ textAlign: 'center', padding: '20px' }}>
                  <Spin />
                  <div style={{ marginTop: '10px', color: '#888' }}>Processing...</div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </div>
            {inputRequest && (
              <div style={{ padding: '20px', borderTop: '1px solid #303030' }}>
                <div style={{ marginBottom: '10px' }}>{inputRequest.prompt}</div>
                <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
                  {inputRequest.options.map((option) => (
                    <Button key={option.id} onClick={() => handleSelectOption(option)}>
                      {option.label}
                    </Button>
                  ))}
                </div>
              </div>
            )}
            <div style={{ padding: '20px', borderTop: '1px solid #303030', display: 'flex', gap: '10px' }}>
              <input
                type="text"
                value={userInput}
                onChange={(e) => setUserInput(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && handleSendUserMessage()}
                placeholder={inputRequest ? "Type your response..." : "Type a message..."}
                style={{ flex: 1, padding: '10px', background: '#1e1e1e', border: '1px solid #303030', color: '#fff' }}
              />
              <Button onClick={handleSendUserMessage} disabled={!userInput.trim()}>Send</Button>
            </div>
          </div>
        );
      case 'artifacts':
        return <ArtifactsViewer sessionId={currentSession} />;
      case 'code':
        return <CodeEditor sessionId={currentSession} />;
      case 'preview':
        return <PreviewPanel sessionId={currentSession} />;
      case 'run':
        return <RunnerPanel sessionId={currentSession} />;
      default:
        return null;
    }
  };

  return (
    <Layout style={{ minHeight: '100vh', background: '#141414', color: '#fff' }}>
      <Header style={{ background: '#1f1f1f', borderBottom: '1px solid #303030', padding: '0 20px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '20px' }}>
          <h1 style={{ margin: 0, fontSize: '18px' }}>🛠️ Cowork Creative Studio</h1>
          <textarea
            id="projectIdeaInput"
            value={projectIdea}
            onChange={(e) => setProjectIdea(e.target.value)}
            placeholder="Describe your project idea..."
            rows={1}
            style={{ width: '400px', background: '#2a2a2a', border: '1px solid #303030', color: '#fff', borderRadius: '4px', padding: '5px 10px', resize: 'none' }}
          />
          <Button onClick={handleCreateProject} disabled={!projectIdea.trim() || isProcessing} type="primary">
            {isProcessing ? 'Processing...' : 'Create Project'}
          </Button>
          <Dropdown menu={{
            items: [
              {
                key: 'new',
                label: <span><FileTextOutlined /> Create New Project</span>,
                onClick: () => document.getElementById('projectIdeaInput')?.focus(),
                disabled: sessions.length > 0,
              },
              {
                key: 'modify',
                label: <span><EditOutlined /> Modify Project</span>,
                onClick: () => handleModifyProject(),
                disabled: !hasCompletedSession(sessions),
              },
              {
                key: 'revert',
                label: <span><RollbackOutlined /> Revert Project</span>,
                onClick: () => handleRevertProject(),
                disabled: !hasCompletedSession(sessions),
              },
            ]
          }}>
            <Button>
              Project Actions <DownOutlined />
            </Button>
          </Dropdown>
        </div>
        <div style={{ fontSize: '13px', color: '#888' }}>
          {getProjectStatus(sessions)}
        </div>
      </Header>

      <Layout style={{ height: 'calc(100vh - 64px - 40px)', display: 'flex', alignItems: 'stretch' }}>
        <Sider width={250} style={{ background: '#1f1f1f', borderRight: '1px solid #303030', display: 'flex', flexDirection: 'column', overflow: 'hidden', height: '100%' }}>
          <Menu 
            mode="inline" 
            selectedKeys={[activeView]} 
            onClick={({ key }) => setActiveView(key)}
            items={[
              { key: 'chat', icon: <MessageOutlined />, label: 'Chat' },
              { key: 'artifacts', icon: <FileTextOutlined />, label: 'Artifacts' },
              { key: 'code', icon: <CodeOutlined />, label: 'Code' },
              { key: 'preview', icon: <EyeOutlined />, label: 'Preview' },
              { key: 'run', icon: <PlayCircleOutlined />, label: 'Run' },
            ]}
            style={{ flexShrink: 0, flex: 'none' }}
          />
          <div style={{ marginTop: '20px', padding: '0 10px', borderTop: '1px solid #303030', overflow: 'auto', flex: 1, minHeight: 0, paddingBottom: '20px' }}>
            <h3 style={{ fontSize: '14px', color: '#888', marginTop: '20px' }}>Sessions {loadingSessions ? '(Loading...)' : `(${sessions.length})`}</h3>
            {loadingSessions ? (
              <div style={{ padding: '20px', textAlign: 'center' }}>
                <Spin size="small" />
              </div>
            ) : sessions.length === 0 ? (
              <div style={{ padding: '10px', color: '#666', fontSize: '12px', textAlign: 'center' }}>
                No sessions yet. Create a new project to get started.
              </div>
            ) : (
              <div>
                {/* 可操作的Sessions（Completed） */}
                {(() => {
                  const activeSessions = sessions.filter(s => s.status === 'Completed');
                  if (activeSessions.length === 0) return null;
                  
                  return (
                    <div>
                      <div style={{ fontSize: '11px', color: '#52c41a', marginTop: '15px', marginBottom: '5px', fontWeight: 'bold' }}>
                        ✓ Active Sessions ({activeSessions.length})
                      </div>
                      {activeSessions.map((session) => {
                        const isSelected = currentSession === session.id;
                        
                        return (
                          <div
                            key={session.id}
                            style={{
                              padding: '8px 10px',
                              cursor: 'pointer',
                              background: isSelected ? '#1890ff33' : 'transparent',
                              borderRadius: '4px',
                              marginBottom: '5px',
                              fontSize: '13px',
                              border: isSelected ? '1px solid #1890ff' : '1px solid transparent',
                              transition: 'all 0.2s'
                            }}
                            onClick={() => setCurrentSession(session.id)}
                          >
                            <div style={{ marginBottom: '8px' }}>
                              <div style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                                {session.description}
                              </div>
                              <div style={{ fontSize: '11px', color: '#888', marginTop: '3px' }}>
                                {new Date(session.created_at).toLocaleDateString()} · {session.status}
                              </div>
                            </div>
                            <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                              <Button
                                size="small"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleResumeSession(session.id);
                                }}
                                style={{ padding: '2px 6px', fontSize: '11px' }}
                              >
                                Resume
                              </Button>
                              <Button
                                size="small"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleModifySession(session.id);
                                }}
                                style={{ padding: '2px 6px', fontSize: '11px' }}
                              >
                                Modify
                              </Button>
                              <Button
                                size="small"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleRevertSession(session.id);
                                }}
                                style={{ padding: '2px 6px', fontSize: '11px' }}
                              >
                                Revert
                              </Button>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  );
                })()}
                
                {/* 不可操作的Sessions（InProgress/Failed） */}
                {(() => {
                  const inactiveSessions = sessions.filter(s => s.status !== 'Completed');
                  if (inactiveSessions.length === 0) return null;
                  
                  return (
                    <div>
                      <div 
                        style={{ 
                          fontSize: '11px', 
                          color: '#888', 
                          marginTop: '20px', 
                          marginBottom: '5px', 
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          gap: '5px'
                        }}
                        onClick={() => setShowInactiveSessions(!showInactiveSessions)}
                      >
                        <span>{showInactiveSessions ? '▼' : '▶'}</span>
                        <span>Inactive Sessions ({inactiveSessions.length})</span>
                      </div>
                      {showInactiveSessions && inactiveSessions.map((session) => {
                        const isSelected = currentSession === session.id;
                        const statusColor = session.status === 'InProgress' ? '#faad14' : '#ff4d4f';
                        
                        return (
                          <div
                            key={session.id}
                            style={{
                              padding: '8px 10px',
                              cursor: 'pointer',
                              background: isSelected ? '#1890ff33' : 'transparent',
                              borderRadius: '4px',
                              marginBottom: '5px',
                              fontSize: '13px',
                              border: isSelected ? '1px solid #1890ff' : '1px solid transparent',
                              transition: 'all 0.2s',
                              opacity: 0.7
                            }}
                            onClick={() => setCurrentSession(session.id)}
                          >
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '8px' }}>
                              <div style={{ flex: 1, minWidth: 0 }}>
                                <div style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                                  {session.description}
                                </div>
                                <div style={{ fontSize: '11px', color: '#888', marginTop: '3px' }}>
                                  {new Date(session.created_at).toLocaleDateString()} · 
                                  <span style={{ color: statusColor, marginLeft: '3px' }}>{session.status}</span>
                                </div>
                              </div>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  );
                })()}
              </div>
            )}
          </div>
        </Sider>

        <Content style={{ overflow: 'auto' }}>
          {renderContent()}
        </Content>
      </Layout>

      <Footer style={{ background: '#1f1f1f', borderTop: '1px solid #303030', padding: '10px 20px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontSize: '13px', color: '#888' }}>
          {currentSession ? `Session: ${currentSession.substring(0, 20)}...` : 'No session selected'}
        </span>
        <span style={{ fontSize: '13px', color: '#888' }}>
          {isProcessing ? '⚙️ Processing...' : '�?Ready'}
        </span>
      </Footer>
    </Layout>
  );
}

export default App;






