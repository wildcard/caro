# Design Dialogue Protocol — Working with Claude Design

**APPLIES TO**: Any work that touches Caro's brand identity (color tokens,
typography, mascot, icons, card/box specs, voice) or that involves
back-channel communication with the UI/UX persona at https://claude.ai/design.

Codified from the v2.1 pixel-icon delivery cycle (2026-04-28/29), where a
clean propose → ship → ack → ingest loop with Claude Design produced 9 SVG
icons + manifest in a single chat turn — but only because the protocol
below was followed exactly.

## Rule 1 — Screenshot custody

**All screenshot work MUST go through a sub-agent.** The parent orchestrator's
context is for coordination signal, not pixel data.

- Spawn `claude-design-frontend-engineer` (or `general-purpose` with the
  persona embedded inline if the agent registry hasn't picked up the agent
  file yet) for any task that involves taking, inspecting, or comparing
  screenshots.
- Sub-agents return **text-only reports** in the 6-section format:
  `Audit summary / Gaps found / Decisions / Files changed / Beads filed /
  Next move`.
- Save screenshots under `.claude/audits/<date>/<slug>.png` and reference
  paths only — never embed bytes in the reply.

## Rule 2 — Peer relationship with Claude Design

Claude Design is the **brand authority**, not a code generator. The parent
agent's role is to translate her decisions into running code, not to override
them.

- Address her as a domain expert. Present evidence, ask for guidance.
- Never instruct ("change X to Y"). Always ask ("the footer at … shows links
  at ~1.6:1 contrast; could you advise on the intended color, or confirm we
  should use `var(--fg-strong)`?").
- Cite specific brand-book rules when escalating ("per the 95/5 composition
  rule, this surface reads as 70/30 — should the homepage shift?").
- Respect deferrals. If she said "v3", don't re-litigate; track it as a
  v3 dependency in beads.
- Token-level changes (`tokens.css`) require her citation in the commit.

## Rule 3 — Send-message gate

Messages to claude.ai/design are public artifacts on her work surface. They
get **explicit user approval** before sending.

- The agent **drafts** the message and shows the verbatim text.
- The user reviews and approves with `send` (or equivalent).
- Sensitive feedback (e.g. "your glyph reads as a horned demon") is **always**
  user-gated, even when prior turns approved sends.
- The first acknowledgment after she ships work is acceptable to auto-queue
  (her direct ask) but still gated behind user `send`.

## Rule 4 — Revert-don't-delete for design assets

When a Claude Design artifact doesn't ship cleanly (visual readability gap,
brand-fidelity regression, contrast failure):

- **Revert the call sites**, not the asset.
- Keep the SVG/PNG/font file in `public/` and the manifest entry in
  `src/data/`.
- Add a `NOTE(caro-XXX)` comment at each reverted call site citing the
  beads issue.
- The future swap when v2.1.1+ lands is then a forward-edit, not a
  re-implementation.

Worked example: caro-bsi (kyaro-mark glyph reads as horned demon). The
SVG and manifest entry stayed at `website/public/icons/kyaro-mark.svg`
and `website/src/data/icon-manifest.json`; only Footer.astro's two call
sites reverted to 🐕 emoji with `NOTE(caro-bsi)` comments. The future
v2.1.1 swap is a 4-line forward-edit.

## Rule 5 — Visual audit before merging brand-touching PRs

PRs that touch `website/src/components/landing/LP*.astro`, `Footer.astro`,
`Navigation.astro`, or `website/public/{icons,fonts,kyaro,*.png}` MUST get
a visual audit pass via `claude-design-frontend-engineer` (or persona-
embedded `general-purpose`) before merge.

The audit checks: composition (95/5 paper rule), token usage (`var(--accent)`
not hardcoded hex), card spec (no left-accent stripes, no scale on hover),
mascot variant choice (smooth on light, pixel on dark + small), glyph
readability across 16/20/24/32/48 px.

Worked example: PR #1007's audit caught a P0 (kyaro-mark readability) that
static code review had missed entirely. Build was green, code was clean,
but the rendered SVG read as the wrong shape to human eyes. The 6-section
audit format forced the finding into a P0 frame rather than a "FYI" note.

## Rule 6 — Existing reference docs

Sub-agents booting into this protocol should refresh from:

- `website/src/ui/tokens.css` — canonical token list (single source of truth)
- `website/src/components/landing/LPHero.astro` — gold-standard component
  pattern (Azeret Mono headline, Kyaro idle GIF, flat-red CTAs)
- `.claude/rules/astro-esbuild-shell-syntax.md` — JSX brace pitfalls
- `.claude/agents/claude-design-frontend-engineer.md` — the agent's persona
  and 6-section output contract

## Why this matters

A bidirectional dialogue with Claude Design is the project's most valuable
design-system process. Without these rules, three things go wrong:

1. **Parent context bloats** with screenshot bytes (one screenshot can
   exceed a coordination-only turn's entire budget).
2. **Tone drift** — agents start instructing instead of asking, eroding
   her authority and inviting her to push back instead of ship.
3. **Silent overrides** — token changes land in `tokens.css` without her
   citation, breaking the single-source-of-truth contract.

Five minutes of process per dialogue, codified once, beats a 50-comment
PR thread debriefing what went wrong.

## See also

- `.claude/agents/claude-design-frontend-engineer.md` — the operational
  agent definition this rule references (persona, tool hierarchy, output
  contract)
- `caro-bsi` (closed) — kyaro-mark readability finding that validated the
  audit-before-merge pattern
- `caro-y5i` (PR #1007) — first cycle through the full propose → ship →
  ack → ingest loop
- `caro-hzr` (open) — agent rollout tracker; documents the session-bound
  agent-discovery limitation that forces inline persona embedding
