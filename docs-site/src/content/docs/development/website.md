---
title: Website Development
description: Development guide for the caro.sh website
---

This document covers development of the caro.sh marketing website built with Astro.

## Project Location

The website source is in the `website/` directory of the monorepo.

## Tech Stack

- **Framework**: [Astro](https://astro.build) v4
- **Styling**: Scoped CSS in Astro components
- **Deployment**: Vercel (static site generation)
- **Analytics**: Vercel Analytics

## Analytics

The website uses Vercel Analytics for privacy-friendly traffic monitoring.

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

## Local Development

```bash
# Navigate to website directory
cd website

# Install dependencies
npm install

# Start dev server
npm run dev
```

Visit `http://localhost:4321` to see the site.

## Building

```bash
# Production build
npm run build

# Preview production build
npm run preview
```

## Deployment

The website auto-deploys to Vercel on push to main. Preview deployments are created for all pull requests.

### Custom Domain

The production site is available at `caro.sh`.
