# ADR-006 — Persistent Learned Memory: Deterministic Cross-Invocation Preference & Correction Store

- **Status**: Proposed
- **Date**: 2026-07-17
- **Authors**: `caro-research--scoping-process` (automated scheduled run)
- **Target**: Community / Power Users
- **Builds on / relates to**: ADR-024 (headless JSON envelope + `ExitCode`),
  ADR-025 (init snapshot cache — the lifecycle slot memory loads in),
  ADR-034 (degraded result contract), ADR-017 (local context indexing —
  explicitly *not* this), `src/context/` (`ExecutionContext`),
  `src/config/` (TOML config at `~/.config/caro`),
  `src/backends/hybrid/sanitizer.rs` (PII redaction)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task with `[FEATURE NAME]`
> unbound. Duplication sweep of both ADR series (lowercase `001–005`,
> uppercase `ADR-001–036`) found no coverage of *learned, persistent,
> per-user/per-project memory*. ADR-017 covers semantic indexing of project
> files; ADR-025/026 cover warm-context amortization *within* a run or
> session. Neither carries **user corrections and preferences across
> invocations**. Feature researched: Claude Code **auto memory**
> (experimental) and gemini-cli **save_memory**.
>
> **Not committed** (git-workflow rule: no work on main). To land:
> `bin/sk-new-feature "persistent learned memory"`, move this file into the
> branch, open a PR.

---

## Context

Caro is a one-shot subprocess: every invocation starts from zero. A user who
corrects `caro`'s output ("use `fd`, not `find`"; "always `--dry-run` rsync
first"; "this repo's linter is `npm run lint:fix`") makes the same correction
again tomorrow. The only persistence today is static config
(`~/.config/caro/config.toml`) — hand-written settings, not learned facts.

Claude Code's auto memory shows the demand: a model-curated
`MEMORY.md` index (first 200 lines / 25KB loaded every session) plus lazy
topic files, per-repo directory derived from the git root, plain markdown the
user can audit, on-by-default with settings/env toggles. gemini-cli ships a
simpler explicit `save_memory` tool that appends facts to
`~/.gemini/GEMINI.md`.

Both are experimental/limited for the same reasons, which are the failure
modes this ADR designs against:

1. **Silent truncation.** Content past the load budget is dropped on next
   load; curation depends on the model noticing a reminder.
2. **No structured contract.** Memory is free-form markdown: scripts cannot
   query what is remembered, why, or since when. No provenance, no TTL.
3. **Memory poisoning.** Memories are injected as quasi-instructions every
   session; a wrong or adversarial memory (e.g. written after processing
   untrusted content) persists and steers future sessions. Advisory-only —
   nothing downstream re-validates.
4. **Append-only growth** (gemini-cli): duplicates accumulate; no dedup or
   eviction.

## Decision

Add a **deterministic, schema-first, machine-local memory store** that caro
loads at init and injects into generation as *data, never instructions*.

1. **Storage.** JSON Lines, one `MemoryRecord` per line:
   - global: `~/.config/caro/memory/global.jsonl`
   - per-project: `~/.config/caro/memory/projects/<project-key>/memory.jsonl`,
     `<project-key>` = sha256 of the git repo root (fallback: cwd), matching
     Claude Code's worktree-sharing behavior.
2. **Typed records** (serde, `schema_version` field from day one):
   `kind ∈ {Preference, Correction, ProjectFact, CommandOutcome}`, with
   `id`, `key`, `content`, `source ∈ {ExplicitAdd, AcceptedEdit,
   RejectedCommand, RememberPrefix}`, `created_at`, `last_used_at`,
   `use_count`, `expires_at: Option`.
3. **Deterministic curation, no LLM.** Hard budget (default 64 records/file,
   2KB injected). Eviction is rule-based: expired → lowest
   `use_count` → oldest `last_used_at`. Dedup by `(kind, key)` — newest wins.
   This replaces Claude Code's "model notices the reminder" loop; nothing is
   ever silently truncated because the budget is enforced at *write* time.
4. **Injection as data.** Selected records render into a fenced
   `USER MEMORY (advisory data)` block in the prompt context, alongside
   `ExecutionContext`. Generated commands still pass `SafetyValidator`
   unconditionally — a poisoned memory cannot bypass validation by design.
   Remote/hybrid backends receive memory content only after
   `sanitizer.rs` redaction, reusing the ADR-015 privacy gateway.
5. **Capture is explicit or rule-based, never model-judged**:
   `caro memory add`, a `remember:` prompt prefix, and (when the
   confirmation UI runs) recording user edits to generated commands as
   `Correction` records.
6. **Machine contract.** `caro memory list|add|forget|clear --output json`
   emits stable JSON; the ADR-024 `HeadlessEnvelope` gains an additive
   optional `memory` trace (`records_loaded`, `record_ids_used`). Exit codes
   reuse ADR-024's `ExitCode`. A corrupt memory file is an ADR-034 degraded
   result (warn, continue, `degraded: true`), never a fatal error.
7. **Off switch.** `--no-memory` flag; `[memory] enabled = false` in config;
   `CARO_NO_MEMORY=1` env.

## Consequences

**Positive.** Corrections stop repeating; static-matcher and embedded
backends both benefit (memory is backend-agnostic context); scripts can
audit/export memory; offline-first (no model needed to curate); the failure
modes that keep the competitor feature experimental are closed structurally.

**Negative / risks.** Prompt budget spent on memory (~2KB) slightly reduces
room for other context on small embedded models; rule-based eviction can
drop a rarely-used-but-important record (mitigation: `expires_at: None` +
explicit `caro memory add --pin`); a second file format (JSONL) beside TOML
config; per-record redaction cost on hybrid path.

**Neutral.** Plain-text auditability is preserved (JSONL is greppable); users
who want prose notes keep using config/docs — this store is facts, not docs.

## Alternatives considered

1. **Markdown index + topic files (Claude Code clone).** Human-friendliest,
   but re-imports failure modes 1–2 (unstructured, truncation) and needs a
   capable model to curate — caro must work with 135M-param local models.
2. **Append-only `GEMINI.md`-style file.** Simplest; rejected — duplicate
   accumulation and no eviction is exactly gemini-cli's gap.
3. **SQLite store.** Query power we don't need for ≤64 records; adds a dep
   and breaks trivial user auditing/editing. Rejected for v1.
4. **Reuse the ADR-017 knowledge index (embeddings).** Semantic recall is
   attractive, but drags in embedder + vector store for what is a tiny,
   exact-match dataset; keeps working offline only with extra models.
   Deferred: memory records may *feed* the index later.
5. **Do nothing / config-only.** Users can hand-edit config today; the
   observed behavior (Claude Code shipping auto memory on-by-default) says
   manual maintenance is the pain point itself.
