import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Layout, Menu, Button, Spin, Empty, Modal, message, Tooltip, Badge, Input, Tag, Space } from 'antd';
import {
  FolderOutlined,
  FileTextOutlined,
  CodeOutlined,
  EyeOutlined,
  PlayCircleOutlined,
  ReloadOutlined,
  MessageOutlined,
  AppstoreOutlined,
  DatabaseOutlined,
  BranchesOutlined,
  CheckCircleOutlined,
  RocketOutlined,
  CloseCircleOutlined,
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

        // Auto-switch to iterations when project is loaded (unless explicitly on projects tab)
        if (activeView === 'projects') {
          setActiveView('iterations');
        }
      } else {
        // No project loaded, ensure we're on projects tab
        setActiveView('projects');
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
  return (
    <div style={{ height: '100%' }}>
      <div style={{ height: '100%', display: activeView === 'iterations' ? 'block' : 'none' }}>
        <IterationsPanel 
          key="iterations"
          onSelectIteration={handleSelectIteration}
          selectedIterationId={currentIteration?.id}
          onExecuteStatusChange={handleExecuteStatusChange}
        />
      </div>

      <div style={{ height: '100%', display: activeView === 'projects' ? 'block' : 'none' }}>
        <ProjectsPanel key="projects" />
      </div>

      <div style={{ height: '100%', display: activeView === 'artifacts' ? 'block' : 'none' }}>
        {currentIteration ? (
          <ArtifactsViewer 
            key={`artifacts-${currentIteration.id}`}
            iterationId={currentIteration.id} 
            activeTab={activeArtifactTab}
            onTabChange={setActiveArtifactTab}
          />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        )}
      </div>

      <div style={{ height: '100%', display: activeView === 'code' ? 'block' : 'none' }}>
        {currentIteration ? (
          <CodeEditor key={`code-${currentIteration.id}`} iterationId={currentIteration.id} />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        )}
      </div>

      <div style={{ height: '100%', display: activeView === 'preview' ? 'block' : 'none' }}>
        {currentIteration ? (
          <PreviewPanel key={`preview-${currentIteration.id}`} iterationId={currentIteration.id} />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        )}
      </div>

      <div style={{ height: '100%', display: activeView === 'run' ? 'block' : 'none' }}>
        {currentIteration ? (
          <RunnerPanel key={`run-${currentIteration.id}`} iterationId={currentIteration.id} />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        )}
      </div>

      <div style={{ height: '100%', display: activeView === 'memory' ? 'block' : 'none' }}>
        <MemoryPanel key="memory" currentSession={currentIteration?.id} />
      </div>

      <div style={{ height: '100%', display: activeView === 'chat' ? 'block' : 'none' }}>
        {currentIteration ? (
          <div style={{ height: '100%', display: 'flex', flexDirection: 'column', padding: '16px' }}>
            <div style={{ marginBottom: '16px' }}>
              <h3 style={{ margin: 0 }}>Chat - {currentIteration.title}</h3>
              <p style={{ margin: '4px 0 0 0', color: '#888', fontSize: '12px' }}>
                {currentIteration.description}
              </p>
            </div>
            
            <div 
              ref={messagesContainerRef}
              style={{ 
                flex: 1, 
                overflow: 'auto', 
                border: '1px solid #e8e8e8', 
                borderRadius: '4px',
                padding: '16px',
                marginBottom: '16px',
                backgroundColor: '#fafafa'
              }}
            >
              {messages.length === 0 ? (
                <div style={{ color: '#888', textAlign: 'center', marginTop: '40px' }}>
                  {isProcessing ? 'Waiting for agent response...' : 'No messages yet. Start the iteration to begin chatting.'}
                </div>
              ) : (
                messages.map((msg, index) => (
                  <div key={index} style={{ marginBottom: '16px' }}>
                    {msg.type === 'user' ? (
                      <div style={{ textAlign: 'right' }}>
                        <div style={{ 
                          display: 'inline-block', 
                          backgroundColor: '#1890ff', 
                          color: '#fff', 
                          padding: '8px 12px', 
                          borderRadius: '4px',
                          maxWidth: '70%',
                          wordBreak: 'break-word'
                        }}>
                          {msg.content}
                        </div>
                      </div>
                    ) : (
                      <div>
                        <div style={{ fontSize: '12px', color: '#888', marginBottom: '4px' }}>
                          {msg.agentName || 'AI Agent'}
                        </div>
                        <div style={{ 
                          backgroundColor: '#fff', 
                          padding: '8px 12px', 
                          borderRadius: '4px',
                          border: '1px solid #e8e8e8',
                          maxWidth: '70%',
                          wordBreak: 'break-word',
                          whiteSpace: 'pre-wrap'
                        }}>
                          {msg.content}
                        </div>
                      </div>
                    )}
                  </div>
                ))
              )}
            </div>

            {inputRequest && (
              <div style={{ 
                padding: '16px', 
                backgroundColor: '#fff7e6', 
                border: '1px solid #ffd591', 
                borderRadius: '4px',
                marginBottom: '16px'
              }}>
                <div style={{ marginBottom: '8px', fontWeight: 'bold' }}>
                  {inputRequest.isArtifactConfirmation 
                    ? `Confirm ${inputRequest.artifactType}`
                    : 'Input Required'}
                </div>
                <div style={{ marginBottom: '12px', color: '#666' }}>
                  {inputRequest.isFeedbackMode ? inputRequest.feedbackPrompt : inputRequest.prompt}
                </div>
                {inputRequest.options && !inputRequest.isFeedbackMode && (
                  <Space direction="vertical" style={{ width: '100%' }}>
                    {inputRequest.options.map((option) => (
                      <Button 
                        key={option.id} 
                        onClick={() => handleSelectOption(option)}
                        block
                      >
                        {option.label}
                      </Button>
                    ))}
                  </Space>
                )}
              </div>
            )}

            <div style={{ display: 'flex', gap: '8px' }}>
              <Input
                value={userInput}
                onChange={(e) => setUserInput(e.target.value)}
                onPressEnter={handleSendUserMessage}
                placeholder={inputRequest ? 'Type your response...' : 'Type a message...'}
                disabled={isProcessing && !inputRequest}
              />
              {inputRequest ? (
                <Button 
                  onClick={handleSubmitFeedback} 
                  type="primary"
                  disabled={!userInput.trim()}
                >
                  Send Feedback
                </Button>
              ) : (
                <Button 
                  onClick={handleSendUserMessage} 
                  type="primary"
                  disabled={!userInput.trim()}
                >
                  Send
                </Button>
              )}
              {inputRequest && inputRequest.isFeedbackMode && (
                <Button onClick={handleCancelFeedback}>
                  Cancel
                </Button>
              )}
            </div>
          </div>
        ) : (
          <Empty 
            description="Select an iteration to view chat" 
            style={{ marginTop: '40px' }} 
          />
        )}
      </div>
    </div>
  );
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
              { key: 'projects', icon: <AppstoreOutlined />, label: 'Projects' },
              { key: 'iterations', icon: <BranchesOutlined />, label: 'Iterations' },
              { key: 'chat', icon: <MessageOutlined />, label: 'Chat' },
              { key: 'artifacts', icon: <FileTextOutlined />, label: 'Artifacts' },
              { key: 'code', icon: <CodeOutlined />, label: 'Code' },
              { key: 'preview', icon: <EyeOutlined />, label: 'Preview' },
              { key: 'run', icon: <PlayCircleOutlined />, label: 'Run' },
              { key: 'memory', icon: <DatabaseOutlined />, label: 'Memory' },
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
