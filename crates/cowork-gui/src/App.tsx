import React, { useEffect, useRef, useState, useMemo, useCallback } from 'react';
import { Layout, Menu, Button, Empty, App as AntApp, Modal, Tag, Spin } from 'antd';
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
import { invoke } from '@tauri-apps/api/core';

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
  PMAgentMessage,
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
  const { message } = AntApp.useApp();
  const { 
    project, 
    iterations, 
    currentIteration, 
    loading, 
    loadProject, 
    loadIterations,
    setCurrentIteration,
    updateCurrentIterationStatus,
    setIsExecuting,
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
    loadPMWelcomeMessage,
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

  // Load PM welcome message when entering PM agent mode (only once)
  useEffect(() => {
    if (chatMode === 'pm_agent' && currentIteration) {
      const pmMessages = useAgentStore.getState().pmMessages;
      if (pmMessages.length === 0) {
        loadPMWelcomeMessage(currentIteration.id);
      }
    }
  }, [chatMode, currentIteration?.id, loadPMWelcomeMessage]);

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
        const iterationId = event.payload as string;
        setProcessing(true);
        setIsExecuting(true);
        updateCurrentIterationStatus('Running');
        setActiveView('chat');
        message.info('Iteration started');
      });

      await listen('iteration_continued', (event) => {
        const iterationId = event.payload as string;
        setProcessing(true);
        setIsExecuting(true);
        updateCurrentIterationStatus('Running');
        setActiveView('chat');
        message.info('Iteration continued');
      });

      await listen('iteration_retrying', (event) => {
        const iterationId = event.payload as string;
        setProcessing(true);
        setIsExecuting(true);
        updateCurrentIterationStatus('Running');
        setActiveView('chat');
        message.info('Retrying iteration...');
      });

      await listen('iteration_completed', (event) => {
        const iterationId = event.payload as string;
        setProcessing(false);
        setIsExecuting(false);
        setCurrentAgent(null);
        setInputRequest(null);
        updateCurrentIterationStatus('Completed');
        loadProject();
        triggerMemoryRefresh();
        triggerKnowledgeRefresh();
        // Clear PM messages and switch to PM Chat for post-delivery interaction
        clearPMMessages();
        setActiveView('pm_chat');
        loadPMWelcomeMessage(iterationId);
        message.success('Iteration completed');
      });

      await listen('iteration_failed', (event) => {
        const [, error] = event.payload as [string, string];
        setProcessing(false);
        setIsExecuting(false);
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
        const { content, agent_name, is_thinking, is_first, is_last } = event.payload as {
          content?: string;
          agent_name?: string;
          is_thinking?: boolean;
          is_first?: boolean;
          is_last?: boolean;
        };
        
        // Handle PM Agent streaming messages separately
        if (agent_name === 'PM Agent') {
          // is_last signal without content means stream ended
          if (is_last && !content) {
            setPMMessages((prev) => {
              const lastMsg = prev[prev.length - 1];
              if (lastMsg?.type === 'pm_agent') {
                return [...prev.slice(0, -1), { ...lastMsg } as PMAgentMessage];
              }
              return prev;
            });
            // Reset processing state when stream ends
            setPmProcessing(false);
            return;
          }
          
          if (!content) return;
          
          setPMMessages((prev) => {
            const lastMsg = prev[prev.length - 1];
            // Create new message if is_first or no existing streaming message
            if (is_first || !lastMsg || lastMsg.type !== 'pm_agent' || !(lastMsg as PMAgentMessage & { isStreaming?: boolean }).isStreaming) {
              return [...prev, {
                type: 'pm_agent' as const,
                content,
                isStreaming: !is_last,
                timestamp: new Date().toISOString(),
              } as PMAgentMessage & { isStreaming?: boolean }];
            }
            // Append to existing streaming message
            return [...prev.slice(0, -1), { 
              ...lastMsg, 
              content: (lastMsg as PMAgentMessage).content + content,
              isStreaming: !is_last,
            } as PMAgentMessage & { isStreaming?: boolean }];
          });
          return;
        }
        
        // Handle Pipeline Agent streaming messages
        if (!content) return;
        const msgType = is_thinking ? 'thinking' : 'agent';

        setMessages((prev) => {
          const lastMsg = prev[prev.length - 1];
          if (lastMsg?.type === msgType && (lastMsg as { isStreaming?: boolean }).isStreaming && (lastMsg as { agentName?: string }).agentName === agent_name) {
            return [...prev.slice(0, -1), { ...lastMsg, content: (lastMsg as { content: string }).content + content, isStreaming: !is_last } as ChatMessage];
          }
          return [...prev, {
            type: msgType,
            content,
            agentName: agent_name || 'AI Agent',
            isStreaming: !is_last,
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

      await listen('pm_actions', (event) => {
        const { actions } = event.payload as {
          actions: PMAction[];
        };
        
        // Add actions to the last PM Agent message
        setPMMessages((prev) => {
          const lastMsg = prev[prev.length - 1];
          if (lastMsg?.type === 'pm_agent') {
            return [...prev.slice(0, -1), { 
              ...lastMsg, 
              actions: [...((lastMsg as PMAgentMessage).actions || []), ...actions] 
            } as PMAgentMessage];
          }
          return prev;
        });
      });

      await listen('input_request', async (event) => {
        const [requestId, prompt, options] = event.payload as [string, string, InputOption[]];
        
        updateCurrentIterationStatus('Paused');
        
        const artifactMatch = prompt.match(/\[ARTIFACT_TYPE:(\w+)\]$/);
        if (artifactMatch) {
          const artifactType = artifactMatch[1];
          const cleanPrompt = prompt.replace(/\[ARTIFACT_TYPE:\w+\]$/, '').trim();
          
          // Refresh iterations and set current to latest (for new iteration created by PM Agent)
          await loadIterations();
          const latestIterations = useProjectStore.getState().iterations;
          if (latestIterations && latestIterations.length > 0) {
            const latestIteration = latestIterations[latestIterations.length - 1];
            const fullIteration = await API.iteration.get(latestIteration.id);
            setCurrentIteration(fullIteration);
          }
          
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
      const { currentIteration, isExecuting } = useProjectStore.getState();
      API.iteration.get(iterationId).then((full) => {
        if (isExecuting && currentIteration?.id === iterationId) {
          setCurrentIteration({ ...full, status: currentIteration.status });
        } else {
          setCurrentIteration(full);
        }
      });
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
    console.log('[App] handlePMSendMessage called, userInput:', userInput, 'currentIteration:', currentIteration?.id);
    if (!userInput.trim() || !currentIteration) {
      console.log('[App] handlePMSendMessage early return: no input or no iteration');
      return;
    }
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
    
    if (option.id === 'yes') {
      updateCurrentIterationStatus('Running');
    }
    
    setUserInput('');
  }, [inputRequest, addMessage, submitInput, setActiveView, setActiveArtifactTab, triggerCodeRefresh, triggerArtifactsRefresh, setInputRequest, updateCurrentIterationStatus]);

  const handleSubmitFeedback = useCallback(async () => {
    if (!inputRequest || !userInput.trim()) return;
    const feedback = userInput.trim();
    addMessage({ type: 'agent', content: '📝 Feedback received. Regenerating...', agentName: 'System', timestamp: new Date().toISOString() } as ChatMessage);
    addMessage({ type: 'user', content: `💬 Feedback:\n${feedback}`, timestamp: new Date().toISOString() } as ChatMessage);
    await submitInput(feedback, 'text');
    updateCurrentIterationStatus('Running');
    setUserInput('');
  }, [inputRequest, userInput, addMessage, submitInput, updateCurrentIterationStatus]);

  const handleToggleThinking = useCallback((index: number) => {
    setMessages((prev) => prev.map((m, i) => 
      i === index && m.type === 'thinking' 
        ? { ...m, isExpanded: !(m as ThinkingMessage).isExpanded } as ChatMessage 
        : m
    ));
  }, [setMessages]);

  // Build feedback text from PM Chat user messages.
  // This will be saved to storage and loaded by the coding stage.
  const buildPMFeedback = useCallback((
    msgs: (ChatMessage & { type: 'user' | 'pm_agent' })[],
    targetStage: string
  ): string => {
    console.log('[PM] buildPMFeedback called with', msgs.length, 'messages, targetStage:', targetStage);
    
    // Extract only user messages - these are the actual issues to fix
    const userMessages = msgs
      .filter(msg => msg.type === 'user')
      .map(msg => (msg as { content: string }).content)
      .filter(content => content && content.trim());
    
    if (userMessages.length === 0) return '';
    
    const result = userMessages.join('\n\n');
    console.log('[PM] buildPMFeedback result length:', result.length);
    return result;
  }, []);

  const handlePMAction = useCallback(async (action: PMAction) => {
    if (!currentIteration) return;
    
    switch (action.action_type) {
      case 'pm_start_app':
        setActiveView('run');
        message.info('Starting application...');
        // Check if already running first
        try {
          const isRunning = await invoke<boolean>('check_project_status', { iterationId: currentIteration.id });
          if (isRunning) {
            message.info('Application is already running');
            return;
          }
        } catch {}
        
        try {
          await invoke('start_iteration_project', { iterationId: currentIteration.id });
        } catch (err) {
          message.error('Failed to start app: ' + err);
        }
        break;
        
      case 'pm_open_folder':
        try {
          await invoke('open_in_file_manager', { path: `workspace_${currentIteration.id}` });
        } catch (err) {
          message.error('Failed to open folder: ' + err);
        }
        break;
        
      case 'pm_view_knowledge':
        setActiveView('project-knowledge');
        break;
        
      case 'pm_view_artifacts':
        setActiveView('artifacts');
        setActiveArtifactTab('design');
        break;
        
      case 'pm_view_code':
        setActiveView('code');
        break;
        
      case 'pm_goto_stage':
        if (action.target_stage) {
          console.log('[PM] pm_goto_stage action received, target_stage:', action.target_stage);
          console.log('[PM] Current pmMessages:', pmMessages);
          console.log('[PM] pmMessages length:', pmMessages.length);
          
          // Capture the full PM conversation as feedback before showing the modal
          const feedbackText = buildPMFeedback(
            pmMessages as (ChatMessage & { type: 'user' | 'pm_agent' })[],
            action.target_stage
          );
          console.log('[PM] buildPMFeedback result:', feedbackText ? `${feedbackText.length} chars: ${feedbackText.substring(0, 100)}...` : 'empty');
          Modal.confirm({
            title: 'Confirm Stage Return',
            content: `Return to ${action.target_stage} stage? Your PM Chat conversation will be passed as feedback to the agent.`,
            onOk: async () => {
              try {
                console.log('[PM] User confirmed, calling API.pm.restart with feedback:', feedbackText ? `${feedbackText.length} chars` : 'none');
                // Clear PM messages so next PM Chat session starts fresh
                clearPMMessages();
                // Pass feedback only if it has content (don't convert empty string to undefined)
                await API.pm.restart(currentIteration.id, action.target_stage!, feedbackText.length > 0 ? feedbackText : undefined);
                message.success(`Restarted from ${action.target_stage}`);
                loadProject();
              } catch (err) {
                message.error('Failed: ' + err);
              }
            },
          });
        }
        break;
        
      case 'pm_create_iteration':
        if (action.iteration_id) {
          Modal.confirm({
            title: '启动新迭代',
            content: `是否启动新迭代「${action.title || 'Untitled'}」？`,
            onOk: async () => {
              try {
                // Clear PM messages for new iteration
                clearPMMessages();
                // Clear pipeline messages
                setMessages([]);
                // Load project to refresh iterations list
                await loadProject();
                // Get and set the new iteration as current
                const newIteration = await API.iteration.get(action.iteration_id!);
                setCurrentIteration(newIteration);
                // Execute the new iteration
                await API.iteration.execute(action.iteration_id!);
                message.success('新迭代已启动');
                setActiveView('chat');
              } catch (err) {
                message.error('启动失败: ' + err);
              }
            },
          });
        }
        break;
        
      default:
        console.log('Unknown PM action:', action);
    }
  }, [currentIteration, loadProject, setActiveView, setActiveArtifactTab, clearPMMessages, setMessages, setCurrentIteration, pmMessages, buildPMFeedback]);

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
