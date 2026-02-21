import React from 'react';
import { Button, Space, Input } from 'antd';
import type { InputRequest, InputOption } from '../../stores';

interface InputAreaProps {
  userInput: string;
  onUserInputChange: (value: string) => void;
  onSend: () => void;
  inputRequest?: InputRequest | null;
  onSelectOption: (option: InputOption) => void;
  onSubmitFeedback: () => void;
  onCancelFeedback: () => void;
  disabled?: boolean;
  mode: 'pipeline' | 'pm_agent';
}

export const InputArea: React.FC<InputAreaProps> = ({
  userInput,
  onUserInputChange,
  onSend,
  inputRequest,
  onSelectOption,
  onSubmitFeedback,
  onCancelFeedback,
  disabled,
  mode,
}) => {
  const handleSendClick = () => {
    console.log('[InputArea] Send button clicked', { mode, userInput, disabled });
    onSend();
  };

  if (mode === 'pm_agent') {
    return (
      <div style={{ display: 'flex', gap: '8px' }}>
        <Input
          value={userInput}
          onChange={(e) => onUserInputChange(e.target.value)}
          onPressEnter={handleSendClick}
          placeholder="Ask about the project or request changes..."
          disabled={disabled}
        />
        <Button onClick={handleSendClick} type="primary" disabled={!userInput.trim() || disabled}>
          Send
        </Button>
      </div>
    );
  }

  return (
    <>
      {inputRequest && (
        <div
          style={{
            padding: '16px',
            backgroundColor: '#fff7e6',
            border: '1px solid #ffd591',
            borderRadius: '4px',
            marginBottom: '16px',
          }}
        >
          <div style={{ marginBottom: '8px', fontWeight: 'bold' }}>
            {inputRequest.isArtifactConfirmation ? `Confirm ${inputRequest.artifactType}` : 'Input Required'}
          </div>
          <div style={{ marginBottom: '12px', color: '#666' }}>
            {inputRequest.isFeedbackMode ? inputRequest.feedbackPrompt : inputRequest.prompt}
          </div>
          {inputRequest.options && !inputRequest.isFeedbackMode && (
            <Space direction="vertical" style={{ width: '100%' }}>
              {inputRequest.options.map((option) => (
                <Button key={option.id} onClick={() => onSelectOption(option)} block>
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
          onChange={(e) => onUserInputChange(e.target.value)}
          onPressEnter={onSend}
          placeholder={inputRequest ? 'Type your response...' : 'Type a message...'}
          disabled={disabled && !inputRequest}
        />
        {inputRequest ? (
          <Button onClick={onSubmitFeedback} type="primary" disabled={!userInput.trim()}>
            Send Feedback
          </Button>
        ) : (
          <Button onClick={onSend} type="primary" disabled={!userInput.trim()}>
            Send
          </Button>
        )}
        {inputRequest && inputRequest.isFeedbackMode && (
          <Button onClick={onCancelFeedback}>Cancel</Button>
        )}
      </div>
    </>
  );
};
