---
name: caro-demo-video
description: Build, render, and ship the caro landing-page demo video using Remotion (React-based programmatic video). Use when creating, updating, re-rendering, or extending the project demo MP4 that lives in website/public/caro-demo.mp4 and feeds the LPVideoDemo slot on the English landing page.
---

# Caro Demo Video

Reproducible, on-brand product demo video for the caro website, rendered
from React code via [Remotion](https://www.remotion.dev/).

## When to Use

- "create the project demo video"
- "render a new caro demo"
- "update the landing page video"
- "add a new scene to the demo"
- "the demo on the website is missing / outdated"

## What This Skill Owns

The Remotion project at `demos/remotion-video/` and its rendered output at
`website/public/caro-demo.mp4` (+ `caro-demo-poster.png`). Wired into
`website/src/components/landing/LPVideoDemo.astro` via `videoUrl` /
`posterUrl` props passed from `website/src/pages/index.astro`.

## Companion Skill

This skill is operational. For Remotion mechanics (animation curves,
timing, fonts, transitions, audio) load the global
`remotion-best-practices` skill at `~/.claude/skills/remotion-best-practices/`.
Specifically `rules/compositions.md`, `rules/timing.md`,
`rules/transitions.md`, and `rules/fonts.md` cover everything you'll need.

## Style Rules

These are caro-specific and not covered by `remotion-best-practices`.

### Visual tokens (mirror of `website/src/ui/tokens.css`)

| Token | Hex | Use |
|---|---|---|
| `bgDeep` | `#0a0a0f` | Composition background |
| `bgTerminal` | `#1a1a2e` | Terminal window body |
| `bgChrome` | `#16161c` | Terminal title bar |
| `borderSubtle` | `#1e1e24` | Window border |
| `accent` | `#ff8c42` | Caro suggestions, primary CTA |
| `accentDark` | `#ff6b35` | Hover states, secondary brand |
| `prompt` | `#4ec9b0` | Shell prompt `$` |
| `command` | `#9cdcfe` | Typed command text |
| `success` | `#22c55e` | Safe / passed indicator |
| `danger` | `#ef4444` | Blocked / error indicator |
| `warning` | `#f59e0b` | High-risk warning |
| `textPrimary` | `#e0e0e0` | Body text on dark bg |
| `textMuted` | `#a0a0a0` | Captions, secondary text |

Mirror these in `demos/remotion-video/src/tokens.ts`. Do **not** import
the website CSS — the Remotion build is isolated.

### Typography

- Body / captions: system-ui sans-serif stack
- Terminal: `'SF Mono', 'Monaco', 'Cascadia Code', 'Fira Code', monospace`
- Letter spacing in terminal: `0` (default monospace metrics; do not condense)

### Window chrome (from `LPDemo.astro`)

macOS traffic-light dots:

| Dot | Color |
|---|---|
| Close | `#ff5f56` |
| Minimize | `#ffbd2e` |
| Maximize | `#27c93f` |

Title bar background `#16161c`, 12px padding, dot diameter 12px, gap 8px.

### Authenticity rules

1. **Never invent commands.** Pull queries + expected outputs from
   `.claude/beta-testing/test-cases.yaml`. If you need a query that isn't
   there, add it to the test fixture first and verify it passes against
   the current `caro` binary, then use it in the demo.
2. **Match the real terminal output.** Caro prints the bare command on
   stdout and metadata (risk, confidence) on stderr — see
   `src/main.rs` around lines 1014–1023 for the literal block messages.
3. **Use real install commands.** `cargo install caro` and
   `brew install wildcard/tap/caro`. No marketing pseudo-syntax.

## Scaffold

The Remotion project lives at `demos/remotion-video/`. Structure:

```
demos/remotion-video/
├── package.json           # remotion, @remotion/cli, react, react-dom
├── remotion.config.ts     # 1920x1080 @ 30fps, h264
├── tsconfig.json
└── src/
    ├── index.ts           # registerRoot(Root)
    ├── Root.tsx           # <Composition id="CaroDemo" durationInFrames={900} />
    ├── CaroDemo.tsx       # <Series> of scenes
    ├── tokens.ts          # color + font tokens
    ├── components/
    │   ├── TerminalWindow.tsx
    │   ├── TypewriterLine.tsx
    │   └── Caption.tsx
    └── scenes/
        ├── ScenePain.tsx
        ├── SceneQueries.tsx
        ├── SceneSafety.tsx
        └── SceneCloser.tsx
```

Total duration: 900 frames @ 30fps = **30 seconds**.

## Scene Cadence

| # | Scene | Frames | Duration | Caption | Terminal action |
|---|---|---|---|---|---|
| 1 | ScenePain | 0–119 | 4s | "Forgot the syntax. Again." | Cursor blinks; `# how do I find...` types in then trails off |
| 2 | SceneQueries | 120–539 | 14s | Per-query badge: `0.3s · 100% local` | 3 caro queries from test-cases.yaml render back-to-back, ~4.5s each |
| 3 | SceneSafety | 540–779 | 8s | "52+ patterns. Blocked before damage." | `caro "delete everything in the current directory"` → red `✗ command blocked by safety validator (Critical)` |
| 4 | SceneCloser | 780–899 | 4s | "Local. Private. No API key." | Logo + `cargo install caro` install line |

Scene 2 queries (verified against `.claude/beta-testing/test-cases.yaml`):

1. `caro "find all PDF files larger than 10MB in Downloads"` → `find ~/Downloads -name "*.pdf" -size +10M -ls`
2. `caro "find python files modified last week"` → `find . -name "*.py" -type f -mtime -7`
3. `caro "find all errors in application logs"` → `grep ERROR logs/app.log`

## Build Workflow

```bash
# 1. First-time scaffold (skip if demos/remotion-video/ already exists)
cd demos/remotion-video
npm install

# 2. Iterate
npx remotion preview
# → opens Remotion Studio at http://localhost:3000
# → hot reloads on file save; tune timing visually

# 3. Render the MP4 + poster
npx remotion render CaroDemo \
  ../../website/public/caro-demo.mp4 \
  --codec=h264 --crf=23 --image-format=jpeg

npx remotion still CaroDemo \
  ../../website/public/caro-demo-poster.png \
  --frame=60

# 4. Verify in the website
cd ../../website
npm run dev
# → load http://localhost:4321/, scroll to "30 Seconds to Terminal Mastery"
```

## Wiring

The slot is `LPVideoDemo.astro` and is rendered from
`website/src/pages/index.astro` (around line 23). Pass:

```astro
<LPVideoDemo
  videoUrl="/caro-demo.mp4"
  posterUrl="/caro-demo-poster.png"
/>
```

When `videoUrl` is truthy the placeholder swaps for `<video controls>`.
The component is already styled — no CSS edits needed.

The same MP4 is **English only** (root `/`). Non-English locales render
`Video.astro` (asciinema embed) instead — leave those untouched unless
explicitly extending scope.

## Render Quality Targets

| Setting | Value | Why |
|---|---|---|
| Resolution | 1920×1080 | Retina-friendly, standard hero |
| FPS | 30 | Web-safe; 60 doubles file size for no perceived gain on text-heavy demo |
| Codec | h264 | Universal browser support, small file |
| CRF | 23 | Visually lossless for terminal text; bigger CRF (lower) bloats text-heavy frames quickly |
| Audio | none | Silent demo; site has `<video controls>` so users opt in to play anyway |
| File size target | < 6 MB | Hero asset budget — re-encode if larger |

## Update Workflow (when caro changes)

1. Re-run each Scene 2 query against the current `caro` binary; if any
   command drifted, update `.claude/beta-testing/test-cases.yaml` first
   to keep the test fixture authoritative.
2. Update the typed line in `SceneQueries.tsx` (and `SceneSafety.tsx`
   if the safety message text changed in `src/main.rs`).
3. Re-render and re-ship per the Build Workflow above.
4. Bump the version number in the demo's title bar (e.g.
   `caro v1.4.0 — demo`) only if a major release dropped.

## Out of Scope (track separately if requested)

- Voiceover audio — the `remotion-best-practices` skill covers ElevenLabs
  TTS at `rules/audio/voiceover.md` if needed
- Localized text overlays — current MP4 is English; per-locale text would
  require either separate renders or runtime SVG overlays
- Lambda / cloud rendering pipeline — local `npx remotion render` is fast
  enough for a 30s clip (~30s render time on M-series)
- A second longer-form demo (1–2 min "deep dive") — would need its own
  composition + scene set
