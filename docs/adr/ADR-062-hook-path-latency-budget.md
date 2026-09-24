# ADR-062: Hook-Path Latency Budget — Make Slow Mean *Blocked*, Not *Allowed*

- **Status**: Proposed (implementation ADR — this one is meant to become code)
- **Date**: 2026-09-01
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **Claude Code's hook system**, specifically the `PreToolUse` command-hook
  contract — stdin JSON payload, the exit-code decision model, `hookSpecificOutput.permissionDecision`,
  the per-hook `timeout` field, and its documented **fail-open-on-timeout** behaviour. Read live
  2026-09-01 at [code.claude.com/docs/en/hooks](https://code.claude.com/docs/en/hooks)
- **Cross-read**: **Kubernetes validating admission webhooks** (`timeoutSeconds` 1–30, default 10;
  `failurePolicy: Fail|Ignore`; 30 s global pod-admission budget), **Envoy `ext_authz`**
  (`failure_mode_allow`, default `false`), and **OPA `opa bench`** (which deliberately excludes
  load/parse/compile time from its reported figures)
- **Implements**: today's strategy memo, `.hermes/digests/2026-09-01-agent-launch-scan.md` §3.3 /
  Recommendation #2 — *"Publish a hook-path latency SLO and gate it in CI"* (Priority: Now,
  Complexity: S)
- **Depends on**: nothing unlanded. `SafetyValidator`, `ValidationResult`, `TimingInfo`, `CliResult`
  and criterion all shipped in 1.4.0. No daemon, no network, no new crate, no new module
- **Relates to**: ADR-036 (`caro guard` — this ADR scopes the budget that adapter inherits; it does
  not re-scope or block on the adapter), ADR-024 (exit-code + envelope precedent; **no new exit code
  is minted here**), ADR-060 (the "CI gate incapable of failing" pattern, found again in a second
  workflow), ADR-059 (the moratorium this ADR stays outside of)
- **Full scope document**: `caro-scope-hook-path-latency-2026-09-01.md`
- **Numbering note**: highest existing is ADR-061. Per `.claude/rules/adr-numbering.md`, renumber on
  merge if another 062 lands first

> **Provenance and boundaries.** No user was present; the task template's `[FEATURE NAME]` was
> unbound, so target selection was mine, and it was constrained three ways.
>
> **ADR-059 declared itself "the last ADR in this space until `src/safety/assessment.rs` merges."**
> Re-verified 2026-09-01: `src/safety/` contains `cve_patterns.rs`, `mod.rs`, `patterns.rs` and
> nothing else. The moratorium holds. **This ADR is outside it** — it mints no policy vocabulary,
> no verdict payload, no approval exchange, no lifecycle event, and no exit code. It adds one
> optional integer to an already-serialized timing struct and deletes work from a hot path.
>
> **Today's memo's single "build or test next" item is the frozen assessment schema, which is
> inside the moratorium.** This ADR is deliberately the memo's *second* recommendation instead. An
> overnight autonomous run should not be the thing that opens the governance-schema PR, and it
> should not start Recommendation #3 either, which is a weeks-long interview programme.
>
> **`src/assessment/` is not the assessment surface.** It is hardware/system profiling — `cpu.rs`,
> `gpu.rs`, `memory.rs`, `recommender.rs` — and its CLI entry point (`Commands::Assess`) is
> commented out at `src/main.rs:425-434`. Anyone reading `ls src/` and concluding the moratorium
> has lifted has read the wrong directory. Worth a rename, someday, in a different PR.
>
> **Validation-discipline note.** Gate 3 (demoware trap) is discharged in scope §3.7. Gate 4
> (`devils-advocate` review) is required before the implementation PR opens and is **not**
> discharged here. Gates 1, 2 and 5 do not attach — this is instrumentation and CI for an already
> shipped capability, which is the rule's own carve-out for "responses to evidence we already have."

---

## Context

### The problem, in one sentence from the vendor's own docs

Claude Code's hooks reference, under **Timeouts**, on `PreToolUse`:

> *"A timed-out `command`, `http`, or `mcp_tool` hook **doesn't block the tool call**. The call
> continues through the normal permission flow, so **don't count on a stalled hook to act as a
> gate**."*

An **Agent SDK callback hook** that exceeds its timeout *does* block. The in-process integration
fails closed; the subprocess integration fails open. Caro is a subprocess and cannot configure its
way off that side of the line — there is no `failurePolicy: Fail` in this host, unlike Kubernetes
(`failurePolicy`, operator-configurable) or Envoy (`failure_mode_allow`, defaulting to fail-closed).

The consequence is the entire thesis of this ADR: **for a subprocess safety gate, latency is not a
UX property, it is a safety property.** A slow validator does not annoy the user. It silently stops
being a validator, while remaining installed and appearing to work.

Two adjacent fail-open paths in the same contract compound it: exit code **1** is explicitly
non-blocking (*"Claude Code treats exit code 1 as a non-blocking error and proceeds with the
action"*), and **unparseable stdout** is a non-blocking error on every exit code except 2. Only
exit 2 blocks unconditionally. Everything Caro can get wrong on this path — slow, crashed,
garbled — resolves to *allow*.

### What the tree looks like today

Verified at HEAD `50859b89`, 2026-09-01. Full receipts and re-check greps in the scope document
§2.4–§2.9.

**Nothing measures validation.** `grep -rn 'Instant\|elapsed' src/safety/` → **zero hits.**
`TimingInfo` (`src/cli/mod.rs:125-130`) has `generation_time_ms`, `execution_time_ms`,
`total_time_ms` and no validation field. The `EventType::SafetyValidation` telemetry variant
(`src/telemetry/events.rs:85`) is defined, storage-tagged, redaction-tested — carries no duration,
and is **emitted from nowhere in production code**.

**The benchmarks that would catch this produce no output.** `benches/performance.rs` defines nine
Criterion benchmarks and ends in `criterion_main!` at `:318`. `Cargo.toml:161-176` declares
`[[bench]]` targets for `cache`, `config`, `context`, `logging` — **and not `performance`** — and
there is no `autobenches = false`. So the file is auto-discovered with the default
`harness = true`, libtest owns `main`, `criterion_main!` never runs, and the file contains zero
`#[bench]`/`#[test]` functions. **No safety-validation benchmark ID has ever reached
`target/criterion`.** The file's own comments claim a `<100ms` startup requirement and a `<100ms`
per-command requirement. Caro already has a latency SLO. It is a comment in a file that does not run.

**The CI gate that works is aimed elsewhere.** `.github/workflows/benchmarks.yml` is a genuinely
good workflow — baseline checkout, `--save-baseline`/`--baseline`, `scripts/benchmark-compare.py`,
and a real `Fail on regression … exit 1` step with **zero `continue-on-error` in the file**. But its
`pull_request` trigger is `branches: ['release/**']`, so a PR to `main` never fires it; and per the
paragraph above it has never had a validation metric to compare. `ci.yml` — the workflow that does
run on every PR — contains **zero** occurrences of `bench`.

**The cold path does avoidable work.** `SafetyValidator::new()` is constructed **four times** per
invocation (`src/cli/mod.rs:270`; `src/backends/embedded/embedded_backend.rs:86` and `:131`;
`src/backends/static_matcher.rs:131`). Each construction calls `patterns::validate_patterns()`
(`src/safety/mod.rs:346`), which compiles all **67** `DANGEROUS_PATTERNS` regexes
(`src/safety/patterns.rs:512-529`) and **discards every one of them**, collecting only error
strings — a runtime validity check whose product is thrown away, over a static table that
`safety-validation.yml`'s `pattern-compilation` job already hard-fails on. That is **268 throwaway
regex compilations per process.** Separately, allowlist patterns are compiled *inside*
`validate_command`'s hot loop (`src/safety/mod.rs:500`: `regex::Regex::new(allow_pattern)`), so a
20-entry allowlist costs 20 compilations **per validated command**. And before any of it,
`ExecutionContext::detect()` (`src/cli/mod.rs:275`) filters 63 command names through
`command_exists`, which is `Command::new("which").arg(command).output()`
(`src/context/mod.rs:270-276`) — up to **63 subprocess spawns per invocation**, unconditionally.

**Nobody has measured any of this.** That is not a rhetorical flourish, it is the state of the
evidence and it bounds what this ADR is allowed to claim. The 67-regex matcher is very probably
already fast. The point is that Caro cannot currently say so, cannot defend it against a
regression, and — per today's memo, §2.7 — is competing against vendors who publish latency figures
(agentOS's claimed ~4.8 ms cold start, Kastra's claimed sub-1 ms authorization, oMLX's claimed
90 s → 5 s; all unverified launch-page marketing, none of it reproduced by anyone here). The memo's
framing is exact: *"Caro has the fast architecture and no published number."*

### The design question Phase 1 answered

Every warm system in the survey amortises initialization across many decisions. K8s webhooks and
Envoy `ext_authz` are long-lived servers. OPA offers both shapes, and encodes the distinction in its
benchmark tooling — *"only the query evaluation is measured. All time spent preparing to evaluate
(loading, parsing, compiling, etc.) is omitted."*

**Claude Code command hooks are cold subprocesses.** No daemon, no connection reuse, no warm mode
that can gate (`async: true` hooks are exempt from timeout enforcement and cannot block a tool call).
So OPA is right to exclude prep time and Caro would be **wrong** to: on this path, prep time is not
overhead around the measurement, prep time *is* the measurement. `benches/performance.rs` currently
constructs a validator inside `b.iter` in all nine of its benchmarks — conflating the two rather
than choosing — which is why the numbers it would produce, if it produced any, would not be the
numbers the deployment experiences.

The tempting escape is a warm daemon. This ADR refuses it, and not only because the task constraints
forbid one: a daemon converts a fail-open *timeout* into a fail-open *connection error*, adding a
socket path, a stale-PID problem, and a privilege boundary in exchange for the same failure. The
correct answer for a subprocess gate is to make the cold path cheap enough that warmth is
unnecessary.

---

## Decision

Ship one PR that makes the cold hook path **measured, bounded, published, and defended** — and that
mints no new decision surface while doing it.

### D1 — Measure the cold path, and label it as cold

Add `benches/hook_path.rs` (new file, no new module) with four Criterion benchmarks:

| Bench ID | Measures | Role |
|---|---|---|
| `hook_path/cold_validate` | `SafetyValidator::new` **plus** one `validate_command`, in one closure | **The headline metric** — what a `PreToolUse` subprocess actually pays |
| `hook_path/validator_new` | construction alone | Attributes the 67-compile cost so D3's win is visible |
| `hook_path/validate_warm` | `validate_command` with the validator hoisted out of `b.iter` | The OPA-style steady-state number. Diagnostic only — **not** the SLO |
| `hook_path/validate_allowlist_20` | warm validation against a 20-entry allowlist | Exercises the hot-loop compile directly |

Register **both** `hook_path` and the currently-dead `performance` as `[[bench]] … harness = false`
in `Cargo.toml`, and hoist validator construction out of `b.iter` in all nine existing benchmarks.

### D2 — The budget is a checked-in, versioned, machine-read artifact

`docs/slo/hook-path.json`, `schemaVersion: "1"`, one entry per SLO with `id`, `metric`, `budget`,
`enforcement` and `rationale`. Changing the promise becomes a reviewable diff, the way
`timeoutSeconds` is a field on a K8s webhook object rather than a sentence in a runbook.

**Budgets are set to numbers the current code already meets.** Per the memo's explicit instruction —
*"Do not tune first — publish the honest number, then improve it"* — the PR lands the benches, runs
them three times on CI, takes the worst p95, rounds up generously, and commits that. A gate below
the current number is a broken build on day one.

### D3 — Delete the avoidable work, additively

- Remove `patterns::validate_patterns()` from `SafetyValidator::new()`. Keep the function; call it
  from a `#[test]`. CI already hard-fails on pattern compilation. This is the largest single win.
- Compile allowlist patterns **once at construction** into a new private
  `compiled_allowlist: Vec<(regex::Regex, String)>` field on `SafetyValidator`, and consume it at
  `src/safety/mod.rs:498-513`. Lenient behaviour is **preserved** — a malformed entry is still
  skipped, now with a `tracing::warn!` instead of silence. Turning a validator from lenient to
  strict is a safety-semantics change and belongs to the moratorium's territory, not to a perf PR.

### D4 — One integer through the existing envelope; no new decision surface

- `TimingInfo` gains `validation_time_us: Option<u64>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
  **Microseconds**, because a matcher this fast rounds to `0 ms` and an SLO you cannot resolve is
  not an SLO. `Option` + `default` keeps existing `CliResult` consumers working and keeps
  pre-ADR-062 envelopes valid — which is the direct mitigation for the "malformed stdout fails open"
  hazard in the host contract.
- `EventType::SafetyValidation` gains `duration_us: u64`, `#[serde(default)]`. (Note its existing
  shape for the implementer: `risk_level: String`, `action_taken: String`, and
  `pattern_category: **Option<String>**` — the scope document's §2.3 table renders the third as a
  bare `String`; the tree is the authority.) **This PR does not
  start emitting the event** — waking a dark telemetry variant is a consent-and-redaction question
  with its own review. The field is added now so that whoever lights it does not need a second
  schema migration.
- **No new exit code.** ADR-024's table (`0/1/2/3/10/11`) is untouched and unextended. The contended
  exit-code registry that ADR-053 and ADR-056 complain about gets no new entrant here.

### D5 — A gate with two metrics of different failure character

This is the load-bearing design decision, and it exists because of Caro's own history:
`safety-validation.yml`'s `static-matcher-tests` and `embedded-backend-tests` jobs both end in
`|| { echo "informational"; exit 0; }`. A flaky assertion was neutralised rather than fixed, and its
`pr-comment` job then posts a hardcoded all-✅ template regardless. Any wall-clock gate added here
will meet the same pressure the first time a noisy runner moves a microbenchmark by 3×.

So the gate asserts **two** things:

1. **A wall-clock *ceiling*, generously set** — not a percentage regression against a baseline.
   Percentage gates compare two noisy measurements; a ceiling compares one noisy measurement to a
   fixed constant. A 10×-headroom ceiling ignores runner noise and still catches every regression
   that matters on this path, because real ones here are order-of-magnitude events (a network call,
   a model load, a compile in a loop), not 15% drifts.
2. **A deterministic count that cannot flake** —
   `safety.regex_compilations_per_validation == 0`, asserted by a **unit test** rather than a
   benchmark, via a counter behind `#[cfg(any(test, feature = "slo-counters"))]` at the `Regex::new`
   sites in `src/safety/`. It runs in `ci.yml`'s existing `unit-tests` job, on every PR, in
   milliseconds, with zero variance.

If the wall-clock ceiling is ever neutralised under pressure, the count assertion still catches the
specific regression class this ADR exists to prevent, and it catches it as an ordinary failing test,
which nobody argues with.

`scripts/slo-check.py` (mirroring the existing `scripts/benchmark-compare.py`) emits
`slo-report.json` (`schemaVersion: "1"`, `status`, `results[]`, `breaches[]`) and exits **0** pass /
**1** breach / **2** usage error / **3** *manifest ID had no corresponding Criterion estimate*.
Code 3 is not decoration: a gate that silently passes when its metric is missing is precisely the
bug this ADR was written to fix, and re-creating it inside the fix would be embarrassing.

A new `slo-gate` job runs on `pull_request` to `main`, skips the baseline double-checkout, and fails
on non-zero. The existing `benchmark` job and its `release/**` triggers are left alone — absolute
budget and relative drift answer different questions and both are worth having.

### D6 — Publish the number

`README.md` gets one line: the measured cold-path p95, **with the CPU it was measured on named**,
linking to `docs/slo/hook-path.json`. Publish the p95 that was measured on the runner class it was
measured on. Not a best-of-five. The entire value of Caro's number is that, unlike every figure in
§1.6 of the scope document, it is reproducible.

---

## Consequences

### Positive

- **The fail-open failure mode is designed against rather than worked around.** Nothing here adds a
  retry, a timeout guard, or a "if slow, fail closed" branch — none of which Caro can implement,
  because the host owns the timeout. It attacks the cause: make the cold path cheap and keep it
  cheap. That is the task constraint *"solve the failure mode by design, not by workaround"*
  discharged literally.
- **A dead benchmark file and a mis-aimed CI gate both start working.** This is the second instance
  of the ADR-060 pattern — a quality mechanism that has never been able to fire — found in a
  different workflow. Worth noting as a recurring class, not a one-off.
- **Determinism becomes a performance argument, not just a correctness one.** Today's memo, §2.7:
  that is the argument engineers act on.
- **No new module, no new dependency, no new exit code, no daemon, no network.** Net line count is
  plausibly negative in `src/`.
- **`caro guard` (ADR-036) inherits a budget it can be held to** when someone builds it.

### Negative / accepted costs

- **A new CI job on every PR to `main`.** One `cargo bench --bench hook_path` — minutes, not
  seconds, given a cold Rust build. Mitigated by skipping the baseline double-checkout and by the
  fact that the *deterministic* half of the gate (D5.2) rides the existing `unit-tests` job for free.
- **A published number is a commitment.** Once it is in the README it constrains future design —
  which is the point, and is why D2 sets it generously rather than aspirationally.
- **The measurement has a known hole in v1.** `hook_path/cold_validate` is an in-process benchmark
  and never touches SQLite. `TelemetryStorage::new` (`src/main.rs:3379`) opens
  `~/.../caro/telemetry/events.db` before validation on every invocation, and concurrent `caro`
  processes contend on that file — a stall there produces the fail-open through a path this
  benchmark cannot see. **The PR body must state this gap explicitly.** The fix (don't open
  telemetry on the hook path) belongs to the fast-path work, not here.
- **The biggest suspected cost is deliberately not fixed.** The 63 `which` spawns are almost
  certainly the dominant cold-start term, and removing them means not calling
  `ExecutionContext::detect()` on paths that don't need shell context — a refactor of
  `CliApp::with_overrides` that belongs to ADR-036's fast path. What this ADR contributes is the
  benchmark that makes the cost visible and attributable when someone does it.
- **Four validator constructions stay four.** The manifest ships
  `validator_constructions_per_invocation: 4` with a ratchet TODO rather than an aspirational `1`
  that reds the build on merge.

### Neutral / to watch

- `validation_time_us` in `-o json` becomes something a script can depend on. It is `Option`al and
  additive, so it is a *capability* rather than a promise — but the moment someone builds a
  dashboard on it, it is a contract. Treat the next change to it as breaking.
- Compiling the allowlist at construction moves its cost from per-call to per-process. On a
  cold-subprocess hook path those are the same path, so a user with a very large allowlist sees no
  improvement — and the CI gate, running a fixed test config, would not notice. Documented in the
  manifest's `rationale`; a `max_allowlist_entries` bound is a v2 conversation, not a v1 guess.

---

## Alternatives considered

**A. A warm `caro` daemon with a unix socket.** Amortises everything; makes the numbers trivially
good. **Rejected.** It violates the pure-subprocess constraint, and on the merits it trades a
fail-open timeout for a fail-open connection error plus a socket path, a stale-PID problem, and a
new privilege boundary. Same failure, more surface. Being fast is the honest version of being warm.

**B. Percentage-regression gating against a `main` baseline, reusing `benchmark-compare.py`
as-is.** The mechanism already exists and works. **Rejected as the primary gate** because it
compares two noisy measurements on shared runners, which is how gates become
`|| { echo "informational"; exit 0; }` — a fate already suffered by two jobs in
`safety-validation.yml`. Kept as the *secondary* gate on `release/**`, where a slower, more
sensitive check is affordable.

**C. Wall-clock only, no count metric.** Simpler, one number, matches what every competitor
publishes. **Rejected.** The count metric is the only part of the gate that cannot flake, and it is
the part that directly catches the regression class this ADR is about. Publishing only what
competitors publish would also forfeit the one assertion a model-backed or network-backed competitor
structurally cannot make.

**D. Add a `caro validate` fast-path subcommand in this PR** — skipping backend construction,
context detection and telemetry — and benchmark that. Much better numbers, immediately.
**Rejected for this PR.** It is the right feature and it is a real refactor of
`CliApp::with_overrides`; doing it *before* the measurement exists means shipping an unmeasured
optimisation and having no way to prove it worked. Measurement first, then the fast path, with the
benchmark as its acceptance criterion. Filed as a bead depending on this ADR.

**E. Fold this into the assessment-contract ADR (memo Recommendation #1).** They are related — a
frozen decision payload and a bounded decision latency are two halves of "you can embed this."
**Rejected.** Recommendation #1 is inside the ADR-059 moratorium and is the largest governance
decision on the board; it should not arrive as a subsection of a benchmarking ADR written overnight
by an unattended process.

**F. Do nothing until someone reports that Caro is slow.** **Rejected**, and the reason is the whole
ADR: on this host contract, nobody will report it. A hook that exceeds its timeout is cancelled
silently and the tool call proceeds. The user's experience of a failing gate is indistinguishable
from the user's experience of a passing one. There is no bug report coming.

---

## References

- [Claude Code hooks reference](https://code.claude.com/docs/en/hooks) — `PreToolUse` payload,
  exit-code model, `hookSpecificOutput.permissionDecision`, common-fields `timeout` row, Timeouts
  section (fetched 2026-09-01)
- [Envoy `ext_authz` proto](https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/filters/http/ext_authz/v3/ext_authz.proto) ·
  [Envoy `ext_authz` HTTP filter](https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/ext_authz_filter)
- [OPA CLI reference](https://www.openpolicyagent.org/docs/cli) ·
  [OPA policy performance](https://www.openpolicyagent.org/docs/policy-performance) ·
  [opa/cmd/bench.go](https://github.com/open-policy-agent/opa/blob/master/cmd/bench.go)
- Kubernetes admission-webhook `timeoutSeconds` / `failurePolicy` behaviour — via search summary,
  **not** fetched from the primary Kubernetes docs:
  [kubernetes#128162](https://github.com/kubernetes/kubernetes/issues/128162),
  [failurePolicy & timeoutSeconds](https://oneuptime.com/blog/post/2026-02-09-webhook-failure-policy-timeout/view)
- `.hermes/digests/2026-09-01-agent-launch-scan.md` — §2.7, §3.3, Recommendation #2 (internal)
- `caro-scope-hook-path-latency-2026-09-01.md` — full scope, receipts, integration tests, gate-3
  section, re-check greps
