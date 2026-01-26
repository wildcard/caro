---
name: "astro-docs-dev"
description: "Develop and maintain the Caro documentation site using Astro Starlight. Use when writing technical documentation, configuring navigation, using MDX components, or creating custom documentation features"
version: "1.0.0"
allowed-tools: "Bash, Read, Write, Edit, Grep, Glob, Task"
license: "AGPL-3.0"
---

# Astro Documentation Site Development Skill

Expert guidance for developing the Caro documentation site at `/home/user/caro/docs-site`.

## What This Skill Does

- Write and structure technical documentation in MDX
- Configure Starlight sidebar navigation and settings
- Use Starlight components (Aside, Card, Tabs, Steps, etc.)
- Extend Starlight with custom components
- Maintain consistent documentation style and format
- Optimize documentation for discoverability and SEO

## When to Use This Skill

Activate when the user:
- Asks to write or update documentation
- Needs to add a new documentation page
- Wants to configure the sidebar navigation
- Needs help with MDX components
- Wants to customize Starlight appearance
- Asks about documentation structure

**Example Triggers:**
- "Add a new guide to the docs"
- "Update the installation documentation"
- "Add a warning callout to this page"
- "Create a tabbed section for different OS instructions"
- "Reorganize the sidebar navigation"
- "Add code examples to the reference docs"

## Technology Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| Astro | 5.16.6 | Static site framework |
| Starlight | 0.37.1 | Documentation framework |
| Sharp | 0.34.5 | Image optimization |
| Vercel Analytics | 1.4.1 | Performance tracking |

## Project Architecture

```
docs-site/
├── src/
│   ├── assets/              # Logo assets
│   │   ├── logo-dark.svg
│   │   └── logo-light.svg
│   ├── components/          # Custom components
│   │   ├── Head.astro       # Analytics integration
│   │   └── Hero.astro       # Custom hero (6.6KB)
│   ├── content/
│   │   └── docs/            # Documentation content
│   │       ├── index.mdx    # Homepage (splash)
│   │       ├── getting-started/
│   │       │   ├── introduction.mdx
│   │       │   ├── installation.mdx
│   │       │   └── quick-start.mdx
│   │       ├── product/
│   │       │   └── jobs-to-be-done.mdx
│   │       ├── guides/
│   │       │   ├── macos-setup.md
│   │       │   └── spec-kitty.md
│   │       ├── development/
│   │       │   ├── agents.md
│   │       │   ├── tdd-workflow.md
│   │       │   └── website.md
│   │       └── reference/
│   │           ├── backends.md
│   │           ├── configuration.md
│   │           ├── naming-history.md
│   │           └── safety.md
│   ├── styles/
│   │   └── custom.css       # Brand styling overrides
│   └── content.config.ts    # Content collection config
├── astro.config.mjs         # Starlight configuration
├── tsconfig.json
└── package.json
```

## Content Organization

### Section Structure

| Section | Purpose | Auto-generate |
|---------|---------|---------------|
| Getting Started | User onboarding | Manual order |
| Product | Product documentation | Manual order |
| Guides | How-to guides | Auto from directory |
| Development | Internal dev docs | Auto from directory |
| Reference | API/config reference | Auto from directory |

### Frontmatter Format

```yaml
---
title: Page Title
description: Brief description for SEO and search
sidebar:
  order: 1        # Optional: manual ordering
  badge:
    text: New     # Optional: sidebar badge
    variant: tip  # success, note, caution, danger
---
```

## Starlight Components

Import from `@astrojs/starlight/components`:

### Aside (Callouts)
```mdx
import { Aside } from '@astrojs/starlight/components';

<Aside type="tip">Helpful tip here</Aside>
<Aside type="note">Important note</Aside>
<Aside type="caution">Warning message</Aside>
<Aside type="danger">Critical warning</Aside>
```

### Cards and CardGrid
```mdx
import { Card, CardGrid } from '@astrojs/starlight/components';

<CardGrid>
  <Card title="Feature 1" icon="rocket">
    Feature description
  </Card>
  <Card title="Feature 2" icon="puzzle">
    Another feature
  </Card>
</CardGrid>
```

### Tabs
```mdx
import { Tabs, TabItem } from '@astrojs/starlight/components';

<Tabs>
  <TabItem label="macOS">macOS instructions</TabItem>
  <TabItem label="Linux">Linux instructions</TabItem>
  <TabItem label="Windows">Windows instructions</TabItem>
</Tabs>
```

### Steps
```mdx
import { Steps } from '@astrojs/starlight/components';

<Steps>
1. First step with detailed instructions
2. Second step
3. Third step
</Steps>
```

### Badge
```mdx
import { Badge } from '@astrojs/starlight/components';

<Badge text="New" variant="success" />
<Badge text="Experimental" variant="caution" />
```

### LinkCard
```mdx
import { LinkCard } from '@astrojs/starlight/components';

<LinkCard
  title="Related Guide"
  description="Learn more about this topic"
  href="/guides/related-guide"
/>
```

## Sidebar Configuration

In `astro.config.mjs`:

```javascript
sidebar: [
  {
    label: 'Getting Started',
    items: [
      { label: 'Introduction', slug: 'getting-started/introduction' },
      { label: 'Installation', slug: 'getting-started/installation' },
      { label: 'Quick Start', slug: 'getting-started/quick-start' },
    ],
  },
  {
    label: 'Product',
    items: [
      { label: 'Jobs To Be Done', slug: 'product/jobs-to-be-done' },
    ],
  },
  {
    label: 'Guides',
    autogenerate: { directory: 'guides' },  // Auto-discovers files
  },
  {
    label: 'Development',
    autogenerate: { directory: 'development' },
  },
  {
    label: 'Reference',
    autogenerate: { directory: 'reference' },
  },
]
```

### Adding New Pages

**For auto-generated sections** (Guides, Development, Reference):
1. Create `.md` or `.mdx` file in the appropriate directory
2. Add frontmatter with title and description
3. Page automatically appears in sidebar

**For manual sections** (Getting Started, Product):
1. Create the file in the appropriate directory
2. Add entry to `sidebar` in `astro.config.mjs`

## Custom Styling

Located at `/src/styles/custom.css`:

### Brand Colors
```css
:root {
  --sl-color-accent: #e85d4c;        /* Coral accent */
  --sl-color-accent-high: #ff8a7a;   /* Light coral */
  --sl-color-accent-low: #2a1515;    /* Dark coral tint */
  --sl-color-bg: #0d0d0d;            /* Background */
}

:root[data-theme='light'] {
  --sl-color-accent: #d4463a;
  --sl-color-bg: #fafafa;
}
```

### Typography
```css
--sl-font-heading: 'Space Grotesk', sans-serif;
--sl-font: 'Inter', sans-serif;
--sl-font-mono: 'JetBrains Mono', monospace;
```

## Custom Components

### Head.astro
Extends Starlight Head with Vercel Analytics:
```astro
---
import type { Props } from '@astrojs/starlight/props';
import Default from '@astrojs/starlight/components/Head.astro';
import { inject } from '@vercel/analytics';
---
<Default {...Astro.props}><slot /></Default>
<script>inject();</script>
```

### Hero.astro
Custom homepage hero with:
- Gradient animated title
- Terminal preview section
- CTA buttons (primary/secondary/minimal)
- Glassmorphism effects
- Dark/light theme support

## Sub-Agents Available

### Technical Writer
Primary agent for documentation content:
```
Task: technical-writer
Use when: Writing new docs, improving existing content, README files
```

### DX Product Manager
For documentation strategy and user flows:
```
Task: dx-product-manager
Use when: Planning documentation architecture, onboarding flows
```

### Explore Agent
For researching codebase before documenting:
```
Task: Explore
Use when: Understanding code to document, finding examples
```

## Related Commands

| Command | Purpose |
|---------|---------|
| `/caro.sync` | Sync docs with roadmap/website |
| `/caro.roadmap` | Check docs-related issues |

## Development Workflow

### 1. Start Development Server
```bash
cd docs-site && npm run dev
```

### 2. Build for Production
```bash
cd docs-site && npm run build
```

### 3. Preview Built Site
```bash
cd docs-site && npm run preview
```

## Writing Guidelines

### Content Structure
1. **Title**: Clear, action-oriented (e.g., "Installing Caro" not "Installation")
2. **Introduction**: 1-2 sentences explaining what the page covers
3. **Prerequisites**: List any requirements (if applicable)
4. **Main Content**: Organized with H2/H3 headings
5. **Code Examples**: Practical, copy-pasteable examples
6. **Next Steps**: Link to related documentation

### Style Conventions
- Use **bold** for UI elements and important terms
- Use `code` for commands, file paths, and variable names
- Use tables for structured reference data
- Use Aside components for tips, warnings, notes
- Use Tabs for platform-specific instructions

### Code Blocks
```markdown
\`\`\`bash title="Install with Cargo"
cargo install caro
\`\`\`

\`\`\`rust title="src/main.rs" {2-4}
fn main() {
    // Highlighted lines
    println!("Hello!");
}
\`\`\`
```

## Best Practices

### Documentation
1. Keep pages focused on a single topic
2. Use descriptive titles and descriptions
3. Include practical code examples
4. Cross-link related documentation
5. Update docs when features change

### MDX
1. Import components at the top of the file
2. Use Starlight components over raw HTML
3. Keep MDX expressions simple
4. Test rendered output in dev server

### Navigation
1. Use manual ordering for critical paths (Getting Started)
2. Use auto-generate for reference sections
3. Keep sidebar depth reasonable (max 2 levels)
4. Add badges for new/experimental content

### SEO
1. Write unique descriptions for each page
2. Use descriptive, keyword-rich titles
3. Structure content with proper headings
4. Include internal links

## Troubleshooting

### Content Not Appearing
- Check frontmatter syntax (YAML)
- Verify file extension is `.md` or `.mdx`
- For manual sections, check `astro.config.mjs` sidebar
- Run `npm run build` to catch errors

### MDX Component Errors
- Ensure correct import path
- Check component props match documentation
- Verify JSX syntax in MDX

### Styling Issues
- Check `custom.css` for conflicts
- Use browser dev tools to inspect CSS variables
- Verify dark/light theme selectors

## File Quick Reference

| Purpose | Location |
|---------|----------|
| Starlight config | `astro.config.mjs` |
| Custom styles | `src/styles/custom.css` |
| Homepage | `src/content/docs/index.mdx` |
| Content collection | `src/content.config.ts` |
| Custom components | `src/components/` |
| Logo assets | `src/assets/` |
