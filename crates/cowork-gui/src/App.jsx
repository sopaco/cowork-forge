import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Layout, Menu, Button, Spin, Empty, Modal, message, Tooltip, Badge, Input, Tag } from 'antd';
import {
  FolderOutlined,
  FileTextOutlined,
  CodeOutlined,
  EyeOutlined,
  PlayCircleOutlined,
  PlusOutlined,
  ReloadOutlined,
  MessageOutlined,
  AppstoreOutlined,
  DatabaseOutlined,
  BranchesOutlined,
  CheckCircleOutlined,
  RocketOutlined,
  CloseCircleOutlined,
  EditOutlined,
  SendOutlined,
} from '@ant-design/icons';
import ArtifactsViewer from './components/ArtifactsViewer';
import CodeEditor from './components/CodeEditor';
import PreviewPanel from './components/PreviewPanel';
import RunnerPanel from './components/RunnerPanel';
import ProjectsPanel from './components/ProjectsPanel';
import MemoryPanel from './components/MemoryPanel';
import CommandPalette from './components/CommandPalette';
import IterationsPanel from './components/IterationsPanel';

const { Sider, Content, Header, Footer } = Layout;

// Stage definitions for status display
const STAGES = [
  { key: 'idea', label: 'Idea', color: '#1890ff' },
  { key: 'prd', label: 'PRD', color: '#52c41a' },
  { key: 'design', label: 'Design', color: '#722ed1' },
  { key: 'plan', label: 'Plan', color: '#fa8c16' },
  { key: 'coding', label: 'Coding', color: '#13c2c2' },
  { key: 'check', label: 'Check', color: '#eb2f96' },
  { key: 'delivery', label: 'Delivery', color: '#52c41a' },
];

function App() {
  const [project, setProject] = useState(null);
  const [iterations, setIterations] = useState([]);
  const [currentIteration, setCurrentIteration] = useState(null);
  const [activeView, setActiveView] = useState('projects'); // Default to projects tab when no project is loaded
  const [hasInitialProject, setHasInitialProject] = useState(false);
  const [messages, setMessages] = useState([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [currentStage, setCurrentStage] = useState(null);
  const [inputRequest, setInputRequest] = useState(null);
  const [userInput, setUserInput] = useState('');
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [loading, setLoading] = useState(true);
  const [activeArtifactTab, setActiveArtifactTab] = useState(null);
  const [currentAgent, setCurrentAgent] = useState(null);

  const listenersRegistered = useRef(false);
  const messagesContainerRef = useRef(null);

  // Load initial data
  const loadData = async () => {
    try {
      setLoading(true);

      // Load project
      const projectData = await invoke('gui_get_project');
      setProject(projectData);

      if (projectData) {
        // Load iterations
        const iterationsData = await invoke('gui_get_iterations');
        setIterations(iterationsData || []);

        // Update current iteration if it exists
        if (currentIteration) {
          const updatedIteration = iterationsData?.find(
            (i) => i.id === currentIteration.id
          );
          if (updatedIteration) {
            setCurrentIteration(updatedIteration);
          }
        } else if (projectData.current_iteration_id) {
          const iteration = iterationsData?.find(
            (i) => i.id === projectData.current_iteration_id
          );
          setCurrentIteration(iteration || null);
        }

        // If we have a project and haven't switched to iterations yet, do it now
        if (!hasInitialProject) {
          setActiveView('iterations');
          setHasInitialProject(true);
        }
      } else {
        // No project loaded, ensure we're on projects tab
        setActiveView('projects');
        setHasInitialProject(false);
      }
    } catch (error) {
      console.error('[App] Failed to load data:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();

    // Listen for events
    const setupListeners = async () => {
      if (listenersRegistered.current) return;
      listenersRegistered.current = true;

      await listen('iteration_created', () => {
        loadData();
        message.success('Iteration created');
      });

      await listen('iteration_started', (event) => {
        const iterationId = event.payload;
        setIsProcessing(true);
        // Directly update currentIteration status if it matches
        setCurrentIteration((prev) => {
          if (prev && prev.id === iterationId) {
            return { ...prev, status: 'Running' };
          }
          return prev;
        });
        message.info('Iteration started');
      });

      await listen('iteration_continued', (event) => {
        const iterationId = event.payload;
        setIsProcessing(true);
        // Directly update currentIteration status if it matches
        setCurrentIteration((prev) => {
          if (prev && prev.id === iterationId) {
            return { ...prev, status: 'Running' };
          }
          return prev;
        });
        message.info('Iteration continued');
      });

      await listen('iteration_completed', (event) => {
        const iterationId = event.payload;
        setIsProcessing(false);
        setCurrentStage(null);
        setCurrentAgent(null);
        // Reset input request in case it was left open
        setInputRequest(null);
        // Update current iteration status
        setCurrentIteration((prev) => {
          if (prev && prev.id === iterationId) {
            return { ...prev, status: 'Completed', current_stage: null };
          }
          return prev;
        });
        // Refresh all data to sync project state
        loadData();
        message.success('Iteration completed');
      });

      await listen('iteration_failed', (event) => {
        const [iterationId, error] = event.payload;
        setIsProcessing(false);
        setCurrentStage(null);
        setCurrentAgent(null);
        // Reset input request in case it was left open
        setInputRequest(null);
        // Update current iteration status
        setCurrentIteration((prev) => {
          if (prev && prev.id === iterationId) {
            return { ...prev, status: 'Failed', current_stage: null };
          }
          return prev;
        });
        // Refresh all data
        loadData();
        message.error('Iteration failed: ' + error);
      });

      await listen('agent_event', (event) => {
        const { content, is_thinking, agent_name } = event.payload;
        // Track current agent for processing display
        if (agent_name) {
          setCurrentAgent(agent_name);
        }
        if (!is_thinking && content) {
          setMessages((prev) => {
            const lastMsg = prev[prev.length - 1];
            if (lastMsg && lastMsg.type === 'agent' && lastMsg.isStreaming) {
              return [
                ...prev.slice(0, -1),
                {
                  ...lastMsg,
                  content: lastMsg.content + content,
                  agentName: agent_name || lastMsg.agentName,
                },
              ];
            } else {
              return [
                ...prev,
                {
                  type: 'agent',
                  content,
                  agentName: agent_name || 'AI Agent',
                  isStreaming: true,
                  timestamp: new Date().toISOString(),
                },
              ];
            }
          });
        }
      });

      await listen('input_request', (event) => {
        const [requestId, prompt, options] = event.payload;
        // Check if this is an artifact confirmation request
        const artifactMatch = prompt.match(/\[ARTIFACT_TYPE:(\w+)\]$/);
        if (artifactMatch) {
          const artifactType = artifactMatch[1];
          const cleanPrompt = prompt.replace(/\[ARTIFACT_TYPE:\w+\]$/, '').trim();
          setInputRequest({ 
            requestId, 
            prompt: cleanPrompt, 
            options,
            isArtifactConfirmation: true,
            artifactType
          });
        } else {
          setInputRequest({ requestId, prompt, options });
        }
        setUserInput('');
      });

      await listen('project_loaded', () => {
        // Reset execution states when loading a different project
        setIsProcessing(false);
        setCurrentAgent(null);
        setInputRequest(null);
        setMessages([]);
        setCurrentIteration(null); // Reset current iteration when switching projects
        setActiveView('iterations'); // Switch to iterations view
        loadData();
        message.success('Project loaded');
      });

      await listen('project_initialized', () => {
        // Reset execution states
        setIsProcessing(false);
        setCurrentAgent(null);
        setInputRequest(null);
        setMessages([]);
        setCurrentIteration(null); // Reset current iteration
        setActiveView('iterations'); // Switch to iterations view
        loadData();
        message.success('Project initialized');
      });
    };

    setupListeners();

    // Keyboard shortcuts
    const handleKeyDown = (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setShowCommandPalette(true);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Auto-scroll messages
  useEffect(() => {
    if (messagesContainerRef.current) {
      messagesContainerRef.current.scrollTop = messagesContainerRef.current.scrollHeight;
    }
  }, [messages]);

  const handleSelectIteration = (iterationId) => {
    const iteration = iterations.find((i) => i.id === iterationId);
    setCurrentIteration(iteration);
    setActiveView('chat');
  };

  const handleExecuteStatusChange = (iterationId, status) => {
    // Update local state when execution status changes
    if (status === 'running') {
      setIsProcessing(true);
      // Find the iteration and update it
      const iteration = iterations.find((i) => i.id === iterationId);
      if (iteration) {
        setCurrentIteration({ ...iteration, status: 'Running' });
      }
    } else if (status === 'completed') {
      setIsProcessing(false);
      loadData(); // Refresh data
    } else if (status === 'error') {
      setIsProcessing(false);
    }
  };

  const handleCreateGenesisIteration = async (title, description) => {
    try {
      const request = {
        title,
        description,
        base_iteration_id: null,
        inheritance: 'none',
      };

      const iteration = await invoke('gui_create_iteration', { request });
      setCurrentIteration(iteration);
      setActiveView('chat');
      message.success('Genesis iteration created');
    } catch (error) {
      message.error('Failed to create iteration: ' + error);
    }
  };

  const handleExecuteIteration = async () => {
    if (!currentIteration) return;

    try {
      setIsProcessing(true);
      await invoke('gui_execute_iteration', { iterationId: currentIteration.id });
      message.info('Iteration execution started');
    } catch (error) {
      message.error('Failed to execute iteration: ' + error);
      setIsProcessing(false);
    }
  };

  const handleSendUserMessage = async () => {
    if (!userInput.trim()) return;

    const message = userInput;
    setMessages((prev) => [
      ...prev,
      { type: 'user', content: message, timestamp: new Date().toISOString() },
    ]);

    if (inputRequest) {
      await invoke('submit_input_response', {
        requestId: inputRequest.requestId,
        response: message,
        responseType: 'text',
      });
      setInputRequest(null);
    }

    setUserInput('');
  };

  const handleSelectOption = async (option) => {
    if (!inputRequest) return;

    // Handle view artifact option specially
    if (option.id === 'view_artifact' && inputRequest.isArtifactConfirmation) {
      // Map artifactType to tab key
      const artifactTypeToTab = {
        'idea': 'idea',
        'requirements': 'requirements',
        'design': 'design',
        'plan': 'plan',
        'code': 'code',
      };
      const targetTab = artifactTypeToTab[inputRequest.artifactType] || 'idea';
      
      // Set the active tab before switching view
      setActiveArtifactTab(targetTab);
      // Switch to artifacts tab
      setActiveView('artifacts');
      message.info(`Switched to Artifacts tab to review ${inputRequest.artifactType}`);
      // Keep the input request active so user can confirm after viewing
      return;
    }

    // Handle feedback option - show feedback input mode
    if (option.id === 'feedback' && inputRequest.isArtifactConfirmation) {
      setInputRequest({
        ...inputRequest,
        isFeedbackMode: true,
        feedbackPrompt: 'Please enter your feedback or suggestions for improvement:'
      });
      setUserInput('');
      return;
    }

    // Add user message with context
    let userMessageContent = option.label;
    if (option.id === 'yes' && inputRequest.isArtifactConfirmation) {
      userMessageContent = `✅ Confirmed: Proceed to next stage`;
    } else if (option.id === 'no') {
      userMessageContent = `❌ Cancelled: Stop iteration`;
    }

    setMessages((prev) => [
      ...prev,
      { type: 'user', content: userMessageContent, timestamp: new Date().toISOString() },
    ]);

    await invoke('submit_input_response', {
      requestId: inputRequest.requestId,
      response: option.id,
      responseType: 'selection',
    });

    setInputRequest(null);
    setUserInput('');
  };

  const handleSubmitFeedback = async () => {
    if (!inputRequest || !userInput.trim()) return;

    const feedback = userInput.trim();
    
    // Add system confirmation message before user feedback
    setMessages((prev) => [
      ...prev,
      { 
        type: 'agent', 
        content: `📝 Feedback received. Regenerating based on your input...`, 
        agentName: 'System',
        timestamp: new Date().toISOString() 
      },
      { 
        type: 'user', 
        content: `💬 Feedback:\n${feedback}`, 
        timestamp: new Date().toISOString() 
      },
    ]);

    // Send feedback as text response
    await invoke('submit_input_response', {
      requestId: inputRequest.requestId,
      response: feedback,
      responseType: 'text',
    });

    setInputRequest(null);
    setUserInput('');
  };

  const handleCancelFeedback = () => {
    // Cancel feedback mode and go back to confirmation options
    if (inputRequest) {
      setInputRequest({
        ...inputRequest,
        isFeedbackMode: false,
      });
      setUserInput('');
    }
  };

  const getStatusBadge = (status) => {
    switch (status?.toLowerCase()) {
      case 'completed':
        return <Badge status="success" text="Completed" />;
      case 'running':
        return <Badge status="processing" text="Running" />;
      case 'paused':
        return <Badge status="warning" text="Paused" />;
      case 'failed':
        return <Badge status="error" text="Failed" />;
      default:
        return <Badge status="default" text="Draft" />;
    }
  };

  const renderContent = () => {
    switch (activeView) {
      case 'iterations':
        return (
          <IterationsPanel
            onSelectIteration={handleSelectIteration}
            selectedIterationId={currentIteration?.id}
            onExecuteStatusChange={handleExecuteStatusChange}
          />
        );

      case 'projects':
        return <ProjectsPanel />;

      case 'chat':
        if (!currentIteration) {
          return (
            <Empty
              description="Select an iteration to view chat"
              style={{ marginTop: '40px' }}
            >
              <Button type="primary" onClick={() => setActiveView('iterations')}>
                Go to Iterations
              </Button>
            </Empty>
          );
        }
        return (
          <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
            <div
              ref={messagesContainerRef}
              style={{ flex: 1, overflow: 'auto', padding: '20px' }}
            >
              <div style={{ marginBottom: '20px', padding: '12px', background: '#f0f5ff', borderRadius: '8px' }}>
                <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>
                  <RocketOutlined style={{ marginRight: '8px' }} />
                  #{currentIteration.number} - {currentIteration.title}
                </div>
                <div style={{ fontSize: '12px', color: '#666' }}>
                  {getStatusBadge(currentIteration.status)}
                  {currentIteration.current_stage && (
                    <span style={{ marginLeft: '16px' }}>
                      Current: <Tag color="blue">{currentIteration.current_stage}</Tag>
                    </span>
                  )}
                </div>
              </div>

              {messages.map((msg, idx) => (
                <div
                  key={idx}
                  style={{
                    marginBottom: '16px',
                    padding: '12px',
                    background: msg.type === 'user' ? '#e6f7ff' : '#f6ffed',
                    borderRadius: '8px',
                    borderLeft: `4px solid ${msg.type === 'user' ? '#1890ff' : '#52c41a'}`,
                  }}
                >
                  <div style={{ fontWeight: 'bold', marginBottom: '4px', fontSize: '12px', color: '#666' }}>
                    {msg.type === 'user' ? 'You' : msg.agentName || 'AI Agent'}
                  </div>
                  <div style={{ whiteSpace: 'pre-wrap' }}>{msg.content}</div>
                </div>
              ))}

              {isProcessing && (
                <div style={{ textAlign: 'center', padding: '20px' }}>
                  <Spin />
                  <div style={{ marginTop: '8px', color: '#888' }}>
                    {currentAgent ? (
                      <>{currentAgent} is working...</>
                    ) : currentIteration?.current_stage ? (
                      `Executing ${currentIteration.current_stage} stage...`
                    ) : (
                      'Processing...'
                    )}
                  </div>
                  {currentIteration?.current_stage && (
                    <div style={{ marginTop: '4px', fontSize: '12px', color: '#aaa' }}>
                      Stage: {currentIteration.current_stage}
                    </div>
                  )}
                </div>
              )}
            </div>

            {inputRequest && (
              <div style={{ padding: '16px', borderTop: '1px solid #e8e8e8', background: '#fafafa' }}>
                {inputRequest.isFeedbackMode ? (
                  // Feedback input mode
                  <>
                    <div style={{ marginBottom: '12px', fontWeight: 'bold' }}>
                      <Tag color="orange" style={{ marginRight: '8px' }}>
                        <EditOutlined /> Feedback Mode
                      </Tag>
                      {inputRequest.feedbackPrompt}
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                      <Input.TextArea
                        value={userInput}
                        onChange={(e) => setUserInput(e.target.value)}
                        placeholder="Enter your feedback or suggestions for improvement..."
                        rows={4}
                        autoFocus
                      />
                      <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
                        <Button onClick={handleCancelFeedback}>
                          Cancel
                        </Button>
                        <Button 
                          type="primary" 
                          icon={<SendOutlined />}
                          onClick={handleSubmitFeedback}
                          disabled={!userInput.trim()}
                        >
                          Submit Feedback
                        </Button>
                      </div>
                    </div>
                  </>
                ) : (
                  // Normal confirmation mode
                  <>
                    <div style={{ marginBottom: '12px', fontWeight: 'bold' }}>
                      {inputRequest.isArtifactConfirmation && (
                        <Tag color="blue" style={{ marginRight: '8px' }}>
                          <EyeOutlined /> Review Required
                        </Tag>
                      )}
                      {inputRequest.prompt}
                    </div>
                    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                      {inputRequest.options.map((option) => (
                        <Button 
                          key={option.id} 
                          type={option.id === 'yes' ? 'primary' : option.id === 'view_artifact' ? 'dashed' : option.id === 'feedback' ? 'dashed' : 'default'}
                          danger={option.id === 'no'}
                          icon={
                            option.id === 'view_artifact' ? <EyeOutlined /> : 
                            option.id === 'yes' ? <CheckCircleOutlined /> : 
                            option.id === 'no' ? <CloseCircleOutlined /> : 
                            option.id === 'feedback' ? <EditOutlined /> : null
                          }
                          onClick={() => handleSelectOption(option)}
                        >
                          {option.label}
                        </Button>
                      ))}
                    </div>
                  </>
                )}
              </div>
            )}

            <div style={{ padding: '16px', borderTop: '1px solid #e8e8e8', display: 'flex', gap: '8px' }}>
              <Input
                value={userInput}
                onChange={(e) => setUserInput(e.target.value)}
                onPressEnter={handleSendUserMessage}
                placeholder={inputRequest ? 'Type your response...' : 'Type a message...'}
                disabled={!inputRequest}
              />
              <Button type="primary" onClick={handleSendUserMessage} disabled={!userInput.trim() || !inputRequest}>
                Send
              </Button>
            </div>
          </div>
        );

      case 'artifacts':
        return currentIteration ? (
          <ArtifactsViewer 
            iterationId={currentIteration.id} 
            activeTab={activeArtifactTab}
            onTabChange={setActiveArtifactTab}
          />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        );

      case 'code':
        return currentIteration ? (
          <CodeEditor iterationId={currentIteration.id} />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        );

      case 'preview':
        return currentIteration ? (
          <PreviewPanel iterationId={currentIteration.id} />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        );

      case 'run':
        return currentIteration ? (
          <RunnerPanel iterationId={currentIteration.id} />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        );

      case 'memory':
        return <MemoryPanel currentSession={currentIteration?.id} />;

      default:
        return null;
    }
  };

  if (loading) {
    return (
      <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <Spin size="large" tip="Loading..." />
      </div>
    );
  }

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ background: '#fff', borderBottom: '1px solid #e8e8e8', padding: '0 24px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <h1 style={{ margin: 0, fontSize: '18px' }}>
            <RocketOutlined style={{ marginRight: '8px', color: '#1890ff' }} />
            Cowork Forge
          </h1>
          {project && (
            <Tag color="blue">{project.name}</Tag>
          )}
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          {currentIteration && (
            <>
              {getStatusBadge(currentIteration.status)}
              {(currentIteration.status === 'Draft' || currentIteration.status === 'Paused') && (
                <Button
                  type="primary"
                  icon={currentIteration.status === 'Draft' ? <PlayCircleOutlined /> : <ReloadOutlined />}
                  onClick={handleExecuteIteration}
                  loading={isProcessing}
                >
                  {currentIteration.status === 'Draft' ? 'Start Iteration' : 'Continue'}
                </Button>
              )}
            </>
          )}
        </div>
      </Header>

      <Layout style={{ height: 'calc(100vh - 64px - 48px)' }}>
        <Sider width={200} style={{ background: '#fff', borderRight: '1px solid #e8e8e8' }}>
          <Menu
            mode="inline"
            selectedKeys={[activeView]}
            onClick={({ key }) => setActiveView(key)}
            style={{ height: '100%', borderRight: 0 }}
            items={[
              { key: 'iterations', icon: <BranchesOutlined />, label: 'Iterations' },
              { key: 'chat', icon: <MessageOutlined />, label: 'Chat' },
              { key: 'artifacts', icon: <FileTextOutlined />, label: 'Artifacts' },
              { key: 'code', icon: <CodeOutlined />, label: 'Code' },
              { key: 'preview', icon: <EyeOutlined />, label: 'Preview' },
              { key: 'run', icon: <PlayCircleOutlined />, label: 'Run' },
              { key: 'memory', icon: <DatabaseOutlined />, label: 'Memory' },
              { key: 'projects', icon: <AppstoreOutlined />, label: 'Projects' },
            ]}
          />
        </Sider>

        <Content style={{ overflow: 'hidden', height: '100%', display: 'flex', flexDirection: 'column' }}>
          {renderContent()}
        </Content>
      </Layout>

      <Footer style={{ background: '#fff', borderTop: '1px solid #e8e8e8', padding: '12px 24px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ fontSize: '12px', color: '#888' }}>
          {project ? (
            <>
              <span style={{ marginRight: '16px' }}>
                Project: <strong>{project.name}</strong>
              </span>
              <span>
                Iterations: <strong>{iterations.length}</strong>
              </span>
            </>
          ) : (
            'No project loaded'
          )}
        </div>
        <div style={{ fontSize: '12px', color: '#888' }}>
          {isProcessing ? (
            <span style={{ color: '#1890ff' }}>
              <Spin size="small" style={{ marginRight: '8px' }} />
              {currentAgent ? `${currentAgent} is working...` : 'Processing...'}
            </span>
          ) : (
            <span style={{ color: '#52c41a' }}>
              <CheckCircleOutlined style={{ marginRight: '4px' }} />
              Ready
            </span>
          )}
        </div>
      </Footer>

      <CommandPalette
        visible={showCommandPalette}
        onClose={() => setShowCommandPalette(false)}
        onCommandSelect={(commandId) => {
          switch (commandId) {
            case 'view-iterations':
              setActiveView('iterations');
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
            case 'view-projects':
              setActiveView('projects');
              break;
          }
        }}
      />
    </Layout>
  );
}

export default App;
