---
name: "astro-website-dev"
description: "Develop and maintain the Caro marketing website (caro.sh). Use when building Astro components, working with CSS Modules, implementing holiday themes, managing the search system, or creating new pages and layouts"
version: "1.0.0"
allowed-tools: "Bash, Read, Write, Edit, Grep, Glob, Task"
license: "AGPL-3.0"
---

# Astro Website Development Skill

Expert guidance for developing the Caro marketing website at `/home/user/caro/website`.

## What This Skill Does

- Build and maintain Astro components (`.astro` and React `.tsx`)
- Work with the CSS Modules design system and design tokens
- Implement and extend the holiday theme system
- Manage the OmniSearch system and search index
- Create new pages following project conventions
- Develop responsive layouts with accessibility in mind
- Use Storybook for component development and testing

## When to Use This Skill

Activate when the user:
- Asks to create or modify website components
- Needs to add a new page to caro.sh
- Wants to update the holiday theme system
- Needs help with CSS styling or design tokens
- Wants to work on the search functionality
- Asks about Storybook component development
- Needs to update layouts or navigation

**Example Triggers:**
- "Add a new page to the website"
- "Create a component for the landing page"
- "Update the holiday theme colors"
- "Fix styling on the hero section"
- "Add a new blog post"
- "Make the navigation responsive"

## Technology Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| Astro | 5.16.6 | Static site framework |
| React | 19.2.3 | Interactive components |
| CSS Modules | - | Scoped component styling |
| Storybook | 8.6.15 | Component development |
| Vite | 6.4.1 | Build tooling |
| Vercel Analytics | 1.6.1 | Performance tracking |

## Project Architecture

```
website/
├── src/
│   ├── components/          # Astro components (25+ major)
│   │   ├── Navigation.astro # Main nav (41KB - complex)
│   │   ├── OmniSearch.astro # Global search (23KB)
│   │   ├── Hero.astro       # Hero sections
│   │   ├── HolidayThemeToggle.astro
│   │   └── explore/         # Explore page components
│   ├── layouts/             # Page layouts (4)
│   │   ├── Layout.astro     # Main layout (27KB)
│   │   ├── BlogPost.astro
│   │   └── ComparisonPageLayout.astro
│   ├── pages/               # Route pages (32+)
│   │   ├── index.astro
│   │   ├── blog/
│   │   ├── use-cases/
│   │   └── compare/
│   ├── ui/                  # React UI components
│   │   ├── Button/
│   │   ├── Card/
│   │   ├── Terminal/
│   │   └── tokens.css       # Design tokens
│   ├── config/              # Configuration
│   │   ├── holidays.ts      # Holiday definitions (18KB)
│   │   ├── pages.ts         # Page routing
│   │   └── types.ts         # TypeScript interfaces
│   ├── lib/                 # Utilities
│   │   ├── holiday-engine.ts
│   │   ├── locale-detector.ts
│   │   └── calendar/        # Multi-calendar support
│   └── data/                # Static data
├── scripts/                 # Build scripts
│   ├── extract-version.mjs
│   └── generate-search-index.mjs
├── .storybook/              # Storybook config
└── astro.config.mjs
```

## Core Patterns

### Component Organization

**Astro Components** (`.astro`):
```astro
---
// Frontmatter - server-side code
interface Props {
  title: string;
  variant?: 'primary' | 'secondary';
}
const { title, variant = 'primary' } = Astro.props;
---

<div class={`component ${variant}`}>
  <h2>{title}</h2>
  <slot />
</div>

<style>
  .component { /* scoped styles */ }
</style>
```

**React UI Components** (`/src/ui/`):
```
ComponentName/
├── ComponentName.tsx          # React component
├── ComponentName.module.css   # CSS Module styles
├── ComponentName.stories.tsx  # Storybook stories
└── index.ts                   # Barrel export
```

### Design Token System

Located at `/src/ui/tokens.css`:

```css
:root {
  /* Brand Colors */
  --color-primary: #ff8c42;
  --color-primary-dark: #ff6b35;
  --color-support: #ea4aaa;

  /* Spacing Scale */
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;

  /* Typography */
  --font-family-sans: system-ui, -apple-system, sans-serif;
  --font-family-mono: Monaco, Menlo, monospace;
}

/* Dark mode overrides */
@media (prefers-color-scheme: dark) {
  :root { /* dark tokens */ }
}
```

### Page Creation Pattern

Each page exports `searchMeta` for the search index:

```astro
---
import Layout from '../layouts/Layout.astro';

export const searchMeta = {
  title: 'Page Title',
  description: 'Page description for search',
  keywords: ['keyword1', 'keyword2'],
  icon: 'icon-name',
  category: 'main' | 'use-cases' | 'compare' | 'blog' | 'docs'
};
---

<Layout title={searchMeta.title} description={searchMeta.description}>
  <!-- Page content -->
</Layout>
```

### Holiday Theme System

The holiday engine (`/src/lib/holiday-engine.ts`) supports:
- Fixed and calculated dates
- Multiple calendar systems (Hebrew, Lunar, Hindu, Islamic)
- Locale-aware defaults
- Timezone/hemisphere awareness
- Visual themes with colors, icons, effects

**To add/modify holidays**, edit `/src/config/holidays.ts`:
```typescript
{
  id: 'holiday-id',
  name: 'Holiday Name',
  names: { en: 'English', es: 'Spanish' },
  dateType: 'fixed' | 'calculated',
  month: 12,
  day: 25,
  leadTimeDays: 7,
  theme: {
    colors: { primary: '#...', secondary: '#...' },
    icon: 'icon-name',
    effects: ['snow', 'confetti', 'petals']
  }
}
```

## Sub-Agents Available

### Cultural Heritage Expert
For holiday theme cultural accuracy and sensitivity:
```
Task: cultural-heritage-expert
Use when: Adding new cultural holidays, reviewing sensitivity
```

### Technical Writer
For content and copy on website pages:
```
Task: technical-writer
Use when: Writing page content, blog posts, documentation
```

### DX Product Manager
For user experience and landing page design:
```
Task: dx-product-manager
Use when: Designing new user flows, landing pages, CTAs
```

## Related Commands

| Command | Purpose |
|---------|---------|
| `/caro.sync` | Sync code across website/docs/roadmap |
| `/caro.roadmap` | Check website-related issues |

## Development Workflow

### 1. Start Development Server
```bash
cd website && npm run dev
```

### 2. Component Development with Storybook
```bash
cd website && npm run storybook
```

### 3. Regenerate Search Index
```bash
cd website && npm run prebuild
```

### 4. Build for Production
```bash
cd website && npm run build
```

## Best Practices

### Component Development
1. Use Astro components for static content
2. Use React components for interactive UI
3. Always co-locate CSS Module with component
4. Add Storybook stories for all UI components
5. Follow accessibility guidelines (ARIA labels, focus states)

### Styling
1. Use design tokens from `tokens.css`
2. Mobile-first responsive design
3. Support both light and dark modes
4. Maintain 44px minimum touch targets
5. Use CSS variables for theming

### Performance
1. Prefer static rendering where possible
2. Use `inlineStylesheets: 'auto'` for CSS optimization
3. Lazy load heavy components
4. Optimize images with appropriate formats

### Search System
1. Export `searchMeta` from all pages
2. Run `npm run prebuild` after adding pages
3. Include relevant keywords for discoverability

## Troubleshooting

### Build Errors
- Check TypeScript types in `config/types.ts`
- Verify all component imports are correct
- Run `npm install` if dependencies missing

### Holiday Theme Issues
- Use `HolidayDebugPanel.astro` component for testing
- Check locale detection in browser console
- Verify date calculations for calculated holidays

### Storybook Issues
- Ensure CSS Modules are properly imported
- Check that stories export correct metadata
- Verify Vite config compatibility

## File Quick Reference

| Purpose | Location |
|---------|----------|
| Main layout | `src/layouts/Layout.astro` |
| Navigation | `src/components/Navigation.astro` |
| Design tokens | `src/ui/tokens.css` |
| Holiday config | `src/config/holidays.ts` |
| Search index | `src/config/search-index.json` |
| Site config | `astro.config.mjs` |
