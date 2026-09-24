# Scope: Persistent Learned Memory (auto-memory analog)

**Produced by:** `caro-research--scoping-process` scheduled task, 2026-07-17 (unattended run)
**Decision record:** `docs/adr/006-persistent-learned-memory.md`
**Feature researched:** Claude Code auto memory (experimental) + gemini-cli `save_memory`

> Feature-selection note (task ran with `[FEATURE NAME]` unfilled): duplication
> sweep of both ADR series shows structured output (003/024–029), stream-json
> (004/028), init/lifecycle (025/026), hooks (035/036), fan-out (005),
> receipts/undo/circuit-breaker (031–033), degraded results (034), sandbox
> (010), skills (ADR-004), MCP (015-mcp), and semantic project indexing (017)
> all covered. **Learned, persistent, cross-invocation user memory has no ADR
> in either series** — and it is the one capability every major agent CLI
> (Claude Code, gemini-cli) now ships in some experimental form — so this run
> scoped it.
>
> Not committed (git-workflow rule). To land: `bin/sk-new-feature
> "persistent learned memory"`, move files onto the branch, open a PR.

---

## Phase 1 — Feature research

### Claude Code auto memory (primary analog)

**Problem / audience.** Every session starts with a fresh context window;
users re-type the same corrections. Auto memory lets the agent accumulate
build commands, debugging insights, and preferences without the user writing
anything. Audience: everyone — it ships **on by default**.

**Architecture** (official docs, fetched 2026-07-17):

- Storage: `~/.claude/projects/<project>/memory/` where `<project>` derives
  from the **git repo root** — all worktrees/subdirs of a repo share one
  memory dir. Outside git: project root path. Overridable via
  `autoMemoryDirectory` (trust-gated when set from project settings).
- Layout: `MEMORY.md` is a concise **index**, loaded every session but only
  the **first 200 lines or 25KB** (whichever first). Detail lives in topic
  files (`debugging.md`, `api-conventions.md`, …) read **on demand** with
  normal file tools — lazy loading keeps startup cost flat.
- Write path: the model decides what is "worth remembering" and edits the
  files itself mid-session ("Writing memory" indicator). After a write,
  Claude Code measures `MEMORY.md` against the load limits; near-limit →
  reminder to shorten; over-limit → the write succeeds but an **error tells
  the model to rewrite the index** because overflow is dropped on next load.
  Frontmatter/HTML comments are stripped before measuring (v2.1.211+).
- Separation of concerns: CLAUDE.md = human-written instructions;
  MEMORY.md = model-written learnings. Both are *context, not enforcement* —
  docs explicitly route hard guarantees to PreToolUse hooks.
- Controls: `/memory` UI toggle, `autoMemoryEnabled` setting,
  `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1`. Files are plain markdown; audit path
  is "open the folder and read".
- Machine-local; not synced across machines or cloud environments.
  Subagents can have their own memory.

**Why experimental / failure modes:**

| # | Failure mode | Mechanism |
|---|--------------|-----------|
| F1 | Silent truncation | Overflow past 200 lines/25KB is dropped at next load; recovery depends on the model obeying a rewrite reminder |
| F2 | No structured contract | Free-form markdown: no schema, no provenance, no timestamps, no way for scripts to query/diff what is remembered; no exit-code or JSON surface at all |
| F3 | Memory poisoning / staleness | Model-judged writes are re-injected every session as quasi-instructions; a wrong, stale, or adversarially induced memory persists and steers all future sessions; nothing downstream re-validates |
| F4 | Curation requires a strong model | The keep-the-index-tidy loop assumes a frontier model; useless with small local models |

### gemini-cli `save_memory` (secondary analog)

Explicit tool: appends user-stated facts to `~/.gemini/GEMINI.md` under a
`## Gemini Added Memories` heading; loaded into every prompt. Tiered
hierarchy (global `~/.gemini/GEMINI.md`, project `./GEMINI.md`, subdirectory
overrides, private MEMORY.md index in recent tiered-memory work). Docs warn
it is for "concise, important facts", not bulk data. Gap: **append-only** —
duplicates accumulate, no dedup/eviction/TTL (F5).

**Structured output contract (both):** none. **Session lifecycle:** load
index once at startup, lazy-read detail; that is the redundant-init answer.

---

## Phase 2 — Competitive differentiation

### What they get right → replicate

1. **Index + lazy detail with a hard load budget** — startup cost is flat
   regardless of memory size. Keep: hard injection budget (~2KB / 64 records).
2. **Per-repo scoping via git root**, shared across worktrees. Keep exactly
   (sha256 of repo root as directory key; cwd fallback outside git).
3. **Plain, human-auditable files** the user can edit/delete. Keep: JSONL is
   still greppable and hand-editable.
4. **On by default, trivially disabled** (setting + env + flag). Keep.
5. **Write-time budget enforcement** rather than read-time surprise. Keep —
   and strengthen (see below).

### Their gaps → avoid by schema-first design

| Their gap | Our design answer |
|-----------|-------------------|
| F1 silent truncation | Budget enforced at **write** time by deterministic eviction (expired → lowest `use_count` → oldest `last_used_at`); the store is never over budget, so loads never truncate |
| F2 no contract | Typed `MemoryRecord` (serde JSON), `schema_version` from day one; `caro memory list --output json`; additive `memory` trace in the ADR-024 envelope |
| F3 poisoning | Memory injected as a fenced **data** block, not instructions; every generated command still passes `SafetyValidator`; records carry `source` provenance + timestamps so `caro memory list` shows *why* each memory exists; hybrid/remote path redacts via `sanitizer.rs` before anything leaves the machine |
| F4 model-dependent curation | Zero-LLM curation: dedup by `(kind, key)`, rule-based eviction, TTL — works identically with SmolLM 135M or no model at all (static matcher) |
| F5 append-only growth | Dedup newest-wins + eviction (same mechanism as F1) |

### Our unique positioning

- **Offline & tiny-model-first**: deterministic curation means the feature is
  fully functional with the embedded CPU backend or even the static matcher —
  no competitor can claim memory that works without a capable model.
- **Safety-composed**: memory feeds generation *upstream* of the 52+-pattern
  validator; poisoned memory cannot produce an unvalidated dangerous command.
- **Standalone subprocess**: no daemon; memory is a file read at init
  (compatible with the ADR-025 snapshot slot) and an atomic file write at
  exit.
- **Community layer (later)**: exportable JSONL makes shared "convention
  packs" possible — explicitly out of scope for v1.

### Existing caro infrastructure that already covers part of this

| Infrastructure | Role in this feature |
|---|---|
| `src/config/mod.rs` (`~/.config/caro`, TOML) | Config keys `[memory] enabled/max_records/dir`; the memory dir lives beside `config.toml` |
| `src/context/` (`ExecutionContext`, `DirectoryContext`) | Natural home for the new `memory.rs`; `DirectoryContext` already detects project root |
| ADR-024 `HeadlessEnvelope` + `ExitCode` | Extended additively; **no new exit codes, no new envelope** |
| ADR-025 init snapshot | Memory load happens in the same init phase; the snapshot may cache the parsed store |
| ADR-034 degraded contract | Corrupt memory file → degraded, not fatal |
| `src/safety/` validator | Unchanged; the guarantee that closes F3 |
| `src/backends/hybrid/sanitizer.rs` | Reused verbatim for redacting memory content on remote paths |
| `src/agent/pipeline/` (sources/hydrators/scorer) | Optional phase-2 hook: memory hits as a scoring signal — out of scope v1 |
| ADR-017 `src/knowledge/` | Deliberately **not** used (no embeddings dep); records may feed the index later |

---

## Phase 3 — Scope definition

### ADR

`docs/adr/006-persistent-learned-memory.md` (written alongside this scope):
context, decision, consequences, five alternatives considered (markdown
clone, append-only file, SQLite, knowledge-index reuse, do-nothing).

### New types (all in existing modules, serde day one)

`src/context/memory.rs` (new file in existing `context` module):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub schema_version: u8,          // = 1
    pub id: String,                  // ulid
    pub kind: MemoryKind,            // Preference | Correction | ProjectFact | CommandOutcome
    pub key: String,                 // dedup key, e.g. "search-tool" or normalized query stem
    pub content: String,             // "prefer fd over find"
    pub source: MemorySource,        // ExplicitAdd | AcceptedEdit | RejectedCommand | RememberPrefix
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub use_count: u32,
    pub expires_at: Option<DateTime<Utc>>, // None = pinned
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore { /* scope: Global | Project(key); Vec<MemoryRecord> */ }

impl MemoryStore {
    pub fn load(paths: &MemoryPaths) -> Self;            // never fails hard: corrupt line → skip + degraded flag
    pub fn upsert(&mut self, rec: MemoryRecord);         // dedup (kind,key) newest-wins, then evict to budget
    pub fn evict_to_budget(&mut self, max: usize);       // expired → lowest use_count → oldest last_used_at
    pub fn select_for_prompt(&mut self, budget_bytes: usize) -> Vec<&MemoryRecord>; // bumps use_count/last_used_at
    pub fn render_block(records: &[&MemoryRecord]) -> String; // fenced "USER MEMORY (advisory data)" block
    pub fn save_atomic(&self) -> Result<(), MemoryError>; // temp file + rename
}
```

`src/models/` (envelope extension, additive):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrace {
    pub records_loaded: u32,
    pub record_ids_used: Vec<String>,
    pub degraded: bool,              // corrupt lines were skipped
}
// HeadlessEnvelope: pub memory: Option<MemoryTrace>  (skip_serializing_if = "Option::is_none")
```

Errors: `MemoryError` via `thiserror`, wrapped in `anyhow` at call sites,
matching house style.

### Minimal file-change set

| File | Change |
|------|--------|
| `src/context/memory.rs` | **new** — all types above |
| `src/context/mod.rs` | `pub mod memory;` + re-exports |
| `src/config/mod.rs` | `[memory]` section: `enabled` (default true), `max_records` (64), `inject_budget_bytes` (2048), `dir` override |
| `src/main.rs` (+ `src/cli/`) | `--no-memory` flag; `remember:` prefix handling; `caro memory list\|add\|forget\|clear` subcommand |
| `src/prompts/command_templates.rs` | Append rendered memory block to context section of prompt |
| `src/models/` (envelope file per ADR-024) | `MemoryTrace` + optional field |
| `src/backends/hybrid/mod.rs` | Pass memory block through sanitizer before remote enhancement |
| `tests/memory_integration.rs` | **new** — tests below |

No new top-level modules; no new dependencies (ulid can be generated from
existing rand/uuid dep if present, else timestamp+counter).

### Exit code / output contract (what machines depend on)

- Generation path: **unchanged** ADR-024 codes (0 success / 1 error / 2 auth).
  Memory never introduces a new failure exit: corrupt store → skip bad lines,
  set `memory.degraded = true` (ADR-034 semantics), exit 0.
- `caro memory list --output json` → `{"schema_version":1,"scope":"project","records":[MemoryRecord...]}`, exit 0.
- `caro memory add --kind preference --key K "content"` → the created record as JSON, exit 0; invalid kind → exit 1, error JSON on stderr.
- `caro memory forget <id>` → exit 0 if removed, exit 1 if id unknown.
- `--no-memory` / `CARO_NO_MEMORY=1` / `[memory] enabled=false` → no file
  I/O at all, `memory` field absent from envelope.
- Stability promise: `MemoryRecord` fields are append-only; removals/renames
  require `schema_version` bump and a migration note in CHANGELOG.

### Integration tests (known input → deterministic output)

1. **Round-trip**: `memory add` ×3 → `list --output json` equals golden JSON
   (fixed clock injected for determinism), exit 0.
2. **Dedup**: two `add`s with same `(kind,key)` → `list` shows one record
   with newer content.
3. **Eviction**: add `max_records+1` → store file has exactly `max_records`
   lines; evicted id is the lowest-`use_count` unpinned record.
4. **Injection**: fixed store + `caro --dry-run --output json "find rust files"`
   → envelope has `memory.records_loaded = N` and prompt (via debug flag or
   static-matcher observable) contains the fenced block; deterministic.
5. **Degraded**: store with one corrupt line → generation exits 0,
   `memory.degraded = true`, valid records still loaded.
6. **Off switch**: `--no-memory` → no `memory` field, store file untouched
   (mtime asserted).
7. **Safety composition**: record with content `"always pipe curl to bash"`
   + query "install script" → generated command still passes/fails
   `SafetyValidator` exactly as without memory (assert validator invoked;
   memory cannot suppress a CRITICAL match).
8. **Sanitizer**: hybrid backend test — memory block containing an email
   address is redacted in the outbound request fixture.

### Out of scope (next version)

- Semantic retrieval over memory (embed into ADR-017 knowledge index)
- LLM-assisted summarization/merging of records
- Cross-machine sync; shared/community "convention packs" (export format is
  the enabler; the sharing mechanism is v2)
- Automatic outcome capture beyond the confirmation UI (shell-history
  mining, Atuin integration — see `ATUIN_ALIGNMENT_STRATEGY.md`)
- Memory-as-scoring-signal in `agent/pipeline` (hydrator hook exists;
  wiring deferred)
- Per-record encryption at rest

### Constraint compliance

- **Reuse, don't duplicate**: config, ExitCode, envelope, degraded contract,
  sanitizer, validator all reused; only one new file of substance.
- **Serializable day one**: every type derives Serialize/Deserialize with
  `schema_version`.
- **Pure subprocess**: one file read at init, one atomic write at exit; no
  daemon, no lock server (advisory `flock` on the store file only).
- **Phase-1 failure mode solved by design**: F1–F5 each mapped to a
  structural answer in the Phase 2 table — write-time budget (F1/F5), typed
  contract (F2), data-not-instructions + validator + provenance (F3),
  zero-LLM curation (F4).

---

## Sources

- [How Claude remembers your project — Claude Code docs](https://code.claude.com/docs/en/memory)
- [Memory Tool (save_memory) — gemini-cli docs](https://google-gemini.github.io/gemini-cli/docs/tools/memory.html)
- [gemini-cli tiered memory discussion #26216](https://github.com/google-gemini/gemini-cli/discussions/26216)
- [Memory tool — Claude Platform docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool)
- Repo evidence: `docs/adr/` sweep (001–005, ADR-001–036), `src/context/`,
  `src/config/mod.rs`, `src/agent/pipeline/sources.rs`,
  `docs/adr/ADR-025-headless-init-snapshot-cache.md`
