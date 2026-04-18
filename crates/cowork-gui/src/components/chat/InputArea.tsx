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
  const isComposingRef = useRef(false);
  const compositionEndTimeRef = useRef(0);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    onUserInputChange(e.target.value);
  }, [onUserInputChange]);

  const handleCompositionStart = useCallback(() => {
    isComposingRef.current = true;
  }, []);

  const handleCompositionEnd = useCallback(() => {
    compositionEndTimeRef.current = Date.now();
    setTimeout(() => {
      isComposingRef.current = false;
    }, 0);
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.nativeEvent.isComposing) return;
    if (Date.now() - compositionEndTimeRef.current < 50) return;
    if (isComposingRef.current) return;

    if (e.shiftKey && e.key === 'Enter') {
      e.preventDefault();
      setExpanded(true);
      return;
    }
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

  useEffect(() => {
    if (expanded) {
      setTimeout(() => {
        modalTextAreaRef.current?.focus();
        const len = userInput.length;
        modalTextAreaRef.current?.setSelectionRange(len, len);
      }, 100);
    }
  }, [expanded, userInput]);

  const handleModalKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.nativeEvent.isComposing) return;
    if (Date.now() - compositionEndTimeRef.current < 50) return;
    if (isComposingRef.current) return;

    if (e.shiftKey && e.key === 'Enter') return;
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      if (userInput.trim() && !disabled) {
        onSend();
        setExpanded(false);
      }
    }
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
        onCompositionStart={handleCompositionStart}
        onCompositionEnd={handleCompositionEnd}
        placeholder={placeholder}
        disabled={disabled}
        autoSize={{ minRows: 1, maxRows: 4 }}
        style={{ resize: 'none' }}
      />
      <Button
        icon={<ExpandOutlined />}
        onClick={handleExpandClick}
        title="Expand (Shift + Enter)"
        size="small"
        style={{ flexShrink: 0 }}
      />
    </div>
  );

  if (mode === 'pm_agent') {
    return (
      <>
        <div className="chat-input-wrapper">
          <div style={{ display: 'flex', gap: '8px', alignItems: 'flex-end' }}>
            <div style={{ flex: 1 }}>
              {renderInputWithExpand('Ask about the project or request changes...')}
            </div>
            <Button onClick={onSend} type="primary" disabled={!userInput.trim() || disabled} size="small">
              Send
            </Button>
            <Button onClick={onDumpChat} icon={<CopyOutlined />} size="small" title="Copy chat">
              Dump
            </Button>
          </div>
        </div>

        <Modal
          open={expanded}
          onCancel={handleCollapse}
          title={<span><CompressOutlined style={{ marginRight: 8 }} />Expanded Input</span>}
          width={700}
          footer={[
            <Button key="dump" onClick={onDumpChat} icon={<CopyOutlined />}>Dump</Button>,
            <Button key="cancel" onClick={handleCollapse}>Cancel</Button>,
            <Button key="send" type="primary" onClick={handleModalSend} disabled={!userInput.trim() || disabled}>
              Send (Ctrl+Enter)
            </Button>,
          ]}
          styles={{ body: { padding: '16px' } }}
        >
          <TextArea
            ref={modalTextAreaRef}
            value={userInput}
            onChange={handleInputChange}
            onKeyDown={handleModalKeyDown}
            onCompositionStart={handleCompositionStart}
            onCompositionEnd={handleCompositionEnd}
            placeholder="Type your message, Ctrl+Enter to send, Escape to close..."
            disabled={disabled}
            autoSize={{ minRows: 8, maxRows: 20 }}
            style={{ resize: 'none' }}
          />
          <div style={{ marginTop: 8, color: 'var(--text-tertiary)', fontSize: '12px' }}>
            Shift+Enter for newline, Ctrl+Enter to send, Escape to close
          </div>
        </Modal>
      </>
    );
  }

  return (
    <>
      {inputRequest && (
        <div className="chat-input-request">
          <div className="chat-input-request-title">
            {inputRequest.isArtifactConfirmation ? `Confirm ${inputRequest.artifactType}` : 'Input Required'}
          </div>
          <div className="chat-input-request-prompt">
            {inputRequest.isFeedbackMode ? inputRequest.feedbackPrompt : inputRequest.prompt}
          </div>
          {inputRequest.options && !inputRequest.isFeedbackMode && (
            <Space direction="vertical" style={{ width: '100%' }}>
              {inputRequest.options.map((option) => (
                <Button key={option.id} onClick={() => onSelectOption(option)} block size="small">
                  {option.label}
                </Button>
              ))}
            </Space>
          )}
        </div>
      )}

      <div className="chat-input-wrapper">
        <div style={{ display: 'flex', gap: '8px', alignItems: 'flex-end' }}>
          <div style={{ flex: 1 }}>
            {renderInputWithExpand(inputRequest ? 'Type your response...' : 'Type a message...')}
          </div>
          {inputRequest ? (
            <Button onClick={onSubmitFeedback} type="primary" disabled={!userInput.trim()} size="small">
              Send Feedback
            </Button>
          ) : (
            <Button onClick={onSend} type="primary" disabled={!userInput.trim()} size="small">
              Send
            </Button>
          )}
          <Button onClick={onDumpChat} icon={<CopyOutlined />} size="small" title="Copy chat">
            Dump
          </Button>
          {inputRequest && inputRequest.isFeedbackMode && (
            <Button onClick={onCancelFeedback} size="small">Cancel</Button>
          )}
        </div>
      </div>

      <Modal
        open={expanded}
        onCancel={handleCollapse}
        title={<span><CompressOutlined style={{ marginRight: 8 }} />Expanded Input</span>}
        width={700}
        footer={[
          <Button key="dump" onClick={onDumpChat} icon={<CopyOutlined />}>Dump</Button>,
          <Button key="cancel" onClick={handleCollapse}>Cancel</Button>,
          inputRequest ? (
            <Button key="send" type="primary" onClick={() => { onSubmitFeedback(); setExpanded(false); }} disabled={!userInput.trim()}>
              Send Feedback
            </Button>
          ) : (
            <Button key="send" type="primary" onClick={handleModalSend} disabled={!userInput.trim()}>
              Send (Ctrl+Enter)
            </Button>
          ),
        ]}
        styles={{ body: { padding: '16px' } }}
      >
        <TextArea
          ref={modalTextAreaRef}
          value={userInput}
          onChange={handleInputChange}
          onKeyDown={handleModalKeyDown}
          onCompositionStart={handleCompositionStart}
          onCompositionEnd={handleCompositionEnd}
          placeholder={inputRequest ? 'Type your response...' : 'Type your message, Ctrl+Enter to send, Escape to close...'}
          disabled={disabled && !inputRequest}
          autoSize={{ minRows: 8, maxRows: 20 }}
          style={{ resize: 'none' }}
        />
        <div style={{ marginTop: 8, color: 'var(--text-tertiary)', fontSize: '12px' }}>
          Shift+Enter for newline, Ctrl+Enter to send, Escape to close
        </div>
      </Modal>
    </>
  );
};

export const InputArea = memo(InputAreaInner);
