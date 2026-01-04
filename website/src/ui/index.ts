/**
 * Caro Design System - Component Library
 *
 * Reusable React components for the Caro website.
 * All components support dark mode and follow accessibility guidelines.
 *
 * @example
 * ```tsx
 * import { Button, Toggle, Badge } from '../ui';
 *
 * <Button variant="primary">Get Started</Button>
 * <Toggle label="Dark mode" checked={isDark} onChange={setIsDark} />
 * <Badge variant="success">Implemented</Badge>
 * ```
 */

// Core Interactive Components
export { Button, type ButtonProps } from './Button';
export { Toggle, type ToggleProps } from './Toggle';

// Display Components
export { Badge, type BadgeProps } from './Badge';
export {
  Terminal,
  TerminalLine,
  type TerminalProps,
  type TerminalLineProps,
  type TerminalExample,
  type LineType,
} from './Terminal';
export {
  Card,
  type CardProps,
  type CardVariant,
  type CardMetadata,
  type FeatureStatus,
} from './Card';
export {
  CopyCodeBlock,
  type CopyCodeBlockProps,
  type CopyCodeBlockVariant,
  type CopyCodeBlockSize,
} from './CopyCodeBlock';
export {
  CodeBlock,
  type CodeBlockProps,
  type CodeBlockVariant,
} from './CodeBlock';
export {
  IconButton,
  type IconButtonProps,
  type IconButtonSize,
  type IconButtonVariant,
} from './IconButton';
export { Link, type LinkProps, type LinkVariant } from './Link';
export {
  Dropdown,
  DropdownTrigger,
  DropdownPanel,
  DropdownItem,
  DropdownDivider,
  DropdownHeader,
  type DropdownProps,
  type DropdownTriggerProps,
  type DropdownPanelProps,
  type DropdownItemProps,
  type DropdownHeaderProps,
} from './Dropdown';

// Design tokens are available via CSS import:
// import '../ui/tokens.css';
