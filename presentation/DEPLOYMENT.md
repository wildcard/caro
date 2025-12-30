# Slidev Presentation Deployment Guide

This guide explains how to deploy the Caro Slidev presentations to Vercel.

## Overview

The project contains two Slidev presentations:
- **Main Presentation** (`slides.md`) - Technical overview of Caro
- **Pitch Presentation** (`pitch-slides.md`) - Investor/partner pitch deck
- **Roadmap Presentation** (`roadmap-slides.md`) - 2026 development roadmap

All presentations are built and deployed together on Vercel.

## Deployment URLs

When deployed to Vercel:
- Main presentation: `https://your-domain.vercel.app/`
- Pitch presentation: `https://your-domain.vercel.app/pitch/`
- Roadmap presentation: `https://your-domain.vercel.app/roadmap/`

---

## Vercel Dashboard Configuration

> **IMPORTANT**: Vercel ignores `vercel.json` files when configured via the dashboard.
> The `vercel.reference.json` file in this directory documents the configuration
> that must be set manually in the Vercel dashboard.

### Step 1: Create New Project

1. Go to [vercel.com/new](https://vercel.com/new)
2. Import the `wildcard/caro` repository
3. Configure as follows:

### Step 2: Build & Development Settings

In **Project Settings → General → Build & Development Settings**:

| Setting | Value |
|---------|-------|
| **Framework Preset** | Other |
| **Root Directory** | `presentation` |
| **Build Command** | `npm run build:all` |
| **Output Directory** | `dist` |
| **Install Command** | `npm install` |
| **Development Command** | `npm run dev` |

### Step 3: Rewrite Rules

In **Project Settings → Functions → Rewrites**, add these rules in order:

| Priority | Source | Destination | Purpose |
|----------|--------|-------------|---------|
| 1 | `/pitch/(.*)` | `/pitch/index.html` | SPA routing for pitch deck |
| 2 | `/roadmap/(.*)` | `/roadmap/index.html` | SPA routing for roadmap |
| 3 | `/(.*)` | `/index.html` | Fallback for main presentation |

**Why Rewrites are Needed:**
- Slidev presentations are Single Page Applications (SPAs)
- Direct navigation to slides (e.g., `/3` or `/roadmap/5`) needs to serve the main HTML
- Without rewrites, Vercel returns 404 for SPA routes

### Step 4: Response Headers (Security)

In **Project Settings → Functions → Headers**, add:

| Source | Header | Value |
|--------|--------|-------|
| `/*` | `X-Content-Type-Options` | `nosniff` |
| `/*` | `X-Frame-Options` | `DENY` |
| `/*` | `X-XSS-Protection` | `1; mode=block` |

These headers:
- Prevent MIME type sniffing attacks
- Block embedding in iframes (clickjacking protection)
- Enable XSS filtering in older browsers

---

## Reference Configuration File

The file `vercel.reference.json` contains the JSON representation of the above settings:

```json
{
  "buildCommand": "npm run build:all",
  "outputDirectory": "dist",
  "installCommand": "npm install",
  "framework": null,
  "rewrites": [
    { "source": "/pitch/(.*)", "destination": "/pitch/index.html" },
    { "source": "/roadmap/(.*)", "destination": "/roadmap/index.html" },
    { "source": "/(.*)", "destination": "/index.html" }
  ],
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "X-Content-Type-Options", "value": "nosniff" },
        { "key": "X-Frame-Options", "value": "DENY" },
        { "key": "X-XSS-Protection", "value": "1; mode=block" }
      ]
    }
  ]
}
```

> **Note**: This file is for documentation only. Vercel does NOT read this file
> when the project is configured via the dashboard. All settings must be
> configured manually in the Vercel dashboard as described above.

---

## Build Process

### Local Build

Build all presentations locally:

```bash
cd presentation
npm install
npm run build:all
```

This will:
1. Build the main presentation to `dist/`
2. Build the pitch presentation to `dist/pitch/`
3. Build the roadmap presentation to `dist/roadmap/`

### Output Structure

```
dist/
├── index.html          # Main presentation
├── assets/             # Main presentation assets
├── pitch/
│   ├── index.html     # Pitch presentation
│   └── assets/        # Pitch presentation assets
├── roadmap/
│   ├── index.html     # Roadmap presentation
│   └── assets/        # Roadmap presentation assets
└── ...
```

---

## Manual Deployment via CLI

### First-Time Setup

1. **Install Vercel CLI**:
   ```bash
   npm install -g vercel
   ```

2. **Login to Vercel**:
   ```bash
   vercel login
   ```

3. **Link Project**:
   ```bash
   cd presentation
   vercel link
   ```

### Deploy to Production

From the `presentation/` directory:

```bash
vercel --prod
```

### Deploy to Preview

For testing before production:

```bash
vercel
```

---

## Continuous Deployment

### GitHub Integration

When connected to GitHub, Vercel automatically:
- Deploys **production** on pushes to `main` branch
- Creates **preview deployments** for pull requests

### Custom Domain

To add a custom domain:

1. Go to Vercel Dashboard → Project Settings → Domains
2. Add your domain (e.g., `slides.caro.sh`)
3. Configure DNS records as instructed

---

## Build Scripts Reference

| Script | Description |
|--------|-------------|
| `npm run dev` | Dev server for main presentation |
| `npm run build` | Build main presentation only |
| `npm run pitch` | Dev server for pitch presentation |
| `npm run build:pitch` | Build pitch presentation only |
| `npm run roadmap` | Dev server for roadmap presentation |
| `npm run build:roadmap` | Build roadmap presentation only |
| `npm run build:all` | Build all presentations (for deployment) |
| `npm run export` | Export main presentation to PDF |

---

## Troubleshooting

### Build Fails on Vercel

Check the build logs:
1. Go to Vercel Dashboard → Deployments
2. Click on the failed deployment
3. View the build logs

Common issues:
- **Missing dependencies**: Run `npm install` locally to verify
- **Build script fails**: Test `npm run build:all` locally
- **Wrong root directory**: Ensure Root Directory is set to `presentation`

### 404 on Slide Navigation

If navigating directly to `/3` or `/roadmap/5` returns 404:
1. Verify rewrite rules are configured in Vercel dashboard
2. Check the order of rewrite rules (more specific first)
3. Ensure rewrites use `(.*)` pattern, not specific paths

### Presentations Not Loading

1. Check that all presentations built successfully:
   ```bash
   ls -la dist/
   ls -la dist/pitch/
   ls -la dist/roadmap/
   ```

2. Verify base paths in build scripts
3. Check browser console for asset loading errors

### Local Build Issues

Clean and rebuild:

```bash
rm -rf dist node_modules
npm install
npm run build:all
```

---

## Additional Resources

- [Slidev Documentation](https://sli.dev/)
- [Vercel Documentation](https://vercel.com/docs)
- [Vercel Rewrites Guide](https://vercel.com/docs/edge-network/rewrites)
- [Vercel Headers Guide](https://vercel.com/docs/edge-network/headers)
