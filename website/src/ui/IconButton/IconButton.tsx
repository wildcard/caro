import React from 'react';
import styles from './IconButton.module.css';

// ============================================
// TYPES
// ============================================

export type IconButtonSize = 'sm' | 'md' | 'lg';
export type IconButtonVariant = 'default' | 'ghost' | 'brand';

export interface IconButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Icon element to display */
  icon: React.ReactNode;
  /** Accessible label (required for accessibility) */
  label: string;
  /** Size variant */
  size?: IconButtonSize;
  /** Visual variant */
  variant?: IconButtonVariant;
  /** Active state (e.g., for toggles) */
  active?: boolean;
  /** Loading state */
  loading?: boolean;
  /** Additional class name */
  className?: string;
}

// ============================================
// ICON BUTTON COMPONENT
// ============================================

/**
 * Square icon-only button for actions like theme toggle, menu trigger, etc.
 *
 * @example
 * ```tsx
 * // Theme toggle
 * <IconButton
 *   icon={<SunIcon />}
 *   label="Toggle dark mode"
 *   onClick={toggleTheme}
 * />
 *
 * // Active state
 * <IconButton
 *   icon={<MoonIcon />}
 *   label="Dark mode enabled"
 *   active={isDark}
 * />
 * ```
 */
export function IconButton({
  icon,
  label,
  size = 'md',
  variant = 'default',
  active = false,
  loading = false,
  disabled = false,
  className = '',
  ...props
}: IconButtonProps) {
  const buttonClasses = [
    styles.iconButton,
    styles[size],
    styles[variant],
    active ? styles.active : '',
    loading ? styles.loading : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <button
      type="button"
      className={buttonClasses}
      disabled={disabled || loading}
      aria-label={label}
      title={label}
      {...props}
    >
      {loading ? (
        <svg
          className={styles.spinner}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden="true"
        >
          <circle cx="12" cy="12" r="10" strokeOpacity="0.25" />
          <path d="M12 2a10 10 0 0 1 10 10" strokeLinecap="round" />
        </svg>
      ) : (
        <span className={styles.iconWrapper} aria-hidden="true">
          {icon}
        </span>
      )}
    </button>
  );
}

export default IconButton;
