# Slidev Presentation Deployment Guide

This guide explains how to deploy the Caro Slidev presentations to Vercel.

## Overview

The project contains two Slidev presentations:
- **Main Presentation** (`slides.md`) - Technical overview of Caro
- **Roadmap Presentation** (`roadmap-slides.md`) - 2026 development roadmap

Both presentations are built and deployed together on Vercel.

## Deployment URLs

When deployed to Vercel:
- Main presentation: `https://your-domain.vercel.app/`
- Roadmap presentation: `https://your-domain.vercel.app/roadmap/`

## Build Process

### Local Build

Build both presentations locally:

```bash
cd presentation
npm install
npm run build:all
```

This will:
1. Build the main presentation to `dist/`
2. Build the roadmap presentation to `dist-roadmap/` with base path `/roadmap/`
3. Copy roadmap build to `dist/roadmap/`
4. Clean up temporary `dist-roadmap/` directory

### Output Structure

```
dist/
├── index.html          # Main presentation
├── assets/             # Main presentation assets
├── roadmap/
│   ├── index.html     # Roadmap presentation
│   └── assets/        # Roadmap presentation assets
└── ...
```

## Vercel Configuration

The project is configured via `vercel.json` at the repository root:

```json
{
  "buildCommand": "cd presentation && npm install && npm run build:all",
  "outputDirectory": "presentation/dist",
  "installCommand": "cd presentation && npm install"
}
```

### Key Configuration Details

- **buildCommand**: Runs the `build:all` script to build both presentations
- **outputDirectory**: Points to `presentation/dist` where the built files are
- **installCommand**: Installs npm dependencies in the `presentation/` directory

## Manual Deployment to Vercel

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
   vercel link
   ```
   - Choose your team/account
   - Confirm the project name
   - Link to the repository

### Deploy to Production

From the repository root:

```bash
vercel --prod
```

This will:
1. Build both presentations using the `build:all` script
2. Deploy the `presentation/dist/` directory
3. Provide deployment URLs

### Deploy to Preview

For testing before production:

```bash
vercel
```

This creates a preview deployment with a unique URL.

## Continuous Deployment

### GitHub Integration

When connected to GitHub, Vercel automatically:
- Deploys **production** on pushes to `main` branch
- Creates **preview deployments** for pull requests

### Environment Variables

No environment variables are required for the presentations.

### Custom Domain

To add a custom domain:

1. Go to Vercel Dashboard → Project Settings → Domains
2. Add your domain (e.g., `slides.caro.sh`)
3. Configure DNS records as instructed

## Build Scripts Reference

| Script | Description |
|--------|-------------|
| `npm run dev` | Dev server for main presentation |
| `npm run build` | Build main presentation only |
| `npm run roadmap` | Dev server for roadmap presentation |
| `npm run build:roadmap` | Build roadmap presentation only |
| `npm run build:all` | Build both presentations (for deployment) |
| `npm run export` | Export main presentation to PDF |
| `npm run roadmap:export` | Export roadmap to PDF |

## Troubleshooting

### Build Fails on Vercel

Check the build logs:
1. Go to Vercel Dashboard → Deployments
2. Click on the failed deployment
3. View the build logs

Common issues:
- **Missing dependencies**: Run `npm install` locally to verify
- **Build script fails**: Test `npm run build:all` locally
- **Wrong output directory**: Verify `vercel.json` points to `presentation/dist`

### Presentations Not Loading

1. Check that both presentations built successfully:
   ```bash
   ls -la presentation/dist/
   ls -la presentation/dist/roadmap/
   ```

2. Verify base paths:
   - Main: No base path (root)
   - Roadmap: `--base /roadmap/`

3. Check browser console for asset loading errors

### Local Build Issues

Clean and rebuild:

```bash
cd presentation
rm -rf dist dist-roadmap node_modules
npm install
npm run build:all
```

## Development Workflow

1. Make changes to `slides.md` or `roadmap-slides.md`
2. Test locally with `npm run dev` or `npm run roadmap`
3. Build and test with `npm run build:all`
4. Commit to feature branch
5. Create PR
6. Vercel creates preview deployment automatically
7. Review preview deployment
8. Merge PR → Auto-deploy to production

## Additional Resources

- [Slidev Documentation](https://sli.dev/)
- [Vercel Documentation](https://vercel.com/docs)
- [Vercel CLI Reference](https://vercel.com/docs/cli)
- [Slidev Theme: Seriph](https://github.com/slidevjs/themes/tree/main/packages/theme-seriph)

## Notes

- Both presentations use the `seriph` theme
- Presentations include Mermaid diagrams for visualization
- The Caro mascot (`mascot.gif`) is shared between both presentations
- All presentations are static sites (no server-side rendering needed)
