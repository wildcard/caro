# Vercel Deployment Skip Configuration

## Changes Made

Added `presentation` and `apps/devrel` to npm workspaces array in `/package.json` to enable proper deployment skipping.

**Update (PR #684):** Added `apps/devrel` to fix cmdai deployment skipping.

## Why This Matters

Vercel treats files outside workspace folders as "global changes" that trigger ALL projects to build. By including all Vercel project folders in the workspaces array, only changed projects will deploy.

## Manual Vercel Dashboard Configuration Required

For EACH Vercel project listed below, verify these settings in the Vercel Dashboard:

### 1. caro-foss-website
- **Root Directory:** `website`
- **Skip Deployment:** ✓ Enabled

### 2. caro-docs
- **Root Directory:** `docs-site`
- **Skip Deployment:** ✓ Enabled

### 3. cmdai (DevRel site)
- **Root Directory:** `apps/devrel`
- **Skip Deployment:** ✓ Enabled

### 4. caro-slides (presentation)
- **Root Directory:** `presentation`
- **Skip Deployment:** ✓ Enabled

### 5. cmdai-saas (landing page)
- **Root Directory:** `landing`
- **Skip Deployment:** ✓ Enabled

### 6. caro-storybook
- **Root Directory:** `website`
- **Build Command:** Custom (likely `npm run build-storybook`)
- **Skip Deployment:** ✓ Enabled (optional - will build with website changes)

## How to Configure Each Project

1. Go to [Vercel Dashboard](https://vercel.com/dashboard)
2. Select the project
3. Navigate to: **Settings → Build & Development Settings**
4. Set **Root Directory** to the correct folder (e.g., `website`)
5. Scroll to **Ignored Build Step** section
6. Ensure the **Skip deployment** toggle is **Enabled**
7. Click **Save**

## Verification Tests

After configuring all projects and merging this PR:

### Test 1: Presentation-only change
```bash
# Make a change to presentation/ only
echo "test" >> presentation/README.md
git add presentation/README.md
git commit -m "test: presentation change"
git push
```

**Expected:** Only `caro-slides` builds, others show "Skipped"

### Test 2: Website-only change
```bash
# Make a change to website/ only
echo "test" >> website/README.md
git add website/README.md
git commit -m "test: website change"
git push
```

**Expected:** Only `caro-foss-website` (and optionally `caro-storybook`) build

### Test 3: Root-level change
```bash
# Make a change outside workspaces
echo "test" >> README.md
git add README.md
git commit -m "test: root change"
git push
```

**Expected:** ALL projects build (this is correct behavior)

## Key Insight from Vercel

> A project in a monorepo is considered to be changed if any of the following conditions are true:
> 1. The project source code has changed
> 2. Any of the project's internal dependencies have changed
> 3. A change to a package manager lockfile has occurred that only impacts the dependencies of the project

Files outside the workspaces array are considered "global changes" and trigger all builds.

## Workspace Configuration

Complete workspaces in `/package.json`:
- `website` → caro-foss-website, caro-storybook
- `docs-site` → caro-docs
- `landing` → cmdai-saas
- `presentation` → caro-slides
- `apps/devrel` → cmdai ✨ **COMPLETE**

### Workspace to Vercel Project Mapping

| Workspace Folder | Vercel Projects | Notes |
|------------------|-----------------|-------|
| `website` | caro-foss-website, caro-storybook | Both build when website/ changes |
| `docs-site` | caro-docs | Documentation site |
| `landing` | cmdai-saas | Landing page |
| `presentation` | caro-slides | Slidev presentations |
| `apps/devrel` | cmdai | DevRel site |

**Verified Working:** PR #685 confirmed all projects skip correctly when only presentation/ changes.

## Troubleshooting

### How to Verify Configuration

Use the Vercel CLI to inspect monorepo configuration:

```bash
vercel link --repo
cat .vercel/repo.json
```

This shows which directory each Vercel project is configured to watch.

### Common Issues

1. **Project still building when it shouldn't:**
   - Check if the workspace folder is in `package.json` workspaces array
   - Verify Vercel project's Root Directory matches the workspace folder
   - Ensure "Skip deployment" toggle is enabled in Vercel dashboard

2. **All projects building on every commit:**
   - Changes to root-level files trigger all projects (expected)
   - Check if a workspace folder is missing from the array

## References

- [Vercel Monorepo Documentation](https://vercel.com/docs/concepts/monorepos)
- [Vercel Ignored Build Step](https://vercel.com/docs/concepts/projects/overview#ignored-build-step)
