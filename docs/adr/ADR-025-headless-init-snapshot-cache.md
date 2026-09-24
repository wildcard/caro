# ADR-025: Headless Init Snapshot Cache — Amortizing Cold-Start Across Stateless Subprocess Invocations

- **Status**: Proposed
- **Date**: 2026-06-24
- **Authors**: caro-research scoping process (automated scheduled run)
- **Target**: Hybrid (Community CLI + Enterprise CI)
- **Builds on / relates to**: ADR-024 (Headless JSON/NDJSON output contract),
  ADR-015 (MCP safety server — the daemon complement), ADR-017 (local context
  indexing), ADR-022 (caro safety library API)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound. The prior run (2026-06-23) already scoped the
> *output contract* for headless mode as **ADR-024**. To avoid duplicating it,
> this run targets the one dimension the task emphasizes that ADR-024 only
> hand-waves: **"session/context lifecycle management — how does it avoid
> redundant initialization?"** The competitor analog remains Claude Code's
> headless mode, but the specific feature studied here is its **warm-context /
> session-resume** behavior and the cold-start cost a pure subprocess pays
> without it. Treat the analog choice as a reviewable assumption.

---

## Context

### The problem and who has it

ADR-024 gives caro a deterministic, machine-callable headless mode: one
`caro --output json "<prompt>"` subprocess in, one versioned JSON envelope out,
a stable exit code, no daemon. That is correct and complete *as a contract*.
What it does not address is **cost amortization across repeated calls**.

The three audiences ADR-024 names — CI pipelines, agent wrappers, and `jq`
scripts — almost never call caro once. They call it in a loop:

1. A **pre-commit / CI** hook that runs `caro --output json` over every changed
   command in a script, or over every line of a generated Makefile — tens to
   hundreds of invocations per run.
2. An **agent framework** that wraps caro as a tool and calls it once per
   planning step — a multi-step task is N subprocess spawns.
3. A **`xargs`/`parallel`** pipeline assessing a directory of shell snippets.

Each of those invocations, under ADR-024's "bare by default" design, pays the
**full cold-start cost every time**, because a pure subprocess keeps nothing
warm between calls.

### What cold-start actually costs in caro (measured against the code)

The Phase-1/Phase-2 inventory located three concrete per-process initialization
costs, all paid again on every headless invocation:

| Init cost | Where | Why it is not free |
| --- | --- | --- |
| **Platform probing** | `src/platform/mod.rs::PlatformContext::detect()` (async) | Spawns subprocesses: `which <tool>` per probed utility with a **500 ms timeout each** (`run_command_with_timeout`, L640), plus `Command::new(...).output()` (L690) for utility-flavor detection. Serial probing of a handful of tools dominates wall-clock for a fast static-backend run. |
| **Safety pattern compilation** | `src/safety/patterns.rs` (`DANGEROUS_PATTERNS`, `COMPILED_PATTERNS`) + `src/safety/cve_patterns.rs` (`CVE_COMPILED`) | Compiled via `once_cell::Lazy` — "compiled once at startup … 30× speedup" (`safety/mod.rs:10`). But "once" means **once per process**. 52+ dangerous patterns + the CVE ruleset are recompiled on every fresh subprocess. The `Lazy` cache dies with the process. |
| **Backend availability probe** | `CommandGenerator::is_available()` (`src/backends/mod.rs`) | For remote/embedded backends this is a network or model-load check. The embedded backend additionally pays model load (hundreds of ms to seconds). |

`PlatformContext::detect()` has **no caching today** — every call re-probes.
There is no `OnceLock`/`Lazy` around it and no file-backed snapshot.

### The competitor's answer — and why we can't copy it

Claude Code makes repeated headless calls cheap two ways, both of which keep
**state between calls**:

- `--continue` / `--resume <session_id>`: the next `claude -p` reuses a prior
  session's loaded context instead of rebuilding it. ([Claude Code headless
  docs](https://code.claude.com/docs/en/headless))
- A warm agent process / SDK object that holds the tool surface, MCP
  connections, and context window in memory across turns.

Both rely on **resident state** — exactly what caro's constraints forbid
("pure subprocess call, no daemon, no state"). caro cannot resume a warm
process because there is no process to resume.

### The failure mode we must design around

> **Redundant initialization on every stateless invocation.** A pure subprocess
> that re-detects the platform (subprocess spawns + 500 ms timeouts) and
> recompiles 52+ safety regexes on every call turns a 100-command CI sweep into
> 100× the unavoidable per-command work. The naïve fixes both violate
> constraints: a daemon (forbidden) or "just make detect() a process-global
> `Lazy`" (no help — each subprocess is a new process, so the `Lazy` is cold
> every time).

ADR-024 chose determinism by *skipping* discovery. ADR-025 must make the work
that **remains** (platform facts, pattern-set identity, backend reachability)
**survive across process boundaries without a resident process** — and do so
without reintroducing the ambient-state nondeterminism ADR-024 eliminated.

---

## Decision

Introduce a **file-backed, content-addressed Init Snapshot Cache**: the
stateless analog of session-resume. The deterministic, slow-to-compute parts of
initialization are serialized once to a snapshot file keyed by a digest of the
inputs that determine them. The *next* subprocess reads the snapshot (one file
read + deserialize) instead of re-probing — staying a pure subprocess, keeping
nothing running, and never changing the command or safety verdict it produces.

Four design commitments, each closing the failure mode or a constraint by
construction:

### 1. Content-addressed key ⇒ self-invalidating (no stale reads by design)

The cache key is `sha256` over the **inputs that change initialization**:

```
key = sha256(
    caro_version           ||  # binary identity ⇒ pattern set + detect logic
    target_triple (os/arch) ||
    shell                  ||
    PATH_fingerprint       ||  # hash of $PATH string (tool surface can change)
    pattern_set_version        # bumped when DANGEROUS_PATTERNS/CVE rules change
)
```

If anything that would change the snapshot changes, the key changes, so a stale
snapshot is simply *never looked up* — it is not invalidated, it is unaddressed.
This is the same content-addressing discipline `src/cache/checksum.rs` already
uses for model files (`sha2` is already a dependency). It is the structural
answer to the determinism hazard: the cache cannot serve a snapshot that
disagrees with the current binary's behavior.

### 2. Short TTL backstop ⇒ catches the few inputs not in the key

A tool installed into an existing `PATH` entry does not change `$PATH`, so the
key would not move. A **default 60 s TTL** (`created_at` checked against now)
bounds that window: a tight CI/agent loop hits the warm snapshot; a human
returning minutes later re-probes. TTL is configurable
(`--init-cache-ttl <secs>`, `0` disables the cache entirely).

### 3. Optimization-only ⇒ never on the correctness or exit-code path

A snapshot **miss, a corrupt file, a permission error, or a race** all fall
back to a full cold `detect()` — silently, exit code unchanged. The cache can
*only* make a run faster; it can never make a run fail or change its output.
This is the load-bearing safety property: machines depending on ADR-024's
exit-code/JSON contract see **identical** results warm or cold. (Enforced by
integration tests 4 and 7 below.)

### 4. Atomic write + lock-free read ⇒ safe under CI parallelism

CI sweeps and `parallel` pipelines spawn many caro processes at once into one
cache dir. Writes use **temp-file + `rename(2)`** (atomic on POSIX; the same
`.part`→`rename` pattern `src/cache/download.rs` already uses, L154). Readers
never lock: they read, and on any parse error
fall through to cold init (commitment 3). A torn write is impossible (rename is
atomic); a partially-written temp is never addressed.

### 5. Observable in the contract (additive to ADR-024)

The envelope and the NDJSON `Init` event gain an **optional** `cache` block so
CI can assert warm-vs-cold and measure savings. Additive-only under
`schema_version: "1"` — exactly the evolution ADR-024 promised. Absent the flag
or on opt-out, the field is omitted (`Option` + `skip_serializing_if`), so
existing parsers are unaffected.

---

## New types

All snapshot/cache types live in a **submodule of the existing cache module** —
`src/cache/init_snapshot.rs`, re-exported from `src/cache/mod.rs`. This honors
"no new top-level modules": `cache/` already owns content-addressed,
file-backed, atomically-written local state. The contract-facing `cache` block
lives next to ADR-024's types in `src/cli/mod.rs`. All types derive
`#[derive(Debug, Clone, Serialize, Deserialize)]` from day one; the
contract-facing one also derives `schemars::JsonSchema` (already a dep).

### `InitSnapshot` (in `src/cache/init_snapshot.rs`)

```rust
/// The serialized, cacheable result of cold initialization. Everything in here
/// is deterministic given the cache key; nothing here is prompt-dependent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitSnapshot {
    /// Snapshot format version (independent of the ADR-024 contract version).
    pub snapshot_version: u32,
    /// The caro binary that wrote it (redundant with the key; kept for debug).
    pub caro_version: String,
    /// Unix-millis write time; compared against TTL on read.
    pub created_at: i64,
    /// The expensive-to-detect platform facts. Reused verbatim — no re-probe.
    pub platform: PlatformContext,
    /// Identity of the compiled safety pattern set this snapshot is valid for.
    pub pattern_set_version: String,
    /// Last-known backend reachability, keyed by backend id (advisory only —
    /// a stale "available" is re-checked lazily by the backend itself).
    pub backend_reachable: std::collections::BTreeMap<String, bool>,
}
```

### `InitCacheKey` (in `src/cache/init_snapshot.rs`)

```rust
/// Newtype over the hex sha256 digest, formatted "sha256:<hex>".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitCacheKey(pub String);
```

### `InitCacheStatus` (contract-facing, in `src/cli/mod.rs`)

```rust
/// Additive `cache` block on the ADR-024 envelope / Init event. Optional:
/// omitted entirely when the init cache is disabled.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
pub struct InitCacheStatus {
    pub hit: bool,
    /// "sha256:<hex>" — lets CI group warm runs and detect key churn.
    pub key: String,
    /// Age of the served snapshot in ms; None on a miss/cold run.
    pub age_ms: Option<u64>,
    /// "warm" | "cold" | "cold_fallback" (corrupt/expired ⇒ cold_fallback).
    pub source: CacheSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheSource { Warm, Cold, ColdFallback }
```

### Method contracts

In `src/cache/init_snapshot.rs`:

- `InitCacheKey::compute(caro_version: &str, target: &str, shell: &str, path: &str, pattern_set_version: &str) -> InitCacheKey`
  — pure function, no I/O; the *only* place the key is defined.
- `InitSnapshot::load_fresh(dir: &Path, key: &InitCacheKey, ttl: Duration, now_ms: i64) -> Option<InitSnapshot>`
  — returns `Some` only if the file exists, parses, *and* `now - created_at <= ttl`.
  Any I/O or parse error returns `None` (never propagates — commitment 3).
- `InitSnapshot::store_atomic(&self, dir: &Path, key: &InitCacheKey) -> io::Result<()>`
  — writes `<dir>/<hex>.json` via temp + `rename`. Errors are logged to stderr
  and swallowed by the caller (a write failure must not fail the run).
- `warm_or_cold(dir, key, ttl, now_ms, cold: impl FnOnce() -> InitSnapshot) -> (InitSnapshot, CacheSource)`
  — the single entry point: `load_fresh` → on `None`, call `cold()`, `store_atomic`,
  return `Cold`; a parse-failure path returns `ColdFallback`.

In `src/cli/mod.rs`:

- `InitCacheStatus::from(source: CacheSource, key: &InitCacheKey, age_ms: Option<u64>) -> Self`.

---

## Minimal set of files that change

No new top-level modules. One new submodule file inside the existing `cache/`
module, plus edits to five existing files and two test/doc artifacts:

1. **`src/platform/mod.rs`** — add `#[derive(Serialize, Deserialize)]` to
   `PlatformContext`, `UtilityType`, and `BsdFlavor` (currently `Debug, Clone`
   only). Fields are private with public getters and a `PlatformContextBuilder`,
   so serde round-trips cleanly without exposing internals. This is the
   "serializable from day one" requirement applied to a previously
   non-serializable type — and the only change to platform logic (detection is
   untouched; the snapshot reuses `detect()`'s output).
2. **`src/cache/init_snapshot.rs`** *(new submodule, not a new module tree)* —
   the four types + impls above. Reuses `sha2`, `serde_json`, `directories`
   (all existing deps).
3. **`src/cache/mod.rs`** — `mod init_snapshot; pub use init_snapshot::{InitSnapshot, InitCacheKey};`
4. **`src/cli/mod.rs`** — add `InitCacheStatus` + `CacheSource`; add an
   optional `cache: Option<InitCacheStatus>` field to ADR-024's
   `HeadlessEnvelope` (with `#[serde(skip_serializing_if = "Option::is_none")]`)
   and to `HeadlessEvent::Init`.
5. **`src/main.rs`** — in the headless init path, compute the key, call
   `warm_or_cold`, and use the resulting `PlatformContext` instead of an
   unconditional `detect()`. Add flags `--no-init-cache` and
   `--init-cache-ttl <secs>` to `Cli`. Populate `envelope.cache`.
6. **`docs/headless-contract.md`** *(extends the ADR-024 doc)* — document the
   additive `cache` block, the key inputs, the TTL, `--no-init-cache`, and the
   "optimization-only, never changes output/exit code" promise.
7. **`tests/headless_init_cache.rs`** *(new integration test file)* — below.
8. **`docs/adr/README.md`** — add the ADR-025 table row.

Reused **without modification**: `PlatformContext::detect()`, the entire
`safety/` pattern infrastructure, `CommandGenerator::is_available`,
`download.rs`'s `.part`→`rename` atomic-write pattern, ADR-024's `HeadlessEnvelope` /
`HeadlessEvent` / `ExitCode`, and the `StaticMatcher` backend (for deterministic
tests).

---

## Exit-code / output contract (what machines depend on)

**The init cache changes no exit code and removes no field.** It is purely
additive to ADR-024:

- Exit codes 0–6 keep ADR-024's exact meanings. A cache hit, miss, corrupt
  snapshot, or write failure never alters the exit code.
- `schema_version` stays `"1"` — `cache` is an *added optional* field, which
  ADR-024 explicitly permits within a version.
- New observable surface for scripts:
  - `.cache.hit` (bool) — assert a CI loop is actually warming.
  - `.cache.key` (string) — detect unexpected key churn (e.g. a `PATH` that
    differs per CI shard, defeating the cache).
  - `.cache.source` — `"warm"` / `"cold"` / `"cold_fallback"`; alert on
    sustained `cold_fallback` (corrupt cache dir).
- New flags (documented, stable): `--no-init-cache` (force cold; `cache` field
  omitted) and `--init-cache-ttl <secs>` (`0` ≡ `--no-init-cache`).

Snapshot files live under `$XDG_CACHE_HOME/caro/init/<hex>.json` (via the
existing `directories` crate). The directory is disposable: deleting it only
forces the next run cold.

---

## Integration tests (deterministic input → fixed JSON + exit code)

In `tests/headless_init_cache.rs`, driven through the `static` backend
(offline, deterministic; no model download) with `$XDG_CACHE_HOME` pointed at a
`tempfile::TempDir`:

1. **Cold → warm.** Run `caro --backend static --output json "list files"`
   twice in the same temp cache dir. Assert run 1 `cache.source == "cold"`,
   `cache.hit == false`; run 2 `cache.hit == true`, `cache.source == "warm"`,
   identical `cache.key`; and the two envelopes are **byte-identical except the
   `cache` block** (proves warm path changes nothing else). Both exit `0`.
2. **Key invalidation on `PATH` change.** Run once; then run again with a
   mutated `$PATH`. Assert `cache.key` differs and the second run is `cold`
   (proves content-addressing self-invalidates).
3. **TTL expiry.** Run with `--init-cache-ttl 0`-adjacent small value, sleep
   past it, re-run. Assert second run `cold` despite a present snapshot file
   (proves the TTL backstop).
4. **Corrupt-snapshot fallback.** Write `"{ garbage"` into the snapshot file for
   the computed key, then run. Assert exit `0`, `cache.source ==
   "cold_fallback"`, and the command + `safety` verdict equal the clean cold run
   (proves the optimization-only / never-fail property).
5. **Concurrency / atomic write.** Spawn N parallel invocations into a clean
   cache dir; assert none crash, all exit `0`, and the final snapshot file
   parses as a valid `InitSnapshot` (proves temp+rename atomicity).
6. **Opt-out.** Run with `--no-init-cache`; assert no `*.json` is created in the
   cache dir and the `cache` field is absent from the envelope.
7. **Output invariance (the contract guarantee).** For a fixed prompt, assert
   `command`, `safety`, `risk_level`, and `exit_code` are identical across
   {cold, warm, cold_fallback}. This is the test that lets downstream `jq`
   pipelines trust the cache is invisible to them.

Every test asserts the parsed envelope **and** the process exit code so they
cannot drift.

---

## Out of scope (next version / other ADRs)

- **Caching inference / generation results** (memoizing a prompt → command).
  Different cache, different invalidation (prompt + model + context), different
  correctness risk. ADR-025 caches only *deterministic init metadata*, never a
  model output.
- **Keeping the embedded LLM warm in memory** across calls. That is a resident
  process = a daemon = forbidden. The model *weights* are already content-cached
  on disk by `src/cache/`; ADR-025 does not touch inference state.
- **Cross-machine / shared snapshot** (NFS, CI artifact cache restore). v1 is a
  local user-cache-dir optimization. A portable, signed snapshot for CI cache
  layers is a clean v2 follow-on (the `snapshot_version` field is the hook).
- **Warming interactive (TUI) mode** from the same snapshot. Plausible and
  additive, but interactive start-up is not the audience here; scoped out to
  keep the change minimal.
- **Partial-message / token streaming** and **`--json-schema` enforcement** —
  already out-of-scope per ADR-024; unchanged here.

---

## Consequences

### Benefits

- Turns the per-call cold-start (subprocess `which` probing + 52+ regex
  recompiles) into a one-file read for every call after the first in a loop —
  the exact CI/agent/`xargs` access pattern ADR-024 targets.
- Solves the "redundant initialization" failure mode **without a daemon and
  without resident state**, the stateless analog of Claude Code's
  session-resume.
- Self-invalidating by content-addressing: no manual cache-busting, no "stale
  platform facts" class of bug — the determinism property ADR-024 won is
  preserved, not eroded.
- Zero new dependencies (`sha2`, `serde_json`, `directories`, `once_cell` all
  present); ~3 small types + one submodule + tests.

### Trade-offs

- Adds a writable cache directory to caro's footprint (disposable; deleting it
  is always safe).
- `PlatformContext` becomes part of a serialized format, so its field shape now
  has a (snapshot-versioned) compatibility surface — mitigated by
  `snapshot_version` and by the cold-fallback-on-parse-error rule.
- The 60 s TTL is a heuristic; a tool installed mid-window into an existing
  `PATH` entry is briefly invisible. Documented; tunable; `0` disables.

### Risks

- **Risk:** a poisoned/world-writable shared cache dir serves a malicious
  snapshot. → **Mitigation:** snapshot influences only platform *facts*, never
  the safety verdict or the command text (commitment 3 + test 7); store under
  the user-private `$XDG_CACHE_HOME`; on any deserialization anomaly, cold-fall
  back. A future signed-snapshot mode (v2) hardens the shared-CI case.
- **Risk:** per-shard `PATH` differences in CI silently defeat the cache. →
  **Mitigation:** `.cache.key` is exposed so CI can alert on key churn (test 2
  documents the behavior).

## Alternatives considered

1. **Process-global `Lazy`/`OnceLock` around `detect()`.** Rejected: each
   headless call is a *new process*; a process-global cache is cold on every
   invocation. It helps a long-lived TUI, not the subprocess loop this ADR
   targets.
2. **A `caro serve` / warm daemon (copy session-resume directly).** Rejected:
   violates "pure subprocess, no daemon, no state." Daemon territory already
   belongs to the MCP safety server (ADR-015); ADR-025 is its stateless
   complement, mirroring how ADR-024 framed the same split.
3. **Bake a snapshot into the binary at build time.** Rejected: platform facts
   (`$PATH`, installed tools, shell) are inherently host- and run-specific; a
   compile-time snapshot would be wrong on the first run on any new host.
4. **Make the cache mandatory and load-bearing (read facts only from cache).**
   Rejected: couples correctness to a disposable file. Keeping the cache
   optimization-only (always able to cold-fall back) is what preserves ADR-024's
   contract guarantees — this is the "solve the failure mode by design, not by
   workaround" requirement.

## References

- [Claude Code — Run Claude Code programmatically (headless)](https://code.claude.com/docs/en/headless)
  — `--continue` / `--resume` warm-context behavior studied as the analog.
- ADR-024 (Headless JSON/NDJSON output contract) — the contract this extends
  additively.
- ADR-015 (MCP safety server) — the resident-process complement; daemon
  concerns live there, not here.
- `src/platform/mod.rs` — `PlatformContext::detect()`, the `which`/`Command`
  probing this snapshots.
- `src/safety/patterns.rs`, `src/safety/cve_patterns.rs` — the per-process
  `once_cell::Lazy` pattern compilation this amortizes.
- `src/cache/checksum.rs`, `src/cache/manifest.rs` — existing content-addressed,
  atomically-written cache infra reused here.
- `.claude/rules/adr-numbering.md` — sequential numbering (this is ADR-025,
  following ADR-024).
- `.claude/rules/validation-discipline.md` — note: this ADR is an internal
  performance/contract refinement of an already-GA path, not a new user-facing
  product line, so the five discovery gates do not gate it (per that rule's
  "what this rule does NOT do" clause).
