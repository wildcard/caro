# Caro Website — AFTER Migration Design Audit

**Date:** 2026-07-12
**Scope:** Local dev build `http://localhost:4321/` (the migrated build — do NOT confuse with
production `caro.sh`, which is the un-migrated "before" baseline audited separately).
**Baseline diffed against:** `.claude/audits/2026-07-12/before/AUDIT.md` (production).
**Auditor:** Caro Frontend Design Engineer sub-agent
**Tools:** `mcp__Claude_Browser__*` (interaction, `getComputedStyle` measurement, network) +
`mcp__plugin_playwright_playwright__*` (screenshot persistence to disk + a second, independent
Playwright browser context used for the systematic page sweep — both agree on every measured
value where cross-checked).

Methodology notes carry over unchanged from the before-audit: WCAG 2.1 relative-luminance
contrast formula computed live from `getComputedStyle`, not eyeballed; alpha-composited
backgrounds resolved by walking the full ancestor chain and compositing every translucent layer
in source order, not a naive single-layer read.

---

## 1. Executive verdict

The migration **landed cleanly on the four intended surfaces it actually touched** — the global
`Layout.astro` token re-alias (beige paper `#f4f1df` / grey-900 `#2b2b2b` dark, `Figtree` +
`Azeret Mono` fonts) and the shared `Navigation.astro` + landing-page body content on **two** of
the four named landing pages (`/use-cases`, `/use-cases/developer`). It did **not** touch:

- The homepage's own component family (`LPNavigation.astro`, `LPHero.astro`'s sibling
  homepage-only sections: `.nav-brand`, `.nav-cta`, `.moment-example`, `.feature-highlight`),
  which still hardcodes the deprecated orange gradient and off-token reds.
- Two of the four named landing pages — `/ai-command-safety` and `/safe-shell-commands` — whose
  nav is wired to the **same un-migrated `LPNavigation.astro`** as the homepage, not the migrated
  `Navigation.astro` that `/use-cases` and `/use-cases/developer` use. This is the single most
  important finding in this audit: the "4 landing pages" migration is **2-of-4 complete at the
  nav level**, not 4-of-4.
- The `#ef4444` Tailwind-red collision named explicitly in `tokens.css`'s own warning comment —
  still present on every landing page (`.persona-card.highlight`, `.persona-tagline`,
  `.job-pain`, inline danger `<code>`, comparison-table `.no` cells).
- The shared `.download-section` ("Try Caro in 30 Seconds") component — still the deprecated
  orange gradient, appears on the homepage and at least 3 of 4 landing pages.
- The `/docs` subsite — shows **zero** measurable evidence of migration (see §2.6). This is very
  likely a separate build/deployment (per `dev-process.md`, `caro-docs` is a distinct Vercel
  project from `caro-foss-website`) rather than a gap in this branch's work, but from a pure
  end-user perspective the brand is still fragmented there today.

The holiday-theme regression risk called out in the task **did not materialize** — see §4. Where
the Christmas theme's CSS actually targets a class that exists in the current markup, its
`!important` overrides win cleanly against the new re-aliased tokens.

---

## 2. Migration verified (per-item pass/fail with measured evidence)

### 2.1 Landing pages — brand paper-and-ink identity

| Page | Body bg | Body text | Nav source component | Nav-brand color | Nav-CTA | Verdict |
|---|---|---|---|---|---|---|
| `/use-cases` | `rgb(244,241,223)` `#f4f1df` beige ✓ | `rgb(79,79,79)` `#4f4f4f` grey-700 ✓ | `Navigation.astro` | `#4f4f4f` ink (no orange) ✓ | solid `#ef3333`, `bgImage: none` — flat red ✓ | **PASS** |
| `/use-cases/developer` | `rgb(244,241,223)` beige ✓ | `#4f4f4f` ✓ | `Navigation.astro` | ink, no orange ✓ | flat red (same pattern) ✓ | **PASS** |
| `/ai-command-safety` | `rgb(244,241,223)` beige ✓ (body-level only) | `#4f4f4f` ✓ | **`LPNavigation.astro`** ✗ | `rgb(255,140,66)` `#ff8c42` orange ✗ | `linear-gradient(135deg,#ff8c42,#ff6b35)` ✗ | **PARTIAL FAIL** (nav unmigrated) |
| `/safe-shell-commands` | `rgb(244,241,223)` beige ✓ (body-level only) | `#4f4f4f` ✓ | **`LPNavigation.astro`** ✗ | `#ff8c42` orange ✗ | orange gradient ✗ | **PARTIAL FAIL** (nav unmigrated, plus `.feature-highlight`/`.privacy-asterisk`/`.cta-link` also orange — this page reuses more homepage-only unmigrated components than the other three) |

Body-level link color spot-check on `/use-cases`: `.drawer-link.active` "📋 All Use Cases" =
`rgb(239,51,51)` `#ef3333` — correctly migrated from the before-audit's `#ff8c42`.

**Unmigrated but present on every landing page checked:** the Tailwind-collision red `#ef4444`
(`.persona-card.highlight` border, `.persona-tagline`, `.job-pain` "Pain: 5/5", inline danger
`<code>`, comparison-table `.no` "✗" cells) — 10 unique selector+property hits on `/use-cases`
alone, 22 on `/safe-shell-commands`. This is the exact collision `tokens.css`'s header comment
warns about; it is **unchanged** from the before-audit.

### 2.2 Homepage + docs + content via `Layout.astro`

| Surface | Body bg | Body text | Result |
|---|---|---|---|
| Homepage, light | `rgb(244,241,223)` `#f4f1df` beige | `rgb(79,79,79)` `#4f4f4f` | **PASS** — was pure white before |
| Homepage, dark | `rgb(43,43,43)` `#2b2b2b` grey-900 | `rgb(244,241,223)` `#f4f1df` | **PASS** — was `#1a1a1a` (grey-950) before |
| `/docs` | `rgb(255,255,255)` pure white | `rgb(44,62,80)` `#2c3e50` (legacy `--color-text`) | **FAIL — unchanged from before-audit** |

`/docs` shows **no** measurable trace of the migration: body background is still pure white, body
text is still the pre-migration `#2c3e50`, `body`'s `font-family` computed value is
`-apple-system, "system-ui", "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif` with **no
"Figtree" anywhere in the stack**, and `h1`'s font-family is the same system fallback (no "Azeret
Mono"). The docs sidebar/body links ("Installation", "Quick Start", "Configuration",
"Configuration", "Safety Levels" — same 5 sample links as the before-audit, same text) still
compute to `rgb(231,76,60)` `#e74c3c`, the third, unrelated "Flat UI/Alizarin" red the before-audit
flagged as not present anywhere in `tokens.css`. 21 total `#e74c3c` hits swept on the page (down
from before's un-deduped count, same order of magnitude). This is almost certainly because
`/docs` is a structurally separate build (see `dev-process.md`: `caro-docs` is a distinct Vercel
project from `caro-foss-website`) that never imports `website/src/ui/tokens.css` or
`Layout.astro` — not a bug in this branch's diff, but a real, user-visible brand gap today.

### 2.3 Orange elimination site-wide

**Not achieved site-wide.** Raw orange-family (`#ff8c42`/`#ff6b35`) hit counts, deduped by
selector+property:

| Page | Orange hits (after) | Orange hits (before, for reference) | Source |
|---|---|---|---|
| Homepage (dark, default) | 33 | 84 | `LPNavigation.astro` nav-brand/nav-cta, `.moment-divider`, `.moment-example.good`, `.nl-input`, `.feature.feature-highlight` (×2), `.download-section` |
| `/use-cases` | 1 | 36 | `.download-section` only |
| `/use-cases/developer` | 1 | 11 | `.download-section` only |
| `/ai-command-safety` | 4 | 22 | `.nav-brand` (×2 props), `.nav-cta`, `.download-section` |
| `/safe-shell-commands` | 12 | 16 | `.nav-brand` (×2), `.nav-cta`, `.feature-highlight` (×2), `.privacy-asterisk` (×2), `.cta-link` (×2), `.download-section` |

Every page's count dropped substantially (this is real, measured progress — /use-cases went from
36 raw hits to 1), but **zero pages reached 0**. The `.download-section` shared component is the
single most consistent surviving offender — it appears on 5 of 5 pages checked and always
resolves to the exact deprecated `linear-gradient(135deg, #ff8c42 0%, #ff6b35 100%)`.

**Nav CTA "flat signal-red, not orange gradient" claim:** verified TRUE on `/use-cases` and
`/use-cases/developer` (`background-image: none`, solid `#ef3333`). Verified **FALSE** on the
homepage, `/ai-command-safety`, and `/safe-shell-commands` — all three still render
`linear-gradient(135deg, rgb(255,140,66) 0%, rgb(255,107,53) 100%)` on the nav CTA, because all
three load `LPNavigation.astro` rather than the migrated `Navigation.astro`.

### 2.4 Fonts

| Check | Result |
|---|---|
| `body` font-family (homepage, `/use-cases`, `/use-cases/developer`) | `Figtree, -apple-system, "system-ui", "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif` — **PASS**, Figtree first |
| `h1`/`h2` font-family (same pages) | `"Azeret Mono", ui-monospace, SFMono-Regular, Menlo, monospace` — **PASS** |
| Figtree network requests | `GET /fonts/Figtree-{Regular,Medium,SemiBold,Bold,Italic}.ttf` → **200 OK** (self-hosted, confirmed via both Claude_Browser and Playwright network logs; second pass showed `304 Not Modified`, i.e. cached, not re-erroring) |
| Azeret Mono network requests | Google Fonts CSS (`fonts.googleapis.com/css2?family=Azeret+Mono...`) → 200, actual glyph file `fonts.gstatic.com/.../3XFuErsiyJsY9O_Gepph-HHhZfn23vRK.woff2` → 200. Still CDN-loaded, not self-hosted (matches `tokens.css`'s own header comment noting this is a tracked follow-up, not a regression) |
| `/docs` body/h1 font-family | System-font stack only, **no Figtree, no Azeret Mono** — **FAIL**, consistent with §2.2 |
| No 404s on any font request | Confirmed — no font 404s anywhere in the sweep. (Two *unrelated* 404s were incidentally observed sitewide: `/favicon-32x32.png` and `/favicon-16x16.png`, unchanged from the before-audit's P1 finding — out of scope for this migration but flagged for completeness since they were directly observed in this session's console log.) |

---

## 3. Contrast re-measurements (before → after, WCAG 2.1, AA = 4.5:1 text / 3:1 large-text-or-UI)

| Element | Before | After | Ratio (after) | AA Result | Changed? |
|---|---|---|---|---|---|
| Homepage hero subtitle (20px paragraph) | `#7f8c8d` on `#ffffff` = 3.48:1 FAIL | `#7a7a7a` (`--caro-grey-500`) on `#f4f1df` beige | **3.78:1** | Still FAIL (4.5 threshold); PASS large-text/UI 3:1 | **Improved** (3.48→3.78) but still fails body-text AA. Token changed from an off-brand hex to the correct `--caro-grey-500` primitive. |
| Homepage attribution ("— Engineering Teams Everywhere") | `#7f8c8d` on `#ffffff` = 3.48:1 FAIL | `#7a7a7a` on `#ffffff` card (`--bg-raised` white is itself correct per tokens.css) | **4.29:1** | Still FAIL by a narrow margin | **Improved** (3.48→4.29), same narrow-miss pattern as the before-audit's footer tagline |
| Homepage footer links ("GitHub"/"AGPL-3.0"/"Contributing") | `#ecf0f1` on `#2c3e50` = 9.57:1 PASS | `#f4f1df` beige on `#4f4f4f` grey-700 | **7.21:1** | PASS | Still passes; ratio dropped somewhat (9.57→7.21) because the surface changed color families, but nowhere near the failure boundary |
| Homepage `.section-label` "SEE IT IN ACTION" | `#ff6b35` orange on `#ffffff` (not separately measured before — orange was flagged as forbidden, not contrast-tested) | `#e63636` (`--accent-hover`) on `#f4f1df` beige | **3.74:1** | PASS large-text/UI, FAIL body-text | **Fixed to brand red family** (was forbidden orange) |
| Homepage `.section-label` "TRUSTED BY DEVELOPERS" | `#ef3333` on `#ffffff` (already brand-correct before) | `#ef3333` on `#f4f1df` beige | **3.56:1** | PASS large-text/UI, FAIL body-text | Same token, background changed from white to beige; ratio moved from unmeasured to 3.56 |
| Homepage `.hero-badge` "Companion Agent" | `#ef3333` text on `#fde7e7` composited = 3.41:1 (fails 4.5, passes 3, borderline) | `#ef3333` text on hero-gradient-composited bg, range **2.26:1 – 3.03:1** depending on gradient stop | **2.26–3.03:1** | **FAIL** at most of the gradient (below 3:1 for the majority of the badge's surface) | **Regressed at the low end** — before was a flat 3.41 (borderline pass of 3:1); after ranges below 3:1 across part of the badge because it now sits on a two-stop grey→beige gradient rather than a flat white hero bg |
| Homepage `.blocked-badge` "⛔ BLOCKED" | `#ef3333` on composited `#d4bfba` = 2.31:1 FAIL | `#ef3333` on composited `rgb(212,191,186)` | **2.31:1** | **FAIL — identical, unchanged** | Same component, not touched by this migration |
| Homepage `.badge.danger` "⛔ BLOCKED" (2nd component) | `#ff5555` on composited (5-layer chain) = 2.08:1 FAIL | `#ff5555` on the same composited chain | **2.08:1** | **FAIL — identical, unchanged** | Same off-token red (`#ff5555`, still not in `tokens.css`), same composite, not touched |
| `/docs` body links ("Installation"/"Quick Start"/"Configuration") | `#e74c3c` on `#ffffff` = 3.82:1 FAIL | `#e74c3c` on `#ffffff` — identical hex pair | **3.82:1** | **FAIL — identical, unchanged** | `/docs` wasn't touched (§2.2); same colors produce the same ratio by definition |

**Summary:** Two genuinely new, correctly-migrated red-family badges/labels moved from
"forbidden orange, unmeasured" to "brand red, measured and borderline-passing large-text/UI".
Two body-copy elements (hero subtitle, attribution) improved their ratio but still fail the 4.5:1
body-text threshold — same shortfall pattern as before, just with a smaller gap. Three elements
(`.blocked-badge`, `.badge.danger`, `/docs` links) are **byte-for-byte unchanged failures** — not
regressions, but not fixed either; they were outside this migration's touched surfaces. One
element (`.hero-badge`) went from a flat borderline-pass number to a **range that dips below the
3:1 floor**, because its backdrop changed from a flat color to a two-stop gradient — worth a
follow-up measurement once the badge's own background is finalized.

---

## 4. Holiday-theme regression check — Christmas theme

**Activation method:** `HolidayDebugPanel.astro` renders a `theme-preview-btn` labeled "🎄
Christmas" directly on the homepage (no localStorage archaeology needed). Clicking it sets
`document.documentElement.className = "christmas"`.

**Source-level context** (read directly from `website/src/layouts/Layout.astro:190-235`): the
brand migration's re-alias lives in a plain `:root { --color-bg: var(--bg); ... }` block
(lines 197-213) with a code comment stating the intent explicitly: *"Editing the values in place
(not a `:root:root` bump) leaves the holiday-theme class overrides below untouched."* The
`.christmas { }` block (line 226 onward) follows immediately after, using `!important` on all of
its component-targeting rules (`.christmas .cta-button`, `.christmas .download-section`,
`.christmas .hero::before/::after`, etc.).

**Verdict: PASS for every element the theme actually targets** — confirmed with computed-style
proof, not just visual inspection:

| Target | Expected (from source) | Measured | Result |
|---|---|---|---|
| `.cta-button.primary` "Install Caro" | `linear-gradient(135deg, #c41e3a 0%, #a01830 100%)` | `linear-gradient(135deg, rgb(196,30,58) 0%, rgb(160,24,48) 100%)` | **PASS — exact match, not clobbered to brand red** |
| `.download-section` "Try Caro in 30 Seconds" | `linear-gradient(135deg, #c41e3a 0%, #228b22 100%)` | `linear-gradient(135deg, rgb(196,30,58) 0%, rgb(34,139,34) 100%)` | **PASS — exact match.** Notably this ALSO means Christmas mode incidentally masks the unrelated orange-gradient bug from §2.3 on this specific component |
| `.hero::before` (🎄 decoration) | `content: '🎄'`, top-left, animated sway | Rendered top-left of hero panel, screenshot-confirmed | **PASS** |
| `.hero::after` (🎅 decoration) | `content: '🎅'`, top-right, animated bob | Rendered top-right of hero panel, screenshot-confirmed | **PASS** |
| `--color-accent` / `--christmas-red` root vars | `#c41e3a` | `#c41e3a` | **PASS** |
| Base body bg/fg | Should remain whatever the light/dark brand token resolves to (Christmas doesn't redefine `--bg`/`--fg`, only decorates specific components) | `#f4f1df` beige / `#4f4f4f` grey-700 — **unchanged from non-Christmas state** | **Correct, not a bug** — the theme was never designed to reskin the base paper color, only to accent specific components |

**Caveat, not a cascade regression:** `.hero-badge` ("Companion Agent" pill), `.nav-brand`, and
`.nav-cta` do **not** turn festive. Investigated at the source level: the Christmas CSS targets
`.companion-badge`, `.logo`, and relies on `.cta-button` — but the current homepage markup
(`LPHero.astro`/`LPNavigation.astro`) uses `.hero-badge`/`.nav-brand`/`.nav-cta` instead.
`document.querySelector('.companion-badge')` and `.logo` both return `null` on the live page —
those classes don't exist in the current DOM at all. This is a **selector/markup naming drift**
between the older component family the holiday-theme CSS was written against and the newer `LP*`
family now rendering the homepage — not a specificity or cascade-order break caused by the
`--color-*` re-aliasing. Everywhere the class names still match, the `!important` override wins
cleanly.

**One narrow, confirmed anomaly, no visual consumer found:** `--color-bg-tertiary` inside
`.christmas` is defined as `#fff5f5` (line 230) but computes live to `#d1cfc9` (`--bg-stage`'s
grey-200 value, from the `:root` block at line 200) — i.e., this *one* custom property, unlike its
siblings in the same rule (`--color-accent`, `--christmas-red`, etc., which all resolve
correctly), does not win. Re-verified twice for reproducibility; both reads agree. Root cause not
fully isolated (candidates: Astro/Vite CSS-processing custom-property reordering, or a rule I
didn't locate in the grep sweep) — flagged as a P2 for whoever owns `Layout.astro`'s CSS, since no
component was found in this session that visibly consumes `--color-bg-tertiary` (no pink tint was
observed anywhere in the Christmas screenshots), so its practical severity is unconfirmed rather
than zero.

**Screenshot:** `.claude/audits/2026-07-12/after/home-christmas-theme-desktop.png` (full page) and
`.claude/audits/2026-07-12/after/home-christmas-hero-crop.png` (hero-section crop showing both
decorations and the red "Install Caro" / red-green download-section gradient clearly).

---

## 5. New regressions introduced by the migration

Distinguishing genuinely **new** problems from **pre-existing, unchanged** ones (the latter are
in §2/§3 above, not repeated here):

1. **`.hero-badge` contrast dipped below the 3:1 floor at part of its range** (§3) — a direct side
   effect of the hero section's backdrop changing from a flat white to a two-stop
   grey-200→beige-100 gradient. This is a genuinely new condition, not present in the before-audit
   (which measured a flat 3.41:1 against flat white). Worth a follow-up fix: either lighten the
   badge's alpha-tint or lock the badge to sit only on the beige end of the gradient.
2. **Two-tier landing-page migration created a new, more confusing brand-consistency problem than
   before.** Before, all four landing pages were uniformly orange (bad, but at least *consistent*).
   After, `/use-cases` and `/use-cases/developer` are clean brand red/beige while
   `/ai-command-safety` and `/safe-shell-commands` still show the orange nav — meaning a user
   clicking between these four pages (which are cross-linked as sibling "use case" pages) will see
   the brand flip inconsistently mid-session. This is a regression in perceived polish even though
   every individual page's *own* orange count went down.
3. No new orange/red hex values outside the already-cataloged families were introduced anywhere
   swept in this session — the "new" gradient stops used by the Christmas theme (`#a01830`,
   `#228b22`, `#ffd700` etc.) are pre-existing, intentional holiday-theme colors, not new brand
   drift.

No components were found that "assumed a white background" and now visibly break on beige — the
`--bg-raised: white` token for cards (quote/testimonial card, docs-style cards) is itself
correctly on-brand per `tokens.css`, so white-on-beige card surfaces are working as designed, not
a regression. I looked specifically for low-contrast text-on-card issues following the beige
change and did not find any beyond the pre-existing §3 items.

---

## 6. Remaining gaps (carried forward + newly precise-located)

| Gap | Severity | Status |
|---|---|---|
| `LPNavigation.astro` (homepage + `/ai-command-safety` + `/safe-shell-commands`) still hardcodes orange nav-brand + orange-gradient nav-cta | **P0** | Unfixed — root cause now precisely located to one file, not fixed in this pass |
| `.download-section` shared component still uses deprecated orange gradient, present on 5/5 pages checked | **P0** | Unfixed |
| `#ef4444` Tailwind-collision red still present on every landing page (`.persona-card.highlight`, `.persona-tagline`, `.job-pain`, danger `<code>`, `.no` cells) | **P0** | Unfixed, exact same named collision `tokens.css` warns about |
| `/docs` subsite shows no measurable trace of migration (white bg, `#2c3e50` text, `#e74c3c` links, no Figtree/Azeret Mono) | **P0 from a user-facing brand-consistency view**, likely **out of this branch's scope** (separate Vercel project) | Unfixed; needs an owner decision on whether `caro-docs` gets its own migration pass |
| Homepage's `.moment-example.good/.bad/.blocked`, `.feature.feature-highlight`, `.nl-input` still hardcode orange/off-token reds | **P1** | Unfixed |
| `.blocked-badge` (2.31:1) and `.badge.danger` (2.08:1) still fail 3:1 UI-contrast floor, unchanged | **P1** | Unfixed |
| `.hero-badge` contrast now dips below 3:1 at part of its gradient range (new, see §5.1) | **P1 (new)** | Needs follow-up |
| Favicon 404s (`/favicon-16x16.png`, `/favicon-32x32.png`) | **P1** | Unchanged, out of migration scope, incidentally reconfirmed |
| `--color-bg-tertiary` doesn't inherit the Christmas override despite sibling vars in the same rule doing so correctly | **P2** | Newly identified this session, no confirmed visual consumer |
| Hero subtitle / attribution text still fail 4.5:1 body-text AA (improved but not fixed) | **P2** | Improved, not closed |
| Azeret Mono still CDN-loaded (Google Fonts), not self-hosted | **P2** | Documented pre-existing follow-up in `tokens.css`'s own header comment, not a regression |

---

## Screenshot index

All paths relative to repo root
`/Users/kobik-private/workspace/caro/.claude/worktrees/strange-goldberg-536bc2/`:

| File | Surface | Theme | Notes |
|---|---|---|---|
| `.claude/audits/2026-07-12/after/home-light-desktop.png` | Homepage | light (fresh-session default) | Full page |
| `.claude/audits/2026-07-12/after/home-dark-desktop.png` | Homepage | dark (toggled) | Full page |
| `.claude/audits/2026-07-12/after/use-cases-light-desktop.png` | `/use-cases` | light | Full page, clean migration |
| `.claude/audits/2026-07-12/after/use-cases-developer-light-desktop.png` | `/use-cases/developer` | light | Full page, clean migration |
| `.claude/audits/2026-07-12/after/ai-command-safety-light-desktop.png` | `/ai-command-safety` | light | Full page, orange nav visible |
| `.claude/audits/2026-07-12/after/safe-shell-commands-light-desktop.png` | `/safe-shell-commands` | light | Full page, orange nav + extra unmigrated components visible |
| `.claude/audits/2026-07-12/after/docs-light-desktop.png` | `/docs` | light (only mode available) | Full page, unmigrated white/slate |
| `.claude/audits/2026-07-12/after/home-christmas-theme-desktop.png` | Homepage | Christmas theme active | Full page |
| `.claude/audits/2026-07-12/after/home-christmas-hero-crop.png` | Homepage hero section | Christmas theme active | Viewport crop, decorations + red CTA clearly visible |

---

## Cross-session note on default color-scheme behavior

The before-audit recorded the homepage as defaulting to dark mode on production. In this session,
a fresh Playwright browser context (no cookies/localStorage) loaded the homepage in **light**
mode, while a fresh `Claude_Browser` preview context loaded it in **dark** mode. Both are
plausible: the app likely respects `prefers-color-scheme` and the two tools' underlying browser
profiles may differ in OS-reported scheme, or there's a genuine default-theme change bundled in
this migration that wasn't explicitly called out in the task brief. Not asserted as fixed or
broken either way — flagged as an open question rather than a claim, since it wasn't isolated to
a single deterministic cause in this session.
