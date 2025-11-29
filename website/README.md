# cmdai Community Hub - Next.js Website

A modern, SEO-optimized website for discovering cmdai's safety guardrails and community guides.

## Features

- 🔍 **Search & Filter** - Find guardrails and guides quickly
- 📊 **Statistics** - View usage metrics and quality scores
- 🎯 **Category Browse** - Organized by topic
- 📱 **Responsive Design** - Works on all devices
- 🌙 **Dark Mode** - Auto-adapts to system preferences
- ⚡ **Static Export** - Fast loading, deploy anywhere
- 🔗 **SEO Optimized** - Rich meta tags, sitemaps, schema.org

## Quick Start

### Install Dependencies

```bash
cd website
npm install
```

### Development Server

```bash
npm run dev
```

Visit http://localhost:3000

### Build for Production

```bash
npm run build
npm start
```

### Static Export

```bash
npm run build
# Output in /out directory
```

## Project Structure

```
website/
├── app/                    # Next.js App Router pages
│   ├── layout.tsx         # Root layout
│   ├── page.tsx          # Homepage
│   ├── guardrails/       # Guardrails pages
│   └── guides/           # Guides pages
├── components/            # Reusable React components
│   ├── GuardrailCard.tsx
│   ├── GuideCard.tsx
│   ├── SearchBar.tsx
│   └── FilterPanel.tsx
├── lib/                   # Utility functions
│   ├── guardrails.ts     # Guardrails loader
│   ├── guides.ts         # Guides loader
│   └── search.ts         # Search utilities
├── types/                 # TypeScript type definitions
│   └── index.ts
└── public/               # Static assets
```

## Data Sources

The website loads content from:
- `../docs/guardrails/*.yml` - Guardrail YAML files
- `../docs/guides/**/*.md` - Guide Markdown files with frontmatter

## Pages

### Homepage (`/`)
- Overview of cmdai features
- Statistics dashboard
- Popular guides
- Recent guardrails

### Guardrails Browser (`/guardrails`)
- List all guardrails
- Filter by category, risk level
- Search by pattern, description, tags
- View statistics

### Guardrail Detail (`/guardrails/[id]`)
- Full guardrail information
- Examples (blocked/safe)
- Community notes
- Related guides

### Guides Browser (`/guides`)
- List all guides
- Filter by category, difficulty, risk
- Search by title, command, tags
- Sort by quality, popularity

### Guide Detail (`/guides/[category]/[id]`)
- Complete guide content
- "Try in cmdai" button
- Prerequisites and outcomes
- Related guides
- Community metrics

## Customization

### Styling

Edit `tailwind.config.js` and `app/globals.css` for theme customization.

### Content

Add new guardrails:
1. Create YAML file in `../docs/guardrails/`
2. Website auto-loads on next build

Add new guides:
1. Create Markdown file in `../docs/guides/[category]/`
2. Include frontmatter metadata
3. Website auto-loads on next build

## SEO Features

- Dynamic meta tags per page
- Open Graph tags for social sharing
- JSON-LD structured data
- Sitemap generation
- Canonical URLs
- Semantic HTML

## Performance

- Static site generation (SSG)
- Image optimization
- Code splitting
- Lazy loading
- Tailwind CSS purging

## Deployment

### Vercel (Recommended)

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel
```

### Netlify

```bash
# Build
npm run build

# Deploy /out directory
netlify deploy --prod --dir=out
```

### GitHub Pages

```bash
# Build
npm run build

# Push /out to gh-pages branch
```

### Self-hosted

```bash
# Build static files
npm run build

# Serve /out directory with any static file server
npx serve out
```

## Environment Variables

Create `.env.local`:

```env
SITE_URL=https://cmdai.dev
SITE_NAME=cmdai Community Hub
SITE_DESCRIPTION=Your description here
```

## Development

### Type Checking

```bash
npm run type-check
```

### Linting

```bash
npm run lint
```

### Adding New Features

1. Create components in `/components`
2. Add utilities in `/lib`
3. Define types in `/types`
4. Update pages in `/app`

## Tech Stack

- **Framework**: Next.js 15 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **Icons**: Lucide React
- **Parsing**: gray-matter, js-yaml, markdown-it
- **Search**: Fuse.js
- **Deployment**: Static export

## Browser Support

- Chrome/Edge (last 2 versions)
- Firefox (last 2 versions)
- Safari (last 2 versions)
- Mobile browsers (iOS Safari, Chrome Android)

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

Same as cmdai project license.

---

**Built with ❤️ by the cmdai community**
