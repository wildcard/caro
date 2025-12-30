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

The website is configured for Vercel deployment via `vercel.json`.

#### Quick Deploy

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https://github.com/wildcard/caro&root-directory=website)

#### Manual Setup

1. Install Vercel CLI:
   ```bash
   npm i -g vercel
   ```

2. Deploy from the website directory:
   ```bash
   cd website
   vercel
   ```

3. For production deployment:
   ```bash
   vercel --prod
   ```

#### Configuration

The `vercel.json` file is pre-configured:
```json
{
  "buildCommand": "npm install && npm run build",
  "outputDirectory": "dist",
  "devCommand": "npm run dev",
  "installCommand": "npm install"
}
```

#### Via Vercel Dashboard

1. Import your repository at [vercel.com/new](https://vercel.com/new)
2. Select the `website` folder as Root Directory
3. Vercel auto-detects settings from `vercel.json`
4. Click Deploy

#### Custom Domain

1. Go to your Vercel project → Settings → Domains
2. Add `caro.sh` as custom domain
3. Configure DNS at your registrar:
   ```
   A     @     76.76.21.21
   CNAME www   cname.vercel-dns.com
   ```

#### Preview Deployments

Every push to a branch creates a preview deployment automatically. Preview URLs follow the pattern:
```
https://caro-git-<branch>-<team>.vercel.app
```

#### Deploying Storybook

Storybook is deployed as a separate Vercel project:

1. Create a new project at [vercel.com/new](https://vercel.com/new)
2. Import this repository
3. Set **Root Directory** to `website`
4. Vercel auto-detects Storybook and configures the build

Live at: https://storybook.caro.sh

#### Environment Variables

No environment variables are required for the static site. If you add features that need them:

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
