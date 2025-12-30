# Master Prompt: Caro Design System & Storybook Implementation

## Context

You are continuing work on the Caro website (caro.sh), a marketing site for an AI-powered POSIX shell command generator CLI tool. The previous session completed:

1. **Navigation redesign**: Modern desktop dropdowns + mobile slide-out drawer with theme controls
2. **Footer redesign**: Multi-column layout organizing growing site content
3. **Hero/Download UX fixes**: Proper responsive breakpoints (768px, 600px, 360px)
4. **UX audit**: Identified component patterns and mobile issues

## Your Mission

Implement a **Storybook component library** that extracts reusable React components from the existing Astro site, creating a consistent design system.

## Key Files to Review

```
website/COMPONENT_LIBRARY_PLAN.md  # Full architecture plan
website/src/components/            # Existing Astro components
website/src/layouts/Layout.astro   # CSS variables and theme setup
```

## Priority Tasks

### Phase 1: Storybook Setup

1. Initialize Storybook with React support:
   ```bash
   cd website
   npx storybook@latest init --type react
   npm install @storybook/addon-a11y @storybook/addon-viewport
   ```

2. Configure custom theme (see COMPONENT_LIBRARY_PLAN.md)

3. Set up viewports for mobile testing (320px, 375px, 768px)

### Phase 2: Core Components

Create these React components with stories:

#### Button (`src/ui/Button/`)
- Variants: `primary` (orange gradient), `secondary` (outline), `ghost`, `support` (pink)
- Sizes: `sm`, `md`, `lg`
- Full-width option for mobile
- Minimum touch target: 44px

#### Toggle (`src/ui/Toggle/`)
- Desktop: 50px × 28px track
- Mobile: 44px × 24px track
- Branded variant (orange when active)
- Animation: 0.2s thumb slide

#### Badge (`src/ui/Badge/`)
- Variants: `primary`, `success` (green), `warning`, `info`
- Pill shape with 12px border-radius
- Small text (11-12px)

### Phase 3: Integration

1. Install `@astrojs/react` integration
2. Import React components into Astro with `client:load`
3. Replace existing inline styles with component usage

## Design Tokens (from Layout.astro)

```css
/* Primary colors */
--color-primary: #ff8c42
--color-primary-dark: #ff6b35
--color-support: #ea4aaa

/* Backgrounds */
--color-bg: #ffffff (light) / #1a1a1a (dark)
--color-bg-secondary: #f5f5f5 / #2d2d2d

/* Touch targets */
Minimum: 44px × 44px
```

## Brand Guidelines

- **Personality**: Friendly, helpful, safety-focused (like a loyal dog)
- **Visual style**: Modern, clean, playful but professional
- **Animations**: Subtle (0.2s ease), not distracting
- **Colors**: Orange gradient for primary, pink for support
- **Typography**: Inter/system for UI, Monaco/Menlo for code

## UX Requirements

1. **Mobile-first**: All components must work at 320px
2. **Accessibility**: WCAG AA contrast, visible focus states
3. **Touch-friendly**: 44px minimum interactive area
4. **Dark mode**: All components support dark theme
5. **Reduced motion**: Respect `prefers-reduced-motion`

## Component Quality Checklist

For each component, ensure:

- [ ] Stories for all variants
- [ ] Mobile viewport story
- [ ] Dark mode story
- [ ] Accessibility (a11y) check passes
- [ ] TypeScript props interface
- [ ] CSS module (no inline styles)
- [ ] Documentation in story

## File Structure

```
website/
├── src/
│   ├── ui/
│   │   ├── Button/
│   │   │   ├── Button.tsx
│   │   │   ├── Button.module.css
│   │   │   ├── Button.stories.tsx
│   │   │   └── index.ts
│   │   ├── Toggle/
│   │   ├── Badge/
│   │   ├── tokens.css          # Design tokens
│   │   └── index.ts            # Barrel export
│   └── components/             # Keep Astro components
├── .storybook/
│   ├── main.ts
│   ├── preview.ts
│   └── theme.ts
└── package.json
```

## Success Criteria

1. Storybook running at `npm run storybook`
2. Button, Toggle, Badge components complete with all variants
3. Stories pass a11y checks
4. Components work in Astro with `client:load`
5. Dark mode supported in Storybook preview

## Notes from Previous Session

- The mobile drawer in Navigation.astro has good toggle patterns to reference
- Download.astro now has responsive CSS - use as reference for breakpoints
- Footer.astro shows proper multi-column responsive grid
- Holiday theme system exists (Christmas/Hanukkah) - components should support dynamic theming

## Getting Started

1. Read `website/COMPONENT_LIBRARY_PLAN.md` for full context
2. Review existing components in `website/src/components/`
3. Set up Storybook
4. Start with Button component (most reused)
5. Test on mobile viewports constantly

Good luck! The goal is a polished, playful design system that makes Caro feel approachable and trustworthy.
