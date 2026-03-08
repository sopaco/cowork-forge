import React, { useMemo, memo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';
import 'highlight.js/styles/github.css';

interface MarkdownMessageProps {
  content: string;
}

// Memoized markdown components to avoid recreation on every render
const markdownComponents = {
  code: ({ className, children, ...props }: React.HTMLAttributes<HTMLElement> & { className?: string }) => {
    return !className ? (
      <code
        style={{
          backgroundColor: '#f6f8fa',
          padding: '2px 6px',
          borderRadius: '3px',
          fontSize: '0.9em',
          fontFamily: 'Consolas, Monaco, "Courier New", monospace',
        }}
        {...props}
      >
        {children}
      </code>
    ) : (
      <div
        style={{
          backgroundColor: '#f6f8fa',
          borderRadius: '6px',
          padding: '12px',
          margin: '8px 0',
          overflowX: 'auto',
          border: '1px solid #e1e4e8',
        }}
      >
        <code className={className} {...props} style={{ fontSize: '13px' }}>
          {children}
        </code>
      </div>
    );
  },
  blockquote: ({ children }: { children?: React.ReactNode }) => (
    <blockquote
      style={{
        borderLeft: '4px solid #dfe2e5',
        margin: '8px 0',
        padding: '8px 16px',
        backgroundColor: '#f6f8fa',
        color: '#6a737d',
      }}
    >
      {children}
    </blockquote>
  ),
  h1: ({ children }: { children?: React.ReactNode }) => (
    <h1 style={{ fontSize: '1.5em', fontWeight: 600, marginBottom: '0.5em', marginTop: '1em' }}>
      {children}
    </h1>
  ),
  h2: ({ children }: { children?: React.ReactNode }) => (
    <h2 style={{ fontSize: '1.3em', fontWeight: 600, marginBottom: '0.5em', marginTop: '0.8em' }}>
      {children}
    </h2>
  ),
  h3: ({ children }: { children?: React.ReactNode }) => (
    <h3 style={{ fontSize: '1.1em', fontWeight: 600, marginBottom: '0.5em', marginTop: '0.6em' }}>
      {children}
    </h3>
  ),
  ul: ({ children }: { children?: React.ReactNode }) => <ul style={{ paddingLeft: '20px', margin: '8px 0' }}>{children}</ul>,
  ol: ({ children }: { children?: React.ReactNode }) => <ol style={{ paddingLeft: '20px', margin: '8px 0' }}>{children}</ol>,
  li: ({ children }: { children?: React.ReactNode }) => <li style={{ marginBottom: '4px' }}>{children}</li>,
  a: ({ children, href }: { children?: React.ReactNode; href?: string }) => (
    <a href={href} target="_blank" rel="noopener noreferrer" style={{ color: '#1890ff', textDecoration: 'underline' }}>
      {children}
    </a>
  ),
  table: ({ children }: { children?: React.ReactNode }) => (
    <table style={{ width: '100%', borderCollapse: 'collapse', margin: '12px 0', fontSize: '13px' }}>
      {children}
    </table>
  ),
  thead: ({ children }: { children?: React.ReactNode }) => <thead style={{ backgroundColor: '#f6f8fa' }}>{children}</thead>,
  th: ({ children }: { children?: React.ReactNode }) => (
    <th style={{ padding: '8px 12px', textAlign: 'left', borderBottom: '2px solid #e1e4e8', fontWeight: 600 }}>
      {children}
    </th>
  ),
  td: ({ children }: { children?: React.ReactNode }) => (
    <td style={{ padding: '8px 12px', borderBottom: '1px solid #e1e4e8' }}>{children}</td>
  ),
};

// Memoized remark/rehype plugins array
const remarkPlugins = [remarkGfm];
const rehypePlugins = [rehypeHighlight, rehypeRaw];

const MarkdownMessageInner: React.FC<MarkdownMessageProps> = ({ content }) => {
  return (
    <div style={{ lineHeight: '1.6', fontSize: '14px' }}>
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

// Export memoized component - only re-renders when content changes
export const MarkdownMessage = memo(MarkdownMessageInner);
