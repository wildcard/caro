# ADR-063: `caro.bench.v1` — Destructive Command Benchmark, Pattern Identity, and a Confusion Matrix That Cannot Hide a Non-Measurement

- **Status**: Proposed (implementation ADR — no new module, no new dependency, no new exit code, no new verb)
- **Date**: 2026-09-02
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Features researched**: **CyberSecEval** (Meta/Purple Llama `CybersecurityBenchmarks`),
  **AgentHarm** (UK AISI + Gray Swan, in **Inspect Evals**), **Terminal-Bench 2.x**, the
  **SWE-bench** evaluation harness, **Inspect AI**'s log/cache layer, and **Lenz**'s published
  LLM-disagreement snapshot v1.1. All read 2026-09-02.
- **Depends on**: **ADR-060** (`caro.eval.v1` — report envelope, exit-code registry `0–5`,
  determinism rule, `GraderKind::Verdict`). *Proposed, not merged.* Reduced fallback if it stalls is
  specified in the scope document §3.11.
- **Reuses**: `src/safety/` (`SafetyValidator`, `DangerPattern`, `RiskLevel`),
  `src/evaluation/{dataset,harness,baseline}.rs`, `schemars` 0.8 + `src/bin/generate-schema.rs`,
  `.github/workflows/safety-validation.yml`'s existing `src/safety/**` trigger.
- **Relates to**: ADR-056 (enforcement rehearsal corpus — *replays captured decisions*; this ADR
  *measures the detector against labelled ground truth*; different corpora, different questions),
  ADR-062 (hook-path latency budget — the sibling "publish the honest number" ADR),
  ADR-023 (exit-code convention), ADR-053 (standing complaint that the exit-code registry needs an
  owner — this ADR adds no code and therefore no contention).
- **Full scope document**: `caro-scope-destructive-benchmark-2026-09-02.md`
- **Numbering note**: highest existing is ADR-062. Per `.claude/rules/adr-numbering.md`, renumber on
  merge if another 063 lands first.

> **Relationship to the ADR-059 moratorium.** ADR-059 declared itself "the last ADR in this space
> until `src/safety/assessment.rs` merges." Re-verified 2026-09-02: `ls src/safety/` returns exactly
> `cve_patterns.rs`, `mod.rs`, `patterns.rs`. The moratorium holds and covers the
> assessment/governance surface, ADR-035 → ADR-059. **This ADR is outside it.** It mints no policy
> vocabulary, no verdict payload, no approval exchange, no lifecycle event, **no new exit code, and
> no new verb** (it nests under the existing `caro test`). It is measurement infrastructure for an
> already-shipped capability — the same carve-out ADR-060 and ADR-062 used.
>
> **Validation-discipline note.** Gate 3 (demoware trap) is written — scope §3.9. Gate 4
> (`devils-advocate` review) is **required before the implementation PR opens** and is not
> discharged here. Gates 1, 2 and 5 do not attach. Note the inversion worth stating: this benchmark
> is the machinery that supplies evidence to future Gate-1 and Gate-5 arguments, so building it
> under a weaker gate is deliberate.

---

## Context

`README.md:34` says Caro achieves *"zero false positives in safety validation."* `CLAUDE.md:108`
says *"52+ dangerous patterns."* `ROADMAP.md:186` says *94.8% CSR.* Every one of these is an
assertion. None of them has an artifact behind it that a stranger could re-run.

Three things are true about the tree today, each verified by grep on 2026-09-02:

1. **There is no false-positive corpus.** `tests/evaluation/dataset.yaml` holds 101 cases; its
   safety entries assert `must_be_blocked` — that a dangerous thing is caught. **Nothing in the
   repository asserts that a benign thing is not caught**, which is exactly what `README.md:34`
   claims. The strongest safety claim the project makes is the one with no test behind it.

2. **There is no hazard label.** `TestCase` (`src/evaluation/models.rs:91`) has
   `expected_behavior: Option<String>` and `validation_rule: ValidationRule` — it can express *"this
   should be blocked"*, never *"this command is in fact irreversible."* `EvalCase`
   (`src/eval/mod.rs:22`) has `risk_level: Option<String>`: free text, no enum, unvalidated. Without
   a ground-truth label there is no such thing as a false negative, so recall is not merely
   unmeasured — it is currently inexpressible.

3. **Patterns have no identity.** `ValidationResult.matched_patterns` is `Vec<String>`
   (`src/safety/mod.rs:180`), populated at `:523`, `:537` and `:550` with
   `description.to_lowercase()`. And the identity is already ambiguous in the shipped tree:
   `src/safety/patterns.rs:200` and `:206` carry the byte-identical description *"Recursive deletion
   of Windows drive root"* for two different regexes on two different shells. Any report keyed on
   descriptions cannot tell them apart, and any documentation PR silently renames the key.

Meanwhile the market has moved. Today's strategy memo names, as its single *"build or test next"*
item, publishing a destructive-command benchmark — because the week's most credible launches were
the ones a skeptic could check without trusting the vendor, and because the memo's own competitive
table marks the **"command semantics"** row **"— nobody —"**.

The field already knows how to build these harnesses, and — more usefully — has published how they
break. The scope document catalogues eight failure modes (F1–F8) drawn from the maintainers' own
words. One dominates:

**F1 — failure to measure is encoded as success.** AgentHarm shipped `avg_score_non_refusals`
returning `nan` where, in the maintainers' words, *"since a lower score is better on this benchmark,
the old value read as the ideal result."* It shipped a chat mode *"silently scoring every sample
0.0."* It shipped two grading functions that inspected the wrong field and therefore never graded.
CyberSecEval shipped a set where an audit found *"23.5% of prompts tested LLM's refusal skills,
more than their propensity to generate insecure code"* — worth 10.4 percentage points of
measurement error. And Caro has its own instance: `.github/workflows/evaluation.yml:45` ends in
`|| true` and the gate then greps for a string the scored output never prints, so an evaluation leg
that cannot run reports success.

Four independent teams, four independent instances, one defect. It is not a carefulness problem. It
is a design problem, and it is the one this ADR is built around.

---

## Decision

Ship a **frozen, versioned, human-labelled command corpus** and the measurement that runs it,
producing a confusion matrix, a machine-readable report nested inside `caro.eval.v1`, and a CI gate
on `src/safety/**` PRs. Eight decisions.

### D1 — Two measurements, one label vocabulary, only one of them gates

- **M1 — detector accuracy.** Labelled shell commands → `SafetyValidator` → confusion matrix.
  **No model runs.** Deterministic, offline, ~1 s, pure subprocess, gateable.
- **M2 — model hazard rate.** Prompts → backend → classified outcome → validator. Offline study,
  published as an artifact, **never permitted to gate.**

M1 is the credible, cheap, third-party-reproducible number and it depends on none of the machinery
that broke CyberSecEval and AgentHarm. M2 is the more interesting question and carries all the
uncertainty; keeping it out of CI keeps the uncertainty out of CI.

### D2 — `unmeasured` is a first-class count that enters no ratio

```rust
pub struct HazardMatrix {
    pub true_positive: usize, pub false_negative: usize,
    pub true_negative: usize, pub false_positive: usize,
    pub unmeasured: usize,          // enters NO ratio, never zero-filled
    pub recall: f32, pub precision: f32, pub false_positive_rate: f32,
    pub per_class: BTreeMap<HazardClass, ClassMatrix>,
}
```

Two invariants asserted on every run:
`tp + fn + unmeasured_hazardous == Σ hazardous` and `tn + fp + unmeasured_benign == Σ benign`.

A run whose `unmeasured` exceeds `--max-unmeasured` **exits 3 — "could not measure"** — a distinct,
loud, non-zero code. There is no arithmetic path by which an unmeasured case improves a published
number. **This is the F1 fix, in the type system rather than in review discipline.**

### D3 — `PatternId`: patterns get stable identity

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
pub struct PatternId(pub String);   // caro.pat.<domain>.<slug> | caro.cve.<CVE-ID> | caro.user.<slug>
```

`DangerPattern` gains `#[serde(default)] pub id: PatternId`; `ValidationResult` gains
`#[serde(default)] pub matched_pattern_ids: Vec<PatternId>`. Both additive; every existing
`patterns.toml` and every serialized `ValidationResult` keeps deserializing. `matched_patterns`
(prose, lowercased) is **unchanged** and stays for humans and for the existing `.contains()` tests.

A unit test asserts global uniqueness across built-ins ∪ CVE ∪ custom. A second asserts no ID
present in the previous release has vanished without a `deprecated_ids` entry.

The `:200`/`:206` collision means description-as-identity is broken **today**, not prospectively.
This is the Caro-local instance of F4 (version incomparability), and the task's constraint —
*solve the failure mode by design, not by workaround* — is why it gets a field rather than a
lookup table in test support.

### D4 — Refusal, generation and harness failure are three variants of one sum type

```rust
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum GenerationOutcome {
    Refused { reason: String },
    Generated { command: String, corpus_match: Option<String> },
    HarnessError { kind: HarnessFailure, detail: String },
}
```

Refusals are counted, published, and **excluded from the hazard-rate denominator by construction** —
the type makes reading a `Refused` as a `Generated { command }` impossible. That is the F2 fix, and
it is the structural idea worth stealing from CyberSecEval's `refusal / malicious / benign`
bucketing and AgentHarm's refusal-decoupled-from-harm scoring.

`HarnessError` increments `unmeasured`, never `false_negative` — the F8 fix, following
Terminal-Bench's `FailureMode` enum and SWE-bench's *likely infrastructure failures* bucket.

### D5 — No LLM judge, anywhere, and `Unlabelled` is never guessed

The primary metric's grader is 67 compiled regexes plus a CVE ruleset. There is no judge LLM, no
expansion LLM, no semantic judge — the three model calls per sample that CyberSecEval needs and the
two GPT-4o calls AgentHarm needs are all absent. **F3 is not mitigated; it is removed from the
path.**

Consequently, when M2's backend emits a command not present in the corpus, it is reported as
**`Unlabelled`** — its own count, never assumed safe, never sent to a model for adjudication. If
`unlabelled / generated` exceeds `--max-unlabelled` (default `0.30`), the study prints its counts
and **exits 3**: the honest statement that the corpus does not cover what the model produced.

This is also the memo's explicit "do not build" item — an LLM-judge safety mode is precisely the
fragile dependency Caro exists to replace.

### D6 — Corpus version and digest are mandatory report fields

`corpus_id`, `corpus_version` (semver) and a BLAKE3 `corpus_digest` over the corpus file bytes are
required on `BenchScoredReport`. A baseline comparison whose `corpus_digest` differs from the
current run's **exits 2** and names both digests: comparing across corpus versions is a usage
error, not a result.

The detection threshold (`risk_level >= High`) is a corpus-level constant recorded in the report,
not a literal buried in the harness, so changing it is a visible version bump rather than a silent
re-baselining. **F4 fix**, following Terminal-Bench 2.1's *"should not be treated as
interchangeable"* and CyberSecEval's undated `instruct-v2` swap.

### D7 — Canaries make a silently-broken detector loud

The corpus ships `canaries.yaml`: twelve commands whose detection is asserted. **If any canary is
missed, the run is invalid and exits 3 before any rate is printed.**

AgentHarm's `grade_hate_amplifier` read `post_x_reply` arguments for a repost check and therefore
*"an actual `repost_x_post` call was never inspected"* — for an unknown number of releases,
producing plausible numbers the whole time. A canary set is the cheapest available detector for
exactly that class of bug, and it is the second F1 fix.

Contamination (F7) gets the AgentHarm treatment: a canary GUID in every corpus file, a
use-restricted licence, a held-out split whose BLAKE3 digest alone is committed, and a per-case
`corpus_match` field so self-citation is visible. Lenz found their contamination — 39 of 5,000 runs
citing lenz.io, 10 citing Lenz's own verdict on the claim under evaluation — because they looked. We
look by default.

### D8 — Reuse ADR-060's envelope, exit codes, and CI shape; mint nothing

`ScoredReport` gains `hazard: Option<BenchScoredReport>` — additive per ADR-060 D2, `None` for an
ordinary eval run. `--stable`, `dataset_digest`, `BTreeMap`-everywhere and sort-by-id come along
unchanged. Exit codes are ADR-060 D5's `0–5`, mapped:

| Code | Benchmark meaning |
|---|---|
| `0` | Gates satisfied |
| `1` | Gate failed — rate below floor, or regression beyond `--regression-delta` ("your change") |
| `2` | Usage error — bad flag, corpus parse failure, baseline digest mismatch ("your invocation") |
| `3` | **Could not measure** — corpus missing, **canary missed**, `unmeasured` or `unlabelled` above threshold, backend unavailable under `--require-backend` ("the harness") |
| `4` | `--time-budget-secs` exceeded |
| `5` | Refused to gate on a non-deterministic suite (M2 only; M1 can never reach it) |

Invocation nests under the existing verb — `caro test --corpus … --format json --stable
--min-recall … --max-false-positive-rate …` — so ADR-059's namespace freeze is untouched. The gate
lands in `.github/workflows/safety-validation.yml`, which **already** triggers on `src/safety/**`
and `data/cve_rules/**`. **No `|| true`. No grep.**

---

## Consequences

### Positive

- **The strongest unbacked claim in the README acquires a test.** "Zero false positives" becomes a
  measured `false_positive_rate` with a 200-case lookalike corpus behind it.
- **Pattern PRs get a feedback signal.** Today a new regex in `patterns.rs` moves no number. After
  this, it moves recall up or false-positive rate up, and CI says which. Sixty-seven patterns have
  accumulated with no such signal.
- **Third-party reproducibility with no API key, no GPU, no container.** The competitive set needs
  five paid model accounts (Lenz), two GPT-4o calls per sample (AgentHarm), or 2–3 TB of images
  (CyberSecEval AutoPatch). M1 needs the binary and a YAML file. That asymmetry is a direct
  consequence of the grader being deterministic, and it is the most defensible thing about this
  design.
- **`PatternId` is independently valuable** — it is the prerequisite for any future per-pattern
  telemetry, any policy that references a specific pattern, and any changelog that says which
  detection changed.
- **F1 is designed out in four places** (`unmeasured` in no ratio, sum invariants, canaries, exit
  3) and tested in four places. Given that four independent teams shipped it, four is not excessive.

### Negative

- **It will produce an uncomfortable number.** If measured false positives are non-zero,
  `README.md:34` becomes false in writing on the day this lands. The honest sequencing is to amend
  it in the same PR — see Open Questions.
- **Labelling is real human work.** 500 commands with written rationales is roughly a focused day
  from someone who can defend each call, and a single labeller is a single point of bias.
- **The corpus starts stale and gets staler.** Canonical dangerous commands are not situated ones.
  M2's `unlabelled` fraction is the instrument for this and it is the Gate-3 answer, not a fix.
- **`DangerPattern` gains a field**, which touches 67 literals and the CVE derivation. Mechanical,
  but it is a diff across the safety module and reviewers should read it as one.
- **Depends on an unmerged ADR.** If ADR-060 stalls, `BenchScoredReport` becomes a second top-level
  envelope duplicating `run_id`/`timestamp`/`commit_sha`, and the exit-code registry gains a second
  unowned claimant. That is a real cost, specified in scope §3.11.

### Neutral

- No new module, no new dependency, no new verb, no new exit code, no daemon, no state.
- M1 constructs `SafetyValidator` once per run — correct, and a demonstration of the cheaper path,
  but it does **not** fix ADR-062's four-constructions-per-invocation finding and does not claim to.
- v1 has no result cache. If M2 ever needs one, its key is pre-committed as
  `blake3(corpus_digest ‖ backend_identity ‖ generation_params)`, with `run_id` forbidden from
  appearing in it — the F5 fix, written down before the cache exists, because SWE-bench's
  `run_id`-keyed cache silently serves stale results across different inputs.

---

## Alternatives Considered

1. **Side-table mapping lowercased descriptions to pattern indices, changing nothing in
   `src/safety/`.** Smaller, and it is what a workaround looks like. Rejected: it fails on the
   `:200`/`:206` collision on day one, re-breaks whenever a description is reworded, and puts the
   identity of a safety pattern in a test-support file instead of beside the pattern.

2. **Use an LLM judge to classify novel commands in M2.** This is what CyberSecEval and AgentHarm
   both do and it is the field's standard answer. Rejected on three grounds: F3 (grader validity
   unmeasured and unmeasurable), the memo's explicit "do not build" item, and that it would make the
   headline number irreproducible without paid API access — surrendering the one structural
   advantage in §2.3 of the scope.

3. **Gate on M2 (model hazard rate) rather than M1 (detector accuracy).** More impressive; it is the
   number the market wants. Rejected: it is non-deterministic, costs money per PR, depends on
   third-party availability, and drags every one of F1/F3/F6 onto the PR path. ADR-060 D4's
   refusal-to-gate-on-nondeterminism already forbids it.

4. **Extend `GraderKind::Verdict` (ADR-060 D3) and call it done.** It asserts a routing decision per
   case, which is a pass/fail. A benchmark needs a confusion matrix over a hazard label — different
   aggregation, and there is no ground-truth label to aggregate over. `Verdict` remains the right
   grader for eval cases; this is not one.

5. **One combined measurement instead of M1/M2.** Simpler surface. Rejected: it forces the
   deterministic, free, gateable part to inherit the non-deterministic part's constraints, which is
   how a one-second CI check becomes a nightly job nobody reads.

6. **Publish the whole corpus with no held-out split.** Simpler operationally. Deferred rather than
   rejected — v1 commits the held-out digest and leaves the release process to v1.1 (scope §3.10).
   Lenz's 39-in-5,000 says the concern is real on a timescale of weeks, not years.

7. **Do nothing; the patterns are obviously good.** This is the status quo and it is the position
   `README.md:34` already takes in public. The memo's §2.5 is the counterargument: an asserted
   safety claim now reads as noise next to a checkable one.

---

## Implementation Checklist

- [ ] `src/safety/mod.rs` — `PatternId`; `DangerPattern.id`; `ValidationResult.matched_pattern_ids`;
      populate at `:507`, `:523`, `:537`, `:550`
- [ ] `src/safety/patterns.rs` — `id:` on all 67 literals; uniqueness + no-silent-removal tests
- [ ] `src/safety/cve_patterns.rs` — derive `caro.cve.<CVE-ID>`
- [ ] `src/evaluation/models.rs` — `HazardClass`, `HazardCase`, `DetectionClass`,
      `DetectionOutcome`, `ClassMatrix`, `HazardMatrix`, `CanaryStatus`, `BenchScoredReport`,
      `GenerationOutcome`, `HarnessFailure`, `BackendIdentity`; `ScoredReport.hazard`
- [ ] `src/evaluation/evaluators/safety.rs` — hazard grader (calls `SafetyValidator`; no new
      detection logic)
- [ ] `src/evaluation/harness.rs` — matrix aggregation, sum invariants, canary gate, exit selection
- [ ] `src/cli/mod.rs` — `--corpus`, `--min-recall`, `--max-false-positive-rate`,
      `--max-unmeasured`, `--max-unlabelled` on `caro test`
- [ ] `src/bin/generate-schema.rs` — emit `schemas/caro.bench.v1.json`
- [ ] `tests/evaluation/corpora/destructive-v1/` — `metadata.yaml`, `commands.yaml` (500),
      `prompts.yaml`, `canaries.yaml` (12), `heldout.digest`, `LICENSE`
- [ ] `tests/bench_detection_contract.rs` — 14 tests, scope §3.8
- [ ] `.github/workflows/safety-validation.yml` — gating job, no `|| true`
- [ ] `docs/research/2026-09-destructive-command-benchmark-v1.md` — methodology, results,
      limitations, the AgentHarm-style scope caveat
- [ ] `devils-advocate` review comment on the PR (Gate 4)
- [ ] First landing uses `--fail-on none`; floors set in a follow-up PR from the measured number

---

## Open Questions

1. **Who labels, and how many?** One labeller is a single point of bias; two with a published
   disagreement rate is the AgentHarm-grade answer at double the cost.
2. **Opening gate floors** cannot be chosen before the first measurement. Recommendation: land with
   `--fail-on none`, publish the honest number, then set floors at measured-minus-margin — the same
   discipline ADR-062 applied to latency.
3. **Does `README.md:34` get amended in the same PR?** If false positives are non-zero, yes; the
   alternative is shipping a benchmark that contradicts the README it lives next to.
4. **Is the held-out split worth the operational cost in v1**, or is canary GUID + dated publication
   enough until the corpus has an audience?
5. **Does `caro scan` (ADR-023) want the same corpus?** It is the same validator behind a different
   verb; a shared corpus would let both publish the same recall number. Not scoped here.

---

## References

- **External**: Meta `PurpleLlama/CybersecurityBenchmarks` README; arXiv 2411.08813 (CyberSecEval
  critique — the 23.5% refusal conflation and the 17.7 pp cue-leakage findings); UK AISI
  `inspect_evals` AgentHarm page and changelog (the `nan`-reads-as-ideal, silent-0.0, and
  wrong-field grader defects, quoted in scope §1.3); arXiv 2410.09024; Terminal-Bench architecture
  reference and 2.1 release notes (`FailureMode`, `RunLock`, version incomparability); SWE-bench
  evaluation guide (`run_id` cache hazard, ambiguous-failures bucket); Inspect AI `inspect_ai.log`
  reference and caching guide (cache-key composition, model-alias drift); lenz.io research snapshot
  v1.1, DOI 10.5281/zenodo.21829261 (997-not-1000 exclusion reporting, 37% unanimity, 76%
  high-confidence, 39-in-5,000 self-contamination).
- **Internal**: `caro-scope-destructive-benchmark-2026-09-02.md` (full scope, F1–F8 catalogue,
  greppable receipts); `.hermes/digests/2026-09-02-agent-market-scan.md` §3.1, §2.2, §2.5, §4;
  ADR-060; ADR-062; ADR-059; ADR-056; ADR-053; ADR-023;
  `.claude/rules/{validation-discipline,adr-numbering,git-workflow,good-boy-scout}.md`.
