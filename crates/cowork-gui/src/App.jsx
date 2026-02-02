import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

function App() {
  const [sessions, setSessions] = useState([]);
  const [currentSession, setCurrentSession] = useState(null);
  const [messages, setMessages] = useState([]);
  const [projectIdea, setProjectIdea] = useState('');
  const [userInput, setUserInput] = useState('');
  const [inputRequest, setInputRequest] = useState(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const listenersRegistered = useRef(false);
  const cleanupFunctions = useRef([]);

  const loadSessions = async () => {
    try {
      const sessionsData = await invoke('get_sessions');
      setSessions(sessionsData || []);
    } catch (error) {
      console.error('Failed to load sessions:', error);
    }
  };

  useEffect(() => {
    // 防止重复注册
    if (listenersRegistered.current) {
      console.log('[Event Listeners] Already registered, skipping...');
      return;
    }

    console.log('[Event Listeners] Registering event listeners...');
    listenersRegistered.current = true;

    // 收到 agent 消息
    listen('agent_event', (event) => {
      const { content, is_thinking } = event.payload;

      console.log('[Agent Event] Received:', { content: content.substring(0, 50), is_thinking, length: content.length });

      // 更新处理状态
      setIsProcessing(is_thinking || content.trim().length > 0);

      if (!is_thinking && content) {
        // Agent有内容输出
        setMessages(prev => {
          // 检查是否正在流式输出
          const lastMsg = prev[prev.length - 1];
          if (lastMsg && lastMsg.type === 'agent' && lastMsg.isStreaming) {
            // 继续流式输出
            console.log('[Agent Event] Appending to streaming message (prev length:', lastMsg.content.length, ')');
            return [
              ...prev.slice(0, -1),
              {
                ...lastMsg,
                content: lastMsg.content + content,
                isStreaming: true
              }
            ];
          } else {
            // 新的消息
            console.log('[Agent Event] Creating new message');
            return [
              ...prev,
              {
                type: 'agent',
                content: content,
                isStreaming: true,
                timestamp: new Date().toISOString()
              }
            ];
          }
        });
      }
    }).then(unlisten => {
      cleanupFunctions.current.push(unlisten);
      console.log('[Event Listeners] agent_event listener registered');
    });

    // 收到 HITL 输入请求
    listen('input_request', (event) => {
      const [requestId, prompt, options] = event.payload;
      console.log('[HITL] Input request:', requestId);
      setInputRequest({ requestId, prompt, options });
      setUserInput('');
    }).then(unlisten => {
      cleanupFunctions.current.push(unlisten);
      console.log('[Event Listeners] input_request listener registered');
    });

    // Session 完成
    listen('session_completed', (event) => {
      console.log('[Session] Completed:', event.payload);
      setIsProcessing(false);
      loadSessions();
    }).then(unlisten => {
      cleanupFunctions.current.push(unlisten);
      console.log('[Event Listeners] session_completed listener registered');
    });

    // Session 失败
    listen('session_failed', (event) => {
      console.error('[Session] Failed:', event.payload);
      setIsProcessing(false);
      loadSessions();
    }).then(unlisten => {
      cleanupFunctions.current.push(unlisten);
      console.log('[Event Listeners] session_failed listener registered');
    });

    // 初始化时加载 sessions
    loadSessions();

    // 清理函数
    return () => {
      console.log('[Event Listeners] Cleaning up listeners...');
      cleanupFunctions.current.forEach(unlisten => {
        if (unlisten) {
          try {
            unlisten();
          } catch (e) {
            console.error('[Event Listeners] Error unlistening:', e);
          }
        }
      });
      cleanupFunctions.current = [];
      listenersRegistered.current = false;
    };
  }, []); // 空依赖数组，只在组件挂载时执行一次

  const handleCreateProject = async () => {
    if (!projectIdea.trim()) return;

    try {
      setIsProcessing(true);
      const sessionId = await invoke('create_project', { idea: projectIdea });
      console.log('Project created with session:', sessionId);

      // 显示用户的消息
      setMessages(prev => [...prev, {
        type: 'user',
        content: projectIdea,
        timestamp: new Date().toISOString()
      }]);

      setProjectIdea('');
      setCurrentSession(sessionId);

      // 等待一小段时间让后端启动
      setTimeout(() => loadSessions(), 500);
    } catch (error) {
      console.error('Failed to create project:', error);
      alert('Failed to create project: ' + error);
      setIsProcessing(false);
    }
  };

  const handleSendUserMessage = async () => {
    if (!userInput.trim()) return;

    try {
      // 显示用户消息
      setMessages(prev => [...prev, {
        type: 'user',
        content: userInput,
        timestamp: new Date().toISOString()
      }]);

      // 如果是HITL请求，提交响应
      if (inputRequest) {
        await invoke('submit_input_response', {
          requestId: inputRequest.requestId,
          response: userInput,
          responseType: 'text',
        });
        setInputRequest(null);
      }

      setUserInput('');
    } catch (error) {
      console.error('Failed to send message:', error);
      alert('Failed to send message: ' + error);
    }
  };

  const handleSelectOption = async (option) => {
    if (!inputRequest) return;

    try {
      // 显示用户消息
      setMessages(prev => [...prev, {
        type: 'user',
        content: option.label,
        timestamp: new Date().toISOString()
      }]);

      // 提交响应
      await invoke('submit_input_response', {
        requestId: inputRequest.requestId,
        response: option.id,
        responseType: 'selection',
      });

      setInputRequest(null);
      setUserInput('');
    } catch (error) {
      console.error('Failed to submit response:', error);
      alert('Failed to submit response: ' + error);
    }
  };

  const handleCancelInput = () => {
    if (!inputRequest) return;

    try {
      invoke('submit_input_response', {
        requestId: inputRequest.requestId,
        response: '',
        responseType: 'cancel',
      });
      setInputRequest(null);
      setUserInput('');
    } catch (error) {
      console.error('Failed to cancel request:', error);
    }
  };

  return (
    <div className="app">
      <header className="header">
        <div className="header-content">
          <h1>🛠️ Cowork Forge</h1>
          <p className="subtitle">AI-powered software development system</p>
        </div>
      </header>

      <div className="content">
        <aside className="sidebar">
          <div className="sidebar-section">
            <h2>New Project</h2>
            <textarea
              value={projectIdea}
              onChange={(e) => setProjectIdea(e.target.value)}
              placeholder="Describe your project idea..."
              rows={3}
            />
            <button
              onClick={handleCreateProject}
              className="btn-primary"
              disabled={!projectIdea.trim() || isProcessing}
            >
              {isProcessing ? 'Processing...' : 'Create Project'}
            </button>
          </div>

          <div className="sidebar-section">
            <h2>Sessions ({sessions.length})</h2>
            <div className="sessions-list">
              {sessions.length === 0 ? (
                <p className="empty-state">No sessions yet</p>
              ) : (
                sessions.map((session) => (
                  <div
                    key={session.id}
                    className={`session-item ${currentSession === session.id ? 'active' : ''}`}
                    onClick={() => setCurrentSession(session.id)}
                  >
                    <div className="session-desc">{session.description}</div>
                    <div className="session-meta">
                      <span className={`status-badge ${session.status.toLowerCase()}`}>
                        {session.status}
                      </span>
                      <span className="session-time">
                        {new Date(session.created_at).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </aside>

        <main className="chat-container">
          {currentSession ? (
            <>
              <div className="chat-header">
                <h2>{sessions.find(s => s.id === currentSession)?.description || 'Unknown Session'}</h2>
                {isProcessing && (
                  <div className="agent-status">
                    <div className="status-indicator">
                      <div className="status-dot status-dot-active"></div>
                      <span>Processing...</span>
                    </div>
                  </div>
                )}
              </div>

              <div className="messages-container">
                {messages.length === 0 && !isProcessing && (
                  <div className="welcome-message">
                    <p>Start by creating a new project!</p>
                  </div>
                )}

                {messages.map((msg, idx) => (
                  <div key={idx} className={`message message-${msg.type}`}>
                    <div className="message-content">
                      {msg.content}
                    </div>
                    <div className="message-time">
                      {new Date(msg.timestamp).toLocaleTimeString()}
                    </div>
                  </div>
                ))}

                {isProcessing && (
                  <div className="message message-agent">
                    <div className="message-content">
                      <div className="typing-indicator">
                        <span></span>
                        <span></span>
                        <span></span>
                      </div>
                    </div>
                  </div>
                )}

                <div ref={el => { if (el) el.scrollIntoView({ behavior: 'smooth' }); }}></div>
              </div>

              {inputRequest && (
                <div className="input-request-area">
                  <div className="input-prompt">{inputRequest.prompt}</div>
                  <div className="input-options">
                    {inputRequest.options.map((option) => (
                      <button
                        key={option.id}
                        onClick={() => handleSelectOption(option)}
                        className="option-btn"
                      >
                        <span className="option-label">{option.label}</span>
                        {option.description && (
                          <span className="option-desc">{option.description}</span>
                        )}
                      </button>
                    ))}
                  </div>
                </div>
              )}

              <div className="input-area">
                <div className="input-wrapper">
                  <input
                    type="text"
                    value={userInput}
                    onChange={(e) => setUserInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && handleSendUserMessage()}
                    placeholder={inputRequest ? "Type your response..." : "Type a message..."}
                    className="chat-input"
                  />
                  <button
                    onClick={handleSendUserMessage}
                    className="send-btn"
                    disabled={!userInput.trim()}
                  >
                    <span>→</span>
                  </button>
                </div>
                {inputRequest && (
                  <button
                    onClick={handleCancelInput}
                    className="cancel-btn"
                  >
                    Cancel
                  </button>
                )}
              </div>
            </>
          ) : (
            <div className="welcome-screen">
              <div className="welcome-content">
                <h2>Welcome to Cowork Forge!</h2>
                <p>Create a new project to get started.</p>
                <div className="features">
                  <div className="feature-card">
                    <span className="feature-icon">🚀</span>
                    <h3>Quick Start</h3>
                    <p>Enter your project idea and let AI generate the complete application.</p>
                  </div>
                  <div className="feature-card">
                    <span className="feature-icon">📊</span>
                    <h3>Session Management</h3>
                    <p>Track all your development sessions and resume from any point.</p>
                  </div>
                  <div className="feature-card">
                    <span className="feature-icon">🔄</span>
                    <h3>Incremental Updates</h3>
                    <p>Modify existing projects with AI-powered code patches.</p>
                  </div>
                </div>
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

export default App;