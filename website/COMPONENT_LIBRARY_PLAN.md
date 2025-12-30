# Caro Component Library Plan

## Overview

This document outlines the plan to extract reusable UI components from the Caro website into a dedicated Storybook component library. The goal is to create a consistent, playful, and modern design system that reflects Caro's brand identity as a friendly shell companion.

## Current State

The website uses Astro components with embedded styles. Key patterns identified:

### Existing Component Patterns

| Pattern | Location | Notes |
|---------|----------|-------|
| **Buttons** | Hero, Download, Navigation | Multiple variants: primary (gradient), secondary (outline), nav CTA |
| **Toggle Switches** | Navigation (theme, snow) | Custom track/thumb implementation |
| **Dropdowns** | Navigation (desktop) | Rich content with icons/descriptions |
| **Cards** | Features, Blog, Download modes | Various padding/styling |
| **Badges** | Hero (companion), Features (alpha/status) | Pill-shaped labels |
| **Terminal Windows** | TerminalShowcase | Macbook-style with dots |
| **Links** | Footer, Navigation | Standard and support (pink) variants |
| **Input/Code blocks** | Download (copy command) | Monospace with copy functionality |

### UX Issues to Address

1. **Inconsistent toggle sizing**: Desktop uses 50px, should be 44px on mobile
2. **Button touch targets**: Some buttons under 44px minimum
3. **Text alignment**: Overuse of center alignment on paragraphs
4. **Missing breakpoints**: Many components lack 600px/360px responsive rules

## Component Library Architecture

### Recommended Stack

```
website/
├── src/
│   ├── components/
│   │   └── (existing Astro components)
│   └── ui/                    # NEW: React component library
│       ├── Button/
│       │   ├── Button.tsx
│       │   ├── Button.stories.tsx
│       │   └── Button.module.css
│       ├── Toggle/
│       ├── Dropdown/
│       ├── Card/
│       ├── Badge/
│       ├── Terminal/
│       └── index.ts
├── .storybook/
│   ├── main.ts
│   ├── preview.ts
│   └── theme.ts              # Caro brand theme
└── package.json
```

### Component Inventory

#### Priority 1: Core Interactive Components

1. **Button**
   - Variants: `primary`, `secondary`, `ghost`, `support`
   - Sizes: `sm`, `md`, `lg`
   - States: `default`, `hover`, `active`, `disabled`
   - With/without icon
   - Full-width option

2. **Toggle**
   - Variants: `default`, `branded` (orange), `holiday` (seasonal)
   - Sizes: `sm` (mobile), `md` (desktop)
   - Label placement: `left`, `right`, `hidden`
   - Animated thumb transition

3. **Dropdown**
   - Trigger: button or custom element
   - Content: list items with icons
   - Position: `bottom-left`, `bottom-center`, `bottom-right`
   - Mobile: Full-width sheet option

#### Priority 2: Display Components

4. **Badge**
   - Variants: `primary`, `success`, `warning`, `info`
   - Sizes: `sm`, `md`
   - Pill/rounded shapes

5. **Card**
   - Variants: `default`, `elevated`, `bordered`, `translucent`
   - With/without header
   - Responsive padding

6. **Terminal**
   - Header with dots
   - Syntax highlighting
   - Copy functionality
   - Animated content option

#### Priority 3: Layout Components

7. **Section**
   - Text alignment control
   - Background variants
   - Responsive container

8. **Grid**
   - Auto-fit responsive columns
   - Gap controls

## Design Tokens

### Colors

```css
/* Brand */
--color-primary: #ff8c42;
--color-primary-dark: #ff6b35;
--color-support: #ea4aaa;
--color-success: #22c55e;

/* Backgrounds */
--color-bg: #ffffff;
--color-bg-dark: #1e1e1e;
--color-bg-secondary: #f5f5f5;

/* Text */
--color-text: #1a1a1a;
--color-text-secondary: #666666;

/* Borders */
--color-border: #e0e0e0;
```

### Spacing

```css
--space-xs: 4px;
--space-sm: 8px;
--space-md: 16px;
--space-lg: 24px;
--space-xl: 32px;
--space-2xl: 48px;
```

### Breakpoints

```css
--breakpoint-sm: 360px;   /* Small mobile */
--breakpoint-md: 600px;   /* Mobile */
--breakpoint-lg: 768px;   /* Tablet */
--breakpoint-xl: 900px;   /* Desktop */
--breakpoint-2xl: 1200px; /* Wide desktop */
```

### Touch Targets

- Minimum: 44px × 44px
- Recommended: 48px × 48px
- Large: 56px

## Storybook Configuration

### Setup Commands

```bash
cd website
npx storybook@latest init --type react
npm install @storybook/addon-a11y @storybook/addon-viewport
```

### Custom Theme (`.storybook/theme.ts`)

```typescript
import { create } from '@storybook/theming/create';

export default create({
  base: 'light',
  brandTitle: 'Caro Design System',
  brandUrl: 'https://caro.sh',
  brandImage: '/caro-logo.svg',

  colorPrimary: '#ff8c42',
  colorSecondary: '#ff6b35',

  // UI
  appBg: '#f8f8f8',
  appContentBg: '#ffffff',
  appBorderColor: '#e0e0e0',
  appBorderRadius: 8,

  // Typography
  fontBase: '"Inter", "Helvetica Neue", sans-serif',
  fontCode: '"Monaco", "Menlo", monospace',

  // Text colors
  textColor: '#1a1a1a',
  textInverseColor: '#ffffff',
});
```

### Viewport Addon Config

```typescript
const customViewports = {
  smallMobile: {
    name: 'Small Mobile (iPhone SE)',
    styles: { width: '320px', height: '568px' },
  },
  mobile: {
    name: 'Mobile',
    styles: { width: '375px', height: '667px' },
  },
  tablet: {
    name: 'Tablet',
    styles: { width: '768px', height: '1024px' },
  },
};
```

## Integration with Astro

### Using React Components in Astro

```astro
---
import { Button } from '../ui';
---

<Button client:load variant="primary" size="lg">
  Get Started
</Button>
```

### Astro Config

```javascript
// astro.config.mjs
import react from '@astrojs/react';

export default defineConfig({
  integrations: [react()],
});
```

## Development Workflow

1. **Design in Storybook**: Create/modify components in isolation
2. **Test responsiveness**: Use viewport addon for all breakpoints
3. **Accessibility**: Run a11y addon checks
4. **Export to Astro**: Use components with `client:` directives
5. **Document**: Keep stories as living documentation

## Migration Strategy

### Phase 1: Setup (1-2 days)
- [ ] Initialize Storybook
- [ ] Configure theme and viewports
- [ ] Create design tokens CSS file

### Phase 2: Core Components (3-5 days)
- [ ] Button component with all variants
- [ ] Toggle component with mobile sizing
- [ ] Badge component

### Phase 3: Complex Components (3-5 days)
- [ ] Dropdown with rich content
- [ ] Terminal window
- [ ] Card variants

### Phase 4: Integration (2-3 days)
- [ ] Replace Astro components with React imports
- [ ] Test across all pages
- [ ] Verify dark mode support

## UX Guidelines for Components

### Playfulness & Brand

- **Animations**: Subtle hover transitions (0.2s ease), playful bounce on buttons
- **Colors**: Orange gradient for primary actions, pink for support/sponsor
- **Dog motif**: Use 🐕 emoji sparingly for brand reinforcement
- **Terminal aesthetic**: Monospace fonts for code, macOS window chrome

### Accessibility

- All interactive elements: 44px min touch target
- Color contrast: WCAG AA minimum
- Focus states: Visible outline (orange ring)
- Motion: Respect `prefers-reduced-motion`

### Mobile-First

- Design for 320px first, enhance up
- Stack layouts vertically on mobile
- Full-width buttons on small screens
- Left-align paragraph text

## Resources

- [Storybook Docs](https://storybook.js.org/)
- [Astro + React](https://docs.astro.build/en/guides/integrations-guide/react/)
- [WCAG Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
