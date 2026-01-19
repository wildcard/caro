# Caro Website

Official website for **Caro** - Your loyal shell companion. Built with [Astro](https://astro.build). Visit at **caro.sh**

## Overview

Caro is a companion agent that specializes in POSIX shell commands. She's available as an MCP for Claude and as a dedicated Skill, helping keep you safe while Claude gets the work done.

## The Story

Caro is the digitalization of Kyaro (Kyarorain Kadosh), the maintainer's beloved dog. Inspired by Portal's Caroline/GLaDOS—loyalty transformed into eternal companionship.

## Tech Stack

- **Framework**: [Astro](https://astro.build) v4
- **Styling**: Scoped CSS in Astro components
- **Deployment**: Static site generation (SSG)
- **Integrations**: Sitemap generation
- **Analytics**: [Vercel Analytics](https://vercel.com/docs/analytics) for privacy-friendly traffic insights

## Analytics

The website uses Vercel Analytics for privacy-friendly traffic monitoring. Analytics is integrated via the official `@vercel/analytics` package.

### Setup

In `src/layouts/Layout.astro`:

```astro
---
import Analytics from '@vercel/analytics/astro';
---

<body>
  <slot />
  <Analytics />
</body>
```

### Features

- **Privacy-first**: No cookies, GDPR compliant by default
- **Automatic tracking**: Page views tracked without additional configuration
- **Web Vitals**: Core Web Vitals automatically collected
- **Real-time**: View live traffic in Vercel dashboard

### Viewing Analytics

1. Go to your [Vercel dashboard](https://vercel.com/dashboard)
2. Select the caro website project
3. Click the "Analytics" tab

Analytics are automatically enabled when deployed to Vercel. No additional configuration is required.

## Project Structure

```
website/
├── src/
│   ├── components/       # Reusable Astro components
│   │   ├── Hero.astro
│   │   ├── Terminal.astro
│   │   ├── Story.astro
│   │   ├── Video.astro
│   │   ├── Features.astro
│   │   ├── Download.astro
│   │   └── Footer.astro
│   ├── layouts/
│   │   └── Layout.astro  # Base layout with global styles
│   └── pages/
│       └── index.astro   # Homepage
├── public/               # Static assets (favicon, images, etc.)
│   └── favicon.svg
├── astro.config.mjs      # Astro configuration
├── package.json
├── tsconfig.json
└── .gitignore
```

## Getting Started

### Prerequisites

- Node.js 18+ or 20+
- npm, pnpm, or yarn

### Installation

```bash
# Install dependencies
npm install
# or
pnpm install
# or
yarn install
```

### Development

Start the development server:

```bash
npm run dev
# or
pnpm dev
# or
yarn dev
```

Visit `http://localhost:4321` to see your site.

### Building for Production

Create a production build:

```bash
npm run build
# or
pnpm build
# or
yarn build
```

The static site will be built to `dist/`.

### Preview Production Build

Preview the production build locally:

```bash
npm run preview
# or
pnpm preview
# or
yarn preview
```

## Customization

### Adding Your Demo Video

Edit `src/components/Video.astro` and replace the placeholder:

```astro
<iframe
  width="100%"
  height="100%"
  src="https://www.youtube.com/embed/YOUR_VIDEO_ID"
  style="border: 0;"
  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
  allowfullscreen>
</iframe>
```

### Updating Features

Edit `src/components/Features.astro` and modify the `features` array:

```astro
const features = [
  {
    icon: "🤝",
    title: "Your Feature",
    description: "Feature description..."
  },
  // Add more features
];
```

### Changing Colors

Colors are defined in component styles. Main brand colors:
- Primary gradient: `#ff8c42` to `#ff6b35`
- Background warmth: `#fff8f0`
- Text: `#2c3e50` (dark blue-gray)
- Accent: `#7f8c8d` (gray)

### Modifying Content

Each section is a separate component in `src/components/`:
- **Hero.astro**: Main tagline and CTAs
- **Terminal.astro**: Example command demo
- **Story.astro**: The Kyaro/Caroline/GLaDOS narrative
- **Features.astro**: Six capability cards
- **Download.astro**: Installation and usage modes
- **Footer.astro**: Links and memorial

## Deployment

### GitHub Pages

1. Add to `.github/workflows/deploy.yml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [ main ]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
      - name: Install dependencies
        run: npm install
        working-directory: ./website
      - name: Build
        run: npm run build
        working-directory: ./website
      - name: Upload artifact
        uses: actions/upload-pages-artifact@v2
        with:
          path: ./website/dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v3
```

2. Enable GitHub Pages in repository settings
3. Configure custom domain to `caro.sh`

### Netlify

1. Connect your repository to Netlify
2. Configure build settings:
   - **Base directory**: `website`
   - **Build command**: `npm run build`
   - **Publish directory**: `website/dist`
3. Add custom domain: `caro.sh`

### Vercel

Both the website and Storybook are deployed on Vercel as separate projects, configured manually via the Vercel dashboard.

> **Note:** The `*.vercel.json` files in this directory are **not used by Vercel** (only `vercel.json` would be). They exist as documentation of the dashboard configuration for each project.

#### Configuration Files (Reference Only)

| File | Purpose | Live URL |
|------|---------|----------|
| `website.vercel.json` | Main website build config | https://caro.sh |
| `storybook.vercel.json` | Storybook build config | https://storybook.caro.sh |

#### Deploying the Website

1. Go to [vercel.com/new](https://vercel.com/new)
2. Import the `wildcard/caro` repository
3. Set **Root Directory** to `website`
4. Configure build settings (or let Vercel auto-detect Astro):
   - **Framework**: Astro
   - **Build Command**: `npm run build`
   - **Output Directory**: `dist`
5. Deploy

#### Deploying Storybook

1. Create a **new** project at [vercel.com/new](https://vercel.com/new)
2. Import the same `wildcard/caro` repository
3. Set **Root Directory** to `website`
4. Configure build settings:
   - **Framework**: Storybook (auto-detected)
   - **Build Command**: `npm run build-storybook`
   - **Output Directory**: `storybook-static`
5. Deploy

#### Custom Domains

Configure in Vercel dashboard under Project → Settings → Domains:

| Project | Domain |
|---------|--------|
| Website | `caro.sh` |
| Storybook | `storybook.caro.sh` |

DNS configuration at your registrar:
```
A     @     76.76.21.21
CNAME www   cname.vercel-dns.com
```

#### Preview Deployments

Every push creates preview deployments for both projects:
```
https://caro-git-<branch>-<team>.vercel.app
https://storybook-caro-git-<branch>-<team>.vercel.app
```

#### Environment Variables

No environment variables are required. If needed later:

1. Go to Project Settings → Environment Variables
2. Add variables for Production, Preview, and Development
3. Redeploy for changes to take effect

### Cloudflare Pages

1. Connect repository to Cloudflare Pages
2. Build settings:
   - **Build command**: `npm run build`
   - **Build output directory**: `dist`
   - **Root directory**: `website`
3. Configure custom domain: `caro.sh`

### Custom Domain (caro.sh)

Configure your DNS:

```
# DNS Records for caro.sh
A     @     <your-hosting-provider-ip>
CNAME www   <your-hosting-provider>
```

## Why Astro?

- **Performance**: Ships zero JavaScript by default
- **SEO-friendly**: Perfect for static content sites
- **Component-based**: Easy to maintain and update
- **Modern DX**: Great developer experience
- **Flexible**: Can add React, Vue, or other frameworks if needed
- **Fast builds**: Optimized for quick deployment

## Key Messaging

- **Tagline**: "Your loyal shell companion"
- **Mission**: Specialized POSIX shell command agent with empathy and agency
- **Safety**: Comprehensive validation like a loyal companion
- **Platform**: Works across macOS, Linux, Windows, GNU, BSD
- **Integration**: MCP for Claude, dedicated Skill, standalone CLI
- **Story**: Kyaro → Caro, inspired by Portal's Caroline → GLaDOS

## Browser Support

- Chrome/Edge: ✅ Full support
- Firefox: ✅ Full support
- Safari: ✅ Full support
- Mobile browsers: ✅ Responsive design

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](../CONTRIBUTING.md) first.

## License

MIT License - Same as the Caro project

## Legacy Version

The original single-file HTML version is preserved as `index.html` in this directory. The Astro version provides:
- Better performance and SEO
- Component reusability
- Easier maintenance
- Modern development workflow

## Workspace Configuration

This website is part of an npm workspaces monorepo. Changes to this workspace will trigger Vercel builds only for the website project, while other workspaces (docs-site, landing) will be automatically skipped.
