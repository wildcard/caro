# Deploying cmdai DevRel Website to Vercel

> **Complete guide for deploying the cmdai Developer Relations website to Vercel**

---

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Deploy (Recommended)](#quick-deploy-recommended)
3. [Manual Setup](#manual-setup)
4. [Configuration](#configuration)
5. [Custom Domain](#custom-domain)
6. [Environment Variables](#environment-variables)
7. [Preview Deployments](#preview-deployments)
8. [Production Deployment](#production-deployment)
9. [Troubleshooting](#troubleshooting)
10. [Alternative Platforms](#alternative-platforms)

---

## ✅ Prerequisites

Before deploying, ensure you have:

- [ ] **GitHub account** with access to the cmdai repository
- [ ] **Vercel account** (free tier works great!)
  - Sign up at: https://vercel.com/signup
  - Recommended: Sign up with GitHub for seamless integration
- [ ] **Local build working**
  ```bash
  cd apps/devrel
  npm install
  npm run build  # Should complete successfully
  ```
- [ ] **All changes committed and pushed** to your branch

---

## 🚀 Quick Deploy (Recommended)

### Option 1: Vercel CLI (Fastest)

**1. Install Vercel CLI**
```bash
npm install -g vercel
```

**2. Login to Vercel**
```bash
vercel login
```

**3. Deploy from the DevRel directory**
```bash
cd apps/devrel
vercel
```

**4. Follow the prompts:**
```
? Set up and deploy "~/cmdai/apps/devrel"? [Y/n] y
? Which scope do you want to deploy to? [Your Team/Username]
? Link to existing project? [y/N] n
? What's your project's name? cmdai-devrel
? In which directory is your code located? ./
? Want to override the settings? [y/N] n
```

**5. Deploy to production**
```bash
vercel --prod
```

**✅ Done!** Your site is live! Vercel will give you a URL like:
```
https://cmdai-devrel.vercel.app
```

---

### Option 2: Vercel Dashboard (Most Visual)

**1. Go to Vercel Dashboard**
- Visit: https://vercel.com/dashboard
- Click **"Add New..."** → **"Project"**

**2. Import Git Repository**
- Click **"Import Git Repository"**
- Select your GitHub account
- Find and select **wildcard/cmdai**
- Click **"Import"**

**3. Configure Project**

**Framework Preset:**
- Select: **Next.js**

**Root Directory:**
- Click **"Edit"**
- Set to: `apps/devrel`
- ✅ **This is critical for monorepo!**

**Build Settings:**
- Build Command: `npm run build` (auto-detected)
- Output Directory: `.next` (auto-detected)
- Install Command: `npm install` (auto-detected)

**Environment Variables:**
- Leave empty for now (add later if needed)

**4. Deploy**
- Click **"Deploy"**
- Wait 1-2 minutes for build
- ✅ **Your site is live!**

---

## 🔧 Manual Setup

### Step-by-Step Vercel Configuration

**1. Create `vercel.json` (Optional)**

For more control, add configuration file:

```bash
cd apps/devrel
```

Create `vercel.json`:
```json
{
  "buildCommand": "npm run build",
  "outputDirectory": ".next",
  "devCommand": "npm run dev",
  "installCommand": "npm install",
  "framework": "nextjs",
  "regions": ["iad1"],
  "github": {
    "enabled": true,
    "autoAlias": true,
    "silent": false,
    "autoJobCancelation": true
  }
}
```

**2. Create `.vercelignore` (Optional)**

Ignore unnecessary files:
```
node_modules
.next
.cache
.turbo
*.log
.env*.local
```

---

## ⚙️ Configuration

### Monorepo Configuration

**Important:** Since DevRel website is in a subdirectory (`apps/devrel/`), you MUST configure the root directory.

#### Via Vercel Dashboard:

1. Go to your project
2. Click **Settings**
3. Go to **General**
4. Find **Root Directory**
5. Click **Edit**
6. Set to: `apps/devrel`
7. Click **Save**

#### Via `vercel.json`:

Add to repository root (`/cmdai/vercel.json`):
```json
{
  "buildCommand": "cd apps/devrel && npm run build",
  "outputDirectory": "apps/devrel/.next",
  "installCommand": "cd apps/devrel && npm install"
}
```

**Or** add to `apps/devrel/vercel.json`:
```json
{
  "buildCommand": "npm run build",
  "outputDirectory": ".next",
  "installCommand": "npm install"
}
```

### Build Configuration

**Next.js 16 with Turbopack:**
- ✅ Auto-detected by Vercel
- ✅ Uses Next.js 16's built-in optimizations
- ✅ Turbopack enabled by default in Next.js 16

**Custom Build Settings (if needed):**

In `apps/devrel/next.config.ts`, add:
```typescript
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'standalone', // For optimized deployments
  images: {
    domains: [], // Add allowed image domains here
  },
  experimental: {
    optimizeCss: true, // CSS optimization
  },
};

export default nextConfig;
```

---

## 🌐 Custom Domain

### Adding cmdai.dev or cmdai.com

**1. Purchase Domain (if needed)**
- Vercel Domains: https://vercel.com/domains
- Namecheap, GoDaddy, Cloudflare, etc.

**2. Add Domain in Vercel**

Via Dashboard:
1. Go to your project
2. Click **Settings** → **Domains**
3. Click **Add**
4. Enter domain: `cmdai.dev` or `www.cmdai.dev`
5. Click **Add**

**3. Configure DNS**

**Option A: Using Vercel Nameservers (Recommended)**
1. Vercel will provide nameservers
2. Update your domain registrar's nameserver settings
3. Wait for DNS propagation (5-60 minutes)

**Option B: Using A/CNAME Records**
1. Add A record:
   ```
   Type: A
   Name: @
   Value: 76.76.21.21
   ```
2. Add CNAME record:
   ```
   Type: CNAME
   Name: www
   Value: cname.vercel-dns.com
   ```

**4. Verify**
- Vercel will auto-verify
- HTTPS certificate auto-generated
- ✅ Your site is now at `cmdai.dev`!

### Subdomain for DevRel

If you want `devrel.cmdai.com`:

1. Add subdomain in Vercel
2. Add CNAME record:
   ```
   Type: CNAME
   Name: devrel
   Value: cname.vercel-dns.com
   ```

---

## 🔐 Environment Variables

### When to Use

Environment variables are needed for:
- API keys (if added later)
- Analytics (Google Analytics, Plausible, etc.)
- Feature flags
- External service configuration

### How to Add

**Via Dashboard:**
1. Go to project **Settings**
2. Click **Environment Variables**
3. Add variables:
   - `NEXT_PUBLIC_SITE_URL` → `https://cmdai.dev`
   - `NEXT_PUBLIC_GA_ID` → Your Google Analytics ID (if using)
4. Select environments (Production, Preview, Development)
5. Click **Save**

**Via CLI:**
```bash
vercel env add NEXT_PUBLIC_SITE_URL production
# Enter value: https://cmdai.dev

vercel env add NEXT_PUBLIC_GA_ID production
# Enter value: G-XXXXXXXXXX
```

### Environment Variable Naming

**For client-side (browser) access:**
- MUST start with `NEXT_PUBLIC_`
- Example: `NEXT_PUBLIC_API_URL`

**For server-side only:**
- No prefix needed
- Example: `API_SECRET_KEY`

---

## 👀 Preview Deployments

### How Preview Deployments Work

Every time you push to a branch or open a PR:
1. Vercel automatically deploys a preview
2. You get a unique URL: `cmdai-devrel-git-branch-name.vercel.app`
3. Perfect for testing before merging!

### Accessing Preview Deployments

**Via GitHub PR:**
- Vercel bot comments on your PR with preview URL
- Click to view your changes live

**Via Vercel Dashboard:**
- Go to your project
- Click **Deployments**
- See all preview deployments

### Testing Preview Deployments

```bash
# 1. Create a new branch
git checkout -b feat/new-feature

# 2. Make changes
# Edit files...

# 3. Commit and push
git add .
git commit -m "feat: Add new feature"
git push -u origin feat/new-feature

# 4. Vercel automatically deploys!
# Check GitHub PR or Vercel dashboard for URL
```

### Disabling Preview Deployments (if needed)

In `vercel.json`:
```json
{
  "github": {
    "enabled": true,
    "autoAlias": false
  }
}
```

---

## 🚢 Production Deployment

### Automatic Production Deployment

By default, Vercel deploys to production when you:
- ✅ Push to `main` branch
- ✅ Merge a PR to `main`

### Manual Production Deployment

**Via CLI:**
```bash
cd apps/devrel
vercel --prod
```

**Via Dashboard:**
1. Go to **Deployments**
2. Find the deployment you want to promote
3. Click **"..."** → **"Promote to Production"**

### Deployment Checklist

Before deploying to production:

- [ ] **All tests pass** (GitHub Actions CI green ✅)
- [ ] **Build succeeds locally** (`npm run build`)
- [ ] **Lighthouse score > 90** (Performance, Accessibility, SEO)
- [ ] **Custom domain configured** (if using)
- [ ] **Environment variables set** (if needed)
- [ ] **Mascot and brand assets added** (if ready)
- [ ] **Content reviewed** (no typos, broken links)
- [ ] **Mobile tested** (responsive design works)
- [ ] **Accessibility tested** (keyboard nav, screen readers)

### Post-Deployment

After deploying:
1. **Test the live site**
2. **Check analytics** (if configured)
3. **Monitor Vercel dashboard** for errors
4. **Share with team** for feedback

---

## 🐛 Troubleshooting

### Build Fails with "Module not found"

**Problem:** Missing dependencies

**Solution:**
```bash
cd apps/devrel
npm install
git add package-lock.json
git commit -m "chore: Update package-lock.json"
git push
```

### Build Fails with "NEXT_PUBLIC_X is not defined"

**Problem:** Environment variable not set

**Solution:**
1. Go to Vercel Dashboard → Settings → Environment Variables
2. Add the missing variable
3. Redeploy: **Deployments** → **"..."** → **"Redeploy"**

### "Error: Root directory not found"

**Problem:** Vercel is looking in the wrong directory

**Solution:**
1. Go to **Settings** → **General**
2. Edit **Root Directory** → Set to `apps/devrel`
3. Click **Save**
4. Redeploy

### Fonts Not Loading

**Problem:** Google Fonts blocked or slow

**Solution:**
In `app/layout.tsx`, ensure proper font configuration:
```tsx
// Fonts are loaded via CSS @import in globals.css
// Vercel auto-optimizes these
```

Or use Next.js font optimization:
```tsx
import { Press_Start_2P } from 'next/font/google';

const pressStart = Press_Start_2P({
  weight: '400',
  subsets: ['latin'],
});
```

### Deployment Slow or Timing Out

**Problem:** Build takes too long

**Solution:**
1. Check bundle size: `npm run build` locally
2. Optimize images: Convert to WebP
3. Remove unused dependencies: `npm prune`
4. Enable output caching in `next.config.ts`:
   ```typescript
   const nextConfig = {
     output: 'standalone',
   };
   ```

### Preview Deployments Not Appearing

**Problem:** GitHub integration not connected

**Solution:**
1. Go to **Settings** → **Git**
2. Check **GitHub** connection
3. Reconnect if needed
4. Enable **Automatic Deployments**

---

## 🌍 Alternative Platforms

If you prefer not to use Vercel, here are alternatives:

### Netlify

**Pros:** Similar to Vercel, great for Next.js
**Setup:**
1. Connect GitHub repo
2. Set build directory: `apps/devrel`
3. Build command: `npm run build`
4. Publish directory: `.next`

**URL:** https://netlify.com

### Cloudflare Pages

**Pros:** Global CDN, free tier is generous
**Setup:**
1. Connect GitHub repo
2. Framework preset: **Next.js (Static HTML Export)**
3. Root directory: `apps/devrel`
4. Build command: `npm run build`

**URL:** https://pages.cloudflare.com

### GitHub Pages (Static Export)

**Pros:** Free, simple
**Cons:** Requires static export (no server features)

**Setup:**
1. Add to `next.config.ts`:
   ```typescript
   const nextConfig = {
     output: 'export',
     images: { unoptimized: true },
   };
   ```
2. Build: `npm run build`
3. Deploy `out/` directory to GitHub Pages

### Self-Hosted (Docker)

**For advanced users:**

Create `Dockerfile`:
```dockerfile
FROM node:22-alpine

WORKDIR /app
COPY apps/devrel/package*.json ./
RUN npm ci
COPY apps/devrel/ ./
RUN npm run build

EXPOSE 3000
CMD ["npm", "run", "start"]
```

Build and run:
```bash
docker build -t cmdai-devrel .
docker run -p 3000:3000 cmdai-devrel
```

---

## 📊 Monitoring & Analytics

### Vercel Analytics

**Built-in analytics:**
1. Go to project dashboard
2. Click **Analytics**
3. View:
   - Page views
   - Unique visitors
   - Top pages
   - Performance metrics

**Enable Web Vitals:**
```bash
npm install @vercel/analytics
```

In `app/layout.tsx`:
```tsx
import { Analytics } from '@vercel/analytics/react';

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Analytics />
      </body>
    </html>
  );
}
```

### Google Analytics (Optional)

**Setup:**
1. Create GA4 property
2. Get Measurement ID (G-XXXXXXXXXX)
3. Add to Vercel environment variables:
   - `NEXT_PUBLIC_GA_ID` → `G-XXXXXXXXXX`
4. Add to `app/layout.tsx`:
   ```tsx
   <Script
     src={`https://www.googletagmanager.com/gtag/js?id=${process.env.NEXT_PUBLIC_GA_ID}`}
     strategy="afterInteractive"
   />
   ```

---

## 🎯 Best Practices

### Performance

- ✅ Use `<Image>` component for all images
- ✅ Enable static generation where possible
- ✅ Optimize fonts (Next.js auto-optimizes Google Fonts)
- ✅ Minimize client-side JavaScript
- ✅ Use `loading="lazy"` for below-fold images

### SEO

- ✅ Set proper metadata in `layout.tsx`
- ✅ Use semantic HTML
- ✅ Add `alt` text to all images
- ✅ Create `robots.txt` and `sitemap.xml`
- ✅ Configure Open Graph tags

### Security

- ✅ Use environment variables for secrets
- ✅ Enable HTTPS (auto with Vercel)
- ✅ Set proper CORS headers if needed
- ✅ Sanitize user inputs (if any)

### Monitoring

- ✅ Enable Vercel Analytics
- ✅ Set up error tracking (Sentry optional)
- ✅ Monitor Core Web Vitals
- ✅ Check deployment logs regularly

---

## 📚 Additional Resources

### Documentation

- **Vercel Docs:** https://vercel.com/docs
- **Next.js Deployment:** https://nextjs.org/docs/deployment
- **Next.js 16 Docs:** https://nextjs.org/docs

### Support

- **Vercel Support:** https://vercel.com/support
- **Vercel Discord:** https://vercel.com/discord
- **Next.js Discord:** https://nextjs.org/discord

### Tutorials

- **Vercel Quickstart:** https://vercel.com/docs/getting-started-with-vercel
- **Next.js Learn:** https://nextjs.org/learn

---

## ✅ Quick Reference

### Deploy Commands

```bash
# Preview deployment
vercel

# Production deployment
vercel --prod

# Deploy specific branch
vercel --prod --yes

# Check deployment status
vercel ls

# View logs
vercel logs [deployment-url]

# Remove project
vercel remove [project-name]
```

### Configuration Files

```
apps/devrel/
├── vercel.json          # Vercel configuration (optional)
├── .vercelignore        # Files to ignore (optional)
├── next.config.ts       # Next.js configuration
└── .env.local           # Local environment variables (gitignored)
```

---

**Ready to deploy? Start with the [Quick Deploy](#quick-deploy-recommended) section!** 🚀

**Questions?** Check [Troubleshooting](#troubleshooting) or ask in #devrel-website Slack channel.
