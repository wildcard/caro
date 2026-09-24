# ADR-007: Deterministic Session-History Compaction for Headless Conversations

- **Status**: Proposed
- **Date**: 2026-07-31
- **Authors**: caro-research scoping process (automated scheduled run)
- **Target**: Community CLI + agent wrappers (headless conversation mode)
- **Builds on / relates to**: ADR-026 (headless multi-turn session), ADR-024
  (headless JSON contract), ADR-025 (init snapshot cache), 006 (persistent
  learned memory), 004 (stream-json event contract)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task on 2026-07-31 with
> `[FEATURE NAME]` unbound. Coverage sweep of both ADR series (001–006,
> ADR-001–043) found context/window **compaction** unscoped anywhere, while
> it ships experimentally in every major agent CLI (Claude Code
> auto-compact/microcompact, Agent SDK `compact_boundary`, gemini-cli chat
> compression). ADR-026 explicitly bounds *stdin*, not *history growth*; the
> in-repo history mechanism is a silent sliding window. Treat the analog
> choice as a reviewable assumption. Not committed (git-workflow rule).

---

## Context

### The problem

A caro session accumulates turns: in ADR-026 conversation mode within one
process, and across processes via `--session` resume (`SessionStore` TTL
resume). Prior turns are injected into the generation prompt via
`AiSession::render_history(max_turns)` (`src/ai/session.rs:110`) — a **silent
sliding window** that drops everything older than the last N turns. The one
call site hardcodes `render_history(6)` (`src/ai/privacy.rs:55`).

Consequence: a constraint stated in turn 1 ("only inside `src/`, never touch
`node_modules`") silently falls out of the window by turn 8, and generated
commands regress without any signal in the envelope. For caro's embedded
models (SmolLM-class, 2–8K context) the window pressure is acute after a
handful of turns — this is not a 200K-model luxury problem.

### What the analogs do (Phase 1 summary)

- **Claude Code auto-compact**: at ~95% of effective window (threshold =
  window − 13K buffer) it pauses, runs an LLM summarization pass, replaces
  older turns with the summary, then reassembles boundary marker + summary +
  recent files + instructions. A cheaper **microcompact** tier prunes stale
  tool results with no model call. Still experimental in behavior: thresholds
  and summary format are unversioned and drift across releases.
- **Agent SDK contract**: a system event `compact_boundary` with
  `trigger: auto|manual` and `pre_tokens` — the only machine-visible signal.
- **gemini-cli**: `model.compressionThreshold` (default moved 0.7 → 0.5),
  manual `/compress`; summarize-and-replace of history.

### Why it is experimental / the failure modes to design around

Documented analog failures, all traceable to **compaction being an LLM call
whose success is not guaranteed**:

1. **Infinite compaction loops** — compaction requests that cannot complete
   retrigger forever (claude-code #6004, #12556, #41984).
2. **Death spiral** — when fixed context (system prompt, tools) consumes most
   of the window, each compaction frees too little and immediately retriggers
   (claude-code #24677: 6 compactions in 3.5 min at 86.5% fixed context).
3. **Compaction itself overflows** — `/compact` fails with "conversation too
   long" (claude-code #23751): the summarizer needs the very context it is
   trying to shrink.
4. **Compounding lossy summarization** — LLM summaries retain ~3.7/5 of
   information; after 3–4 summarize-the-summary cycles, early constraints are
   permanently gone, and the original is destroyed (replace-in-place).
5. **Post-compact amnesia** — the agent forgets what it already did/read and
   loops re-doing it.

## Decision

Introduce **deterministic, lossless-at-rest history compaction** as the way
session history is rendered into prompts. Five commitments, each closing a
failure mode **by construction**:

1. **No LLM in the compaction path (v1).** Compaction is a pure fold over
   `AiSession.turns`: pin the first user turn (task framing), render the last
   K turns verbatim, collapse the middle into one-line deterministic gists
   (assistant turns → their `command`; user turns → first line, capped).
   Failure modes 1–3 are structurally impossible: a fold cannot loop, cannot
   fail, and its output size is computable a priori
   (`≤ pinned + K·turn_cap + folded·gist_cap` chars).
2. **Lossless at rest.** Compaction changes only what is *rendered*, never
   what is *stored*. `AiSession` on disk keeps every turn; re-rendering from
   the full session each time makes compaction idempotent and reversible —
   no summarize-the-summary decay (failure mode 4).
3. **Salience beats recency.** The sliding window's bug is dropping the
   *oldest* content, which is where task framing lives. The pin-first +
   fold-middle + verbatim-tail shape keeps both ends. (Persistent preferences
   across sessions remain ADR/006 memory's job — not duplicated here.)
4. **Versioned, observable contract.** An additive `compaction` field on the
   ADR-024/026 envelope and an additive ndjson `compaction` event mirror the
   analog's `compact_boundary` (`trigger`, `turns_folded`, `pre_chars`,
   `post_chars`) under `schema_version: "1"`. What the analogs leave implicit
   (summary format, threshold) is a published schema here.
5. **Reuse, don't rebuild.** New types live in `src/ai/session.rs` beside
   `AiSession`; policy in the existing config; envelope/event/exit-code types
   are ADR-024/026's, extended additively. No new module, no new exit codes —
   compaction can never fail a run (malformed policy is the existing usage
   exit `2`).

## Consequences

- Constraint retention becomes testable: golden-file tests can assert a
  turn-1 constraint survives 20 turns within a fixed char budget.
- Char-based budgeting (not tokens) is deliberate for v1: deterministic,
  tokenizer-independent, and conservative for the embedded models. A
  token-accurate budget is a v2 refinement behind the same policy type.
- A pure fold preserves less nuance than a good LLM summary. Accepted: the
  session file retains everything, so a future LLM tier (v2) can *add*
  fidelity without schema change, and degraded gists are visible in the
  envelope rather than silently wrong.

## Alternatives considered

- **LLM summarization tier first (analog parity)** — rejected for v1: it
  reimports failure modes 1–4 and breaks offline determinism; deferred to v2
  as an optional tier on top of the same contract.
- **Bigger sliding window** — rejected: still recency-only; the bug is
  *which* turns are dropped, not how many are kept.
- **Whole-history embedding/retrieval (ADR-017 index)** — rejected here:
  heavier, non-deterministic ordering; belongs to semantic indexing, not the
  rendering contract.
- **Do nothing (document the 6-turn window)** — rejected: silent regression
  of early constraints is a correctness bug in generated commands, caro's
  core promise.

## See scope

Full type definitions, file-change list, output contract, and integration
tests: `caro-scope-context-compaction-2026-07-31.md` (repo root).
