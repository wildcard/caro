# Implementation Scope — `caro replay`: offline rehearsal against a corpus the user already has

**Feature under analysis:** **`gator`**, the Open Policy Agent Gatekeeper offline policy CLI
(`gator test` / `gator verify` / `gator sync test` / `gator bench`), still marked
**"v3.11+ (beta)"** in the v3.23 docs, together with Gatekeeper's per-enforcement-point
action model (`gator.gatekeeper.sh` vs `validation.gatekeeper.sh` vs `audit.gatekeeper.sh`).
Cross-read against **Kyverno CLI** (`kyverno apply` / `kyverno test`, `Audit` vs `Enforce`,
`--audit-warn`, `--warn-exit-code`), **Conftest** (`--fail-on-warn`, `--no-fail`, SARIF
`exitCodeDescription`), **Envoy RBAC `shadow_rules`**, and **AWS IAM Access Analyzer
`check-no-new-access`**. Researched **2026-08-31**.

**Equivalent we are scoping for Caro:** **`caro replay`** — a pure subprocess that reads a
corpus of commands the user *already ran*, runs the existing `SafetyValidator` over it, and
prints what Caro *would have* done, with a deterministic JSON contract, a stable exit-code
table, and an optional diff against a stored baseline. No daemon. No state. No network. No
new module tree.

**Date:** 2026-08-31 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-061-history-replay-and-decision-diff.md`

---

> ### Provenance note — target selection, and the two moratoria it has to respect
>
> This scheduled task's template leaves `[FEATURE NAME]` unbound and no user was present.
> Target selection was mine. Two prior constraints bound it:
>
> 1. **ADR-059 (2026-08-26) declared itself "the last ADR in this space until
>    `src/safety/assessment.rs` merges."** Re-checked today: `src/safety/` contains
>    `cve_patterns.rs`, `mod.rs`, `patterns.rs` and nothing else. **The moratorium holds**
>    and it covers the assessment/governance surface, ADR-035 → ADR-059. This scope is
>    outside it: it defines no policy vocabulary, no verdict payload, no approval exchange,
>    no lifecycle event. It consumes `ValidationResult`, which shipped in 1.4.0.
> 2. **Today's strategy memo** (`.hermes/digests/2026-08-31-agent-launch-scan.md`) names
>    exactly one thing under *"Build or test next — one thing"*: **`caro replay --history`**,
>    human-readable v1, `--json` following. Its §3.1 dependency note is explicit and is the
>    reason this document exists in the shape it does: *"Either that schema lands first, or
>    `caro replay` ships human-readable-only in v1 with `--json` following. **Recommend the
>    latter** — coupling this to an unlanded blocker is what stalled it last week."*
>
> **Where this scope disagrees with the memo, and why.** The memo says human-readable
> first, JSON later. §3.4 below argues the opposite ordering is cheaper *given what is
> already in the tree*: ADR-023 already froze an exit-code table and a JSON envelope for
> the identical operation (validator-over-a-batch-of-commands), and `ValidationResult`
> already derives `Serialize`. Emitting JSON is not the expensive part; **deciding** the
> contract is, and that decision was made on 2026-06-12 and never implemented. So this
> scope ships both, and reuses ADR-023's table verbatim rather than minting a second one.
> That is a smaller change than the memo assumed, not a larger one.
>
> **Relationship to ADR-056.** ADR-056 (2026-08-21, *Enforcement Rehearsal — Observe Mode
> and Offline Corpus Replay Without a Daemon*) already scoped this idea from Phinq's
> `npm run replay` and its own header says it is *"blocked on them landing first"* — on
> ADR-024's `ExitCode` enum and ADR-032's `ExecutionReceipt` hash chain. Re-checked today:
> `grep -rn "enum ExitCode" src/` → **0 hits**; `grep -rn "HeadlessEnvelope" src/` → **0
> hits**. Both blockers are still unlanded, ten days on. **This scope is the unblocked
> subset of ADR-056**: the half that needs neither the envelope nor the chain, because the
> corpus is plain text on the user's disk and the report is a leaf document. ADR-056 keeps
> the agent-transcript corpus, the hash chain, and `caro audit rehearse`; ADR-061 takes
> `~/.zsh_history` and ships.
>
> ### Verification status
>
> - **Verified against the Caro tree (2026-08-31, branch `integrator/20260711-postmerge`,
>   `Cargo.toml` version `1.4.0`, `rust-version = "1.85"`):** every path, line number,
>   struct field, derive, tuple shape and absence claim in Phase 2 was produced by reading
>   `src/`, `tests/`, `Cargo.toml` and `docs/adr/`. Line numbers drift; each claim below is
>   written so it can be re-checked with one `grep`.
> - **Verified by live fetch (2026-08-31), quoted verbatim in Phase 1:** the gator command
>   reference and its "Gotchas" section, the Gatekeeper enforcement-points page, the
>   Gatekeeper audit page, gator issues #2863 / #2945 / #3023, the Kyverno CLI reference,
>   Kyverno issues #4255 / #7681 / #9804, the Conftest options page and issue #387, the
>   Envoy RBAC filter page, and the AWS `check-no-new-access` CLI reference.
> - **Explicitly NOT verified:** Conftest's *default-mode* exit code for a genuine tool
>   error (as opposed to a policy failure) — inferred from issue #387's framing, not shown
>   on the options page. The **process** exit code of `aws accessanalyzer
>   check-no-new-access` — the reference page documents the JSON `result` field and says
>   nothing about `$?`. Kyverno's outbound-network restriction — search summary only.
> - **Vendor marketing, unverified, and load-bearing nowhere in Phase 3:** every Kastra
>   claim. Kastra is a commercial product launched on Product Hunt 2026-07-22; everything
>   attributed to it below is a founder comment on that page. Nobody on this project has
>   installed it. Today's memo says the same thing in its own confidence notes and calls it
>   *"the single largest weakness in the memo."* It is quoted here **once**, in §1.6, for
>   one reason: it is the only party claiming to have built decision-corpus replay, and its
>   self-disclosed limitation is a useful design input. No decision in Phase 3 depends on
>   it.

---

# Phase 1 — Feature research

## 1.1 What problem does offline rehearsal solve, and for whom?

Nobody enables a gate that can refuse things on day one. The person evaluating a safety
layer has a working system and is being asked to install something whose only observable
effect, until the day it saves them, is refusal. The rational move is to decline.

Every mature policy engine in the survey answers this the same way, and all of them put
the answer in the onboarding path rather than in the accuracy claims:

- **Gatekeeper** does not have a single "dry run" switch; it has **four enforcement points**
  (`validation.gatekeeper.sh`, `audit.gatekeeper.sh`, `gator.gatekeeper.sh`, `vap.k8s.io`)
  with **per-point actions**, so a constraint can be `deny` in shift-left CI while it is
  only `warn` at live admission. The doc gives the migration narrative literally: *"You are
  trying out a new constraint template, and you want to deny violating resources in
  shift-left testing, but do not want to block any resources admitted to clusters to reduce
  impact for faulty rejections. You may want to use `deny` action for the
  `gator.gatekeeper.sh` shift-left enforcement point and `warn` for
  `validation.gatekeeper.sh`."*
- **Kyverno** puts it on the policy: `validationFailureAction: Audit` writes a PolicyReport
  and admits the resource; `Enforce` blocks. `--audit-warn` plus `--warn-exit-code` lets
  the same corpus be a warning this quarter and a hard failure next.
- **Envoy** states it in one sentence: *"This filter also supports policy in both
  enforcement and shadow mode, shadow mode won't affect real users, it is used to test that
  a new set of policies work before rolling out to production."*
- **Conftest** carries the weakest version — `warn` vs `deny` rule categories gated by
  `--fail-on-warn` — and no phased-rollout narrative in its docs.

The buyer is a person who will not accept a claim and will accept a diff.

## 1.2 Core architecture — the one distinction that matters

**Is the offline evaluator the same engine as the online one, or a second implementation?**

- **gator: same framework, declared as a peer.** gator is enumerated inside the *same*
  `scopedEnforcementActions` model as the webhook and audit, i.e. it is a first-class
  evaluation path in the constraint framework rather than a reimplementation.
  `gator bench` corroborates architecturally — it benchmarks the same client and constraint
  framework the runtime uses, and says what it excludes: *"gator bench measures compute-only
  policy evaluation latency, which does not include network round-trip time, TLS overhead,
  or Kubernetes API server processing. Real-world webhook latency will be higher."*
- **Kyverno: same codebase, different binary, and the doc says so.** *"The CLI, although
  composed of the same Kyverno codebase, is a purpose-built binary available via multiple
  installation methods but is distinct from the Kyverno container image which runs as a Pod
  in a target Kubernetes cluster."*
- **Conftest: no online counterpart at all**, so nothing to diverge from.

Data flow is identical in all three: `policy set` + `corpus` → compile-once → evaluate-N →
structured results + exit code. Policy and corpus are separate inputs; neither tool has a
persistent store between invocations. Every one of them is a cold start per process.

## 1.3 Why it is still beta — the failure modes, which are the point

This is the section the rest of the document is built from. Five failure classes, each with
a citation.

**FM-1 — Offline/online divergence, because the offline path lacks context the live path
had.** gator's own Gotchas: *"Gator cannot determine if a type is Namespace-scoped or not,
so it does not assign objects to the default Namespace automatically. Always specify
`metadata.namespace` for Namespace-scoped objects to prevent test failures, or to keep from
specifying templates which will fail in a real cluster."* Gatekeeper's audit doc has the
same shape for user identity: *"When using `input.review.userInfo` … cannot be populated by
Kubernetes for audit reviews and therefore constraint templates that rely on `userInfo` are
not auditable"* — and adds *"Note that `audit` or `gator test` are different enforcement
points and they don't have the AdmissionReview request metadata."* Kyverno's is blunter:
*"The Kyverno CLI via the `test` command does not embed the Kubernetes control plane
components and therefore is not able to perform the types of initial mutations subjected to
a resource as part of an in-cluster creation flow."* Three independent tools, one bug: the
rehearsal is missing an input the real thing had, and **the tools default it silently
rather than declaring it unknown.**

**FM-2 — Exit-code conflation: "the tool broke" and "the tool found something" share a
code.** gator documents this against itself: *"An error during evaluation, for example a
failure to read a file, will result in a `1` exit status with an error message printed to
stderr. Policy violations will generate a `1` exit status as well, but violation
information will be printed to stdout."* A caller cannot distinguish the two without
parsing prose off two different streams. `gator sync test` and `gator bench --compare`
repeat the pattern. Conftest defaults to the same (`1` for a failed policy, and no
documented distinct code for a genuine tool error outside `--fail-on-warn`; **this
particular inference is unverified**). AWS is the extreme case: `check-no-new-access`
returns `{"result": "FAIL", ...}` in a **successful** API response, and the reference page
documents no process exit code at all — the caller has to `--query result`.

**FM-3 — Silent skip counted as pass.** Kyverno CLI, before v1.14, hit a `context.apiCall`
without a cluster, logged `disabled loading of APICall context entry`, **skipped the rule**,
and reported the resource as passing — the doc's own retrospective calls the effect
*"misleading pass/fail counts in CI."* Fixed by building an in-memory dynamic client from
the supplied manifests, with a stated residual limit: *"Only `GlobalContextEntry` entries
with a `spec.kubernetesResource` source are resolved this way. Entries that point to an
external HTTP service are out of scope for offline evaluation."* gator #2863 is the same
family from the other side: a Constraint with `enforcementAction: enforce` (not a legal
value) and a junk field `randomField: foobar` passed `gator verify` and was only rejected
by a real cluster — the reporter's ask was *"Gator should exit non-zero to indicate that
the constraint resource is invalid."*

**FM-4 — Non-determinism and order-dependence.** Kyverno #7681: *"depending on the order of
the tests defined on the kyverno-test.yaml file, kyverno outputs the following errors: …
If only the two results entries are swapped, the error doesn't occur."* gator's Gotchas
warn that *"Rego de-duplicates identical violation messages. If you want to be sure that a
test returns multiple violations, use a unique message for each violation"* — i.e. the
finding count is a function of message *text*, which is prose. Neither tool documents a
stable output ordering guarantee.

**FM-5 — Findings that cannot fail the build, with no flag to change it.** gator issue
**#2945** is open: violations at `enforcementAction: warn` produce exit 0 and there is no
CLI option to escalate. A rehearsal that cannot fail is a rehearsal nobody watches.

## 1.4 Structured output contract — the literal tables

| Tool | Flags | Success | Findings | Tool error |
|---|---|---|---|---|
| `gator test` | `-f`, `--image`, `-o {yaml,json}`, `--deny-only` (3.19+) | `0` | `1` (stdout) | `1` (stderr) — **conflated** |
| `gator sync test` | — | `0` | `1` | `1` — **conflated** |
| `kyverno apply` | `--policy-report`, `--audit-warn`, `--warn-exit-code N`, `--warn-no-pass` | `0` | `1` | non-zero | 
| `kyverno test` | — | `0` | `1` | `2` (some paths; see #9804) |
| `conftest test` | `-o {stdout,json,tap,table,junit,github,azuredevops,sarif}`, `--fail-on-warn`, `--no-fail` | `0` | `1` (or `2` under `--fail-on-warn`) | `1` (**unverified**) |
| `aws … check-no-new-access` | `--existing-policy-document`, `--new-policy-document`, `--policy-type` | `result: PASS` | `result: FAIL` | API error | 

Shapes worth copying:

- Kyverno's **five-way** result taxonomy — `pass: N, fail: N, warn: N, error: N, skip: N` —
  is the only one in the survey that makes "we could not evaluate this" a first-class
  count rather than folding it into pass. It is the direct answer to FM-3.
- Conftest's SARIF emits its own exit semantics inline:
  `"invocations":[{"executionSuccessful":true,"exitCode":1,"exitCodeDescription":"Policy violations found"}]`
  — `executionSuccessful: true` alongside `exitCode: 1` is precisely the distinction FM-2
  loses at the process boundary, recovered inside the document.
- Conftest's JSON is per-file and flat:
  `[{"filename": "...", "successes": N, "failures": [{"msg": "..."}], "warnings": [...]}]`.
- AWS's diff result is `{result, message, reasons[{description, statementIndex, statementId}]}`.

## 1.5 Session / context lifecycle

Every tool in the survey is **compile-once, evaluate-N, exit** — no daemon, no cache
between invocations. `gator bench` publishes the breakdown as a first-class output
(*"Setup Duration … Client Creation / Template Compilation / Constraint Loading / Data
Loading"*), which is how you can tell the client is built once and reused across all
`Constraint × Object` pairs rather than per-object. It notes CEL evaluates 1.5–3× faster
per call but compiles 2–3× slower, offers `--concurrency=N`, and says scaling is roughly
linear to 4–8 workers. Kyverno v1.14 builds its in-memory dynamic client once per run and
resolves every `apiCall` in the corpus against it — the fix that turned a per-object
round-trip design into build-once/lookup-many.

**The lesson for Caro is not "add concurrency."** It is that setup cost is amortized only
if setup happens outside the per-item loop, and that these projects treat *where* the
compile happens as an architectural fact worth publishing. §2.5 shows Caro currently
compiles part of its policy set **inside** the per-command loop.

## 1.6 "Would-flip" — nobody in the OSS survey has it

Replaying a stored corpus of *past decisions* through a *candidate policy* and reporting
only the changed outcomes does not exist in gator, Kyverno CLI, Conftest, or Envoy.
`gator bench --save/--compare` diffs latency, not verdicts. Kyverno PolicyReports are
per-run snapshots with no baseline comparison in the CLI. Envoy runs shadow rules against
live traffic in-process and exports `shadow_allowed` / `shadow_denied` counters plus
`shadow_effective_policy_id` / `shadow_engine_result` dynamic metadata — an operator can
compute the delta, but Envoy does not. AWS `check-no-new-access` *is* a diff, but it is
policy-vs-policy for a single pair, not policy-vs-corpus.

The only party claiming it is **Kastra** (commercial, unverified, founder comment on
Product Hunt): *"You can replay your stored decisions through the candidate policy and diff
against what actually happened. You get would-flip counts (how many past ALLOWs become
DENYs and vice versa) plus concrete samples, before it's ever active."* Its self-disclosed
limitation is a clean instance of FM-1 and is the single most useful line in the whole
Kastra material: *"pre-inference rules (tool calls, commands) replay exactly, but rules
matching raw prompt content don't, since we store prompts only hashed. Those you validate
in shadow rather than by replay."*

Note what that concedes. Kastra can only replay decisions **Kastra already made** — the
corpus is a byproduct of prior adoption. You must buy in before you can evaluate. §2.3
is about the fact that Caro does not have that constraint.

---

# Phase 2 — Competitive differentiation

## 2.1 What they get right that we should replicate

1. **Compile-once, evaluate-N, exit — no daemon.** All three OSS tools. Caro is a CLI
   already; this costs nothing and is the whole reason the feature is S-sized.
2. **A published exit-code table, and separate codes for "found something" vs "broke."**
   The tools that conflate them say so in their own docs and get issues filed about it.
   Caro can simply not have that bug — see §2.4, where the table already exists.
3. **A first-class "could not evaluate" count** (Kyverno's `skip` / `error`). This is what
   turns FM-3 from a silent lie into a visible number.
4. **A stated exclusion list** — gator's Gotchas, Kyverno's control-plane note, Kastra's
   hashed-prompt admission. Every one of them documents where the rehearsal is *not* the
   real thing. That paragraph is the credibility of the feature.
5. **Per-enforcement-point actions** (Gatekeeper). The generalization: the same rule can
   carry a different action depending on *where* it is evaluated. Caro's version is
   `--fail-on <level>`, already specified in ADR-023.

## 2.2 Their design gaps, which we avoid by deciding the schema first

| Gap | Where | Caro's answer (Phase 3) |
|---|---|---|
| Findings and tool errors share exit 1 | gator (documented), Conftest (default) | D5 — reuse ADR-023's table: `1/2/3` = findings by tier, `10` = input error, `11` = internal error |
| Unevaluable item silently counted as pass | Kyverno pre-1.14 | D3 — `indeterminate` is a distinct outcome; D4 — a reconciliation invariant that fails the run if the counts do not add up |
| Missing context silently defaulted | gator namespace, Gatekeeper `userInfo` | D2 — the record carries its context explicitly; unknown fields are `null` and any verdict that *depends* on a null field is `indeterminate`, never guessed |
| No stable output ordering | Kyverno #7681, gator message-dedup | D6 — sorted output, `BTreeMap`, and a byte-equality test across two runs |
| Warn-tier findings cannot fail CI | gator #2945 (open) | D5 — `--fail-on` already covers it |
| Finding identity is prose | gator (Rego de-dupes on *message text*) | Named as the top out-of-scope item (§3.8, OOS-1) — Caro has the same bug and v1 does not pretend otherwise |

## 2.3 Caro's positioning — what we can do that they cannot

**The corpus already exists and does not require prior adoption.** This is the whole
argument, and it is not available to any product in the survey:

- gator, Kyverno, Conftest replay **configuration manifests**. Their corpus is authored.
- Kastra replays **its own stored decisions**. Its corpus is a byproduct of having already
  deployed Kastra. To evaluate it you must first adopt it.
- Caro replays `~/.zsh_history`. **The corpus predates Caro.** A person who has never run
  `caro` once can, in ten seconds, see every command they personally typed that Caro would
  have stopped. That inverts the evaluation funnel: adopt-then-measure becomes
  measure-then-adopt.

**Determinism makes replay meaningful; the market's chosen alternative cannot do this at
all.** Today's memo, §2.3: *"Determinism is now the explicit selling point against
LLM-judged safety."* An LLM-judged safety layer structurally cannot offer replay, because
two runs over the same corpus are not guaranteed to produce the same verdicts, so a diff
against a baseline is not attributable to the policy change. Caro's 52+ regex patterns are
the only asset in this market that is *reproducible by construction*, and replay is the
cleanest possible demonstration of that. It is a demo, not a dashboard — which matters,
because §4 of the same memo says do not build a dashboard.

**Offline is a privacy claim here, not a feature bullet.** The corpus is the user's shell
history. It contains `export AWS_SECRET_ACCESS_KEY=…`, `mysql -pHunter2`, hostnames,
internal paths, and customer names. Any hosted competitor asking for that corpus is asking
for something no security team will approve. Caro reads it, evaluates it in-process, and
emits redacted output by default — nothing leaves the machine, and the AGPL means anyone
can check that claim. This is the one place where "local-first" stops being a preference
and becomes the only viable architecture.

## 2.4 What already exists in the tree that covers part of this

**ADR-023 (`caro scan`, 2026-06-12, Status: Proposed) already froze the contract.** It
specifies `--fail-on <LEVEL>` (default `high`), `-q/--quiet`, stdin via `-`, a
`schema_version` field, SARIF output, and this exit-code table, quoted verbatim:

| Exit code | Meaning |
|---|---|
| 0 | All commands safe at the configured `--fail-on` level |
| 1 | At least one command blocked (`suggested_routing: block`) |
| 2 | At least one command requires human gate (`suggested_routing: human_gate`) and `--fail-on` ≤ `high` |
| 3 | At least one async-log warning and `--fail-on` ≤ `moderate` |
| 10 | Input error (no command, unreadable file, unknown shell) |
| 11 | Internal validator error |

**Findings and tool errors are already separated (1/2/3 vs 10/11).** Caro's unimplemented
2026-06 design is, on this specific point, better than shipping gator and shipping
Conftest. That is worth noticing before inventing anything.

**Also present and reusable:**

- `SafetyValidator::new(SafetyConfig) -> Result<Self, ValidationError>` and
  `validate_command(&self, &str, ShellType) -> Result<ValidationResult, ValidationError>`
  (`src/safety/mod.rs:344`, `:459`). The validator is already the shared engine; there is
  no second implementation to keep in sync — Caro starts where gator ended up.
- `ValidationResult { allowed, risk_level, explanation, warnings, matched_patterns,
  confidence_score }` (`src/safety/mod.rs:175`), deriving `Serialize, Deserialize`.
- `SafetyDecision::from_validation_result(&ValidationResult, SafetyLevel)`
  (`src/safety/mod.rs:225`) — already computes `SuggestedRouting`, which is the field
  ADR-023's exit table keys on. The mapping the exit codes need is written.
- `SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }` (`src/models/mod.rs:189`)
  and `RiskLevel { Safe, Moderate, High, Critical }` (`src/models/mod.rs:152`).
- `ShellType { Bash, Zsh, Fish, Sh, PowerShell, Cmd, Unknown }` (`src/models/mod.rs:419`)
  with `ShellType::detect()`.
- `caro::cli::OutputFormat { Json, Yaml, Plain }` (`src/cli/mod.rs:105`) with `FromStr`.
- `schemars = { version = "0.8", features = ["chrono"] }` (`Cargo.toml:46`) and
  `src/bin/generate-schema.rs`, which today emits only `UserConfiguration` into `.vscode/`.
- `serde_json` (`Cargo.toml:42`). No new dependency is required by anything below.

**And what is absent — each verified today by grep:**

- No `Scan`, `Validate`, or `Replay` subcommand. The `Commands` enum (`src/main.rs:381`)
  holds `Doctor, Integration, Init, Config, Knowledge, Profile, Test, Completion, Suggest,
  Ai, ShellInit, Check, List, Jobs, New, Generate, Run, Export, Experiment, Adopt, History,
  Why, Do, Render, Skill`. `History` is CaroML task lineage, not shell history.
- **No shell-history reading anywhere.** `grep -rn "bash_history\|zsh_history\|HISTFILE\|
  fish_history\|atuin" src/ --include=*.rs` → **0 hits**, despite
  `ATUIN_ALIGNMENT_STRATEGY.md` at the repo root.
- `tests/scan_integration.rs` does not exist.
- No `schemas/` directory.
- No `enum ExitCode`, no `HeadlessEnvelope`, no `RuleId`. `EXIT_CODE_EDIT: i32 = 201`
  (`src/main.rs:938`) remains the only exit constant in the tree.
- `ValidationResult` does **not** derive `JsonSchema`, although `src/safety/mod.rs:32`
  imports it and two other types in the same file (`:134`, `:333`) do derive it.

## 2.5 Three defects found while verifying, all in the replay path

These were not sought; they fell out of reading `src/safety/` against Phase 1's failure
modes. Each maps to a Phase 1 failure class, which is the point of doing it in this order.

**DEF-1 — the allowlist is recompiled once per command (FM-5 / §1.5).**
`src/safety/mod.rs:499`, inside `validate_command`:

```rust
for allow_pattern in &self.config.allowlist_patterns {
    if let Ok(regex) = regex::Regex::new(allow_pattern) {
```

Custom patterns are compiled once in `new()` and cached in
`compiled_patterns: Vec<(Regex, RiskLevel, String)>` (`:161`, populated `:378`). The
allowlist is not. Over a 20,000-line history with M allowlist entries this is 20,000 × M
regex compilations that the built-in path does exactly zero of. This is the *precise*
anti-pattern §1.5 describes, and replay is what makes it observable.

**DEF-2 — an invalid allowlist regex is silently ignored (FM-3).** Same two lines:
`if let Ok(regex)` drops the `Err` with no warning, no counter, no exit code. A user with a
typo in `allowlist_patterns` gets *stricter* behaviour than configured and is never told.

**DEF-3 — a built-in pattern that fails to compile silently vanishes (FM-3, worse).**
`src/safety/patterns.rs:551`:

```rust
pub static COMPILED_PATTERNS: Lazy<Vec<CompiledPattern>> = Lazy::new(|| {
    DANGEROUS_PATTERNS.iter().filter_map(|pattern| {
        Regex::new(&pattern.pattern).ok().map(|regex| { … })
```

`filter_map` + `.ok()` means a malformed built-in pattern is dropped from the compiled set.
`validate_patterns()` prints `WARN:` to stderr at construction (`src/safety/mod.rs:346`)
and then *continues*, with the comment *"this is a defensive check."* Net effect: **the
"52+ patterns" claim in `README.md`, `CLAUDE.md`, and on the website is not enforced at
runtime.** Coverage can silently shrink and every test still passes. This is Kyverno #4255
and gator #2863 in Rust, and it is in the load-bearing path.

*Corollary found while counting:* `DANGEROUS_PATTERNS` actually holds **67** entries
(`grep -c "DangerPattern {" src/safety/patterns.rs` → 67, all inside the `Lazy` block),
while `src/safety/mod.rs:8` documents *"52 pre-compiled regex patterns"* and the public
claim is "52+". The "+" makes the claim true, but nothing in the tree knows the real
number, which is precisely why DEF-3 can go unnoticed. D9's assertion is
`COMPILED_PATTERNS.len() == DANGEROUS_PATTERNS.len()` — a relative invariant, deliberately
not a hard-coded count, so it cannot rot the way the doc comment did.

DEF-1 and DEF-3 are fixed in this PR (both are one-line-ish and both are directly in the
feature's path — Boy Scout rule, scoped to what this PR touches). DEF-2 is fixed by D4's
reconciliation invariant surfacing it as an `errored` count.

---

# Phase 3 — Scope definition

## 3.1 The shape

One subcommand. One new source file. No new module tree, no new dependency, no daemon, no
state, no network.

```
caro replay [SOURCE] [--baseline <path>] [--output {plain,json,yaml}]
            [--fail-on {safe,moderate,high,critical}] [--fail-on-flip {none,any,looser,stricter}]
            [--shell <shell>] [--safety <level>] [--limit N] [--include-raw] [--stable] [-q]

SOURCE := --history [path]        # auto-detect ~/.zsh_history, ~/.bash_history, ~/.local/share/fish/…
        | --file <path>           # one command per line
        | -                       # stdin
```

Default with no arguments is `--history` with auto-detection, because the demo is the
product: `caro replay` must do the interesting thing with zero flags.

## 3.2 Decisions

**D1 — `caro replay` is the first implementation of ADR-023's contract, not a second
contract.** Exit codes, `--fail-on`, `schema_version` and the JSON envelope are ADR-023's,
adopted verbatim. `caro scan` remains unimplemented and, when it lands, is the same code
path with a different input source. *Consequence:* if ADR-023's table is wrong we inherit
the mistake — accepted, because two contracts for one operation is a worse outcome, and
§2.4 shows the table is already better than the shipping competition on the point that
matters.

**D2 — the corpus record carries its own context; unknown context is `null`, never
defaulted.** This is the direct answer to FM-1. A history line has no recorded shell, no
`cwd`, no timestamp on most platforms, and no safety level. `caro replay` therefore records
what it *assumed*:

- `shell` is taken from `--shell`, else inferred from **which history file it read**
  (`~/.zsh_history` ⇒ `Zsh`), else `ShellType::detect()`, else `Unknown`. The chosen value
  and its provenance both appear in the report header.
- `cwd`, `timestamp`, `exit_status`, `user` are `null` for a plain history file. They are
  populated only when the source format carries them (zsh `EXTENDED_HISTORY`).
- **A verdict that depends on a field which is `null` is `indeterminate`, not a guess.**
  In v1 exactly one rule depends on such a field: a pattern with `shell_specific: Some(s)`
  evaluated against a record whose `shell` provenance is `Unknown`. That record is
  `indeterminate` and is counted, printed, and (with `--fail-on-indeterminate`) can fail.

**D3 — five outcomes, borrowed from Kyverno, because four is the bug.**
`Clean | Flagged | Indeterminate | Errored | Skipped`. `Skipped` is a record excluded before
evaluation (blank line, comment, `--limit` truncation, timestamp metadata line). `Errored`
is a record the validator returned `Err` for. Neither is ever folded into `Clean` — that
fold is FM-3.

**D4 — a reconciliation invariant, enforced, not documented.**
`total_records == clean + flagged + indeterminate + errored + skipped` is asserted before
the report is emitted. On violation, `caro replay` prints the discrepancy to stderr and
exits `11` (internal validator error) rather than printing a report that under-counts. This
is what makes DEF-2 impossible to hide.

**D5 — the exit-code table is ADR-023's, unchanged, plus one flag.**

| Code | Meaning |
|---|---|
| 0 | No record exceeded `--fail-on` |
| 1 | ≥1 record with `suggested_routing: block` |
| 2 | ≥1 record with `suggested_routing: human_gate` and `--fail-on` ≤ `high` |
| 3 | ≥1 record with `suggested_routing: async_log` and `--fail-on` ≤ `moderate` |
| 10 | Input error — no history file found, unreadable path, unknown `--shell` |
| 11 | Internal error — validator `Err`, or D4 reconciliation failure |

`--fail-on-indeterminate` maps indeterminate records onto code **2**, not a new code: an
indeterminate verdict is definitionally "a human must look at this," which is what 2 means.
No sixth code is minted. This closes gator #2945 by construction.

**D6 — deterministic by construction, and tested for it.** Records are emitted in **corpus
order** (input order is the user's own history; re-sorting it destroys the thing they
recognize). All maps are `BTreeMap`. Nothing in the report body carries a timestamp, a
path, a duration, or a hostname. `--stable` additionally zeroes the header's
`generated_at` and `caro_version` so two runs are **byte-identical**, which is what the CI
test asserts. Diff mode compares on `(index, command_digest)`, never on message text —
gator's Rego-dedupes-on-message gotcha is designed out.

**D7 — redaction is the default; raw command text is opt-in.** The JSON report never
contains raw command text unless `--include-raw` is passed. By default each record carries
`command_digest` (SHA-256, hex, first 16 chars) and a `redacted` rendering produced by a
small deny-list of secret-shaped tokens (`--password=…`, `-p<value>` for mysql-family,
`AWS_SECRET_ACCESS_KEY=`, `Bearer <tok>`, `ghp_*`/`sk-*`/`xox[baprs]-*` prefixes,
`Authorization:` header values, anything after `--token`/`--api-key`). The **human-readable**
default output *does* print the command, because it is the user's own terminal and the
whole demo is recognition — but the moment it becomes a file someone might paste into an
issue, it is redacted. *Consequence:* redaction is best-effort and must be labelled as
such; `--include-raw` exists precisely so nobody builds a workaround. This has no analogue
in the survey because no surveyed tool reads a human's personal history.

**D8 — diff mode is verdict-level in v1, not rule-level.** `--baseline <report.json>`
compares the current run against a stored report and emits per-record
`unchanged | flipped_stricter | flipped_looser | added | removed`, plus counts.
`--fail-on-flip` defaults to **`looser`**: a policy edit that would newly *allow* something
previously blocked is the regression worth failing on; tightening is usually the intent.
Rule-level diffing ("which pattern stopped firing") requires stable rule identity, which
Caro does not have (`matched_patterns` is a `Vec<String>` of `description.to_lowercase()`,
pushed at `src/safety/mod.rs:523`, `:537`, `:550` for built-in, CVE and custom patterns
respectively). That is OOS-1, named honestly rather than half-built.

**D9 — DEF-1 and DEF-3 are fixed in this PR.** Allowlist regexes compile once in
`SafetyValidator::new` into a `compiled_allowlist: Vec<(Regex, String)>` field beside the
existing `compiled_patterns`; a compile failure is now a loud `ValidationError::PatternError`
consistent with how custom patterns are already treated. `COMPILED_PATTERNS`'s `filter_map`
gains a companion test asserting `COMPILED_PATTERNS.len() == DANGEROUS_PATTERNS.len()`, so
a silently-dropped built-in fails CI. Neither change alters any existing verdict; both are
in this feature's hot path.

## 3.3 New types

All in `src/safety/replay.rs`. Every type derives
`#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]` — serializable
from day one, per the constraint. `#[serde(rename_all = "snake_case")]` throughout;
`#[serde(deny_unknown_fields)]` on nothing, so the additive-field rule holds.

```rust
/// Top-level report. `schema_version` is 1 and is the only breaking-change signal.
pub struct ReplayReport {
    pub schema_version: u32,              // 1
    pub header: ReplayHeader,
    pub summary: ReplaySummary,
    pub records: Vec<ReplayRecord>,       // corpus order, always
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<ReplayDiff>,         // present iff --baseline
}

pub struct ReplayHeader {
    pub caro_version: String,             // "" under --stable
    pub generated_at: Option<String>,     // RFC3339; None under --stable
    pub source: CorpusSource,
    pub shell: ShellType,
    pub shell_provenance: ContextProvenance,
    pub safety_level: SafetyLevel,
    pub fail_on: RiskLevel,
    pub redacted: bool,                   // false only under --include-raw
    pub rules_declared: usize,            // DANGEROUS_PATTERNS.len() + cve + custom
    pub rules_loaded: usize,              // must equal rules_declared (DEF-3)
}

pub enum CorpusSource { History { shell_hint: ShellType }, File, Stdin }

/// Why we believe the context value we used. Answers FM-1 in the payload.
pub enum ContextProvenance { Explicit, InferredFromSource, Detected, Unknown }

pub struct ReplaySummary {
    pub total_records: usize,
    pub clean: usize,
    pub flagged: usize,
    pub indeterminate: usize,
    pub errored: usize,
    pub skipped: usize,
    pub by_risk: BTreeMap<RiskLevel, usize>,
    pub by_routing: BTreeMap<SuggestedRouting, usize>,
    pub highest_risk: RiskLevel,
}

pub struct ReplayRecord {
    pub index: usize,                     // 0-based position in corpus
    pub command_digest: String,           // sha256 hex, first 16 chars
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,          // redacted, or raw under --include-raw
    pub outcome: RecordOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,    // None for Skipped/Errored
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_routing: Option<SuggestedRouting>,
    pub matched_patterns: Vec<String>,    // descriptions today; see OOS-1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,           // why skipped / errored / indeterminate
    pub context: RecordContext,
}

pub enum RecordOutcome { Clean, Flagged, Indeterminate, Errored, Skipped }

/// Every field is Option. A None that a verdict depended on ⇒ Indeterminate (D2).
pub struct RecordContext {
    pub cwd: Option<String>,
    pub timestamp: Option<String>,
    pub exit_status: Option<i32>,
}

pub struct ReplayDiff {
    pub baseline_schema_version: u32,
    pub baseline_caro_version: String,
    pub unchanged: usize,
    pub flipped_stricter: usize,          // was allowed, now gated/blocked
    pub flipped_looser: usize,            // was gated/blocked, now allowed
    pub added: usize,                     // in current, absent from baseline
    pub removed: usize,
    pub flips: Vec<RecordFlip>,           // ordered by index
}

pub struct RecordFlip {
    pub index: usize,
    pub command_digest: String,
    pub from: SuggestedRouting,
    pub to: SuggestedRouting,
    pub direction: FlipDirection,
}

pub enum FlipDirection { Stricter, Looser }
```

**Method contracts:**

- `ReplayReport::reconcile(&self) -> Result<(), String>` — D4's invariant. Called before
  every emission, including human-readable.
- `ReplayReport::exit_code(&self, fail_on: RiskLevel, fail_on_indeterminate: bool,
  fail_on_flip: FlipPolicy) -> i32` — the only place an exit code is computed. Total,
  pure, unit-testable without a process.
- `ReplayReport::diff_against(&self, baseline: &ReplayReport) -> ReplayDiff` — keyed on
  `(index, command_digest)`; a mismatch in digest at the same index counts as
  `removed` + `added`, never a silent flip.
- `run_replay(args: ReplayArgs) -> Result<ReplayReport, ReplayError>` — the whole feature.
  Builds **one** `SafetyValidator` and loops. Takes an `impl BufRead` so tests do not touch
  the filesystem.
- `CorpusSource::resolve() -> Result<(Box<dyn BufRead>, ShellType, ContextProvenance), ReplayError>`
  — the only code that touches `$HOME`.

`ValidationResult` gains `JsonSchema` (one derive, `src/safety/mod.rs:174`) so the schema
generator can reach it.

## 3.4 Minimal file set

| File | Change |
|---|---|
| `src/safety/replay.rs` | **new** — the types above, corpus parsing, `run_replay`, redaction, diff |
| `src/safety/mod.rs` | `pub mod replay;`; add `JsonSchema` to `ValidationResult`; D9 — `compiled_allowlist` field, compiled in `new()`, used at `:499` |
| `src/safety/patterns.rs` | D9 — test asserting `COMPILED_PATTERNS.len() == DANGEROUS_PATTERNS.len()` |
| `src/main.rs` | one `Replay { … }` variant on `Commands`; one dispatch arm calling `run_replay` and `std::process::exit(report.exit_code(…))` |
| `src/bin/generate-schema.rs` | also emit `schema_for!(ReplayReport)` to `schemas/caro.replay.v1.json` |
| `schemas/caro.replay.v1.json` | **new, committed** — the contract as a reviewable artifact |
| `tests/replay_contract.rs` | **new** — §3.6 |
| `docs/adr/ADR-061-…md` | **new** |
| `CLAUDE.md`, `README.md` | one line each documenting `caro replay` |

Nine files, one of them generated, two of them documentation. No new module tree. No new
crate. **No new dependency: `sha2 = "0.10"` is already a direct dependency
(`Cargo.toml:90`), verified 2026-08-31.** `command_digest` uses it. Open question 1 below
is therefore resolved — recorded rather than deleted so the resolution is auditable.

## 3.5 Output contract

**JSON** (`--output json`) is `ReplayReport` verbatim. `schema_version: 1`. Additive fields
only within v1; a removal or a semantic change bumps to 2.

**Human-readable** (default) is the demo and is not machine-parsed:

```
caro replay — 4,812 commands from ~/.zsh_history (zsh, inferred from source)

  4,796 clean          16 flagged          0 indeterminate    0 errors    31 skipped

  CRITICAL  line  882   rm -rf ~/Projects/*
                        └─ recursive deletion of root, home, current, or parent directory
  HIGH      line 2,104  chmod -R 777 /var/www
                        └─ world-writable recursive permission change
  …

  16 of 4,812 commands (0.33%) would have been stopped or gated.
  52 patterns loaded of 52 declared.
```

The last two lines are the product. The percentage makes "zero false positives" checkable
by the only person whose opinion matters, and the pattern-count line makes DEF-3 visible.

**stderr** carries every diagnostic. **stdout** carries only the report. A caller that
redirects stdout to a file and gets exit 11 has an empty file and a reason on stderr —
never a truncated report. That is the FM-2 fix stated as an invariant.

## 3.6 Integration tests — `tests/replay_contract.rs`

Known inputs → deterministic JSON + exit code. No LLM, no network, no `$HOME` access
(every case feeds `run_replay` a fixture reader or invokes the binary with `--file`).

| # | Test | Input | Expected |
|---|---|---|---|
| 1 | `clean_corpus_exits_0` | `ls -la\ncd /tmp\necho hi` | `flagged: 0`, exit `0` |
| 2 | `critical_exits_1` | `rm -rf /` | `highest_risk: critical`, `routing: block`, exit `1` |
| 3 | `moderate_default_fail_on_high_exits_0` | `chmod 777 /tmp` | exit `0` (default `--fail-on high`) |
| 4 | `moderate_with_fail_on_moderate_exits_3` | same + `--fail-on moderate` | exit `3` |
| 5 | `mixed_corpus_counts_reconcile` | 3 safe, 1 critical, 1 blank, 1 `#comment` | `total == clean+flagged+indeterminate+errored+skipped`, `skipped: 2`, exit `1` |
| 6 | `unreadable_source_exits_10` | `--file /nonexistent` | exit `10`, **empty stdout**, message on stderr |
| 7 | `reconciliation_failure_exits_11` | injected miscount via test-only constructor | exit `11`, empty stdout |
| 8 | `byte_identical_under_stable` | same corpus, two runs, `--stable --output json` | outputs compare `==` byte-for-byte |
| 9 | `record_order_is_corpus_order` | shuffled fixture | `records[i].index == i` for all `i` |
| 10 | `shell_specific_pattern_unknown_shell_is_indeterminate` | PowerShell-only pattern + `--shell unknown` | `outcome: indeterminate`, not `clean` |
| 11 | `redaction_is_default` | `export AWS_SECRET_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE` | JSON `command` contains no `AKIA…`; `redacted: true` |
| 12 | `include_raw_opts_in` | same + `--include-raw` | raw text present, `redacted: false` |
| 13 | `diff_detects_looser_flip` | baseline with `block`, current with `auto_approve` | `flipped_looser: 1`, `--fail-on-flip looser` ⇒ exit `1` |
| 14 | `diff_detects_stricter_flip` | inverse | `flipped_stricter: 1`, default `--fail-on-flip looser` ⇒ exit `0` |
| 15 | `diff_digest_mismatch_is_add_remove` | same index, different command | `added: 1, removed: 1, flips: []` |
| 16 | `schema_validates` | any report | validates against committed `schemas/caro.replay.v1.json` |
| 17 | `rules_loaded_equals_declared` | any report | `rules_loaded == rules_declared` (DEF-3) |
| 18 | `invalid_allowlist_regex_is_loud` | config with `allowlist_patterns = ["[unclosed"]` | `SafetyValidator::new` returns `Err`, not a silent drop (DEF-2/D9) |

Tests 5, 7, 10, 17 and 18 exist specifically because Phase 1 found shipping tools that got
each one wrong. Test 8 is the determinism claim; without it the word is marketing.

## 3.7 Gate 3 — what breaks at 100 real users

Per `.claude/rules/validation-discipline.md`.

- **Assumption that holds at demo scale:** the corpus fits in memory and is a few thousand
  lines. **Breaks at:** a 500 MB `~/.zsh_history` from a decade-old dotfiles repo, or an
  `atuin` SQLite store presented as a file. **Failure mode:** OOM, or a multi-minute hang
  with no output. **Instrumentation:** the header records `total_records`; the human output
  prints a progress line every 10,000 records to stderr. **Fallback:** streaming via
  `BufRead::lines()` — never `read_to_string` — and `--limit` defaulting to the whole file
  but documented, with a stderr warning above 100,000 records.
- **Assumption:** history lines are one command each, UTF-8. **Breaks at:** zsh
  `EXTENDED_HISTORY` (`: <ts>:<elapsed>;<cmd>`), fish's YAML-ish format, multi-line
  continuations, and non-UTF-8 bytes from a terminal paste. **Failure mode:** garbage
  commands validated as if real, inflating or deflating the headline percentage — which is
  the *one number the whole feature exists to produce*. **Instrumentation:** `skipped` with
  a `reason`; a `skipped / total` ratio above 5% prints a stderr warning naming the format.
  **Fallback:** parse zsh extended format explicitly (it is the most common), lossy-decode
  non-UTF-8 into `skipped`, and never guess.
- **Assumption:** redaction catches the secrets. **Breaks at:** a secret shaped like none
  of the deny-list entries — a bare token as a positional argument. **Failure mode:** a
  user pastes a replay report into a GitHub issue containing a live credential. **This is
  the most serious risk in the feature.** **Instrumentation:** the report carries
  `redacted: true` and the docs state plainly that redaction is best-effort.
  **Fallback:** default JSON output omits `command` entirely unless `--include-raw`; the
  digest is what the diff needs, and the diff is the machine use case. Human output prints
  to the user's own terminal and is not a file.
- **Assumption:** the validator is fast enough to loop. **Breaks at:** DEF-1's per-command
  allowlist recompilation × 100,000 records. **Instrumentation:** a `criterion` bench in
  `benches/` over a 10,000-record synthetic corpus. **Fallback:** D9 fixes it before the
  feature can expose it. This is the whole reason DEF-1 is in scope rather than deferred.
- **Nothing breaks in concurrency, because there is none.** Single-threaded, single-pass,
  no shared state, no daemon. Stated so a reviewer can check it rather than assume it.

## 3.8 Explicitly out of scope

- **OOS-1 — stable rule identity (`RuleId`).** `matched_patterns` holds
  `description.to_lowercase()` (`src/safety/mod.rs:534`), so a typo fix in a description
  silently changes the diff key. Rule-level would-flip analysis needs a declared `id` on
  `DangerPattern` and a migration across all 52+ built-ins and the CVE YAML. That is its
  own PR and its own decision (declared id vs. digest of the regex source). v1 diffs on
  verdict; §3.2 D8 says so out loud. **This is the top follow-up.**
- **OOS-2 — agent-transcript corpora.** ADR-056 owns this. Replaying Claude Code / Codex
  tool-call logs needs a transcript parser per host and belongs with the hook adapters
  (memo §3.3), not here.
- **OOS-3 — the hash chain and `caro audit rehearse`.** ADR-056 §, blocked on ADR-032.
  A replay report is a leaf document; it is not chained, not signed, not tamper-evident,
  and the docs must not imply otherwise.
- **OOS-4 — `caro scan`.** ADR-023 stays Proposed. `caro replay` implements its contract
  over one input source; `caro scan` is the same code over script files and stdin, and is
  a small follow-up PR once the contract is exercised.
- **OOS-5 — SARIF output.** ADR-023 specifies it; it belongs with `caro scan` and CI
  annotation, not with the local demo.
- **OOS-6 — environment-scoped tiers** (memo §3.2). `RecordContext` deliberately has room
  for it — `cwd` is already there — but v1 does not read it, and adding an `environment`
  field now would be designing against an unwritten ADR.
- **OOS-7 — `caro.assessment.v1`.** Under the ADR-059 moratorium. `ReplayReport` embeds
  `RiskLevel` and `SuggestedRouting`, which shipped in 1.4.0; when the assessment payload
  lands, `ReplayRecord` gains one additive field and `schema_version` stays 1.
- **OOS-8 — anything hosted.** No upload, no telemetry, no "share your replay." Today's
  memo's do-not-build list, and §2.3's privacy argument would not survive it.

## 3.9 Validation-discipline status

- **Gate 1 (20 transcripts) — NOT discharged, and this is the honest reading.** Today's
  memo says so directly: *"It does not satisfy validation-discipline gate 1. Shell
  histories are artifacts, not first-hand interviews."* The defensible position is that
  `caro replay` is a **diagnostic on existing behaviour** — it adds no new verdict, no new
  pattern, no new decision; it prints what `SafetyValidator` already computes over input
  the user supplies. The rule exempts work that responds to evidence we already have. **That
  exemption must be claimed explicitly in the PR description and argued there**, per the
  memo's own gate note. If a reviewer rejects the exemption, the feature waits for the
  interviews; it does not ship on a technicality.
- **Gate 2 (no surveys) — n/a.**
- **Gate 3 (demoware trap) — discharged**, §3.7.
- **Gate 4 (devil's advocate) — REQUIRED before the PR opens.** Not discharged here.
- **Gate 5 (Sean Ellis) — n/a**; this scope makes no PMF claim.

## 3.10 Implementation checklist

- [ ] `src/safety/mod.rs`: `compiled_allowlist` compiled in `new()`, used at `:499`;
      invalid allowlist regex ⇒ `ValidationError::PatternError` (DEF-1, DEF-2)
- [ ] `src/safety/patterns.rs`: test `COMPILED_PATTERNS.len() == DANGEROUS_PATTERNS.len()` (DEF-3)
- [ ] `src/safety/mod.rs`: `JsonSchema` on `ValidationResult`; `pub mod replay;`
- [ ] `src/safety/replay.rs`: types from §3.3, all `Serialize + Deserialize + JsonSchema`
- [ ] `src/safety/replay.rs`: `reconcile()` called on every emission path (D4)
- [ ] `src/safety/replay.rs`: `exit_code()` as the single source of process codes (D5)
- [ ] `src/safety/replay.rs`: zsh `EXTENDED_HISTORY` parsing; lossy-decode ⇒ `Skipped`
- [ ] `src/safety/replay.rs`: redaction deny-list; `--include-raw` opt-in (D7)
- [ ] `src/main.rs`: `Replay` variant + dispatch arm; no logic in `main.rs`
- [ ] `src/bin/generate-schema.rs` + committed `schemas/caro.replay.v1.json`
- [ ] `tests/replay_contract.rs`: all 18 cases from §3.6
- [ ] `benches/`: 10,000-record corpus bench, before and after DEF-1
- [ ] `README.md` / `CLAUDE.md`: one line each
- [ ] PR description: explicit gate-1 exemption claim (§3.9)
- [ ] **Gate 4**: `devils-advocate` review posted on the PR before merge
- [ ] Feature branch per `.claude/rules/git-workflow.md` — never `main`

## 3.11 Open questions

1. ~~**Is `sha2` (or `blake3`) a direct dependency?**~~ **Resolved 2026-08-31**:
   `sha2 = "0.10"` at `Cargo.toml:90`. `command_digest` adds no dependency.
2. **Does `--fail-on-flip looser` default correctly?** It fails a run where a policy edit
   newly permits something. Defensible, but it means a user who *deliberately* relaxes a
   pattern gets a red CI. One reviewer opinion, not a rewrite.
3. **Should `caro replay` with no history file found exit 10 or print an onboarding
   message and exit 0?** Exit 10 is contract-correct; it also makes the flagship demo fail
   loudly on a fresh container. Recommend 10 with a human-readable stderr hint.
4. **Who owns ADR-023 now?** `caro replay` implements its contract without superseding it.
   If ADR-023's author disagrees with D1, that is a comment on the PR, and the exit table
   is the thing to defend.

## 3.12 References

- `.hermes/digests/2026-08-31-agent-launch-scan.md` §3.1, §4 — the target, and the
  dependency-decoupling instruction this scope follows
- ADR-023 — `caro scan`; the exit-code contract adopted verbatim in D1/D5
- ADR-056 — enforcement rehearsal; this is its unblocked subset (§ provenance note)
- ADR-059 — the moratorium this scope stays outside of
- ADR-060 — the `--stable` byte-equality pattern reused in D6 and test 8
- `.claude/rules/validation-discipline.md` — gates, §3.9
- `.claude/rules/good-boy-scout.md` — DEF-1/DEF-3 scoped to this PR's path
- gator: `open-policy-agent.github.io/gatekeeper/website/docs/gator/`,
  `/docs/enforcement-points/`, `/docs/audit/`; issues #2863, #2945 (open), #3023
- Kyverno CLI: `kyverno.io/docs/subprojects/kyverno-cli/`; issues #4255, #7681, #9804
- Conftest: `conftest.dev/options/`; issue #387
- Envoy RBAC filter: `envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/rbac_filter.html`
- AWS: `docs.aws.amazon.com/cli/latest/reference/accessanalyzer/check-no-new-access.html`
- Kastra: Product Hunt product page + founder comments — **unverified vendor marketing**,
  cited once in §1.6, load-bearing nowhere
