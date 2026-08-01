---
name: caro-demo-video
description: Build, render, ship, AND MAINTAIN the caro landing-page demo video using Remotion. Use when creating, updating, re-rendering, extending the project demo MP4 at website/public/caro-demo.mp4 — or when responding to a drift alert from the caro-demo-drift CI workflow or a beads task with label `caro-demo-video`.
---

# Caro Demo Video

Reproducible, on-brand product demo video for the caro website, rendered
from React code via [Remotion](https://www.remotion.dev/). This skill
covers the full lifecycle: **build → render → ship → detect drift →
re-render**.

## When to Use

- "create the project demo video"
- "render a new caro demo"
- "update the landing page video"
- "add a new scene to the demo"
- "the demo on the website is missing / outdated"
- "the caro-demo-drift workflow flagged my PR — what now?"
- A beads task with label `caro-demo-video` is in your `bd ready` queue

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

| Token | Hex | CSS source | Use |
|---|---|---|---|
| `bgDeep` | `#1a1a1a` | `--caro-grey-950` | Composition background |
| `bgTerminal` | `#4f4f4f` | `--caro-grey-700` = `--term-bg` | Terminal window body |
| `bgChrome` | `#1a1a1a` | `--caro-grey-950` | Terminal title bar |
| `borderSubtle` | `#3a3a3a` | `--caro-grey-800` | Window border |
| `accent` | `#ef3333` | `--caro-red-500` = `--accent` | Caro suggestions, primary CTA |
| `accentDark` | `#e63636` | `--caro-red-600` | Hover states, secondary brand |
| `prompt` | `#fcfc62` | `--caro-yellow-400` = `--term-prompt` | Shell prompt `$` |
| `command` | `#f4f1df` | `--caro-beige-100` = `--term-fg` | Typed command text |
| `success` | `#22c55e` | `--color-success` | Safe / passed indicator |
| `danger` | `#ef3333` | `--status-danger` | Blocked / error indicator |
| `warning` | `#f59e0b` | `--color-warning` | High-risk warning |
| `textPrimary` | `#f4f1df` | `--caro-beige-100` | Body text on dark bg |
| `textMuted` | `#a0a0a0` | `--caro-grey-400` | Captions, secondary text |

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
| 3 | SceneSafety | 540–779 | 8s | "67+ patterns. Blocked before damage." | `caro "delete everything in the current directory"` → red `✗ command blocked by safety validator (Critical)` |
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

## Drift Detection

The video bakes in mutable facts: brand colors, terminal chrome design,
real CLI output strings, the "52+" pattern count, install commands.
These come from 7 specific files. Drift between those files and the
shipped MP4 means the video is lying about the product.

### Baseline manifest

[`demos/remotion-video/.baseline-manifest.json`](../../demos/remotion-video/.baseline-manifest.json)
is the source of truth. It records:

1. **Tripwire SHAs** — SHA-256 of each watched file at the last
   successful render. Mechanical, cheap to check.
2. **Semantic claims** — the actual scene-text strings baked into the
   video: pattern count, install command, block message, the three
   Scene 2 query/output pairs, the Scene 3 query. So the next agent
   reading the manifest can see at a glance what the video *says*,
   without having to re-derive it from the source code.

### Watched files (7)

| File | Why it matters | Affects |
|---|---|---|
| `website/src/ui/tokens.css` | Brand palette | All scenes |
| `website/src/components/landing/LPDemo.astro` | Terminal chrome design | All scenes |
| `src/main.rs` (lines ~1014–1023) | CLI block-message string | Scene 3 |
| `.claude/beta-testing/test-cases.yaml` | Verified caro queries + outputs | Scene 2 |
| `src/safety/patterns.rs` | Pattern count (currently 52) | Scene 3 caption |
| `homebrew-tap/README.md` | Install command | Scene 4 |
| `install.sh` | Curl-pipe install | Scene 4 |

### Run the check

```bash
demos/remotion-video/scripts/check-drift.sh --human
```

Exit codes: `0` no drift, `1` drift detected, `2` tooling error.
Default output is JSON; `--human` is for terminals.

### Triggers (how an agent gets here)

Three independent channels — at least one will catch drift:

1. **CI watcher (path-filtered)** —
   [`.github/workflows/caro-demo-drift.yml`](../../.github/workflows/caro-demo-drift.yml)
   runs `check-drift.sh` on every PR that touches a watched file. If
   drift is detected, the workflow comments on the PR with the specific
   files that changed and links here.
2. **Monthly cron** — a scheduled beads task (label `caro-demo-video`)
   forces a fresh check even when no PR touched a watched path. Catches
   slow brand drift over many small commits.
3. **Manual / human-triggered** — anyone can run the check script and
   file a beads task tagged `caro-demo-video`.

## Trust Model — when can an agent ship without human review?

The re-rendered MP4 is a public-facing marketing asset. Two paths:

| Drift type | Examples | Required action |
|---|---|---|
| **Output-equivalent** | tokens.css color literal renames (e.g. variable rename, same hex value); whitespace-only changes in LPDemo.astro; install.sh comment edits | Agent re-renders and ships directly. Visual diff against the prior poster.png must be within ±5% pixel delta. |
| **Cosmetic but visible** | Hex color value changes; LPDemo chrome restyle; new pattern count claim ("52+" → "60+") | Agent renders, opens a PR with the new MP4 + poster, requests human review. |
| **Semantic** | Scene 2 query strings drift; block message changes; install command changes | Agent renders, opens a PR with side-by-side commentary (old vs new strings), requests human review. **Do not auto-ship.** |

The agent decides which bucket applies by reading the
`claims_baked_into_video` section of the manifest against the current
working tree. If a query string in `test-cases.yaml` changed, that's
semantic → PR. If only a hex value changed in `tokens.css`, that's
cosmetic-but-visible → PR. If nothing semantic changed and the render
is byte-identical, the agent may push directly to main.

## Claiming a Maintenance Task

This skill is **agent-claimable**. A Claude Code session reaching this
section should:

1. **Check the trigger**.
   - Beads task in `bd ready` with label `caro-demo-video` → read
     `bd show <id>` for the trigger note.
   - PR comment from `caro-demo-drift workflow` → read the comment
     for the drifted-files list.
   - Manual invocation → run `check-drift.sh --human` for the report.
2. **Decide the trust bucket** from the table above.
3. **Re-render** with
   `demos/remotion-video/scripts/render-and-ship.sh` — this also
   refreshes the baseline manifest, so subsequent drift checks anchor
   to this render.
4. **Ship per trust bucket**:
   - *Output-equivalent*: commit + merge directly.
   - *Cosmetic / Semantic*: open a PR with the rendered MP4 + poster +
     refreshed manifest, request human review.
5. **Close the beads task** with `bd close <id> --reason="re-rendered
   <reason>; commit <sha>"`.

If unable to complete (e.g. queries failed when verified against the
current `caro` binary, terminal chrome rebuild needed), keep the task
`in_progress` and add a comment explaining what's blocking, so the next
agent (or a human) can pick up.

## Out of Scope (track separately if requested)

- Voiceover audio — the `remotion-best-practices` skill covers ElevenLabs
  TTS at `rules/audio/voiceover.md` if needed
- Localized text overlays — current MP4 is English; per-locale text would
  require either separate renders or runtime SVG overlays
- Lambda / cloud rendering pipeline — local `npx remotion render` is fast
  enough for a 30s clip (~30s render time on M-series)
- A second longer-form demo (1–2 min "deep dive") — would need its own
  composition + scene set
