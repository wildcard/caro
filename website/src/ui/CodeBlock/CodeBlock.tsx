import React from 'react';
import { useClipboard } from '../../hooks/useClipboard';
import styles from './CodeBlock.module.css';

export type CodeBlockVariant = 'default' | 'muted';

export interface CodeBlockProps {
  /** Code content to display */
  children: string;
  /** Visual variant */
  variant?: CodeBlockVariant;
  /** Language label for accessibility (not displayed) */
  lang?: string;
  /** Show line numbers for multiline code */
  lineNumbers?: boolean;
  /** Disable copy functionality */
  noCopy?: boolean;
  /** Additional class name */
  className?: string;
}

/**
 * Minimal code block with copy-to-clipboard.
 * Automatically renders as inline or block based on content.
 *
 * @example
 * ```tsx
 * // Inline code (single line)
 * <CodeBlock>npm install caro</CodeBlock>
 *
 * // Block code (multiline)
 * <CodeBlock lang="typescript" lineNumbers>
 * {`const x = 1;
 * const y = 2;`}
 * </CodeBlock>
 * ```
 */
export function CodeBlock({
  children,
  variant = 'default',
  lang,
  lineNumbers = false,
  noCopy = false,
  className = '',
}: CodeBlockProps) {
  const { copied, copy } = useClipboard();
  const isMultiline = children.includes('\n');
  const lines = lineNumbers ? children.split('\n') : null;

  const handleCopy = () => copy(children);

  const copyButton = !noCopy && (
    <button
      type="button"
      className={styles.copyBtn}
      onClick={handleCopy}
      aria-label={copied ? 'Copied to clipboard' : 'Copy code'}
    >
      {copied ? (
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
      ) : (
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <rect x="9" y="9" width="13" height="13" rx="2" />
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
      )}
    </button>
  );

  // Inline code (single line)
  if (!isMultiline) {
    return (
      <span
        className={`${styles.inline} ${styles[variant]} ${className}`.trim()}
        data-lang={lang}
      >
        <code className={styles.code}>{children}</code>
        {copyButton}
      </span>
    );
  }

  // Block code (multiline)
  return (
    <div
      className={`${styles.block} ${styles[variant]} ${className}`.trim()}
      data-lang={lang}
    >
      {copyButton}
      <pre className={styles.pre}>
        {lines ? (
          <code className={styles.code}>
            {lines.map((line, i) => (
              <span key={i} className={styles.line}>
                <span className={styles.lineNum}>{i + 1}</span>
                <span className={styles.lineContent}>{line}</span>
                {i < lines.length - 1 && '\n'}
              </span>
            ))}
          </code>
        ) : (
          <code className={styles.code}>{children}</code>
        )}
      </pre>
    </div>
  );
}

export default CodeBlock;
