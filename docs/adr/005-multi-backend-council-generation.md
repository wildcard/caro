# ADR 005 — Multi-Backend Council Generation (Stateless Fan-Out Candidates)

- **Status:** Proposed (drafted 2026-07-16 by the `caro-research--scoping-process` scheduled task)
- **Deciders:** Caro maintainers
- **Supersedes / relates to:** ADR 003 (structured output envelope — the council result rides inside `CommandEnvelope`), ADR 004 (stream-json — per-candidate events deferred there), ADR-025 (init snapshot — amortizes the embedded cold start the council pays once)
- **Numbering note:** placed in the lowercase architecture series (`001-`…`004-`).
  Per `.claude/rules/adr-numbering.md`, renumber on merge if another `005-` lands first.

> Scope note: this ADR is the decision record. The full Phase 1–3 analysis,
> type definitions, file-change list, and test matrix live in
> `caro-scope-multi-backend-council-2026-07-16.md` at the repo root.

> Feature-selection note: the task template left `[FEATURE NAME]` unfilled and ran
> unattended. I selected **Claude Code's subagents + experimental Agent Teams**
> (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`, still experimental as of July 2026)
> because it is the last major headless-adjacent capability with no Caro ADR in
> either numbering series (fan-out/parallel generation is uncovered; hooks,
> sessions, streaming, permissions, sandbox, and skills all have ADRs), and its
> documented failure modes map cleanly onto a schema-first design we can do better.

---

## Context

Claude Code parallelizes work by spawning **subagents** (isolated context windows
reporting a final prose message back to the parent) and, experimentally, **agent
teams** (peer sessions with a shared task list and direct messaging, coordinated
through daemon-managed background sessions). The value: independent workers explore
in isolation, and the parent synthesizes. The documented failure modes: teammate
sessions cannot be resumed, task-status tracking lags, results come back as
unstructured prose rather than a typed contract, costs multiply per worker, and the
daemon coordination layer has produced environment-corruption bugs (stale `PATH`
inherited from the daemon; subagents silently stopped and re-run on session return
— both fixed only in July 2026 patches).

Caro's unit of work is one shell command, produced in seconds. It does not need
collaborating peers; it needs **diverse independent opinions on the same query**.
Caro already has exactly the substrate for that and uses it one-backend-at-a-time:
seven CLI-servable backends behind one trait (`CLI_SERVABLE_BACKENDS` in
`src/backends/mod.rs`: embedded, ollama, exo, vllm, mesh, ai-horde, hybrid), all
returning a serializable `GeneratedCommand` (`src/models/mod.rs`) with a validated
`confidence_score`, and a single `SafetyValidator` (`src/safety/mod.rs`) that can
judge every candidate identically. Today `--backend` picks one; there is no way for
a caller — human or agent — to ask "what do several backends say, and do they
agree?"

## Decision

Add **council mode**: a stateless fan-out that queries N configured backends
concurrently for the same request, validates every candidate through the existing
`SafetyValidator`, ranks candidates with a **deterministic, documented total order**
(no LLM judge), and returns them as typed, serializable data inside the ADR 003
`CommandEnvelope`. Pure subprocess: one `caro` invocation, `tokio` join over
backend futures, per-backend timeout, no daemon, no shared state, no inter-worker
messages.

Concretely:

1. New CLI surface: `--council <b1,b2,...>` (comma list of CLI-servable backend
   names, 2–5 entries) and `--council-timeout-ms <n>` (per-backend budget,
   default 10000). `--council` conflicts with `--backend` and with `-x/--execute`
   in v1 (generation-only).
2. New serializable types in `src/models/mod.rs` (the existing serde home):
   `CouncilCandidate` and `CouncilSummary`. Every field `Serialize + Deserialize`
   from day one. Backend failures are **first-class data**: a candidate whose
   backend timed out or errored still appears, with `error: Some(GeneratorError)`
   (already serializable) and no command — never silently dropped. This solves,
   by schema design, the "worker fails silently / result is prose" failure mode
   found in Phase 1.
3. Envelope integration, not a second envelope: `CommandEnvelope` gains an
   **optional** `council: Option<CouncilSummary>` field. The ranked winner
   populates the existing top-level command/risk fields, so ADR 003 consumers
   work unchanged. Additive-optional ⇒ `schema_version` stays 1.
4. Deterministic ranking (documented in `docs/headless-contract.md`):
   (a) `allowed` candidates before blocked; (b) lower `RiskLevel` first;
   (c) higher agreement count (exact command match after whitespace
   normalization); (d) higher `confidence_score`; (e) CLI `--council` list order
   as the stable tie-break. Identical inputs ⇒ identical ordering.
5. Exit codes reuse the ADR 003 table unchanged: `0` if ≥1 allowed candidate
   exists (winner selected), `3` if all candidates were generated but blocked,
   `4` if all backends failed/unavailable, `2` for usage errors (unknown backend
   name, <2 entries). Partial backend failure with ≥1 allowed candidate is `0`.
6. Orchestration lives as one free function in `src/backends/mod.rs`
   (`run_council`) reusing existing constructors and
   `SafetyValidator::validate_batch`. No new module.

## Rationale

- **Safety consensus is Caro's move, not theirs.** Claude Code cannot run every
  subagent result through one deterministic 52-pattern validator; Caro can, and
  cross-backend disagreement on risk is itself a signal agents can branch on.
- **Schema-first beats prose synthesis.** Their parent agent re-reads worker prose;
  our caller gets a typed, ranked array with per-candidate errors.
- **No daemon, no daemon bugs.** The stale-`PATH`/unresumable-session failure class
  cannot exist in a single-process fan-out.
- **Offline and community-inclusive by construction**: a council of
  `embedded,ollama` runs air-gapped; `ai-horde` adds a free community voice.
- Reuses everything: trait objects, `GeneratedCommand`, `ValidationResult`,
  `RiskLevel` ordering, envelope, exit codes. The council is a coordination layer
  over data Caro already computes.

## Consequences

**Benefits:** agents get second opinions + agreement signal in one subprocess call;
deterministic and testable; zero new persistent state; per-candidate latency/error
visibility.

**Trade-offs:** wall-clock cost is max(backend latencies) and token cost is N×;
embedded cold start still paid once per process (ADR-025 snapshot mitigates);
one more optional envelope surface to keep stable.

**Risks:** remote backends (ai-horde) can be slow → per-backend timeout with the
candidate recorded as `error: Timeout`; command-string agreement is a coarse
equivalence (flag reordering reads as disagreement) → documented; AST-level
equivalence deferred (ADR-007 parser is prior art).

## Alternatives considered

1. **LLM judge / rerank of candidates** — rejected for v1: nondeterministic,
   doubles latency, contradicts the reproducible-contract goal. Possible later
   behind a flag.
2. **Peer messaging between backends (true "agent team")** — rejected: Caro's task
   has no sub-goals to negotiate; adds daemon-shaped state for no accuracy gain.
3. **Sequential fallback chain instead of parallel fan-out** — already implicit
   today; provides availability, not diversity/agreement signal.
4. **Separate `caro council` subcommand** — rejected: a flag composes with the
   existing generate path, envelope, and future `caro ai` integration.

## Out of scope (next versions)

Per-candidate NDJSON streaming (extends ADR 004), council inside multiturn
`caro ai` sessions, weighted/configured voting in `config.toml`, AST-based
command equivalence, LLM judge, execution of the winner (`-x`), parallel model
downloads.
