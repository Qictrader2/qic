import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { colors, fonts } from '../theme';

const markdownStyles: React.CSSProperties = {
  color: colors.textSecondary,
  fontSize: '0.9375rem',
  lineHeight: 1.7,
  wordBreak: 'break-word',
};

const componentStyles: Record<string, React.CSSProperties> = {
  h1: {
    fontSize: '1.5rem',
    fontWeight: 'bold',
    marginBottom: '1rem',
    marginTop: '1.5rem',
    color: colors.textPrimary,
  },
  h2: {
    fontSize: '1.25rem',
    fontWeight: 600,
    marginBottom: '0.75rem',
    marginTop: '1.25rem',
    color: colors.textPrimary,
  },
  h3: {
    fontSize: '1.125rem',
    fontWeight: 500,
    marginBottom: '0.5rem',
    marginTop: '1rem',
    color: colors.textPrimary,
  },
  p: {
    marginBottom: '1rem',
  },
  ul: {
    listStyleType: 'disc',
    paddingLeft: '1.5rem',
    marginBottom: '1rem',
  },
  ol: {
    listStyleType: 'decimal',
    paddingLeft: '1.5rem',
    marginBottom: '1rem',
  },
  li: {
    marginBottom: '0.25rem',
  },
  code: {
    fontFamily: fonts.mono,
    backgroundColor: colors.bgSurface,
    color: colors.textPrimary,
    padding: '0.125rem 0.25rem',
    borderRadius: '0.25rem',
    fontSize: '0.875rem',
    border: `1px solid ${colors.border}`,
  },
  pre: {
    backgroundColor: colors.bgSurface,
    color: colors.textPrimary,
    padding: '1rem',
    borderRadius: '0.375rem',
    overflowX: 'auto',
    marginBottom: '1rem',
    border: `1px solid ${colors.border}`,
  },
  a: {
    color: colors.accent,
    textDecoration: 'underline',
  },
  strong: {
    fontWeight: 700,
    color: colors.textPrimary,
  },
  blockquote: {
    borderLeft: `3px solid ${colors.border}`,
    paddingLeft: '0.75rem',
    margin: '0 0 1rem 0',
    color: colors.textMuted,
  },
  hr: {
    border: 0,
    borderTop: `1px solid ${colors.border}`,
    margin: '1rem 0',
  },
};

export function MarkdownView({ content }: { content: string }) {
  return (
    <div style={markdownStyles} className="markdown-content">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h1: ({ children }) => <h1 style={componentStyles.h1}>{children}</h1>,
          h2: ({ children }) => <h2 style={componentStyles.h2}>{children}</h2>,
          h3: ({ children }) => <h3 style={componentStyles.h3}>{children}</h3>,
          p: ({ children }) => <p style={componentStyles.p}>{children}</p>,
          ul: ({ children }) => <ul style={componentStyles.ul}>{children}</ul>,
          ol: ({ children }) => <ol style={componentStyles.ol}>{children}</ol>,
          li: ({ children }) => <li style={componentStyles.li}>{children}</li>,
          code: ({ className, children, ...props }) => {
            const isInline = !className;
            if (isInline) {
              return (
                <code style={componentStyles.code} {...props}>
                  {children}
                </code>
              );
            }
            return (
              <code
                style={{
                  ...componentStyles.code,
                  backgroundColor: 'transparent',
                  border: 0,
                  padding: 0,
                  fontSize: '0.875rem',
                }}
                {...props}
              >
                {children}
              </code>
            );
          },
          pre: ({ children }) => (
            <pre style={componentStyles.pre}>{children}</pre>
          ),
          a: ({ href, children }) => (
            <a href={href} style={componentStyles.a}>
              {children}
            </a>
          ),
          strong: ({ children }) => (
            <strong style={componentStyles.strong}>{children}</strong>
          ),
          em: ({ children }) => <em>{children}</em>,
          blockquote: ({ children }) => (
            <blockquote style={componentStyles.blockquote}>{children}</blockquote>
          ),
          hr: () => <hr style={componentStyles.hr} />,
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
