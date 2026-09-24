# ADR 004 — Headless `stream-json` NDJSON Event Contract

- **Status:** Proposed (drafted 2026-07-01 by the `caro-research--scoping-process` scheduled task)
- **Deciders:** Caro maintainers
- **Supersedes / relates to:** ADR 003 (structured output envelope & stateless session contract) — this ADR is the streaming layer ADR 003 explicitly deferred as "Alternative 1 / a later ADR."
- **Numbering note:** placed in the lowercase architecture series (`001-`, `002-`, `003-`, `004-`).
  Per `.claude/rules/adr-numbering.md`, renumber on merge if another `004-` lands first.

> Scope note: this ADR is the decision record. The full Phase 1–3 analysis,
> type definitions, file-change list, and test matrix live in
> `caro-scope-stream-json.md` at the repo root.

> Feature-selection note: the task template left `[FEATURE NAME]` unfilled and
> ran unattended. I selected **Claude Code's `--output-format stream-json`**
> (the NDJSON event stream of headless mode) because it is the exact capability
> ADR 003 named as its deferred next step, it reuses the `CommandEnvelope` ADR 003
> already defines, and it maps cleanly onto every task constraint (structured
> event contract, exit codes, session lifecycle, pure subprocess).

---

## Context

ADR 003 gave Caro a single, versioned `CommandEnvelope` on `caro -o json`: one
object in, one object out, a disambiguated exit-code table, and a first-class
`session_id`. It deliberately deferred the **streaming** case — "generation is
single-shot today; defer until a streaming consumer exists."

Three things have changed that context since 2026-06-25:

1. **Streaming consumers now exist.** `caro fix` (ADR 016, self-healing repair)
   and the agentic context pipeline (`src/agent/`) are multi-step: they
   generate, validate, and can re-generate. A caller driving these as a
   subprocess today sees nothing until the process exits — no progress, no
   intermediate verdict, no way to stream a long local-model generation to a
   TUI or CI log.
2. **Local generation is slow enough to want progress.** An embedded MLX/CPU
   model can take seconds to first token. A single terminal envelope gives the
   caller a frozen pipe until completion; a progress event stream lets a wrapper
   render "generating…", surface `model_init_ms` live, and abort early.
3. **The reference implementation's stream contract is a moving target.** Claude
   Code's `--output-format stream-json` emits typed NDJSON events
   (`system/init`, `stream_event`, `system/api_retry`, terminal `result`), but
   the community has open issues asking Anthropic to even *document the event
   types* ([#24596](https://github.com/anthropics/claude-code/issues/24596),
   [#24612](https://github.com/anthropics/claude-code/issues/24612)). The event
   shapes are coupled to the binary version, carry no `schema_version`, expose
   no explicit ordering/sequence guarantee, and — as headless lifecycle bugs
   (unbounded stdin, background tasks holding the process open) showed — the
   stream can hang or interleave. These are contract gaps we can close by
   designing the event schema first.

Caro needs a streaming contract that a machine can consume as reliably as the
ADR 003 envelope, without inheriting any of those gaps, and without a daemon.

## Decision

Add `caro -o stream-json`: a **versioned NDJSON event stream** in which every
line is one fully-serializable event object, the terminal event reuses the
ADR 003 `CommandEnvelope` verbatim, and the process still exits with
`envelope.exit_code`. The feature stays a pure subprocess concern: no daemon, no
shared state, no ambient discovery.

Concretely:

1. Add serializable event types to `src/models/mod.rs` (beside `CommandEnvelope`):
   a single `StreamEvent` enum tagged by `type`, whose variants are
   `Init`, `Progress`, and `Result`. Every event carries the same
   `schema_version: u32` as the envelope plus a monotonic `seq: u64`.
2. The **first** line is always exactly one `Init` event (session id, backend,
   shell, model-warm flag). The **last** line is always exactly one `Result`
   event whose payload is the ADR 003 `CommandEnvelope`. Zero or more `Progress`
   events may appear between them. This ordering is a guaranteed part of the
   contract — the specific gap (no guaranteed terminal event, ambiguous order)
   we found in Claude Code's stream.
3. Reuse everything: `Result.envelope` is the ADR 003 type unchanged; risk from
   `models::RiskLevel`; safety from `safety::SafetyValidator`; session from
   `ai::store::SessionStore`; backend/shell from `cli::CliApp`. `stream-json` is
   a *presentation* of the same computation the envelope already drives.
4. Exit codes are **identical to ADR 003's table** — the stream changes framing,
   not semantics. A machine that only reads the last line and the exit code
   behaves exactly as under `-o json`.
5. Statelessness makes the Claude Code lifecycle foot-guns structurally
   impossible: one process, bounded stdin, a `flush()` after every line, no
   background tasks, so the stream can neither hang open nor interleave with
   another turn's events.

## Consequences

**Positive**
- Callers get live progress and a guaranteed, versioned terminal result from a
  daemonless subprocess — the thing ADR 003's single envelope could not provide.
- `schema_version` + `seq` on every event fix Claude Code's unversioned,
  unordered stream by design, not by convention.
- The terminal event *is* the ADR 003 envelope, so the two output modes never
  drift: a wrapper can read `-o json` or the last line of `-o stream-json`
  interchangeably.
- All event types round-trip through serde from day one, reusable by the eval
  harness, `caro fix`, and telemetry.

**Negative / costs**
- A second wire surface (`StreamEvent`) to keep stable alongside `CommandEnvelope`;
  its `schema_version` moves in lockstep with the envelope's.
- Emitting progress means threading a lightweight event sink through the
  one-shot and `ai --once` paths (an `FnMut(StreamEvent)`), a small plumbing cost.

**Neutral**
- `Plain` stays the default and `-o json` is unchanged; `stream-json` is purely
  additive opt-in.

## Alternatives Considered

1. **Mirror Claude Code's token-delta `stream_event` (per-token deltas).**
   Rejected for this ADR: token-level advisory streaming is a distinct concern
   (backend-agnostic, best-effort) and is better scoped separately; this ADR
   delivers the *lifecycle* event contract (init → progress → result) that has
   an immediate consumer. Token deltas can later ride as an additional
   `StreamEvent::Token` variant without breaking the schema.
2. **Emit a bare stream of `CommandEnvelope`s with no wrapper events.** Rejected:
   no room for an ordering/sequence guarantee, no init metadata before work
   starts, and no clean place for progress — reproducing Claude Code's
   ambiguity.
3. **A `caro serve` daemon streaming over a socket.** Rejected: violates the
   "pure subprocess, no daemon, no state" constraint. NDJSON over stdout needs
   no daemon.
4. **A new `src/stream/` module.** Rejected: the event types belong in `models`
   beside the envelope, and the emit loop is a thin helper in `cli`/`main`.
   "No new modules unless unavoidable."
