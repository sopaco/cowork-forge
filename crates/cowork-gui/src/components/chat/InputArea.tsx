import React, { memo, useCallback, useState, useRef, useEffect } from 'react';
import { Button, Space, Input, Modal } from 'antd';
import { CopyOutlined, ExpandOutlined, CompressOutlined } from '@ant-design/icons';
import type { InputRequest, InputOption } from '../../stores';

const { TextArea } = Input;

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
  const [expanded, setExpanded] = useState(false);
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const modalTextAreaRef = useRef<HTMLTextAreaElement>(null);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    onUserInputChange(e.target.value);
  }, [onUserInputChange]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Shift + Enter: 展开输入框
    if (e.shiftKey && e.key === 'Enter') {
      e.preventDefault();
      setExpanded(true);
      return;
    }
    // Enter alone: 发送消息
    if (!e.shiftKey && e.key === 'Enter') {
      e.preventDefault();
      if (userInput.trim() && !disabled) {
        onSend();
      }
    }
  }, [userInput, disabled, onSend]);

  const handleExpandClick = useCallback(() => {
    setExpanded(true);
  }, []);

  const handleCollapse = useCallback(() => {
    setExpanded(false);
    // 折叠后聚焦回主输入框
    setTimeout(() => {
      textAreaRef.current?.focus();
    }, 100);
  }, []);

  const handleModalSend = useCallback(() => {
    if (userInput.trim() && !disabled) {
      onSend();
      setExpanded(false);
    }
  }, [userInput, disabled, onSend]);

  // 展开时聚焦 Modal 内的 TextArea
  useEffect(() => {
    if (expanded) {
      setTimeout(() => {
        modalTextAreaRef.current?.focus();
        // 将光标移到末尾
        const len = userInput.length;
        modalTextAreaRef.current?.setSelectionRange(len, len);
      }, 100);
    }
  }, [expanded, userInput]);

  const handleModalKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Shift + Enter: 在展开模式下输入换行
    if (e.shiftKey && e.key === 'Enter') {
      // 默认行为，输入换行
      return;
    }
    // Ctrl/Cmd + Enter: 发送消息
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      if (userInput.trim() && !disabled) {
        onSend();
        setExpanded(false);
      }
    }
    // Escape: 关闭展开模式
    if (e.key === 'Escape') {
      e.preventDefault();
      setExpanded(false);
    }
  }, [userInput, disabled, onSend]);

  const renderInputWithExpand = (placeholder: string) => (
    <div style={{ display: 'flex', gap: '8px', alignItems: 'flex-end' }}>
      <TextArea
        ref={textAreaRef}
        value={userInput}
        onChange={handleInputChange}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        autoSize={{ minRows: 1, maxRows: 4 }}
        style={{ resize: 'none' }}
      />
      <Button
        icon={<ExpandOutlined />}
        onClick={handleExpandClick}
        title="展开输入框 (Shift + Enter)"
        style={{ flexShrink: 0 }}
      />
    </div>
  );

  if (mode === 'pm_agent') {
    return (
      <>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'flex-end' }}>
          <div style={{ flex: 1 }}>
            {renderInputWithExpand('Ask about the project or request changes...')}
          </div>
          <Button onClick={onSend} type="primary" disabled={!userInput.trim() || disabled}>
            Send
          </Button>
          <Button onClick={onDumpChat} icon={<CopyOutlined />} title="Copy chat to clipboard">
            Dump
          </Button>
        </div>

        <Modal
          open={expanded}
          onCancel={handleCollapse}
          title={
            <span>
              <CompressOutlined style={{ marginRight: 8 }} />
              展开输入
            </span>
          }
          width={700}
          footer={[
            <Button key="dump" onClick={onDumpChat} icon={<CopyOutlined />}>
              Dump
            </Button>,
            <Button key="cancel" onClick={handleCollapse}>
              取消
            </Button>,
            <Button key="send" type="primary" onClick={handleModalSend} disabled={!userInput.trim() || disabled}>
              发送 (Ctrl+Enter)
            </Button>,
          ]}
          styles={{
            body: { padding: '16px' },
          }}
        >
          <TextArea
            ref={modalTextAreaRef}
            value={userInput}
            onChange={handleInputChange}
            onKeyDown={handleModalKeyDown}
            placeholder="输入内容，按 Ctrl+Enter 发送，按 Escape 关闭..."
            disabled={disabled}
            autoSize={{ minRows: 8, maxRows: 20 }}
            style={{ resize: 'none' }}
          />
          <div style={{ marginTop: 8, color: '#888', fontSize: '12px' }}>
            提示：Shift+Enter 换行，Ctrl+Enter 发送，Escape 关闭
          </div>
        </Modal>
      </>
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

      <div style={{ display: 'flex', gap: '8px', alignItems: 'flex-end' }}>
        <div style={{ flex: 1 }}>
          {renderInputWithExpand(inputRequest ? 'Type your response...' : 'Type a message...')}
        </div>
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

      <Modal
        open={expanded}
        onCancel={handleCollapse}
        title={
          <span>
            <CompressOutlined style={{ marginRight: 8 }} />
            展开输入
          </span>
        }
        width={700}
        footer={[
          <Button key="dump" onClick={onDumpChat} icon={<CopyOutlined />}>
            Dump
          </Button>,
          <Button key="cancel" onClick={handleCollapse}>
            取消
          </Button>,
          inputRequest ? (
            <Button key="send" type="primary" onClick={() => { onSubmitFeedback(); setExpanded(false); }} disabled={!userInput.trim()}>
              Send Feedback
            </Button>
          ) : (
            <Button key="send" type="primary" onClick={handleModalSend} disabled={!userInput.trim()}>
              发送 (Ctrl+Enter)
            </Button>
          ),
        ]}
        styles={{
          body: { padding: '16px' },
        }}
      >
        <TextArea
          ref={modalTextAreaRef}
          value={userInput}
          onChange={handleInputChange}
          onKeyDown={handleModalKeyDown}
          placeholder={inputRequest ? 'Type your response...' : '输入内容，按 Ctrl+Enter 发送，按 Escape 关闭...'}
          disabled={disabled && !inputRequest}
          autoSize={{ minRows: 8, maxRows: 20 }}
          style={{ resize: 'none' }}
        />
        <div style={{ marginTop: 8, color: '#888', fontSize: '12px' }}>
          提示：Shift+Enter 换行，Ctrl+Enter 发送，Escape 关闭
        </div>
      </Modal>
    </>
  );
};

export const InputArea = memo(InputAreaInner);