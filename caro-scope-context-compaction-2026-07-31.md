# Scope: Deterministic Session-History Compaction (auto-compact analog)

**Produced by:** `caro-research--scoping-process` scheduled task, 2026-07-31 (unattended run)
**Decision record:** `docs/adr/007-session-history-compaction.md`
**Feature researched:** Claude Code auto-compact / microcompact (experimental behavior),
Claude Agent SDK `compact_boundary` event, gemini-cli chat compression

> Feature-selection note (task ran with `[FEATURE NAME]` unfilled): coverage
> sweep of both ADR series (001–006, ADR-001–043) shows structured output,
> stream-json, init cache, multiturn sessions, hooks, memory, sandbox, MCP,
> receipts/undo, and council generation all covered. **Context/window
> compaction has no ADR in either series**, ADR-026's "fail loud, never
> truncate" covers stdin only, and the in-repo history path
> (`AiSession::render_history`) is a silent sliding window — so this run
> scoped it.
>
> Not committed (git-workflow rule). To land: `bin/sk-new-feature
> "session history compaction"`, move both files onto the branch, open a PR.

---

## Phase 1 — Feature research

### Problem and audience

Long agent sessions overflow the model context window. All three major agent
CLIs ship a compaction answer: Claude Code compacts automatically near ~95%
of the effective window (threshold ≈ window − 13K buffer), gemini-cli
compresses at a configurable fraction (`model.compressionThreshold`, default
recently moved 0.7 → 0.5), both also expose a manual command (`/compact`,
`/compress`). Audience: anyone running sessions longer than a few turns —
for caro this is *every* multi-turn user, because embedded SmolLM-class
models have 2–8K windows, not 200K.

### Core architecture (Claude Code, the richest analog)

- **Three tiers**: *microcompact* (deterministically prune stale tool
  results, no model call, runs continuously), *full compact* (dedicated LLM
  summarization pass, replaces older turns), *session-memory compact*
  (pre-extracted notes skip the summarization call).
- **Post-compaction reassembly**: boundary marker + summary + recently read
  files (capped) + skills + tool definitions + project instructions.
- **Data flow**: token watermark check per turn → pause turn → summarize →
  splice → resume.

### Structured output contract

The Agent SDK emits a system event with subtype **`compact_boundary`**
carrying `trigger: "auto" | "manual"` and `pre_tokens`. That is the entire
machine-visible surface — no schema version, no post-tokens, no account of
*what* was dropped. gemini-cli exposes even less (a config knob and a slash
command).

### Session/context lifecycle

Compaction is Claude Code's answer to redundant *re-initialization of
conversations*: instead of starting fresh when full, it folds history and
continues in-process. Post-compact reassembly re-injects instructions and
recent files so the next turn doesn't cold-start. (caro's analog spots:
ADR-025 amortizes init across processes; ADR-026 amortizes within one; this
scope amortizes *history* within and across both.)

### Why it is experimental — documented failure modes

| # | Failure | Evidence |
|---|---------|----------|
| F1 | Infinite compaction loops — compaction retriggers forever | claude-code #6004, #12556, #41984 |
| F2 | Death spiral — fixed context ≈ whole window, each compact frees too little, immediately retriggers (6 compactions / 3.5 min) | claude-code #24677 |
| F3 | Compaction itself overflows — `/compact` fails "conversation too long"; the summarizer needs the context it's shrinking | claude-code #23751 |
| F4 | Compounding loss — LLM summaries retain ~3.7/5 of info; summarize-the-summary cycles permanently destroy early constraints (replace-in-place) | community measurements / bug reports |
| F5 | Post-compact amnesia — agent re-reads / re-does completed work | claude-code #11487 |

Common root cause: **compaction is an unbounded LLM call with no convergence
guarantee, applied destructively to the only copy of the history.**

---

## Phase 2 — Competitive differentiation

### What they get right (replicate)

- A machine-visible **boundary event** with trigger + size info.
- **Tiering**: try a cheap deterministic prune before any model call.
- **Protected segments**: system prompt, recent turns, and instructions are
  never summarized away; reassembly re-injects framing.
- Manual + automatic triggers through the same path.

### Their gaps (avoid by schema-first design)

| Analog gap | caro design answer |
|---|---|
| Compaction is an LLM call that can loop/fail (F1–F3) | v1 compaction is a **pure deterministic fold** — cannot loop, cannot fail, output size computable a priori |
| Destructive replace-in-place; compounding loss (F4) | **Lossless at rest**: full history stays in the session file; compaction only changes prompt *rendering*, recomputed from the full source each turn — idempotent by construction |
| Recency-only retention drops task framing (F5) | **Pin-first + fold-middle + verbatim-tail** shape keeps both the original framing and recent detail |
| Unversioned, implicit summary format | Folded gists + report are serde types with `schemars` JSON Schema under `schema_version: "1"` |
| No post-size / no record of what was folded | Report carries `pre_chars`, `post_chars`, `turns_folded`, and per-turn gists |

### Unique positioning

Offline, deterministic, universal: identical input session + policy →
byte-identical rendering on every platform, with no network and no model.
None of the analogs can promise that. And because caro sessions are small
file-backed JSON (commands, not 100K-token tool dumps), keeping the full
history at rest is free — the analogs' destructive replace is a constraint
caro simply doesn't have.

### Existing infrastructure that already covers part of this

| Piece | Where | Role here |
|---|---|---|
| `AiSession`, `Turn`, `Role` (serde, file-backed) | `src/ai/session.rs` | storage layer — unchanged |
| `render_history(max_turns)` sliding window | `src/ai/session.rs:110` | the naive mechanism being replaced |
| Hardcoded `render_history(6)` call site | `src/ai/privacy.rs:55` | wire-up point |
| `SessionStore` TTL resume | `src/ai/store.rs` | cross-process continuity — unchanged |
| Headless envelope / events / exit codes | ADR-024/026 (`src/cli/mod.rs`) | extended additively |
| `ConfigManager` | `src/config/mod.rs` | policy home |
| ADR/006 learned memory | scoped 2026-07-17 | owns *cross-session* preferences; compaction owns *in-session* history — explicitly disjoint |

---

## Phase 3 — Scope definition

### ADR

`docs/adr/007-session-history-compaction.md` (written this run): context,
decision (5 commitments), consequences, alternatives considered.

### New types (all in `src/ai/session.rs`; contract-facing ones also derive `schemars::JsonSchema`)

```rust
/// How history is folded into the prompt. Lives under [ai.history] in config.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct CompactionPolicy {
    pub enabled: bool,          // default true; false ⇒ legacy render_history
    pub tail_turns: usize,      // verbatim recent turns, default 4
    pub gist_chars: usize,      // per-folded-turn cap, default 120
    pub char_budget: usize,     // total rendered-history cap, default 2048
    pub pin_first_user_turn: bool, // default true
}

/// Deterministic one-line digest of a folded turn.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FoldedTurn {
    pub index: u32,
    pub role: Role,                     // reuses ai::session::Role serde repr
    pub gist: String,                   // assistant: command; user: first line; ≤ gist_chars
}

/// Result of rendering under a policy. `rendered` goes into the prompt;
/// the rest is observability.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CompactedHistory {
    pub schema_version: String,         // "1"
    pub rendered: String,
    pub turns_total: u32,
    pub turns_rendered: u32,            // pinned + tail
    pub turns_folded: u32,
    pub pre_chars: u64,                 // full naive rendering size
    pub post_chars: u64,                // == rendered.len()
}

/// Additive envelope field + ndjson event payload (mirror of the analog's
/// compact_boundary, but versioned and with post-size).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CompactionReport {
    pub trigger: CompactionTrigger,     // Auto | Off  (Manual reserved for v2)
    pub turns_folded: u32,
    pub pre_chars: u64,
    pub post_chars: u64,
}
```

Method contract on `AiSession`:

```rust
/// Pure fold: pinned first user turn + folded middle (FoldedTurn gists)
/// + last `tail_turns` verbatim. Recomputed from the full turn list every
/// call (idempotent, lossless at rest). Never errors; over-budget input
/// degrades gists before tail, tail before pin. Deterministic:
/// same (turns, policy) ⇒ byte-identical output.
pub fn render_history_compacted(&self, policy: &CompactionPolicy) -> CompactedHistory;
```

`render_history(max_turns)` stays for one release, delegating to the new
path when `enabled`, and is removed in the next breaking window.

### Minimal file-change set (no new modules)

| File | Change |
|---|---|
| `src/ai/session.rs` | new types + `render_history_compacted` + unit tests |
| `src/ai/privacy.rs` | replace `render_history(6)` with policy-driven call |
| `src/config/mod.rs` | `[ai.history]` section → `CompactionPolicy` (serde defaults) |
| `src/cli/mod.rs` | additive `compaction: Option<CompactionReport>` on the ADR-024/026 envelope; additive `Compaction` variant on the ndjson event enum |
| `docs/headless-contract.md` | document field/event + JSON Schema (schemars) |
| `tests/history_compaction.rs` | integration tests below |

### Exit code / output contract (what machines depend on)

- **No new exit codes.** Compaction is infallible by design; a malformed
  `[ai.history]` config is the existing usage error, exit `2`.
- `--output json`: envelope gains optional `compaction` object (absent when
  `enabled = false` or nothing folded) — additive under `schema_version "1"`.
- `--output ndjson`: one `{"type":"compaction","trigger":"auto",
  "turns_folded":N,"pre_chars":A,"post_chars":B}` event per turn that folded
  anything, emitted before that turn's assistant event.
- Stability promise: `post_chars ≤ char_budget` always; identical session
  file + policy ⇒ identical `rendered` bytes across platforms/releases
  within a schema version.

### Integration tests (known input → deterministic JSON + exit code)

Static backend, `TempDir` session store, fixed clock — same harness as the
ADR-026 test plan.

1. **Golden fold**: fixture session with 12 turns, default policy →
   `rendered` equals checked-in golden file byte-for-byte; exit 0.
2. **Constraint retention (the bug this kills)**: turn 1 = "only in src/,
   never node_modules", turns 2–20 = refinements → `rendered` contains the
   turn-1 constraint; naive `render_history(6)` provably does not.
3. **Budget invariant (F2/F3 answer)**: 200-turn adversarial session with
   10K-char turns → `post_chars ≤ char_budget`, single pass, exit 0.
4. **Idempotence (F1/F4 answer)**: render → append 1 turn → render; second
   report's `pre_chars` reflects full history (source of truth intact);
   rendering twice with no new turns is byte-identical.
5. **Envelope/event wiring**: headless 8-turn conversation → envelope has
   `compaction` with `turns_folded > 0`; ndjson stream contains `compaction`
   events; both parse against the schemars-generated schema.
6. **Off switch**: `enabled = false` → no `compaction` field, no event,
   legacy rendering used; exit 0.
7. **Safety composition**: folded gist containing `rm -rf /` in history must
   not alter validator behavior on the *new* command — assert
   `SafetyValidator` verdicts identical with compaction on/off.

### Out of scope (next version)

- **LLM summarization tier** (analog's full compact) as an optional layer on
  the same `CompactionReport` contract — requires ADR/006-style budget rules.
- **Manual trigger** (`CompactionTrigger::Manual` reserved; needs a CLI verb).
- **Token-accurate budgets** (tokenizer-dependent; chars are the v1 proxy).
- **Microcompact of context-gathering payloads** (directory listings etc. —
  belongs to `src/context/`, different data shape).
- Cross-session preference retention (ADR/006 memory owns it).
- Semantic retrieval over folded turns (ADR-017 index).

### Constraint compliance

- **Reuse, don't duplicate**: session/store/config/envelope/validator all
  reused; zero new modules, zero new exit codes.
- **Serializable day one**: every new type derives Serialize/Deserialize +
  JsonSchema with `schema_version`.
- **Pure subprocess**: a pure function over an already-loaded session; no
  daemon, no state beyond the existing session file.
- **Phase-1 failure mode solved by design, not workaround**: F1–F5 each map
  to a structural property (fold not LLM ⇒ F1/F3; computable output bound ⇒
  F2; lossless-at-rest recompute ⇒ F4; pin-first salience ⇒ F5).

---

## Sources

- [Compaction — Claude Platform docs](https://platform.claude.com/docs/en/build-with-claude/compaction)
- [Claude Code auto-compact behavior (threshold, tiers, reassembly)](https://decodeclaude.com/compaction-deep-dive/), [source-level analysis](https://barazany.dev/blog/claude-codes-compaction-engine)
- [Agent SDK compact_boundary / streaming output](https://platform.claude.com/docs/en/agent-sdk/streaming-output), [SDK hooks request #772](https://github.com/anthropics/claude-agent-sdk-python/issues/772)
- [gemini-cli compression threshold PRs #12317 / #13517](https://github.com/google-gemini/gemini-cli/pull/13517), [config docs](https://google-gemini.github.io/gemini-cli/docs/get-started/configuration.html)
- Failure evidence: claude-code issues [#6004](https://github.com/anthropics/claude-code/issues/6004), [#12556](https://github.com/anthropics/claude-code/issues/12556), [#24677](https://github.com/anthropics/claude-code/issues/24677), [#23751](https://github.com/anthropics/claude-code/issues/23751), [#11487](https://github.com/anthropics/claude-code/issues/11487), [#41984](https://github.com/anthropics/claude-code/issues/41984)
- Repo evidence: `src/ai/session.rs:110` (`render_history`), `src/ai/privacy.rs:55` (hardcoded window), `docs/adr/ADR-026-headless-multiturn-agentic-session.md` (§stability, §types), ADR sweep 001–006 / ADR-001–043
