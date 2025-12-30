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

// Design tokens are available via CSS import:
// import '../ui/tokens.css';
