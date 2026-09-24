# Scope: Multi-Backend Council Generation

**Produced by:** `caro-research--scoping-process` scheduled task, 2026-07-16 (unattended run)
**Decision record:** `docs/adr/005-multi-backend-council-generation.md`
**Feature researched:** Claude Code subagents + experimental Agent Teams

> Feature-selection note (task ran with `[FEATURE NAME]` unfilled): previous runs
> of this task already scoped the structured-output envelope (ADR 003,
> 2026-06-25), stream-json events (ADR 004, 2026-07-01), and lifecycle hooks
> (2026-07-09). A duplication sweep of both ADR series (`001–004` lowercase,
> `ADR-001–036` uppercase) shows sessions (026), permissions (027), streaming
> (028), constrained output (029), degraded results (034), hooks (035/036),
> sandbox (010), and skills/plugins (004) are all covered. **Parallel/fan-out
> generation is the one major agent-CLI capability with no ADR in either
> series**, so this run scoped it.

---

## Phase 1 — Feature research: Claude Code subagents & Agent Teams

**What it solves, for whom.** Developers hit a single-context-window bottleneck:
verbose exploration pollutes the main conversation, and independent subtasks run
serially. Subagents give each worker an isolated context (defined in
`.claude/agents/*.md` with name/description/tools/model frontmatter, spawned via
the Task tool); the parent receives only the final report. Agent Teams
(experimental, `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`) upgrade workers to peers:
own sessions, a shared task list, direct inter-agent messages, a lead that
synthesizes.

**Core architecture.**
- Subagents: parent → Task tool → child with fresh context → single final message
  back as the tool result. No child↔child communication; parent is the only bus.
- Teams: daemon-managed background sessions; coordination via shared task list +
  explicit `SendMessage`; context isolation is preserved — only explicit messages
  cross the boundary. Hierarchical spawning to 3 levels.

**Why experimental / failure modes (as of July 2026).**
1. Teammate sessions **cannot be resumed**; shutdown behavior is flaky.
2. Task-status tracking lags actual work.
3. Results return as **unstructured prose** — no typed result contract, no exit
   codes, nothing a script can branch on without re-parsing natural language.
4. Cost multiplies per worker with no cap.
5. Daemon coordination bugs: background agents inherited a **stale `PATH` from
   the daemon** (missing tools); returning to a session **silently stopped
   subagents and re-ran prompts from scratch**. Both needed July 2026 fixes —
   a bug class that only exists because coordination state lives in a daemon.

**Structured output contract.** There is none at the worker boundary: the
subagent's final message is prose returned as a tool result; teams communicate
via messages + a task list. Headless `claude -p --output-format json` wraps the
*top-level* run only (`result`, `session_id`, `total_cost_usd`); exit codes are
Unix-conventional (0/1/2) with no published global table.

**Session/context lifecycle.** Every subagent starts cold and re-explores the
repo (redundant initialization is a known cost sink); teams avoid re-work only
via explicit messages. No snapshot/warm-start mechanism.

## Phase 2 — Competitive differentiation

**Worth replicating.**
- Isolation with explicit-only information flow (predictable, testable).
- Parallelism over *independent* work.
- Per-worker model choice (cheap model for cheap subtasks) → maps to Caro's
  per-backend cost/latency diversity.

**Their gaps we avoid by designing the schema first.**
- Prose results → Caro returns a typed, ranked `Vec<CouncilCandidate>`.
- Silent worker death → a failed backend is a candidate with
  `error: Some(GeneratorError)`; the schema makes dropping it impossible.
- Daemon-held coordination state (stale env, unresumable sessions) → single
  process, `futures::join_all`, per-backend timeout, no state.
- Unbounded cost → council is an explicit 2–5 backend list + timeout budget.

**Caro's unique positioning.**
- **Safety consensus**: every candidate passes the *same* deterministic
  `SafetyValidator` (52+ patterns). Cross-backend risk disagreement is a signal
  no cloud agent CLI offers.
- **Offline council**: `embedded` + local `ollama` runs air-gapped.
- **Community voice**: `ai-horde` as a free volunteer-cluster council member.
- **Determinism**: rank function is a documented total order, not an LLM judge —
  identical inputs give identical output, which is the whole point of ADR 003.

**Existing infrastructure already covering part of this.**
- `src/backends/mod.rs`: `CommandGenerator` async trait; `CLI_SERVABLE_BACKENDS`
  (embedded, ollama, exo, vllm, mesh, ai-horde, hybrid); serializable
  `GeneratorError`, `BackendInfo`.
- `src/models/mod.rs`: `GeneratedCommand` (Serialize; validated
  `confidence_score` 0.0–1.0; `alternatives`; `generation_time_ms`;
  `backend_used`), `RiskLevel` (ordered, JsonSchema), `SuggestedRouting`.
- `src/safety/mod.rs`: `SafetyValidator::validate_batch`, serializable
  `ValidationResult`.
- ADR 003 `CommandEnvelope` + exit-code table; ADR 004 NDJSON layer (future
  home of per-candidate events).
- `tokio` runtime already drives async generation.

## Phase 3 — Scoped implementation plan

### New types (all in `src/models/mod.rs`; Serialize + Deserialize from day one)

```rust
/// One backend's opinion in a council run. A backend that failed still
/// produces a candidate (command fields None, error Some) — never dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilCandidate {
    pub backend: String,                  // name as passed to --council
    pub command: Option<String>,          // None on backend error
    pub explanation: Option<String>,
    pub risk_level: Option<RiskLevel>,
    pub allowed: Option<bool>,            // SafetyValidator verdict
    pub warnings: Vec<String>,            // from ValidationResult
    pub confidence_score: Option<f64>,
    pub generation_time_ms: Option<u64>,
    pub agreement_count: u32,             // candidates (incl. self) with same normalized command
    pub rank: u32,                        // 0 = winner; deterministic (see ranking)
    pub error: Option<GeneratorError>,    // already Serialize in backends/mod.rs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSummary {
    pub requested: Vec<String>,           // --council list, order preserved
    pub candidates: Vec<CouncilCandidate>,// sorted by rank
    pub winner_index: Option<usize>,      // None when no allowed candidate
    pub unanimous: bool,                  // all non-error candidates share normalized command
    pub timeout_ms: u64,
}
```

Method contracts:
- `CouncilSummary::rank(&mut self)` — applies the documented total order
  (below); pure, deterministic, unit-testable without any backend.
- `CommandEnvelope` (ADR 003) gains `pub council: Option<CouncilSummary>`
  with `#[serde(skip_serializing_if = "Option::is_none")]`. Additive-optional
  ⇒ `schema_version` remains 1; winner mirrors into the existing top-level
  fields so ADR 003 consumers are untouched.

### Ranking (documented total order — the contract, not an implementation detail)

1. `allowed == Some(true)` before everything else; errors sort last.
2. Lower `RiskLevel` first (Safe < Moderate < High < Critical).
3. Higher `agreement_count` (exact match on whitespace-collapsed command text).
4. Higher `confidence_score`.
5. Position in the `--council` list (stable tie-break).

### Orchestration (no new module)

`src/backends/mod.rs`:
```rust
pub async fn run_council(
    generators: Vec<(String, Box<dyn CommandGenerator>)>,
    request: &CommandRequest,
    validator: &SafetyValidator,
    timeout: Duration,
) -> CouncilSummary
```
`join_all` over `tokio::time::timeout(...)`-wrapped `generate_command` calls;
timeout/error ⇒ candidate with `error` set; then `validate_batch` over the
successful commands; then `rank()`. Backend construction reuses the existing
per-name constructors `main.rs` already routes `--backend` through.

### Minimal file-change list

| File | Change |
|---|---|
| `src/models/mod.rs` | Add `CouncilCandidate`, `CouncilSummary`; add optional `council` field to the ADR 003 envelope type |
| `src/backends/mod.rs` | Add `run_council` free function (reuses trait objects + `GeneratorError`) |
| `src/main.rs` | Add `--council`, `--council-timeout-ms` clap flags (validate against `CLI_SERVABLE_BACKENDS`, 2–5 entries, conflicts with `--backend`/`-x`); wire into the generate path; plain-mode table render; exit-code mapping |
| `src/cli/mod.rs` | Thread `Option<CouncilSummary>` through `CliResult` so `-o json`/`yaml` serialize it |
| `docs/headless-contract.md` | Document council fields, ranking order, exit-code interactions |
| `tests/council_contract.rs` | New integration test file (below) |

No new modules; no config-file surface in v1 (flags only, consistent with
ADR 003's bare-by-default rule). `futures = "0.3"` and tokio `time` are already
dependencies — no new crates.

**Dependency note:** `CommandEnvelope` is defined by ADR 003 but **not yet
implemented in code**. If council lands first, `CouncilSummary` ships on
`CliResult` (already `Serialize`, already emitted by `-o json`) and moves into
the envelope when ADR 003 is implemented — the types are envelope-agnostic by
design.

### Exit code / output contract (machines depend on this)

Reuses the ADR 003 table verbatim:

| Code | Council condition |
|---|---|
| 0 | ≥1 allowed candidate; winner in top-level envelope fields; full ranking in `council.candidates` |
| 2 | Usage: unknown backend name, <2 or >5 entries, `--council` with `--backend` or `-x` |
| 3 | All candidates generated but all blocked by the validator |
| 4 | All backends errored/timed out (no candidate has a command) |

Guarantee: `council.candidates.len() == council.requested.len()` always —
per-candidate `error` is how failure is reported, never omission. Stdout carries
exactly one JSON document under `-o json` (modulo `timings`, byte-stable for
identical inputs with deterministic backends).

### Integration tests (`tests/council_contract.rs`; known inputs → deterministic JSON + exit code)

1. **Deterministic council**: two static-matcher-backed generators, query
   "list files" → exit 0; snapshot-assert candidate order, `rank`,
   `agreement_count == 2`, `unanimous == true`.
2. **All blocked**: static council, query resolving to `rm -rf /` template →
   every candidate `allowed == false` → exit 3, `winner_index == null`.
3. **Partial failure**: one real static backend + one ollama pointed at an
   unreachable port → exit 0; failed candidate has `error.kind` populated and
   sorts last; `candidates.len() == 2`.
4. **Total failure**: two unreachable remotes, 500 ms timeout → exit 4;
   both candidates carry timeout errors; wall clock < 2 s (parallelism check).
5. **Usage errors**: `--council embedded` (1 entry) and
   `--council embedded --backend ollama` → exit 2, error envelope per ADR 003.
6. **Ranking is pure**: unit test on `CouncilSummary::rank` with handcrafted
   candidates covering every tie-break level.
7. **Schema stability**: serialize a fixed `CouncilSummary`, snapshot the JSON;
   envelope without `--council` contains no `council` key (additive-optional
   proof).

### Constraint compliance

- **Reuse validator/safety/config**: single `SafetyValidator` via
  `validate_batch`; no per-backend validators; no new config surface.
- **Serializable day one**: both new types derive Serialize+Deserialize;
  `GeneratorError` already does.
- **Pure subprocess**: one process, `join_all`, no daemon, no state files.
- **Phase-1 failure mode solved by design, not workaround**: their workers fail
  silently and report prose; our schema makes every requested backend appear
  exactly once with typed success-or-error, and ranking is a documented pure
  function — there is no coordination state to corrupt and no prose to re-parse.

### Explicitly out of scope (next version)

- Per-candidate NDJSON progress events (belongs to ADR 004's event stream).
- Council inside multiturn `caro ai` sessions (ADR-026 territory).
- `[council]` config-file defaults and weighted voting.
- AST-level command equivalence for agreement (ADR-007 parser is the prior art;
  v1 uses normalized string equality and documents the coarseness).
- LLM judge / semantic rerank.
- Executing the winner (`-x`) directly from a council run.
- Parallel model downloads / prefetch for cold embedded backends (ADR-025).

---

## Sources

- [Claude Code docs — Orchestrate teams of Claude Code sessions](https://code.claude.com/docs/en/agent-teams)
- [Claude Code docs — Run Claude Code programmatically (headless)](https://code.claude.com/docs/en/headless)
- [anthropics/claude-code CHANGELOG](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md)
- [Claude Code Changelog (July 2026) — gradually.ai](https://www.gradually.ai/en/changelogs/claude-code/)
- [Agent Teams shipped in Claude Code 2.1.32 — when they beat subagents](https://charlesjones.dev/blog/claude-code-agent-teams-vs-subagents-parallel-development)
- [Claude Sub Agents and Agent Teams — HatchWorks](https://hatchworks.com/blog/claude/claude-sub-agents-and-agent-teams/)
- [Claude Code Agent Teams: Setup & Usage Guide 2026 — claudefa.st](https://claudefa.st/blog/guide/agents/agent-teams)
