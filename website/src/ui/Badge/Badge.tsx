import React from 'react';
import styles from './Badge.module.css';

export interface BadgeProps {
  /** Badge visual variant */
  variant?: 'primary' | 'success' | 'warning' | 'info' | 'neutral';
  /** Badge size */
  size?: 'sm' | 'md';
  /** Icon to display before the badge text */
  icon?: React.ReactNode;
  /** Additional class name */
  className?: string;
  /** Children elements */
  children: React.ReactNode;
}

/**
 * Badge component for status indicators and labels.
 *
 * Used throughout the Caro website for feature status,
 * version tags, and category labels.
 *
 * @example
 * ```tsx
 * <Badge variant="success">Implemented</Badge>
 * <Badge variant="warning">In Progress</Badge>
 * <Badge variant="info" icon="🐕">Alpha</Badge>
 * ```
 */
export function Badge({
  variant = 'primary',
  size = 'md',
  icon,
  className = '',
  children,
}: BadgeProps) {
  const classNames = [styles.badge, styles[variant], styles[size], className]
    .filter(Boolean)
    .join(' ');

  return (
    <span className={classNames}>
      {icon && <span className={styles.icon}>{icon}</span>}
      <span className={styles.content}>{children}</span>
    </span>
  );
}

export default Badge;
