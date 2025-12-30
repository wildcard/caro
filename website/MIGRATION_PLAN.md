# Caro Component Library Migration Plan

## Overview

This document outlines the migration strategy for extracting reusable UI components from Astro components into a React-based Storybook component library.

## Current State

### Existing React Components (in `src/ui/`)
| Component | Status | Notes |
|-----------|--------|-------|
| Button | ✅ Done | primary, secondary, ghost, support variants |
| Toggle | ⚠️ Needs Fix | Doesn't match website behavior |
| Badge | ✅ Done | primary, success, warning, info, neutral variants |

### Components to Add
Based on analysis of 53 Astro components, the following UI patterns need extraction:

## Migration Phases

### Phase 1: Core Interactive Components (Priority: High)

#### 1.1 Toggle (Fix)
- **Issue**: Current implementation doesn't match Navigation.astro styling
- **Fix**: Update track/thumb sizes, colors, transitions
- **Desktop**: 52px × 28px track, 24px thumb
- **Mobile**: 44px × 24px track, 20px thumb

#### 1.2 Dropdown
- **Source**: Navigation.astro (`.dropdown-panel`, `.dropdown-item`)
- **Features**:
  - Trigger button with chevron
  - Panel with rich content items
  - Icon + title + description per item
  - Hover states, transitions
- **Variants**: Default, with icons, mobile sheet

#### 1.3 Terminal
- **Source**: Terminal.astro, TerminalShowcase.astro
- **Features**:
  - Macbook-style header with 3 dots (red, yellow, green)
  - Dark background (#1e1e1e)
  - Syntax-highlighted content
  - Copy functionality option
  - Animated content option
- **Variants**: Static, animated, with copy button

#### 1.4 IconButton
- **Source**: Navigation.astro (`.icon-button`)
- **Features**:
  - Square 40px × 40px
  - Icon-only content
  - Border + background styling
  - Hover states
- **Variants**: Default, active (for toggles like theme)

### Phase 2: Display Components (Priority: Medium)

#### 2.1 Card
- **Source**: Features.astro, Blog.astro, Comparison.astro
- **Features**:
  - Icon/emoji header
  - Title + description
  - Optional status badge
  - Hover lift effect
- **Variants**: Feature, blog (with metadata), summary

#### 2.2 CopyCodeBlock
- **Source**: Hero.astro, Download.astro
- **Features**:
  - Monospace code display
  - Copy button with "Copied!" feedback
  - Dark/light variants
- **Variants**: Inline, block

#### 2.3 Link
- **Source**: Navigation.astro, Footer.astro
- **Features**:
  - Text link styling
  - Hover color change
  - Optional arrow animation
- **Variants**: Default, support (pink), arrow, external

### Phase 3: Layout Components (Priority: Lower)

#### 3.1 Section
- **Features**:
  - Title + subtitle
  - Background variants
  - Responsive container

#### 3.2 FlowStep
- **Source**: SafetyShowcase.astro
- **Features**:
  - Numbered circle badge
  - Title + description
  - Arrow connector

## Component API Design

### Button (Existing - Verified)
```tsx
<Button
  variant="primary|secondary|ghost|support"
  size="sm|md|lg"
  fullWidth={boolean}
  leftIcon={ReactNode}
  rightIcon={ReactNode}
  loading={boolean}
/>
```

### Toggle (To Fix)
```tsx
<Toggle
  checked={boolean}
  onChange={(checked) => void}
  size="sm|md"  // sm=44×24, md=52×28
  variant="default|branded"  // default=green, branded=orange
  label={string}
  labelPlacement="left|right|hidden"
/>
```

### Dropdown (New)
```tsx
<Dropdown>
  <DropdownTrigger>{trigger}</DropdownTrigger>
  <DropdownPanel>
    <DropdownItem
      icon={ReactNode}
      title={string}
      description={string}
      href={string}
    />
  </DropdownPanel>
</Dropdown>
```

### Terminal (New)
```tsx
<Terminal
  title={string}
  variant="default|animated"
  showCopy={boolean}
>
  <TerminalLine type="command|output|status" color={string}>
    {content}
  </TerminalLine>
</Terminal>
```

### Card (New)
```tsx
<Card
  variant="feature|blog|summary"
  icon={ReactNode}
  title={string}
  description={string}
  badge={ReactNode}
  href={string}
  metadata={{ date, readTime }}  // for blog variant
/>
```

### CopyCodeBlock (New)
```tsx
<CopyCodeBlock
  code={string}
  language={string}
  variant="inline|block"
  showLineNumbers={boolean}
/>
```

### IconButton (New)
```tsx
<IconButton
  icon={ReactNode}
  label={string}  // aria-label
  active={boolean}
  variant="default|theme"
/>
```

### Link (New)
```tsx
<Link
  href={string}
  variant="default|support|arrow|external"
  external={boolean}
>
  {children}
</Link>
```

## Design Tokens Reference

From `src/ui/tokens.css`:

```css
/* Brand */
--color-primary: #ff8c42
--color-primary-dark: #ff6b35
--color-support: #ea4aaa

/* Status */
--color-success: #22c55e
--color-warning: #f59e0b
--color-error: #ef4444
--color-info: #3b82f6

/* Terminal */
--terminal-bg: #1e1e1e
--terminal-dot-red: #ff5f56
--terminal-dot-yellow: #ffbd2e
--terminal-dot-green: #27c93f

/* Touch Targets */
--touch-target-min: 44px
--touch-target-recommended: 48px
```

## Integration Strategy

### Step 1: Build in Storybook
1. Create React component with CSS Module
2. Write stories for all variants
3. Test responsive behavior with viewport addon
4. Verify accessibility with a11y addon
5. Test dark mode support

### Step 2: Export from UI Library
```tsx
// src/ui/index.ts
export { Button } from './Button';
export { Toggle } from './Toggle';
export { Dropdown, DropdownTrigger, DropdownPanel, DropdownItem } from './Dropdown';
export { Terminal, TerminalLine } from './Terminal';
export { Card } from './Card';
export { CopyCodeBlock } from './CopyCodeBlock';
export { IconButton } from './IconButton';
export { Link } from './Link';
export { Badge } from './Badge';
```

### Step 3: Use in Astro Components
```astro
---
import { Button, Card, Terminal } from '../ui';
---

<Button client:load variant="primary" size="lg">
  Get Started
</Button>
```

**Note**: Components with interactivity need `client:load` or `client:visible` directive.

## File Structure

```
src/ui/
├── tokens.css              # Design tokens (existing)
├── index.ts                # Barrel export (existing)
│
├── Button/                 # ✅ Existing
├── Toggle/                 # ⚠️ Needs fix
├── Badge/                  # ✅ Existing
│
├── Dropdown/               # 🆕 New
│   ├── Dropdown.tsx
│   ├── Dropdown.module.css
│   ├── Dropdown.stories.tsx
│   └── index.ts
│
├── Terminal/               # 🆕 New
│   ├── Terminal.tsx
│   ├── Terminal.module.css
│   ├── Terminal.stories.tsx
│   └── index.ts
│
├── Card/                   # 🆕 New
│   ├── Card.tsx
│   ├── Card.module.css
│   ├── Card.stories.tsx
│   └── index.ts
│
├── CopyCodeBlock/          # 🆕 New
│   ├── CopyCodeBlock.tsx
│   ├── CopyCodeBlock.module.css
│   ├── CopyCodeBlock.stories.tsx
│   └── index.ts
│
├── IconButton/             # 🆕 New
│   ├── IconButton.tsx
│   ├── IconButton.module.css
│   ├── IconButton.stories.tsx
│   └── index.ts
│
└── Link/                   # 🆕 New
    ├── Link.tsx
    ├── Link.module.css
    ├── Link.stories.tsx
    └── index.ts
```

## Implementation Order

1. **Fix Toggle** - Match website styling (30 min)
2. **Add Terminal** - High visual impact, frequently used (1 hr)
3. **Add Card** - Multiple variants needed (1 hr)
4. **Add Dropdown** - Complex but essential for nav (1 hr)
5. **Add CopyCodeBlock** - Utility component (30 min)
6. **Add IconButton** - Simple extraction (30 min)
7. **Add Link** - Simple extraction (30 min)

## Success Criteria

For each component:
- [ ] Stories for all variants
- [ ] Mobile viewport story
- [ ] Dark mode story
- [ ] Accessibility check passes
- [ ] TypeScript props interface
- [ ] CSS module (no inline styles)
- [ ] Documentation in story
- [ ] Works in Astro with client directive
