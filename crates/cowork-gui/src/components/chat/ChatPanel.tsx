import React from 'react';
import { Spin, Tag } from 'antd';
import { TeamOutlined } from '@ant-design/icons';
import { MessageList } from './MessageList';
import { InputArea } from './InputArea';
import type { ChatMessage, InputRequest, InputOption, PMAction } from '../../stores';

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

export const ChatPanel: React.FC<ChatPanelProps> = ({
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
  if (mode === 'disabled') {
    return (
      <div style={{ padding: '16px', color: '#888', textAlign: 'center' }}>
        Select an iteration to view chat
      </div>
    );
  }

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', padding: '16px' }}>
      {mode === 'pm_agent' ? (
        <div style={{ marginBottom: '16px' }}>
          <h3 style={{ margin: 0 }}>
            <TeamOutlined style={{ marginRight: '8px' }} />
            Project Manager Agent
          </h3>
          <p style={{ margin: '4px 0 0 0', color: '#888', fontSize: '12px' }}>
            {iterationTitle} - Ask questions, request changes, or discuss next steps
          </p>
          <Tag color="green" style={{ marginTop: '8px' }}>
            Post-Delivery Chat
          </Tag>
        </div>
      ) : (
        <>
          <div style={{ marginBottom: '16px' }}>
            <h3 style={{ margin: 0 }}>Chat - {iterationTitle}</h3>
            {iterationDescription && (
              <p style={{ margin: '4px 0 0 0', color: '#888', fontSize: '12px' }}>{iterationDescription}</p>
            )}
          </div>

          {isProcessing && currentAgent && (
            <div
              style={{
                padding: '12px 16px',
                backgroundColor: '#e6f7ff',
                border: '1px solid #91d5ff',
                borderRadius: '6px',
                marginBottom: '16px',
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
              }}
            >
              <Spin size="small" />
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: '14px', fontWeight: 500, color: '#1890ff', marginBottom: '4px' }}>
                  {currentAgent} is working...
                </div>
                <div style={{ fontSize: '12px', color: '#666' }}>
                  {currentStage ? `Stage: ${currentStage}` : 'Processing...'}
                </div>
              </div>
            </div>
          )}
        </>
      )}

      <div
        ref={mode === 'pm_agent' ? pmMessagesContainerRef : messagesContainerRef}
        style={{
          flex: 1,
          overflow: 'auto',
          border: '1px solid #e8e8e8',
          borderRadius: '4px',
          padding: '16px',
          marginBottom: '16px',
          backgroundColor: '#fafafa',
        }}
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

      <InputArea
        userInput={userInput}
        onUserInputChange={onUserInputChange}
        onSend={onSend}
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
