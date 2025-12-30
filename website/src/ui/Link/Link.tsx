import React from 'react';
import styles from './Link.module.css';

// ============================================
// TYPES
// ============================================

export type LinkVariant = 'default' | 'support' | 'arrow' | 'external' | 'nav';

export interface LinkProps
  extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  /** Link URL */
  href: string;
  /** Visual variant */
  variant?: LinkVariant;
  /** Force external link behavior */
  external?: boolean;
  /** Link content */
  children: React.ReactNode;
  /** Additional class name */
  className?: string;
}

// ============================================
// ICONS
// ============================================

const ArrowIcon = () => (
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
    <line x1="5" y1="12" x2="19" y2="12" />
    <polyline points="12 5 19 12 12 19" />
  </svg>
);

const ExternalIcon = () => (
  <svg
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
    <polyline points="15 3 21 3 21 9" />
    <line x1="10" y1="14" x2="21" y2="3" />
  </svg>
);

// ============================================
// LINK COMPONENT
// ============================================

/**
 * Styled link component with multiple variants.
 *
 * @example
 * ```tsx
 * // Default link
 * <Link href="/features">Features</Link>
 *
 * // Support/sponsor link (pink)
 * <Link href="/support" variant="support">
 *   ♥ Support
 * </Link>
 *
 * // Arrow link (with animated arrow)
 * <Link href="/blog" variant="arrow">
 *   Read more
 * </Link>
 *
 * // External link (with icon)
 * <Link href="https://github.com" external>
 *   GitHub
 * </Link>
 * ```
 */
export function Link({
  href,
  variant = 'default',
  external,
  children,
  className = '',
  ...props
}: LinkProps) {
  // Auto-detect external links
  const isExternal =
    external !== undefined
      ? external
      : href.startsWith('http') || href.startsWith('//');

  const linkClasses = [styles.link, styles[variant], className]
    .filter(Boolean)
    .join(' ');

  const externalProps = isExternal
    ? {
        target: '_blank',
        rel: 'noopener noreferrer',
      }
    : {};

  const showArrow = variant === 'arrow';
  const showExternal = variant === 'external' || (isExternal && variant === 'default');

  return (
    <a href={href} className={linkClasses} {...externalProps} {...props}>
      <span className={styles.content}>{children}</span>
      {showArrow && (
        <span className={styles.arrow}>
          <ArrowIcon />
        </span>
      )}
      {showExternal && (
        <span className={styles.externalIcon}>
          <ExternalIcon />
        </span>
      )}
    </a>
  );
}

export default Link;
