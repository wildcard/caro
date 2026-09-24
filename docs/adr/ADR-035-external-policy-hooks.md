# ADR-035: External Policy Hooks — User-Extensible Pre-Execution Validators with a Fail-Closed JSON Contract

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-13
- **Authors**: caro-research scoping process (automated scheduled run)
- **Builds on / relates to**: ADR-020 (tiered approval protocol — remote HTTP
  approval webhook), ADR-022 (caro safety library API), ADR-024 (headless
  JSON contract + exit-code registry), ADR-034 (degraded-result provenance),
  CaroML multi-angle validator framework (`src/caroml/validators/`)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. The analog selected
> this run is **Claude Code Hooks** (code.claude.com/docs/en/hooks, fetched
> live 2026-07-13): the most mature user-extensible lifecycle-hook system in
> any agentic CLI, now spanning 30 events and five handler types. The target
> was chosen because `src/caroml/validators/mod.rs` already promises an
> "external-validator hook (path to a binary that reads JSON on stdin /
> writes JSON on stdout)" for v0.2 with no ADR, no schema, and no scope
> behind it. Treat the analog choice as a reviewable assumption.

---

## Context

### Phase 1 — what Claude Code Hooks do, and where the design leaks

Claude Code lets users register external handlers (shell commands, HTTP
endpoints, MCP tools, prompts, agents) at ~30 lifecycle events
(`PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `SessionStart`, …).
Matchers filter by tool name; handlers receive JSON on stdin and answer
through **two mutually exclusive channels**: exit codes, or (on exit 0
only) a JSON object on stdout carrying `decision` /
`hookSpecificOutput.permissionDecision` (allow / deny / ask / defer).
Matching hooks run in parallel with dedup by command string; per-event
default timeouts range 10 s–600 s. Command hooks support *exec form*
(`command` + `args`, no shell) and *shell form* (string through `sh -c`).

It solves a real problem — deterministic org policy around a
non-deterministic agent — and it is the most-requested integration surface
in that ecosystem. But the reference documentation itself records five
failure modes we can design out rather than inherit:

1. **Exit 1 proceeds.** "Claude Code treats exit code 1 as a non-blocking
   error and proceeds with the action, even though 1 is the conventional
   Unix failure code." A policy script that crashes *approves by
   accident*. Only the magic value 2 blocks — and its meaning varies per
   event, requiring a 30-row behavior table.
2. **Fail-open everywhere.** HTTP hook timeouts, connection failures, and
   non-2xx responses are all "non-blocking errors that allow execution to
   continue." The `if` pre-filter "fails open, running your hook
   regardless of pattern, when the Bash command can't be parsed." Their
   docs concede the point: "use the permission system rather than a hook
   to enforce a hard allow or deny." The hook layer cannot be trusted as
   a policy boundary *by their own admission*.
3. **Dual signaling channels.** "You must choose one approach per hook,
   not both … Claude Code only processes JSON on exit 0. If you exit 2,
   any JSON is ignored." Authors routinely lose their structured reason
   string by combining `exit 2` with a JSON body.
4. **Coverage gaps.** `PreToolUse` never fires for `@`-referenced files;
   only tool calls trigger it. The policy surface has holes that must be
   patched by a *different* subsystem (permission rules).
5. **Session coupling.** Hook input carries `session_id` and
   `transcript_path`; several behaviors (hook snapshot semantics, `/hooks`
   review menu) assume a long-lived interactive session. Nothing about
   the contract works as a pure subprocess primitive.

### The caro gap

Caro has three adjacent mechanisms and a hole in the middle:

- The **built-in validator chain** (`src/caroml/validators/`): safety,
  platform, secrets, side-effects angles — compiled in, not extensible
  without a Rust toolchain and a fork.
- **ADR-020 approval webhook**: remote, human-in-the-loop, HTTP — answers
  "may I?", not "is this compliant with local policy?". Requires a server.
- **ADR-022 safety library API**: caro embedded in *other* tools; the
  inverse of letting users extend caro.

An org that wants "never touch `/etc/nginx` on prod boxes", "block any
command referencing our customer-data bucket", or "warn on package
installs outside our mirror" today has exactly one option: fork
`patterns.rs`. The validators module has promised the external-hook
escape hatch since the multi-angle framework landed; this ADR gives it a
contract.

### Validation-discipline gates

Per `.claude/rules/validation-discipline.md`, this scope extends the
validated core loop (generation → safety validation → execution) with
infrastructure rather than opening a new product line; it follows the
same exemption reasoning recorded for ADRs 024–034. The demoware section
below is included regardless.

---

## Decision

### One event, not thirty

Caro's entire output surface is a single action — one shell command about
to run. The 30-event lattice collapses to one choke point. v1 ships
exactly one hook event: **`pre_execute`**, fired after generation and the
built-in validator chain, before confirmation/execution/output. Everything
else (post-execution observation, prompt-time context injection) is
deliberately out of scope; ADR-032 receipts already serve post-hoc
observers.

### Configuration (reuses `ConfigManager` / `config.toml`)

```toml
[[hooks.pre_execute]]
name       = "block-prod-nginx"          # required, unique, [a-z0-9-]+
command    = "/usr/local/bin/caro-hook-nginx"  # exec form ONLY — no shell
args       = ["--mode", "strict"]        # optional argv, no quoting pitfalls
match      = "nginx|/etc/"               # optional regex on generated command; absent = always
timeout_ms = 5000                        # default 5000, max 60000
on_failure = "closed"                    # "closed" (default) | "open"
```

Design deltas from the analog, each answering a Phase 1 failure mode:

- **Exec form only.** No `sh -c` string variant. Their own docs steer
  authors to `args` form for anything with paths; shell form exists there
  for legacy reasons caro doesn't have. Removes an entire quoting/injection
  class. (Failure mode: config-driven shell injection.)
- **`on_failure = "closed"` by default.** A hook that crashes, times out,
  or emits garbage *blocks execution* unless the user explicitly opts that
  hook into fail-open. This inverts analog failure modes 1 and 2 by
  design — for a safety tool, a broken policy check must never approve.
- **`match` is best-effort narrowing only, never widening.** A regex that
  fails to compile is a config error at load time (exit 51), not a
  silently-skipped filter. There is no equivalent of their fail-open `if`
  parse path.
- **User-level config only in v1.** Hooks load from
  `~/.config/caro/config.toml`, never from repo-local files, so cloning a
  malicious repository cannot install a hook. Project-level hooks need a
  trust-gating design first (out of scope).
- **No coverage gap.** The hook fires at the executor choke point, so
  every command — every backend, `--execute` or dry-run, interactive or
  headless — passes through it. There is no `@`-reference-style bypass
  because there is no second path to execution.

### Wire contract (single channel, versioned, session-free)

The hook binary receives one JSON object on stdin and must reply with one
JSON object on stdout, then exit 0. **Exit 0 with parseable JSON is the
only success path.** Any non-zero exit, timeout, or unparseable stdout is
a *hook failure* routed through `on_failure` — never a decision. This
replaces the analog's dual-channel exit-code protocol (failure modes 1
and 3) with one channel; the exit code carries no semantics beyond
"protocol kept / protocol broken", so there is nothing like the per-event
exit-2 behavior table to memorize, and a reason string can never be lost.

`HookRequest` (schema_version 1) carries: the generated command, the
natural-language intent, platform triple, `RiskLevel`, active
`SafetyLevel`, and a summary of built-in `ValidationOutcome`s. It carries
**no session id, no transcript path, no cwd of any caro-internal state** —
the same request can be replayed byte-identically in a test. Stdout
hygiene is a non-issue by construction: caro reads the *last* line of
stdout that parses as a JSON object, and everything else is captured into
the outcome's `stdout_noise` field for `--debug` display rather than
breaking the parse (failure mode: fragile "stdout must be only JSON").

`HookResponse` (schema_version 1): `decision` = `allow` | `warn` |
`deny`, plus optional `reason` and `repair_hint`. `warn` surfaces the
reason and proceeds; `deny` blocks with provenance. There is no `ask`
tier in v1 — escalation-to-human is ADR-020's webhook job, and the two
compose (a hook may deny; it may not approve past the built-in safety
validator, which remains `must_pass`). **Hooks can only tighten, never
loosen** — an `allow` from a hook cannot override a Block from
`patterns.rs`.

### Integration point (reuses the validator framework)

The implementation is an adapter, `ExternalHookAngle`, implementing the
existing `Validator` trait (`angle() = "hook:<name>"`, `must_pass()` =
`on_failure == closed`). One mechanism then serves both pipelines: the
CaroML per-step interpreter chain and the single-command CLI path. Hook
outcomes serialize into the ADR-024 envelope as `hook_outcomes[]` using
the same typed-provenance style ADR-034 established for degraded results.
Multiple hooks run concurrently via the existing `run_all` dispatcher,
deduplicated by `name`.

---

## New types (all in existing modules, `serde` + `schemars` from day one)

`src/caroml/validators/external.rs` (new file inside the existing module):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HookConfig {
    pub name: String,
    pub command: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default, rename = "match")]
    pub match_pattern: Option<String>,   // compiled at load; invalid = exit 51
    #[serde(default = "default_hook_timeout_ms")]
    pub timeout_ms: u64,                 // default 5000, clamped to 60000
    #[serde(default)]
    pub on_failure: FailurePolicy,       // Closed (default) | Open
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FailurePolicy { #[default] Closed, Open }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HookRequest {
    pub schema_version: u32,             // 1
    pub event: String,                   // "pre_execute"
    pub command: String,
    pub intent: String,
    pub platform: String,
    pub risk_level: RiskLevel,           // existing type, already Serialize
    pub safety_level: SafetyLevel,       // existing type
    pub builtin_outcomes: Vec<OutcomeSummary>, // angle + verdict + note
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HookResponse {
    pub schema_version: u32,             // 1; unknown major = protocol error
    pub decision: HookDecision,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub repair_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HookDecision { Allow, Warn, Deny }

/// Recorded per hook per run; serialized into the ADR-024 envelope.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HookOutcome {
    pub name: String,
    pub decision: Option<HookDecision>,  // None = hook failed
    pub failure: Option<HookFailure>,    // typed: Spawn|Timeout|NonZeroExit|BadResponse
    pub applied_policy: FailurePolicy,
    pub duration_ms: u64,
    pub reason: Option<String>,
    pub stdout_noise: Option<String>,    // non-JSON stdout, debug only
}
```

Method contracts:

- `ExternalHookAngle::from_config(&HookConfig) -> Result<Self, HookConfigError>`
  — compiles `match`, verifies the binary exists and is executable
  (missing → load-time error, exit 51). The only constructor.
- `impl Validator for ExternalHookAngle` — `validate()` maps
  `Allow → Verdict::Pass`, `Warn → Verdict::Warn`,
  `Deny / (failure + Closed) → Verdict::Fail`,
  `(failure + Open) → Verdict::Warn`; the `HookOutcome` is attached for
  envelope serialization.
- `HookRequest::to_stdin_bytes(&self) -> Vec<u8>` /
  `HookResponse::from_stdout(bytes) -> Result<Self, ResponseError>` — the
  only ser/de sites; `from_stdout` implements last-JSON-line parsing and
  populates `stdout_noise`.

## Exit code / output contract

| Code | Meaning | New? |
|---|---|---|
| 3 | Command blocked — now also by hook `deny`; envelope `blocked_by: "hook:<name>"` + `reason` | reused (ADR-024 `Blocked`) |
| 50 | Hook infrastructure failure under `on_failure = "closed"` (spawn / timeout / non-zero exit / bad response); command **not** executed | new |
| 51 | Hook misconfiguration at load (binary missing, invalid regex, duplicate name) | new |

Non-colliding with the registry: 0–9 (024/027/031/033), 11 (020), 30–32
(030), 40–42 (033). Fail-open hook failures do not change the exit code;
they appear as `hook_outcomes[].failure` with `applied_policy: "open"` —
same honesty-over-silence principle as ADR-033's `CaptureFidelity` and
ADR-034's degraded provenance.

## Files changed (7, no new top-level module)

| # | File | Change |
|---|---|---|
| 1 | `src/config/mod.rs` | Parse `[[hooks.pre_execute]]`; surface `Vec<HookConfig>` on `UserConfiguration` |
| 2 | `src/config/schema.rs` | Schema validation for the hooks table |
| 3 | `src/caroml/validators/external.rs` | New file: all types above + `ExternalHookAngle` |
| 4 | `src/caroml/validators/mod.rs` | `pub mod external;` re-export; append hook angles to chain construction |
| 5 | `src/cli/mod.rs` | Run hook angles in the single-command path; `hook_outcomes` field on `HeadlessEnvelope` (additive, schema-compatible) |
| 6 | `src/main.rs` | Exit-code wiring for 50 / 51 |
| 7 | `tests/hooks_integration.rs` | Integration suite below (fixture hook scripts under `tests/fixtures/hooks/`) |

## Integration tests (known input → deterministic JSON + exit code)

Fixture hooks are tiny POSIX scripts (and one `sleep`), driven through the
ADR-024 headless harness; each test asserts parsed envelope **and**
process exit code:

1. Allow hook → exit 0; `hook_outcomes[0].decision == "allow"`.
2. Deny hook → exit 3; `blocked_by == "hook:deny-writes"`; command not executed.
3. Hook exits 1, `on_failure=closed` → exit 50; typed `failure: "non_zero_exit"`; not executed.
4. Hook exits 1, `on_failure=open` → exit 0; outcome records failure + `applied_policy: "open"`; executed.
5. Hook sleeps past `timeout_ms=100`, closed → exit 50; `failure: "timeout"`.
6. Hook prints logs then JSON → exit 0; decision parsed; `stdout_noise` populated.
7. Hook prints garbage only → exit 50; `failure: "bad_response"`.
8. Binary missing → exit 51 before any generation runs.
9. `match` doesn't match command → hook skipped; exit 0; no outcome entry.
10. Hook returns `allow` for a command the built-in safety validator
    blocks → still exit 3, `blocked_by: "safety"` (hooks cannot loosen).

## What breaks at 100 real users (demoware-trap section)

The fragile assumption is **hook latency and fan-out**: at demo scale one
5 ms script is invisible; a team config with 6 hooks at 2 s each on every
command destroys caro's sub-second value proposition. Failure mode:
users disable hooks wholesale, silently losing policy coverage.
Instrumentation: `duration_ms` per outcome is in the envelope from day
one, and `--debug` prints a per-hook timing line. Mitigations by design:
concurrent dispatch via `run_all`, 5 s default / 60 s ceiling timeouts,
and `match` narrowing. Fallback: a documented `CARO_HOOKS=0` env kill
switch (recorded in the envelope as `hooks_disabled: true` so scripts
can detect policy-off runs).

## Alternatives considered

1. **Replicate the multi-event lattice** (SessionStart, PostToolUse, …) —
   rejected: caro has one choke point; every additional event is contract
   surface without a caller. Revisit `post_execute` when ADR-032 receipt
   consumers ask for synchronous notification.
2. **Shell-form command strings** — rejected: quoting/injection class the
   analog carries for legacy reasons; exec form covers all uses.
3. **Exit-code signaling (exit 2 = block)** — rejected: inherits failure
   modes 1 and 3 verbatim; single JSON channel instead.
4. **WASM plugin runtime** — rejected for v1: heavy dependency; per
   `.claude/rules/external-sdk-integration.md` it would require its own
   build-spike PR first. The subprocess contract doesn't preclude it later.
5. **HTTP hooks** — rejected: ADR-020 already owns the remote channel,
   and the analog shows HTTP policy hooks degrade to fail-open under
   network failure.

## Consequences

Positive: org policy without forking `patterns.rs`; a community layer of
shareable hook scripts in any language; fail-closed defaults consistent
with caro's safety brand; works offline and as a pure subprocess — the
analog's session-coupled design cannot do either; validator-trait reuse
means zero new dispatch machinery.

Negative / accepted: hooks execute arbitrary user-configured binaries
(bounded: user-level config only, exec form, load-time existence check);
per-command latency bounded by the slowest hook; two new exit codes for
downstream scripts to learn; `deny` provenance adds one field to the
ADR-024 envelope (additive, non-breaking).

## Out of scope (next version)

- `post_execute` / observational hooks (receipts cover post-hoc today)
- Project-level (repo-local) hook config + trust gating design
- `ask` decision tier and composition with ADR-020's webhook approvals
- Hook-supplied context injection into prompts
- Async / background hooks; `asyncRewake`-style wake-ups
- Hook distribution/marketplace and a `/hooks`-style TUI browser
- WASM or dynamic-library plugin runtimes
