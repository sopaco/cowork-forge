import React, { useState, useEffect, useRef, useMemo } from 'react';
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
  BookOutlined,
  TeamOutlined,
} from '@ant-design/icons';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';
import 'highlight.js/styles/github.css';
import ArtifactsViewer from './components/ArtifactsViewer';
import CodeEditor from './components/CodeEditor';
import RunnerPanel from './components/RunnerPanel';
import ProjectsPanel from './components/ProjectsPanel';
import MemoryPanel from './components/MemoryPanel';
import KnowledgePanel from './components/KnowledgePanel';
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

// Markdown message component for rendering AI agent responses
function MarkdownMessage({ content }) {
  return (
    <div style={{
      lineHeight: '1.6',
      fontSize: '14px',
    }}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight, rehypeRaw]}
        components={{
          // Style for code blocks
          code({ node, inline, className, children, ...props }) {
            const match = /language-(\w+)/.exec(className || '');
            return !inline ? (
              <div style={{
                backgroundColor: '#f6f8fa',
                borderRadius: '6px',
                padding: '12px',
                margin: '8px 0',
                overflowX: 'auto',
                border: '1px solid #e1e4e8',
              }}>
                <code className={className} {...props} style={{ fontSize: '13px' }}>
                  {children}
                </code>
              </div>
            ) : (
              <code style={{
                backgroundColor: '#f6f8fa',
                padding: '2px 6px',
                borderRadius: '3px',
                fontSize: '0.9em',
                fontFamily: 'Consolas, Monaco, "Courier New", monospace',
              }} {...props}>
                {children}
              </code>
            );
          },
          // Style for blockquotes
          blockquote({ children }) {
            return (
              <blockquote style={{
                borderLeft: '4px solid #dfe2e5',
                margin: '8px 0',
                padding: '8px 16px',
                backgroundColor: '#f6f8fa',
                color: '#6a737d',
              }}>
                {children}
              </blockquote>
            );
          },
          // Style for headings
          h1({ children }) {
            return <h1 style={{ fontSize: '1.5em', fontWeight: 600, marginBottom: '0.5em', marginTop: '1em' }}>{children}</h1>;
          },
          h2({ children }) {
            return <h2 style={{ fontSize: '1.3em', fontWeight: 600, marginBottom: '0.5em', marginTop: '0.8em' }}>{children}</h2>;
          },
          h3({ children }) {
            return <h3 style={{ fontSize: '1.1em', fontWeight: 600, marginBottom: '0.5em', marginTop: '0.6em' }}>{children}</h3>;
          },
          // Style for lists
          ul({ children }) {
            return <ul style={{ paddingLeft: '20px', margin: '8px 0' }}>{children}</ul>;
          },
          ol({ children }) {
            return <ol style={{ paddingLeft: '20px', margin: '8px 0' }}>{children}</ol>;
          },
          li({ children }) {
            return <li style={{ marginBottom: '4px' }}>{children}</li>;
          },
          // Style for links
          a({ children, href }) {
            return (
              <a href={href} target="_blank" rel="noopener noreferrer" style={{ color: '#1890ff', textDecoration: 'underline' }}>
                {children}
              </a>
            );
          },
          // Style for tables
          table({ children }) {
            return (
              <table style={{
                width: '100%',
                borderCollapse: 'collapse',
                margin: '12px 0',
                fontSize: '13px',
              }}>
                {children}
              </table>
            );
          },
          thead({ children }) {
            return <thead style={{ backgroundColor: '#f6f8fa' }}>{children}</thead>;
          },
          th({ children }) {
            return <th style={{
              padding: '8px 12px',
              textAlign: 'left',
              borderBottom: '2px solid #e1e4e8',
              fontWeight: 600,
            }}>{children}</th>;
          },
          td({ children }) {
            return <td style={{
              padding: '8px 12px',
              borderBottom: '1px solid #e1e4e8',
            }}>{children}</td>;
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}

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
  const [artifactsRefreshTrigger, setArtifactsRefreshTrigger] = useState(0);
  const [codeRefreshTrigger, setCodeRefreshTrigger] = useState(0);
  const [memoryRefreshTrigger, setMemoryRefreshTrigger] = useState(0);
  const [knowledgeRefreshTrigger, setKnowledgeRefreshTrigger] = useState(0);
  
  // PM Agent states
  const [pmMessages, setPmMessages] = useState([]);
  const [pmProcessing, setPmProcessing] = useState(false);
  const pmMessagesContainerRef = useRef(null);
  
  const listenersRegistered = useRef(false);
  const messagesContainerRef = useRef(null);
  
  // Compute chat mode based on iteration status
  const chatMode = useMemo(() => {
    if (!currentIteration) {
      return 'disabled';
    }
    
    if (currentIteration.status === 'Completed') {
      return 'pm_agent';  // Delivery 后，启用项目经理 Agent
    }
    
    if (isProcessing || currentIteration.status === 'Running') {
      return 'pipeline';  // Pipeline 执行中，显示 Agent 消息流
    }
    
    return 'pipeline';  // Draft/Paused/Failed 状态
  }, [currentIteration, isProcessing]);

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
        // Trigger refresh for Memory and Knowledge panels
        setMemoryRefreshTrigger(prev => prev + 1);
        setKnowledgeRefreshTrigger(prev => prev + 1);
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

      // Main agent event handler - updated to use actual agent_name from backend
      await listen('agent_event', (event) => {
        const { content, agent_name, message_type, stage_name, level } = event.payload;
        // Track current agent for processing display (now using real agent name)
        if (agent_name) {
          setCurrentAgent(agent_name);
        }

        if (!content) return;

        setMessages((prev) => {
          const lastMsg = prev[prev.length - 1];
          const isThinking = message_type === 'thinking';

          if (isThinking) {
            // Handle thinking messages
            if (lastMsg && lastMsg.type === 'thinking' && lastMsg.isStreaming && lastMsg.agentName === agent_name) {
              // Append to existing thinking message
              return [
                ...prev.slice(0, -1),
                {
                  ...lastMsg,
                  content: lastMsg.content + content,
                },
              ];
            } else {
              // Create new thinking message
              return [
                ...prev,
                {
                  type: 'thinking',
                  content,
                  agentName: agent_name || 'AI Agent',
                  stageName: stage_name,
                  isStreaming: true,
                  isExpanded: false,
                  timestamp: new Date().toISOString(),
                },
              ];
            }
          } else {
            // Handle regular agent messages
            if (lastMsg && lastMsg.type === 'agent' && lastMsg.isStreaming && lastMsg.agentName === agent_name) {
              return [
                ...prev.slice(0, -1),
                {
                  ...lastMsg,
                  content: lastMsg.content + content,
                },
              ];
            } else {
              return [
                ...prev,
                {
                  type: 'agent',
                  content,
                  agentName: agent_name || 'AI Agent',
                  stageName: stage_name,
                  level: level,
                  isStreaming: true,
                  timestamp: new Date().toISOString(),
                },
              ];
            }
          }
        });
      });

      // Streaming content handler - for real-time token output
      await listen('agent_streaming', (event) => {
        const { content, agent_name, is_thinking } = event.payload;

        if (!content) return;

        setMessages((prev) => {
          const lastMsg = prev[prev.length - 1];
          const msgType = is_thinking ? 'thinking' : 'agent';

          if (lastMsg && lastMsg.type === msgType && lastMsg.isStreaming && lastMsg.agentName === agent_name) {
            // Append to existing streaming message
            return [
              ...prev.slice(0, -1),
              {
                ...lastMsg,
                content: lastMsg.content + content,
              },
            ];
          } else {
            // Create new streaming message
            return [
              ...prev,
              {
                type: msgType,
                content,
                agentName: agent_name || 'AI Agent',
                isStreaming: true,
                isExpanded: false,
                timestamp: new Date().toISOString(),
              },
            ];
          }
        });
      });

      // Tool call handler - shows when agent calls a tool
      await listen('tool_call', (event) => {
        const { tool_name, arguments: toolArguments, agent_name } = event.payload;

        setMessages((prev) => [
          ...prev,
          {
            type: 'tool_call',
            toolName: tool_name,
            arguments: toolArguments,
            agentName: agent_name || 'AI Agent',
            timestamp: new Date().toISOString(),
          },
        ]);
      });

      // Tool result handler - shows tool execution result
      await listen('tool_result', (event) => {
        const { tool_name, result, success, agent_name } = event.payload;

        setMessages((prev) => [
          ...prev,
          {
            type: 'tool_result',
            toolName: tool_name,
            result: result,
            success: success,
            agentName: agent_name || 'AI Agent',
            timestamp: new Date().toISOString(),
          },
        ]);
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

      // Listen for knowledge regeneration events
      await listen('knowledge_regeneration_completed', () => {
        setKnowledgeRefreshTrigger(prev => prev + 1);
        message.success('Knowledge updated');
      });

      await listen('knowledge_regeneration_failed', (event) => {
        const [, error] = event.payload;
        message.error('Knowledge update failed: ' + error);
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

  // Auto-scroll PM Agent messages
  useEffect(() => {
    if (pmMessagesContainerRef.current && pmMessages.length > 0) {
      pmMessagesContainerRef.current.scrollTop = pmMessagesContainerRef.current.scrollHeight;
    }
  }, [pmMessages]);

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

  const handleOpenProjectFolder = async () => {
    try {
      await invoke('open_in_file_manager', { path: '.' });
    } catch (error) {
      console.error('Failed to open project folder:', error);
      message.error('Failed to open project folder');
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

  // PM Agent message handler
  const handlePMSendMessage = async () => {
    if (!userInput.trim() || !currentIteration) return;

    const userMessage = userInput.trim();
    const userMsgObj = { type: 'user', content: userMessage, timestamp: new Date().toISOString() };
    
    // 构建包含当前消息的历史（解决闭包问题）
    const historyWithCurrent = [...pmMessages, userMsgObj];
    
    setUserInput('');
    setPmProcessing(true);

    try {
      const response = await invoke('pm_send_message', {
        iterationId: currentIteration.id,
        message: userMessage,
        history: historyWithCurrent,
      });

      // 一次更新所有消息（解决竞态问题）
      setPmMessages([
        ...historyWithCurrent,
        {
          type: 'pm_agent',
          content: response.agent_message,
          actions: response.actions || [],
          timestamp: new Date().toISOString(),
        },
      ]);

      // Handle actions that need immediate execution
      if (response.needs_restart && response.target_stage) {
        // Show confirmation for restart
        Modal.confirm({
          title: 'Restart Pipeline',
          content: `Do you want to restart the pipeline from ${response.target_stage} stage?`,
          onOk: async () => {
            try {
              await invoke('pm_restart_iteration', {
                iterationId: currentIteration.id,
                targetStage: response.target_stage,
              });
              message.success(`Pipeline restarted from ${response.target_stage}`);
              loadData();
            } catch (err) {
              message.error('Failed to restart: ' + err);
            }
          },
        });
      }

      if (response.new_iteration_id) {
        message.success(`New iteration created! Switching to it...`);
        loadData();
      }

    } catch (error) {
      message.error('Failed to process message: ' + error);
      setPmMessages((prev) => [
        ...prev,
        {
          type: 'error',
          content: `Error: ${error}`,
          timestamp: new Date().toISOString(),
        },
      ]);
    } finally {
      setPmProcessing(false);
    }
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

      // For code artifact, switch to code view and trigger refresh
      if (inputRequest.artifactType === 'code') {
        setActiveView('code');
        setCodeRefreshTrigger(prev => prev + 1);
        message.info('Switched to Code tab to review code files');
      } else {
        // For other artifacts, switch to artifacts tab and trigger refresh
        setActiveView('artifacts');
        setArtifactsRefreshTrigger(prev => prev + 1);
        message.info(`Switched to Artifacts tab to review ${inputRequest.artifactType}`);
      }

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
        <MemoryPanel key={`memory-${memoryRefreshTrigger}`} currentSession={currentIteration?.id} refreshTrigger={memoryRefreshTrigger} />
      </div>

      <div style={{ height: '100%', display: activeView === 'project-knowledge' ? 'block' : 'none' }}>
        <KnowledgePanel key={`knowledge-${knowledgeRefreshTrigger}`} currentSession={project?.id} refreshTrigger={knowledgeRefreshTrigger} />
      </div>

      <div style={{ height: '100%', display: activeView === 'chat' ? 'block' : 'none' }}>
        {currentIteration ? (
          <div style={{ height: '100%', display: 'flex', flexDirection: 'column', padding: '16px' }}>
            {/* PM Agent 模式头部 */}
            {chatMode === 'pm_agent' ? (
              <div style={{ marginBottom: '16px' }}>
                <h3 style={{ margin: 0 }}>
                  <TeamOutlined style={{ marginRight: '8px' }} />
                  Project Manager Agent
                </h3>
                <p style={{ margin: '4px 0 0 0', color: '#888', fontSize: '12px' }}>
                  {currentIteration.title} - Ask questions, request changes, or discuss next steps
                </p>
                <Tag color="green" style={{ marginTop: '8px' }}>Post-Delivery Chat</Tag>
              </div>
            ) : (
              <>
                <div style={{ marginBottom: '16px' }}>
                  <h3 style={{ margin: 0 }}>Chat - {currentIteration.title}</h3>
                  <p style={{ margin: '4px 0 0 0', color: '#888', fontSize: '12px' }}>
                    {currentIteration.description}
                  </p>
                </div>

                {/* Status Indicator */}
                {isProcessing && currentAgent && (
                  <div style={{
                    padding: '12px 16px',
                    backgroundColor: '#e6f7ff',
                    border: '1px solid #91d5ff',
                    borderRadius: '6px',
                    marginBottom: '16px',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '12px',
                  }}>
                    <Spin size="small" />
                    <div style={{ flex: 1 }}>
                      <div style={{ fontSize: '14px', fontWeight: 500, color: '#1890ff', marginBottom: '4px' }}>
                        {currentAgent} is working...
                      </div>
                      <div style={{ fontSize: '12px', color: '#666' }}>
                        {currentIteration.current_stage ? `Stage: ${currentIteration.current_stage}` : 'Processing...'}
                      </div>
                    </div>
                  </div>
                )}
              </>
            )}
            
            {/* 消息列表区域 */}
            <div 
              ref={chatMode === 'pm_agent' ? pmMessagesContainerRef : messagesContainerRef}
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
              {chatMode === 'pm_agent' ? (
                /* PM Agent 消息列表 */
                pmMessages.length === 0 ? (
                  <div style={{ color: '#888', textAlign: 'center', marginTop: '40px' }}>
                    <div style={{ fontSize: '48px', marginBottom: '16px' }}>👋</div>
                    <div style={{ fontSize: '16px', marginBottom: '8px' }}>Welcome to Project Manager Chat!</div>
                    <div style={{ fontSize: '13px' }}>
                      Ask me anything about this project, request changes, or discuss next steps.
                    </div>
                    <div style={{ marginTop: '24px', fontSize: '12px', color: '#999' }}>
                      <div>I can help you with:</div>
                      <div style={{ marginTop: '8px' }}>• Fix bugs by returning to earlier stages</div>
                      <div>• Add new features through new iterations</div>
                      <div>• Answer questions about the project</div>
                    </div>
                  </div>
                ) : (
                  pmMessages.map((msg, index) => (
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
                      ) : msg.type === 'pm_agent' ? (
                        <div>
                          <div style={{ fontSize: '12px', color: '#888', marginBottom: '4px' }}>
                            <TeamOutlined style={{ marginRight: '4px' }} />
                            Project Manager Agent
                          </div>
                          <div style={{
                            backgroundColor: '#fff',
                            padding: '12px 16px',
                            borderRadius: '4px',
                            border: '1px solid #e8e8e8',
                            maxWidth: '70%',
                            wordBreak: 'break-word',
                          }}>
                            <MarkdownMessage content={msg.content} />
                          </div>
                          {/* 显示操作建议 */}
                          {msg.actions && msg.actions.length > 0 && (
                            <div style={{ marginTop: '12px' }}>
                              {msg.actions.map((action, idx) => (
                                <div key={idx} style={{ 
                                  display: 'inline-block',
                                  marginRight: '8px',
                                  marginBottom: '8px'
                                }}>
                                  {action.action_type === 'pm_goto_stage' && (
                                    <Tag 
                                      color="orange" 
                                      style={{ cursor: 'pointer' }}
                                      onClick={async () => {
                                        Modal.confirm({
                                          title: 'Confirm Stage Return',
                                          content: `Return to ${action.target_stage} stage?`,
                                          onOk: async () => {
                                            try {
                                              await invoke('pm_restart_iteration', {
                                                iterationId: currentIteration.id,
                                                targetStage: action.target_stage,
                                              });
                                              message.success(`Restarted from ${action.target_stage}`);
                                              loadData();
                                            } catch (err) {
                                              message.error('Failed: ' + err);
                                            }
                                          },
                                        });
                                      }}
                                    >
                                      ↩️ Return to {action.target_stage}
                                    </Tag>
                                  )}
                                  {action.action_type === 'pm_create_iteration' && (
                                    <Tag 
                                      color="blue" 
                                      style={{ cursor: 'pointer' }}
                                      onClick={async () => {
                                        try {
                                          await invoke('pm_restart_iteration', {
                                            iterationId: currentIteration.id,
                                            targetStage: 'plan',
                                          });
                                          message.success('New iteration created!');
                                          loadData();
                                        } catch (err) {
                                          message.error('Failed: ' + err);
                                        }
                                      }}
                                    >
                                      ➕ New Iteration
                                    </Tag>
                                  )}
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      ) : (
                        <div style={{ color: '#f44336', padding: '12px', backgroundColor: '#ffebee', borderRadius: '4px' }}>
                          {msg.content}
                        </div>
                      )}
                    </div>
                  ))
                )
              ) : (
                /* Pipeline 消息列表 */
                messages.length === 0 ? (
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
                      ) : msg.type === 'thinking' ? (
                        <div>
                          <div
                            style={{
                              fontSize: '12px',
                              color: '#888',
                              marginBottom: '4px',
                              display: 'flex',
                              alignItems: 'center',
                              gap: '6px',
                              cursor: 'pointer',
                            }}
                            onClick={() => {
                              setMessages((prev) =>
                                prev.map((m, i) =>
                                  i === index ? { ...m, isExpanded: !m.isExpanded } : m
                                )
                              );
                            }}
                          >
                            <span>🤔</span>
                            <span style={{ fontStyle: 'italic' }}>{msg.agentName || 'AI Agent'} thinking...</span>
                            <span style={{ fontSize: '10px' }}>{msg.isExpanded ? '▼' : '▶'}</span>
                          </div>
                          {msg.isExpanded && (
                            <div style={{
                              backgroundColor: '#f6f8fa',
                              padding: '10px 14px',
                              borderRadius: '4px',
                              border: '1px solid #e1e4e8',
                              maxWidth: '70%',
                              wordBreak: 'break-word',
                              fontSize: '13px',
                              fontStyle: 'italic',
                              color: '#555',
                              lineHeight: '1.6',
                            }}>
                              {msg.content}
                            </div>
                          )}
                        </div>
                      ) : msg.type === 'tool_call' ? (
                        <div style={{
                          backgroundColor: '#fff3e0',
                          padding: '8px 12px',
                          borderRadius: '4px',
                          maxWidth: '70%',
                          fontSize: '13px',
                          borderLeft: '3px solid #ff9800',
                        }}>
                          <div style={{ fontWeight: 500, color: '#e65100', marginBottom: '4px' }}>
                            🔧 {msg.agentName} 调用工具: <code style={{ 
                              backgroundColor: 'rgba(0,0,0,0.05)',
                              padding: '1px 4px',
                              borderRadius: '2px',
                              fontSize: '12px'
                            }}>{msg.toolName}</code>
                          </div>
                          {msg.arguments && Object.keys(msg.arguments).length > 0 && (
                            <pre style={{ 
                              margin: '4px 0 0', 
                              fontSize: '11px', 
                              color: '#666',
                              backgroundColor: 'rgba(0,0,0,0.02)',
                              padding: '6px',
                              borderRadius: '3px',
                              overflow: 'auto',
                              maxHeight: '100px',
                            }}>
                              {JSON.stringify(msg.arguments, null, 2)}
                            </pre>
                          )}
                        </div>
                      ) : msg.type === 'tool_result' ? (
                        <div style={{
                          backgroundColor: msg.success ? '#e8f5e9' : '#ffebee',
                          padding: '6px 12px',
                          borderRadius: '4px',
                          maxWidth: '70%',
                          fontSize: '12px',
                          borderLeft: msg.success ? '3px solid #4caf50' : '3px solid #f44336',
                        }}>
                          <span>
                            {msg.success ? '✓' : '✗'} 工具 <code style={{
                              backgroundColor: 'rgba(0,0,0,0.05)',
                              padding: '1px 4px',
                              borderRadius: '2px',
                              fontSize: '11px'
                            }}>{msg.toolName}</code> 执行{msg.success ? '成功' : '失败'}
                          </span>
                        </div>
                      ) : (
                        <div>
                          <div style={{ fontSize: '12px', color: '#888', marginBottom: '4px' }}>
                            {msg.agentName || 'AI Agent'}
                          </div>
                          <div style={{
                            backgroundColor: '#fff',
                            padding: '12px 16px',
                            borderRadius: '4px',
                            border: '1px solid #e8e8e8',
                            maxWidth: '70%',
                            wordBreak: 'break-word',
                          }}>
                            <MarkdownMessage content={msg.content} />
                          </div>
                        </div>
                      )}
                    </div>
                  ))
                )
              )}
            </div>

            {/* 输入区域 */}
            {chatMode === 'pm_agent' ? (
              /* PM Agent 输入区域 */
              <div style={{ display: 'flex', gap: '8px' }}>
                <Input
                  value={userInput}
                  onChange={(e) => setUserInput(e.target.value)}
                  onPressEnter={handlePMSendMessage}
                  placeholder="Ask about the project or request changes..."
                  disabled={pmProcessing}
                />
                <Button 
                  onClick={handlePMSendMessage} 
                  type="primary"
                  disabled={!userInput.trim() || pmProcessing}
                  loading={pmProcessing}
                >
                  Send
                </Button>
              </div>
            ) : (
              /* Pipeline 输入区域 */
              <>
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
              </>
            )}
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
            <Tag 
              color="blue" 
              style={{ cursor: 'pointer' }}
              onClick={handleOpenProjectFolder}
            >
              {project.name}
            </Tag>
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
              { key: 'run', icon: <PlayCircleOutlined />, label: 'Run' },
              { key: 'execution-memory', icon: <DatabaseOutlined />, label: 'Memory' },
              { key: 'project-knowledge', icon: <BookOutlined />, label: 'Knowledge' },
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
            case 'view-run':
              setActiveView('run');
              break;
            case 'view-memory':
              setActiveView('execution-memory');
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
