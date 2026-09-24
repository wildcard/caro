# Implementation Scope — `caro.eval.v1`: a report contract and a CI gate that actually gates

**Feature under analysis:** **`claude plugin eval`** — Claude Code's early-access plugin
evaluation suite: `evals/<case>/prompt.md` + `graders/*.md`, per-run isolated `claude -p`
sessions, a typed grader taxonomy, `aggregate-result.json` at `schemaVersion: "1"`, a
`--threshold` CI gate, and a five-value exit-code contract. Researched **2026-08-27**.

**Equivalent we are scoping for Caro:** not a new harness. Caro has **four**. This scopes
the PR that picks one, gives its report a version and a stable serialization, and replaces
a CI gate that is currently **incapable of failing**.

**Date:** 2026-08-27 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-060-eval-report-contract-and-harness-consolidation.md`

---

> ### Provenance note — target selection and the ADR-059 moratorium
>
> This scheduled task's template leaves `[FEATURE NAME]` unbound. No user was present.
> Target selection was mine, and it was constrained by yesterday's run:
>
> **ADR-059 (2026-08-26) declared itself "the last ADR in this space until
> `src/safety/assessment.rs` merges."** That file still does not exist — `src/safety/`
> contains `cve_patterns.rs`, `mod.rs`, `patterns.rs` and nothing else. The moratorium
> holds, and it applies to the assessment/governance surface: ADR-035 → ADR-059.
>
> This scope is deliberately **outside** that surface. It touches no policy, no verdict
> payload, no approval exchange, no lifecycle event. It is about test infrastructure.
>
> It also does the thing the 2026-08-26 strategy memo actually asked for, which was not
> "write fewer ADRs" but "close the gap between the ADR backlog and the `src/` tree." The
> headline of this scope is a **net deletion**: one engine survives, one is folded in, one
> whole cargo package leaves the tree along with **669 committed build artifacts**. No new
> module is created. No new dependency is added.
>
> **Why this target and not another.** The memo's central claim — and the README's, and
> `CLAUDE.md`'s — is a number: *93.1% pass rate*, *94.8% CSR*. Phase 2 establishes that
> the CI job which is supposed to defend those numbers has, by construction, never failed
> and cannot fail. Everything downstream of a quality claim depends on the claim being
> measured. That makes this infrastructure, not polish.
>
> ### Verification status
>
> - **Verified against the Caro tree (2026-08-27, branch `integrator/20260711-postmerge`):**
>   every file path, line number, struct field, flag, exit code, workflow step and absence
>   claim in Phase 2 and Phase 3 was produced by reading `src/`, `tests/`, `Cargo.toml`,
>   `.github/workflows/evaluation.yml` and `git ls-files`. Line numbers will drift; the
>   claims are stated so they can be re-checked with a single grep.
> - **Second-hand, single-source:** the `claude plugin eval` surface in Phase 1. There is
>   **no public documentation page** for this command — it is early-access and gated per
>   organization. The description came from the `claude-code-guide` agent's embedded
>   early-access reference (current as of client v2.1.224). Field names, flags, exit codes
>   and the JSON shape below are quoted from that reference and are **not independently
>   verifiable from a public URL.** Every Phase 1 claim should be read with that caveat,
>   and none of Phase 3's decisions depend on any single one of them being exactly right —
>   they depend on the *design patterns*, which are robust to detail drift.
> - **Explicitly not verified:** whether the JSON field casing is uniformly camelCase (the
>   reference shows `duration_ms` alongside `passRate`, which is internally inconsistent
>   and may be a transcription artifact); the default judge model; whether the runner
>   parallelizes cases.
> - **Blocked:** `https://code.claude.com/docs/en/plugin-evals` returns HTTP 404. A
>   third-party OSS reference (`wshobson/agents/docs/plugin-eval.md`) surfaced in search
>   but could not be fetched — outside the fetch provenance set. Reported, not worked around.
>
> ### Process compliance
>
> - **`.claude/rules/git-workflow.md`** — this file and the ADR are written **uncommitted**
>   on the working tree. Nothing was committed to `main` or to the current branch. A human
>   branches and PRs.
> - **`.claude/rules/adr-numbering.md`** — ADR-060 is the next sequential number; ADR-059
>   is the highest present in `docs/adr/`.
> - **`.claude/rules/validation-discipline.md`** — Gate 3 (demoware trap) is written at
>   §3.7. Gate 4 (`devils-advocate` review) is **required before the implementation PR
>   opens** and is not discharged here. Gates 1, 2 and 5 do not attach: this is test
>   infrastructure for an existing shipped capability, not a new user-facing product line
>   (the rule's own carve-out for "responses to evidence we already have").
> - **`.claude/rules/good-boy-scout.md`** — §3.6 is scoped as cleanup of things this PR
>   touches. It does not refactor the evaluators, the dataset schema, or the backends.

---

## Phase 1 — Feature Research: `claude plugin eval`

*Source: `claude-code-guide` early-access reference, client ≥ v2.1.224. No public URL.
See Verification status above.*

### 1.1 What problem it solves, and for whom

The buyer is someone shipping a **plugin, skill, or agent extension** who cannot tell
whether it works. Unit tests answer "does the code run"; they cannot answer "does the
model *reach for* this skill when it should, and produce something acceptable when it
does." The failure being targeted is silent: a skill that never triggers, or triggers and
degrades the answer, looks identical to a skill that works, from the outside.

The mechanism: each **eval case** is a prompt plus a set of **graders**, run in a fresh
isolated `claude -p` session with only the plugin under test loaded, **three times by
default**, and scored. Optionally a **baseline arm** runs the same prompt with the plugin
*absent*, and the report gives the delta — *with minus without*. That ablation is the
sharpest idea in the design: it separates "the model got it right" from "the plugin made
the model get it right," which is the only question a plugin author actually has.

### 1.2 Why it is early-access, and the failure modes

The reference does not state a rationale for early-access status. The failure modes are
inferable from the mechanics, and three of them are load-bearing for Phase 3:

**F1 — The gate is deterministic-looking and stochastically fed.** `--threshold` defaults
to `1.0`: every case must pass. But the `llm` grader is decided by a **2-of-3 vote** of a
judge model, and the reference notes it is "noisy on long inputs." A suite mixing `regex`
graders with `llm` graders produces a pass rate whose variance is unquantified and
unreported. A CI job that goes red therefore has two possible meanings — regression, or
judge noise — and the report carries **no field that distinguishes them**. There is no
seed, no confidence interval, and no per-grader determinism flag. Three runs per case is a
tacit admission that variance exists; nothing in the output contract lets a consumer reason
about it.

**F2 — Exit code 2 conflates unrelated causes.** `2` means "partial," and the reference
gives two triggers: cost ceiling hit mid-run, and credentials rejected at the first run.
Those are disambiguated only by a `partialReason` field *inside the JSON* (`cost_ceiling`
/ `auth_failed`). A CI step that branches on `$?` alone — which is what an exit code is
*for* — cannot tell "your budget ran out" from "your token is invalid." The exit code and
the report disagree about how much information the exit code carries.

**F3 — The report is not diffable.** The artifact lands at
`<eval dir>/results/<timestamp>/aggregate-result.json`. The path contains a timestamp, so
finding it requires a glob; the body contains `duration_ms` and token counts, so two runs
over an identical suite on an identical plugin **never** produce identical bytes. There is
no documented "scored content only" projection. Every consumer that wants "did anything
change" must write its own field-subsetting logic, and every consumer will pick a slightly
different subset.

Secondary, worth noting: the isolation is **not an OS sandbox** — "network is not blocked";
the report **auto-publishes to a private URL by default** unless `--no-publish`; and the
Artifact tool is unavailable in-run, so a skill whose output *is* an artifact is graded on
something other than what it ships.

### 1.3 Structured output contract

```jsonc
{
  "schemaVersion": "1",                       // stable since client 2.1.210
  "suite":      { "name": "...", "total": 5, "passed": 4, "passRate": 0.8 },
  "cases": [{
    "id": "case-id", "name": "Case name",
    "arms": {
      "with":    [{ "runIndex": 0, "passed": true, "duration_ms": 5432,
                    "tokens": { "input": 1234, "output": 567 },
                    "graders": [{ "name": "...", "type": "regex",
                                  "passed": true, "score": 1.0, "message": "..." }],
                    "aborted": null,          // or {server, tool, reason}
                    "withOnly": false, "scored": true }],
      "without": [ /* baseline arm, only under --ablation with-without */ ]
    }
  }],
  "aggregates": { "total_runs": 5, "total_passed": 4, "pass_rate": 0.8,
                  "avg_tokens": 1900.5, "total_cost_usd": 0.042 }
}
```

Versioning is **additive-only**; unknown fields are tolerated for forward compatibility.
An HTML `report.html` lands beside it.

**Exit codes**

| Code | Meaning |
|---|---|
| 0 | all cases at or above `--threshold` (default `1.0`) |
| 1 | below threshold, load error, no cases found, bad options, gate closed |
| 2 | partial — cost ceiling mid-run, or auth rejected at first run |
| 130 | interrupted (SIGINT) |
| 143 | terminated (SIGTERM) |

**Grader taxonomy** — this is the part worth copying:

| Type | Decides by | Deterministic |
|---|---|---|
| `regex` | `pattern` + `match: contains \| not_contains \| count:N` | yes |
| `tool_used` | tool invoked, optional `input_match`, `min`/`max` | yes |
| `tool_order` | `before` / `after` sequencing | yes |
| `file_exists` | glob over files the agent created | yes |
| `llm` | `criteria` judged by a model, 2-of-3 vote | **no** |
| `baseline` | LLM comparison against a `baseline_file` | **no** |

Graders select what they inspect with a **`target`**: `last_message` (default), `trace`,
`files`, `{source: file, path: ...}`, or `mock_calls`. Separating *what to look at* from
*how to judge it* is the right factoring and Caro has no equivalent.

### 1.4 Session and context lifecycle

Each run gets a throwaway workspace, a fresh `CLAUDE_CONFIG_DIR` and `HOME`, only the
plugin under test loaded, `dontAsk` mode with read-only tools unless `--allow-tools` grants
more, and credentials copied in after scaffold then deleted at run end. **No state is
shared between cases.**

**Redundant initialization is not addressed.** There is no snapshotting, no session reuse,
no warm pool. The reference documents no caching mechanism at all. With `--runs 3` as the
default and a fresh `claude -p` session per run, a 20-case suite is 60 cold model sessions,
each paying full initialization. `--max-cost-usd` exists precisely because this is
expensive; there is a `--mocks record` mode for MCP servers, which caches *tool responses*
but not the session itself. This is the clearest place where Caro's architecture is simply
better, and §2.3 says why.

---

## Phase 2 — Competitive Differentiation

### 2.1 What they get right and we should replicate

1. **A versioned report.** `schemaVersion: "1"`, additive-only, with a stated compatibility
   rule. Caro's report struct (`BenchmarkReport`, `src/evaluation/models.rs:251`) has
   **no version field at all**.
2. **A typed grader taxonomy with a separate `target` selector.** Caro's entire notion of
   "expected" is `expected_output: Option<String>` plus `expected_outputs: Vec<String>`
   (`src/eval/mod.rs:29–37`) — exact string match against a list of literals. There is no
   regex, no negative assertion, no "the safety verdict must be `block`."
3. **Ablation.** `--ablation with-without` and the `withOnly` / `scored` flags on a run.
   Caro has no way to express "this case exists to prove the feature is doing the work."
4. **Explicit exit codes as a published contract**, distinct per cause.
5. **Cost ceiling as a first-class abort.** Not applicable to Caro (local inference is
   free), but the *pattern* — a resource ceiling that aborts and says so in the report —
   maps onto wall-clock budget on slow CI runners.

### 2.2 Their gaps, which we avoid by designing the schema first

F1, F2 and F3 from §1.2. Restated as design obligations:

- **Against F1:** every grader result carries a `deterministic: bool`. A suite containing
  any non-deterministic grader **cannot be a CI gate** unless the operator passes
  `--allow-nondeterministic-gate`. The gate refuses rather than reporting a number it
  cannot defend.
- **Against F2:** one cause, one exit code. Never a `reason` field that the exit code
  needed and did not carry.
- **Against F3:** the report is split into a volatile envelope and a stable scored body,
  and `--json --stable` emits only the body — byte-identical across runs. Caro can offer
  this and Anthropic structurally cannot (§2.3).

### 2.3 Caro's unique positioning

**The runner is a pure function, not an agent session.** `claude plugin eval` scores a
stochastic system, so it runs three times and votes. Caro's `StaticMatcher` is a
deterministic template matcher, and the embedded backends run locally at fixed sampling.
For the static backend, *one run is the correct number of runs*, and two runs over the same
dataset produce the same bytes. **Byte-reproducible eval output is a claim Caro can make
and its competitors cannot.** For a project whose pitch is "deterministic floor beneath a
probabilistic gate," having a *non-reproducible* quality metric is an own-goal; fixing it
is on-thesis, not incidental.

**One process, one model load, N cases.** Anthropic spawns a fresh session per run because
the unit under test is a session. Caro's unit under test is a function call. The harness
already loads a backend once and iterates (`EvaluationHarness::new(...)` then `.run()`,
`tests/evaluation/main.rs:174–207`). The template's "how does it avoid redundant
initialization?" has, for Caro, a structural answer rather than a caching answer — which
is the better kind of answer, and it needs to be *stated in the ADR* so nobody later
"optimizes" toward per-case subprocesses.

**Offline, free, no publishing.** No cost ceiling because there is no cost. No
`--publish-report` default-on, because Caro's audience is the air-gapped and the regulated.
No network in the loop at all for the static backend.

**A safety verdict is a legitimate assertion.** Caro is the only one of these tools where
"the correct outcome is that this command is *refused*" is a first-class expectation.
`EvalCategory::DangerousCommands` already exists (`src/eval/mod.rs:100`) and there is
nothing to assert against it except the literal text of the generated command.

### 2.4 Existing infrastructure that already covers part of this — and the actual finding

Caro does not lack an eval harness. **It has four**, and the good one is not the one CI
runs.

| # | Location | What it is | Reachable from |
|---|---|---|---|
| 1 | `src/eval/mod.rs` (1 file, 17 KB) | `EvalSuite` / `EvalCase` / `EvalResults`; YAML loader; 12 categories incl. `DangerousCommands`; `print_summary()` via `colored` | `caro test` (`src/main.rs:435`, handler at `:2839`) |
| 2 | `src/evaluation/` (7 files) | `BenchmarkReport` with `run_id`/`branch`/`commit_sha`/`regression_detected`; `BaselineStore` with configurable regression threshold; trait-based evaluators (correctness, safety, posix, consistency); parallel harness | `tests/evaluation/main.rs` only — **zero call sites in `src/`** |
| 3 | `tests/evaluation/src/` (23 files) | a whole separate cargo package, `caro-evaluation` 0.1.0, with its own `Cargo.toml` and `Cargo.lock`: `reporter`, `executor`, `execution_cache`, `token_tracker`, `dashboard`, `timeseries`, `quality_metrics`, `capability_matrix`, `issue_automation`, `pattern_extraction`, `prompt_comparison`, `training_tracker`… | **nothing.** Root `Cargo.toml` has no `[workspace]`, so `cargo test` never builds it |
| 4 | `tests/evaluation.rs` + `tests/evaluation/{dataset,harness,validators,reporter}.rs` | a fourth harness over `test_cases.toml`, with its own `EvaluationResult`/`CSR` shape and its own JSON schema at `kitty-specs/025-llm-evaluation-harness/contracts/evaluation_result_schema.json` | itself |

Plus `run_eval.py`, `run_eval.sh`, `run_eval_v2.sh` in `tests/evaluation/`.

**`git ls-files tests/evaluation` returns 732 tracked files. 669 of them are under
`tests/evaluation/target/`** — a build-artifact directory committed to the repository.

**`CLAUDE.md` documents `cargo run --bin caro-eval`. No such binary exists.** `Cargo.toml`
declares exactly two: `caro` and `generate-schema`.

#### 2.4.1 The CI gate cannot fail

`.github/workflows/evaluation.yml` runs on every PR to `main` across a four-way matrix
(`static_matcher`, `mlx`, `embedded-smollm`, `embedded-qwen`). Its scoring step is:

```yaml
cargo test --test evaluation -- --backend ${{ matrix.backend }} 2>&1 \
  | tee evaluation-${{ matrix.backend }}.log || true

if grep -q "Passed:" evaluation-${{ matrix.backend }}.log; then
  PASS_RATE=$(grep "Passed:" ... | grep -o '( *[0-9.]*%)' | tr -d '()% ')
  echo "backend_available=true" >> $GITHUB_OUTPUT
else
  echo "backend_available=false" >> $GITHUB_OUTPUT
  echo "pass_rate=0" >> $GITHUB_OUTPUT
fi
```

Five independently sufficient reasons this never goes red:

1. **`|| true`** discards the exit code. `tests/evaluation/main.rs` is a `harness = false`
   custom test binary (`Cargo.toml:178–181`) that carefully computes an exit code at
   `:230–235` and calls `process::exit`. The workflow throws it away.
2. **The gate greps for a string the harness never prints.** The scored output path is
   `output_table()` (`main.rs:260+`), which prints `Run ID:`, `Branch:`, `Commit:` and a
   box-drawn Overall Results table. `"Passed:"` appears nowhere in it. The only
   `println!("Passed: …")` in the tree is `tests/evaluation/tests/test_correctness.rs:161`,
   which belongs to package #3 and is never built.
3. **Two of the four matrix legs are rejected as usage errors.** `validate_args`
   (`main.rs:104–115`) accepts `static_matcher | mlx | ollama | vllm`. `embedded-smollm`
   and `embedded-qwen` fail validation → `process::exit(2)` (`main.rs:75`). Swallowed by
   `|| true`.
4. **When the grep misses, the threshold step is skipped entirely** —
   `if: steps.evaluation.outputs.backend_available == 'true'`. Failure to measure is
   encoded as success.
5. **Three of four baselines are `0.0`.** The gate is a bash `case` statement with
   `static_matcher) BASELINE="31.0"`, everything else `"0.0" ;; # TBD`, and an explicit
   `if BASELINE == 0.0 → exit 0`.

And even if all five were fixed, `--backend` is **accepted, validated, and then ignored**:

```rust
// tests/evaluation/main.rs:167–171
if args.backend.is_some() {
    eprintln!("Warning: Backend filtering is not yet implemented. Running all backends.");
}
```

…where "all backends" is one hard-coded `StaticMatcher` with an `ubuntu()` capability
profile (`:180–184`). The four-way matrix runs the same static matcher four times.

**Consequence.** The `93.1% pass rate` in `CLAUDE.md` and the `94.8% CSR` in ROADMAP and
the strategy memo are not defended by CI. They may well be true. Nothing in the repository
currently establishes that they are, and nothing would notice if they stopped being.

#### 2.4.2 Infrastructure to reuse, not rebuild

- `schemars = "0.8"` is **already a dependency** (`Cargo.toml:46`) and
  `src/bin/generate-schema.rs` already emits a JSON Schema from a Rust type. The schema
  artifact is an extension of an existing binary, not a new one.
- `BaselineStore` (`src/evaluation/baseline.rs:14`) already stores reports per branch,
  loads them, and does threshold-based regression detection with per-category and
  per-backend deltas (`:212–294`), fully unit-tested (`:405–424`). The bash `case`
  statement is reimplementing, worse, a thing that exists.
- `kitty-specs/025-llm-evaluation-harness/contracts/evaluation_result_schema.json` (253
  lines, draft-07, `$id: .../schemas/evaluation_result.json`) is a real, well-formed
  contract — for harness #4. Its `backend` enum is `["mlx","vllm","ollama"]`, which
  excludes every backend the CI matrix names, including `static_matcher`. It is prior art
  and a warning: a schema nobody generates from code drifts from the code.
- `SafetyValidator` (`src/safety/`) is the thing a `verdict` grader calls. No new
  validation logic.

#### 2.4.3 Two meanings of "threshold" in one function

`--threshold` (default `0.05`) is a **regression delta** passed to `BaselineStore::compare`.
`HarnessConfig.regression_threshold` (hard-coded `0.95` at `main.rs:161`) is a **pass rate**.
The actual exit code uses neither:

```rust
// tests/evaluation/main.rs:230–235
let exit_code = if regression_detected || report.overall_pass_rate < 1.0 { 1 } else { 0 };
```

— a hard-coded 100% gate. Three thresholds, one of them silently authoritative.

---

## Phase 3 — Scope Definition

**Deliverable:** one PR. `caro.eval.v1` — a versioned, stably-serializable evaluation report
and an exit-code contract, emitted by a single consolidated harness, consumed by a CI
workflow that parses JSON instead of grepping a table.

### 3.1 New and changed types

All in `src/evaluation/models.rs`. **No new module.** All types already derive
`Serialize, Deserialize`; the additions do too, plus `JsonSchema` (schemars, already a dep).

```rust
/// Wire envelope. The only thing a consumer matches on before parsing.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvalReport {
    /// Always "caro.eval.v1". First field. Additive-only evolution.
    pub schema_version: String,
    /// Everything that legitimately differs between two identical runs.
    pub run: RunMetadata,
    /// Everything that must NOT differ between two identical runs.
    pub scored: ScoredReport,
}

/// Volatile. Omitted entirely under `--stable`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunMetadata {
    pub run_id: String,              // moved from BenchmarkReport
    pub timestamp: DateTime<Utc>,    // moved
    pub branch: String,              // moved
    pub commit_sha: String,          // moved
    pub caro_version: String,        // new — env!("CARGO_PKG_VERSION")
    pub execution_time_ms: u64,      // moved
    pub host_platform: String,       // new — target triple
}

/// Stable. Byte-identical across runs for a deterministic backend + fixed dataset.
/// Every map is serialized as a key-sorted BTreeMap; every Vec is sorted by test_id.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScoredReport {
    pub dataset_digest: String,      // new — BLAKE3 of the dataset file; ties a score to an input
    pub backend: String,             // new — the backend ACTUALLY registered, not the one requested
    pub deterministic: bool,         // new — false if any grader or backend is stochastic
    pub overall_pass_rate: f32,
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub category_results: BTreeMap<TestCategory, CategoryResult>,   // was HashMap
    pub backend_results: BTreeMap<String, BackendResult>,           // was HashMap
    pub cases: Vec<CaseResult>,      // new — per-case, sorted by test_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_comparison: Option<BaselineDelta>,
    pub regression_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaseResult {
    pub test_id: String,
    pub category: TestCategory,
    pub passed: bool,
    pub graders: Vec<GraderResult>,  // sorted by grader name
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GraderResult {
    pub name: String,
    pub kind: GraderKind,
    pub passed: bool,
    /// The F1 fix. A grader that cannot be replayed says so, here, per result.
    pub deterministic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// v1 is deterministic-only, by construction. There is no `LlmJudge` variant to
/// accidentally use. Adding one is a schema change and an ADR.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GraderKind {
    /// Existing behaviour: literal equality against expected_output / expected_outputs.
    ExactAnyOf { expected: Vec<String> },
    /// New. Adapted from their `regex` grader.
    Regex { pattern: String, negate: bool },
    /// New. Caro-specific and the reason this taxonomy is worth having:
    /// asserts the SafetyValidator verdict, reusing src/safety/. No new logic.
    Verdict { expected: SuggestedRouting },
    /// New. Adapted from their `target:` selector, minimal form.
    ExitStatus { expected: i32 },
}

impl GraderKind {
    /// Every v1 variant returns true. The method exists so that the day a
    /// stochastic grader is added, `deterministic` is a compile-time obligation
    /// rather than a field someone forgets to set.
    pub fn is_deterministic(&self) -> bool { true }
}
```

`BenchmarkReport` is **not deleted** — it becomes the internal accumulator that
`EvaluationHarness::run()` returns, and gains one method:

```rust
impl BenchmarkReport {
    pub fn into_report(self, dataset_digest: String, backend: String) -> EvalReport;
}
```

`BaselineStore` continues to store and compare `BenchmarkReport`. Baselines on disk are
unaffected; only the *emitted* shape changes.

### 3.2 Files that change

Nine files touched, one directory removed. No new module, no new dependency, no new binary.

| File | Change |
|---|---|
| `src/evaluation/models.rs` | Add the six types above; `HashMap` → `BTreeMap` on the two report maps |
| `src/evaluation/harness.rs` | Populate `Vec<CaseResult>` / `Vec<GraderResult>` while running; compute `dataset_digest`; record the backend actually registered |
| `src/evaluation/evaluators/mod.rs` | Evaluators return `GraderResult` instead of a bare bool |
| `src/evaluation/dataset.rs` | Parse optional `graders:` on a test case; absence ⇒ one `ExactAnyOf` grader, so **every existing dataset entry keeps working unchanged** |
| `src/bin/generate-schema.rs` | Second `schema_for!(EvalReport)` → `schemas/caro.eval.v1.json`, committed |
| `tests/evaluation/main.rs` | New flags (§3.3); real exit codes (§3.4); `--backend` honoured or refused; delete the "not yet implemented" warning |
| `src/main.rs` | `caro test` re-points at `src/evaluation`; `src/eval::EvalSuite` YAML loading is kept as a dataset front-end |
| `.github/workflows/evaluation.yml` | Rewritten (§3.5) |
| `CLAUDE.md` | Delete `cargo run --bin caro-eval`; document the real invocation |
| `tests/evaluation/src/`, `tests/evaluation/target/`, `tests/evaluation/Cargo.{toml,lock}`, `run_eval*.{sh,py}` | **Removed** — 669 committed build artifacts plus an unbuildable package |

`src/eval/mod.rs` survives as the YAML suite format and the 12-category taxonomy. It loses
`print_summary()` (rendering moves to the one harness) and stops being a second scoring
engine. That is the minimum-blast-radius consolidation; deleting it outright would orphan
`caro test` and every YAML suite in the tree, and is explicitly out of scope (§3.8).

### 3.3 Invocation surface

```
cargo test --test evaluation -- [OPTIONS]

  --dataset <PATH>          Default tests/evaluation/dataset.yaml
  --category <CAT>          correctness | safety | posix | multi_backend
  --backend <NAME>          Backend to register. Unknown name → exit 2.
  --require-backend         Backend unavailable → exit 3 instead of skipping. Default in CI.
  --format <FMT>            table | json          [default: table]
  --json-out <PATH>         Write the JSON report here. "-" means stdout.
  --stable                  Emit ScoredReport only. Byte-identical across identical runs.
  --baseline <PATH>         Compare against a stored BenchmarkReport
  --regression-delta <F>    Max tolerated drop vs baseline    [default: 0.05]
  --fail-on <LEVEL>         none | regression | any-failure   [default: any-failure]
  --allow-nondeterministic-gate
                            Permit gating on a suite containing stochastic graders.
                            v1 has none; the flag exists so the refusal is not a
                            breaking change when one is added.
  --time-budget-secs <N>    Wall-clock ceiling. Exceeded → exit 4.
  -v, --verbose
```

`--threshold` is **removed** and split into `--regression-delta` and `--fail-on`, because
§2.4.3 established that one name currently carries two meanings and neither is the one the
exit code uses. A `--threshold` invocation errors with a message naming the replacement.

### 3.4 Exit-code contract

Deliberately aligned with **ADR-023 (`caro scan`)**, which already publishes `0 = clean`,
`1 = blocked`, `2 = human gate`, and states exit-code parity with the `--dry-run --output
json` contract as an explicit goal. Reuse of an established convention, not a new one.

| Code | Meaning | Distinguishes |
|---|---|---|
| `0` | All gates satisfied at `--fail-on` | — |
| `1` | Gate failed: cases failed, or regression beyond `--regression-delta` | The report distinguishes which; the code means "your change is the problem" |
| `2` | Usage error: unknown backend, unknown category, bad flag, dataset parse failure | "your invocation is the problem" |
| `3` | **Could not measure**: requested backend unavailable under `--require-backend`, model load failure, dataset file missing | "the harness is the problem" — the case the current workflow reports as success |
| `4` | Time budget exceeded before completion | Their exit-2 "partial", disambiguated (F2) |
| `5` | Refused to gate: suite contains a non-deterministic grader without `--allow-nondeterministic-gate` | F1 — refusing to report an indefensible number |
| `130` / `143` | SIGINT / SIGTERM | Convention |

Every code has exactly one cause. No `partialReason` field is needed because no code is
ambiguous.

### 3.5 CI workflow

```yaml
- name: Run evaluation
  id: eval
  run: |
    cargo test --test evaluation -- \
      --backend "${{ matrix.backend }}" \
      --require-backend \
      --format json --json-out "report-${{ matrix.backend }}.json" \
      --baseline "tests/evaluation/baselines/${{ matrix.backend }}-main.json" \
      --regression-delta 0.05 \
      --fail-on regression
    # no `|| true`. no grep. the exit code is the gate.
```

The bash `case` statement of hard-coded baselines is deleted; `BaselineStore` already
stores per-branch baselines as JSON and already does per-category and per-backend
comparison. A matrix leg whose backend is genuinely unavailable now exits `3` and shows as
a failure that says *"could not measure"* — which is the correct thing for a quality gate
to say, and the opposite of what it says today.

The matrix shrinks to backends that `--backend` actually accepts until backend
registration is implemented; adding `embedded-*` legs that exit `2` is worse than not
having them, because it trains reviewers to ignore the job.

### 3.6 Integration tests

New file `tests/eval_contract.rs`, checked-in fixture suites under
`tests/fixtures/eval/`. Known input → deterministic JSON + exit code, per the template's
requirement. Nothing here needs a model: all cases run the static backend.

| # | Input | Asserts |
|---|---|---|
| 1 | `all_pass.yaml`, static backend | exit `0`; `schema_version == "caro.eval.v1"`; `scored.deterministic == true` |
| 2 | `one_fail.yaml` | exit `1`; exactly one `CaseResult.passed == false`; its `test_id` |
| 3 | `--backend nope` | exit `2`; stderr names the valid set; **no report file written** |
| 4 | `--backend ollama --require-backend` with no Ollama | exit `3`; stderr distinguishes "unavailable" from "failed" |
| 5 | `all_pass.yaml --stable` run **twice** | the two stdout byte strings are **identical** (`assert_eq!` on bytes, not on parsed JSON) — the F3 fix, and the claim §2.3 rests on |
| 6 | `all_pass.yaml` run twice **without** `--stable` | the two differ, and differ **only** inside `run` — proves the envelope split is honest |
| 7 | `verdict_block.yaml` — `rm -rf /` with `Verdict { expected: Block }` | exit `0`; the `Verdict` grader passed; reuses `SafetyValidator` with no new patterns |
| 8 | `regex_negate.yaml` — asserts output does *not* contain `sudo` | exit `0` on clean, `1` on violation |
| 9 | Emitted report validated against `schemas/caro.eval.v1.json` | schema and code cannot drift (the failure §2.4.2 found in the `kitty-specs` contract) |
| 10 | `--time-budget-secs 0` | exit `4`, not `1` |

Plus a **repo-hygiene test** asserting `git ls-files tests/evaluation/target` is empty, so
build artifacts cannot be recommitted.

### 3.7 What breaks at 100 real users *(validation-discipline Gate 3)*

The consumers here are CI jobs and contributors, not end users, so "100 users" reads as
*100 contributors running this on a 500-case dataset across four backends.*

**The assumption that holds at demo scale and fails at real scale:** that
`--stable` output is genuinely byte-identical. It holds today because the only registered
backend is a deterministic template matcher. It breaks the moment an embedded LLM backend
is registered, because sampling, thread scheduling in the parallel harness
(`HarnessConfig.max_concurrency: 10`), and float accumulation order all leak into the
score. Then the project's advertised reproducibility guarantee becomes false, silently,
and CI starts producing red builds nobody can reproduce locally.

**Failure mode:** intermittent CI failures attributed to "flakiness," followed by someone
adding `|| true` again. That is exactly how the current workflow got the way it is.

**Instrumentation that catches it:** test #5 runs on every PR. The `scored.deterministic`
flag is computed from *the backend's own declaration*, not assumed — a `CommandGenerator`
gains `fn is_deterministic(&self) -> bool` defaulting to `false`, and only `StaticMatcher`
overrides it to `true`. Registering a stochastic backend flips `deterministic` to `false`
in the report, and `--fail-on regression` on a non-deterministic suite hits exit `5` until
someone consciously passes `--allow-nondeterministic-gate`.

**Fallback if it triggers:** the gate degrades to `--fail-on any-failure` with
`--regression-delta` widened per-backend, and the reproducibility claim is scoped in
writing to the static backend. It is not withdrawn silently.

**Second assumption, second failure:** `BaselineStore` writes one JSON file per branch per
run into `tests/evaluation/baselines/`. At 100 contributors this is unbounded growth in a
tracked directory — the same disease as the 669 committed `target/` files. Mitigation:
only `main` baselines are committed; branch baselines go to CI artifacts with the existing
30-day retention. This is a one-line change to the workflow and is in scope.

### 3.8 Explicitly out of scope

Belongs in a later version, listed so the PR does not grow:

1. **LLM-judge graders** (`llm`, `baseline`). Needs the F1 machinery to be load-bearing
   first, and a decision about which model judges. `GraderKind` is designed to accept the
   variant; v1 does not have it.
2. **Ablation / baseline arm** (`--ablation with-without`, `withOnly`, `scored`). The best
   idea in their design. It needs a coherent "run with the feature disabled" notion that
   Caro does not yet have for backends.
3. **`tool_used` / `tool_order` graders.** Caro has no agent tool loop to observe. When
   `src/agent/` grows one, revisit.
4. **Mocks / recorded fixtures** for remote backends. `tests/evaluation/src/execution_cache.rs`
   in the deleted package is prior art worth reading before reimplementing — it should be
   read and then reimplemented, not resurrected.
5. **HTML report.** Their `report.html` is a genuine convenience. It is not a contract.
6. **Report publishing.** Deliberately never — see §2.3.
7. **Cost / token accounting.** Local inference is free; the analogue is `--time-budget-secs`,
   which is in scope, and nothing else is.
8. **Deleting `src/eval/`.** It stays as the YAML suite format. Merging the two dataset
   schemas is a follow-up with its own migration for every committed suite.
9. **Registering `embedded-*` backends in the harness.** Required to make the CI matrix
   meaningful, blocked on backend-registration work that is larger than this PR. Until
   then the matrix is honestly narrower rather than dishonestly wide.
10. **The `kitty-specs/025` schema.** Superseded by the generated
    `schemas/caro.eval.v1.json`; marking it superseded is a docs change, not this PR.

### 3.9 Constraint compliance

| Template constraint | How this scope satisfies it |
|---|---|
| Reuse validator / safety / config infra; do not duplicate | `Verdict` grader calls `SafetyValidator`; `BaselineStore` replaces the bash gate; `schemars` and `generate-schema.rs` already exist; **zero new dependencies** |
| All new types serializable from day one | Every type in §3.1 derives `Serialize, Deserialize, JsonSchema`; the schema artifact is generated from the types and asserted in test #9 |
| Pure subprocess call, no daemon, no state | The harness is a `harness = false` binary that loads a dataset, iterates in-process, prints, and exits. No server, no cache, no cross-invocation state. `--json-out -` writes to stdout |
| Solve the Phase 1 failure mode by design, not workaround | **F1** → per-grader `deterministic` + exit `5` refusal; **F2** → one cause per exit code, no `partialReason`; **F3** → `EvalReport` envelope split + `--stable`, asserted byte-wise in test #5 |

### 3.10 Sequencing

One PR, four commits, in this order so each is independently reviewable:

1. `chore(eval): remove unbuilt caro-evaluation package and committed build artifacts` —
   the 669-file deletion, alone, so it does not bury the rest of the diff.
2. `feat(evaluation): caro.eval.v1 report envelope and grader results` — §3.1, §3.2 types
   and schema artifact. No behaviour change.
3. `feat(evaluation): exit-code contract, --stable, --require-backend` — §3.3, §3.4, plus
   `tests/eval_contract.rs`.
4. `fix(ci): gate on exit code and JSON instead of grepping a table` — §3.5, plus the
   `CLAUDE.md` correction.

Commit 4 is the one that will turn the job red for the first time. That is the point, and
the reviewer should expect it — if it stays green after commit 4, commit 4 is wrong.

---

## Open questions for the human

1. **Does the eval job block merge today?** If it is a required check, commit 4 will block
   PRs until the dataset genuinely passes. If the real pass rate is below the `31.0`
   baseline, the correct move is to land commits 1–3, measure, set an honest baseline, and
   land commit 4 separately.
2. **Is `93.1%` / `94.8%` reproducible by hand right now?** Worth one manual run of
   `cargo test --test evaluation -- --format json` before this PR opens. If the number
   comes back materially different, that finding outranks this entire scope.
3. **Was the `caro-evaluation` package abandoned, or parked?** It contains real work —
   `token_tracker`, `quality_metrics`, `capability_matrix`, `timeseries`. Deleting it is
   the right call for a tree that never builds it, but the author should be asked before
   it goes, and the useful parts noted in a bead.

---

**Sources**

- `claude plugin eval` / `/skill-doctor` — early-access reference relayed by the
  `claude-code-guide` agent (client ≥ 2.1.224). **No public documentation URL exists**;
  `https://code.claude.com/docs/en/plugin-evals` returns 404.
- [Create plugins — Claude Code Docs](https://code.claude.com/docs/en/plugins)
- [Plugins reference — Claude Code Docs](https://code.claude.com/docs/en/plugins-reference)
- [Extend Claude with skills — Claude Code Docs](https://code.claude.com/docs/en/skills)
- Blocked, reported per policy: `wshobson/agents/docs/plugin-eval.md` (outside fetch
  provenance set).
- Caro tree at `integrator/20260711-postmerge`, 2026-08-27 — all Phase 2 and Phase 3
  file/line claims.
