import React, { useMemo, memo, useCallback } from 'react';
import { Spin, Tag, message } from 'antd';
import { TeamOutlined } from '@ant-design/icons';
import { MessageList } from './MessageList';
import { InputArea } from './InputArea';
import type { ChatMessage, InputRequest, InputOption, PMAction } from '../../stores';
import { useConfigStore } from '../../stores/configStore';

interface ChatPanelProps {
  messages: ChatMessage[];
  pmMessages: (ChatMessage & { type: 'user' | 'pm_agent' })[];
  mode: 'pipeline' | 'pm_agent' | 'disabled';
  isProcessing: boolean;
  pmProcessing: boolean;
  currentAgent: string | null;
  iterationTitle: string;
  iterationDescription?: string;
  currentStage?: string | null;
  inputRequest?: InputRequest | null;
  userInput: string;
  messagesContainerRef: React.RefObject<HTMLDivElement>;
  pmMessagesContainerRef: React.RefObject<HTMLDivElement>;
  onUserInputChange: (value: string) => void;
  onSend: () => void;
  onSelectOption: (option: InputOption) => void;
  onSubmitFeedback: () => void;
  onCancelFeedback: () => void;
  onToggleThinking: (index: number) => void;
  onActionClick?: (action: PMAction) => void;
}

const ChatPanelInner: React.FC<ChatPanelProps> = ({
  messages = [],
  pmMessages = [],
  mode,
  isProcessing,
  pmProcessing,
  currentAgent,
  iterationTitle,
  iterationDescription,
  currentStage,
  inputRequest,
  userInput,
  messagesContainerRef,
  pmMessagesContainerRef,
  onUserInputChange,
  onSend,
  onSelectOption,
  onSubmitFeedback,
  onCancelFeedback,
  onToggleThinking,
  onActionClick,
}) => {
  const { flows, default_flow_id } = useConfigStore();
  const stageDisplayName = useMemo(() => {
    if (!currentStage) return null;
    if (default_flow_id && flows[default_flow_id]) {
      const flow = flows[default_flow_id];
      const stage = flow.stages.find(s => s.stage_id === currentStage);
      if (stage?.alias) return stage.alias;
    }
    return currentStage.charAt(0).toUpperCase() + currentStage.slice(1);
  }, [currentStage, flows, default_flow_id]);

  const handleDumpChat = useCallback(() => {
    const currentMessages = mode === 'pm_agent' ? pmMessages : messages;
    
    if (currentMessages.length === 0) {
      message.info('No messages to copy');
      return;
    }

    const formatMessage = (msg: ChatMessage): string => {
      const timestamp = msg.timestamp ? new Date(msg.timestamp).toLocaleString() : '';
      const timePrefix = timestamp ? `[${timestamp}] ` : '';
      
      switch (msg.type) {
        case 'user':
          return `${timePrefix}User: ${msg.content || ''}`;
        case 'agent':
          return `${timePrefix}${msg.agentName || 'Agent'}: ${msg.content || ''}`;
        case 'thinking':
          return `${timePrefix}[Thinking] ${msg.agentName || 'Agent'}: ${msg.content || ''}`;
        case 'tool_call':
          return `${timePrefix}[Tool Call] ${msg.toolName || 'unknown'}: ${JSON.stringify(msg.arguments || {}, null, 2)}`;
        case 'tool_result':
          return `${timePrefix}[Tool Result] ${msg.toolName || 'unknown'}: ${msg.result || ''}`;
        case 'pm_agent':
          return `${timePrefix}PM Agent: ${msg.content || ''}`;
        case 'error':
          return `${timePrefix}[Error] ${msg.content || ''}`;
        default:
          return `${timePrefix}${JSON.stringify(msg)}`;
      }
    };

    const header = `=== Chat Export ===\nIteration: ${iterationTitle}\nMode: ${mode}\nExported at: ${new Date().toLocaleString()}\n\n`;
    const content = currentMessages.map(formatMessage).join('\n\n');
    
    navigator.clipboard.writeText(header + content)
      .then(() => message.success('Chat copied to clipboard'))
      .catch(() => message.error('Failed to copy to clipboard'));
  }, [mode, pmMessages, messages, iterationTitle]);

  if (mode === 'disabled') {
    return (
      <div style={{ padding: '16px', color: '#888', textAlign: 'center' }}>
        Select an iteration to view chat
      </div>
    );
  }

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', background: 'var(--bg-base)' }}>
      {/* Chat Header */}
      <div className="chat-header">
        {mode === 'pm_agent' ? (
          <>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <TeamOutlined style={{ color: 'var(--success)' }} />
              <h3 className="chat-header-title">Project Manager</h3>
              <Tag color="green" style={{ marginLeft: '4px', fontSize: '11px' }}>Post-Delivery</Tag>
            </div>
            <p className="chat-header-desc">{iterationTitle}</p>
          </>
        ) : (
          <>
            <h3 className="chat-header-title">{iterationTitle}</h3>
            {iterationDescription && (
              <p className="chat-header-desc">{iterationDescription}</p>
            )}
          </>
        )}
      </div>

      {/* Processing Indicator */}
      {mode === 'pm_agent' ? (
        pmProcessing && (
          <div className="chat-processing-bar" style={{ background: 'var(--success-light)', color: 'var(--success)', borderBottomColor: 'var(--success)' }}>
            <Spin size="small" />
            <span>Project Manager is thinking...</span>
          </div>
        )
      ) : (
        isProcessing && currentAgent && (
          <div className="chat-processing-bar">
            <Spin size="small" />
            <span>{currentAgent}</span>
            {stageDisplayName && (
              <span className="chat-processing-stage">Stage: {stageDisplayName}</span>
            )}
          </div>
        )
      )}

      {/* Messages Area */}
      <div
        ref={mode === 'pm_agent' ? pmMessagesContainerRef : messagesContainerRef}
        className="chat-messages"
        style={{ flex: 1, overflow: 'auto' }}
      >
        <MessageList
          messages={messages}
          pmMessages={pmMessages}
          mode={mode}
          isProcessing={isProcessing}
          currentAgent={currentAgent}
          onToggleThinking={onToggleThinking}
          onActionClick={onActionClick}
        />
      </div>

      {/* Input Area */}
      <InputArea
        userInput={userInput}
        onUserInputChange={onUserInputChange}
        onSend={onSend}
        onDumpChat={handleDumpChat}
        inputRequest={inputRequest}
        onSelectOption={onSelectOption}
        onSubmitFeedback={onSubmitFeedback}
        onCancelFeedback={onCancelFeedback}
        disabled={mode === 'pipeline' ? isProcessing && !inputRequest : pmProcessing}
        mode={mode}
      />
    </div>
  );
};

export const ChatPanel = memo(ChatPanelInner);
