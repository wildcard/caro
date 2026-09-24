# ADR 003 — Structured Output Envelope & Stateless Session Contract

- **Status:** Proposed (drafted 2026-06-25 by the `caro-research--scoping-process` scheduled task)
- **Deciders:** Caro maintainers
- **Supersedes / relates to:** ADR 001 (LLM inference architecture)
- **Numbering note:** placed in the lowercase architecture series (`001-`, `002-`, …).
  Per `.claude/rules/adr-numbering.md`, renumber on merge if another `003-` lands first.

> Scope note: this ADR is the decision record. The full Phase 1–3 analysis,
> type definitions, file-change list, and test matrix live in
> `caro-scope-structured-output.md` at the repo root.

---

## Context

Caro is a stateless natural-language → POSIX command CLI. Today the `-o/--output`
flag exists (`OutputFormat::{Json,Yaml,Plain}` in `src/cli/mod.rs`), but there is
no stable machine-readable result document and no documented exit-code contract.
The `caro ai --once` path (`src/main.rs::run_ai_once`) prints **only the command
text** to stdout and routes everything else (risk, warnings, session id) to stderr
as free-form human strings. `AiOutcome` (`src/ai/runner.rs`) is `#[derive(Debug,
Clone)]` only — it is **not serializable**. The only declared exit code is
`EXIT_CODE_EDIT = 201`.

Agentic callers in our own ecosystem (Hermes, CI linters, shell widgets, the
CaroML runner) currently have to scrape stdout/stderr text to learn whether a
command was generated, blocked, or failed — exactly the fragile pattern we want to
remove.

The reference implementation we studied is Claude Code's headless mode
(`claude -p`, the Agent SDK CLI). It does many things right — three output formats,
a `session_id` in JSON, `--continue`/`--resume`. But its contract has gaps we can
avoid by designing our schema first:

1. **No schema version field.** The event/result shapes are coupled to the binary
   version (the docs are littered with `min-version:` / "as of v2.1.x" notes), so a
   script written against one release can silently break on the next.
2. **`session_id` must be scraped** out of the JSON result of the first call and
   passed back via `--resume`; it is not modeled as a stable first-class handle.
3. **Ambiguous exit semantics.** Argument errors and authentication errors are not
   cleanly separable from a single exit code, so scripts can't branch on *why* a
   run failed without parsing text.
4. **Redundant initialization by default.** Plain `claude -p` re-discovers hooks,
   skills, plugins, MCP servers, and `CLAUDE.md` on every invocation. Anthropic had
   to add `--bare` to skip that and make CI deterministic — and is making `--bare`
   the future default. The default path is non-deterministic across machines and
   pays a re-init cost every call.
5. **Lifecycle foot-guns shipped and were patched later** — unbounded piped stdin
   (capped at 10 MB only as of v2.1.128) and background tasks holding the process
   open indefinitely (fixed v2.1.163).

## Decision

Introduce a single **versioned, fully-serializable result envelope**
(`models::CommandEnvelope`) emitted on stdout when `--output json` is selected, and
a **documented, disambiguated exit-code table**. The feature is delivered as a pure
subprocess concern: no daemon, no shared state, no ambient auto-discovery.

Concretely:

1. Add serializable types to `src/models/mod.rs` (the existing home for serde
   types): `CommandEnvelope`, `EnvelopeStatus`, `EnvelopeError`, `Timings`. Every
   field is `Serialize + Deserialize` from day one. The envelope carries
   `schema_version: u32 = 1`.
2. Derive `Serialize` on `AiOutcome` and add `AiOutcome::into_envelope(...)`; add an
   equivalent builder for the one-shot `generate` path from its existing
   `GeneratedCommand` + safety validation result.
3. Make `session_id` and `resumed` first-class typed fields of the envelope — never
   scraped from prose. They reuse the session id already produced by
   `ai::store::SessionStore`.
4. Define exit-code constants in `src/main.rs` next to `EXIT_CODE_EDIT`, and exit
   with the same code reported in `envelope.exit_code`:

   | Code | Meaning | Envelope `status` / `error.kind` |
   |---|---|---|
   | 0 | Command generated and allowed | `ok` |
   | 1 | Unexpected internal error | `error` / `internal` |
   | 2 | Usage / bad arguments (e.g. no prompt) | `error` / `usage` |
   | 3 | Command blocked by safety validator | `blocked` |
   | 4 | Backend unavailable (model missing / remote unreachable) | `error` / `backend_unavailable` |
   | 5 | Configuration error | `error` / `config` |
   | 6 | Remote auth/credential error | `error` / `auth` |
   | 201 | Edit mode (existing; preserved) | n/a |

5. **Caro is "bare by default."** As a stateless subprocess it already does not
   auto-discover ambient hooks/plugins; we make that an explicit, documented
   guarantee of the JSON contract — identical input + flags ⇒ identical envelope
   (modulo `timings`), no special flag required.
6. Surface the cold-start cost rather than hiding it: `Timings { total_ms,
   model_init_ms, generate_ms, validate_ms, model_warm }`. Context-level redundant
   init is *avoided* by reusing the persisted session (history is not re-sent);
   model-weight warmth (mmap cache / optional `caro serve`) is explicitly deferred
   to a later ADR.

Reuse, do not duplicate: safety verdicts come from the existing
`safety::SafetyValidator`; backend selection from the existing `cli::CliApp`;
session lifecycle from the existing `ai::store::SessionStore`; risk taxonomy from
the existing `models::RiskLevel`. The envelope is a presentation layer over data we
already compute.

## Consequences

**Positive**
- Machine callers branch on a typed `status` + stable `exit_code` instead of
  parsing text. The contract is greppable and testable.
- `schema_version` decouples the contract from the binary version — the exact gap
  that makes Claude Code's headless output brittle across releases.
- Distinct exit codes for *blocked* vs *backend-unavailable* vs *auth* give scripts
  precise control flow that `claude -p` cannot.
- Determinism is a contract guarantee, not an opt-in flag.
- All new types round-trip through serde, so they are reusable by a future
  `stream-json` mode, the eval harness, and telemetry without redesign.

**Negative / costs**
- One more public type surface in `models` to keep stable; `schema_version` bumps
  now require a deliberate, documented breaking-change process.
- `AiOutcome` gaining `Serialize` couples its field names to the wire format; the
  envelope conversion isolates this but the discipline must be maintained.
- Timing capture adds a few `Instant::now()` calls on the hot path (negligible).

**Neutral**
- `Plain` remains the default output; JSON is opt-in via `-o json`, so existing
  shell-widget behavior (command-only stdout) is unchanged.

## Alternatives considered

1. **Mirror Claude Code's `stream-json` NDJSON event stream now.** Rejected for v1:
   Caro generation is single-shot (no agentic tool loop, no token streaming need),
   so per-event NDJSON is complexity without a consumer. The versioned envelope is
   forward-compatible with adding a stream later.
2. **Emit serialized `AiOutcome`/`GeneratedCommand` directly.** Rejected: these are
   internal types; serializing them directly couples the wire contract to internal
   refactors and lacks `schema_version`, `exit_code`, and a unified `status`.
3. **A persistent `caro serve` daemon to keep the model warm** (the true fix for
   redundant init). Rejected for this ADR: violates the "pure subprocess, no daemon,
   no state" constraint. Deferred to a future ADR; this design surfaces the cost
   (`timings.model_init_ms`, `model_warm`) so the daemon decision is evidence-driven.
4. **A new `src/output/` module.** Rejected: unnecessary. The serializable types
   belong in `models`, and the emit logic is a thin function in `main.rs`/`cli`.
   "No new modules unless unavoidable."
