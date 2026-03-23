import React, { memo, useCallback } from 'react';
import { Button, Space, Input } from 'antd';
import { CopyOutlined } from '@ant-design/icons';
import type { InputRequest, InputOption } from '../../stores';

interface InputAreaProps {
  userInput: string;
  onUserInputChange: (value: string) => void;
  onSend: () => void;
  onDumpChat: () => void;
  inputRequest?: InputRequest | null;
  onSelectOption: (option: InputOption) => void;
  onSubmitFeedback: () => void;
  onCancelFeedback: () => void;
  disabled?: boolean;
  mode: 'pipeline' | 'pm_agent';
}

const InputAreaInner: React.FC<InputAreaProps> = ({
  userInput,
  onUserInputChange,
  onSend,
  onDumpChat,
  inputRequest,
  onSelectOption,
  onSubmitFeedback,
  onCancelFeedback,
  disabled,
  mode,
}) => {
  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    onUserInputChange(e.target.value);
  }, [onUserInputChange]);

  if (mode === 'pm_agent') {
    return (
      <div style={{ display: 'flex', gap: '8px' }}>
        <Input
          value={userInput}
          onChange={handleInputChange}
          onPressEnter={onSend}
          placeholder="Ask about the project or request changes..."
          disabled={disabled}
        />
        <Button onClick={onSend} type="primary" disabled={!userInput.trim() || disabled}>
          Send
        </Button>
        <Button onClick={onDumpChat} icon={<CopyOutlined />} title="Copy chat to clipboard">
          Dump
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
          onChange={handleInputChange}
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
        <Button onClick={onDumpChat} icon={<CopyOutlined />} title="Copy chat to clipboard">
          Dump
        </Button>
        {inputRequest && inputRequest.isFeedbackMode && (
          <Button onClick={onCancelFeedback}>Cancel</Button>
        )}
      </div>
    </>
  );
};

export const InputArea = memo(InputAreaInner);
