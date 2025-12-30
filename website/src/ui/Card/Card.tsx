import React from 'react';
import styles from './Card.module.css';

// ============================================
// TYPES
// ============================================

export type CardVariant = 'feature' | 'blog' | 'summary';
export type FeatureStatus = 'implemented' | 'in-progress' | 'planned';

export interface CardMetadata {
  /** Publication date for blog cards */
  date?: string;
  /** Read time estimate */
  readTime?: string;
}

export interface CardProps {
  /** Card variant */
  variant?: CardVariant;
  /** Icon or emoji (for feature variant) */
  icon?: React.ReactNode;
  /** Card title */
  title: string;
  /** Card description */
  description: string;
  /** Status badge content */
  badge?: React.ReactNode;
  /** Feature status (affects styling) */
  status?: FeatureStatus;
  /** Status label text */
  statusLabel?: string;
  /** Link URL (makes card clickable) */
  href?: string;
  /** Blog metadata */
  metadata?: CardMetadata;
  /** Click handler */
  onClick?: () => void;
  /** Additional class name */
  className?: string;
  /** Children for custom content */
  children?: React.ReactNode;
}

// ============================================
// STATUS BADGE COMPONENT
// ============================================

interface StatusBadgeProps {
  status: FeatureStatus;
  label: string;
}

function StatusBadge({ status, label }: StatusBadgeProps) {
  return (
    <span
      className={`${styles.statusBadge} ${styles[`status-${status}`]}`}
      role="status"
      aria-label={`Status: ${label}`}
    >
      {label}
    </span>
  );
}

// ============================================
// CARD COMPONENT
// ============================================

/**
 * Card component for features, blog posts, and summaries.
 *
 * @example
 * ```tsx
 * // Feature card
 * <Card
 *   variant="feature"
 *   icon="🛡️"
 *   title="Safety Guardian"
 *   description="Comprehensive validation..."
 *   status="implemented"
 *   statusLabel="Available Now"
 * />
 *
 * // Blog card
 * <Card
 *   variant="blog"
 *   title="Announcing Caro"
 *   description="We're excited to announce..."
 *   badge="NEW"
 *   metadata={{ date: "2025-12-20", readTime: "5 min read" }}
 *   href="/blog/announcing-caro"
 * />
 * ```
 */
export function Card({
  variant = 'feature',
  icon,
  title,
  description,
  badge,
  status,
  statusLabel,
  href,
  metadata,
  onClick,
  className = '',
  children,
}: CardProps) {
  const isClickable = href || onClick;

  const cardClasses = [
    styles.card,
    styles[variant],
    status ? styles[`feature-${status}`] : '',
    isClickable ? styles.clickable : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  const handleClick = () => {
    if (onClick) {
      onClick();
    } else if (href) {
      window.location.href = href;
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.key === 'Enter' || e.key === ' ') && isClickable) {
      e.preventDefault();
      handleClick();
    }
  };

  // Format date for blog cards
  const formattedDate = metadata?.date
    ? new Date(metadata.date).toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      })
    : null;

  const cardContent = (
    <>
      {/* Blog badge */}
      {variant === 'blog' && badge && (
        <div className={styles.blogBadge}>{badge}</div>
      )}

      {/* Feature header with icon and status */}
      {variant === 'feature' && (
        <div className={styles.featureHeader}>
          {icon && <div className={styles.featureIcon}>{icon}</div>}
          {status && statusLabel && (
            <StatusBadge status={status} label={statusLabel} />
          )}
        </div>
      )}

      {/* Blog header with date and read time */}
      {variant === 'blog' && metadata && (
        <div className={styles.blogHeader}>
          {formattedDate && (
            <time dateTime={metadata.date} className={styles.blogDate}>
              {formattedDate}
            </time>
          )}
          {metadata.readTime && (
            <span className={styles.blogReadTime}>{metadata.readTime}</span>
          )}
        </div>
      )}

      {/* Title */}
      <h3 className={styles.title}>
        {variant === 'blog' && href ? (
          <a href={href} className={styles.titleLink}>
            {title}
          </a>
        ) : (
          title
        )}
      </h3>

      {/* Description */}
      <p className={styles.description}>{description}</p>

      {/* Blog read more link */}
      {variant === 'blog' && href && (
        <a href={href} className={styles.readMore}>
          Read full story →
        </a>
      )}

      {/* Custom content */}
      {children}
    </>
  );

  // Render as article for blog, div for others
  const Component = variant === 'blog' ? 'article' : 'div';

  return (
    <Component
      className={cardClasses}
      onClick={isClickable ? handleClick : undefined}
      onKeyDown={isClickable ? handleKeyDown : undefined}
      tabIndex={isClickable ? 0 : undefined}
      role={isClickable ? 'button' : undefined}
      aria-label={isClickable ? title : undefined}
    >
      {cardContent}
    </Component>
  );
}

export default Card;
