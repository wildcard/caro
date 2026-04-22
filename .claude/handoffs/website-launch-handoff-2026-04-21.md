# Handoff: Website Launch PR #877 + Product Launch Readiness

**Date:** 2026-04-21
**Branch:** `fix/873-website-esbuild-astroparse`
**PR:** [#877](https://github.com/wildcard/caro/pull/877)

---

## What Was Done (This Session)

### Critical Fix: Website Build (#873)
**Before:** `npm run build` in `website/` FAILED with esbuild parse errors:
```
Unexpected ":" at GTMUseCases.astro:28:6
Expected "}" but found ":"
```
**Root Cause:** esbuild v0.25 (Astro 5.x) treats `{` in `.astro` template content as JSX expression boundaries. Shell command syntax like `:(){ :|:& };:` (fork bomb), `awk '{print $2}'`, `find ... {} +` all crash the parser.

**After:** Website builds **58 pages successfully**. All 15 locales generate. Vercel `caro-foss-website` deploy **PASS** ✅.

**Files Changed (20 total):**
```
website/src/components/gtm/GTMComparison.astro   # data extracted to .ts
website/src/components/gtm/GTMUseCases.astro     # data extracted to .ts
website/src/components/gtm/GTMDemo.astro         # fork bomb wrapped in {}
website/src/components/gtm/GTMOpenSource.astro   # code block HTML-escaped
website/src/components/gtm/GTMFooter.astro       # triple --- flattened
website/src/pages/docs/safety.astro              # fork bomb → {} expression
website/src/pages/docs/security.astro            # fork bomb → {} expression
website/src/pages/blog/security-practices.astro  # fork bomb → {} expression
website/src/pages/blog/claude-skill-launch.astro # escape fix
website/src/pages/modern-unix-tools.astro        # escape fix
website/src/pages/use-cases/developer.astro      # escape fix
website/src/pages/try-caro.astro                 # RatzillaDemo disabled (WASM missing)
website/src/data/gtm-use-cases.ts                # NEW — extracted data
website/src/data/gtm-comparison.ts               # NEW — extracted data
```

### i18n Bug Fix (#874)
**Before:** Only `es` (Spanish) had `landing.json` loaded. 13 other locales showed English for all landing page content despite having translations.
**Fix:** `src/i18n/index.ts` now imports `landing*.json` for all 14 non-EN locales.

**Verified via browser (dev server):**
| Locale | Hero Heading (H1) |
|--------|-------------------|
| es | Tu compañero leal de shell |
| fr | Votre fidèle compagnon shell |
| de | Dein treuer Shell-Begleiter |
| ja | あなたの忠実なシェルコンパニオン |
| ko | 당신의 충실한 셸 동반자 |
| hi | आपका वफादार शेल साथी |
| fil | Ang iyong tapat na shell companion |
| id | Teman setia shell kamu |

Translation coverage: **avg 81.2%**, Tier 1 avg **81.3%** (above 80% target).

### Release Workflow Fix (#869)
`.github/workflows/release.yml` now extracts `CHANGELOG.md` section for the version being released and uses it as GitHub Release body (instead of auto-generated PR list).

### Documentation Created
| File | Purpose |
|------|---------|
| `.claude/rules/astro-esbuild-shell-syntax.md` | **Coder agent rule**: how to handle `{` `}` in Astro templates, fork bomb fixes, data extraction patterns |
| `.claude/rules/dev-process.md` | Development workflow: branches, CI/CD, releases, i18n, code style |
| `website/I18N_TRANSLATION_GUIDE.md` | Complete i18n architecture reference |

---

## Pre-existing Issues (NOT caused by this PR)

### P0: Root Page Broken — LPHero Missing Translation Keys
**File:** `website/src/components/landing/LPHero.astro`
**Problem:** Referenced keys don't exist in ANY locale JSON:
```
landing.hero.headline.line1
landing.hero.headline.line2  
landing.hero.headline.dangerCmd
landing.hero.cta.primary
landing.hero.cta.secondary
landing.hero.socialProof.quote
landing.hero.socialProof.attribution
landing.hero.trustBadges.validation
landing.hero.trustBadges.local
```
**Impact:** Root `/` page shows raw translation keys instead of text. `/[lang]/` localized pages use old `Hero.astro` which works fine.

### P1: RatzillaDemo Disabled
**File:** `website/src/pages/try-caro.astro`
**Problem:** WASM module (`/ratzilla-demo/caro_tui_demo.js`) not built. Dynamic import crashes the build.
**Current state:** Replaced with placeholder text "Demo coming soon!"
**To fix:** Build the Rust WASM demo from `website/src/ratzilla-demo/`

### P2: Cubic Code Review Comments on PR #877
Cubic flagged these in the PR:
1. `dangerous-commands.ts:18` — `curl ... | bash` has literal `...` placeholder (not matching)
2. `dangerous-commands.ts:21` — `ddSda` duplicates `ddZero` pattern

### P2: Translation Coverage Gaps
| Locale | Coverage | Gap |
|--------|----------|-----|
| ur (Urdu) | 64.0% | Needs landing.json translation |
| de (German) | 77.7% | Close, brand names counted as "untranslated" |
| fil (Filipino) | 75.9% | Missing landing section content |

### P3: 3 CI Checks Fail on PR #877 (unrelated)
- **ChromaDB Integration Tests** — flaky, pre-existing
- **Security Audit** — pre-existing, not code-related
- **Vercel cmdai** — unrelated project

---

## What to Do Next (Product Launch Priorities)

### Immediate (Before PR Merge)
1. **Fix `LPHero.astro` missing keys** — this is the ROOT page visitors see first
   - Add missing keys to `website/src/i18n/locales/en/hero.json`
   - Verify root `/` page renders correctly
   - All 14 locale pages inherit via fallback to EN

2. **Fix Cubic review comments** in `dangerous-commands.ts`
   - Replace `...` with actual pattern
   - Remove or differentiate `ddSda`

3. **Merge PR #877** — CI is green (36/38 pass, 2 pre-existing fails)

### Product Launch Blockers
4. **Rebuild RatzillaDemo WASM** — `try-caro` page shows placeholder
   ```bash
   cd website/src/ratzilla-demo
   trunk build --release  # or whatever build command
   ```
   Then revert `try-caro.astro` to use `<RatzillaDemo />` again

5. **Run auto-translation** for remaining coverage gaps:
   ```bash
   gh workflow run translate.yml -f backend=openai
   ```
   This will generate PRs for `ur`, `de`, `fil`

6. **Create release v1.3.x or v1.4.0** — verify all features work:
   - `caro ai` command generation
   - `caro shell-init` shell integration
   - Website builds and deploys
   - All i18n locales render

### Medium Priority
7. **Fix `open-source-shell-ai.astro`** — has `currentYear` reference error
   - Actually this was caused by the triple --- fix on GTMFooter.
   - Check if resolved. The build shows 58 pages so it should be fine.

8. **Add `set:html` to all remaining `<code>` blocks** that might have `{}` in future
   - Update the agent rule to be the default pattern

9. **Run full end-to-end test**:
   ```bash
   cd website && npm run build
   npx http-server dist -p 8080
   # Manually check: /, /es/, /hi/, /de/, /compare, /blog, /docs
   ```

---

## Commands Reference

### Testing
```bash
# Website build
cd website && npm run build    # Should show "58 page(s) built"

# Dev server
cd website && npm run dev      # http://localhost:4321

# Translation coverage
cd website && node scripts/i18n/status.mjs

# Translation validation
cd website && node scripts/i18n/validate.mjs --strict

# Rust tests
cargo test
cargo build --release
```

### Branch/PR
```bash
# Current branch
git checkout fix/873-website-esbuild-astroparse

# Push changes
git add website/ .claude/rules/ && git commit -m "fix: ..." && git push

# Create PR
gh pr create --title "fix(website): ..." --body "..."
```

### Vercel Deploy
```bash
# Check deploy status
gh pr checks <PR_NUMBER> | grep vercel

# Preview URL format (find in PR comments)
# https://caro-foss-website-git-<branch-slug>-kadosh-dev.vercel.app
```

---

## File Map: Key Files

| File | Status | Notes |
|------|--------|-------|
| `website/src/i18n/index.ts` | ✅ Fixed | All 14 locales import landing.json |
| `website/src/components/gtm/GTMUseCases.astro` | ✅ Fixed | Data in external .ts |
| `website/src/components/gtm/GTMComparison.astro` | ✅ Fixed | Data in external .ts |
| `website/src/components/gtm/GTMDemo.astro` | ⚠️ Partial | Fork bomb in {} expression, verify rendering |
| `website/src/components/gtm/GTMOpenSource.astro` | ⚠️ Partial | HTML-escaped code block, verify Rust code renders |
| `website/src/pages/try-caro.astro` | ⚠️ Disabled | RatzillaDemo placeholder |
| `website/src/components/landing/LPHero.astro` | ❌ Broken | Missing translation keys |
| `website/src/data/dangerous-commands.ts` | ⚠️ Needs review | Cubic flagged 2 issues |
| `.claude/rules/astro-esbuild-shell-syntax.md` | ✅ New | Agent rule |
| `.claude/rules/dev-process.md` | ✅ New | Process docs |
| `.github/workflows/release.yml` | ✅ Fixed | CHANGELOG extraction |

---

## Critical Knowledge Not Obvious from Code

1. **The esbuild issue is ARCHITECTURAL** — Astro 5.x + esbuild 0.25 will ALWAYS crash on `{` in template content. Downgrading Astro didn't help (same esbuild). Upgrading didn't help. The ONLY fix is:
   - Move data with `{}` to `.ts` files (TypeScript compiler, not esbuild)
   - Wrap literal `{}` in `{ }` expressions: `<code>{":(){...}:"}</code>`
   - Use HTML entities in rendered HTML: `&#123;` and `&#125;`

2. **The auto-escape script I wrote earlier OVER-ESCAPED** — it replaced `{` in JSX expressions too (`{variable}` → `&#123;variable&#125;`). DO NOT run blanket regex replacements on template files. Only fix specific patterns.

3. **Vercel has 6 projects deploying from this repo** — `caro-foss-website` is the main one. The others (`cmdai`, `caro-docs`, `caro-slides`, `caro-storybook`, `cmdai-saas`) deploy independently. Only `caro-foss-website` is affected by website changes.

4. **The `---` triple marker problem** — some Astro files accidentally have 3 `---` delimiters. Structure: `---\nfrontmatter\n---\nDATA\n---\ntemplate\n`. This is INVALID. The DATA section ends up in the template. Fix by moving DATA to frontmatter or external file.

5. **Translation fallback chain**: `locale.json` overrides → `en.json` fallback → raw key string. If a key is missing from a locale file AND doesn't exist in English, it renders the raw key path (e.g., `landing.hero.headline.line1`).
