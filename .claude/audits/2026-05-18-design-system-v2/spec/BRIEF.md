# Caro Design System — Brief (verbatim from claude.ai/design project)
# Captured: 2026-05-18
# Source: https://claude.ai/design/p/332f73e9-84c2-4078-ac8d-169c1855385d

## Conversation Summary

### Token Reconciliation (caro-5wo) — RESOLVED

Claude Design ruled: **Option (b) — adopt implementation naming, not brand book naming.**

- `tokens.css` stays as-is with `--shadow-sm/md/lg/xl`
- Brand book updated to use `--shadow-sm/md/lg/xl`
- Card spec: `--shadow-sm` at rest, `--shadow-lg` on hover, `--radius-md` (8 px)
- `--shadow-card-hover` maps to `--shadow-lg` semantically — keep as intent token
- Radius: `--radius-md` = 8 px for cards (not `--radius-lg`)

### Section Composition Rulings (caro-l06) — RESOLVED

Per-surface rulings on dark-vs-paper for dense card grids:

| Surface | Verdict | Token |
|---|---|---|
| LPMoments | Keep dark | `--bg-inverse` |
| LPTestimonials | Stage grey | `--bg-stage` (#d1cfc9) |
| LPPersonas | Paper | `--bg` |
| LPWaitlist | Keep dark | `--bg-inverse` |

**Critical correction:** hardcoded `#111116` / `#12121a` / `#16161f` → `var(--bg-inverse)` (#4f4f4f brand grey). Never pure black.

### Hero CTA — RESOLVED

- **Flat `var(--accent)` is canonical** — no gradient on the Hero CTA
- "No gradients" rule applies to CTAs without exception
- Flat red across every distro keeps brandmark and primary action visually identical

### Token Wiring Bug (caro-ci1) — SHIPPED via PR #1060

- `tokens.css` was never imported by `Layout.astro`
- Fix: `import '../ui/tokens.css'` in layout frontmatter
- Without this, all brand tokens silently failed to resolve at runtime

### Icon System Reframe (v2.1 correction) — RESOLVED

**Kyaro states ARE the icon system.** The 8 hand-drawn SVGs (home/location/teams/tools/rocket/shield/lightning/brain) were deleted as incorrect.

Hard rules:
1. Don't import Lucide
2. Don't hand-draw new pixel iconography (goes through Alrezky / Morning Moon Studios)
3. Don't recolor Kyaro

Semantic map:
| Kyaro state | UI role |
|---|---|
| Ready (idle) | default · safe |
| Thinking (blink) | short loading |
| Paused (sleeping) | long idle |
| Asking (prompt bubble) | awaiting input |
| Running (walking) | in progress |
| Success (happy bounce) | completed |
| Cleanup (pooping) | freed disk · "took care of it" |
| Danger (shocked) | high risk · pause-and-confirm |
| Error (upside down) | unexpected state |

Terminal glyphs for surfaces where mascot doesn't fit: `>_`, `❯`, `$`, `■`, `▸`

`kyaro-mark.svg` kept — legitimate glyph for inline runs + footer where GIF would be too heavy.

## Design Files in the System

- UI Kit — CLI (Interactive terminal demo)
- UI Kit — Website (caro.sh marketing recreation)
- Type · Pair (Azeret Mono display + Figtree body)
- Type · Scale (H1–H3, lead, body, eyebrow)
- Type · Wide-track labels (200–300‰ caps)
- Colors · Primary (Grey, beige, red, yellow, black, stage)
- Colors · Semantic tokens
- Colors · Dark mode (Light vs ink — same tokens)
- Colors · Grey + beige scale (10-step neutral ramp)
- Colors · Command risk levels (Safe · Moderate · High · Critical)
- Colors · Chart palette (Categorical 8 + sequential ramp)
- Colors · Semantic feedback (Info · Success · Warning · Error)
- Spacing · Scale (4px base ramp)
- Spacing · Radii & shadows (Radius + soft & pixel shadows)
- Components · Cards (Paper, inverse, pixel card variants)
- Components · Buttons (Primary, secondary, CTA, pixel, ghost)
- Components · Badges (Risk pills, brand badges, chips)
- Components · Form inputs (Input, select, prompt)
- Components · Terminal (Window chrome + caro prompt)
- Brand · Mark (Smooth & pixel primary marks)
- Brand · Horizontal lockup (Smooth and pixel horizontal wordmarks)
- Brand · Kyaro mascot (5 of 9 sprite animations)
- Brand · Pixel icons (v2.1 — REFRAMED: Kyaro states are the icon system)
- Brand · Social kit (Open Graph, X, GitHub, LinkedIn)

## PRs Shipped (context from conversation)

- **#1057**: kyaro-mark v2.1.1 with cream-cheek mask → production
- **#1058 (caro-pt1)**: light-bg sweep — Card.tsx, LPDifferentiators, LPHero, LPCommunityVoices, LPScenarios, LPBestPractices
- **#1059 (caro-l06)**: per-surface dark/stage/paper ruling for 4 dense card grids; hardcoded near-blacks swept to `var(--bg-inverse)`
- **#1060 (caro-ci1)**: `tokens.css` import wiring fix in `Layout.astro`
