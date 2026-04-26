# Caro Demo Video (Remotion)

The 30-second product demo that fills the `LPVideoDemo` slot on the
caro landing page.

Output: [`website/public/caro-demo.mp4`](../../website/public/caro-demo.mp4)
+ poster `caro-demo-poster.png`.

For the full workflow, scene cadence, and style rules see the project skill at
[`.claude/skills/caro-demo-video/SKILL.md`](../../.claude/skills/caro-demo-video/SKILL.md).

## Quickstart

```bash
npm install
npm run dev      # Remotion Studio at http://localhost:3000
npm run ship     # Render mp4 + poster into website/public/
```

## Layout

| File | Role |
|---|---|
| `src/Root.tsx` | `<Composition>` registration (1920×1080 @ 30fps, 900 frames) |
| `src/CaroDemo.tsx` | Top-level `<Series>` of 4 scenes |
| `src/tokens.ts` | Color + font tokens (mirrored from `website/src/ui/tokens.css`) |
| `src/components/TerminalWindow.tsx` | macOS chrome wrapper |
| `src/components/TypewriterLine.tsx` | Char-by-char typing with blinking cursor |
| `src/components/Caption.tsx` | On-screen caption overlay |
| `src/scenes/ScenePain.tsx` | Scene 1 (4s) — "Forgot the syntax. Again." |
| `src/scenes/SceneQueries.tsx` | Scene 2 (14s) — three real caro queries |
| `src/scenes/SceneSafety.tsx` | Scene 3 (8s) — safety validator blocks `rm -rf` |
| `src/scenes/SceneCloser.tsx` | Scene 4 (4s) — logo + tagline + install line |

## Scene timeline

```
 frame  0 ────────────────────────────────────────────────── 900
        ┌──────┬─────────────────────────┬──────────┬──────┐
        │Pain  │       Queries           │  Safety  │Closer│
        │ 4s   │        14s              │    8s    │  4s  │
        └──────┴─────────────────────────┴──────────┴──────┘
```

## Authenticity

Real queries pulled from `.claude/beta-testing/test-cases.yaml`. Block
message text matches `src/main.rs:1014-1023`. Do not invent commands —
re-verify against the current `caro` binary if anything changes.
