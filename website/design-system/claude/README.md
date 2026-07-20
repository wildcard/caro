# Caro Design System

> **Caro** (formerly **cmdai**) is a friendly terminal companion that turns the intimidating command line into an approachable, guided experience. It converts natural-language prompts into safe POSIX shell commands using local LLMs, and ships with **Kyaro**, a pixel-art Shiba Inu mascot. The product is built in Rust, distributed on crates.io, and lives at [caro.sh](https://caro.sh).

This folder is the brand + UI design system for Caro: fonts, colors, tokens, logos, Kyaro sprite animations, and UI kits for the CLI and the marketing website.

---

## Sources used

- **Brand Guideline PDF** — `uploads/Caro_Brand Guideline.pdf` (49 pages, by Alrezky Caesaria / Morning Moon Studios, Bandung, Indonesia — `alrezkycaesaria@gmail.com`). Source of truth for palette, type pairing, brandmark rules, and stationery specs.
- **Brandmark Presentation PDF** — `uploads/CARO_Brandmark Presentation.pdf` (not parsed in full; mirrored in guideline).
- **cmdai Concept & Analysis PDF** — `uploads/cmdai_Concept and Analysis.pdf` (product strategy).
- **Figma — Business Card** — mounted VFS at `/Page-1/Business-Card_Business-Card---{Front,Back}-1/`. Used for exact stationery dims + layout.
- **Social Media Kit** — `uploads/Social Media Kit_*.svg` (blog banners, Discord, GitHub, LinkedIn, Open Graph, X banners). Copied to `assets/social/`.
- **Logo screenshots** — `uploads/Screenshot 2026-01-05 at 11.54.54.png` (smooth + pixel marks) and `11.56.41.png` (horizontal lockups). Clean crops are in `assets/`.
- **Fonts** — Figtree TTFs uploaded directly; copied to `fonts/`.
- **GitHub — wildcard/caro@main** — `https://github.com/wildcard/caro`. README and `website/index.html` read directly. Kyaro sprite animations (`assets/kyaro/001-idle` … `009-upside-down`) imported verbatim — **209 files** of PNG frames + GIFs + ASCII art.

### Sources requested but not provided / accessible

- `uploads/Figtree.zip`, `uploads/Logo Pack.zip`, `uploads/Logo Pack_UPDATED.zip`, `uploads/Social Media Pack.zip` — listed in the brief but **not present in the filesystem**. The individual TTFs and SVGs were provided alongside, so no functional loss.
- **Azeret Mono** font files — not provided. Loaded from Google Fonts; flagged below.
- **"CARO - Social Media Templates.fig"** and **"Caro - Newsletter Template.fig"** — listed as mounted Figma files, but only the **Business Card** file is actually present in the VFS. Individual social-media SVGs cover the same ground, so the templates are represented — but the newsletter design is **not** reproducible without that file.

---

## What's at the root

| Path | Purpose |
|---|---|
| `README.md` | This file — brand context, fundamentals, and manifest. |
| `SKILL.md` | Agent-skill entrypoint. Drop this folder into a Claude skill directory and it's usable. |
| `colors_and_type.css` | The canonical CSS variable + semantic-class definitions. Import this first. |
| `fonts/` | Figtree .ttf family (Light → Black + italics). |
| `assets/` | Logos, logo crops, Kyaro sprite animations, and social-media SVG kit. |
| `assets/kyaro/` | **209 files** — 9 animation sets (idle, blink, sleeping, prompt-bubble, walking, happy-bounce, pooping, shocked, upside-down). Each has `{name}_animation/` PNG frames + a GIF, and `{name}_ASCII/` text-mode frames. **Licensed separately — see `assets/kyaro/` terms before reuse.** |
| `assets/social/` | 14 SVGs: blog-post banners, Discord profile + server, GitHub, LinkedIn, Open Graph, X. Ready to pull into designs. |
| `preview/` | Self-contained HTML cards registered in the Design System tab. |
| `ui_kits/website/` | High-fidelity recreation of caro.sh marketing site — hero, terminal demo, feature grid, blog grid, download CTA. |
| `ui_kits/cli/` | Recreation of the **terminal UX** — prompt, generated-command review, risk badges, Kyaro idle/shocked/happy states inline. |
| `slides/` | Not built. No slide template was provided in the brief; skipping per instructions. |

---

## Brand in one sentence

**Caro is your loyal shell companion** — a pixel-nostalgic, paper-and-ink aesthetic wrapped around a modern Rust CLI. Warmly precise. Collaboratively empowering. Never intimidating.

---

## CONTENT FUNDAMENTALS

### Voice

Caro speaks as a **companion, not a tool**. The brand guideline's own words: *"warmly precise and collaboratively empowering."* She (Caro is a she — the digitalization of Kyaro, the maintainer's Shiba Inu) is by your side, not above you.

- **Pronouns.** Site + blog use "she/her" for Caro, and "you" for the reader. Never "we/our" for the product — that's reserved for the Caro team, the open-source "pack."
- **Tense / mood.** Present, active, imperative when helping: *"Generated command:"*, *"Execute this command?"*, *"Safe to run on your macOS system."*
- **Plain tech talk, no marketing bloat.** Feature copy is concrete: *"Sub-100ms startup, sub-2s inference on Apple Silicon."* Avoid filler superlatives like "blazing-fast", "powerful", "seamlessly", "next-gen" — they undercut the brand's precision. (Find-and-replace these in any inherited copy.)
- **Empathy lines.** Scattered through the product: *"turning confusing or complex tasks into simple, and even delightful, steps."* These are deliberate — they're the brand's heart.

### Casing

- **Product name: `Caro`**, not `CARO` or `caro` in prose. The `caro` CLI binary is lowercase (always as `<code>`).
- **Eyebrow labels (metadata, section titles, legal): `ALL CAPS` with WIDE tracking.** 200‰ (`--track-wider`) for 14px+; 300‰ (`--track-widest`) for 12px. This is the single most distinctive type move in the whole system — see business-card `FOUNDER` and `CARO.SH`.
- **Headings:** Sentence case, not Title Case. *"Your loyal shell companion"*, not *"Your Loyal Shell Companion"*.
- **Terminal examples:** real shell, real prompt: `$ caro "list all PDF files..."` — never pseudocode.

### Emoji

Sparingly, and **only a small set in use on the current site/README:**

- 🐕 — Caro herself. Used as a companion badge / prompt prefix: `🐕 Caro:`.
- 🛡️ 🧠 🌍 ⚡ ✅ 🎯 🚀 🤝 🔌 ✨ — feature-grid icons on the website.
- 🎉 — announcement banner.
- 🦀 — Rust signal (footer only).
- 📝 — blog eyebrow.

**Pixel-icon versions are preferred over emoji** whenever the design has room. Emoji are a fallback for text-only contexts (banner, README).

### Sample copy (lifted from shipping product)

> *"Caro is a friendly terminal companion that transforms the intimidating command line into an approachable, guided experience. It's like having a helpful guide by your side, turning confusing or complex tasks into simple, and even delightful, steps."*

> *"A specialized POSIX shell command agent with empathy and agency. She keeps you safe while helping Claude get the job done."*

> *"In memory of Kyaro (Kyarorain Kadosh) — Inspired by Portal's Caroline, loyalty transformed into eternal companionship."*

The Portal/Caroline reference is real — it's the name's origin and appears on the site footer. Lean into it for hero-level storytelling; avoid it in utility copy.

---

## VISUAL FOUNDATIONS

### Colors

Paper-and-ink, not web-chrome. The brand is fundamentally two colors — a **retro-console grey** and a **retro-console beige** — with **signal red** and **highlighter yellow** as secondary accents, and **black** as a tertiary.

| Role | Name | Hex | Use |
|---|---|---|---|
| Primary | Retro console grey | `#4f4f4f` | Body text, brandmark ink, dark surfaces |
| Primary | Retro console beige | `#f4f1df` | Page background, "paper" |
| Secondary | Signal red | `#ef3333` | Accents, CTAs, command-risk danger |
| Secondary | Highlighter yellow | `#fcfc62` | Terminal prompt, selection highlights |
| Tertiary | Black | `#000000` | Pixel-art mascot, mass-fill supergraphic |

No gradients in the brand guideline. The shipping website **does** use an orange gradient (`#ff8c42 → #ff6b35`) — that predates the finalized identity and is deprecated. **Don't use it.** When this design system is applied to the website, replace the orange gradient with flat red `#ef3333`.

### Type

**Two-family system**, both from Google Fonts originally:

- **Azeret Mono** (display / titles) — geometric monospace with OCR-font personality. Weights 400/600/700/800. Used for H1–H4, terminal text, wordmark-adjacent UI.
- **Figtree** (body + UI chrome) — clean geometric sans. Full 100–900 weight axis shipped. Used for paragraphs, buttons, eyebrow labels, navigation.

The two are mixed **intentionally and often within a single composition** — e.g. an Azeret H1 followed by a Figtree subtitle. Don't default to one family everywhere.

Signature moves:
- 12–14px **bold Figtree** labels in **uppercase, 200–300‰ tracking, grey `#a0a0a0`**. The business card's `FOUNDER` and `CARO.SH` both.
- 32px **Azeret Mono ExtraBold** for the flagship number on the front of stationery.
- Body copy is comfortably large (16–18px) and loose-leading (1.55+) — generous, never cramped.

### Section color ratio

**Sections default to PAPER. Dark surfaces are the exception, not the rule.**

The brand book says 95% paper / 5% stationery dark. Translate that to layout:

- **Paper (`--bg`)** — hero, story, features, blog grid, personas, testimonials, pricing, waitlist, every long-form section.
- **Dark (`--bg-inverse`)** — used sparingly. The shipping site reserves it for: the **terminal demo**, the **install / download CTA**, and the **footer**. That's the whole list. Adding a fourth dark section is a brand decision, not an implementer decision.
- **Stage grey (`--bg-stage`)** — use for marketing surfaces meant to feel like "the back of a business card" (testimonial walls, founder bios). Not a default.
- **Tertiary (`--bg-tertiary` = `#faf8ec`)** — third surface for code blocks and accent panels that still need to read as paper. Never use warm-amber `#fff8f0` — it reads as the deprecated orange identity.

A good page-section pattern, top-to-bottom: `paper → paper → dark (terminal demo) → paper → paper → paper → dark (download CTA) → dark (footer)`.

### Backgrounds

- **Paper:** solid `--caro-beige-100` (`#f4f1df`) is the default page fill. 95% of compositions sit on this.
- **Terminal:** solid `--caro-grey-700` (`#4f4f4f`) inverse panels. Never pure black — always the brand grey.
- **Stage / card-back:** solid `--caro-grey-200` (`#d1cfc9`), like the reverse of the business card.
- **No gradients. No photographic backgrounds.** Brand guideline explicitly shows flat color.
- **Supergraphic elements** (large pixelated `>_`, window-chrome containers, oversized pixel letters) act as optional background furniture on marketing surfaces. See the social media SVG kit for examples — they're how a "hero image" is constructed in this system.
- **No noise, grain, or texture.** The brand reads clean and digital.

### Animation

- Snappy and stepped. Use `steps(N, end)` for sprite playback (Kyaro's 4-frame idle loops at ~6 fps).
- UI transitions are **fast + curvy**: 120–220 ms, ease `cubic-bezier(.2,.8,.2,1)`.
- **No bouncy spring animations.** No long fades. The pixel aesthetic rewards crispness.
- Hover effects: 2 px translate-y or a color shift. Press: scale down 0.98, immediate.
- Kyaro herself is the system's primary animated element — she idles, blinks, walks, bounces, or reacts (shocked) inline in product UI. Use the provided sprite sheets; don't redraw her.

### Hover / press states

- **Buttons (primary, red):** hover darkens to `#e63636`; press shifts to `#c02020` and nudges down 1 px.
- **Buttons (secondary, outline):** hover fills beige `#f4f1df`; press fills darker `#e9e4c8`.
- **Links:** hover adds a 2 px red underline, offset 3 px from the baseline.
- **Cards:** hover lifts 4 px with a subtle `var(--shadow-3)` and the border turns `--accent`. Never scale.

### Borders

1 px hairlines, `--border` (`#c9c7c1`). For pixel-themed chrome, use **2 px solid `--caro-grey-700`** — the "window-like illustration container" from the brand guideline.

### Shadows

Two systems:

1. **Soft paper drop** — `--shadow-1/2/3` for cards, modals, lifted elements.
2. **Pixel shadow** — `--shadow-pixel: 4px 4px 0 var(--caro-grey-900)` for chunky pixel buttons / retro panels. Use on at most one element per view; it's loud.

**Card hover lift is canonical** — use `var(--shadow-card-hover)`, do not invent rgba shadows. Never tint shadows with brand red or yellow; shadows are always neutral ink. Focus rings use `var(--focus-ring)` (3 px brand-red glow).

### Semantic feedback colors

Don't conflate `--status-*` (CLI command-risk levels) with general UI feedback. Use:

- **`--info` / `--info-tint`** — informational toasts, doc callouts.
- **`--success` / `--success-tint`** — form success, build-passing badges. Distinct from `--status-safe` (which means "this command is safe to execute").
- **`--warning` / `--warning-tint`** — form warnings. Amber, not the highlighter yellow.
- **`--error` / `--error-tint`** — form errors. Brand red.

### Charts / data viz

- **Categorical:** `--chart-1` … `--chart-8` in declared order. The first 5 are pulled from the brand swatch + close cousins; 7–8 step outside (clay, teal-grey) only because eight categorical hues require it.
- **Sequential:** `--chart-seq-0` … `--chart-seq-5` — monochrome ramp from paper to ink. Use this for heatmaps and density.
- **Diverging:** not specified yet. If you need one, run it through brand red → paper → brand grey, and flag for v3.

### Per-distro accents (CLI / website)

The website themes per OS using `--distro-accent`. The contract:

- `--distro-accent` overrides ONLY decorative chrome: per-OS download tile border, OS pill, OS-specific install-command icon.
- It MUST NOT replace `--accent` for primary CTAs, `--status-*` colors, or the brandmark. **Brand red stays brand red across every distro.**
- Default: `var(--accent)`.

### Holiday theme overlays

The website ships seasonal overlays (Christmas, Hanukkah, Diwali, New Year, Pride). Recipe:

1. Layer is **additive only** — a `<div data-holiday="…">` overlay attached to the announcement banner or hero, never the body.
2. Holiday palettes may introduce 1–2 new hues (e.g. evergreen, menorah blue, rangoli marigold) **alongside** brand red, never replacing it.
3. Kyaro gets a holiday accessory sprite. Never recolor Kyaro herself.
4. Auto-disable on `prefers-reduced-motion` and on the install / download CTA section.

### Storybook chrome

Storybook's `theme.ts` should reference these tokens (not raw hex):

```ts
import { create } from '@storybook/theming';
export default create({
  base: 'light',
  brandTitle: 'Caro',
  brandUrl: 'https://caro.sh',
  colorPrimary: '#ef3333',     // --accent
  colorSecondary: '#4f4f4f',   // --caro-grey-700
  appBg: '#f4f1df',            // --bg
  appContentBg: '#ffffff',     // --bg-raised
  textColor: '#4f4f4f',        // --fg
  fontBase: '"Figtree", system-ui, sans-serif',
  fontCode: '"Azeret Mono", ui-monospace, monospace',
});
```

### CLI spinner / indicatif

The CLI uses [`indicatif`](https://docs.rs/indicatif). When templating with brand colors, **indicatif accepts truecolor only as `#RRGGBB`**, not `color(r,g,b)`. The brand-yellow tick should be:

```rust
.template("{spinner:.bright.green} {wide_msg}")
// for brand colors, use truecolor explicitly:
.template("{spinner:.#fcfc62} {wide_msg}")
```

### Transparency + blur

Used **rarely**. The terminal demo on caro.sh puts copy buttons on `rgba(255,255,255,.2)` over the accent fill. No blur backgrounds, no glass morphism. If a panel needs visual separation, it should be a solid color.

### Corner radii

- `0–2 px` for pixel-furniture (terminal chrome, window titlebars).
- `4 px` for inputs and buttons.
- `8 px` for cards and the terminal window.
- `999px` for eyebrow pill badges (feature-grid `COMPANION AGENT` style).
- **Never fully rounded buttons.** The brand stays square-ish.

### Cards

Default card: `--bg-raised` fill, 1 px `--border`, `--radius-lg` (8 px), `--shadow-1` at rest. No left-accent-only borders. Hover behavior described above. When used on beige paper, cards are white; when used on grey stages, cards are beige.

### Layout rules

- **Generous whitespace.** Hero sections are tall (min-100vh), features use `minmax(280px, 1fr)` grids with 40 px gaps.
- **Max content width 900–1040 px.** The brand was laid out for readable long-form and doesn't use ultra-wide hero layouts.
- **Asymmetric framing on brand surfaces** — business card has the mascot clipped bleeding off the right edge. Apply the same trick on social-media covers (the SVG kit demonstrates it).
- **Fixed elements:** sticky top banner (announcement) + sticky nav below it. Don't stick anything else.

---

## ICONOGRAPHY

The brand guideline names four "basic icons" — **Home, Location, Teams, Tools** — drawn as **pixel + line-art** black-on-beige. They're referenced but the source artwork isn't in the uploaded pack.

Today's website uses emoji as feature-grid "icons." The **correct long-term move** per the brand is: pixel-art glyphs in the same family as Kyaro, rendered at 48 px minimum, 1-bit black-on-beige or white-on-grey.

**Substitution I'm using until real assets arrive:** [**Lucide**](https://lucide.dev) (CDN). Closest in spirit — crisp geometric stroke, open-source. 2 px stroke. Use at 24 px for UI, 40 px for feature tiles. **Flagged to the user.**

Alternatives to consider:
- [Pixelarticons](https://pixelarticons.com) — 1-bit pixel set, **closer to brand aesthetic** but less breadth. Would be my recommendation once the product needs more icons.
- Emoji — fallback in text-only contexts (READMEs, chat).

**No custom SVG iconography has been drawn by hand in this system.** All pixel art comes from the Kyaro sprite packs.

### Logo assets on hand

| File | Use |
|---|---|
| `assets/logo-caro-horizontal-clean.png` | Smooth brandmark + horizontal `CARO` wordmark. Primary lockup. |
| `assets/logo-caro-horizontal-pixel-clean.png` | Pixel brandmark + horizontal wordmark. Secondary lockup. |
| `assets/mark-caro-smooth.png` | Standalone symbol (smooth). |
| `assets/mark-caro-pixel.png` | Standalone symbol (pixel). |
| `assets/kyaro/**` | Full 209-asset sprite library with 9 animation states. Behaves as our "stock illustration" system. |

---

## Caveats + open questions

- **Azeret Mono is loaded from Google Fonts, not bundled.** Please provide the TTFs if you want true offline use.
- **No real icon system yet** — using Lucide as a placeholder. Flag if you have pixel-icon SVGs.
- **Newsletter Figma file and Social Media Templates Figma file** were advertised as mounted but only the Business Card file is accessible. If there's a way to reattach them, we can mirror those layouts too.
- The logo crops in `assets/` are derived from the brandmark-presentation screenshot. **The real vector `.ai`/`.eps` files from the Logo Pack zips would be better** and should replace these.

---

## v2 changelog (April 2026)

Updates from shipping the system across caro.sh + the Rust binary (PR #993). Each item closes a gap surfaced by the implementer.

- **Dark mode token layer added.** `<html class="dark">` or `data-theme="dark"` now flips every semantic token. Accent lifts to `--accent-light` (`#f56565`) for AA contrast on ink. See `preview/color-dark-mode.html`.
- **Semantic feedback colors added** (`--info`, `--success`, `--warning`, `--error` + `*-tint`). Distinct from `--status-*`, which is reserved for CLI command-risk levels only.
- **`--bg-tertiary`** (`#faf8ec`) canonized as the third paper surface for code blocks and accent panels. Do **not** use warm-amber `#fff8f0` — it reads as the deprecated orange identity.
- **`--accent-tint` / `--accent-light`** added for card-hover surfaces and dark-mode contrast.
- **`--shadow-card-hover`** is now the canonical card hover lift. Stop inventing rgba shadows in components.
- **`--focus-ring`** standardized at 3 px brand-red glow.
- **Chart palette** added (`--chart-1…8` categorical, `--chart-seq-0…5` sequential).
- **`--distro-accent`** contract documented: decorative chrome only; brand red holds for primary CTAs across every distro.
- **`--link-visited`** added (desaturated plum) to satisfy WCAG 1.4.1.
- **Section color ratio rule** added — sections default to paper. Dark surfaces are reserved for terminal demo / install CTA / footer.
- **Holiday overlays, Storybook chrome, indicatif spinner syntax** documented.
- **Voice:** "blazing-fast" and similar superlatives are now explicitly disallowed in feature copy.

## v2.1 — Icon system reframed (May 18, 2026)

**Caro's icon system is the mascot, not a parallel pixel-icon family.** An earlier v2.1 release shipped 8 hand-drawn pixel SVGs (home, location, teams, tools, rocket, shield, lightning, brain). They were the wrong instinct — Caro already has a rigorously hand-pixeled icon system in the form of Kyaro's 9 animation states, each of which carries a UI semantic role. The v2.1 SVGs are removed; manifest version is now `0.2.0`.

### The Kyaro icon system

| State | UI role |
|---|---|
| **Ready** (idle) | default · safe · ready to take a prompt |
| **Thinking** (blink) | short loading |
| **Paused** (sleeping) | long idle · session asleep |
| **Asking** (prompt bubble) | awaiting user input |
| **Running** (walking) | command in progress |
| **Success** (happy bounce) | completed |
| **Cleanup** (pooping) | cleanup · freed disk · "took care of it." Yes, this is what the maintainer drew. Use it earnestly. |
| **Danger** (shocked) | high-risk command · pause-and-confirm |
| **Error** (upside down) | error · unexpected state |

These are the icons. At decision-point sizes (32–96 px), use the **GIF**. For static contexts (PDFs, print), grab a single frame from the matching `*_animation/animation_*.png` set in `assets/kyaro/`.

### Terminal glyphs — for everything else

Where no mascot state fits (the prompt prefix, a risk dot, an expandable affordance), use real shell characters from the brand's own typographic vocabulary:

| Glyph | Use |
|---|---|
| `>_` | Brand wordmark glyph. Sparingly — once per page max. |
| `❯` | Caro prompt prefix (U+276F). Used as `caro ❯` in CLI. |
| `$` | Shell prompt. Real shell snippets only — not decorative. |
| `■` | Risk indicator (U+25A0). Pair with `--status-*`. |
| `▸` / `▾` | Expandable affordance. |
| `·` | Eyebrow + meta separator. |
| `↵` | Submit / return. |
| `~` | Home directory shorthand. |
| `…` | Thinking / truncated. |

### Retained

- **`assets/icons/kyaro-mark.svg`** (v2.1.1 redraw). Monochrome Shiba glyph for the footer brand mark and inline runs where a full GIF would be too heavy.

### Hard rules

- **Don't import Lucide.** If you can't find a Kyaro state or terminal glyph that fits, you're describing a new emotional UI state for Kyaro — open a v3 ticket and we'll commission a 10th sprite from the brand's illustrator.
- **Don't draw new pixel iconography by hand.** That's exactly what produced the v2.1 misstep. The brand has one pixel artist (Alrezky Caesaria, Morning Moon Studios); pixel additions go through them.
- **Don't recolor Kyaro.** Holiday accessories can layer *on top of* her sprites; her body palette is fixed.

### Deferred to v3

## v2.1.2 — Token reconciliation + composition rulings (May 9, 2026)

Closes drift surfaced by PRs #1058–1060. Brand book retreats where the implementation is more idiomatic; pushes back where authorial intent matters.

### Shadow + radius scale (brand book retreats)

The brand book previously specified `--shadow-1/2/3` and a 3-step radius ramp. Production `tokens.css` had already shipped with t-shirt sizing, consumed by ~80 components. **System updates, not the codebase.**

- **Shadows:** use `--shadow-sm` / `--shadow-md` / `--shadow-lg` / `--shadow-xl`. The numeric `--shadow-1/2/3` and `--shadow-pixel` names are deprecated in the brand book; references should read t-shirt-size.
- **Radii:** 5-step ramp (`xs/sm/md/lg/xl`) where `--radius-md` = 8 px, `--radius-lg` = 12 px. Cards use `--radius-md`, not `--radius-lg`.
- **Card spec (canonical):** `box-shadow: var(--shadow-sm)` rest → `var(--shadow-lg)` hover; `border-radius: var(--radius-md)`; `transform: translateY(-4px)` on hover, **never scale**.
- `--shadow-card-hover` is retained as a semantic intent token; it maps to `--shadow-lg`.

### Section composition — "earn the ink"

The 95% paper / 5% stationery rule is not "force everything to paper." It's "earn the ink." Decision-moment and commitment-CTA surfaces qualify; empathy and storytelling surfaces don't.

| Surface | Background token | Why |
|---|---|---|
| LPMoments | `--bg-inverse` | Decision-moment surface, conceptually adjacent to terminal demo + inline-Kyaro pause. |
| LPWaitlist | `--bg-inverse` | Conversion CTA asking for commitment — earns visual weight. |
| LPTestimonials | `--bg-stage` | "Back of the business card." Stage grey was canonized for testimonial walls / founder bios — use it. |
| LPPersonas | `--bg` | Empathy work, not authority work. Paper carries "warmly precise"; ink overstates. |

Generalize: **decision + commitment ⇒ ink. Empathy + storytelling ⇒ paper. Authority over a list of people or quotes ⇒ stage.**

### Never hardcode near-black

Hardcoded `#111116` / `#12121a` / `#16161f` and the like are forbidden — they read cold, and three different blacks across one page break the "ink is one surface" reading. **All dark surfaces use `var(--bg-inverse)` (`#4f4f4f`).** Brand guideline: never pure black, always brand grey. Lint rule recommended; PR #1059 swept the existing offenders.

### Hero CTA: flat brand red is canonical

Earlier wording read the gradient option as permissive — that was loose. **Flat `var(--accent)` is canonical for the Hero CTA across all surfaces.** No gradient, no orange-era holdover, no per-distro recoloring of the primary CTA. The "no gradients" rule from the visual-foundations section applies to CTAs without exception.

## v2.2 — React component library + landing template (June 2026)

The website's Storybook components are now first-class, **compiled** design-system components — not just static preview cards. They live under `components/<Name>/` and are exposed on the bundle namespace for consuming projects.

### What shipped

- **11 React components ported** from `website/src/ui/*` into `components/`, each as a `<Name>.tsx` + `<Name>.d.ts` + an `@dsCard` variant-matrix preview (light + dark where relevant): **Button, Badge, Card, Link, IconButton, Toggle, Terminal** (+ `TerminalLine`), **CodeBlock, CopyCodeBlock, Dropdown** (compound: `DropdownTrigger/Panel/Item/Divider/Header`), and **DistroSelector**.
- **Root `styles.css`** added — the compiler entry point. It `@import`s `colors_and_type.css` (brand tokens + fonts) then a per-component CSS file, and defines a **bridge layer** that maps the legacy website token names (`--color-primary`, `--font-family-base`, `--space-sm`, `--control-height-*`, t-shirt shadows…) onto brand tokens, so there is one source of truth and no orange-era drift.
- **Landing-page template** at `templates/landing/` — nav, hero, animated terminal demo, feature grid, two "earn the ink" dark sections, install CTA, footer — composed entirely from the component library. Consumers copy the folder and re-point the `base` line in `ds-base.js`.

### Brand corrections applied on the way in

The shipped website CSS still carried orange-era artifacts; the ported components are corrected to the finalized identity:

- **Button primary is FLAT signal red** (`--accent`), not the `red→dark-red` gradient. Button/input radius is `--radius-md` (4px) per the brand's square-ish rule.
- **`IconButton`/`Toggle` "brand" + hover states** use brand red + `--accent-tint` instead of `rgba(255,140,66,…)` orange.
- **`Terminal`/`CopyCodeBlock` surfaces** use brand grey (`--terminal-bg`/`--terminal-header-bg`), never pure black; the Caro prefix is brand red and the prompt is highlighter yellow. The `CopyCodeBlock` `brand` variant is a flat signal-red install tile.
- **`Card` hover** lifts with `--shadow-lg` and a brand-red border (no scale); status pills use the brand semantic-feedback tints.
- **Sponsor "support" pink** is retained as the one deliberate non-paper accent (`--color-support`), flat — reserved for GitHub-Sponsors-style CTAs only.

### How a consuming project uses them

```html
<!-- React UMD BEFORE the bundle (the bundle calls React.createElement) -->
<script src="…/react.development.js"></script>
<script src="…/react-dom.development.js"></script>
<link rel="stylesheet" href="_ds/caro/styles.css" />
<script src="_ds/caro/_ds_bundle.js"></script>
<script>
  const { Button, Card, Terminal } = window.CaroDesignSystem_332f73;
  // …render with ReactDOM
</script>
```

`DistroSelector` was made **self-contained** on port — the original depended on 5 config/lib modules (`distro-types`, `distros`, `distro-preferences`, `user-agent-detector`, PostHog). The DS version bundles a curated OS/distro/shell dataset inline and keeps selection state local (`detectedOS` prop + `onChange`), so it drops into any page. Re-wire it to the live preference store when used inside the website itself.

### Not yet brought over

- **Astro section libraries** (`website/src/components/landing|gtm|explore/*`) and the global Astro chrome (`Nav`, `Footer`, `Hero`…) are **not** ported — only the React/Storybook `ui/` set and one landing template. The Astro sections are largely compositions of these same primitives; port the ones you need into new `templates/` as the marketing site grows.
- `Glyph.astro` (the only non-React `ui/` item) is left in the website; its job is covered by the brand's terminal-glyph vocabulary in the icon section above.

## v3 backlog

- **Diverging chart palette.** Spec when needed; baseline = brand red → paper → brand grey.
- **Expanded icon set.** `git`, `terminal`, `package`, `model`, `gear`, `doc`, `search`, `spinner`, `check`, `cross`.
- **Azeret Mono offline TTFs.** The system still loads from Google Fonts. The "no data collection" copy on the install CTA is a contradiction until this is bundled.
- **Newsletter Figma + Logo Pack vectors.** Re-attach when available.
