import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Layout, Menu, Button, Spin, Empty, Modal, Dropdown, message, Tooltip } from 'antd';
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
  SettingOutlined,
  AppstoreOutlined,
  DatabaseOutlined,
  BgColorsOutlined
} from '@ant-design/icons';
import ArtifactsViewer from './components/ArtifactsViewer';
import CodeEditor from './components/CodeEditor';
import PreviewPanel from './components/PreviewPanel';
import RunnerPanel from './components/RunnerPanel';
import ProjectsPanel from './components/ProjectsPanel';
import MemoryPanel from './components/MemoryPanel';
import CommandPalette from './components/CommandPalette';

const { Sider, Content, Header, Footer } = Layout;

function App() {
  const [sessions, setSessions] = useState([]);
  const [currentSession, setCurrentSession] = useState(null);
  const [projectIdea, setProjectIdea] = useState('');
  const [userInput, setUserInput] = useState('');
  const [inputRequest, setInputRequest] = useState(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [activeView, setActiveView] = useState('chat'); // chat, artifacts, code, preview, run, memory
  const [messages, setMessages] = useState([]);
  const [showModifySuggestion, setShowModifySuggestion] = useState(false);
  const [modifySuggestion, setModifySuggestion] = useState(null);
  const [pendingSessionId, setPendingSessionId] = useState(null);
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const listenersRegistered = useRef(false);
  const messagesEndRef = useRef(null);
  const messagesContainerRef = useRef(null);
  const [isUserScrolled, setIsUserScrolled] = useState(false);
  const [loadingSessions, setLoadingSessions] = useState(true);
  const [showInactiveSessions, setShowInactiveSessions] = useState(true);
  const [currentWorkspace, setCurrentWorkspace] = useState('');
  const initialLoadRef = useRef(true);

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e) => {
      // Ctrl+K or Cmd+K: Open command palette
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setShowCommandPalette(true);
      }
      // Ctrl+1: View chat
      if ((e.ctrlKey || e.metaKey) && e.key === '1') {
        e.preventDefault();
        setActiveView('chat');
      }
      // Ctrl+2: View artifacts
      if ((e.ctrlKey || e.metaKey) && e.key === '2') {
        e.preventDefault();
        setActiveView('artifacts');
      }
      // Ctrl+3: View code
      if ((e.ctrlKey || e.metaKey) && e.key === '3') {
        e.preventDefault();
        setActiveView('code');
      }
      // Ctrl+4: View preview
      if ((e.ctrlKey || e.metaKey) && e.key === '4') {
        e.preventDefault();
        setActiveView('preview');
      }
      // Ctrl+5: View run
      if ((e.ctrlKey || e.metaKey) && e.key === '5') {
        e.preventDefault();
        setActiveView('run');
      }
      // Ctrl+6: View memory
      if ((e.ctrlKey || e.metaKey) && e.key === '6') {
        e.preventDefault();
        setActiveView('memory');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Handle command selection from palette
  const handleCommandSelect = (commandId) => {
    switch (commandId) {
      case 'create-project':
        document.getElementById('projectIdeaInput')?.focus();
        break;
      case 'open-project':
        setActiveView('projects');
        break;
      case 'view-chat':
        setActiveView('chat');
        break;
      case 'view-artifacts':
        setActiveView('artifacts');
        break;
      case 'view-code':
        setActiveView('code');
        break;
      case 'view-preview':
        setActiveView('preview');
        break;
      case 'view-run':
        setActiveView('run');
        break;
      case 'view-memory':
        setActiveView('memory');
        break;
      default:
        console.log('Command selected:', commandId);
    }
  };

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

  const checkWorkspace = async () => {
    try {
      const workspace = await invoke('get_workspace');
      if (!workspace) {
        console.log('[App] No workspace set, showing projects view');
        setCurrentWorkspace('No workspace set');
        // Default to projects view when no workspace is set
        setActiveView('projects');
        // Clear sessions when no workspace is set
        setSessions([]);
      } else {
        console.log('[App] Workspace set:', workspace);
        setCurrentWorkspace(workspace);
      }
    } catch (error) {
      console.error('[App] Failed to check workspace:', error);
      setCurrentWorkspace('Unknown');
      // Default to projects view on error
      setActiveView('projects');
      // Clear sessions on error
      setSessions([]);
    }
  };

  useEffect(() => {
    console.log('[App] Component mounted, checking workspace...');
    checkWorkspace();
    loadSessions();
    
    // Listen for project loaded event
    const unlistenProjectLoaded = listen('project_loaded', () => {
      console.log('[App] Project loaded event received, reloading sessions...');
      loadSessions();
      checkWorkspace();
    });
    
    // Listen for project created event
    const unlistenProjectCreated = listen('project_created', () => {
      console.log('[App] Project created event received, reloading sessions...');
      loadSessions();
      checkWorkspace();
      // Switch to chat view when a new session is created
      setActiveView('chat');
    });
    
    // Listen for session completed event
    const unlistenSessionCompleted = listen('session_completed', () => {
      console.log('[App] Session completed event received, reloading sessions...');
      loadSessions();
    });
    
    return () => {
      if (unlistenProjectLoaded) {
        unlistenProjectLoaded.then(fn => fn()).catch(e => console.error('[App] Failed to unlisten project_loaded:', e));
      }
      if (unlistenProjectCreated) {
        unlistenProjectCreated.then(fn => fn()).catch(e => console.error('[App] Failed to unlisten project_created:', e));
      }
      if (unlistenSessionCompleted) {
        unlistenSessionCompleted.then(fn => fn()).catch(e => console.error('[App] Failed to unlisten session_completed:', e));
      }
    };
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
      
      // Refresh workspace info and sessions after creating project
      await checkWorkspace();
      setTimeout(() => loadSessions(), 500);
    } catch (error) {
      alert('Failed to create project: ' + error);
      setIsProcessing(false);
    }
  };

  const handleSendUserMessage = async () => {
    if (!userInput.trim()) return;
    const message = userInput;
    setMessages(prev => [...prev, { type: 'user', content: message, timestamp: new Date().toISOString() }]);
    
    if (inputRequest) {
      // HITL 交互
      await invoke('submit_input_response', { requestId: inputRequest.requestId, response: message, responseType: 'text' });
      setInputRequest(null);
    } else if (currentSession) {
      // 检查 session 状态，如果已 completed，使用 send_chat_message 命令
      const session = sessions.find(s => s.id === currentSession);
      if (session && session.status === 'Completed') {
        try {
          const response = await invoke('send_chat_message', {
            sessionId: currentSession,
            message: message
          });
          
          // 处理响应
          handleChatResponse(response);
        } catch (error) {
          console.error('[App] Failed to send chat message:', error);
          message.error('Failed to send message: ' + error);
        }
      }
    }
    
    setUserInput('');
  };

  const handleChatResponse = (response) => {
    switch (response.type) {
      case 'direct_processing':
        // 直接处理中
        setIsProcessing(true);
        message.info('Starting modification...');
        break;
        
      case 'await_confirmation':
        // 等待确认
        setModifySuggestion(response.data);
        setPendingSessionId(response.session_id);
        setShowModifySuggestion(true);
        break;
        
      case 'await_clarification':
        // 需要澄清
        Modal.info({
          title: 'Need More Information',
          content: (
            <div>
              <p>I need more information to help you:</p>
              {response.data?.questions?.map((q, i) => (
                <p key={i}>• {q}</p>
              ))}
            </div>
          )
        });
        break;
        
      case 'suggest_new_project':
        Modal.confirm({
          title: 'Create New Project?',
          content: 'It seems you want to start a new project. Would you like to create one?',
          onOk: () => {
            document.getElementById('projectIdeaInput')?.focus();
          }
        });
        break;
        
      default:
        console.log('[App] Unknown response type:', response.type);
    }
  };

  const handleConfirmModify = async () => {
    if (!modifySuggestion || !pendingSessionId) return;
    
    try {
      const newSessionId = await invoke('confirm_modify', {
        sessionId: pendingSessionId,
        suggestionStr: modifySuggestion
      });
      
      setShowModifySuggestion(false);
      setModifySuggestion(null);
      setPendingSessionId(null);
      
      message.success('Modification started successfully!');
      
      // 重新加载 sessions
          await loadSessions();
          
          // 切换到新的 session
          setCurrentSession(newSessionId);
          setMessages([]);
          setIsProcessing(true);
          
        } catch (error) {
          console.error('[App] Failed to confirm modification:', error);
          message.error('Failed to confirm modification: ' + error);
        }
      };
      
      const handleCancelModify = () => {
        setShowModifySuggestion(false);
        setModifySuggestion(null);
        setPendingSessionId(null);
      };
  const getChatPlaceholder = () => {
    if (inputRequest) {
      return "Type your response...";
    }
    if (currentSession) {
      const session = sessions.find(s => s.id === currentSession);
      if (session && session.status === 'Completed') {
        return "Continue developing? Describe what you want to change...";
      }
    }
    return "Type a message...";
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
    // Projects and Memory views don't require a session
    if (activeView === 'projects') {
      return <ProjectsPanel />;
    }

    if (activeView === 'memory') {
      return <MemoryPanel currentSession={currentSession} />;
    }

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
                <div key={idx} style={{ marginBottom: '20px', padding: '10px', background: msg.type === 'user' ? '#1890ff22' : msg.type === 'system' ? '#52c41a22' : msg.type === 'thinking' ? '#ffeb3b22' : '#262626', borderRadius: '8px' }}>
                  <div style={{ fontWeight: 'bold', marginBottom: '5px', color: msg.type === 'system' ? '#52c41a' : msg.type === 'thinking' ? '#ffeb3b' : '#1890ff' }}>
                    {msg.type === 'thinking' ? '🤔 AI Thinking' : msg.type}
                  </div>
                  <div style={{ whiteSpace: 'pre-wrap' }}>{msg.content}</div>
                  {msg.type === 'thinking' && (
                    <div style={{ marginTop: '8px', fontSize: '12px', color: '#888', fontStyle: 'italic' }}>
                      AI is reasoning about the next step...
                    </div>
                  )}
                </div>
              ))}
              {isProcessing && (
                <div style={{ textAlign: 'center', padding: '20px' }}>
                  <Spin />
                  <div style={{ marginTop: '10px', color: '#888' }}>
                    {isProcessing ? 'Processing...' : 'Ready'}
                  </div>
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
                placeholder={getChatPlaceholder()}
                style={{ flex: 1, padding: '10px', background: '#1e1e1e', border: '1px solid #303030', color: '#fff' }}
              />
              <Button onClick={handleSendUserMessage} disabled={!userInput.trim()}>Send</Button>
            </div>
          </div>
        );
      case 'artifacts':
        return <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}><ArtifactsViewer sessionId={currentSession} /></div>;
      case 'code':
        return <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}><CodeEditor sessionId={currentSession} /></div>;
      case 'preview':
        return <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}><PreviewPanel sessionId={currentSession} /></div>;
      case 'run':
        return <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}><RunnerPanel sessionId={currentSession} /></div>;
      case 'memory':
        return <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}><MemoryPanel currentSession={currentSession} /></div>;
      default:
        return null;
    }
  };

  return (
    <Layout style={{ minHeight: '100vh', background: '#141414', color: '#fff' }}>
      <Header style={{ background: '#1f1f1f', borderBottom: '1px solid #303030', padding: '0 20px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '20px' }}>
          <h1 style={{ margin: 0, fontSize: '16px', whiteSpace: 'nowrap' }}>Cowork Creative Studio</h1>
          <textarea
            id="projectIdeaInput"
            value={projectIdea}
            onChange={(e) => setProjectIdea(e.target.value)}
            placeholder="Describe your project idea..."
            rows={1}
            style={{ width: '350px', background: '#2a2a2a', border: '1px solid #303030', color: '#fff', borderRadius: '4px', padding: '5px 10px', resize: 'none' }}
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
              { key: 'projects', icon: <AppstoreOutlined />, label: 'Projects' },
              { key: 'chat', icon: <MessageOutlined />, label: 'Chat' },
              { key: 'artifacts', icon: <FileTextOutlined />, label: 'Artifacts' },
              { key: 'code', icon: <CodeOutlined />, label: 'Code' },
              { key: 'preview', icon: <EyeOutlined />, label: 'Preview' },
              { key: 'run', icon: <PlayCircleOutlined />, label: 'Run' },
              { key: 'memory', icon: <DatabaseOutlined />, label: 'Memory' },
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
                            <div style={{ marginBottom: '8px' }}>
                              <div style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                                {session.description}
                              </div>
                              <div style={{ fontSize: '11px', color: '#888', marginTop: '3px' }}>
                                {new Date(session.created_at).toLocaleDateString()} · 
                                <span style={{ color: statusColor, marginLeft: '3px' }}>{session.status}</span>
                              </div>
                            </div>
                            {session.status === 'InProgress' ? (
                              <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                                <div style={{ fontSize: '11px', color: '#faad14' }}>
                                  ⏳ {session.id === currentSession ? 'Currently running...' : 'In progress'}
                                </div>
                                {session.id !== currentSession && (
                                  <Button
                                    size="small"
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      message.info('This session is marked as in progress. Check if it is still running or needs to be retried.');
                                    }}
                                    style={{ padding: '2px 6px', fontSize: '11px' }}
                                  >
                                    View Details
                                  </Button>
                                )}
                              </div>
                            ) : session.status === 'Failed' ? (
                              <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                                <Button
                                  size="small"
                                  type="primary"
                                  danger
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    message.info('Session failed. Check logs for details or create a new session.');
                                  }}
                                  style={{ padding: '2px 6px', fontSize: '11px' }}
                                >
                                  View Details
                                </Button>
                              </div>
                            ) : null}
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

        <Content style={{ overflow: 'hidden', height: '100%', display: 'flex', flexDirection: 'column' }}>
          {renderContent()}
        </Content>
      </Layout>

      <Footer style={{ background: '#1f1f1f', borderTop: '1px solid #303030', padding: '10px 20px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '20px' }}>
          <Tooltip title={currentWorkspace || 'No workspace'} placement="top">
            <span style={{ fontSize: '13px', color: '#888', cursor: 'default' }}>
              {currentWorkspace ? `📁 ${currentWorkspace.split(/[/\\]/).pop()}` : '📁 No workspace'}
            </span>
          </Tooltip>
          <span style={{ fontSize: '13px', color: '#666' }}>
            {currentSession ? `Session: ${currentSession.substring(0, 20)}...` : 'No session selected'}
          </span>
        </div>
        <span style={{ fontSize: '13px', color: '#888' }}>
          {isProcessing ? '⚙️ Processing...' : '✅ Ready'}
        </span>
      </Footer>

      {/* 修改建议确认对话框 */}
      <Modal
        title="修改建议"
        open={showModifySuggestion}
        onOk={handleConfirmModify}
        onCancel={handleCancelModify}
        width={700}
        okText="开始修改"
        cancelText="取消"
      >
        {modifySuggestion && (
          <div>
            <h3 style={{ marginTop: 0 }}>{modifySuggestion.title}</h3>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>修改类型:</strong>{' '}
              <span style={{ marginLeft: '8px', padding: '2px 8px', background: '#1890ff22', color: '#1890ff', borderRadius: '4px' }}>
                {modifySuggestion.modification_type}
              </span>
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>受影响模块:</strong>
              <div style={{ marginTop: '8px' }}>
                {modifySuggestion.affected_modules?.map((module, idx) => (
                  <span key={idx} style={{ 
                    display: 'inline-block', 
                    marginRight: '8px', 
                    marginBottom: '8px',
                    padding: '2px 8px', 
                    background: '#52c41a22', 
                    color: '#52c41a', 
                    borderRadius: '4px',
                    fontSize: '12px'
                  }}>
                    {module}
                  </span>
                ))}
              </div>
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>实施计划:</strong>
              <ol style={{ marginTop: '8px', marginLeft: '20px' }}>
                {modifySuggestion.implementation_plan?.map((step, idx) => (
                  <li key={idx} style={{ marginBottom: '4px' }}>{step}</li>
                ))}
              </ol>
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>风险评估:</strong>
              <div style={{ marginTop: '8px', padding: '12px', background: '#262626', borderRadius: '4px' }}>
                <p style={{ margin: '0 0 8px 0' }}>
                  <strong>风险等级:</strong>{' '}
                  <span style={{ 
                    color: modifySuggestion.risk_assessment?.risk_level === 'high' ? '#ff4d4f' : 
                           modifySuggestion.risk_assessment?.risk_level === 'medium' ? '#faad14' : '#52c41a'
                  }}>
                    {modifySuggestion.risk_assessment?.risk_level?.toUpperCase()}
                  </span>
                </p>
                <p style={{ margin: '0 0 8px 0' }}>
                  <strong>预估工作量:</strong> {modifySuggestion.estimated_effort}
                </p>
                <p style={{ margin: '0 0 8px 0' }}>
                  <strong>置信度:</strong> {(modifySuggestion.confidence * 100).toFixed(0)}%
                </p>
                
                {modifySuggestion.risk_assessment?.risks && modifySuggestion.risk_assessment.risks.length > 0 && (
                  <div style={{ marginBottom: '8px' }}>
                    <strong>风险:</strong>
                    <ul style={{ marginTop: '4px', marginLeft: '20px', marginBottom: '0' }}>
                      {modifySuggestion.risk_assessment.risks.map((risk, idx) => (
                        <li key={idx}>{risk}</li>
                      ))}
                    </ul>
                  </div>
                )}
                
                {modifySuggestion.risk_assessment?.mitigation_strategies && modifySuggestion.risk_assessment.mitigation_strategies.length > 0 && (
                  <div>
                    <strong>缓解策略:</strong>
                    <ul style={{ marginTop: '4px', marginLeft: '20px', marginBottom: '0' }}>
                      {modifySuggestion.risk_assessment.mitigation_strategies.map((strategy, idx) => (
                        <li key={idx}>{strategy}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </Modal>

      {/* 命令面板 */}
      <CommandPalette
        visible={showCommandPalette}
        onClose={() => setShowCommandPalette(false)}
        onCommandSelect={handleCommandSelect}
      />
    </Layout>
  );
}

export default App;






