import React, { memo, useMemo, useRef, useState, useEffect } from 'react';
import { VariableSizeList as List } from 'react-window';
import { Spin } from 'antd';
import { RobotOutlined } from '@ant-design/icons';
import { MarkdownMessage } from '../common';
import type { ChatMessage, ThinkingMessage, PMAction } from '../../stores';

import avatarPm from '../../assets/avatars/avatar_role_pm.png';
import avatarDesigner from '../../assets/avatars/avatar_role_designer.png';
import avatarRd from '../../assets/avatars/avatar_role_rd.png';
import avatarQa from '../../assets/avatars/avatar_role_qa.png';
import avatarController from '../../assets/avatars/avatar_role_controller.png';

// Map agent name / stage name to avatar image
const getAgentAvatar = (agentName?: string, stageName?: string): string => {
  const stage = (stageName || '').toLowerCase();
  if (stage === 'idea' || stage === 'prd') return avatarPm;
  if (stage === 'design') return avatarDesigner;
  if (stage === 'plan' || stage === 'coding') return avatarRd;
  if (stage === 'check' || stage === 'delivery') return avatarQa;

  const name = (agentName || '').toLowerCase();
  if (name.includes('idea') || name.includes('prd') || name.includes('product manager') || name.includes('pm agent')) return avatarPm;
  if (name.includes('design') || name.includes('architect')) return avatarDesigner;
  if (name.includes('plan') || name.includes('project manager') || name.includes('engineer') || name.includes('coding') || name.includes('developer')) return avatarRd;
  if (name.includes('check') || name.includes('qa') || name.includes('delivery') || name.includes('reviewer')) return avatarQa;

  return avatarController;
};

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

// 估算每条消息行高（用于 react-window）
const estimateItemSize = (msg: ChatMessage | undefined, containerWidth: number): number => {
  if (!msg) return 80;
  const width = Math.max(containerWidth - 80, 200); // 减去 padding/avatar
  switch (msg.type) {
    case 'user': {
      const content = (msg as { content: string }).content || '';
      // 估算：每行 ~50 字符，行高 22px，最小 50
      const lines = Math.ceil(content.length / Math.max(width / 8, 30));
      return Math.max(50, lines * 22 + 24);
    }
    case 'tool_call':
    case 'tool_result':
      return 56;
    case 'thinking':
      return (msg as ThinkingMessage).isExpanded ? 200 : 44;
    case 'error':
      return 60;
    case 'pm_agent':
    case 'agent':
    default: {
      const content = (msg as { content: string }).content || '';
      // markdown 估算：每 8 字符约 1px 宽度，行高 22
      const lines = Math.ceil(content.length / Math.max(width / 8, 30));
      return Math.max(80, lines * 22 + 60);
    }
  }
};

// ---- Thinking Message ----
const ThinkingMessageItem = memo<{ message: ThinkingMessage; onToggle: () => void }>(
  ({ message, onToggle }) => {
    const avatarSrc = getAgentAvatar(message.agentName, (message as { stageName?: string }).stageName);
    return (
      <div className={`chat-msg-row chat-msg-thinking ${message.isExpanded ? 'chat-thinking-expanded' : ''}`}>
        <div className="chat-thinking-toggle" onClick={onToggle}>
          <img className="chat-agent-avatar" src={avatarSrc} alt={message.agentName || 'Agent'} />
          <span className="chat-thinking-label">{message.agentName} thinking...</span>
          <span className="chat-thinking-chevron">▶</span>
        </div>
        {message.isExpanded && (
          <div className="chat-thinking-body">{message.content}</div>
        )}
      </div>
    );
  }
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
  const avatarSrc = getAgentAvatar(agentName, stageName);
  const isStreaming = (message as { isStreaming?: boolean }).isStreaming;

  return (
    <div className="chat-msg-row chat-msg-agent">
      <div className="chat-msg-header">
        <img className="chat-agent-avatar" src={avatarSrc} alt={agentName} />
        <span className="chat-agent-name">{agentName}</span>
        {stageName && <span className="chat-agent-stage">{stageName}</span>}
      </div>
      <div className="chat-msg-content">
        <MarkdownMessage content={content} streaming={isStreaming} />
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
  ({ message, onActionClick }) => {
    const isStreaming = (message as { isStreaming?: boolean }).isStreaming;
    return (
      <div className="chat-msg-row chat-msg-pm-agent">
        <div className="chat-pm-header">
          <img className="chat-agent-avatar" src={avatarPm} alt="Project Manager" />
          <span className="chat-pm-name">Project Manager</span>
        </div>
        <div className="chat-msg-content">
          <MarkdownMessage content={message.content} streaming={isStreaming} />
        </div>
        {!isStreaming && message.actions && message.actions.length > 0 && (
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
    );
  }
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

// ---- Virtualized List Row ----
interface RowData {
  messages: ChatMessage[];
  onToggleThinking: (index: number) => void;
  onActionClick?: (action: PMAction) => void;
}

const PipelineRow: React.FC<{ index: number; style: React.CSSProperties; data: RowData }> = ({ index, style, data }) => {
  const msg = data.messages[index];
  const key = getMessageKey(msg, index);

  switch (msg.type) {
    case 'user':
      return <div style={style}><UserMessageItem key={key} content={(msg as { content: string }).content} /></div>;
    case 'thinking':
      return (
        <div style={style}>
          <ThinkingMessageItem
            key={key}
            message={msg as ThinkingMessage}
            onToggle={() => data.onToggleThinking(index)}
          />
        </div>
      );
    case 'tool_call':
      return <div style={style}><ToolCallMessageItem key={key} message={msg as ToolCallMessage} /></div>;
    case 'tool_result':
      return <div style={style}><ToolResultMessageItem key={key} message={msg as ToolResultMessage} /></div>;
    case 'agent':
      return <div style={style}><AgentMessageItem key={key} message={msg} /></div>;
    case 'error':
      return <div style={style}><ErrorMessageItem key={key} content={(msg as { content: string }).content} /></div>;
    default:
      return <div style={style}><AgentMessageItem key={key} message={msg} /></div>;
  }
};

const PMRow: React.FC<{ index: number; style: React.CSSProperties; data: RowData }> = ({ index, style, data }) => {
  const msg = data.messages[index];
  const key = `pm-${getMessageKey(msg, index)}`;

  if (msg.type === 'user') {
    return <div style={style}><UserMessageItem key={key} content={(msg as { content: string }).content} /></div>;
  }
  if (msg.type === 'pm_agent') {
    return (
      <div style={style}>
        <PMAgentMessageItem
          key={key}
          message={msg as PMAgentMessage}
          onActionClick={data.onActionClick}
        />
      </div>
    );
  }
  return <div style={style}><ErrorMessageItem key={key} content={(msg as { content: string }).content} /></div>;
};

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

  // 测量容器宽度，用于估算行高
  const containerRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(800);
  useEffect(() => {
    if (!containerRef.current) return;
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) {
        setContainerWidth(entry.contentRect.width);
      }
    });
    observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, []);

  // 滚动到底部 ref
  const listRef = useRef<List>(null);
  const lastCountRef = useRef(0);

  // 列表数据
  const activeMessages = mode === 'pm_agent' ? safePmMessages : safeMessages;
  const isEmpty = activeMessages.length === 0;

  // 自动滚动到底部
  useEffect(() => {
    if (activeMessages.length === 0) return;
    // 新消息到达时滚动到底部
    if (activeMessages.length !== lastCountRef.current) {
      lastCountRef.current = activeMessages.length;
      requestAnimationFrame(() => {
        listRef.current?.scrollToItem(activeMessages.length - 1, 'end');
      });
    }
  }, [activeMessages.length]);

  // 流式期间持续滚到底部（最后一条还在变化但 length 没变）
  const lastContentLen = activeMessages.length > 0
    ? (activeMessages[activeMessages.length - 1] as { content?: string }).content?.length || 0
    : 0;
  useEffect(() => {
    if (activeMessages.length === 0) return;
    // 只在最后一条 content 增长时滚（流式）
    requestAnimationFrame(() => {
      listRef.current?.scrollToItem(activeMessages.length - 1, 'end');
    });
  }, [lastContentLen, activeMessages.length]);

  const rowData: RowData = useMemo(() => ({
    messages: activeMessages as ChatMessage[],
    onToggleThinking,
    onActionClick,
  }), [activeMessages, onToggleThinking, onActionClick]);

  const itemSize = useMemo(() => (index: number) => {
    return estimateItemSize(activeMessages[index], containerWidth);
  }, [activeMessages, containerWidth]);

  // 空状态
  if (mode === 'pm_agent' && safePmMessages.length === 0) {
    return <PMWelcome />;
  }
  if (mode !== 'pm_agent' && safeMessages.length === 0) {
    return <EmptyState isProcessing={isProcessing} />;
  }

  const RowComponent = mode === 'pm_agent' ? PMRow : PipelineRow;

  return (
    <div ref={containerRef} style={{ height: '100%', width: '100%' }}>
      <List
        ref={listRef}
        height={containerRef.current?.clientHeight || 600}
        itemCount={activeMessages.length}
        itemSize={itemSize}
        width="100%"
        itemData={rowData}
        overscanCount={4}
      >
        {RowComponent}
      </List>
    </div>
  );
};

export const MessageList = memo(MessageListInner);
