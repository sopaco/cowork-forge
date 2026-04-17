import React, { memo, useMemo, useCallback } from 'react';
import { Spin } from 'antd';
import { TeamOutlined, RobotOutlined, UserOutlined } from '@ant-design/icons';
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

// Stable key generation
const getMessageKey = (msg: ChatMessage, index: number): string => {
  if (msg.type === 'thinking') return `t-${index}-${msg.agentName || 'a'}`;
  if (msg.type === 'tool_call') return `tc-${index}-${(msg as ToolCallMessage).toolName}`;
  if (msg.type === 'tool_result') return `tr-${index}-${(msg as ToolResultMessage).toolName}`;
  const ts = (msg as { timestamp?: string | number }).timestamp;
  return ts ? `m-${ts}` : `m-${index}`;
};

// Get initials for agent icon
const getAgentInitials = (name?: string): string => {
  if (!name) return 'AI';
  return name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
};

// ---- Thinking Message ----
const ThinkingMessageItem = memo<{ message: ThinkingMessage; onToggle: () => void }>(
  ({ message, onToggle }) => (
    <div className={`chat-msg-row chat-msg-thinking ${message.isExpanded ? 'chat-thinking-expanded' : ''}`}>
      <div className="chat-thinking-toggle" onClick={onToggle}>
        <span className="chat-thinking-icon">
          <RobotOutlined />
        </span>
        <span className="chat-thinking-label">{message.agentName} thinking...</span>
        <span className="chat-thinking-chevron">▶</span>
      </div>
      {message.isExpanded && (
        <div className="chat-thinking-body">{message.content}</div>
      )}
    </div>
  )
);

// ---- Tool Call Message ----
const ToolCallMessageItem = memo<{ message: ToolCallMessage }>(({ message }) => (
  <div className="chat-msg-row chat-msg-tool-call">
    <div className="chat-tool-bar">
      <span className="chat-tool-icon">⚡</span>
      <span className="chat-tool-name">{message.toolName}</span>
    </div>
    {message.arguments && Object.keys(message.arguments).length > 0 && (
      <div className="chat-tool-args">
        {JSON.stringify(message.arguments, null, 2)}
      </div>
    )}
  </div>
));

// ---- Tool Result Message ----
const ToolResultMessageItem = memo<{ message: ToolResultMessage }>(({ message }) => (
  <div className={`chat-msg-row chat-msg-tool-result ${message.success ? 'chat-tool-success' : 'chat-tool-fail'}`}>
    <div className="chat-tool-result-bar">
      <span className="chat-tool-result-icon">{message.success ? '✓' : '✗'}</span>
      <span className="chat-tool-result-name">{message.toolName}</span>
    </div>
  </div>
));

// ---- Agent Message ----
const AgentMessageItem = memo<{ message: ChatMessage }>(({ message }) => {
  const agentName = (message as { agentName?: string }).agentName || 'AI Agent';
  const stageName = (message as { stageName?: string }).stageName;
  const content = (message as { content: string }).content;

  return (
    <div className="chat-msg-row chat-msg-agent">
      <div className="chat-msg-header">
        <span className="chat-agent-icon">{getAgentInitials(agentName)}</span>
        <span className="chat-agent-name">{agentName}</span>
        {stageName && <span className="chat-agent-stage">{stageName}</span>}
      </div>
      <div className="chat-msg-content">
        <MarkdownMessage content={content} />
      </div>
    </div>
  );
});

// ---- User Message ----
const UserMessageItem = memo<{ content: string }>(({ content }) => (
  <div className="chat-msg-row chat-msg-user">
    <div className="chat-msg-content">
      <div className="chat-user-bubble">{content}</div>
    </div>
  </div>
));

// ---- PM Agent Message ----
const PMAgentMessageItem = memo<{ message: PMAgentMessage; onActionClick?: (action: PMAction) => void }>(
  ({ message, onActionClick }) => (
    <div className="chat-msg-row chat-msg-pm-agent">
      <div className="chat-pm-header">
        <span className="chat-pm-icon">
          <TeamOutlined style={{ fontSize: '12px' }} />
        </span>
        <span className="chat-pm-name">Project Manager</span>
      </div>
      <div className="chat-msg-content">
        <MarkdownMessage content={message.content} />
      </div>
      {message.actions && message.actions.length > 0 && (
        <div className="chat-pm-actions">
          {message.actions.map((action, idx) => (
            <span
              key={idx}
              className="chat-pm-action"
              onClick={() => onActionClick?.(action)}
            >
              {action.label || action.description || action.action_type}
            </span>
          ))}
        </div>
      )}
    </div>
  )
);

// ---- Error Message ----
const ErrorMessageItem = memo<{ content: string }>(({ content }) => (
  <div className="chat-msg-row chat-msg-error">
    <div className="chat-error-content">{content}</div>
  </div>
));

// ---- PM Welcome State ----
const PMWelcome: React.FC = () => (
  <div className="chat-pm-welcome">
    <div className="chat-pm-welcome-icon">👋</div>
    <h3>Project Manager Chat</h3>
    <p>Ask me anything about this project, request changes, or discuss next steps.</p>
    <ul>
      <li>Fix bugs by returning to earlier stages</li>
      <li>Add new features through new iterations</li>
      <li>Answer questions about the project</li>
    </ul>
  </div>
);

// ---- Empty State ----
const EmptyState: React.FC<{ isProcessing: boolean }> = ({ isProcessing }) => (
  <div className="chat-empty-state">
    {isProcessing ? (
      <>
        <Spin size="large" />
        <h3>Waiting for agent response...</h3>
      </>
    ) : (
      <>
        <RobotOutlined style={{ fontSize: '40px', color: 'var(--text-muted)' }} />
        <h3>No messages yet</h3>
        <p>Start the iteration to begin collaborating with AI agents.</p>
      </>
    )}
  </div>
);

// ---- Main Message List ----
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

  const renderPipelineMessages = useMemo(() => {
    if (safeMessages.length === 0) {
      return <EmptyState isProcessing={isProcessing} />;
    }

    return safeMessages.map((msg, index) => {
      const key = getMessageKey(msg, index);

      switch (msg.type) {
        case 'user':
          return <UserMessageItem key={key} content={(msg as { content: string }).content} />;

        case 'thinking':
          return (
            <ThinkingMessageItem
              key={key}
              message={msg as ThinkingMessage}
              onToggle={() => onToggleThinking(index)}
            />
          );

        case 'tool_call':
          return <ToolCallMessageItem key={key} message={msg as ToolCallMessage} />;

        case 'tool_result':
          return <ToolResultMessageItem key={key} message={msg as ToolResultMessage} />;

        case 'agent':
          return <AgentMessageItem key={key} message={msg} />;

        case 'error':
          return <ErrorMessageItem key={key} content={(msg as { content: string }).content} />;

        default:
          return <AgentMessageItem key={key} message={msg} />;
      }
    });
  }, [safeMessages, isProcessing, onToggleThinking]);

  const renderPMMessages = useMemo(() => {
    if (safePmMessages.length === 0) {
      return <PMWelcome />;
    }

    return safePmMessages.map((msg, index) => {
      const key = `pm-${getMessageKey(msg, index)}`;

      if (msg.type === 'user') {
        return <UserMessageItem key={key} content={(msg as { content: string }).content} />;
      }

      if (msg.type === 'pm_agent') {
        return (
          <PMAgentMessageItem
            key={key}
            message={msg as PMAgentMessage}
            onActionClick={onActionClick}
          />
        );
      }

      return <ErrorMessageItem key={key} content={(msg as { content: string }).content} />;
    });
  }, [safePmMessages, onActionClick]);

  if (mode === 'pm_agent') {
    return <>{renderPMMessages}</>;
  }

  return <>{renderPipelineMessages}</>;
};

export const MessageList = memo(MessageListInner);
