/**
 * CategoryTag Component
 * ======================
 * Interactive tag/pill component for categories and tags.
 * Outlined with accent colors, interactive hover states.
 */

import React from 'react';
import styles from './CategoryTag.module.css';

export interface CategoryTagProps {
  label: string;
  href?: string;
  variant?: 'default' | 'primary' | 'blue' | 'coral';
  size?: 'sm' | 'md' | 'lg';
  interactive?: boolean;
  selected?: boolean;
  onClick?: () => void;
}

export function CategoryTag({
  label,
  href,
  variant = 'default',
  size = 'md',
  interactive = true,
  selected = false,
  onClick,
}: CategoryTagProps) {
  const classNames = [
    styles.tag,
    styles[variant],
    styles[size],
    interactive ? styles.interactive : '',
    selected ? styles.selected : '',
  ]
    .filter(Boolean)
    .join(' ');

  const content = <span className={styles.label}>{label}</span>;

  if (href) {
    return (
      <a href={href} className={classNames}>
        {content}
      </a>
    );
  }

  if (onClick) {
    return (
      <button type="button" className={classNames} onClick={onClick}>
        {content}
      </button>
    );
  }

  return <span className={classNames}>{content}</span>;
}

export default CategoryTag;
