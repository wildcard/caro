# ADR-017 — Local Context Indexing: Project-Aware Command Generation

**Status**: Proposed

**Date**: 2026-06-04

**Authors**: `caro-research--scoping-process` (automated research/scoping agent)

**Target**: Community / Power Users

**Hypothesis ID**: `local-context-indexing`

**Relates to**: ROADMAP.md v2.0.0 `Local Context Indexing`, ADR-016 (self-healing), `src/knowledge/` (LanceDB/ChromaDB backends), `src/context/` (ExecutionContext, DirectoryContext), `docs/discovery/v2.0-validation-audit.md`

---

## Context

Caro generates POSIX shell commands from natural language. Today the
generator receives:

- `ExecutionContext` — OS, arch, shell, cwd, available commands
- `DirectoryContext` — project type (Rust/Node/Python/…), Makefile presence,
  Docker presence, npm/cargo scripts

What it **does not** receive:

1. **Project metadata** — git branch, recent commits, README excerpt,
   dependency versions. A user asking "run the linter" when in a Node.js
   repo that uses `eslint --fix` via an npm script gets a generic `eslint`
   invocation, not the project-specific one.
2. **Shell history** — the user's past successful commands for this cwd.
   Asking "compress the build output" the second time should recall the
   exact `tar -czf dist.tar.gz dist/` pattern the user ran last time.
3. **Semantically similar past commands** — the knowledge index (`src/knowledge/`)
   stores successful executions but nothing wires a top-K similarity query
   into the LLM prompt.

### Infrastructure that already exists

| Module | What it does | Gap |
|---|---|---|
| `src/knowledge/collections.rs` | Defines 5 collections incl. `Context` (`caro_project_context`) | Nothing ever writes to `Context` |
| `src/knowledge/indexers/` | `man`, `tldr`, `help`, `github` indexers → `Docs` | No indexer writes to `Context` |
| `src/context/directory.rs` | Detects `ProjectType`, Makefile, npm scripts | Does not persist/embed anything |
| `src/models/` `AiCapabilities.enable_history_search` | Config toggle exists | Never used; no history reader exists |
| `src/agent/` `AgentLoop::knowledge_index` | Feature-gated Option | Only used in post-execution `record_success`; not queried pre-generation |

The `Context` collection has been a **declared intent with zero implementation**
since it was introduced.

### Competitive landscape (Phase 1 research summary)

| Tool | Context mechanism | Failure mode | Caro's advantage |
|---|---|---|---|
| **GitHub Copilot CLI** | Shells out to `git status`, `ls` on each invocation; injects into system prompt as raw text | Ephemeral — no learning, latency on every call, no semantic similarity | Indexed + embedded: single vector lookup, off-call |
| **Amazon Q CLI** | Same shell-out pattern + "workspace profile" (manual config) | Cloud-only; no offline | Local-first, zero network required |
| **Atuin AI** (preview) | Vector search over personal shell history | Cloud sync required for AI features; history-only (no project metadata) | Offline; combines history + project metadata |
| **Claude Code memory** | `CLAUDE.md` plain-text read at session start | Static; must be manually maintained; not searchable | Auto-indexed; semantic retrieval |
| **Warp AI** | Terminal scrollback capture + cwd | Warp-only; no subprocess model | Works in any terminal via subprocess |

**Caro's unique position**: local-first (no cloud), works in any terminal as a
subprocess, combines project metadata + history into a single vector query,
privacy-gated by explicit opt-in toggles that already exist in config.

---

## Decision

Implement Local Context Indexing in three layers:

1. **`ContextSnapshot`** — a new serializable type that packages the full
   context bundle (execution + directory + top-K similar past commands +
   project metadata + recent history) for injection into the LLM prompt.

2. **`ProjectMetadataIndexer`** — a new `Indexer` impl that reads project
   root files (`README.md`, `Cargo.toml`, `package.json`, `pyproject.toml`,
   `go.mod`, git log summary) and writes structured entries to the
   `Context` collection.

3. **`ShellHistoryReader`** — a new reader (not an indexer — history is
   queried live, not embedded) that reads the last N cwd-filtered
   commands from Atuin's SQLite DB if present, else from
   `~/.bash_history` / `~/.zsh_history` / fish's binary history.
   This is gated by `[ai].capabilities.enable_history_search = true`.

The `AgentLoop` assembles a `ContextSnapshot` before calling the backend
and injects it as a structured section in the system prompt.

---

## New Types

### `src/context/snapshot.rs` (new file)

```rust
/// Full context bundle assembled before command generation.
///
/// Serializable from day one; printed as JSON with `--dry-run --json`.
/// All optional fields are empty-by-default; callers fill only what
/// the user has opted into.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextSnapshot {
    /// Platform context (always present)
    pub execution: ExecutionContextSummary,

    /// Project directory metadata (always present; no opt-in required)
    pub directory: DirectoryContextSummary,

    /// Top-K semantically similar past commands from knowledge index.
    /// Populated only when `[knowledge].enabled = true`.
    #[serde(default)]
    pub similar_commands: Vec<SimilarCommand>,

    /// Recent successful commands for this cwd from shell history.
    /// Populated only when `[ai].capabilities.enable_history_search = true`.
    #[serde(default)]
    pub recent_history: Vec<HistoryEntry>,

    /// Project metadata from indexed README / manifest files.
    /// Populated when `ProjectMetadataIndexer` has run for this cwd.
    #[serde(default)]
    pub project_summary: Option<ProjectSummary>,
}

/// Slim summary of ExecutionContext for prompt injection (avoids huge available_commands list)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionContextSummary {
    pub os: String,
    pub arch: String,
    pub shell: String,
    pub cwd: String,
    pub user: String,
}

/// Slim summary of DirectoryContext for prompt injection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectoryContextSummary {
    pub project_types: Vec<String>,   // ["Rust", "Docker"]
    pub has_git: bool,
    pub build_scripts: Vec<String>,   // ["cargo test", "cargo clippy", "make"]
}

/// A past command with its similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarCommand {
    pub request: String,
    pub command: String,
    pub similarity: f32,
}

/// A recent command from shell history, filtered by cwd
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
}

/// Project-level summary extracted from README / manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    /// One-line project description (from README H1 or Cargo.toml `description`)
    pub description: Option<String>,
    /// Key build commands detected from manifest
    pub key_commands: Vec<String>,
    /// Primary language/runtime version (e.g. "Rust 1.83", "Node 22")
    pub runtime: Option<String>,
    /// Git remote URL (origin, if present)
    pub git_remote: Option<String>,
    /// Current branch
    pub git_branch: Option<String>,
}
```

### `src/context/history.rs` (new file)

```rust
/// Reads recent cwd-filtered commands from available shell history sources.
///
/// Priority: Atuin SQLite → zsh histfile → bash histfile → fish binary.
/// Returns empty Vec if no history source is found (never errors).
///
/// This is a pure read; it does NOT embed or index. Indexing is intentionally
/// excluded from v1 to avoid the embedding overhead on every invocation.
pub struct ShellHistoryReader {
    limit: usize,
}

impl ShellHistoryReader {
    pub fn new(limit: usize) -> Self { Self { limit } }

    /// Read recent commands for the given cwd. Returns at most `self.limit` entries,
    /// most-recent first. Exit code and duration are best-effort (null if not available
    /// from the history source).
    pub fn read_for_cwd(&self, cwd: &Path) -> Vec<HistoryEntry> { ... }
}
```

### `src/knowledge/indexers/project.rs` (new file)

```rust
/// Indexes project root files into the Context collection.
///
/// Triggered by `caro index --project` or automatically on first run
/// when `[knowledge].auto_index_project = true` (default false).
///
/// Files indexed:
///   README.md (first 500 chars of body text)
///   Cargo.toml / package.json / pyproject.toml / go.mod  (name, description, version)
///   git log --oneline -10 (recent commits as context)
///
/// Staleness: each entry carries a content hash. Re-index is skipped when hash
/// matches stored value. This avoids re-embedding on every `caro` invocation.
pub struct ProjectMetadataIndexer {
    project_root: PathBuf,
}
```

---

## Files That Must Change

### New files (3)

| File | Purpose |
|---|---|
| `src/context/snapshot.rs` | `ContextSnapshot` and sub-types |
| `src/context/history.rs` | `ShellHistoryReader` |
| `src/knowledge/indexers/project.rs` | `ProjectMetadataIndexer` |

### Modified files (6)

| File | Change |
|---|---|
| `src/context/mod.rs` | Re-export `ContextSnapshot`, `ShellHistoryReader`; add `ContextSnapshot::build()` constructor |
| `src/knowledge/indexers/mod.rs` | Register `ProjectMetadataIndexer`; add `project` sub-module |
| `src/agent/mod.rs` | `AgentLoop::build_context_snapshot()` — assembles snapshot pre-generation; inject into prompt |
| `src/prompts/` (any system-prompt builder) | Accept `&ContextSnapshot`; render `## Project Context` and `## Similar Past Commands` sections |
| `src/models/mod.rs` | Add `[knowledge] auto_index_project: bool` (default false) |
| `Cargo.toml` | Add `rusqlite` as optional dep under `feature = "history-atuin"` for Atuin SQLite read |

### NOT touched

- `src/safety/` — no changes; snapshot context is informational, never bypasses validation
- `src/knowledge/backends/` — no schema changes; `Context` collection already declared
- `src/knowledge/schema.rs` — `EntryType` needs one new variant (`ProjectMetadata`), but this is backward-compatible (existing Arrow schema is additive)
- `src/ai/` — `AiCapabilities.enable_history_search` config toggle already exists; wired from there

---

## Exit Code / Output Contract

No new exit codes. The feature is entirely in the prompt-construction path;
the output to the user is unchanged (a shell command or a `--dry-run` JSON
blob).

`--dry-run --json` output gains a new optional `context_snapshot` key:

```json
{
  "command": "cargo clippy -- -D warnings",
  "safety": "safe",
  "confidence": 0.92,
  "context_snapshot": {
    "directory": { "project_types": ["Rust"], "has_git": true, "build_scripts": ["cargo test", "cargo clippy"] },
    "similar_commands": [
      { "request": "run clippy", "command": "cargo clippy", "similarity": 0.94 }
    ],
    "recent_history": [],
    "project_summary": { "description": "Natural language to shell commands", "runtime": "Rust 1.83" }
  }
}
```

Machines and scripts that parse `--dry-run --json` today are unaffected;
`context_snapshot` is an optional key.

---

## Integration Tests

All tests use deterministic fixtures (no network, no live history files).

| Test | Input | Expected exit | Expected JSON shape |
|---|---|---|---|
| `snapshot_build_rust_project` | Fake cwd with `Cargo.toml` + README | 0 | `project_types: ["Rust"]`, `description` non-null |
| `snapshot_build_no_project` | Empty temp dir | 0 | `project_types: []`, `project_summary: null` |
| `history_reader_atuin_sqlite` | Fixture `.atuin/history.db` with 5 cwd-matching rows | 0 | `recent_history.len() == 5`, exit codes present |
| `history_reader_zsh_fallback` | No Atuin DB; fixture `~/.zsh_history` | 0 | `recent_history.len() > 0` |
| `history_reader_empty` | No history source at all | 0 | `recent_history == []` |
| `project_indexer_content_hash` | Index same README twice | 0 | Second run: `IndexStats { skipped: 1, successful: 0 }` |
| `similar_commands_injected_into_prompt` | Seed knowledge index; call `AgentLoop` | 0 | System prompt contains `## Similar Past Commands` |
| `history_gated_by_capability` | `enable_history_search = false` | 0 | `recent_history == []` regardless of history files |

---

## Failure Modes Solved by Design (not workaround)

The Phase 1 audit of competitors revealed three recurring failure modes.
This implementation solves each structurally:

**1. Ephemeral context (GitHub Copilot CLI shell-out pattern)**
Competitors shell out to `git status`, `ls` on every invocation. This adds
latency and produces no accumulated knowledge. Caro's `ProjectMetadataIndexer`
runs once (or on explicit `caro index --project`), then all subsequent
invocations do a sub-millisecond vector lookup. No repeated shell-out.

**2. Privacy leakage (Atuin cloud-only, Amazon Q cloud-only)**
History and project metadata leave the machine in competitor implementations.
Caro's `ShellHistoryReader` reads local files only. `enable_history_search`
defaults to `false`. The off-host context warning from `caro ai` (PR #861)
already exists for the remote-backend case. No new privacy surface.

**3. Staleness (Claude Code `CLAUDE.md` pattern)**
Static files rot. Caro's `ProjectMetadataIndexer` tracks a content hash per
indexed file. Re-indexing is a no-op if the file hasn't changed. The vector
entry is updated only when the hash changes. This is the same idempotency
pattern used by `tldr` and `man` indexers.

---

## Out-of-Scope for This Version

| Item | Reason | Next version |
|---|---|---|
| Embedding shell history (not just reading it) | Adds 100ms+ per invocation for embedding; unclear value until user validation clears Gate 1 | v2.1: behind `knowledge.embed_history` flag |
| Cross-repo context ("I've run this in a similar repo") | Requires cross-cwd vector search; complex staleness model | v2.1 |
| `.caroignore` file for privacy exclusions | Useful but not blocking; users can disable `enable_history_search` entirely | v2.1 |
| Team/shared knowledge index | Requires ChromaDB Cloud or self-hosted; operational complexity | v3.0 (Karo distributed) |
| Automatic `caro index --project` on every `caro` invocation | Too slow; would break the <100ms UX target | Out of scope permanently; use `--watch` or CI hook |
| Windows history (PowerShell `PSReadLine` history) | Requires separate reader; low priority given current Linux/macOS user base | v2.1 |

---

## Validation Gate Status

Per `.claude/rules/validation-discipline.md`, this feature is Research-only
(Gate 1: 0/20 transcripts). This ADR is a **scope specification for when
the feature graduates**, not an authorization to open an implementation PR.

Implementation PR preconditions:
- [ ] Gate 1: 20 external-user interview transcripts tagged `local-context-indexing`
- [ ] Gate 2: Evidence beyond GitHub Issues (both existing issues are founder-authored)
- [ ] Gate 3: Demoware-trap section above covers the 100-user failure modes ✅
- [ ] Gate 4: Devil's-advocate review of this ADR
- [ ] Gate 5: N/A (no PMF claim)

The ADR is safe to merge now as a **research artifact**. The implementation
branch may not be opened until Gates 1, 2, and 4 clear.

---

## Alternatives Considered

**A. Shell-out on every invocation (Copilot CLI pattern)**
Simple, no indexing infra. Rejected: adds 50–200ms latency per call; no
learning from past commands; breaks the "pure subprocess" contract (spawning
git/ls from inside caro creates process-graph complexity).

**B. CLAUDE.md-style static project file (`.caro-context.md`)**
Users write a project context file once. Caro reads it. Simple. Rejected:
requires user maintenance; no semantic search; doesn't help with history;
misses the learning loop that makes Caro get smarter over time.

**C. Embed history immediately (Atuin-style full vector index)**
Index every history entry with embeddings. Rejected for v1: embedding
100k history entries takes minutes; incremental embedding requires a
background daemon (violates the "no daemon, no state" constraint). Deferred
to v2.1 behind a feature flag.

**D. Chosen: index-once project metadata + live history read**
Project metadata is indexed once (content-hash idempotent). Shell history
is read live (no embedding, just recency filter). Both are opt-in. This
hits the sweet spot between richness and latency.
