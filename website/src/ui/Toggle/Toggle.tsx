import React from 'react';
import styles from './Toggle.module.css';

export interface ToggleProps {
  /** Current toggle state */
  checked?: boolean;
  /** Callback when toggle state changes */
  onChange?: (checked: boolean) => void;
  /** Toggle size variant */
  size?: 'sm' | 'md';
  /** Visual variant */
  variant?: 'default' | 'branded';
  /** Label text (for accessibility, can be visually hidden) */
  label: string;
  /** Label placement */
  labelPlacement?: 'left' | 'right' | 'hidden';
  /** Disabled state */
  disabled?: boolean;
  /** HTML id for the input */
  id?: string;
  /** Additional class name */
  className?: string;
}

/**
 * Toggle switch component for binary on/off states.
 *
 * Follows accessibility best practices with proper ARIA attributes
 * and minimum touch target sizes.
 *
 * @example
 * ```tsx
 * <Toggle
 *   label="Dark mode"
 *   checked={isDark}
 *   onChange={setIsDark}
 * />
 * ```
 */
export function Toggle({
  checked = false,
  onChange,
  size = 'md',
  variant = 'default',
  label,
  labelPlacement = 'right',
  disabled = false,
  id,
  className = '',
}: ToggleProps) {
  const inputId = id || `toggle-${label.toLowerCase().replace(/\s+/g, '-')}`;

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!disabled) {
      onChange?.(e.target.checked);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      if (!disabled) {
        onChange?.(!checked);
      }
    }
  };

  const containerClasses = [
    styles.container,
    styles[size],
    styles[variant],
    disabled ? styles.disabled : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  const labelClasses = [
    styles.label,
    labelPlacement === 'hidden' ? styles.visuallyHidden : '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <label className={containerClasses}>
      {labelPlacement === 'left' && <span className={labelClasses}>{label}</span>}

      <input
        type="checkbox"
        id={inputId}
        checked={checked}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        className={styles.input}
        role="switch"
        aria-checked={checked}
      />

      <span className={styles.track} aria-hidden="true">
        <span className={styles.thumb} />
      </span>

      {labelPlacement === 'right' && <span className={labelClasses}>{label}</span>}
      {labelPlacement === 'hidden' && <span className={labelClasses}>{label}</span>}
    </label>
  );
}

export default Toggle;
