import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';
import 'highlight.js/styles/github.css';

interface MarkdownMessageProps {
  content: string;
}

export const MarkdownMessage: React.FC<MarkdownMessageProps> = ({ content }) => {
  return (
    <div style={{ lineHeight: '1.6', fontSize: '14px' }}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight, rehypeRaw]}
        components={{
          code: ({ className, children, ...props }) => {
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
          blockquote: ({ children }) => (
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
          h1: ({ children }) => (
            <h1 style={{ fontSize: '1.5em', fontWeight: 600, marginBottom: '0.5em', marginTop: '1em' }}>
              {children}
            </h1>
          ),
          h2: ({ children }) => (
            <h2 style={{ fontSize: '1.3em', fontWeight: 600, marginBottom: '0.5em', marginTop: '0.8em' }}>
              {children}
            </h2>
          ),
          h3: ({ children }) => (
            <h3 style={{ fontSize: '1.1em', fontWeight: 600, marginBottom: '0.5em', marginTop: '0.6em' }}>
              {children}
            </h3>
          ),
          ul: ({ children }) => <ul style={{ paddingLeft: '20px', margin: '8px 0' }}>{children}</ul>,
          ol: ({ children }) => <ol style={{ paddingLeft: '20px', margin: '8px 0' }}>{children}</ol>,
          li: ({ children }) => <li style={{ marginBottom: '4px' }}>{children}</li>,
          a: ({ children, href }) => (
            <a href={href} target="_blank" rel="noopener noreferrer" style={{ color: '#1890ff', textDecoration: 'underline' }}>
              {children}
            </a>
          ),
          table: ({ children }) => (
            <table style={{ width: '100%', borderCollapse: 'collapse', margin: '12px 0', fontSize: '13px' }}>
              {children}
            </table>
          ),
          thead: ({ children }) => <thead style={{ backgroundColor: '#f6f8fa' }}>{children}</thead>,
          th: ({ children }) => (
            <th style={{ padding: '8px 12px', textAlign: 'left', borderBottom: '2px solid #e1e4e8', fontWeight: 600 }}>
              {children}
            </th>
          ),
          td: ({ children }) => (
            <td style={{ padding: '8px 12px', borderBottom: '1px solid #e1e4e8' }}>{children}</td>
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};
