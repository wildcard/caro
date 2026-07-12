# Caro Website — BEFORE Baseline Design Audit

**Date:** 2026-07-12
**Scope:** Production `https://www.caro.sh/` (pre-migration state — the design-system PRs in this
worktree are not yet merged, so production still reflects the legacy identity).
**Auditor:** Caro Frontend Design Engineer sub-agent
**Tools:** `mcp__Claude_Browser__*` (interaction, DOM/computed-style measurement, network/console)
+ `mcp__plugin_playwright_playwright__*` (screenshot persistence to disk only — Claude_Browser's
`computer` screenshot action returns inline image bytes with no filesystem path in this
environment, so Playwright was used purely as a capture mechanism against the same production
URLs, not as a second inspection methodology).

This is the **BEFORE** snapshot. Its purpose is to give the upcoming design pass (and Claude
Design, if consulted) a measured, falsifiable baseline to diff against once the new tokens.css
identity ships to production.

---

## Methodology notes (read before trusting any number below)

- All contrast ratios use the WCAG 2.1 relative-luminance formula, computed from
  `getComputedStyle` in the live page, not eyeballed.
- Where a background is a **solid, opaque color**, the ratio is exact.
- Where a background involves **semi-transparent layers or CSS gradients**, a naive
  `getComputedStyle().backgroundColor` read is *wrong* (it ignores alpha-compositing and ignores
  `background-image` entirely). Two corrections were applied and are noted inline:
  1. Alpha-compositing the full ancestor chain (not just the first non-transparent layer).
  2. For gradients, sampling **both° endpoint stops individually** (not just an average) to report
     a defensible range rather than a single potentially-misleading number.
- An early pass produced several false "ratio: 1 (invisible)" artifacts for `.hero-badge`,
  `.copy-button`, `.badge.danger`, and `.trust-point` before this correction — those false
  positives are **not** included below. Every number in the tables below survived the corrected
  methodology.
- Colors are bucketed by exact hex, not by "looks orange-ish" — see the raw `orangeHits`/`redHits`
  JSON captured during the session for the full per-selector list; only representative samples are
  reproduced here.

---

## 1. Color / brand-token divergence

### 1.1 Deprecated orange gradient (`#ff8c42 → #ff6b35`) — still the dominant accent

The brand book calls this gradient explicitly forbidden. It is measured, live, on **every
surveyed page**, in `background-image: linear-gradient(135deg, rgb(255, 140, 66), rgb(255, 107,
53))` form or as the solid stops individually:

| Surface | Element | Property | Value |
|---|---|---|---|
| Homepage | `.nav-brand` "Caro" logo | `color` | `#ff8c42` |
| Homepage | `.nav-cta` "Get Started Free" | `background-image` | `linear-gradient(135deg, #ff8c42, #ff6b35)` |
| Homepage | `.download-section` ("Try Caro in 30 Seconds") | `background-image` | `linear-gradient(135deg, #ff8c42, #ff6b35)` (full-bleed section) |
| Homepage | `.back-to-top` FAB | `background-image` | same gradient |
| Homepage | `.section-label` "SEE IT IN ACTION" | `color`/`background-color`/`border` | `#ff6b35` |
| Homepage | `.moment-example.good` (the "good" before/after example) | `background-color` | `#ff6b35` |
| Homepage | `.feature.feature-highlight` cards ×2 | `background-image` | `linear-gradient(135deg, rgba(255,140,66,.03), #fff)` |
| Homepage | inline `<code>`, `.privacy-asterisk`, `.cta-link`, `.brand-name`, `.story-link` | `color` | `#ff8c42` |
| /use-cases | `.brand-text` (nav logo), `.nav-cta`, `.drawer-cta` | `background-image` | same gradient |
| /use-cases | `.hero-badge` "Jobs To Be Done" | `background-image` | `linear-gradient(135deg, rgba(255,140,66,.15), rgba(255,107,53,…))` |
| /use-cases | `.drawer-brand` "🐕 Caro", `.drawer-link.active` "📋 All Use Cases" | `color`/`background-color` | `#ff8c42` |
| /use-cases | `.subtitle-emphasis` "Find your role below…" | `color` | `#ff8c42` |
| /use-cases/developer | `.drawer-link.active` "💻 Developers", `.drawer-brand` | `color` | `#ff8c42` |
| /ai-command-safety | `.nav-brand`, `.hero-badge` "AI Shell Commands You Can…", `.cta-button.secondary` "See Your Risks" | `color` | `#ff8c42` |
| /safe-shell-commands | `.nav-brand`, `.terminal-caro` | `color` | `#ff8c42` |
| /use-cases (separate CTA) | `.install-btn` "🐕 Install Caro" | `background-image` | `linear-gradient(#ff8c42, #e67635)` — a **fourth, distinct** orange gradient recipe |

Per-page raw counts of orange-family hits (deduped by selector+property+hex):
homepage dark = 84 raw color/bg/border matches (before hue-band correction), homepage light = 41,
/use-cases = 36, /use-cases/developer = 11, /ai-command-safety = 22, /safe-shell-commands = 16.
Every single surveyed page carries the deprecated orange in its nav bar at minimum.

**Severity: P0** — this is the exact gradient the brand book names as forbidden, live on 100% of
surveyed pages, in the single most prominent UI element (primary nav CTA) on every one of them.

### 1.2 Brand red is fragmented into (at least) four different hex values

The brand book specifies signal red `#ef3333` as the one accent. Production runs four distinct
reds simultaneously, sometimes on the **same page**, sometimes on the **same component class**:

| Hex | Where measured | Note |
|---|---|---|
| `#ef3333` | Homepage `.hero-badge`, `.danger-cmd`, `.cta-button.primary` "Install Caro", `.blocked-badge`, `.label.problem`, `.section-label` "TRUSTED BY DEVELOPERS", `.author-avatar`, `.submit-button` | Correct brand token |
| `#ff6464` | Homepage **dark mode** equivalents of the above (`.hero-badge`, `.danger-cmd`, `.cta-button.primary`, `.cta-button.secondary`) | Matches `tokens.css`'s documented dark-mode `--accent` — correctly on-brand |
| `#ef4444` | **All four** landing pages: `.persona-card.highlight` border, `.persona-tagline`, `.job-pain` "Pain: 5/5", `.learning-warning`, `.wrong-label` "❌ Common mistake", `.scenario-label.danger-label`, inline `<code>` danger examples, comparison-table `.no` "✗" cells | Tailwind red-500 — this is the exact collision `tokens.css`'s own header comment warns about ("Tailwind's `#ef4444` collides with `#ef3333`") |
| `#e74c3c` | Docs subsite (`/docs`): body links ("Installation", "Quick Start", "Configuration"), sidebar active-item pill | "Flat UI / Alizarin" red — a **third, unrelated** template default, not in `tokens.css` at all |
| `#ff5555` | Homepage `.moment-example.bad`/`.moment-example.blocked`, `.badge.danger` "⛔ BLOCKED" | A **fourth** red, also not in `tokens.css` |

Two directly-comparable same-page, same-semantic collisions:

- `.section-label` renders `#ff6b35` (orange) for "SEE IT IN ACTION" and `#ef3333` (brand red) for
  "TRUSTED BY DEVELOPERS" — **identical class, same homepage, two different hex families.**
- Two visually-identical "⛔ BLOCKED" pills exist as separate components: `.blocked-badge` uses
  `#ef3333`, `.badge.danger` uses `#ff5555`.
- At least three different "install / get started" CTA treatments coexist on production:
  nav `.nav-cta` "Get Started Free" (orange gradient), hero `.cta-button.primary` "Install Caro"
  (`#ef3333`/`#ff6464` solid), and `.install-btn` "🐕 Install Caro" on /use-cases (a fourth orange
  gradient recipe, different stops).

**Severity: P0** — direct, named collision with an existing `tokens.css` warning comment, present
on every landing page and the docs subsite.

### 1.3 95/5 paper-composition rule: the beige `--caro-beige-100` paper background appears nowhere

| Surface | `document.body` background-color | Expected |
|---|---|---|
| Homepage (default load) | `rgb(26,26,26)` / `.dark` class active | beige paper `#f4f1df` (95%), dark panels as 5% accent |
| Homepage (light toggle) | `rgb(255,255,255)` pure white | beige paper `#f4f1df` |
| /use-cases | `rgb(255,255,255)` pure white | beige paper |
| /use-cases/developer | `rgb(255,255,255)` pure white | beige paper |
| /ai-command-safety | `rgb(255,255,255)` pure white | beige paper |
| /safe-shell-commands | `rgb(255,255,255)` pure white | beige paper |
| /docs | `rgb(255,255,255)` pure white + separate grey sidebar/terracotta system | beige paper |

Production currently ships **four different surface languages** for what should be one 95/5
paper-and-ink system: (a) homepage dark-by-default, (b) homepage light-toggle white, (c) landing
pages permanently white, (d) docs subsite white-plus-terracotta. None of the four is the beige
paper token.

**Severity: P0** — foundational composition rule, 0-for-6 pages surveyed.

---

## 2. Logo / brand-mark inconsistency

Three different logo treatments were measured across three sub-surfaces of the same production
site:

| Surface | Logo treatment | Color |
|---|---|---|
| Homepage nav | Plain text "Caro", no icon | `#ff8c42` |
| Landing pages nav (`/use-cases*`, `/ai-command-safety`, `/safe-shell-commands`) | 🐕 (Unicode dog-face emoji) + "Caro" text | `#ff8c42` |
| Docs nav (`/docs`) | `caro-pixel.png` (real pixel-art mark asset) + "Caro" text | dark ink, no orange/red |

**Severity: P1** — not a broken surface, but three different "what is our logo" answers live at
once on the same domain.

---

## 3. Broken assets (network-verified — curl HTTP status, same-origin `fetch()`, AND live browser
console errors all agree)

| Resource | HTTP status | Verified via |
|---|---|---|
| `https://www.caro.sh/favicon-32x32.png` | **404** | curl, page `fetch()`, browser console error |
| `https://www.caro.sh/favicon-16x16.png` | **404** | curl, page `fetch()`, browser console error |
| `https://www.caro.sh/apple-touch-icon.png` (180×180) | **404** | curl, page `fetch()` |
| `https://www.caro.sh/favicon.ico` | **404** | curl, page `fetch()` |
| `https://www.caro.sh/caro-pixel.png` (the one `<link rel="icon">` with no `sizes` attr) | 200 OK | curl |
| `https://caro.sh/og-image.png` (declared in `<meta property="og:image">`) | **404** | curl (both `caro.sh` and `www.caro.sh`) |
| `https://www.caro.sh/caro-demo.mp4` (206 range request) | `net::ERR_ABORTED` mid-flight | live network log |

4 of the 5 `<link rel="icon"|"apple-touch-icon">` tags in `<head>` 404. Only the sizes-less
`caro-pixel.png` fallback resolves — meaning browsers that specifically request the 16×16 or
32×32 sized icon (most desktop browsers do) may fall back to a generic/blank tab icon rather than
any Caro mark. Separately, **any social share of a caro.sh link right now (Twitter/X, Slack,
LinkedIn, iMessage, Discord) has no working preview image** — the declared `og:image` 404s.

**Severity: P1** (favicons — cosmetic but sitewide and console-visible), **P1** (og-image — breaks
a real growth surface: every shared link looks broken).

---

## 4. Iconography

- **Confirmed emoji+SVG mixing in the same row**, homepage `.trust-badges`: 🔒 (Unicode emoji,
  "Zero Telemetry") and 📖 (Unicode emoji, "Open Source") sit in the same flex row as 2 real
  `<svg>` icons (shield/lightning, "52+ Safety Patterns" / "100% Local"). Visually confirmed in
  `home-light-mobile.png` (bottom badge row) and `home-dark-desktop.png`.
- Emoji used as functional/semantic glyphs throughout, none matching the brand book's four named
  (but never-delivered) pixel icons: 🚨 (SRE persona), 💻 (developer persona), 📋 (nav), 🐕 (logo,
  ×2 surfaces), ⚠️/❌ (warning states), 🍎 (macOS tab picker), 🎊 (seasonal banner). This matches
  the documented gap ("emoji placeholders are stand-ins, not the design intent") — still true,
  confirmed live.
- An unidentified **violet/purple** floating action button (audio/mute control, bottom-right on
  mobile, next to the orange back-to-top FAB) uses a hue with no home anywhere in `tokens.css`.
  Flagged for follow-up identification; not measured to exact hex (low priority, small surface).

**Severity: P1** (emoji/SVG mixing — tracked brand-book gap, reconfirmed live), **P2** (mystery
purple FAB — needs identification).

---

## 5. Measured contrast (WCAG 2.1, AA thresholds: 4.5:1 normal text, 3:1 large text/18.66px+bold
or 24px+, and UI components)

### 5.1 Solid colors (exact, no approximation)

| Element | Text | Color | Effective BG | Ratio | Threshold | Result |
|---|---|---|---|---|---|---|
| Homepage `.subtitle` (hero intro paragraph, 20px) | "A specialized POSIX shell command agent…" | `#7f8c8d` | `#ffffff` | **3.48:1** | 4.5 | **FAIL** |
| Homepage `.attribution` (social-proof quote byline) | "— Engineering Teams Everywhere" | `#7f8c8d` | `#ffffff` | **3.48:1** | 4.5 | **FAIL** |
| Homepage footer tagline | "Built with Rust. Privacy-first." | `#95a5a6` | `#2c3e50` | **4.29:1** | 4.5 | **FAIL** (narrow miss) |
| Docs body links | "Installation" / "Quick Start" / "Configuration" | `#e74c3c` | `#ffffff` | **3.82:1** | 4.5 | **FAIL** |
| Docs sidebar active pill | "Overview" | `#ffffff` | `#e74c3c` | **3.82:1** | 4.5 (text) / 3 (UI) | FAIL text / pass UI |
| Homepage footer links | "GitHub" / "Privacy" | `#ecf0f1` | `#2c3e50` | 9.57:1 | 4.5 | PASS |
| Homepage `.section-subtitle` (video-demo section) | "Watch how Caro transforms…" | `#888888` | `#0a0a0f` | 5.57:1 | 4.5 | PASS |

### 5.2 Gradient/alpha-composited (bounded range — both gradient endpoints computed individually)

| Element | Text color | Background | Ratio range | Threshold | Result |
|---|---|---|---|---|---|
| Homepage `.nav-cta` "Get Started Free" (renders on **every page**) | `#ffffff` | `linear-gradient(#ff8c42, #ff6b35)` | **2.31–2.84:1** (both stops fail) | 3 (large/UI) | **FAIL at every point along the gradient** |
| Homepage `.trust-point` checkmarks (download section) | `#ffffff` | same orange gradient | **2.31–2.84:1** | 3 | **FAIL** |
| Homepage `.copy-button` "Copy" | `#ffffff` (20%-alpha chip) | black-30%-overlay over orange gradient | ~3.4:1 (gradient-averaged estimate) | 4.5 / 3 | Fails 4.5, passes 3 (borderline) |
| Homepage `.download-section h2` "Try Caro in 30 Seconds" (36px/700) | `#2c3e50` | same orange gradient | ~4.3:1 (averaged) | 3 (large text) | PASS |

### 5.3 Semi-transparent badge pills (alpha-composited against actual page backdrop)

| Element | Text | Color | Effective composited BG | Ratio | Threshold | Result |
|---|---|---|---|---|---|---|
| Homepage `.hero-badge` "Companion Agent" (light mode) | — | `#ef3333` | `#fde7e7` (12%-tint over hero bg) | 3.41:1 | 4.5 / 3 | Fails 4.5, passes 3 (borderline) |
| Homepage `.badge.danger` "⛔ BLOCKED" | — | `#ff5555` | `#805151` (20%-tint composited) | **2.08:1** | 3 | **FAIL** |
| Homepage `.blocked-badge` "⛔ BLOCKED" (different component, same label) | — | `#ef3333` | `#d4bfba` (10%-tint composited) | **2.31:1** | 3 | **FAIL** |

**Severity:** P1 for anything failing the 3:1 floor (`.nav-cta`, `.trust-point`, `.badge.danger`,
`.blocked-badge`, hero `.subtitle`, `.attribution`, docs links) since these are either
high-visibility body copy or a sitewide nav element; P2 for narrow 4.5-threshold misses that clear
3:1 (footer tagline, `.copy-button`, `.hero-badge`).

---

## 6. Semantic color misuse (P2)

The homepage's before/after comparison uses `#ff6b35` (deprecated orange) for the **"good"**
example and `#ff5555` (off-token red) for the **"bad"/"blocked"** examples. Neither draws on the
`--status-safe` (`#3fa34d`) or `--accent` (`#ef3333`) tokens already defined in `tokens.css` for
exactly this purpose.

---

## 7. Mascot readability verdict (Task B)

**Asset:** `website/public/icons/kyaro-mark.svg` — 16×16 viewBox, `shape-rendering: crispEdges`,
monochrome (`fill="currentColor"`), no external references.

**Method:** Read the SVG source directly (reproduced verbatim below), reconstructed the 16×16
pixel grid from the `<rect>` coordinates, then rendered it live in a local test harness at 16 /
20 / 24 / 32 / 48px on both `#f4f1df` (light/ink `#2b2b2b`) and `#2b2b2b` (dark/ink `#f4f1df`)
backgrounds. Screenshot saved to
`.claude/audits/2026-07-12/before/kyaro-mark-readability-matrix.png` (full 10-swatch matrix,
both themes, all five sizes, captured via Playwright at device pixel scale).

**Geometry, row by row** (16-wide grid, y=0 top to y=13 bottom):

- `y=1`: two small isolated 2px tabs at x=2–3 and x=12–13 — separated, near the outer edges.
- `y=2`: tabs widen to 4px (x=1–4 and x=11–14) — still two separate shapes.
- `y=3–4`: **full 16px-wide solid band** — the two shapes merge into the head here.
- `y=5–6`: three blocks (x=0–3, x=6–9, x=12–15) with **two background-colored gaps** at x=4–5 and
  x=10–11 — these gaps are the eyes.
- `y=7–8`: full-width solid band again (lower face / muzzle).
- `y=9–13`: progressively narrowing bands with one more gap at y=11 (x=7–8) — chin tapering to a
  point, with a small mouth/chin notch.

**Verdict — confirmed, size-dependent ambiguity:**

- **At 16px and 20px**: the two top protrusions are rendered as sharp, angular, symmetric tabs
  positioned at the extreme outer edges of the canvas, with **no internal shading or color cue**
  to distinguish "ear" from "horn" (the icon is a flat monochrome silhouette). At this pixel
  density the two eye-gaps (only 2×2px each) are barely legible, so the dominant visual signal is
  the two-pointed top silhouette in isolation — which is a well-established pixel-art shorthand
  for devil horns at least as much as it is for ears. **This independently corroborates the
  documented "horned demon" complaint** — it is not merely inherited from history, the geometry
  itself supports the misread at these two sizes.
- **At 24px**: transitional. The eye-gaps and side "cheek" blocks (x=0–3 / x=12–15 at y=5–6) begin
  to register as supporting face detail, but the pointed top tabs remain the single most salient
  feature and the read stays borderline.
- **At 32px and 48px**: reads clearly as a friendly stylized animal (dog/cat) head. The additional
  pixel density lets the cheek/jowl mass flanking each ear-tab register as a plausible anatomical
  base for an ear (horns don't usually have jowls at their base), and the two eye-gaps become
  unambiguous.

**Conclusion:** readability crosses from ambiguous to clear somewhere **between 24px and 32px**.
The two sizes named in the brand book as requiring the pixel variant specifically because "every
pixel matters" (≤40px, and by extension the smallest UI chrome uses — favicons, inline glyphs,
16–20px icon slots) are **exactly the sizes that fail to clearly read as a friendly dog** in this
test. This is evidence-grade support for escalating a redraw (or a documented minimum-size
restriction, e.g. "never deploy kyaro-mark.svg below 24px; use a name-only wordmark or a
different, rounder glyph below that size") to Claude Design as the brand authority — not a
prescription of which fix to make, just the measured problem.

---

## Screenshot index

All paths relative to repo root
`/Users/kobik-private/workspace/caro/.claude/worktrees/strange-goldberg-536bc2/`:

| File | Surface | Theme | Viewport |
|---|---|---|---|
| `.claude/audits/2026-07-12/before/home-dark-desktop.png` | Homepage | dark (default load) | 1280×desktop, full page |
| `.claude/audits/2026-07-12/before/home-light-desktop.png` | Homepage | light (toggled) | 1280×desktop, full page |
| `.claude/audits/2026-07-12/before/home-light-mobile.png` | Homepage | light | 375×mobile, full page |
| `.claude/audits/2026-07-12/before/use-cases-desktop.png` | /use-cases | light | 1280×desktop, full page |
| `.claude/audits/2026-07-12/before/use-cases-developer-desktop.png` | /use-cases/developer | light | 1280×desktop, full page |
| `.claude/audits/2026-07-12/before/ai-command-safety-desktop.png` | /ai-command-safety | light | 1280×desktop, full page |
| `.claude/audits/2026-07-12/before/safe-shell-commands-light-desktop.png` | /safe-shell-commands | light | 1280×desktop, full page |
| `.claude/audits/2026-07-12/before/safe-shell-commands-dark-desktop.png` | /safe-shell-commands | dark (toggled) | 1280×desktop, full page |
| `.claude/audits/2026-07-12/before/docs-light-desktop.png` | /docs | light | 1280×desktop, full page |
| `.claude/audits/2026-07-12/before/kyaro-mark-readability-matrix.png` | kyaro-mark.svg test harness | both | 5 sizes × 2 bg, device-pixel scale |

Not captured (scope trade-off, noted for the record): homepage dark-mobile, and dark-mode passes
for /use-cases, /use-cases/developer, /ai-command-safety (only /safe-shell-commands got the
dark-mode landing-page sample; the orange/red-fragmentation finding was already saturated by the
four light-mode landing-page sweeps and did not need a second theme's worth of confirmation to be
solid).

---

## Out-of-scope observations (not scored, noted for completeness)

- A temporary "🎊 Ring in 2026" New Year countdown banner and a separate "Humanity Event" banner
  are live on the homepage, using their own gold/teal/orange gradient palette unrelated to
  `tokens.css`. Almost certainly seasonal/time-boxed marketing content, not part of the core
  design-system migration — flagged only so it isn't mistaken for a brand-token regression if it's
  still present in the AFTER audit.
