import React, { memo, useState, useCallback } from 'react';
import ReactMarkdown from 'react-markdown';
import { remarkPlugins, fullRehypePlugins } from '@/utils/markdown';

interface MarkdownMessageProps {
  content: string;
  /** 流式输出中：跳过 highlight.js / rehype-raw，streaming 结束后再切回完整渲染 */
  streaming?: boolean;
}

// Copy button for code blocks
const CodeCopyButton: React.FC<{ text: string }> = ({ text }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }, [text]);

  return (
    <button className="markdown-code-copy" onClick={handleCopy}>
      {copied ? 'Copied' : 'Copy'}
    </button>
  );
};

// Extract language from className like "language-rust"
const getLanguage = (className?: string): string => {
  if (!className) return '';
  const match = className.match(/language-(\w+)/);
  return match ? match[1] : '';
};

// 完整渲染组件（streaming 结束后用）
const markdownComponents = {
  code: ({ className, children, ...props }: React.HTMLAttributes<HTMLElement> & { className?: string }) => {
    const isBlock = className || (children && String(children).includes('\n'));
    const codeText = String(children).replace(/\n$/, '');

    if (!isBlock) {
      return <code {...props}>{children}</code>;
    }

    const lang = getLanguage(className);
    return (
      <div className="markdown-code-block">
        <div className="markdown-code-header">
          <span>{lang || 'code'}</span>
          <CodeCopyButton text={codeText} />
        </div>
        <pre>
          <code className={className} {...props}>
            {children}
          </code>
        </pre>
      </div>
    );
  },
  blockquote: ({ children }: { children?: React.ReactNode }) => (
    <blockquote>{children}</blockquote>
  ),
  h1: ({ children }: { children?: React.ReactNode }) => <h1>{children}</h1>,
  h2: ({ children }: { children?: React.ReactNode }) => <h2>{children}</h2>,
  h3: ({ children }: { children?: React.ReactNode }) => <h3>{children}</h3>,
  h4: ({ children }: { children?: React.ReactNode }) => <h4>{children}</h4>,
  ul: ({ children }: { children?: React.ReactNode }) => <ul>{children}</ul>,
  ol: ({ children }: { children?: React.ReactNode }) => <ol>{children}</ol>,
  li: ({ children }: { children?: React.ReactNode }) => <li>{children}</li>,
  a: ({ children, href }: { children?: React.ReactNode; href?: string }) => (
    <a href={href} target="_blank" rel="noopener noreferrer">{children}</a>
  ),
  table: ({ children }: { children?: React.ReactNode }) => <table>{children}</table>,
  thead: ({ children }: { children?: React.ReactNode }) => <thead>{children}</thead>,
  th: ({ children }: { children?: React.ReactNode }) => <th>{children}</th>,
  td: ({ children }: { children?: React.ReactNode }) => <td>{children}</td>,
  p: ({ children }: { children?: React.ReactNode }) => <p>{children}</p>,
};

// 流式期间用的轻量 code 组件：不跑 highlight.js，只渲染纯 pre/code
const streamingCode = ({ className, children, ...props }: React.HTMLAttributes<HTMLElement> & { className?: string }) => {
  const isBlock = className || (children && String(children).includes('\n'));
  if (!isBlock) {
    return <code {...props}>{children}</code>;
  }
  return (
    <pre>
      <code className={className} {...props}>{children}</code>
    </pre>
  );
};

const streamingComponents = {
  ...markdownComponents,
  code: streamingCode,
};

// 流式期间不用 rehype（highlight.js 是同步阻塞的，长代码块 50–200ms）
const MarkdownMessageInner: React.FC<MarkdownMessageProps> = ({ content, streaming }) => {
  // streaming=true：用轻量组件 + 不带 rehype，纯文本快速渲染
  // streaming=false/undefined：完整渲染（gfm + highlight + raw）
  return (
    <div className="markdown-body">
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={streaming ? undefined : fullRehypePlugins}
        components={streaming ? streamingComponents : markdownComponents}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

// 自定义比较：content 与 streaming 都没变才跳过重渲
export const MarkdownMessage = memo(
  MarkdownMessageInner,
  (prev, next) => prev.content === next.content && prev.streaming === next.streaming
);
