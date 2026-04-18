import React, { memo, useState, useCallback } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';

interface MarkdownMessageProps {
  content: string;
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

// Memoized markdown components
const markdownComponents = {
  code: ({ className, children, ...props }: React.HTMLAttributes<HTMLElement> & { className?: string }) => {
    const isBlock = className || (children && String(children).includes('\n'));
    const codeText = String(children).replace(/\n$/, '');

    if (!isBlock) {
      // Inline code
      return <code {...props}>{children}</code>;
    }

    // Block code with wrapper
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

// Memoized plugins
const remarkPlugins = [remarkGfm];
const rehypePlugins = [rehypeHighlight, rehypeRaw];

const MarkdownMessageInner: React.FC<MarkdownMessageProps> = ({ content }) => {
  return (
    <div className="markdown-body">
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        components={markdownComponents}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

export const MarkdownMessage = memo(MarkdownMessageInner);
