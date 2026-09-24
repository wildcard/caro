# ADR-060: `caro.eval.v1` — Evaluation Report Contract, Exit-Code Gate, and Harness Consolidation

- **Status**: Proposed (implementation ADR — net-negative LOC; no new module, no new dependency)
- **Date**: 2026-08-27
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **`claude plugin eval`** — Claude Code's early-access plugin
  evaluation suite: isolated per-run `claude -p` sessions, a typed grader taxonomy,
  `aggregate-result.json` at `schemaVersion: "1"`, `--threshold` CI gate, five-value exit
  codes. **No public documentation URL exists**; source is the early-access reference
  relayed by the `claude-code-guide` agent (client ≥ 2.1.224).
- **Supersedes in practice**: the schema at
  `kitty-specs/025-llm-evaluation-harness/contracts/evaluation_result_schema.json`
- **Depends on**: nothing
- **Reuses**: `src/safety/` (`SafetyValidator`), `src/evaluation/baseline.rs`
  (`BaselineStore`), `schemars` 0.8 + `src/bin/generate-schema.rs`, ADR-023's exit-code
  convention
- **Full scope document**: `caro-scope-eval-contract-2026-08-27.md`

> **Relationship to the ADR-059 moratorium.** ADR-059 declared itself "the last ADR in
> this space until `src/safety/assessment.rs` merges." That file still does not exist and
> the moratorium holds — it governs the assessment/governance surface, ADR-035 → ADR-059.
> This ADR is outside it: no policy, no verdict payload, no approval exchange, no lifecycle
> event. It is test infrastructure, and its headline change is a **deletion of 669 tracked
> files and one unbuildable cargo package**, which is the shape the 2026-08-26 strategy
> memo asked for when it named the ADR-backlog-to-`src/`-tree gap as the project's largest
> risk.
>
> **Validation-discipline note.** Gate 3 (demoware trap) is written — scope §3.7. Gate 4
> (`devils-advocate` review) is **required before the implementation PR opens** and is not
> discharged here. Gates 1, 2 and 5 do not attach; this is test infrastructure for an
> already-shipped capability, which the rule explicitly carves out.

---

## Context

Caro's README, `CLAUDE.md` and ROADMAP assert quality numbers — *93.1% pass rate*, *94.8%
CSR*. `.github/workflows/evaluation.yml` runs on every PR to `main` and is the mechanism
that is supposed to defend them.

**It cannot fail.** Verified against the tree on 2026-08-27, five independently sufficient
reasons:

1. `cargo test --test evaluation … || true` — the harness computes an exit code at
   `tests/evaluation/main.rs:230–235` and calls `process::exit`; the workflow discards it.
2. The gate then greps the log for `"Passed:"`. The scored output path is `output_table()`
   (`main.rs:260+`), which prints `Run ID:`, `Branch:`, `Commit:` and a box-drawn table.
   `"Passed:"` appears nowhere in it.
3. Two of the four matrix legs (`embedded-smollm`, `embedded-qwen`) fail `validate_args`
   (`main.rs:104–115`, valid set `static_matcher | mlx | ollama | vllm`) and
   `process::exit(2)` — swallowed by `|| true`.
4. When the grep misses, the threshold step is skipped entirely
   (`if: backend_available == 'true'`). **Failure to measure is encoded as success.**
5. Three of four baselines are hard-coded `0.0` in a bash `case` statement with an explicit
   `if BASELINE == 0.0 → exit 0`.

And even with all five fixed, `--backend` is accepted, validated, then ignored —
`"Warning: Backend filtering is not yet implemented. Running all backends."`
(`main.rs:167–171`) — where "all backends" is one hard-coded `StaticMatcher`. The four-way
matrix runs the same static matcher four times.

The underlying reason nobody noticed: **Caro has four evaluation harnesses**, and the good
one is not the one CI consumes.

| # | Location | Reachable from |
|---|---|---|
| 1 | `src/eval/mod.rs` — `EvalSuite`/`EvalResults`, human `print_summary()` only | `caro test` |
| 2 | `src/evaluation/` — `BenchmarkReport`, `BaselineStore`, trait evaluators, parallel harness | `tests/evaluation/main.rs` only; **zero call sites in `src/`** |
| 3 | `tests/evaluation/src/` — separate cargo package `caro-evaluation`, 23 files, own `Cargo.lock` | **nothing** — root `Cargo.toml` has no `[workspace]` |
| 4 | `tests/evaluation.rs` + `tests/evaluation/{dataset,harness,validators,reporter}.rs` | itself |

`git ls-files tests/evaluation` → **732 tracked files, 669 of them under
`tests/evaluation/target/`**. `CLAUDE.md` documents `cargo run --bin caro-eval`; no such
binary exists.

Reading `claude plugin eval` supplied the missing shape. Three of its properties are worth
copying — a **versioned report** (`schemaVersion: "1"`, additive-only), a **typed grader
taxonomy** with a separate `target` selector, and **published per-cause exit codes**. Three
of its failure modes are worth designing against:

- **F1** — `--threshold` defaults to `1.0` (all cases must pass) while the `llm` grader is
  a **2-of-3 vote** of a judge model, "noisy on long inputs." A red build means either
  regression or judge noise, and the report carries no field that distinguishes them.
- **F2** — exit code `2` means both "cost ceiling hit mid-run" and "credentials rejected,"
  disambiguated only by a `partialReason` field *inside the JSON*. A CI step branching on
  `$?` cannot tell them apart.
- **F3** — the report lands at `results/<timestamp>/aggregate-result.json` and contains
  `duration_ms` and token counts, so two runs over an identical suite never produce
  identical bytes. There is no scored-content-only projection.

Caro can beat F3 outright, and this is on-thesis rather than incidental: `StaticMatcher` is
a deterministic template matcher, so **one run is the correct number of runs and two runs
produce the same bytes.** A project whose pitch is "deterministic floor beneath a
probabilistic gate" should not have a non-reproducible quality metric.

---

## Decision

Ship **`caro.eval.v1`** in one PR: a versioned, stably-serializable evaluation report and a
per-cause exit-code contract, emitted by a single consolidated harness, consumed by a CI
workflow that parses JSON.

### D1 — One scoring engine: `src/evaluation/`

`src/evaluation/` survives; it already has `BenchmarkReport`, `BaselineStore` with
per-category and per-backend regression comparison, and trait-based evaluators.

`src/eval/` is demoted from a scoring engine to a **dataset front-end** — it keeps the YAML
`EvalSuite` format and the 12-category taxonomy (including `DangerousCommands`) and loses
`print_summary()`. `caro test` re-points at `src/evaluation`. Deleting `src/eval/` outright
would orphan `caro test` and every committed YAML suite; that migration is out of scope.

The `caro-evaluation` package (`tests/evaluation/src/`, `Cargo.toml`, `Cargo.lock`,
`target/`, `run_eval*.{sh,py}`) is **removed** — 669 committed build artifacts plus a
package the build never touches.

### D2 — `EvalReport` is a two-part envelope: `run` (volatile) + `scored` (stable)

```rust
pub struct EvalReport {
    pub schema_version: String,   // "caro.eval.v1", first field, additive-only
    pub run: RunMetadata,         // run_id, timestamp, branch, commit_sha,
                                  // caro_version, execution_time_ms, host_platform
    pub scored: ScoredReport,     // dataset_digest, backend, deterministic,
                                  // rates, BTreeMaps, Vec<CaseResult> sorted by test_id
}
```

`--stable` emits `scored` alone. Every map is a key-sorted `BTreeMap` (currently `HashMap`,
whose serialization order is non-deterministic *within a single process*); every `Vec` is
sorted by `test_id`. `scored.dataset_digest` is a BLAKE3 of the dataset file, so a score is
inseparable from the input that produced it.

This is the **F3 fix**, and it is asserted byte-wise in the test plan rather than assumed:
two `--stable` runs must be `assert_eq!` on bytes, and two non-`--stable` runs must differ
**only inside `run`**.

### D3 — Typed graders, deterministic-only in v1

```rust
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GraderKind {
    ExactAnyOf { expected: Vec<String> },   // today's behaviour
    Regex { pattern: String, negate: bool },
    Verdict { expected: SuggestedRouting }, // calls SafetyValidator — no new logic
    ExitStatus { expected: i32 },
}
```

`Verdict` is the Caro-specific one and the reason the taxonomy earns its keep: Caro is the
only tool in this comparison where *"the correct outcome is that this command is refused"*
is a first-class expectation. `EvalCategory::DangerousCommands` already exists and today
has nothing to assert against but the literal text of the generated command.

A test case with no `graders:` key gets one `ExactAnyOf` grader, so **every committed
dataset entry keeps working unchanged**.

There is deliberately **no `LlmJudge` variant to accidentally use.** Adding one is a schema
change and its own ADR.

### D4 — Determinism is a reported field, and a non-deterministic suite refuses to gate

Every `GraderResult` carries `deterministic: bool`. `CommandGenerator` gains
`fn is_deterministic(&self) -> bool` defaulting to `false`, overridden to `true` only by
`StaticMatcher`. `ScoredReport.deterministic` is the conjunction.

If any grader or the backend is non-deterministic and the operator asked for a gate, the
harness **exits 5 and refuses**, unless `--allow-nondeterministic-gate` is passed. The flag
exists in v1 — which has no stochastic graders — precisely so that adding one later is not
a breaking change to the contract.

This is the **F1 fix**: rather than printing a pass rate whose variance it cannot defend,
the harness declines.

### D5 — One cause per exit code, aligned with ADR-023

| Code | Meaning |
|---|---|
| `0` | All gates satisfied at `--fail-on` |
| `1` | Gate failed — cases failed, or regression beyond `--regression-delta` ("your change") |
| `2` | Usage error — unknown backend/category/flag, dataset parse failure ("your invocation") |
| `3` | **Could not measure** — backend unavailable under `--require-backend`, model load failure, dataset missing ("the harness") |
| `4` | `--time-budget-secs` exceeded |
| `5` | Refused to gate on a non-deterministic suite (D4) |
| `130` / `143` | SIGINT / SIGTERM |

Exit `3` is the **F2 fix and the local fix simultaneously**: the case that today is
reported as success is now a distinct, loud failure that says *could not measure*. No
`partialReason` field is required, because no code is ambiguous.

ADR-023 already publishes `0/1/2` for `caro scan` and names exit-code parity with the
`--dry-run --output json` contract as a goal. This extends that convention rather than
inventing one.

### D6 — `--threshold` is removed and split

`--threshold` currently means a **regression delta** (default `0.05`) passed to
`BaselineStore::compare`, while `HarnessConfig.regression_threshold` (hard-coded `0.95`)
means a **pass rate**, and the actual exit code
(`regression_detected || overall_pass_rate < 1.0`) uses neither. Three thresholds, one
silently authoritative.

Replaced by `--regression-delta <f32>` and `--fail-on none|regression|any-failure`. A
`--threshold` invocation errors and names the replacement.

### D7 — The schema artifact is generated from the types

`src/bin/generate-schema.rs` gains a second `schema_for!(EvalReport)` emitting a committed
`schemas/caro.eval.v1.json`, and an integration test validates an emitted report against
it. `schemars` 0.8 is already a dependency.

This is the direct lesson of the existing
`kitty-specs/025-llm-evaluation-harness/contracts/evaluation_result_schema.json`: a
well-formed, 253-line, draft-07 contract, hand-written, now drifted — its `backend` enum is
`["mlx","vllm","ollama"]`, which excludes every backend the CI matrix names, including
`static_matcher`. A schema nobody generates from code is a schema that lies.

### D8 — CI gates on the exit code

```yaml
cargo test --test evaluation -- \
  --backend "${{ matrix.backend }}" --require-backend \
  --format json --json-out "report-${{ matrix.backend }}.json" \
  --baseline "tests/evaluation/baselines/${{ matrix.backend }}-main.json" \
  --regression-delta 0.05 --fail-on regression
# no `|| true`. no grep.
```

The hard-coded bash baselines are deleted in favour of `BaselineStore`, which already does
this correctly and is already unit-tested. The matrix shrinks to backends `--backend`
actually accepts: a leg that exits `2` on every run is worse than no leg, because it trains
reviewers to ignore the job.

---

## Consequences

### Positive

- The quality gate becomes capable of failing. Everything downstream of the 93.1% / 94.8%
  claims gets a mechanism.
- **Net-negative diff.** ~670 files removed, one package deleted, no new module, no new
  dependency, no new binary. This is the shape the strategy memo asked for.
- `--stable` gives Caro a claim its competitors structurally cannot make: **byte-reproducible
  evaluation output.** `claude plugin eval` scores a stochastic agent session and therefore
  runs three times and votes; Caro's static backend is a pure function.
- **One process, one model load, N cases** becomes a stated architectural property rather
  than an accident, so nobody later "optimizes" toward per-case subprocesses. Their runner
  pays full initialization per run × 3 runs and documents no caching at all; Caro's
  advantage here is structural, not a cache.
- `Verdict` graders let the safety corpus be an eval corpus, reusing `SafetyValidator` with
  zero new patterns.

### Negative

- **Commit 4 will turn a green job red for the first time.** If it stays green, commit 4 is
  wrong. If the eval job is a required check and the true pass rate is below the `31.0`
  baseline, merges block until an honest baseline is set — see Open Questions.
- Deleting `caro-evaluation` discards real work (`token_tracker`, `quality_metrics`,
  `capability_matrix`, `timeseries`, `execution_cache`). Mitigation: file a bead naming the
  salvageable parts before deletion, and read `execution_cache.rs` before reimplementing
  mocks later.
- `BenchmarkReport` gains an `into_report()` conversion and its two maps change type;
  stored baselines are unaffected but any out-of-tree consumer of the raw struct breaks.
  There are none in the tree.
- `--threshold` is a breaking CLI change for anyone with a local script. It errors loudly
  and names the replacement.
- The CI matrix gets narrower before it gets wider. Honest narrowness over dishonest breadth.

### Neutral

- `src/eval/` survives in reduced form. Merging the two dataset schemas is a follow-up with
  its own migration.
- Report **publishing is declined permanently**, not deferred. Their `--publish-report`
  defaults on; Caro's audience is the air-gapped and the regulated, and there is no version
  of this feature that ships.

---

## Alternatives Considered

**A1 — Just fix the workflow YAML.** Delete `|| true`, grep for the string the table
actually prints. Rejected: it leaves the gate parsing human-rendered output, which is what
broke it. It also cannot fix exit `2` on two matrix legs, the ignored `--backend`, or the
`0.0` baselines. One-line fixes to a five-way-broken gate produce a gate that is
four-ways broken and looks fixed.

**A2 — Adopt Anthropic's schema verbatim** (`schemaVersion`, `arms`, `withOnly`, `scored`).
Rejected: `arms` only means something with ablation, which is out of scope; `total_cost_usd`
is meaningless for local inference; and the shape carries F3 by construction. Copy the
*patterns* — versioning, typed graders, published exit codes — not the fields.

**A3 — Emit the report but keep the gate advisory** (warn, never fail). Rejected: this is
what exists. A gate that cannot fail is indistinguishable from no gate and is worse,
because it produces a green check that reviewers trust.

**A4 — Build a new `src/evalv2/` module and migrate later.** Rejected outright. The problem
is four harnesses; the fix is not five. This is the failure mode the strategy memo named.

**A5 — Make `deterministic` a suite-level config flag rather than a computed field.**
Rejected: an operator-asserted flag is a lie waiting to happen. Computing it from the
backend's own `is_deterministic()` and each grader's kind makes the claim mechanical.

**A6 — Keep `--threshold` and document which meaning wins.** Rejected: §2.4.3 of the scope
shows the meaning that wins is a *third*, hard-coded `< 1.0`. Documenting the current state
would document a bug.

---

## Implementation Checklist

- [ ] `chore(eval)`: remove `tests/evaluation/{src,target,Cargo.toml,Cargo.lock}`,
      `run_eval*.{sh,py}`; file a bead naming salvageable parts first
- [ ] `src/evaluation/models.rs`: `EvalReport`, `RunMetadata`, `ScoredReport`,
      `CaseResult`, `GraderResult`, `GraderKind`; `HashMap` → `BTreeMap`
- [ ] `src/evaluation/harness.rs`: populate per-case/per-grader results, `dataset_digest`,
      actually-registered backend
- [ ] `src/evaluation/evaluators/mod.rs`: return `GraderResult`
- [ ] `src/evaluation/dataset.rs`: optional `graders:`; absent ⇒ one `ExactAnyOf`
- [ ] `src/backends/mod.rs`: `CommandGenerator::is_deterministic()` default `false`;
      `StaticMatcher` overrides `true`
- [ ] `src/bin/generate-schema.rs`: emit `schemas/caro.eval.v1.json`; commit it
- [ ] `tests/evaluation/main.rs`: new flags, D5 exit codes, `--backend` honoured or exit 3,
      delete the "not yet implemented" warning
- [ ] `src/main.rs`: `caro test` → `src/evaluation`
- [ ] `tests/eval_contract.rs`: 10 cases from scope §3.6, incl. byte-equality under
      `--stable` and schema validation
- [ ] Repo-hygiene test: `git ls-files tests/evaluation/target` is empty
- [ ] `.github/workflows/evaluation.yml`: D8; branch baselines → artifacts, `main` baselines
      committed
- [ ] `CLAUDE.md`: delete `cargo run --bin caro-eval`; document real invocation
- [ ] Mark `kitty-specs/025-…/contracts/evaluation_result_schema.json` superseded
- [ ] **Gate 4**: `devils-advocate` review posted on the PR before merge

---

## Open Questions

1. **Is the eval job a required check?** If so, land commits 1–3, measure honestly, set the
   baseline, then land commit 4 separately.
2. **Is `93.1%` / `94.8%` reproducible by hand today?** One manual
   `cargo test --test evaluation -- --format json` before this PR opens. If the number
   differs materially, that finding outranks this ADR.
3. **Who owns `caro-evaluation`?** Ask before deleting 23 files of real work, even though
   the build has never touched them.

---

## References

- `caro-scope-eval-contract-2026-08-27.md` — full scope, Phase 1–3, with line-level evidence
- ADR-023 — `caro scan` exit-code contract, extended here
- ADR-009 — website claims verification; the claims this gate is supposed to defend
- `.claude/rules/validation-discipline.md` — Gate 3 discharged in scope §3.7; Gate 4 pending
- `.claude/rules/good-boy-scout.md` — cleanup scoped to what this PR touches
- `market-scans/2026-08-26-ai-agent-strategy-memo.md` §2.7 — scoping velocity decoupled
  from shipping velocity
- `claude plugin eval` — early-access reference via `claude-code-guide`; **no public
  documentation URL** (`code.claude.com/docs/en/plugin-evals` → 404)
