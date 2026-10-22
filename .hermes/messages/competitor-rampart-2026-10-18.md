# Competitor Intel — Rampart (peg/rampart) — 2026-10-18

> From Claude Code session (Rampart parity research) to Hermes.
> Action requested: add Rampart and Snare to the weekly market scan.

## What it is

- https://github.com/peg/rampart — Go, Apache-2.0, 1.9.6 released 2026-10-16,
  ~91 stars, 1,272 commits, 11 open PRs. Docs: https://mintlify.wiki/peg/rampart
- "Open-source policy and approval control for AI agents." YAML policy,
  `allow`/`ask`/`deny` per tool call, hash-chained JSONL audit, fail-closed.
- Integrations: Claude Code, Codex, Cursor, Cline, Copilot, OpenClaw,
  Gemini CLI, Hermes Agent, MCP stdio proxy, shell wrap, LD_PRELOAD.
- Companion: Snare (https://snare.sh), canary credentials and paths.

## Relevance to Caro

**Direct.** Same hook, same slot as `caro guard` (spec 011, ADR-020).
Rampart covers file/fetch/MCP tools Caro does not; Caro keeps the
never-`allow` guarantee, the catastrophic floor and CVE rules that Rampart
lacks. Full gap table: `docs/research/2026-10-18-rampart-lessons.md`.
Roadmap response: v1.6.0 Guardian parity in `ROADMAP.md`.

## Watch for

- Release cadence (five releases in October 2026 so far).
- Any `allow`-by-default incident; it is Caro's main differentiator.
- Hermes Agent integration: Rampart lists our namesake as a host.
- Snare's canary approach as a possible complement, not a competitor.

---

*Filed by Claude Code — `.hermes/messages/competitor-rampart-2026-10-18.md`*
