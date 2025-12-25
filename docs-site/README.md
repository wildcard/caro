# Caro Documentation Site

A beautiful documentation site for [caro](https://github.com/wildcard/caro) built with [Astro](https://astro.build) and [Starlight](https://starlight.astro.build/).

## Quick Start

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Project Structure

```
docs-site/
├── src/
│   ├── assets/          # Logo SVGs
│   ├── content/
│   │   └── docs/        # Documentation markdown files
│   │       ├── getting-started/
│   │       ├── guides/
│   │       ├── development/
│   │       └── reference/
│   └── styles/
│       └── custom.css   # Custom styling
├── astro.config.mjs     # Astro configuration
└── package.json
```

## Adding Documentation

Add new documentation files in `src/content/docs/` as `.md` or `.mdx` files:

```md
---
title: My Page Title
description: A brief description
---

# Content here
```

## Design System

The site uses a custom design with:

- **Colors**: Warm coral/salmon palette (reflecting the "caro" name meaning "dear/beloved")
- **Typography**: Space Grotesk for headings, Inter for body, JetBrains Mono for code
- **Theme**: Dark-first with light mode support
- **Components**: Custom feature cards, hero section, and code blocks

## Deployment

The site builds to static files in `dist/`:

```bash
npm run build
```

Deploy the `dist/` folder to any static hosting (Vercel, Netlify, GitHub Pages, etc.).

## License

Same as the main caro project.
