import { useState, useCallback, useRef, useEffect } from 'react';

export interface UseClipboardOptions {
  /** Duration in ms to show "copied" state (default: 2000) */
  resetDelay?: number;
  /** Callback on successful copy */
  onCopy?: (text: string) => void;
  /** Callback on copy error */
  onError?: (error: Error) => void;
}

export interface UseClipboardReturn {
  /** Whether content was recently copied */
  copied: boolean;
  /** Copy text to clipboard */
  copy: (text: string) => Promise<boolean>;
  /** Reset copied state manually */
  reset: () => void;
}

/**
 * Hook for clipboard operations with copy feedback state.
 *
 * @example
 * ```tsx
 * const { copied, copy } = useClipboard();
 *
 * <button onClick={() => copy('Hello')}>
 *   {copied ? 'Copied!' : 'Copy'}
 * </button>
 * ```
 */
export function useClipboard(
  options: UseClipboardOptions = {}
): UseClipboardReturn {
  const { resetDelay = 2000, onCopy, onError } = options;
  const [copied, setCopied] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const reset = useCallback(() => {
    setCopied(false);
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  const copy = useCallback(
    async (text: string): Promise<boolean> => {
      try {
        await navigator.clipboard.writeText(text);
        setCopied(true);
        onCopy?.(text);

        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current);
        }
        timeoutRef.current = setTimeout(() => {
          setCopied(false);
          timeoutRef.current = null;
        }, resetDelay);

        return true;
      } catch (err) {
        const error =
          err instanceof Error ? err : new Error('Failed to copy to clipboard');
        onError?.(error);
        return false;
      }
    },
    [resetDelay, onCopy, onError]
  );

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return { copied, copy, reset };
}
