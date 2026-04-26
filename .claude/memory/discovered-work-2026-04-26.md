# Discovered Work Items — Session 2026-04-26

## ⚠️ Process Mistakes (Lessons Learned)

### L4: Direct Pushes to `main` Are Destructive
**Date:** 2026-04-26
**Symptom:** Multiple agents pushing code changes directly to `main`, causing
race conditions and potential work loss.
**Impact:** Other agents' parallel work on `main` could be overwritten or
conflict with these pushes.
**Fix:** Created `.github/INSTRUCTIONS.md` documenting the mandatory
worktree/branch/PR workflow.
**Rule:** ALL code changes must go through a dedicated worktree, branch, and PR.
Only `bd sync` and automated translation merges can land on `main` directly.

## 🔴 High Priority (Blocking Launch)

### P1: `validate` CI Check Fails on Every PR
**File:** `.github/workflows/validate-translations.yml`
**Symptom:** `validate` check fails on PR #877 with exit code 1.
Root cause: workflow expects `website/validation-report.txt` artifact but the
script produces `website/coverage-report.txt` instead (path mismatch).
**Fix:** Align artifact paths in the workflow file.

### P1: `test_get_model_fails_offline_for_uncached` Flaky Test
**File:** `tests/cache_contract.rs:100-121`
**Symptom:** Test expects `DownloadFailed` error to contain "network" or
"connection" but actual error message may vary based on reqwest's runtime
error format (depends on OS, network state, DNS resolution).
**Fix:** Either:
- Make the assertion more flexible: check for `CacheError::DownloadFailed`
  variant type rather than error message substring
- Or ensure error conversion from `reqwest::Error` always produces a
  predictable "network" prefix

### P1: Translation Coverage Below 80% for 5 Locales
**File:** `website/src/i18n/locales/{ur,fil,de,id,pt}/`
**Symptom:** Average coverage is 75.8%. Five locales are below target:
- ur: 61.6%
- fil: 57.1%  
- de: 73.0%
- id: 74.7%
- pt: 77.9%
**Fix:** Run auto-translation workflow: `gh workflow run translate.yml -f backend=openai`

### P1: Vercel `cmdai` Deploy Fails
**Symptom:** Vercel's `cmdai` project deployment fails on every PR. This is
a separate project from `caro-foss-website` but triggers a failing CI check
that could block merges.
**Fix:** Either fix the cmdai project or mark the Vercel check as non-blocking
in branch protection rules.

## 🟡 Medium Priority

### P2: RatzillaDemo WASM — Content Security Policy Risk
**File:** `website/src/components/RatzillaDemo.astro` lines 359-371
**Symptom:** The component fetches `/ratzilla-demo/index.html` to extract the
hashed WASM filename, then does a dynamic `import()`. If Vercel doesn't serve
directory contents correctly or CSP headers block inline module imports, the
demo will silently fall back to the error state.
**Fix:** Instead of fetching index.html at runtime:
- Use Astro build-time asset discovery to inject the correct filename
- Or add a build step in CI that scans `public/ratzilla-demo/` and generates
  a static `manifest.json` with the current filenames
- Alternative: configure Trunk to use `--filehash=false` (requires `^0.18`)

### P2: `mut terminal` Warning in WASM Build
**File:** `website/ratzilla-demo/src/lib.rs:618`
**Symptom:** `warning: variable does not need to be mutable`
**Fix:** Remove `mut` keyword from `let mut terminal = ...` since
`domBackend` doesn't require mutable terminal after creation.

### P2: `hero.json` Is Orphaned / Confusing
**File:** `website/src/i18n/locales/{en,es}/hero.json`
**Symptom:** These files export `{ "hero": { ... } }` which is spread into
the main translation object at root level (`translations.en.hero.*`).
But `LPHero.astro` uses `landing.hero.*` keys (from `landing.json`).
The `hero.json` files serve a different purpose (old Hero component) but
have overlapping keys causing confusion.
**Fix:** Rename `hero.json` → `legacy-hero.json` and add a comment explaining
it's for the old `Hero.astro` component, not `LPHero.astro`.

### P2: 24 Open PRs with No Clear Prioritization
**Symptom:** PR #877 merged, but 24+ PRs remain open including:
- 14 auto-translate PRs (automated, should auto-merge)
- Several feature PRs that may have drifted off `main`
- PR #805 "Close the Credibility Gap" — large feature PR, 16+ days old
**Fix:** Implement PR triage workflow (see project-manager agent).

### P3: `getrandom` Unused in Cargo.toml
**File:** `website/ratzilla-demo/Cargo.toml`
**Symptom:** `getrandom = { version = "0.2", features = ["js"] }` is listed
but `getrandom` is not imported in `src/lib.rs`. Leftover from old `rand`
dependency.
**Fix:** Remove `getrandom` from Cargo.toml.

### P3: Build Artifacts in `public/` Directory
**File:** `website/public/ratzilla-demo/*` (465KB binary files)
**Symptom:** WASM build artifacts committed to `public/`. If the build is
re-run with different hash, these will be out of sync with source.
**Fix:** Consider moving WASM build output to `dist/` and adding a CI step
that runs `trunk build --release` before the Astro build. Or add a git hook
that rebuilds WASM when source changes.

### P3: `search-index.json` Committed But Untracked Changes
**File:** `website/src/config/search-index.json`
**Symptom:** Every website build regenerates this file with new timestamps.
Git shows it as modified on every build, causing noise in status checks.
**Fix:** Add to `.gitignore` or regenerate it during CI deploy only.

## 🟢 Low Priority / Nice-to-Have

### P3: RatzillaDemo Uses Canvas Backend Instead of WebGL2
**File:** `website/ratzilla-demo/Cargo.toml`
**Symptom:** `ratzilla` supports WebGL2 rendering (`beamterm-renderer`)
for better performance. Current build uses DOM/Canvas backend which is
slower but simpler.
**Fix:** Add `ratzilla = { version = "0.2" }` — it already includes the
WebGL2 renderer. The `DomBackend` used in `lib.rs` is canvas-based. Switch
to `WebGl2Backend` for production-grade rendering.

### P3: No Integration Tests for RatzillaDemo
**File:** `website/` (missing test coverage)
**Symptom:** The WASM demo has no tests. A broken build or API change in
ratzilla would only be caught by manual review.
**Fix:** Add a simple build test that runs `trunk build --release` in CI.
Add a basic component test that verifies `/try-caro` renders without errors.

### P4: Rust LSP Errors on Non-Rust Files
**Symptom:** `rust-analyzer` reports syntax errors on `.md`, `.json`, `.toml`,
`.astro`, and `.ts` files. This is a Crush configuration issue — the
Rust LSP is indexing non-Rust files.
**Fix:** Configure `rust-analyzer.files.excludeDirs` and/or
`server.extraEnv.RA_LOG` in `crush.json` to exclude non-Rust files.

## 📋 Checklist for Next Session

- [ ] Create `.github/workflows/auto-merge-translations.yml` for auto-translate PRs
- [ ] Fix `validate-translations.yml` artifact path mismatch
- [ ] Fix flaky `test_get_model_fails_offline_for_uncached` cache contract test
- [ ] Run auto-translation workflow for `ur`, `fil`, `de`, `id`, `pt`
- [ ] Remove `getrandom` from `website/ratzilla-demo/Cargo.toml`
- [ ] Remove `mut` from WASM `src/lib.rs:618`
- [ ] Consider moving `search-index.json` out of git tracking
- [ ] Add WASM rebuild step to website CI pipeline
- [ ] Enforce worktree/branch/PR workflow (rule in `.github/INSTRUCTIONS.md`)
- [ ] Consider `WebGl2Backend` instead of `DomBackend` for RatzillaDemo
