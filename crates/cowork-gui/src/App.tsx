import React, { useEffect, useRef, useState, useMemo, useCallback } from 'react';
import { Layout, Menu, Button, Empty, message, Modal, Tag, Spin } from 'antd';
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
  BookOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import { listen } from '@tauri-apps/api/event';

import { useProjectStore, useAgentStore, useUIStore } from './stores';
import { LoadingScreen, StatusBadge } from './components/common';
import { ChatPanel } from './components/chat';
import API from './api';

import type { 
  Iteration, 
  ChatMessage, 
  ThinkingMessage,
  InputOption, 
  ChatMode,
  PMAction,
} from './stores';

import ArtifactsViewer from './components/ArtifactsViewer';
import CodeEditor from './components/CodeEditor';
import RunnerPanel from './components/RunnerPanel';
import ProjectsPanel from './components/ProjectsPanel';
import MemoryPanel from './components/MemoryPanel';
import KnowledgePanel from './components/KnowledgePanel';
import CommandPalette from './components/CommandPalette';
import IterationsPanel from './components/IterationsPanel';
import SettingsPanel from './components/SettingsPanel';

const { Sider, Content, Header, Footer } = Layout;

function App() {
  const { 
    project, 
    iterations, 
    currentIteration, 
    loading, 
    loadProject, 
    setCurrentIteration,
    updateCurrentIterationStatus,
    clearProject,
  } = useProjectStore();
  
  const {
    messages,
    pmMessages,
    isProcessing,
    currentAgent,
    inputRequest,
    pmProcessing,
    addMessage,
    setMessages,
    clearMessages,
    setPMMessages,
    clearPMMessages,
    setProcessing,
    setCurrentAgent,
    setInputRequest,
    setPmProcessing,
    submitInput,
    sendPMMessage,
  } = useAgentStore();
  
  const {
    activeView,
    commandPaletteVisible,
    activeArtifactTab,
    artifactsRefreshTrigger,
    codeRefreshTrigger,
    memoryRefreshTrigger,
    knowledgeRefreshTrigger,
    setActiveView,
    setCommandPaletteVisible,
    setActiveArtifactTab,
    triggerArtifactsRefresh,
    triggerCodeRefresh,
    triggerMemoryRefresh,
    triggerKnowledgeRefresh,
  } = useUIStore();

  const [userInput, setUserInput] = useState('');
  const listenersRegistered = useRef(false);
  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const pmMessagesContainerRef = useRef<HTMLDivElement>(null);

  const chatMode = useMemo<ChatMode>(() => {
    if (!currentIteration) return 'disabled';
    if (currentIteration.status === 'Completed') return 'pm_agent';
    if (isProcessing || currentIteration.status === 'Running') return 'pipeline';
    return 'pipeline';
  }, [currentIteration, isProcessing]);

  // Load initial data
  useEffect(() => {
    loadProject();

    const setupListeners = async () => {
      if (listenersRegistered.current) return;
      listenersRegistered.current = true;

      await listen('iteration_created', () => {
        loadProject();
        message.success('Iteration created');
      });

      await listen('iteration_started', (event) => {
        setProcessing(true);
        updateCurrentIterationStatus('Running');
        message.info('Iteration started');
      });

      await listen('iteration_completed', () => {
        setProcessing(false);
        setCurrentAgent(null);
        setInputRequest(null);
        updateCurrentIterationStatus('Completed');
        loadProject();
        triggerMemoryRefresh();
        triggerKnowledgeRefresh();
        message.success('Iteration completed');
      });

      await listen('iteration_failed', (event) => {
        const [, error] = event.payload as [string, string];
        setProcessing(false);
        setCurrentAgent(null);
        setInputRequest(null);
        updateCurrentIterationStatus('Failed');
        loadProject();
        message.error('Iteration failed: ' + error);
      });

      await listen('agent_event', (event) => {
        const { content, agent_name, message_type, stage_name, level } = event.payload as {
          content?: string;
          agent_name?: string;
          message_type?: string;
          stage_name?: string;
          level?: string;
        };
        
        if (agent_name) setCurrentAgent(agent_name);
        if (!content) return;

        setMessages((prev) => {
          const lastMsg = prev[prev.length - 1];
          const isThinking = message_type === 'thinking';

          if (isThinking) {
            if (lastMsg?.type === 'thinking' && (lastMsg as ThinkingMessage).isStreaming && (lastMsg as ThinkingMessage).agentName === agent_name) {
              return [...prev.slice(0, -1), { ...lastMsg, content: (lastMsg as ThinkingMessage).content + content } as ChatMessage];
            }
            return [...prev, {
              type: 'thinking',
              content,
              agentName: agent_name || 'AI Agent',
              stageName: stage_name,
              isStreaming: true,
              isExpanded: false,
              timestamp: new Date().toISOString(),
            } as ThinkingMessage];
          } else {
            if (lastMsg?.type === 'agent' && (lastMsg as { isStreaming?: boolean }).isStreaming && (lastMsg as { agentName?: string }).agentName === agent_name) {
              return [...prev.slice(0, -1), { ...lastMsg, content: (lastMsg as { content: string }).content + content } as ChatMessage];
            }
            return [...prev, {
              type: 'agent',
              content,
              agentName: agent_name || 'AI Agent',
              stageName: stage_name,
              level,
              isStreaming: true,
              timestamp: new Date().toISOString(),
            } as ChatMessage];
          }
        });
      });

      await listen('agent_streaming', (event) => {
        const { content, agent_name, is_thinking } = event.payload as {
          content?: string;
          agent_name?: string;
          is_thinking?: boolean;
        };
        
        if (!content) return;
        const msgType = is_thinking ? 'thinking' : 'agent';

        setMessages((prev) => {
          const lastMsg = prev[prev.length - 1];
          if (lastMsg?.type === msgType && (lastMsg as { isStreaming?: boolean }).isStreaming && (lastMsg as { agentName?: string }).agentName === agent_name) {
            return [...prev.slice(0, -1), { ...lastMsg, content: (lastMsg as { content: string }).content + content } as ChatMessage];
          }
          return [...prev, {
            type: msgType,
            content,
            agentName: agent_name || 'AI Agent',
            isStreaming: true,
            isExpanded: false,
            timestamp: new Date().toISOString(),
          } as ChatMessage];
        });
      });

      await listen('tool_call', (event) => {
        const { tool_name, arguments: args, agent_name } = event.payload as {
          tool_name: string;
          arguments: Record<string, unknown>;
          agent_name?: string;
        };
        addMessage({
          type: 'tool_call',
          toolName: tool_name,
          arguments: args,
          agentName: agent_name || 'AI Agent',
          timestamp: new Date().toISOString(),
        } as ChatMessage);
      });

      await listen('tool_result', (event) => {
        const { tool_name, result, success, agent_name } = event.payload as {
          tool_name: string;
          result: string;
          success: boolean;
          agent_name?: string;
        };
        addMessage({
          type: 'tool_result',
          toolName: tool_name,
          result,
          success,
          agentName: agent_name || 'AI Agent',
          timestamp: new Date().toISOString(),
        } as ChatMessage);
      });

      await listen('input_request', (event) => {
        const [requestId, prompt, options] = event.payload as [string, string, InputOption[]];
        const artifactMatch = prompt.match(/\[ARTIFACT_TYPE:(\w+)\]$/);
        if (artifactMatch) {
          const artifactType = artifactMatch[1];
          const cleanPrompt = prompt.replace(/\[ARTIFACT_TYPE:\w+\]$/, '').trim();
          setInputRequest({
            requestId,
            prompt: cleanPrompt,
            options,
            isArtifactConfirmation: true,
            artifactType,
          });
        } else {
          setInputRequest({ requestId, prompt, options });
        }
        setUserInput('');
      });

      await listen('project_loaded', () => {
        setProcessing(false);
        setCurrentAgent(null);
        setInputRequest(null);
        clearMessages();
        setCurrentIteration(null);
        setActiveView('iterations');
        loadProject();
        message.success('Project loaded');
      });

      await listen('project_initialized', () => {
        setProcessing(false);
        setCurrentAgent(null);
        setInputRequest(null);
        clearMessages();
        setCurrentIteration(null);
        setActiveView('iterations');
        loadProject();
        message.success('Project initialized');
      });

      await listen('knowledge_regeneration_completed', () => {
        triggerKnowledgeRefresh();
        message.success('Knowledge updated');
      });
    };

    setupListeners();

    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setCommandPaletteVisible(!commandPaletteVisible);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Auto-scroll
  useEffect(() => {
    if (messagesContainerRef.current) {
      messagesContainerRef.current.scrollTop = messagesContainerRef.current.scrollHeight;
    }
  }, [messages]);

  useEffect(() => {
    if (pmMessagesContainerRef.current && pmMessages.length > 0) {
      pmMessagesContainerRef.current.scrollTop = pmMessagesContainerRef.current.scrollHeight;
    }
  }, [pmMessages]);

  const handleSelectIteration = useCallback((iterationId: string) => {
    const iteration = iterations.find((i) => i.id === iterationId);
    if (iteration) {
      API.iteration.get(iterationId).then((full) => setCurrentIteration(full));
      setActiveView('chat');
    }
  }, [iterations, setCurrentIteration, setActiveView]);

  const handleExecuteIteration = useCallback(async () => {
    if (!currentIteration) return;
    try {
      setProcessing(true);
      await API.iteration.execute(currentIteration.id);
      message.info('Iteration execution started');
    } catch (error) {
      message.error('Failed to execute iteration: ' + error);
      setProcessing(false);
    }
  }, [currentIteration, setProcessing]);

  const handleSendUserMessage = useCallback(async () => {
    if (!userInput.trim()) return;
    const msgContent = userInput;
    addMessage({ type: 'user', content: msgContent, timestamp: new Date().toISOString() } as ChatMessage);

    if (inputRequest) {
      await submitInput(msgContent, 'text');
    }
    setUserInput('');
  }, [userInput, inputRequest, addMessage, submitInput]);

  const handlePMSendMessage = useCallback(async () => {
    if (!userInput.trim() || !currentIteration) return;
    const userMessage = userInput.trim();
    setUserInput('');
    setPmProcessing(true);

    try {
      await sendPMMessage(currentIteration.id, userMessage);
    } catch (error) {
      message.error('Failed to process message: ' + error);
    } finally {
      setPmProcessing(false);
    }
  }, [userInput, currentIteration, sendPMMessage, setPmProcessing]);

  const handleSelectOption = useCallback(async (option: InputOption) => {
    if (!inputRequest) return;

    if (option.id === 'view_artifact' && inputRequest.isArtifactConfirmation) {
      const artifactTypeToTab: Record<string, string> = {
        idea: 'idea', requirements: 'requirements', design: 'design', plan: 'plan', code: 'code',
      };
      const targetTab = artifactTypeToTab[inputRequest.artifactType || ''] || 'idea';
      setActiveArtifactTab(targetTab);

      if (inputRequest.artifactType === 'code') {
        setActiveView('code');
        triggerCodeRefresh();
        message.info('Switched to Code tab to review code files');
      } else {
        setActiveView('artifacts');
        triggerArtifactsRefresh();
        message.info(`Switched to Artifacts tab to review ${inputRequest.artifactType}`);
      }
      return;
    }

    if (option.id === 'feedback' && inputRequest.isArtifactConfirmation) {
      setInputRequest({ ...inputRequest, isFeedbackMode: true, feedbackPrompt: 'Please enter your feedback:' });
      setUserInput('');
      return;
    }

    addMessage({ type: 'user', content: option.label, timestamp: new Date().toISOString() } as ChatMessage);
    await submitInput(option.id, 'selection');
    setUserInput('');
  }, [inputRequest, addMessage, submitInput, setActiveView, setActiveArtifactTab, triggerCodeRefresh, triggerArtifactsRefresh, setInputRequest]);

  const handleSubmitFeedback = useCallback(async () => {
    if (!inputRequest || !userInput.trim()) return;
    const feedback = userInput.trim();
    addMessage({ type: 'agent', content: '📝 Feedback received. Regenerating...', agentName: 'System', timestamp: new Date().toISOString() } as ChatMessage);
    addMessage({ type: 'user', content: `💬 Feedback:\n${feedback}`, timestamp: new Date().toISOString() } as ChatMessage);
    await submitInput(feedback, 'text');
    setUserInput('');
  }, [inputRequest, userInput, addMessage, submitInput]);

  const handleToggleThinking = useCallback((index: number) => {
    setMessages((prev) => prev.map((m, i) => 
      i === index && m.type === 'thinking' 
        ? { ...m, isExpanded: !(m as ThinkingMessage).isExpanded } as ChatMessage 
        : m
    ));
  }, [setMessages]);

  const handlePMAction = useCallback(async (action: PMAction) => {
    if (!currentIteration) return;
    
    if (action.action_type === 'pm_goto_stage' && action.target_stage) {
      Modal.confirm({
        title: 'Confirm Stage Return',
        content: `Return to ${action.target_stage} stage?`,
        onOk: async () => {
          try {
            await API.pm.restart(currentIteration.id, action.target_stage!);
            message.success(`Restarted from ${action.target_stage}`);
            loadProject();
          } catch (err) {
            message.error('Failed: ' + err);
          }
        },
      });
    }
  }, [currentIteration, loadProject]);

  const handleOpenProjectFolder = useCallback(async () => {
    try {
      await API.util.openInFileManager('.');
    } catch (error) {
      message.error('Failed to open project folder');
    }
  }, []);

  const handleCommandSelect = useCallback((commandId: string) => {
    const viewMap: Record<string, string> = {
      'view-iterations': 'iterations',
      'view-chat': 'chat',
      'view-artifacts': 'artifacts',
      'view-code': 'code',
      'view-run': 'run',
      'view-memory': 'execution-memory',
      'view-projects': 'projects',
      'view-settings': 'settings',
    };
    if (viewMap[commandId]) {
      setActiveView(viewMap[commandId] as typeof activeView);
    }
  }, [setActiveView]);

  const renderContent = () => (
    <div style={{ height: '100%' }}>
      <div style={{ height: '100%', display: activeView === 'iterations' ? 'block' : 'none' }}>
        <IterationsPanel
          key="iterations"
          onSelectIteration={handleSelectIteration}
          selectedIterationId={currentIteration?.id}
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
            refreshTrigger={artifactsRefreshTrigger}
          />
        ) : (
          <Empty description="Select an iteration" style={{ marginTop: '40px' }} />
        )}
      </div>

      <div style={{ height: '100%', display: activeView === 'code' ? 'block' : 'none' }}>
        {currentIteration ? (
          <CodeEditor
            key={`code-${currentIteration.id}`}
            iterationId={currentIteration.id}
            refreshTrigger={codeRefreshTrigger}
          />
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

      <div style={{ height: '100%', display: activeView === 'execution-memory' ? 'block' : 'none' }}>
        <MemoryPanel
          key={`memory-${memoryRefreshTrigger}`}
          currentSession={currentIteration?.id}
          refreshTrigger={memoryRefreshTrigger}
        />
      </div>

      <div style={{ height: '100%', display: activeView === 'project-knowledge' ? 'block' : 'none' }}>
        <KnowledgePanel
          key={`knowledge-${knowledgeRefreshTrigger}`}
          currentSession={project?.id}
          refreshTrigger={knowledgeRefreshTrigger}
        />
      </div>

      <div style={{ height: '100%', display: activeView === 'settings' ? 'block' : 'none', overflow: 'auto' }}>
        <SettingsPanel />
      </div>

      <div style={{ height: '100%', display: activeView === 'chat' ? 'block' : 'none' }}>
        {currentIteration ? (
          <ChatPanel
            messages={messages}
            pmMessages={pmMessages as (ChatMessage & { type: 'user' | 'pm_agent' })[]}
            mode={chatMode}
            isProcessing={isProcessing}
            pmProcessing={pmProcessing}
            currentAgent={currentAgent}
            iterationTitle={currentIteration.title}
            iterationDescription={currentIteration.description}
            currentStage={currentIteration.current_stage}
            inputRequest={inputRequest}
            userInput={userInput}
            messagesContainerRef={messagesContainerRef as React.RefObject<HTMLDivElement>}
            pmMessagesContainerRef={pmMessagesContainerRef as React.RefObject<HTMLDivElement>}
            onUserInputChange={setUserInput}
            onSend={chatMode === 'pm_agent' ? handlePMSendMessage : handleSendUserMessage}
            onSelectOption={handleSelectOption}
            onSubmitFeedback={handleSubmitFeedback}
            onCancelFeedback={() => inputRequest && setInputRequest({ ...inputRequest, isFeedbackMode: false })}
            onToggleThinking={handleToggleThinking}
            onActionClick={handlePMAction}
          />
        ) : (
          <Empty description="Select an iteration to view chat" style={{ marginTop: '40px' }} />
        )}
      </div>
    </div>
  );

  if (loading) {
    return <LoadingScreen />;
  }

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header
        style={{
          background: '#fff',
          borderBottom: '1px solid #e8e8e8',
          padding: '0 24px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <h1 style={{ margin: 0, fontSize: '18px' }}>
            <RocketOutlined style={{ marginRight: '8px', color: '#1890ff' }} />
            Cowork Forge
          </h1>
          {project && (
            <Tag color="blue" style={{ cursor: 'pointer' }} onClick={handleOpenProjectFolder}>
              {project.name}
            </Tag>
          )}
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          {currentIteration && (
            <>
              <StatusBadge status={currentIteration.status} />
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
            onClick={({ key }) => setActiveView(key as typeof activeView)}
            style={{ height: '100%', borderRight: 0 }}
            items={[
              { key: 'projects', icon: <AppstoreOutlined />, label: 'Projects' },
              { key: 'iterations', icon: <BranchesOutlined />, label: 'Iterations' },
              { key: 'chat', icon: <MessageOutlined />, label: 'Collaborate' },
              { key: 'artifacts', icon: <FileTextOutlined />, label: 'Artifacts' },
              { key: 'code', icon: <CodeOutlined />, label: 'Code' },
              { key: 'run', icon: <PlayCircleOutlined />, label: 'Run' },
              { key: 'execution-memory', icon: <DatabaseOutlined />, label: 'Memory' },
              { key: 'project-knowledge', icon: <BookOutlined />, label: 'Knowledge' },
              { type: 'divider' },
              { key: 'settings', icon: <SettingOutlined />, label: 'Settings' },
            ]}
          />
        </Sider>

        <Content style={{ overflow: 'hidden', height: '100%', display: 'flex', flexDirection: 'column' }}>
          {renderContent()}
        </Content>
      </Layout>

      <Footer
        style={{
          background: '#fff',
          borderTop: '1px solid #e8e8e8',
          padding: '12px 24px',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <div style={{ fontSize: '12px', color: '#888' }}>
          {project ? (
            <>
              <span style={{ marginRight: '16px', cursor: 'pointer' }} onClick={handleOpenProjectFolder}>
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
        visible={commandPaletteVisible}
        onClose={() => setCommandPaletteVisible(false)}
        onCommandSelect={handleCommandSelect}
      />
    </Layout>
  );
}

export default App;
