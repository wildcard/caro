import React, { useState, useCallback } from 'react';
import styles from './CopyCodeBlock.module.css';

// ============================================
// TYPES
// ============================================

export type CopyCodeBlockVariant = 'light' | 'dark' | 'brand';
export type CopyCodeBlockSize = 'sm' | 'md' | 'lg';

export interface CopyCodeBlockProps {
  /** Code content to display and copy */
  code: string;
  /** Visual variant */
  variant?: CopyCodeBlockVariant;
  /** Size variant */
  size?: CopyCodeBlockSize;
  /** Whether to show line numbers */
  showLineNumbers?: boolean;
  /** Language label (displayed in header if provided) */
  language?: string;
  /** Custom copy button text */
  copyLabel?: string;
  /** Custom copied feedback text */
  copiedLabel?: string;
  /** Disable copy functionality */
  disableCopy?: boolean;
  /** Additional class name */
  className?: string;
}

// ============================================
// COPY CODE BLOCK COMPONENT
// ============================================

/**
 * Code block with copy-to-clipboard functionality.
 *
 * @example
 * ```tsx
 * // Simple usage
 * <CopyCodeBlock code="cargo install caro" />
 *
 * // With language label
 * <CopyCodeBlock
 *   code="npm install @caro/sdk"
 *   language="bash"
 *   variant="dark"
 * />
 *
 * // Brand variant for install commands
 * <CopyCodeBlock
 *   code="bash <(curl -sSfL https://setup.caro.sh)"
 *   variant="brand"
 *   size="lg"
 * />
 * ```
 */
export function CopyCodeBlock({
  code,
  variant = 'dark',
  size = 'md',
  showLineNumbers = false,
  language,
  copyLabel = 'Copy',
  copiedLabel = 'Copied!',
  disableCopy = false,
  className = '',
}: CopyCodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }, [code]);

  const containerClasses = [
    styles.container,
    styles[variant],
    styles[size],
    className,
  ]
    .filter(Boolean)
    .join(' ');

  const lines = showLineNumbers ? code.split('\n') : null;

  return (
    <div className={containerClasses}>
      {/* Header with language and copy button */}
      {(language || !disableCopy) && (
        <div className={styles.header}>
          {language && <span className={styles.language}>{language}</span>}
          {!disableCopy && (
            <button
              className={styles.copyButton}
              onClick={handleCopy}
              aria-label={copied ? copiedLabel : `${copyLabel} code`}
            >
              {copied ? (
                <>
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    aria-hidden="true"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  <span>{copiedLabel}</span>
                </>
              ) : (
                <>
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    aria-hidden="true"
                  >
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                  <span>{copyLabel}</span>
                </>
              )}
            </button>
          )}
        </div>
      )}

      {/* Code content */}
      <div className={styles.codeWrapper}>
        {showLineNumbers && lines ? (
          <pre className={styles.code}>
            <code>
              {lines.map((line, index) => (
                <div key={index} className={styles.line}>
                  <span className={styles.lineNumber}>{index + 1}</span>
                  <span className={styles.lineContent}>{line}</span>
                </div>
              ))}
            </code>
          </pre>
        ) : (
          <pre className={styles.code}>
            <code>{code}</code>
          </pre>
        )}
      </div>
    </div>
  );
}

export default CopyCodeBlock;
