# ADR-033: Command Undo Snapshots — `caro undo` for Shell Side Effects

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-08
- **Produced by**: `caro-research--scoping-process` scheduled run
- **Feature researched**: Claude Code Checkpointing / `/rewind` + Agent SDK file checkpointing (Anthropic)

## Context

### The market problem

Claude Code ships "checkpointing": before every file edit the agent makes, it
backs up the file; `/rewind` (or the Agent SDK's `rewindFiles()`) restores code
to any prior prompt boundary. The pitch is a safety net that lets users attempt
ambitious changes knowing they can undo them. OpenAI's Codex has an open
feature request for the same capability
([openai/codex#12558](https://github.com/openai/codex/issues/12558)) — the
market has decided agents need an undo button.

### Their architecture (as documented)

- **Capture**: automatic backup of a file's content immediately before each
  `Write` / `Edit` / `NotebookEdit` tool call. Copy-on-first-write per session.
- **Checkpoint identity**: the UUID of the user message that started the turn.
  Every prompt is a restore point.
- **Storage/lifecycle**: session-scoped store, persists across resumes,
  garbage-collected with the session after 30 days (configurable).
- **Restore semantics**: files created since the checkpoint are deleted;
  files modified are restored to captured content. Conversation state can be
  restored independently of code state.
- **Programmatic contract (Agent SDK)**: checkpoint UUIDs only appear in the
  stream if a *hidden* extra arg (`replay-user-messages`) is set; CLI rewind
  requires an env var (`CLAUDE_CODE_ENABLE_SDK_FILE_CHECKPOINTING`) plus a
  `--rewind-files` flag that is *absent from `claude --help`*; rewinding after
  a stream ends requires resuming the session with an empty prompt to reopen a
  process transport ("ProcessTransport is not ready for writing" error class).

### Why it is limited — documented failure modes

1. **Bash side effects are not tracked.** Their own docs lead with it: if the
   agent runs `rm file.txt`, `mv old.txt new.txt`, `sed -i`, or `echo > f`,
   rewind cannot undo it. Only edits made through the file-editing tools are
   captured. The undo system has a hole exactly where the highest-risk
   mutations happen.
2. **Directory operations are not undone.** Creating, moving, or deleting
   directories is out of scope even for tracked tools.
3. **External and concurrent changes are invisible.** Manual edits and other
   sessions' writes are not captured, so a rewind can silently clobber them.
4. **Session-coupled lifecycle.** A checkpoint is meaningless without its
   session; restore requires reopening a live agent connection. There is no
   standalone "restore checkpoint X" subprocess call, no documented exit
   codes, no versioned payload — the contract is hidden flags + env vars.

Failure mode #1 is the one caro must solve *by design*: *caro's entire output
surface is shell commands.* A checkpoint system that excludes shell side
effects would protect exactly nothing for caro users.

### Prior art in caro (reuse, do not duplicate)

| Infrastructure | What it gives this feature | Status |
|---|---|---|
| `SafetyValidator` + 52-pattern library (`src/safety/`) | Risk level (`RiskLevel` in `src/models/mod.rs`) that decides *when* a snapshot is mandatory; patterns already identify destructive verbs and their path arguments | Shipped |
| `CommandExecutor` (`src/execution/executor.rs`) | The single choke point where every `--execute` passes; the natural capture hook | Shipped |
| `CacheManager` (`src/cache/`) | Checksummed, size-budgeted local store with manifest — the snapshot store reuses this instead of inventing storage | Shipped |
| Headless contract + exit codes (ADR-024) | `HeadlessEnvelope`, versioned JSON, exit-code registry (0–6 taken; 7 ADR-027; 8 ADR-031; 30–32 ADR-030) | Scoped |
| Execution attribution (ADR-021) / receipts (ADR-032) | Checkpoint IDs slot into the receipt chain as evidence of what was captured before execution | Scoped |
| AST parser (ADR-007 / PRD-ast-parser) | Higher-fidelity affected-path extraction when it lands; v1 works without it | Scoped |
| Bubblewrap sandbox (ADR-010) | Complementary: sandbox = prevention, undo = recovery. Not overlapping | Scoped |

### Our unique positioning

- **We know the command before it runs.** Claude Code sees bash as an opaque
  tool call; caro *generates and parses* the command, so affected paths are
  predictable at validation time. Their gap is our home turf.
- **Offline and session-free.** No daemon, no resumed connection: a checkpoint
  is a directory on disk plus a JSON receipt; restore is one subprocess call.
- **Universal.** Works in any directory, no git repo required, any POSIX shell.
- **Honest about fidelity.** Where prediction is impossible we say so in a
  typed field instead of silently not capturing (their failure mode #3/#1).

## Decision

Add **pre-execution filesystem snapshots** and a **`caro undo` subcommand**,
both pure subprocess calls, gated behind an `undo` config section.

### Data flow

```
caro "delete all logs" --execute
  └─ generate → SafetyValidator (existing)
       └─ NEW: PathPredictor::predict(&GeneratedCommand) → Vec<PredictedPath> + CaptureFidelity
            └─ NEW: SnapshotWriter::capture() → UndoCheckpoint (JSON receipt + blobs in cache dir)
                 └─ CommandExecutor::execute (existing)
                      └─ receipt finalized with post-execution hashes

caro undo --last            # restore most recent checkpoint
caro undo <checkpoint-id>   # restore specific checkpoint
caro undo --list --json     # machine-readable inventory
```

Snapshots are automatic when `--execute` runs a command whose `RiskLevel` is
`Medium` or higher (configurable), and on-demand via `--undo-snapshot` for any
execution. Capture failure **aborts execution** (fail-safe, same posture as
ADR-010: no silent fall-through).

### New types (all in `src/models/mod.rs`, all `Serialize + Deserialize` from day one)

```rust
/// How completely the snapshot covers the command's write set.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureFidelity {
    /// Every predicted write target was captured before execution.
    Full,
    /// Some targets captured; others unpredictable (globs resolved at
    /// runtime, command substitution). `gaps` lists why.
    Partial,
    /// No meaningful capture possible (network effects, pipes into
    /// unknown binaries, sudo). Snapshot refused, recorded as such.
    None,
}

/// One filesystem entry captured before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedEntry {
    pub path: PathBuf,
    /// `file` | `dir` | `symlink` | `absent` (path predicted to be
    /// created — restore deletes it, mirroring Claude Code semantics).
    pub kind: EntryKind,
    /// SHA-256 of captured content (None for `absent` / `dir` markers).
    pub pre_hash: Option<String>,
    /// SHA-256 observed immediately after execution; used for
    /// conflict detection at restore time.
    pub post_hash: Option<String>,
    /// Relative blob location inside the checkpoint directory.
    pub blob: Option<PathBuf>,
    pub mode: Option<u32>,
    pub bytes: u64,
}

/// The on-disk receipt: `<cache>/undo/<id>/checkpoint.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoCheckpoint {
    pub schema_version: u32,          // = 1
    /// Content-addressed: sha256(command + cwd + unix_millis)[..16].
    /// NOT a session UUID — restorable with zero session context.
    pub id: String,
    pub created_unix_ms: u64,
    pub command: String,              // the executed command text
    pub cwd: PathBuf,
    pub risk_level: RiskLevel,        // existing type
    pub fidelity: CaptureFidelity,
    /// Human-and-machine-readable reasons fidelity < Full.
    pub gaps: Vec<String>,
    pub entries: Vec<CapturedEntry>,
    pub total_bytes: u64,
    /// Attribution / receipt linkage (ADR-021 / ADR-032), optional.
    pub receipt_id: Option<String>,
}

/// Result of `caro undo` (emitted inside the ADR-024 envelope).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOutcome {
    pub schema_version: u32,          // = 1
    pub checkpoint_id: String,
    pub restored: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,        // entries that were `absent` pre-exec
    pub skipped_conflicts: Vec<PathBuf>,
    pub dry_run: bool,
}
```

Method contracts (implemented in `src/execution/`, not new modules):

- `PathPredictor::predict(cmd: &GeneratedCommand, cwd: &Path) -> Prediction`
  — pure function; v1 uses tokenization + the safety pattern library's
  argument extraction; upgrades transparently to the ADR-007 AST when merged.
- `SnapshotWriter::capture(prediction, budget) -> Result<UndoCheckpoint, UndoError>`
  — copies files/dirs (recursive, bounded by `max_snapshot_bytes`, default
  256 MiB) into `CacheManager`-managed storage; write receipt last (a
  checkpoint without a fully written receipt is invisible → crash-safe).
- `Restorer::restore(id, force, dry_run) -> Result<RestoreOutcome, UndoError>`
  — for each entry, compare current hash with `post_hash`; mismatch means the
  file changed since caro touched it → skip and report unless `--force`
  (solves their failure mode #3 with detection instead of silence).

### Designed answers to their failure modes

| Their failure mode | Our design answer |
|---|---|
| Bash side effects untracked | Capture happens *because* we run the command: snapshot precedes execution at the single executor choke point; there is no untracked mutation path |
| Directories not undone | `EntryKind::Dir` entries snapshot recursively within the byte budget; `mv`/`rm -r` of a directory is restorable |
| External changes clobbered by rewind | `post_hash` conflict detection: restore refuses (exit 41) if the file changed after execution, unless `--force` |
| Hidden flags, env vars, session coupling | First-class documented subcommand; content-addressed ID independent of any session; versioned JSON receipt; registered exit codes |
| Unpredictable writes silently untracked | Typed `CaptureFidelity` + `gaps[]` in receipt and stderr warning; `fidelity=none` is a recorded fact, never an omission |

### Output contract (extends ADR-024 — machines will depend on this)

`caro undo --list --json` and `caro undo <id> --json` emit the standard
`HeadlessEnvelope` with a `data` payload of `Vec<UndoCheckpoint>` (receipts
only, no blobs) / `RestoreOutcome` respectively. `schema_version` bumps only
additively per ADR-024 rules.

Exit codes (to be registered in ADR-024's `ExitCode` — non-colliding with
0–6, 7, 8, 30–32):

| Code | Meaning |
|---|---|
| 9  | Snapshot capture failed pre-execution → execution **aborted** (fail-safe) |
| 40 | `caro undo`: checkpoint not found |
| 41 | `caro undo`: restore conflict (file changed since execution); nothing partially applied without `--force` |
| 42 | `caro undo`: checkpoint has `fidelity=none` / nothing restorable |

### Retention

Reuse `CacheManager` accounting: prune checkpoints older than
`undo.retention_days` (default 30, mirroring Claude Code) or beyond
`undo.max_total_bytes` (default 1 GiB), oldest first. `caro undo --prune`
runs it explicitly; capture runs it opportunistically.

### Files changed (minimal set — no new top-level modules)

| # | File | Change |
|---|------|--------|
| 1 | `src/models/mod.rs` | `CaptureFidelity`, `EntryKind`, `CapturedEntry`, `UndoCheckpoint`, `RestoreOutcome`, `UndoError` |
| 2 | `src/execution/mod.rs` (+ small `undo.rs` submodule) | `PathPredictor`, `SnapshotWriter`, `Restorer`; capture hook in the execute path |
| 3 | `src/cache/mod.rs` | expose namespaced storage root + size accounting for `undo/` |
| 4 | `src/cli/mod.rs` | `undo` subcommand (`--list`, `--last`, `--prune`, `--force`, `--dry-run`, `--json`), `--undo-snapshot` flag; envelope wiring |
| 5 | `src/config/mod.rs` | `[undo]` section: `enabled`, `min_risk_level`, `max_snapshot_bytes`, `retention_days`, `max_total_bytes` |
| 6 | `src/main.rs` | exit-code wiring for 9 / 40 / 41 / 42 |
| 7 | `tests/undo_snapshots.rs` | new integration test file (flat, per existing convention) |

### Integration tests (deterministic input → fixed JSON + exit code)

All tests run in a tempdir fixture tree with pinned mtimes; command execution
uses the real executor on plain `rm`/`mv`/`cp`/`>` commands (no LLM in the
loop — commands supplied via `--dry-run`-style direct injection as in
ADR-024's test harness). Each test asserts parsed JSON **and** process exit
code:

1. `rm fixture.txt` → capture → execute → `caro undo --last` → file restored
   byte-identical, exit 0, `RestoreOutcome.restored == [fixture.txt]`.
2. `mv a.txt b.txt` → undo restores `a.txt`, deletes `b.txt` (`absent` entry).
3. `rm -r subdir/` → directory restored recursively.
4. Post-execution external edit → undo exits 41, file untouched,
   `skipped_conflicts` names it; `--force` restores, exit 0.
5. Command with substitution (`rm $(cat list)`) → `fidelity=partial`,
   `gaps` non-empty, capture proceeds for predictable args.
6. `curl -X DELETE https://…` → `fidelity=none`, receipt written, exit 0 on
   execute; later `caro undo` of it exits 42.
7. Capture I/O failure (unreadable predicted path, injected) → exit 9,
   command **not** executed.
8. `caro undo deadbeef` → exit 40.
9. `caro undo --list --json` on three checkpoints → stable envelope,
   `schema_version: 1`, sorted newest-first.

### Alternatives considered

- **Git-based (stash/worktree per execution).** Rejected: requires a repo,
  pollutes reflog, ignores untracked/ignored files — caro often runs against
  `~/Downloads`, `/etc`, not repos. (This is also why "use git" is Claude
  Code's documented cop-out, not a solution.)
- **Filesystem CoW snapshots (APFS/btrfs/ZFS).** Rejected for v1: platform-
  and privilege-dependent; violates "universal". Viable future backend behind
  the same `UndoCheckpoint` contract.
- **Syscall interception (LD_PRELOAD/fanotify) for exact write sets.**
  Rejected: fragile, platform-specific, overlaps ADR-010's sandbox layer;
  prediction + honest fidelity is simpler and testable.
- **Execute-in-overlay, commit-on-success (sandbox pairing).** Deferred to a
  future ADR-010 integration — it is prevention-plus-undo in one, but drags
  in the whole sandbox dependency; v1 must stand alone.

## Consequences

**Positive.** Caro gains the undo button competitors document as a hole, in
the exact domain (shell side effects) they exclude; recovery complements
ADR-010's prevention and ADR-032's attestation; the receipt is one more
hash-chainable artifact. Zero daemon, zero session state, works offline.

**Negative / accepted costs.** Snapshot copies cost time and disk on large
targets (bounded by budget; capture of >256 MiB downgrades to `partial` with
a `gap` naming the oversized path). Prediction is heuristic until ADR-007
lands — fidelity typing makes that honesty explicit rather than a bug class.
Four new exit codes for integrators.

## Out of scope (next version)

- Conversation/session rewind (ADR-026 sessions may *reference* checkpoint
  IDs; restoring conversational state is their concern, not this ADR's)
- CoW filesystem backends; overlay/sandboxed execute-then-commit (ADR-010 v2)
- Undo for remote/network side effects and `sudo` commands (always
  `fidelity=none` in v1)
- Cross-checkpoint selective restore (per-file cherry-pick)
- Interactive TUI picker (Claude Code's `Esc Esc` menu equivalent)
- Hook for `caro fix` (ADR-016) to auto-offer undo after a failed execution

## References

- [Claude Code — Checkpointing](https://code.claude.com/docs/en/checkpointing) (fetched 2026-07-08)
- [Claude Agent SDK — Rewind file changes with checkpointing](https://code.claude.com/docs/en/agent-sdk/file-checkpointing) (fetched 2026-07-08)
- [openai/codex#12558 — request for /rewind-style checkpoint restore](https://github.com/openai/codex/issues/12558)
- ADR-007 (AST parser), ADR-010 (sandbox), ADR-021 (attribution), ADR-024
  (headless contract), ADR-030/031/032 (exit-code precedents)
