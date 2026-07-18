# Reconciliation: shipped design-system-v2 PRs ↔ the Fable-5 design-pass plan

**Author:** Claude Code (Opus 4.8) · **Date:** 2026-05-19
**Audience:** the agent executing `claude/caro-design-fable5-bea825`
("Caro.sh Design Pass — Fix Colors, Accessibility & Icons + Fable 5 Creative Polish")

> **TL;DR** — Four design-system PRs (#1155, #1156, #1158, #1159) are **already
> open, rebased onto current `main`, and CI-green** (only the pre-existing
> Vercel-SSO deploy check is red — see `caro-ftp`). They are **not "stalled /
> likely superseded"** — they *are* a large chunk of your Phase 1 and Phase 2.
> **Subtract them from your plan before you implement, or you will re-do the work
> and collide** on `tokens.css`, `LPMoments`, `LPTestimonials`, and `SEO.astro`.

---

## Why this matters mechanically

Your branch `claude/caro-design-fable5-bea825` sits at `origin/main`. **My four
PRs are not merged into `main` yet.** So if you branch from `main` and implement
your Phase 1 as written, you will independently re-remap the exact same
`tokens.css` orange values I already remapped — and when both land, one of us
gets a conflict or an overwrite. Two safe options:

1. **Merge my 4 PRs first** (they're green, small, reviewed), then branch — your
   plan shrinks to the genuinely-remaining items below.
2. **Stack your branch on top of mine** (`git merge` PR #1159's branch, which
   itself carries #1155's token work) so you build on the shipped state.

Either way: **reconcile first, implement second.**

---

## Exact overlap map (what is ALREADY shipped)

| Your plan item | Status | Where |
|---|---|---|
| P1 · `tokens.css`: `--color-link*`, `--color-border-hover`, `--color-bg-hover`, `--shadow-primary`, `--distro-accent` orange→`--accent` (light **and** dark) | ✅ **DONE** | PR #1155 |
| P1 · `LPDownload` white-on-gradient → tokenized | ✅ **DONE** (restructured to dark card on paper, gradient removed) | #1155 |
| P1 · `LPMoments` hardcoded text colors → tokens; orange divider/tints → red | ✅ **DONE** | #1156 |
| P1 · `LPTestimonials` surface composition (was paper) → `--bg-stage`; cards → `--bg-raised` | ✅ **DONE** | #1156 |
| P2 · `LPTestimonials` social-proof-bar emoji → non-emoji icons | ✅ **DONE** (terminal glyphs — see method note) | #1158 |
| P2 · `LPFeatures` emoji feature icons → non-emoji | ✅ **DONE** (terminal glyphs `■ ▸ ❯ $`) + 5 orange cleanups | #1158 |
| P2 · `LPPersonas` deleted rocket/shield/lightning SVGs → sanctioned mark | ✅ **DONE** (kyaro-mark) | #1158 |
| P2 · `SEO.astro` broken favicon 404s | ✅ **DONE** (broken PNG refs removed — see method note) | #1159 |
| P3 · hero headline → `--font-display` (Azeret Mono) | ✅ **DONE** (LPHero `.headline` only) | #1159 |
| token collision `--color-error #ef4444` vs `--accent #ef3333` | ✅ **DONE** (`--color-error → #dc2626`) | #1159 |

Baseline + after screenshots and the verbatim Claude-Design spec brief:
`.claude/audits/2026-05-18-design-system-v2/{spec/BRIEF.md,current-site,after-phase-1..4}/`.

---

## Three points that need your explicit RECONCILIATION (we differ)

1. **`LPMoments` — dark vs theme-aware.** Your plan calls its dark-only text a
   bug ("fails in light mode"). But Claude Design's **May 9 caro-l06 ruling**
   (in `spec/BRIEF.md`) says LPMoments is **intentionally `--bg-inverse`
   (always dark)** — it's the "decision-moment" surface. I implemented it as
   always-dark with tokenized text (#1156). **Do not make it theme-aware
   without re-opening that ruling with Claude Design** (design-dialogue Rule 2).

2. **Favicon — remove vs generate.** I *removed* the 3 broken PNG refs
   (`favicon-32x32`, `favicon-16x16`, `apple-touch-icon`) in #1159, keeping the
   working `favicon.svg` + `caro-pixel.png`. Your plan wants to *generate* the
   missing sized PNGs instead. Both fix the 404 — **pick one**; if you generate
   them, revert my SEO.astro deletion and re-point. (I left `og-image.png` and
   the `LandingPage` `og:image` gap **untouched** — those are yours.)

3. **Icon method — terminal glyphs vs `<Glyph>` SVG.** For LPFeatures /
   LPTestimonials I used **terminal glyphs** (`■ ▸ ❯ $`, Azeret Mono, `--accent`)
   because the spec brief sanctions them for "surfaces where the mascot doesn't
   fit," and the deferred v3 glyph pack (lock/book/star) isn't drawn yet. Your
   plan wants Fable to draw those glyphs and switch to `<Glyph>`. That's a fine
   **upgrade** — just know the emoji are already gone; you're swapping one
   sanctioned system for another, not fixing raw emoji.

---

## What is GENUINELY REMAINING (your plan is correct and additive here)

These I did **not** touch — they are real, high-value, non-overlapping work:

- **`LandingPage.astro` inline palette (11 pages).** I missed this entirely.
  **This is the single biggest lever in your plan — prioritize it.** None of my
  PRs touch `LandingPage.astro`.
- **Component-library orange fallback literals** (`var(--color-primary, #ff8c42)`
  in `ui/{Dropdown,Terminal,DistroSelector,Toggle,Link,IconButton}`,
  `config/distros.ts`). I remapped the **token layer** (so the resolved value is
  now red), but the hardcoded `#ff8c42` *fallbacks* remain as dead orange. Your
  `caro-ebg` sweep is valid.
- **Figtree `@font-face`** — never loaded; body still system-sans. Untouched.
- **Azeret Mono on all headings** — I did the hero headline only; h2/h3 across
  components still inherit system fonts. Untouched.
- **`Navigation.astro`** theme-toggle grays + orange literals + missing nav logo.
  Untouched.
- **Dead landing theme toggle** (`LPNavigation.astro` button has no handler).
  Untouched.
- **`og-image.png`** (missing → broken social preview) + `LandingPage` `og:image`.
  Untouched.
- **`kyaro-mark` redraw** (Fable) + `icon-manifest.json:23` prose-in-array bug.
  Untouched.
- **Regression-guard vitests** (no-orange-literal, asset-existence, `@font-face`).
  Excellent idea — I have none; please add them.

---

## Suggested revised delivery order

1. **Reconcile + merge my 4 PRs** (or stack on them). ~30 min, unblocks the rest.
2. **Your Phase 1 minus my overlap** = `LandingPage.astro` + component-lib
   fallback sweep + Navigation + dead toggle. This is where the real remaining
   color work is.
3. Phase 2 minus my overlap = og-image, `LandingPage` og:image, `<Glyph>`
   upgrade (optional), manifest bug.
4. Phase 3 (Figtree + heading fonts) and Phase 4 (Fable polish + mascot) as
   written — almost fully additive.

## Bead pointers (so the tracker reflects reality)

- `caro-ebg` (orange→accent ~80 components): **token layer done in #1155**;
  remaining = component fallback literals + `LandingPage.astro`.
- `caro-0qp` (emoji→icons): **homepage emoji removed in #1158** via terminal
  glyphs; remaining = LPHero/Navigation + the `<Glyph>` upgrade + v3 glyph draw.
- `caro-ftp` (Vercel SSO blocks auditors): still the only red check on my PRs —
  **not a design regression**; audit the local `npm run dev` build, as you planned.

*My original tracking epic (`caro-j15x`) and phase beads were wiped by a Dolt
re-sync during the rebase (known store-sync quirk). The GitHub PRs #1155/#1156/
#1158/#1159 are the durable record; the rebase-queue beads `caro-jac.24/.25/.35/.36`
are closed with evidence.*
