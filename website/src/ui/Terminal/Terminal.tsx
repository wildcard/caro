import React, { useState, useEffect, useCallback } from 'react';
import styles from './Terminal.module.css';

// ============================================
// TYPES
// ============================================

export type LineType = 'command' | 'output' | 'status' | 'caro';

export interface TerminalLineProps {
  /** Type of line - affects styling */
  type: LineType;
  /** Line content */
  children: React.ReactNode;
  /** Custom prefix (e.g., "$" for commands, "🐕 Caro:" for output) */
  prefix?: React.ReactNode;
}

export interface TerminalExample {
  /** Command line content */
  command: string;
  /** Caro's output */
  output: string;
  /** Safety status message */
  status?: string;
}

export interface TerminalProps {
  /** Optional title shown in header */
  title?: string;
  /** Visual variant */
  variant?: 'default' | 'animated';
  /** Show copy button */
  showCopy?: boolean;
  /** Code to copy (if different from children content) */
  copyText?: string;
  /** Terminal content - either children or examples for animated */
  children?: React.ReactNode;
  /** Examples for animated variant */
  examples?: TerminalExample[];
  /** Animation interval in ms (default: 4000) */
  animationInterval?: number;
  /** Additional class name */
  className?: string;
}

// ============================================
// TERMINAL LINE COMPONENT
// ============================================

/**
 * Single line within a Terminal.
 * Supports different line types with appropriate styling.
 */
export function TerminalLine({ type, children, prefix }: TerminalLineProps) {
  const defaultPrefixes: Record<LineType, React.ReactNode> = {
    command: '$',
    output: '',
    status: '✓',
    caro: '🐕 Caro:',
  };

  const displayPrefix = prefix !== undefined ? prefix : defaultPrefixes[type];

  return (
    <div className={`${styles.line} ${styles[type]}`}>
      {displayPrefix && (
        <span className={styles.prefix}>{displayPrefix}</span>
      )}
      <span className={styles.content}>{children}</span>
    </div>
  );
}

// ============================================
// TERMINAL COMPONENT
// ============================================

/**
 * Terminal window component with macOS-style header.
 *
 * Supports static content or animated examples that cycle automatically.
 *
 * @example
 * ```tsx
 * // Static terminal
 * <Terminal title="caro — shell companion">
 *   <TerminalLine type="command">caro "list files"</TerminalLine>
 *   <TerminalLine type="caro">ls -la</TerminalLine>
 *   <TerminalLine type="status">Safe to run</TerminalLine>
 * </Terminal>
 *
 * // Animated terminal
 * <Terminal
 *   variant="animated"
 *   examples={[
 *     { command: 'caro "list files"', output: 'ls -la', status: 'Safe' },
 *     { command: 'caro "disk usage"', output: 'du -sh *', status: 'Safe' },
 *   ]}
 * />
 * ```
 */
export function Terminal({
  title,
  variant = 'default',
  showCopy = false,
  copyText,
  children,
  examples = [],
  animationInterval = 4000,
  className = '',
}: TerminalProps) {
  const [copied, setCopied] = useState(false);
  const [currentExampleIndex, setCurrentExampleIndex] = useState(0);

  // Handle copy
  const handleCopy = useCallback(async () => {
    const textToCopy = copyText || '';
    try {
      await navigator.clipboard.writeText(textToCopy);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }, [copyText]);

  // Animate through examples
  useEffect(() => {
    if (variant !== 'animated' || examples.length <= 1) return;

    const timer = setInterval(() => {
      setCurrentExampleIndex((prev) => (prev + 1) % examples.length);
    }, animationInterval);

    return () => clearInterval(timer);
  }, [variant, examples.length, animationInterval]);

  const containerClasses = [
    styles.terminal,
    variant === 'animated' ? styles.animated : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div className={containerClasses}>
      {/* Header with dots */}
      <div className={styles.header}>
        <div className={styles.dots}>
          <span className={`${styles.dot} ${styles.red}`} />
          <span className={`${styles.dot} ${styles.yellow}`} />
          <span className={`${styles.dot} ${styles.green}`} />
        </div>
        {title && <div className={styles.title}>{title}</div>}
        {showCopy && (
          <button
            className={styles.copyButton}
            onClick={handleCopy}
            aria-label={copied ? 'Copied!' : 'Copy to clipboard'}
          >
            {copied ? (
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            ) : (
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            )}
          </button>
        )}
      </div>

      {/* Body */}
      <div className={styles.body}>
        {variant === 'animated' && examples.length > 0 ? (
          <div className={styles.examples}>
            {examples.map((example, index) => (
              <div
                key={index}
                className={`${styles.example} ${index === currentExampleIndex ? styles.active : ''}`}
              >
                <TerminalLine type="command">{example.command}</TerminalLine>
                <TerminalLine type="caro">{example.output}</TerminalLine>
                {example.status && (
                  <TerminalLine type="status">{example.status}</TerminalLine>
                )}
              </div>
            ))}
          </div>
        ) : (
          children
        )}
      </div>
    </div>
  );
}

export default Terminal;
