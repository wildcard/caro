# Implementation Scope — the hook-path latency budget: make slow mean *blocked*, not *allowed*

**Feature under analysis:** **Claude Code's hook system** — specifically the `PreToolUse`
command-hook contract: stdin JSON, the exit-code decision model, `hookSpecificOutput.permissionDecision`,
the per-hook `timeout` field and its **documented fail-open-on-timeout behaviour**. Read live
2026-09-01 at [code.claude.com/docs/en/hooks](https://code.claude.com/docs/en/hooks).
Cross-read against **Kubernetes validating admission webhooks** (`timeoutSeconds` 1–30, default 10,
`failurePolicy: Fail|Ignore`, and the 30-second global pod-admission budget), **Envoy `ext_authz`**
(`failure_mode_allow`, default `false`), and **OPA's `opa bench`** (which deliberately *excludes*
load/parse/compile time from its reported numbers).

**Equivalent we are scoping for Caro:** not a feature. A **measurement, a published number, and a
CI gate that can see it.** Today Caro's validator does ~268 throwaway regex compilations and up to
63 `which` subprocess spawns before it validates a single command; the benchmark file that was
written to catch exactly this is **not wired into `cargo bench` and produces no output**; and the
one CI job that could fail on a latency regression never runs on a PR to `main`. This scope closes
all three, and adds a deterministic gate metric that does not flake on a shared CI runner.

**Date:** 2026-09-01 · **Author:** `caro-research--scoping-process` (autonomous run, no user present)
**Companion ADR:** `docs/adr/ADR-062-hook-path-latency-budget.md`

---

> ### Provenance note — target selection, and the moratorium it respects
>
> The scheduled task's template leaves `[FEATURE NAME]` unbound and no user was present. Target
> selection was mine, and it was constrained three ways:
>
> 1. **ADR-059 (2026-08-26) declared itself "the last ADR in this space until
>    `src/safety/assessment.rs` merges."** Re-verified today: `ls src/safety/` returns exactly
>    `cve_patterns.rs`, `mod.rs`, `patterns.rs`. The moratorium **still holds** and covers the
>    assessment/governance surface, ADR-035 → ADR-059. **This scope is outside it.** It mints no
>    policy vocabulary, no verdict payload, no approval exchange, no lifecycle event, and — see
>    §3.3 — **no new exit code**. It adds one integer field to an already-serialized struct and
>    deletes work from a hot path.
> 2. **Today's strategy memo** (`.hermes/digests/2026-09-01-agent-launch-scan.md`) makes the frozen
>    assessment schema its single "build or test next" item. This scope is **not** that item, and
>    does not pretend to be. It is the memo's **Recommendation #2** — §3.3, *"Publish a hook-path
>    latency SLO and gate it in CI,"* rated **Priority: Now, Complexity: S** — chosen because
>    Recommendation #1 is inside the moratorium and Recommendation #3 is a weeks-long interview
>    programme, neither of which an autonomous overnight run should start.
> 3. **ADR-036 (2026-07-14) already scopes `caro guard`,** the PreToolUse adapter itself. This ADR
>    does **not** re-scope it, redesign it, or block on it. It scopes the budget that adapter will
>    have to live inside, and every change here pays off with or without ADR-036 landing.
>
> ### What this scope claims and does not claim
>
> **Claims, and defends in Phase 2 with greppable receipts:** four `SafetyValidator::new()`
> constructions per invocation; 67 throwaway regex compilations per construction; allowlist regexes
> recompiled inside the per-call hot loop; `benches/performance.rs` absent from `Cargo.toml`'s
> `[[bench]]` list and therefore emitting nothing; `benchmarks.yml` not triggered by PRs to `main`;
> zero timing instrumentation anywhere in `src/safety/`; the `SafetyValidation` telemetry event
> defined, redaction-tested, and never emitted.
>
> **Does not claim:** that any of this is currently *slow enough to matter*. **Nobody has measured
> it.** That is the point — §3.1's first deliverable is the measurement, and §3.5 sets the budget
> to a number the code already meets, per the memo's explicit instruction *"Do not tune first —
> publish the honest number, then improve it."* If the honest number turns out to be 400 µs, the
> waste catalogued in Phase 2 is a cleanup item and not an emergency, and this document should be
> read that way.
>
> ### Verification status
>
> - **Verified against the Caro tree (2026-09-01, branch `integrator/20260711-postmerge`,
>   `Cargo.toml` version `1.4.0`, `rust-version = "1.85"`, HEAD `50859b89`):** every file path,
>   line number, struct field, derive, count and absence claim in Phase 2. Line numbers drift; each
>   claim is written so it can be re-checked with one `grep`, and the greps are given in §2.9.
> - **Verified by live fetch (2026-09-01), quoted verbatim in Phase 1:** the Claude Code hooks
>   reference — common-fields `timeout` row, the Timeouts section, the exit-code model, the
>   `PreToolUse` `hookSpecificOutput` example, and the `PreToolUse` stdin payload.
> - **Verified by search summary only, not by fetching the primary doc:** the Kubernetes
>   `timeoutSeconds` 1–30 range and 10 s default, the 30 s global pod-admission budget, and Envoy's
>   `failure_mode_allow` default of `false`. These are cross-reads that corroborate a pattern; no
>   decision in Phase 3 depends on any of them being exactly right.
> - **Second-hand, from today's memo, load-bearing nowhere:** agentOS's claimed 4.8 ms cold start,
>   Kastra's claimed sub-1 ms authorization, oMLX's 90 s → 5 s claim. All three are vendor figures
>   on launch-day pages. They are quoted once, in §1.6, as evidence that *publishing a number* is
>   now table stakes — not as benchmarks to beat.
> - **Explicitly NOT verified:** whether `caro`'s actual p99 on the validation path is single-digit
>   microseconds or double-digit milliseconds. No measurement was taken. `cargo bench` was not run
>   — this session had no build environment for the repo and running it would have been the wrong
>   call anyway, since §3.1 is precisely the PR that makes the measurement meaningful.
>
> ### Process compliance
>
> - **`.claude/rules/git-workflow.md`** — this file and the ADR are written **uncommitted** on the
>   working tree. Nothing was committed, to `main` or to the current branch. A human branches and PRs.
> - **`.claude/rules/adr-numbering.md`** — ADR-062 is the next sequential number; ADR-061 is the
>   highest present in `docs/adr/`. Renumber on merge if another 062 lands first.
> - **`.claude/rules/validation-discipline.md`** — Gate 3 (demoware trap) is discharged at §3.7.
>   Gate 4 (`devils-advocate` review) is **required before the implementation PR opens** and is not
>   discharged here. Gates 1, 2 and 5 do not attach: this is instrumentation and CI for an existing
>   shipped capability, which is the rule's own carve-out for "responses to evidence we already have."
> - **`.claude/rules/good-boy-scout.md`** — §3.6 is scoped to files this PR already touches. It
>   does not refactor the pattern library, the backends, or the telemetry schema beyond one field.

---

# Phase 1 — Feature research

## 1.1 The problem, and who has it

A safety layer that sits in the execution path of an agent is not judged on accuracy first. It is
judged on whether anyone leaves it installed. There are two ways to lose:

- **Visible:** the layer adds perceptible delay to every tool call, the user notices, the user
  uninstalls it. This is the failure mode everyone talks about.
- **Invisible, and much worse:** the layer is slow enough to hit its host's timeout, the host
  **cancels it and proceeds anyway**, and the user keeps the layer installed believing they are
  protected. The gate is not enforcing. Nothing tells them.

The second failure mode is the reason this document exists, and it is not hypothetical — it is the
*documented, intended* behaviour of the host Caro most wants to plug into.

## 1.2 Claude Code's `PreToolUse` contract, quoted

**The hook is a pure subprocess.** From the hooks reference: command hooks *"run a shell command.
Your script receives the event's JSON input on stdin and communicates results back through exit
codes and stdout."* The `PreToolUse` stdin payload is:

```json
{
  "session_id": "abc123",
  "prompt_id": "550e8400-e29b-41d4-a716-446655440000",
  "transcript_path": "/home/user/.claude/projects/.../transcript.jsonl",
  "cwd": "/home/user/my-project",
  "permission_mode": "default",
  "hook_event_name": "PreToolUse",
  "tool_name": "Bash",
  "tool_input": { "command": "npm test", "description": "Run test suite", "timeout": 120000, "run_in_background": false },
  "tool_use_id": "toolu_01ABC123..."
}
```

**The decision contract** is either an exit code or a JSON object on stdout:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "Database writes are not allowed"
  }
}
```

`permissionDecision` accepts `allow` / `deny` / `ask` / `defer`. Exit 2 blocks unconditionally:
*"exit 2 blocks whether or not you print JSON: even a JSON `permissionDecision` of `"allow"` can't
override it."* Exit 1 does **not** block — the docs call this out in a warning: *"Without valid
JSON on stdout, Claude Code treats exit code 1 as a non-blocking error and proceeds with the
action, even though 1 is the conventional Unix failure code. If your hook is meant to enforce a
policy, use `exit 2`."*

**The timeout, and the fail-open.** The `timeout` common field: *"Seconds before canceling…
Defaults: 600 for `command`, `http`, and `mcp_tool`; 30 for `prompt`; 60 for `agent`."* And then,
under **Timeouts**, the sentence this whole scope turns on:

> *"Claude Code cancels a `command`, `http`, or `mcp_tool` hook that reaches its `timeout`,
> discarding the hook's output, so on most events a timed-out hook renders no decision."*
>
> *"On `PreToolUse` … A timed-out `command`, `http`, or `mcp_tool` hook **doesn't block the tool
> call**. The call continues through the normal permission flow, so **don't count on a stalled hook
> to act as a gate**."*

There is an asymmetry worth recording: an **Agent SDK callback hook** that exceeds its timeout
*does* block the tool call. The in-process integration fails closed; the subprocess integration
fails open. Caro is a subprocess. Caro is therefore on the fail-open side of that line, by
construction, and cannot opt out of it through configuration.

**Consequence.** For a subprocess safety gate, **latency is not a UX property. It is a safety
property.** Every millisecond of p99 is a millisecond of probability mass that the gate silently
stops being a gate. A 600-second default makes a *timeout* unlikely for a healthy validator — but
"unlikely" is a statement about a distribution nobody has measured, and users routinely lower
`timeout` (the docs' own example config sets `"timeout": 30`), and the tail is where hooks that
stat a cold NFS home directory or spawn 63 subprocesses actually live.

## 1.3 Why the pattern is not new: admission control has solved this three times

Every mature in-path authorization system in the survey has the same three-part answer — a
**bounded budget**, an **explicit failure policy**, and a **published expectation**. Caro currently
has none of the three.

| System | Budget | Failure policy | Published expectation |
|---|---|---|---|
| **Claude Code `PreToolUse`** | per-hook `timeout`, default 600 s (30 s on `UserPromptSubmit`, 10 s on `MessageDisplay`) | **fail-open**, not configurable, documented | none stated |
| **K8s validating webhook** | `timeoutSeconds`, range 1–30, default 10; global pod-admission budget 30 s | `failurePolicy: Fail` (default) or `Ignore` — **configurable** | community guidance: responses *"well under 1 second"* |
| **Envoy `ext_authz`** | filter timeout | `failure_mode_allow`, **default `false`** i.e. fail-closed | none stated |
| **Caro today** | — | — | — |

Two of the three let the operator choose fail-closed. The one Caro is targeting does not. That
removes the usual mitigation and leaves exactly one lever: **be fast enough that the question never
arises, and be able to prove it.**

## 1.4 The structured-output contract, and what a machine depends on

Across all four systems the machine-readable contract is small and stable, and it separates the
*decision* from the *diagnostics*:

- Claude Code: `hookSpecificOutput.{hookEventName, permissionDecision, permissionDecisionReason}`
  on stdout, plus the exit code. Anything else on stdout is either parsed as JSON or, on the events
  that support it, injected as plain-text context — and a parse failure is *"a non-blocking error
  on every exit code other than 2."* Malformed output fails open too.
- K8s: `AdmissionReview.response.{uid, allowed, status.message}`.
- Envoy: a gRPC `CheckResponse` with an OK/Denied status.
- OPA `opa bench`: `ns/op`, `B/op`, `allocs/op`, optional `timer_rego_query_eval_ns` under
  `--metrics`, in `pretty` / `json` / `gobench` formats.

The last one is the one this ADR borrows from, and §1.5 explains why it borrows it **inverted**.

## 1.5 Session/context lifecycle: how each avoids redundant initialization — and where Caro cannot follow

This is the template question that turned out to matter most, because the honest answer for Caro is
*"it can't."*

- **K8s webhooks and Envoy `ext_authz`** are **long-lived servers**. Policy compilation, TLS setup
  and cache warming happen once at process start and are amortised over every subsequent decision.
  Per-request cost is matching only.
- **OPA** offers both shapes — `opa run --server` (warm) and `opa eval` (one-shot) — and its
  benchmark tooling encodes the distinction explicitly. The `opa bench` docs: *"These results
  capture metrics of sample runs, where **only the query evaluation is measured**. All time spent
  preparing to evaluate (loading, parsing, compiling, etc.) is omitted."*
- **Claude Code command hooks** are **cold subprocesses**. There is no warm mode, no daemon, no
  connection reuse. Every `PreToolUse` firing is a fresh `fork`/`exec`, a fresh address space, a
  fresh everything. The only amortisation available is `async: true`, which the docs exclude from
  timeout enforcement — and an async hook cannot gate a tool call, so it is not available to a
  safety layer.

**The inversion, stated plainly.** OPA is right to exclude prep time, because OPA's users run a
server and prep happens once. Caro measuring the same way would be **measuring the wrong thing** —
for a cold-subprocess gate, prep time is not overhead around the measurement, prep time **is** the
measurement. `benches/performance.rs` currently does the OPA thing (§2.4) while Caro lives in the
Claude Code world, and that mismatch is the single most consequential design error this scope
corrects.

**This also settles a tempting non-answer.** The obvious fix for cold-start cost is a daemon: keep
a warm `caro` process, talk to it over a socket, amortise the regex compilation. This scope refuses
it, and not only because the task constraints forbid it. A daemon converts a fail-open timeout into
a fail-open *connection error* — the same failure with more moving parts, a socket path, a stale-PID
problem, and a new privilege boundary. The correct answer for a subprocess gate is to make the cold
path cheap enough that warmth is unnecessary.

## 1.6 Why now: the market started publishing numbers

From today's strategy memo, and flagged there and here as **unverified vendor marketing**:
agentOS leads its Apache-2.0 launch with a claimed **~4.8 ms cold start** and ~22 MB per instance;
Kastra markets **sub-1 ms** authorization; oMLX took 96 points and 18 comments for a claimed
**90 s → 5 s** improvement in agent wait time. Nobody on this project has installed any of them and
none of these numbers has been reproduced.

What survives that discount is not a benchmark to beat, it is a **norm**: in this category, a
latency figure is now part of the pitch. The memo's framing is the right one — *"Caro has the fast
architecture and no published number."* A deterministic 67-regex matcher should win this argument
comfortably. It currently cannot enter it.

## 1.7 Failure modes found in Phase 1, to be designed against

| # | Failure mode | Source | Designed against in |
|---|---|---|---|
| **F1** | **Timeout ⇒ silent fail-open.** A slow hook does not block; nothing surfaces to the user. | Claude Code hooks reference, Timeouts | §3.1, §3.5 — bound and publish the cold path |
| **F2** | **Exit 1 ⇒ fail-open.** The conventional Unix failure code is non-blocking. A panicking or misconfigured validator silently allows. | Claude Code hooks reference, exit-code warning | §3.3 — no new exit codes, and ADR-036 owns the adapter's use of exit 2 |
| **F3** | **Malformed stdout ⇒ fail-open.** A JSON parse failure is a non-blocking error on every exit code except 2. | Claude Code hooks reference, Exit code 0 | §3.3 — the timing field is additive and `#[serde(default)]` |
| **F4** | **Measuring warm when you run cold.** Benchmarks that exclude construction report a number the deployment never experiences. | `opa bench` docs, contrasted | §3.1, §3.2 — cold-start bench is the headline metric |
| **F5** | **A wall-clock gate on shared CI flakes**, gets marked `continue-on-error`, and stops gating — the exact fate of `safety-validation.yml`'s behavioural jobs (§2.6). | Caro's own tree | §3.4 — a deterministic count metric alongside the wall clock |

---

# Phase 2 — Competitive differentiation: what the Caro tree actually looks like

Every claim in this phase was produced by reading the tree at HEAD `50859b89` on 2026-09-01. The
re-check greps are collected in §2.9.

## 2.1 What they get right that we should replicate

1. **A budget is a number in a file, not a paragraph in a README.** K8s puts `timeoutSeconds` in
   the webhook object; Envoy puts `failure_mode_allow` in the filter config. Caro should put its
   budget in a checked-in manifest that the CI gate reads (§3.2), so changing the promise is a
   reviewable diff.
2. **Separate the decision contract from the diagnostics contract.** Claude Code's
   `permissionDecision` is four values and never grows; the metrics live elsewhere. Caro's timing
   number must therefore go in `TimingInfo`, next to the timings that already exist — **not** into
   the decision payload, which is frozen-by-moratorium anyway.
3. **Measure what the deployment does.** OPA excludes prep because OPA is warm. Caro must include
   prep because Caro is cold. Same principle, opposite implementation.

## 2.2 Their gaps we can avoid by designing the schema first

- Claude Code's timeout fail-open is **not configurable** — there is no `failurePolicy: Fail`. Caro
  cannot fix that in someone else's product, but it can refuse to be the reason the timeout is
  reached, and it can make the cost of *not* being fast visible in CI.
- OPA's benchmark output has no notion of a *budget*: `opa bench` reports, it does not assert.
  Nothing in `opa bench` fails a build. Caro already owns half of the missing half — a working
  comparison script and a hard-fail step (§2.6) — and needs only to point it at the right metric.
- None of the four publish an **allocation or compilation count**, only wall-clock. Wall-clock is
  exactly the metric that flakes on shared CI. §3.4 adds the deterministic one.

## 2.3 Existing infrastructure that already covers part of this

Substantial, and this is why the memo rated the item **Complexity: S**:

| Asset | Location | State |
|---|---|---|
| Criterion, MSRV-pinned | `Cargo.toml:155` — `criterion = { version = "=0.7.0", features = ["html_reports"] }`, pinned because *"0.8+ requires rustc 1.86, above MSRV 1.85"* | Working, 4 benches wired |
| Benchmark comparison script | `scripts/benchmark-compare.py` (9 195 B), plus `benchmark-aggregate.py`, `benchmark-monthly-aggregate.py` | Present, executable |
| A CI job that can hard-fail | `.github/workflows/benchmarks.yml` — baseline checkout, `--save-baseline` / `--baseline`, `jq` on `regression-report.json`, and a `Fail on regression` step that runs `exit 1`; **zero `continue-on-error` in the file** | Working; wrong triggers (§2.6) |
| Serialized timing struct | `src/cli/mod.rs:125-130` — `TimingInfo { generation_time_ms, execution_time_ms, total_time_ms }`, reached through `CliResult` (`src/cli/mod.rs:75-101`, `#[derive(Debug, Clone, Serialize, Deserialize)]`) and emitted by `-o json` at `src/main.rs:3629` | Working; has no validation field |
| A telemetry event for exactly this | `src/telemetry/events.rs:85` — `EventType::SafetyValidation { risk_level: String, action_taken: String, pattern_category: Option<String> }`; schema'd, storage-tagged (`src/telemetry/storage.rs:89`), redaction-tested (`src/telemetry/redaction.rs:299`) | **Never emitted.** Zero production emit sites — the only three references in `src/` are a display arm, a storage tag, and a unit test. Carries no duration field |
| Compiled-pattern cache | `src/safety/patterns.rs:551` — `pub static COMPILED_PATTERNS: Lazy<Vec<CompiledPattern>>`; `src/safety/cve_patterns.rs:28,55` for the CVE ruleset | Working, and the reason the *steady-state* path is probably already fast |

So: the cache exists, the bench framework exists, the gate mechanism exists, the serialized struct
exists, the telemetry event exists. Almost nothing here is new construction. Four things are
disconnected from each other, and one hot path does avoidable work.

## 2.4 Finding 1 — the safety benchmarks produce no output at all

`benches/performance.rs` (318 lines) defines nine Criterion benchmarks: `cli_startup`,
`safety_validation_single`, the `validation_comparison` group (`individual_validation`,
`batch_validation`), the `safety_levels` group, `concurrent_validation`,
`large_command_validation`, the `shell_types` group, `pattern_matching`, and
`sustained_validation_load`, wired with `criterion_group!` at `:305-316` and `criterion_main!` at
`:318`.

`Cargo.toml:161-176` declares exactly four `[[bench]]` targets — `cache`, `config`, `context`,
`logging` — each with `harness = false`. **There is no `[[bench]] name = "performance"` entry**, and
there is no `autobenches = false` key anywhere in `Cargo.toml`. So `benches/performance.rs` is
auto-discovered with the **default `harness = true`**, libtest supplies `main`, `criterion_main!` is
never the entry point, and the file contains zero `#[bench]` and zero `#[test]` functions.

**Net effect: no safety-validation benchmark ID has ever reached `target/criterion`.** The IDs that
do are `cache/*` (4), `config/*` (4), `context/*` (2) and `logging/*` (3). The regression gate in
§2.6 can only compare IDs Criterion produced, so it has never had a validation number to compare.

The file's own comments make this sharper. Line 1: `// Performance benchmarks - THESE MUST FAIL
INITIALLY (TDD)`. Line 14: `// Benchmark CLI startup time (<100ms requirement)`. Line 26:
`// Benchmark safety validation performance (<100ms per command)`. **Caro already has a latency
requirement. It is written in a comment, in a file that does not run.**

## 2.5 Finding 2 — the cold path does a large amount of avoidable work

**Four validators per invocation.** `SafetyValidator::new()` is called at `src/cli/mod.rs:270`,
`src/backends/embedded/embedded_backend.rs:86`, again at `:131` via `.with_safety_config(...)`
(called from `src/cli/mod.rs:326`), and at `src/backends/static_matcher.rs:131`.

**Each construction throws away 67 compiled regexes.** `new()` opens with
`patterns::validate_patterns()` (`src/safety/mod.rs:346`). That function
(`src/safety/patterns.rs:512-529`) iterates `DANGEROUS_PATTERNS` and calls `Regex::new(&pattern.pattern)`
for every entry — and **collects only the error strings**. Every successfully compiled `Regex` is
dropped on the spot. It is a validity check whose entire product is discarded, run at runtime, on
every construction, over a static table that is validated in CI already (`safety-validation.yml`'s
`pattern-compilation` job runs `cargo test --lib safety::patterns` and hard-fails).

67 is the real count. `sed -n '13,507p' src/safety/patterns.rs | grep -c 'DangerPattern {'` → **67**
(30 Critical, 20 High, 17 Moderate). The module doc at `src/safety/mod.rs:8` says *"52 pre-compiled
regex patterns"*; `src/caroml/validators/safety.rs:1` says *"52+"*; `CLAUDE.md` and `README.md` say
52+. **All stale.** Plus 2 CVE patterns embedded by `build.rs` from `data/cve_rules/`
(`CVE-2021-3156.yaml`, `CVE-2024-3094.yaml`; `EXAMPLE-TEMPLATE.yaml` is filtered out at
`src/dogma/compiler.rs:214`).

**4 × 67 = 268 regex compilations per process, all discarded**, before the one real
`validate_command` call at `src/cli/mod.rs:778-784`.

**Allowlist regexes are compiled inside the per-call hot loop.** `src/safety/mod.rs:500`:

```rust
for allow_pattern in &self.config.allowlist_patterns {
    if let Ok(regex) = regex::Regex::new(allow_pattern) {
```

Compiled fresh on every `validate_command`, cached nowhere. A user with a 20-entry allowlist pays
20 regex compilations per validated command — and pays them *again* on the next command. Note also
that the failure is silent: `if let Ok(...)` means a malformed allowlist entry is skipped without a
warning, which is a correctness bug adjacent to, but out of scope for, this PR (filed in §3.6 as a
follow-up rather than fixed here).

**Two `Vec` allocations per call for the pattern views.** `get_compiled_patterns_for_shell`
(`src/safety/patterns.rs:568-577`) ends in `.collect()`, allocating a fresh ~63-element `Vec` of
references on every call; `get_cve_compiled_patterns_for_shell` (`src/safety/cve_patterns.rs:85-92`)
allocates a second. Both are pure functions of `shell`, of which there are six.

**Up to 63 subprocess spawns per invocation.** `CliApp::with_overrides` calls
`ExecutionContext::detect()` (`src/cli/mod.rs:275` → `src/context/mod.rs:39-51`), which calls
`scan_available_commands()` (`:194`), which filters a hardcoded list of 63 command names through
`command_exists` — and `command_exists` (`src/context/mod.rs:270-276`) is:

```rust
Command::new("which").arg(command).output()
```

That is up to 63 `fork`/`exec` pairs, unconditionally, on every invocation, before any validation.
On a warm local machine this is milliseconds. On a container with a cold page cache, or a home
directory on a network filesystem, it is the tail. **This is the single largest suspected
contributor to cold-start p99, and it has nothing to do with safety at all.**

**Config is read from disk twice.** `ConfigManager::new()` + `load()` at `src/main.rs:3325-3328`
(`std::fs::read_to_string` at `src/config/mod.rs:133`), then a second `ConfigManager` constructed
and `load()`ed at `src/cli/mod.rs:205-214`.

**Telemetry opens SQLite before validation.** `TelemetryStorage::new` at `src/main.rs:3379`, an
optional background uploader `.start()` at `:3391-3395`, and a `SessionStart` event at `:3401-3409`.

**And there is no fast path.** There is no `caro validate` or `caro check-command` subcommand —
the `Commands` enum (`src/main.rs:380-680`) has no such variant. `caro check <file>` is a CaroML
lint that performs **no safety validation** (`grep -rn 'safety' src/caroml/runner.rs` → 0 hits).
`CliApp::with_overrides` **always** constructs a backend (`src/cli/mod.rs:258` → `:322`,
`EmbeddedModelBackend::new()`), which resolves the cache dir, consults `ModelCatalog`, and stats
bundled and cached model paths — even when the requested operation cannot possibly need a model.
Model *weights* are not loaded (`CpuBackend::new` stores `Arc<Mutex<None>>`), so this is stat cost,
not load cost — but it is stat cost on the gate path.

## 2.6 Finding 3 — the gate that works is aimed somewhere else

`.github/workflows/benchmarks.yml` is a **good** workflow. Triggers (`:13-37`):

```yaml
pull_request:
  branches: ['release/**']
schedule:
  - cron: '0 0 * * 0'
workflow_dispatch:
  inputs: { baseline_ref: main, threshold_time: 15, threshold_memory: 20 }
```

It checks out the baseline ref, runs `cargo bench -- --save-baseline baseline`, returns to the PR
head, runs `cargo bench -- --baseline baseline`, feeds `target/criterion` to
`scripts/benchmark-compare.py`, and at `:184-190` has a real gate:

```yaml
- name: Fail on regression
  if: steps.analyze.outputs.has_regression == 'true'
  run: ... exit 1
```

Two problems, both structural:

1. **It does not run on the PRs that matter.** `pull_request: branches: ['release/**']` means a PR
   to `main` never triggers it. A latency regression merges to `main` and is discovered at release
   time, by a job that also has to bisect it.
2. **It has nothing to compare.** Per §2.4, no validation benchmark ID exists in `target/criterion`.

`.github/workflows/ci.yml` — the workflow that *does* run on every PR — has **zero** occurrences of
`bench`. Its only numeric gate is binary size (`if [ $SIZE -gt 52428800 ]; then ... exit 1`).

`.github/workflows/safety-validation.yml` is correctness-only, and contains the cautionary tale for
§3.4: its `static-matcher-tests` and `embedded-backend-tests` jobs end in

```
|| { echo "⚠️ Some ... tests failed"; echo "This is informational ..."; exit 0; }
```

A flaky assertion was neutralised rather than fixed. Its `pr-comment` job then posts a hardcoded
all-✅ template regardless of upstream results. **Any latency gate this ADR adds will meet the same
pressure the first time it flakes**, and must be designed so that it doesn't.

## 2.7 Finding 4 — nothing measures validation, anywhere

`grep -rn 'Instant\|elapsed' src/safety/` → **zero hits.** The validator has never been timed in
process.

In `src/cli/mod.rs`, `start_time` (`:719`) brackets the whole request and `gen_start` (`:767`)
brackets LLM generation only, ending at `:775`. The validation call sits at `:778-784` — **between**
`generation_time` being taken and the approval branch — with no `Instant` around it and no field to
put one in. `TimingInfo` (`:125-130`) has `generation_time_ms`, `execution_time_ms`,
`total_time_ms`, and no `validation_time_*`.

`EventType::SafetyValidation` (`src/telemetry/events.rs:85`) carries `risk_level`, `action_taken`,
`pattern_category` — and no duration. It is emitted from nowhere: the only production emit sites in
`src/` are `CommandGeneration` (`src/agent/mod.rs:147, 232, 374`) and `SessionStart`
(`src/main.rs:3401`). Its display arm lives in `src/cli/telemetry.rs:150`, inside `handle_telemetry`,
whose only call site is **commented out** at `src/main.rs:3271`.

`src/logging/`'s `OperationSpan` (`src/logging/mod.rs:232-241`) is `pub struct OperationSpan { _name: String }`
with a `new` and no `Drop`. It records nothing.

## 2.8 Our unique positioning — what we can do that they cannot

- **Deterministic regex matching has no tail that depends on a model.** K8s webhooks and Envoy
  `ext_authz` call out to a network service; Caro's steady-state match is CPU-bound over 69
  pre-compiled regexes with a bounded input (`max_command_length`). That is the architecture that
  wins the argument in §1.6 — and it is the reason today's memo's §2.7 concludes that an LLM call
  on the hook path is *"disqualifying"* and the deterministic matcher is *"the only viable one."*
- **We can gate on a count, not just a clock.** Because the pattern set is static and the matcher is
  deterministic, "number of regex compilations performed during one validation" is a fixed integer
  for a given config. A competitor whose evaluation depends on a model or a network hop cannot
  assert on such a number. Caro can, and §3.4 does — which is what makes the CI gate survive
  contact with a noisy shared runner.
- **Offline, no daemon, no network** — the constraint that forces the cold path to be cheap is the
  same constraint that makes the number reproducible on the user's own machine.

---

# Phase 3 — Scope definition

## 3.0 What ships

One PR, titled `perf(safety): hook-path latency budget, published SLO, and a CI gate that can see it`.

1. Wire `benches/performance.rs` into `cargo bench` and fix its warm/cold conflation.
2. Add `benches/hook_path.rs` — the cold-start benchmark that models the actual deployment.
3. Delete the avoidable work from the construction and validation paths.
4. Instrument validation with one integer, surfaced through the existing `-o json` envelope.
5. Add `docs/slo/hook-path.json` — the budget, as a checked-in, versioned, machine-read manifest.
6. Add `scripts/slo-check.py` and a fast SLO job that runs on **every PR to `main`**.
7. Fix the 52 → 67 drift in the four places that state it.

## 3.1 Deliverable A — measure the right thing

**`benches/hook_path.rs` (new file, ~120 lines, no new module).** Four Criterion benchmarks, named
so the gate can find them:

| Bench ID | What it measures | Why |
|---|---|---|
| `hook_path/cold_validate` | `SafetyValidator::new(SafetyConfig::moderate())` **plus** one `validate_command` — in one closure, deliberately | The number a `PreToolUse` subprocess actually pays. **This is the headline metric.** |
| `hook_path/validator_new` | construction alone, hoisted out of validation | Isolates the 67-compile cost so §3.3's improvement is attributable |
| `hook_path/validate_warm` | `validate_command` with the validator hoisted **outside** `b.iter` | The steady-state matcher cost — the OPA-style number, kept for diagnosis, **not** for the SLO |
| `hook_path/validate_allowlist_20` | warm validation with a 20-entry allowlist | Directly exercises the §2.5 hot-loop compile; regresses loudly if it comes back |

`Cargo.toml`: add `[[bench]] name = "hook_path" \n harness = false` **and** `[[bench]] name = "performance" \n harness = false`.

**`benches/performance.rs` (edit).** All nine benchmarks currently construct a validator *inside*
`b.iter` (`:47, 85, 100, 135, 168, 195, 226, 255, 276`), so each conflates construction with
matching — `safety_validation_single`, for instance, reports `1 × new()` + `10 × validate_command`
as one number. Hoist construction out of the closure in all nine. The cold number is not lost; it
moves to `hook_path/cold_validate`, where it is labelled.

**Explicitly out of this deliverable:** an end-to-end `Command::new("caro")` subprocess benchmark.
It would measure the linker, the dynamic loader, and the CI runner's page cache as much as Caro,
and Criterion's statistics are not built for a ~10 ms unit of work with that much external variance.
§3.8 keeps it as a v2 item behind `hyperfine`, which is the right tool for it.

## 3.2 Deliverable B — the budget as a checked-in artifact

**`docs/slo/hook-path.json` (new file).** Serializable from day one, versioned from day one, and
read by CI rather than by a human:

```json
{
  "schemaVersion": "1",
  "slo": [
    {
      "id": "hook_path/cold_validate",
      "metric": "wall_clock_p95_us",
      "budget": 2000,
      "enforcement": "ceiling",
      "rationale": "Cold-subprocess validation for one command. Ceiling, not a regression %, so a noisy runner cannot flake the gate."
    },
    {
      "id": "safety.regex_compilations_per_validation",
      "metric": "count",
      "budget": 0,
      "enforcement": "exact",
      "rationale": "Deterministic. Machine-independent. Catches a reintroduced compile-in-the-hot-loop even when the clock says nothing."
    },
    {
      "id": "safety.validator_constructions_per_invocation",
      "metric": "count",
      "budget": 1,
      "enforcement": "ceiling",
      "rationale": "Four today (scope §2.5). Ratchet, do not rewrite the CLI."
    }
  ]
}
```

**The budget numbers above are placeholders and the PR must replace them.** Per the memo's
instruction — *"Set the gate at a number the current code already meets… Do not tune first —
publish the honest number, then improve it"* — the procedure is: land the benches, run
`cargo bench --bench hook_path` on CI three times, take the worst p95, round **up** generously, and
commit that. A gate set below the current number is a broken build on day one; a gate set at 10× the
current number still catches the regressions that matter, because real latency regressions on this
path are order-of-magnitude events (a network call, a model load, a compile in a loop), not 15%
drifts.

**A note on `validator_constructions_per_invocation: 1`.** Getting from four to one means changing
who owns the validator across `CliApp`, `EmbeddedModelBackend` and `StaticMatcher` — that is a
refactor with real blast radius and it is **out of scope for this PR** (§3.8). The manifest entry
ships with `budget: 4` and a `TODO` comment referencing the follow-up bead. Ratchet down later; do
not let an aspirational entry red the build.

## 3.3 Deliverable C — new types and field contracts

Three additive changes. **No new module. No new struct in a new file. No new exit code.**

**C1 — `TimingInfo` gains one field.** `src/cli/mod.rs:125-130`:

```rust
pub struct TimingInfo {
    pub generation_time_ms: u64,
    pub execution_time_ms: u64,
    pub total_time_ms: u64,
    /// Microseconds spent inside `SafetyValidator::validate_command`, excluding
    /// construction. Absent in envelopes produced before ADR-062.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_time_us: Option<u64>,
}
```

`Option` + `#[serde(default)]` + `skip_serializing_if` is deliberate and is the F3 mitigation from
§1.7: an existing consumer that deserializes `CliResult` keeps working, and an emitted envelope that
predates the field is not malformed. **Microseconds, not milliseconds** — a matcher this fast
rounds to `0 ms` and an SLO you cannot resolve is not an SLO. Populated at `src/cli/mod.rs:778-784`
by an `Instant` around the existing call and threaded into the `TimingInfo` construction at `:946-950`.

**C2 — `EventType::SafetyValidation` gains one field.** `src/telemetry/events.rs:85`:

```rust
SafetyValidation {
    risk_level: String,
    action_taken: String,
    pattern_category: String,
    #[serde(default)]
    duration_us: u64,
},
```

**This PR does not start emitting the event.** The variant has been dark since it was written
(§2.7); waking it up is a telemetry-consent question with its own redaction review, and it is not
this PR's business. The field is added so that when someone does wake it, the duration is already
in the schema and no second migration is needed. Storage tagging (`src/telemetry/storage.rs:89`)
and redaction (`src/telemetry/redaction.rs:299`) are unaffected — a `u64` duration carries no PII.

**C3 — `SafetyValidator` gains one private field.** `src/safety/mod.rs:154-162`:

```rust
pub struct SafetyValidator {
    config: SafetyConfig,
    patterns: Vec<DangerPattern>,
    compiled_patterns: Vec<(regex::Regex, RiskLevel, String)>,
    /// Allowlist patterns compiled once at construction. Was compiled per
    /// `validate_command` call before ADR-062. Not serialized; `Regex` is not
    /// `Serialize` and this is derived state, not configuration.
    compiled_allowlist: Vec<(regex::Regex, String)>,
}
```

Populated in `new()` from `config.allowlist_patterns`; consumed at `src/safety/mod.rs:498-513` in
place of the inline `Regex::new`. `SafetyValidator` derives only `Debug` today, so this adds no
serialization obligation — and per the task's *"all new types must be serializable from day one"*
constraint: `LatencySlo` (§3.2, the manifest) and the two report shapes in §3.4 **are** serializable
from day one; `compiled_allowlist` is not a new type, it is a private cache of derived state on an
existing non-serialized struct, and making a compiled `Regex` serializable would be nonsense.

**Behaviour change to note in the PR body:** compiling the allowlist at construction turns a
malformed allowlist entry from a silent per-call skip into something `new()` can see. **This PR keeps
the lenient behaviour** — skip and continue — because changing a validator from lenient to strict is
a safety-semantics change that belongs in the moratorium's territory, not in a perf PR. It adds a
`tracing::warn!` on the skipped entry, which is strictly more information than today's silence.

## 3.4 Deliverable D — the gate, and why it will not be neutralised

**The problem, from Caro's own history.** `safety-validation.yml` has two jobs whose assertions were
replaced with `|| { echo "informational"; exit 0; }` (§2.6). That is what happens to a flaky gate.
A p95 wall-clock assertion on a shared GitHub runner *will* flake — noisy neighbours, thermal
throttling, and runner-class drift move microbenchmarks by multiples, not percentages.

**The design answer: two metrics with different failure characters.**

1. **A wall-clock *ceiling*, set generously** (§3.2). Not a percentage regression against a baseline —
   percentage gates are precisely the ones that flake, because they compare two noisy measurements
   instead of one noisy measurement against a fixed constant. A 10×-headroom ceiling ignores runner
   noise and still catches every regression that actually matters on this path.
2. **A deterministic count**, which cannot flake at all. `safety.regex_compilations_per_validation`
   is asserted by a **unit test**, not a benchmark: a test-only counter behind
   `#[cfg(any(test, feature = "slo-counters"))]`, incremented at the `Regex::new` call sites in
   `src/safety/`, asserted to be `0` across a validation with a populated allowlist. That test runs
   in `ci.yml`'s existing `unit-tests` job, on every PR, in milliseconds, with zero variance.

The count metric is the load-bearing one. If the wall-clock ceiling ever gets neutralised under
pressure, the count assertion still catches the specific class of regression this scope exists to
prevent — someone reintroducing a compile inside a loop — and it catches it as a normal failing
test, which nobody argues with.

**`scripts/slo-check.py` (new, ~120 lines, mirrors `scripts/benchmark-compare.py`).**

```
usage: slo-check.py --criterion-dir target/criterion --slo docs/slo/hook-path.json
                    --output slo-report.json
```

Emits `slo-report.json`, schema-versioned from day one:

```json
{
  "schemaVersion": "1",
  "status": "pass",
  "generatedAt": "2026-09-01T02:47:00Z",
  "results": [
    { "id": "hook_path/cold_validate", "metric": "wall_clock_p95_us",
      "budget": 2000, "measured": 812, "headroom": 1188, "verdict": "pass" }
  ],
  "breaches": []
}
```

Exit codes — the **script's**, not the binary's:

| Code | Meaning |
|---|---|
| `0` | every SLO within budget (`status: "pass"`) |
| `1` | at least one breach (`status: "fail"`, `breaches[]` non-empty) |
| `2` | usage error — bad flags, missing manifest, unparseable Criterion output |
| `3` | a manifest ID had **no corresponding Criterion estimate** |

Code `3` exists because of §2.4. A gate that silently passes when its metric is missing is exactly
the bug this scope was written to fix, and re-creating it inside the fix would be embarrassing.
Missing data is a distinct, loud failure.

**`.github/workflows/benchmarks.yml` (edit).** Add a second job, `slo-gate`, that runs on
`pull_request` to `main`, does **not** do the baseline double-checkout (so it is fast — one
`cargo bench --bench hook_path`), runs `slo-check.py`, uploads `slo-report.json`, and fails on
non-zero. Leave the existing `benchmark` job and its `release/**` triggers alone; the two answer
different questions (absolute budget vs. relative drift) and both are worth having.

## 3.5 Deliverable E — the published number

Once the gate is green, the number goes in three places, and this is the memo's Recommendation #2
actually discharged:

- `README.md` — one line under the safety section: the measured cold-path p95 with the CPU it was
  measured on and a link to `docs/slo/hook-path.json`. **With the hardware named**, because an
  unqualified microsecond figure is marketing and a qualified one is engineering.
- `docs/slo/hook-path.json` — the machine-readable source of truth.
- The ADR-036 `caro guard` docs, when they exist, as the budget that adapter inherits.

**A discipline the PR must hold:** publish the number that was measured, on the runner class it was
measured on, with the p95 stated as p95. Do not publish a best-of-five. The competitors' numbers
in §1.6 are unreproducible vendor figures and the entire value of Caro's is that it is not one.

## 3.6 Boy-scout cleanup, scoped to files this PR touches

- `src/safety/mod.rs:8` — `52 pre-compiled regex patterns` → `67`.
- `src/caroml/validators/safety.rs:1` — `52+` → `67`.
- `CLAUDE.md` and `README.md` — `52+ dangerous patterns` → `67 dangerous patterns + 2 CVE rules`.
  Consider deriving the count in a test (`assert_eq!(DANGEROUS_PATTERNS.len(), 67)`) so the next
  drift fails CI instead of surviving a year.
- `patterns::validate_patterns()` — **remove the call from `new()`** (`src/safety/mod.rs:346`),
  keep the function, and call it from a `#[test]`. CI already hard-fails on pattern compilation in
  `safety-validation.yml`'s `pattern-compilation` job, so the runtime check buys nothing and costs
  67 compilations per construction. This is the largest single win in the PR.
- The duplicated `ConfigManager::load()` (`src/main.rs:3325-3328` and `src/cli/mod.rs:205-214`) —
  **noted, not fixed.** Threading one config through is a refactor with blast radius across the CLI
  and it does not belong in a perf PR. Filed as a bead (§3.8).

**Explicitly not touched:** the 63 `which` spawns in `ExecutionContext::detect()`. It is almost
certainly the biggest cold-start cost (§2.5) and it is *the wrong thing to fix in this PR* — the
right fix is to not call `detect()` on a path that doesn't need shell context, which is the fast-path
refactor ADR-036 needs and this ADR deliberately does not do. What this PR contributes is the
benchmark that will make the cost visible and attributable when someone does fix it. Filed as a bead.

## 3.7 Gate 3 — what breaks at 100 real users

Per `.claude/rules/validation-discipline.md`, the assumption that holds at demo scale and fails in
the field, its failure mode, its instrumentation, and its fallback.

**Assumption 1 — "one validation per user action."** Holds for interactive `caro`. Breaks the moment
Caro is a `PreToolUse` hook on a machine running parallel agents: N concurrent agent sessions × one
subprocess per Bash tool call. At 4 concurrent Claude Code sessions doing tool-heavy work, that is
dozens of `caro` processes per minute, each independently paying cold start.
*Failure mode:* aggregate CPU and fork pressure, not per-call latency. The p95 stays fine; the
machine gets slower.
*Instrumentation:* `hook_path/cold_validate` gives the per-call cost; multiply by observed tool-call
rate. The already-defined-but-dark `SafetyValidation` telemetry event (§3.3 C2) is the field
instrument, once someone lights it.
*Fallback:* none needed in v1 — the fix if it bites is the fast path in §3.8, and the numbers this
PR produces are the input to deciding whether it bites.

**Assumption 2 — "SQLite telemetry open is free."** `TelemetryStorage::new` (`src/main.rs:3379`)
opens `~/.../caro/telemetry/events.db` on every invocation, before validation. Multiple concurrent
`caro` processes contend on the same SQLite file.
*Failure mode:* under a lock, a `caro` process **stalls before it validates anything**. That is F1
from §1.7 — the fail-open — arriving through a path that has nothing to do with pattern matching.
*Instrumentation:* not covered by `hook_path/cold_validate`, which is an in-process bench and never
touches SQLite. **This is a known measurement gap in v1 and the PR body must say so.**
*Fallback:* the hook path should not open telemetry at all. Deferred to the fast-path work (§3.8),
because doing it here means touching the startup sequence in `main.rs`, which is a different PR.

**Assumption 3 — "the CI runner is representative."** It is not, and it does not need to be. The
ceiling-not-percentage design (§3.4) means the gate is answering "is this within an order of
magnitude of sane," which a shared runner can answer reliably, rather than "did this get 15%
slower," which it cannot.

**Assumption 4 — "allowlists are short."** The `compiled_allowlist` fix makes construction cost
linear in allowlist size while making per-call cost constant. A user with a 500-entry allowlist
moves the cost from the per-call path to the per-process path — and on a cold-subprocess hook path,
those are the same path.
*Failure mode:* `hook_path/cold_validate` regresses for that user and for nobody else; the CI gate,
running with a fixed test config, sees nothing.
*Instrumentation:* `hook_path/validate_allowlist_20` catches the shape, not the scale.
*Fallback:* document the linearity in the SLO manifest's `rationale`. A `max_allowlist_entries`
config bound is a v2 conversation, not a v1 guess.

## 3.8 Explicitly out of scope

| Item | Why | Where it goes |
|---|---|---|
| The `caro guard` PreToolUse adapter | ADR-036 owns it, unimplemented | ADR-036 |
| A dedicated `caro validate` fast path that skips backend, context detection and telemetry | The right fix for §2.5's biggest costs, and a real refactor of `CliApp::with_overrides` | New bead, depends on this PR's numbers |
| Collapsing 4 `SafetyValidator::new()` calls to 1 | Ownership refactor across `CliApp`, `EmbeddedModelBackend`, `StaticMatcher` | New bead; manifest ships `budget: 4` and ratchets |
| Removing the 63 `which` spawns | Belongs to the fast path, not to a perf-measurement PR | New bead |
| Single-load config | Blast radius across the CLI | New bead |
| End-to-end subprocess benchmark (`hyperfine`) | Right tool, wrong PR; Criterion is not built for it | v2 |
| Emitting `EventType::SafetyValidation` | Telemetry-consent and redaction review | Separate PR |
| Cross-platform SLO (Windows, Linux ARM) | v1 publishes one runner class honestly rather than four badly | v2 |
| Any latency claim about LLM-backed generation | Different path, different budget; today's memo calls an LLM on the hook path *"disqualifying"* and this ADR agrees | Never, on this path |
| Anything touching the assessment payload | ADR-059 moratorium | ADR-058/059 |

## 3.9 Integration tests — known input → deterministic output

Per the task's constraint, deterministic JSON plus an exit code. All four are new; none needs a
network, a model, or a fixture larger than a string.

**T1 — `tests/slo_contract.rs::validation_time_present_in_json`.**
Run the library path with a known-safe command and serialize `CliResult`. Assert
`timing_info.validation_time_us` is `Some(_)`. Assert the *rest* of the envelope is byte-identical
to the pre-change golden fixture with that one key removed — proving C1 is purely additive.

**T2 — `tests/slo_contract.rs::no_regex_compilation_during_validation`.** The load-bearing one.
Build a `SafetyValidator` with a 20-entry allowlist under the `slo-counters` feature, reset the
counter, run `validate_command("ls -la", ShellType::Bash)`, assert the counter is exactly `0`.
Deterministic, sub-millisecond, machine-independent, and it fails loudly the day someone puts a
`Regex::new` back in a loop.

**T3 — `tests/slo_contract.rs::slo_check_exit_codes`.** Table-driven over `scripts/slo-check.py`
with three fixture directories: within budget → exit `0`, `status: "pass"`; over budget → exit `1`,
`status: "fail"` and a populated `breaches[]`; manifest ID absent from the Criterion output → exit
`3`. Assert the parsed JSON *and* the process exit code, per ADR-024's precedent at its §"Integration
tests" — the report and the code are two halves of one contract and a test that checks only one of
them has checked neither.

**T4 — `tests/slo_contract.rs::allowlist_semantics_unchanged`.** A malformed allowlist entry
(`"[unclosed"`) plus a valid one. Assert `SafetyValidator::new` still returns `Ok`, that the valid
entry still matches, and that the malformed one is skipped — the §3.3 lenient-behaviour promise,
pinned so a later strictness change has to be deliberate.

**Not a test, but required in the PR body:** the three `cargo bench --bench hook_path` runs whose
worst p95 became the committed budget, pasted verbatim with the runner class named.

## 3.10 Definition of done

- [ ] `cargo bench --bench hook_path` produces four IDs in `target/criterion`.
- [ ] `cargo bench --bench performance` produces nine IDs — i.e. `benches/performance.rs` is wired
      and no longer silently dead.
- [ ] `docs/slo/hook-path.json` committed with **measured** budgets, not the placeholders in §3.2.
- [ ] `slo-gate` job green on a PR to `main`, and **demonstrably red** on a PR that reintroduces a
      `Regex::new` inside `validate_command` — proven by pushing that commit once and screenshotting
      the failure in the PR body. An ungated gate is what §2.6 is about; do not ship a second one.
- [ ] T1–T4 pass.
- [ ] The number is in `README.md`, with hardware named.
- [ ] Four beads filed for the §3.8 follow-ups, with dependency edges so `bd ready` surfaces the
      fast-path work next.
- [ ] `devils-advocate` review comment present on the PR (gate 4).

---

## Appendix — re-check greps (§2.9)

```bash
# §2.4 — performance bench is not a declared target
grep -n '\[\[bench\]\]' -A2 Cargo.toml          # cache, config, context, logging only
grep -n 'autobenches' Cargo.toml                 # 0 hits
grep -c 'criterion_main' benches/performance.rs  # 1, and it is never the entry point

# §2.5 — the cold-path waste
# 4 sites on the DEFAULT invocation path: cli/mod.rs:270, embedded_backend.rs:86 and :131,
# static_matcher.rs:131. The grep also returns constructions on paths this scope does not
# touch (caroml/validators/safety.rs:24, ai/runner.rs:152, evaluation/evaluators/safety.rs:26,34,
# agent/pipeline/hydrators.rs:193,202) — count only the four above when re-checking §2.5.
grep -rn 'SafetyValidator::new' src/ --include=*.rs
sed -n '512,529p' src/safety/patterns.rs         # validate_patterns discards every Regex
sed -n '13,507p' src/safety/patterns.rs | grep -c 'DangerPattern {'  # 67
sed -n '498,513p' src/safety/mod.rs              # Regex::new inside validate_command
sed -n '270,276p' src/context/mod.rs             # Command::new("which") per command

# §2.6 — the gate's triggers
sed -n '13,37p' .github/workflows/benchmarks.yml # pull_request: branches: ['release/**']
grep -c 'continue-on-error' .github/workflows/benchmarks.yml  # 0
grep -n 'bench' .github/workflows/ci.yml         # 0 hits
grep -n 'informational' .github/workflows/safety-validation.yml  # the neutralised assertions

# §2.7 — nothing measures validation
grep -rn 'Instant\|elapsed' src/safety/          # 0 hits
grep -rn 'EventType::SafetyValidation' src/ | grep -v 'events.rs\|storage.rs\|redaction.rs\|telemetry.rs'  # 0 emit sites
sed -n '125,130p' src/cli/mod.rs                 # TimingInfo has no validation field

# moratorium status
ls src/safety/                                    # cve_patterns.rs, mod.rs, patterns.rs — still no assessment.rs
```

---

**Sources (Phase 1):** [Claude Code hooks reference](https://code.claude.com/docs/en/hooks) ·
[Envoy `ext_authz` proto](https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/filters/http/ext_authz/v3/ext_authz.proto) ·
[Envoy `ext_authz` HTTP filter](https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/ext_authz_filter) ·
[OPA CLI reference](https://www.openpolicyagent.org/docs/cli) ·
[OPA policy performance](https://www.openpolicyagent.org/docs/policy-performance) ·
[opa/cmd/bench.go](https://github.com/open-policy-agent/opa/blob/master/cmd/bench.go) ·
Kubernetes admission-webhook timeout/failurePolicy behaviour via search summary
([kubernetes#128162](https://github.com/kubernetes/kubernetes/issues/128162),
[webhook failurePolicy & timeoutSeconds](https://oneuptime.com/blog/post/2026-02-09-webhook-failure-policy-timeout/view)) ·
`.hermes/digests/2026-09-01-agent-launch-scan.md` (internal)
