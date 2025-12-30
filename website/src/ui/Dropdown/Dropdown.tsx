import React, { createContext, useContext, useState, useRef, useEffect, useCallback } from 'react';
import styles from './Dropdown.module.css';

// ============================================
// CONTEXT
// ============================================

interface DropdownContextValue {
  isOpen: boolean;
  setIsOpen: (open: boolean) => void;
  triggerId: string;
  panelId: string;
}

const DropdownContext = createContext<DropdownContextValue | null>(null);

function useDropdownContext() {
  const context = useContext(DropdownContext);
  if (!context) {
    throw new Error('Dropdown components must be used within a Dropdown');
  }
  return context;
}

// ============================================
// TYPES
// ============================================

export interface DropdownProps {
  /** Controlled open state */
  open?: boolean;
  /** Callback when open state changes */
  onOpenChange?: (open: boolean) => void;
  /** Close dropdown when clicking outside */
  closeOnClickOutside?: boolean;
  /** Close dropdown when pressing Escape */
  closeOnEscape?: boolean;
  /** Children (trigger and panel) */
  children: React.ReactNode;
  /** Additional class name */
  className?: string;
}

export interface DropdownTriggerProps {
  /** Trigger content */
  children: React.ReactNode;
  /** Show chevron indicator */
  showChevron?: boolean;
  /** Additional class name */
  className?: string;
}

export interface DropdownPanelProps {
  /** Panel content */
  children: React.ReactNode;
  /** Panel alignment */
  align?: 'left' | 'center' | 'right';
  /** Additional class name */
  className?: string;
}

export interface DropdownItemProps {
  /** Icon (emoji or component) */
  icon?: React.ReactNode;
  /** Item title */
  title: string;
  /** Item description */
  description?: string;
  /** Link URL */
  href?: string;
  /** External link */
  external?: boolean;
  /** Click handler */
  onClick?: () => void;
  /** Additional class name */
  className?: string;
}

// ============================================
// DROPDOWN (CONTAINER)
// ============================================

let dropdownCounter = 0;

/**
 * Dropdown container component using compound component pattern.
 *
 * @example
 * ```tsx
 * <Dropdown>
 *   <DropdownTrigger showChevron>
 *     Compare
 *   </DropdownTrigger>
 *   <DropdownPanel>
 *     <DropdownItem
 *       icon="📊"
 *       title="Overview"
 *       description="See all comparisons"
 *       href="/compare"
 *     />
 *   </DropdownPanel>
 * </Dropdown>
 * ```
 */
export function Dropdown({
  open: controlledOpen,
  onOpenChange,
  closeOnClickOutside = true,
  closeOnEscape = true,
  children,
  className = '',
}: DropdownProps) {
  const [internalOpen, setInternalOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const idRef = useRef<number>(++dropdownCounter);

  const isOpen = controlledOpen !== undefined ? controlledOpen : internalOpen;

  const setIsOpen = useCallback(
    (open: boolean) => {
      if (onOpenChange) {
        onOpenChange(open);
      } else {
        setInternalOpen(open);
      }
    },
    [onOpenChange]
  );

  // Handle click outside
  useEffect(() => {
    if (!closeOnClickOutside || !isOpen) return;

    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen, closeOnClickOutside, setIsOpen]);

  // Handle escape key
  useEffect(() => {
    if (!closeOnEscape || !isOpen) return;

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsOpen(false);
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, closeOnEscape, setIsOpen]);

  const contextValue: DropdownContextValue = {
    isOpen,
    setIsOpen,
    triggerId: `dropdown-trigger-${idRef.current}`,
    panelId: `dropdown-panel-${idRef.current}`,
  };

  return (
    <DropdownContext.Provider value={contextValue}>
      <div
        ref={containerRef}
        className={`${styles.dropdown} ${isOpen ? styles.open : ''} ${className}`}
      >
        {children}
      </div>
    </DropdownContext.Provider>
  );
}

// ============================================
// DROPDOWN TRIGGER
// ============================================

export function DropdownTrigger({
  children,
  showChevron = true,
  className = '',
}: DropdownTriggerProps) {
  const { isOpen, setIsOpen, triggerId, panelId } = useDropdownContext();

  const handleClick = () => {
    setIsOpen(!isOpen);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      setIsOpen(!isOpen);
    }
    if (e.key === 'ArrowDown' && !isOpen) {
      e.preventDefault();
      setIsOpen(true);
    }
  };

  return (
    <button
      id={triggerId}
      className={`${styles.trigger} ${className}`}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      aria-expanded={isOpen}
      aria-haspopup="true"
      aria-controls={panelId}
    >
      {children}
      {showChevron && (
        <svg
          className={`${styles.chevron} ${isOpen ? styles.chevronOpen : ''}`}
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
          aria-hidden="true"
        >
          <path
            d="M3 4.5L6 7.5L9 4.5"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      )}
    </button>
  );
}

// ============================================
// DROPDOWN PANEL
// ============================================

export function DropdownPanel({
  children,
  align = 'center',
  className = '',
}: DropdownPanelProps) {
  const { isOpen, panelId, triggerId } = useDropdownContext();

  if (!isOpen) return null;

  return (
    <div
      id={panelId}
      className={`${styles.panel} ${styles[`align-${align}`]} ${className}`}
      role="menu"
      aria-labelledby={triggerId}
    >
      {children}
    </div>
  );
}

// ============================================
// DROPDOWN ITEM
// ============================================

export function DropdownItem({
  icon,
  title,
  description,
  href,
  external = false,
  onClick,
  className = '',
}: DropdownItemProps) {
  const { setIsOpen } = useDropdownContext();

  const handleClick = () => {
    onClick?.();
    setIsOpen(false);
  };

  const content = (
    <>
      {icon && <span className={styles.itemIcon}>{icon}</span>}
      <div className={styles.itemContent}>
        <span className={styles.itemTitle}>{title}</span>
        {description && (
          <span className={styles.itemDescription}>{description}</span>
        )}
      </div>
    </>
  );

  if (href) {
    return (
      <a
        href={href}
        className={`${styles.item} ${className}`}
        role="menuitem"
        target={external ? '_blank' : undefined}
        rel={external ? 'noopener noreferrer' : undefined}
        onClick={handleClick}
      >
        {content}
      </a>
    );
  }

  return (
    <button
      className={`${styles.item} ${className}`}
      role="menuitem"
      onClick={handleClick}
    >
      {content}
    </button>
  );
}

// ============================================
// DROPDOWN DIVIDER
// ============================================

export function DropdownDivider() {
  return <div className={styles.divider} role="separator" />;
}

// ============================================
// DROPDOWN HEADER
// ============================================

export interface DropdownHeaderProps {
  children: React.ReactNode;
  className?: string;
}

export function DropdownHeader({ children, className = '' }: DropdownHeaderProps) {
  return (
    <div className={`${styles.header} ${className}`} role="presentation">
      {children}
    </div>
  );
}

export default Dropdown;
