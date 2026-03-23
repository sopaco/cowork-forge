import React, { memo, useMemo, useCallback } from 'react';
import { Spin, Tag } from 'antd';
import { TeamOutlined } from '@ant-design/icons';
import { MarkdownMessage } from '../common';
import type { ChatMessage, ThinkingMessage, PMAction } from '../../stores';

interface ToolCallMessage extends ChatMessage {
  toolName: string;
  arguments: Record<string, unknown>;
  agentName: string;
}

interface ToolResultMessage extends ChatMessage {
  toolName: string;
  result: string;
  success: boolean;
  agentName: string;
}

interface PMAgentMessage extends ChatMessage {
  actions?: PMAction[];
}

interface MessageListProps {
  messages: ChatMessage[];
  pmMessages?: (ChatMessage & { type: 'user' | 'pm_agent' })[];
  mode: 'pipeline' | 'pm_agent';
  isProcessing: boolean;
  currentAgent: string | null;
  onToggleThinking: (index: number) => void;
  onActionClick?: (action: PMAction) => void;
}

// Helper to generate stable key for messages
const getMessageKey = (msg: ChatMessage, index: number): string => {
  if (msg.type === 'thinking') {
    return `thinking-${index}-${msg.agentName || 'unknown'}`;
  }
  if (msg.type === 'tool_call') {
    return `tool-call-${index}-${(msg as ToolCallMessage).toolName}`;
  }
  if (msg.type === 'tool_result') {
    return `tool-result-${index}-${(msg as ToolResultMessage).toolName}`;
  }
  // For user/agent messages, use timestamp if available, otherwise index
  const timestamp = (msg as { timestamp?: string | number }).timestamp;
  if (timestamp) {
    return `msg-${timestamp}`;
  }
  return `msg-${index}`;
};

// Memoized message item components
const ThinkingMessageItem = memo<{ message: ThinkingMessage; onToggle: () => void }>(({ message, onToggle }) => (
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
      onClick={onToggle}
    >
      <span>🤔</span>
      <span style={{ fontStyle: 'italic' }}>{message.agentName} thinking...</span>
      <span style={{ fontSize: '10px' }}>{message.isExpanded ? '▼' : '▶'}</span>
    </div>
    {message.isExpanded && (
      <div
        style={{
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
        }}
      >
        {message.content}
      </div>
    )}
  </div>
));

const ToolCallMessageItem = memo<{ message: ToolCallMessage }>(({ message }) => (
  <div
    style={{
      backgroundColor: '#fff3e0',
      padding: '8px 12px',
      borderRadius: '4px',
      maxWidth: '70%',
      fontSize: '12px',
      borderLeft: '3px solid #ff9800',
    }}
  >
    <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>
      🔧 Calling tool: <code style={{ backgroundColor: 'rgba(0,0,0,0.05)', padding: '1px 4px', borderRadius: '2px' }}>{message.toolName}</code>
    </div>
    {message.arguments && Object.keys(message.arguments).length > 0 && (
      <pre
        style={{
          margin: '4px 0 0 0',
          padding: '6px',
          backgroundColor: 'rgba(0,0,0,0.03)',
          borderRadius: '3px',
          overflow: 'auto',
          maxHeight: '100px',
        }}
      >
        {JSON.stringify(message.arguments, null, 2)}
      </pre>
    )}
  </div>
));

const ToolResultMessageItem = memo<{ message: ToolResultMessage }>(({ message }) => (
  <div
    style={{
      backgroundColor: message.success ? '#e8f5e9' : '#ffebee',
      padding: '6px 12px',
      borderRadius: '4px',
      maxWidth: '70%',
      fontSize: '12px',
      borderLeft: message.success ? '3px solid #4caf50' : '3px solid #f44336',
    }}
  >
    <span>
      {message.success ? '✓' : '✗'} tool{' '}
      <code style={{ backgroundColor: 'rgba(0,0,0,0.05)', padding: '1px 4px', borderRadius: '2px', fontSize: '11px' }}>
        {message.toolName}
      </code>{' '}
      {message.success ? 'succeeded' : 'failed'}
    </span>
  </div>
));

// Style constants to avoid creating new objects on each render
const actionStyles: Record<string, React.CSSProperties> = {
  base: {
    cursor: 'pointer',
    marginRight: '8px',
    marginBottom: '8px',
    padding: '4px 12px',
    borderRadius: '4px',
    display: 'inline-flex',
    alignItems: 'center',
    gap: '6px',
    border: '1px solid',
    transition: 'all 0.2s',
  },
  pm_start_app: { backgroundColor: '#f6ffed', borderColor: '#b7eb8f', color: '#52c41a' },
  pm_open_folder: { backgroundColor: '#e6f7ff', borderColor: '#91d5ff', color: '#1890ff' },
  pm_view_knowledge: { backgroundColor: '#fff7e6', borderColor: '#ffd591', color: '#fa8c16' },
  pm_view_artifacts: { backgroundColor: '#f9f0ff', borderColor: '#d3adf7', color: '#722ed1' },
  pm_view_code: { backgroundColor: '#fff1f0', borderColor: '#ffa39e', color: '#f5222d' },
  pm_goto_stage: { backgroundColor: '#fffbe6', borderColor: '#ffe58f', color: '#d48806' },
  default: { backgroundColor: '#f5f5f5', borderColor: '#d9d9d9', color: '#595959' },
};

const actionIcons: Record<string, string> = {
  pm_start_app: '🚀',
  pm_open_folder: '📁',
  pm_view_knowledge: '📚',
  pm_view_artifacts: '📄',
  pm_view_code: '💻',
  pm_goto_stage: '↩️',
  pm_create_iteration: '➕',
};

const getActionStyle = (actionType: string): React.CSSProperties => ({
  ...actionStyles.base,
  ...(actionStyles[actionType] || actionStyles.default),
});

const getActionIcon = (actionType: string): string => actionIcons[actionType] || '▶️';

const PMAgentMessageItem = memo<{ message: PMAgentMessage; onActionClick?: (action: PMAction) => void }>(({ 
  message, 
  onActionClick 
}) => (
  <div>
    <div style={{ fontSize: '12px', color: '#888', marginBottom: '4px' }}>
      <TeamOutlined style={{ marginRight: '4px' }} />
      Project Manager Agent
    </div>
    <div
      style={{
        backgroundColor: '#fff',
        padding: '12px 16px',
        borderRadius: '4px',
        border: '1px solid #e8e8e8',
        maxWidth: '70%',
        wordBreak: 'break-word',
        maxHeight: '300px',
        overflowY: 'auto',
      }}
    >
      <MarkdownMessage content={message.content} />
    </div>
    {message.actions && message.actions.length > 0 && (
      <div style={{ marginTop: '12px' }}>
        {message.actions.map((action, idx) => (
          <span
            key={idx}
            style={getActionStyle(action.action_type)}
            onClick={() => onActionClick?.(action)}
            onMouseEnter={(e) => {
              e.currentTarget.style.transform = 'scale(1.02)';
              e.currentTarget.style.boxShadow = '0 2px 8px rgba(0,0,0,0.1)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.transform = 'scale(1)';
              e.currentTarget.style.boxShadow = 'none';
            }}
          >
            {getActionIcon(action.action_type)} {action.label || action.description || action.action_type}
          </span>
        ))}
      </div>
    )}
  </div>
));

// User message style constants
const userMessageStyle: React.CSSProperties = {
  display: 'inline-block',
  backgroundColor: '#1890ff',
  color: '#fff',
  padding: '8px 12px',
  borderRadius: '4px',
  maxWidth: '70%',
  wordBreak: 'break-word',
  maxHeight: '200px',
  overflowY: 'auto',
  textAlign: 'left',
};

const agentMessageContainerStyle: React.CSSProperties = {
  backgroundColor: '#fff',
  padding: '12px 16px',
  borderRadius: '4px',
  border: '1px solid #e8e8e8',
  maxWidth: '70%',
  wordBreak: 'break-word',
  maxHeight: '300px',
  overflowY: 'auto',
};

const MessageListInner: React.FC<MessageListProps> = ({
  messages = [],
  pmMessages = [],
  mode,
  isProcessing,
  currentAgent,
  onToggleThinking,
  onActionClick,
}) => {
  const safeMessages = Array.isArray(messages) ? messages : [];
  const safePmMessages = Array.isArray(pmMessages) ? pmMessages : [];

  // Memoize message rendering
  const renderPipelineMessages = useMemo(() => {
    if (safeMessages.length === 0) {
      return (
        <div style={{ color: '#888', textAlign: 'center', marginTop: '40px' }}>
          {isProcessing ? 'Waiting for agent response...' : 'No messages yet. Start the iteration to begin chatting.'}
        </div>
      );
    }

    return safeMessages.map((msg, index) => {
      const key = getMessageKey(msg, index);
      
      if (msg.type === 'user') {
        return (
          <div key={key} style={{ marginBottom: '16px', textAlign: 'right' }}>
            <div style={userMessageStyle}>
              {(msg as { content: string }).content}
            </div>
          </div>
        );
      }
      
      if (msg.type === 'thinking') {
        return (
          <div key={key} style={{ marginBottom: '16px' }}>
            <ThinkingMessageItem 
              message={msg as ThinkingMessage} 
              onToggle={() => onToggleThinking(index)} 
            />
          </div>
        );
      }
      
      if (msg.type === 'tool_call') {
        return (
          <div key={key} style={{ marginBottom: '16px' }}>
            <ToolCallMessageItem message={msg as ToolCallMessage} />
          </div>
        );
      }
      
      if (msg.type === 'tool_result') {
        return (
          <div key={key} style={{ marginBottom: '16px' }}>
            <ToolResultMessageItem message={msg as ToolResultMessage} />
          </div>
        );
      }
      
      // Default agent message
      return (
        <div key={key} style={{ marginBottom: '16px' }}>
          <div style={{ fontSize: '12px', color: '#888', marginBottom: '4px' }}>
            {(msg as { agentName?: string }).agentName || 'AI Agent'}
          </div>
          <div style={agentMessageContainerStyle}>
            <MarkdownMessage content={(msg as { content: string }).content} />
          </div>
        </div>
      );
    });
  }, [safeMessages, isProcessing, onToggleThinking]);

  const renderPMMessages = useMemo(() => {
    if (safePmMessages.length === 0) {
      return (
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
      );
    }

    return safePmMessages.map((msg, index) => {
      const key = `pm-${getMessageKey(msg, index)}`;
      
      if (msg.type === 'user') {
        return (
          <div key={key} style={{ marginBottom: '16px', textAlign: 'right' }}>
            <div style={userMessageStyle}>
              {(msg as { content: string }).content}
            </div>
          </div>
        );
      }
      
      if (msg.type === 'pm_agent') {
        return (
          <div key={key} style={{ marginBottom: '16px' }}>
            <PMAgentMessageItem message={msg as PMAgentMessage} onActionClick={onActionClick} />
          </div>
        );
      }
      
      return (
        <div key={key} style={{ marginBottom: '16px', color: '#f44336', padding: '12px', backgroundColor: '#ffebee', borderRadius: '4px' }}>
          {(msg as { content: string }).content}
        </div>
      );
    });
  }, [safePmMessages, onActionClick]);

  if (mode === 'pm_agent') {
    return <>{renderPMMessages}</>;
  }

  return <>{renderPipelineMessages}</>;
};

export const MessageList = memo(MessageListInner);