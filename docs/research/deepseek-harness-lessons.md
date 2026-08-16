# DeepSeek Harness (dsh) Lessons for Caro's Agent & Eval Harness

> Research into [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness)'s
> plugin-first agent architecture, mapped against caro's `AgentLoop`, validator
> stack, and evaluation harness, with tiered adoption proposals. The forcing
> function is local: `Cargo.toml` gated the `candidate-ranking` default flip on
> "the eval harness shows ranking wins" *in v1.4.0* — we are at v1.5.0 and the
> harness cannot yet render that verdict.

**Status**: Research (proposals; not a spec). **Date**: 2026-08-16.
**Companion commit**: the Tier-1 eval repairs in this PR (§4.2, P6a).

---

## Executive Summary

- **dsh is an agent *runtime*, not an eval framework.** DeepSeek Harness
  ("Everything is a Plugin", TypeScript, built on a vendored Cordis) treats
  model adapters, the tool registry, the session log, sandboxing, and the agent
  loop itself as swappable plugins on a shared context. Its lessons for caro
  are architectural, not benchmark-shaped.
- **One dsh principle inverts for caro.** dsh's flagship idea — *no privileged
  core* — is exactly what caro must not adopt: the safety validator's
  catastrophic floor is deliberately privileged, and that privilege *is the
  product*. Every other dsh idea survives translation.
- **Three ideas are worth taking**: (1) an append-only **generation journal**
  ("model-visible means logged") that also makes the currently-unreachable SFT
  export real; (2) **capability seams** for execution and validation, in the
  trait-object style caro already uses for backends; (3) **CI doc-verification
  gates** that would have caught our own drift (a `caro-eval` binary that
  doesn't exist; a "93.1% pass rate" claim no harness output supports).
- **Caro already invented two dsh ideas locally — for CaroML only.** The
  CaroML validator chain (safety → platform → secrets → side-effects with
  per-step repair) is dsh's tool-execution pipeline; the CaroML run journal is
  dsh's session log. The proposals below mostly *generalize existing caro
  patterns* rather than import foreign ones.
- **The eval harness is the bottleneck, and it is quietly broken.** Three
  generations of eval code coexist; the CI matrix has been silently evaluating
  only `static_matcher` while appearing to cover four backends; concurrency
  config was dead; results are never persisted. Appendix A inventories 24
  verified gaps. The smallest repairs ship alongside this document.
- **Proposal count**: 3 quick wins (Tier 1), 4 medium flag-friendly builds
  (Tier 2), 2 large items that need their own specs (Tier 3).

---

## 1. deepseek-harness: Case Study

### 1.1 Overview & philosophy

DeepSeek Harness (`dsh`, npm `@deepseek-ai/dsh`) is DeepSeek's open-source
agent harness, released in developer preview and iterating rapidly. The README
reduces its philosophy to one line: **"Everything is a Plugin."** It is powered
by a vendored version of the Cordis framework, whose design is described in
*"A Programming Paradigm for Spatiotemporal Composability"*: plugins contribute
**services, typed events, and reversible effects** to a shared context, and
registrations unwind when a plugin unloads. Third-party plugins are discovered
via the `dsh-plugin` GitHub topic.

The consequence: dsh is less an application than a runtime you assemble. Model
adapters, tools, sandboxing policy, approval logic, session persistence, and
the UI layer are separate, swappable components — not a pipeline hardcoded
into one binary. There is no privileged core to patch; you extend dsh by
mounting a plugin beside the others.

### 1.2 Capability seams

dsh names a recurring pattern the **capability seam**, with three roles:

| Role | Meaning | Examples |
|------|---------|----------|
| Service Definition | the interface | `ctx.llm`, `ctx.fs`, `ctx.subprocess`, `ctx.sandbox`, `ctx.tools`, `ctx.sessions` |
| Service Provider | an implementation | `llm-deepseek`, `fs-local` |
| Consumer | code using the service | `tool-bash` |

Swapping a single provider changes the whole product: an agent moves from a
local filesystem to a remote sandbox without touching any tool implementation,
because tools consume `ctx.fs`, not `std.fs`. Sandbox backends wrap argv
before spawning; all child processes route through `ctx.subprocess`.

### 1.3 The session log: "model-visible means logged"

dsh's session layer is an **append-only `SessionEvent` log**, and the system
enforces one invariant: *everything the model sees must be reconstructible
from the log*. Model history is derived from the event stream, which is what
makes resume, fork, replay, and transcripts cheap — there is no fragile
temporary state to reconstruct, and auditing "why did the agent do that?" is a
log read, not a debugging session.

### 1.4 Turn/step loop and the typed event waterfall

Work divides into **turns** (a complete task) containing **steps** (one model
request plus the tool calls it triggers). Each boundary is a typed event:

```
turn/start → agent/pre-step → step/start → agent/request → llm/stream
          → tool execution → step/end → … → turn/end
```

Listeners can inspect, reject, or rewrite at each point; waterfall events like
`agent/pre-step` delegate by calling `next()`. The granularity matters: every
extension (approval UIs, budget caps, retry policy) attaches to a named
boundary instead of monkey-patching the loop.

### 1.5 Tool execution pipeline

Tool runs pass through `tools/pre-execute` → `tools/execute` →
`tools/post-execute`. Policy — approval prompts, sandbox wrapping, audit
records — attaches to these events **without importing the loop**. This is the
architectural trick that keeps permissions/recovery/remote-execution concerns
from tangling into the chat loop as a product grows.

### 1.6 Profiles & bundles

A **profile** is a named composition (bundles + config patches); `web` starts
the browser UI, `headless` runs one-shot tasks, both over the same `dsh-base`.
The stated principle: *deployment differences belong in composition, not code
branches*. No fork of the codebase per deployment shape.

### 1.7 CI quality gates

The monorepo gates merges with `scripts/run-gates.ts` plus lefthook pre-push
hooks, including **type-equivalence and doc-verification gates** — CI checks
that documentation matches the shipped surface, not just that code compiles.

### 1.8 Subagent orchestration

A `subagent` package delegates work to child agents driven by the default
`ReactLoopAgent`. Described here for completeness; §5.4 argues caro should not
adopt it.

---

## 2. Caro's Harness Today

"Harness" means three things in this repo. All three were mapped for this
research; the proposals target the first two.

### 2.1 The agent runtime: `AgentLoop`

`src/agent/mod.rs` (1,108 lines) is the NL→shell orchestrator:

```
generate_command (mod.rs:144)
  └─ generate_command_impl (:226)
       ├─ generate_initial (:479)          ← backend draft
       ├─ should_refine (:820)             ← confidence gate (0.8)
       ├─ refine_command (:549) / repair_command (:582)
       └─ try_advisor (:422)               ← frontier advisor, re-validated
```

Two things stand out against dsh. First, the **frontier advisor**
(mod.rs:88–96) is a well-cut seam: a second `CommandGenerator` consulted only
when the local draft's confidence is low, whose output is re-run through the
safety validator and discarded on opt-out/unavailability/unsafety — four tests
pin this. Second, everything else is **hardcoded method calls**: prompts are
built by private methods, the refine/repair/advisor order is fixed, and no
stage boundary is observable from outside. There is no record of what the
model saw or produced once the call returns.

### 2.2 The dormant pipeline

`src/agent/pipeline/` already implements a dsh-style staged loop — behind the
`candidate-ranking` feature, modeled on x-algorithm's feed pipeline:
`CandidateSource → Hydrator → Filter → LinearScorer → ArgmaxSelector`, with
`CandidateFeatures` carrying `llm_confidence`, `safety_confidence`,
`risk_level`, `platform_fit`. It is **not wired into `AgentLoop`**. The
feature's own comment in `Cargo.toml` (`[features]`) reads:

> Off by default; flip to default in v1.4.0 after the eval harness shows
> ranking wins. See PR #1108 for design and rollout phases.

We are at v1.5.0. The flip is two minor versions overdue — not because ranking
lost, but because the eval harness cannot render the verdict (§2.5).

### 2.3 Backends: the seam caro already has right

`src/backends/` is a genuine capability seam in dsh's sense:
`Arc<dyn CommandGenerator>` with providers for static matching, embedded
MLX/CPU, remote (Ollama, vLLM, Exo, Mesh, AI-Horde, Claude, OpenRouter), and a
hybrid privacy gateway (sanitize → enhance → restore → fallback). The model
catalog (`src/model_catalog.rs`) and loader (env → bundled → cache → HF
download) complete the provider story. The proposals below extend this
pattern; nothing about it needs replacing.

### 2.4 CaroML: caro's local inventions of two dsh ideas

CaroML (v1.4.0) already contains, scoped to `.caro` task files only:

- **A validator chain** — `src/caroml/validators/{safety,platform,secrets,side_effects}.rs`,
  multi-angle validation with a per-step repair loop. This *is* dsh's
  `tools/pre-execute` pipeline, born independently.
- **A run journal** — `src/caroml/history.rs`, a per-user record of runs
  feeding the A/B challenger lifecycle (`caro experiment` / `caro adopt`).
  This *is* dsh's session log, at task-file granularity.

The single biggest architectural take-away of this research: **caro does not
need to import dsh's ideas — it needs to promote its own CaroML versions of
them from the DSL layer into the main generation path** (P1, P3).

### 2.5 The evaluation harness: three generations, quietly broken

| Generation | Location | Status |
|---|---|---|
| Live | `src/evaluation/` (3,285 LOC) + `tests/evaluation/main.rs` + `dataset.yaml` (101 cases) | Active; from PR #1245 "apply Fireworks hybrid-harness learnings" |
| Legacy | `src/eval/` (483 LOC), powers `caro test` | Active in CI (`safety-validation.yml`, informational-only) and beta cycles; exact-string-containment scoring |
| Orphaned | `tests/evaluation/src/` (~6,400 LOC, 20 modules) | Not a workspace member; never compiled by anything |

Top defects (full 24-item inventory in Appendix A; items marked ✅ are fixed
in this PR's companion commit):

1. ✅ `HarnessConfig.max_concurrency` was declared and defaulted but **never
   read** — `run_all_tests` spawned one unbounded task per (case × backend).
2. ✅ The CI matrix (`.github/workflows/evaluation.yml`) sent backend names
   the runner rejects (`embedded-smollm`, `embedded-qwen` → exit 2, masked by
   `|| true`), and `--backend` doesn't filter anyway
   (`tests/evaluation/main.rs:172`) — only `static_matcher` is ever registered
   (main.rs:179–182). Three of four matrix legs evaluated nothing.
3. ✅ `benches/performance.rs` was never declared `harness = false`, so its
   `criterion_main!` never executed.
4. 25 of 101 dataset cases are MultiBackend and structurally unpassable with
   one registered backend — a large share of the 31% baseline story.
5. `BaselineStore` (store/compare/list, `src/evaluation/baseline.rs`) is
   exported but never called; `branch`/`commit_sha` are hardcoded `"unknown"`;
   no run's results are persisted anywhere (CI keeps a 30-day text log).
6. `sft_export.rs` (passing trajectories → JSONL SFT records) has **no
   caller**, while `docs/fine-tune-pipeline.md` and the ml-ds-engineer agent
   wait on exactly that data.
7. `criteria_passed`/`criteria_total` are always `0,0`, so `mean_score`
   equals `pass_rate` by construction — the Fireworks mean-score work is
   plumbed but inert.
8. Four incompatible test-case schemas across `src/evaluation` YAML,
   `src/eval` YAML, `tests/evaluation/datasets/*.json`, and
   `test_cases.toml`.

### 2.6 The automation harness (out of scope here)

`.claude/automation/` (cron registry + loop skills), `.hermes/` (strategic
agent + inter-agent protocol), `.claude/beta-testing/` (cycles/runs), and the
beads queue form a third, agent-ecosystem harness. Two drift items noticed in
passing, flagged without proposals: the automation README documents config
files that don't exist (`qa_profiles.yaml`, `idea_sources.yaml`), and
`caro-coder-loop.md` hardcodes a personal macOS path. Improving this layer is
its own effort with its own owner (Hermes protocol), deliberately not bundled
into an architecture-research PR.

---

## 3. Principle-by-Principle Gap Analysis

| # | dsh principle | Caro today | Gap or inversion | Proposal |
|---|---------------|-----------|------------------|----------|
| 1 | Everything is a plugin; no privileged core | `CommandGenerator` seam is real; safety validator is a privileged, hardwired call | **Inversion, not gap** — the catastrophic floor must stay privileged | §5.3 |
| 2 | Capability seams (llm/fs/subprocess/sandbox) | LLM seam exists; execution has no seam (`--dry-run` is a flag, not a provider) | Execution provider seam missing | P2 |
| 3 | "Model-visible means logged" session log | CaroML `history.rs` journals `.caro` runs only; `AgentLoop` generations leave no record | No generation journal → no replay, no audit, no SFT feed | P1 |
| 4 | Turn/step loop with typed stage boundaries | Fixed private-method call chain; dormant staged pipeline exists but unwired | Stages exist twice, observable zero times | P4 |
| 5 | Tool pre/post-execute policy pipeline | CaroML validator chain (4 angles + repair), scoped to CaroML | Generalize chain to every generated command | P3 |
| 6 | Profiles: composition in config, not code branches | Cargo features = compile-time composition; runtime is flag soup | No named runtime compositions | P5 |
| 7 | CI doc-verification gates | Release-alignment rule exists as a *checklist*; no grep-able gate; drift shipped (`caro-eval`, 93.1%) | Automate the checklist the repo already believes in | P7 |
| 8 | Honest signal from CI | Eval matrix legs silently no-op; dead concurrency config | Fixed in companion commit; persistence still missing | P6a/P6b |

---

## 4. Proposals (Tiered)

House adoption pattern (precedents: `candidate-ranking` / PR #1108, Fireworks
learnings / PR #1245): **research doc → feature-flagged implementation →
eval-gated default flip**. Tier 1 = small, unambiguous, no flag needed.
Tier 2 = medium, each behind a flag or config default-off. Tier 3 = large
enough to need its own spec first. Per
`.claude/rules/validation-discipline.md`, the five evidence gates bind
user-facing feature specs; of the list below only P5 crosses that line.

### 4.1 Tier 1 — quick wins

**P6a. Eval harness repairs** *(shipped in this PR's companion commit)*
- **What**: consume `max_concurrency` via a `tokio::sync::Semaphore` in
  `run_all_tests` (permit spans generate+evaluate), with a regression-guard
  test (`test_max_concurrency_bounds_in_flight_generations`); reduce the CI
  matrix to the one leg that evaluates anything (`static_matcher`) with a
  comment explaining what restores the other legs; declare
  `benches/performance.rs` as a criterion bench; fix CLAUDE.md's nonexistent
  `caro-eval` reference and stale version banner.
- **dsh principle**: honest CI signal (§1.7).
- **What would prove it wrong**: nothing plausible — these were dead configs
  and no-op legs; the guard test pins the one behavior change.

**P7. CI doc-verification gates**
- **What**: a CI step (or pre-push hook) that greps documentation claims
  against the shipped surface: binary names mentioned in docs exist as
  `[[bin]]` targets; version banners match `Cargo.toml`; quantitative claims
  ("93.1% pass rate", README.md:34 and CLAUDE.md:119) must cite a harness
  artifact or be removed. The repo already believes in checklist-as-grep —
  `.claude/rules/release-version-alignment.md` *is* this idea, run by hand at
  release time; P7 runs it on every PR.
- **Insertion**: new step in `.github/workflows/ci.yml`; a
  `scripts/check-doc-claims.sh` seeded with the two known drifts.
- **Effort**: S. **Risk**: false positives on prose — keep the pattern list
  explicit and small, grow it per incident (same policy as safety patterns).
- **What would prove it wrong**: the gate firing on legitimate prose more
  than ~once a month — then the patterns are too broad.

**P8. DeepSeek model housekeeping** *(footnote-scale)*
- **What**: caro documents DeepSeek models as candidates
  (`docs/fine-tune-pipeline.md`, ADR-006 alternative, Ollama docs page) but
  has zero code presence for them. If one is ever registered,
  `src/evaluation/pricing.rs::price_for()` silently prices it at the generic
  `$1.0/$3.0` unknown-hosted fallback (pricing.rs:109–113) — materially wrong
  for DeepSeek's list prices, corrupting the cost-per-passed-task axis the
  Fireworks work added. File a beads issue to add pricing rows + a
  `model_catalog` entry alongside any future DeepSeek backend registration.
- **Effort**: S (when it lands with a backend; not before).

### 4.2 Tier 2 — medium, flag-friendly

**P1. Generation session journal** *(highest-leverage proposal in this doc)*
- **What**: an append-only, typed event log per generation:
  `QueryReceived → ContextGathered → PromptAssembled → DraftProduced →
  ValidationVerdict* → Refined/Repaired → AdvisorConsulted/AdvisorRejected →
  FinalCommand → UserDecision → ExecutionOutcome`. Adopt dsh's invariant
  verbatim: **anything the model saw must be reconstructible from the log.**
  A closed `enum GenerationEvent` (serde-serializable), JSONL sink under the
  existing cache/config dir, off by default behind config until the privacy
  story is reviewed (queries may contain paths/secrets — the hybrid gateway's
  sanitizer sets the precedent for redaction).
- **Why it pays three times**: (1) `sft_export.rs` finally gets a caller —
  the fine-tune pipeline's dataset hook falls out of the journal for free;
  (2) bug reports/beta cycles get replayable transcripts instead of prose
  reconstruction; (3) safety audits can answer "which validator vetoed and
  why" from the record.
- **Insertion**: new `src/agent/journal.rs`; emit points already enumerated —
  `generate_command_impl` (mod.rs:226), `generate_initial` (:479),
  `refine_command` (:549), `repair_command` (:582), `try_advisor` (:422);
  extends the `src/caroml/history.rs` concept to every generation.
- **Effort**: M. **Risk**: privacy (redact before write; reuse
  `backends/hybrid/sanitizer.rs`), disk growth (size-capped rotation).
- **What would prove it wrong**: measurable generation-latency overhead
  (>1–2 ms) or the privacy review concluding queries can't be stored safely
  even redacted — then scope the journal to opt-in debug runs only.

**P3. Validator middleware chain**
- **What**: generalize CaroML's four-angle chain into the main path: an
  ordered `Vec<Arc<dyn CommandPolicy>>` run between generation and
  presentation. Each stage may **annotate** or **veto**, never un-veto —
  monotonic verdicts, with the catastrophic-floor validator pinned first and
  non-removable by construction (not registered through config at all).
  Custom TOML safety patterns and CVE rules become just more stages.
- **dsh principle**: `tools/pre-execute` policy attachment (§1.5), minus the
  open listener surface (§5.2 explains why closed and ordered).
- **Insertion**: generalize `src/caroml/validators/*` into `src/agent/`;
  call sites in `generate_command_impl` (mod.rs:226) and the advisor
  re-validation path (:422).
- **Effort**: M. **Risk**: subtle behavior drift vs. the current hardwired
  call — mitigate by pinning current verdicts with contract tests before the
  refactor (the safety suite already exists for exactly this).
- **What would prove it wrong**: if the chain's flexibility is never used —
  i.e. after two releases the registered stages are exactly the hardwired
  set — the abstraction wasn't earning its keep (ponytail-review criterion).

**P2. Execution provider seam**
- **What**: `trait ExecutionProvider { fn execute(&self, argv…) }` with
  providers `Real`, `DryRun` (today a flag), and room for `Sandboxed`
  (argv-wrapping: `bwrap`/`sandbox-exec`) — dsh's `ctx.subprocess`/
  `ctx.sandbox` translated to one trait object. The journal (P1) records
  which provider ran.
- **Effort**: M (mostly moving existing execution code behind a trait).
  **Risk**: low; behavior-preserving refactor for Real/DryRun. Sandboxed is
  future work with its own safety review.
- **What would prove it wrong**: if no third provider ever materializes,
  a two-variant trait is over-abstraction — acceptable bet given sandboxing
  is already on the safety roadmap's horizon.

**P6b. Eval consolidation**
- **What**: one harness, one schema, persisted results. Fold `src/eval/`'s
  `caro test` surface onto `src/evaluation/` (keep the CLI verb, swap the
  engine); wire `BaselineStore` into the runner with real
  branch/commit_sha; converge the four dataset schemas on the
  `src/evaluation` one (converters for the beta YAML and JSON sets);
  **delete** the orphaned `tests/evaluation/src/` sub-crate (6,400 LOC that
  has never compiled in-tree) after harvesting ideas per Appendix B.
- **Effort**: M–L. **Risk**: the legacy `caro test` YAML is CI-consumed by
  `safety-validation.yml` and the beta-cycle skill — migrate those callers in
  the same PR or keep a compatibility loader for one release.
- **What would prove it wrong**: if `caro test`'s exact-containment scoring
  turns out to be load-bearing for safety-pattern TDD ergonomics, keep that
  scorer as an evaluator *inside* the unified harness rather than a separate
  engine.

### 4.3 Tier 3 — needs its own spec

**P4. Wire the candidate pipeline as *the* loop**
- **What**: make `src/agent/pipeline/` the actual structure of
  `AgentLoop::generate_command_impl`, expressing today's flow as stages
  (initial generation and advisor as `CandidateSource`s; refinement/repair as
  re-entrant stages; validators as `Filter`s via P3; confidence gating in the
  scorer) — dsh's turn/step decomposition, in caro's existing vocabulary.
  Then run the A/B the flag was waiting for and flip `candidate-ranking` per
  its own Cargo.toml contract.
- **Blockers**: needs P6a (honest eval) and ideally P1 (journal, for
  side-by-side evidence). This is the payoff item: the overdue flip is the
  measurable definition of done.
- **Effort**: L. Needs a spec (kitty-spec) + eval evidence; not a weekend
  refactor of the safety-critical path.

**P5. Runtime profiles**
- **What**: named compositions in config — e.g. `offline` (static+embedded,
  no advisor), `privacy` (hybrid gateway mandatory, journal redaction max),
  `fast` (static+small model, low timeout), `thorough` (ranking on, advisor
  on) — selecting backend, advisor, validator stages (P3), and execution
  provider (P2). dsh's profiles/bundles, minus runtime plugin loading.
- **Why Tier 3**: this is a *user-facing capability* — per
  `.claude/rules/validation-discipline.md` it needs the evidence gates
  (20 transcripts, demoware-trap section, devil's-advocate review, defended
  cohort for any PMF claim) before an implementation PR. Do not build this
  from an architecture doc.

---

## 5. What We Should NOT Adopt

### 5.1 Runtime plugin loading (the Cordis model itself)

Rust has no stable ABI; "mount a plugin beside the others" means `dlopen`
hazards or an embedded interpreter, both hostile to the <50MB single-binary
target — and fatally, to the safety story: the catastrophic-floor allowlist is
trustworthy *because* it is compiled in and nothing can load in beside it and
shadow the validator. Caro's translation of "everything is a plugin" already
exists and is idiomatic: trait objects behind seams (`Arc<dyn
CommandGenerator>`) plus Cargo features for compile-time composition. The
principle survives as **"cut clean seams"** (P2, P3), not "make them
hot-swappable at runtime". (An AGPL plugin ecosystem would also raise
license-boundary questions caro has no reason to invite.)

### 5.2 An open event bus with listener rewrite

dsh lets listeners inspect/reject/rewrite at named events, `next()`-style. In
Rust this degenerates into `Box<dyn Any>` downcasts or serde round-trips, and
it forfeits the property that makes caro's safety code auditable: **exhaustive
match on closed enums**. Adopt the waterfall *as data* instead: the closed
`GenerationEvent` enum (P1) records every boundary; `tracing` spans give live
observability; interception happens only in the fixed-order middleware chain
(P3). Nothing outside the compiled binary can subscribe to — or rewrite — the
generation path.

### 5.3 "No privileged core", taken literally

The sharpest finding of this research: dsh's flagship principle **inverts**
for caro. dsh has no privileged core because a general agent harness must let
integrators replace anything. Caro has exactly one privileged component — the
safety validator with its catastrophic floor — and that privilege is the
product promise ("safe POSIX commands"). The design rule the proposals encode:
**middleware may add verdicts, never remove one** (monotonic veto, P3), and
the floor is pinned first and is not registered through any configurable
mechanism. Everything else — backends, advisor, ranking, execution, even the
journal — is legitimately seam-swappable.

### 5.4 Subagent orchestration

dsh ships a subagent package because general agent tasks decompose. Caro is
single-shot generation with one optional second opinion; the frontier advisor
is already correctly modeled as *just another `CommandGenerator`* whose output
re-enters validation (mod.rs:422, with opt-out and unsafe-rejection tests).
An orchestration layer has no current user story here — importing one would be
scope creep (the ponytail criterion: code the problem doesn't need).

---

## 6. Adoption Sequencing

```
P6a (shipped) ──► P6b (one harness, persisted) ──► P4 (wire pipeline, A/B, FLIP)
                                                     ▲
P1 (journal) ── feeds evidence + SFT ────────────────┘
P3 (validator chain) ── independent, feeds P4's Filter stages
P2 (execution seam) ─── independent, feeds P5
P7 (doc gates) ──────── independent, ship next
P5 (profiles) ───────── last; needs P2+P3 seams and validation-discipline gates
```

Everything routes toward one measurable outcome: **rendering the verdict the
`candidate-ranking` flag has been waiting for since v1.4.0**, on an eval
harness whose numbers can be believed.

---

## 7. Key Takeaways

1. dsh's deepest idea is not plugins — it is **"model-visible means logged"**.
   For a safety product, an append-only generation journal is audit
   infrastructure, SFT harvest, and debugging transcript in one (P1).
2. **Caro already invented dsh's two best patterns locally** (CaroML validator
   chain, CaroML run journal); the work is promotion to the main path, not
   importation.
3. **One dsh principle must be inverted**: keep the safety floor privileged;
   make everything else a seam.
4. The eval harness is the gating asset: the `candidate-ranking` flip has
   been blocked on it since v1.4.0, and until this PR its CI signal was
   partly fictional. Repair (done), persist (P6b), then decide (P4).
5. dsh validates a caro instinct: **composition in config, not code branches**
   — but caro's version is Cargo features today and, only after evidence
   gates, runtime profiles (P5).

---

## 8. Sources

- [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) — README ("Everything is a Plugin", Cordis, developer-preview status, `dsh-plugin` topic)
- [`docs/architecture.md`](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/architecture.md) — capability seams, session log, turn/step events, tool pipeline, profiles/bundles
- [DeepWiki: deepseek-harness](https://deepwiki.com/deepseek-ai/deepseek-harness) — package layout, `ReactLoopAgent`, gates scripts
- Third-party analyses: [StableLearn](https://stable-learn.com/en/deepseek-harness-open-source-agent-framework/), [AgentsPulse](https://agentspulse.github.io/tutorials/deepseek-harness-and-cordis-why-everything-is-a-plugin/)
- Caro precedents: [PR #1108](https://github.com/wildcard/caro/pull/1108) (candidate pipeline), [PR #1245](https://github.com/wildcard/caro/pull/1245) (Fireworks hybrid-harness learnings), `thoughts/shared/plans/evaluation-harness-maturity-milestone.md`
- Internal rules cited: [`validation-discipline`](../../.claude/rules/validation-discipline.md), [`feature-evidence`](../../.claude/rules/feature-evidence.md), [`release-version-alignment`](../../.claude/rules/release-version-alignment.md), [`external-sdk-integration`](../../.claude/rules/external-sdk-integration.md)

---

## Appendix A: Eval-Harness Gap Inventory (24 items, verified 2026-08-16)

Fixed in this PR's companion commit: A1–A3. File anchors are as of this
branch.

| # | Gap | Anchor |
|---|-----|--------|
| A1 ✅ | `max_concurrency` declared, never read → unbounded spawns | `src/evaluation/harness.rs` (`HarnessConfig`, `run_all_tests`) |
| A2 ✅ | CI matrix legs pass rejected/ignored backend names; masked by `\|\| true` | `.github/workflows/evaluation.yml`; `tests/evaluation/main.rs:107,172,179–182` |
| A3 ✅ | `benches/performance.rs` never declared → criterion never runs | `Cargo.toml` `[[bench]]` block |
| A4 | `--backend` filtering unimplemented; only `static_matcher` registered | `tests/evaluation/main.rs:171–182` |
| A5 | 25/101 MultiBackend cases unpassable with one backend | `tests/evaluation/main.rs:184–193` TODO |
| A6 | `BaselineStore` never called by any runner | `src/evaluation/baseline.rs` |
| A7 | `branch`/`commit_sha` hardcoded `"unknown"` → baseline files collide | `src/evaluation/harness.rs` (`aggregate_results`) |
| A8 | No result persistence; CI keeps a 30-day text log only | `tests/evaluation/results/` (`.gitkeep`) |
| A9 | No trend/time-series regression tracking; baselines hardcoded in YAML | `evaluation.yml` threshold step |
| A10 | No generation caching across runs | (orphan crate had `execution_cache.rs`) |
| A11 | One sample per (case, backend); no pass@k, no variance handling | `run_all_tests` |
| A12 | No prompt versioning in the live path | `tests/evaluation/prompts/v1.0/` unreachable |
| A13 | `criteria_passed/total` always `0,0` → mean_score ≡ pass_rate | all evaluator construction sites |
| A14 | `BackendProfile` sampling/priority never applied | `src/evaluation/models.rs:217–240` |
| A15 | Errors/panics dropped from results → denominator shrinks silently | `run_all_tests` collect loop |
| A16 | Aggregation re-runs `get_by_category` per result (O(cases×results)) | `aggregate_results` |
| A17 | Four incompatible dataset schemas, no converters | §2.5 |
| A18 | No shellcheck; hand-rolled POSIX heuristics | `src/evaluation/evaluators/utils.rs:180` |
| A19 | Cost computed but never budget-enforced; price table hardcoded | `src/evaluation/pricing.rs` |
| A20 | `sft_export` unreachable (no caller) | `src/evaluation/sft_export.rs` |
| A21 | Orphaned sub-crate: 6,400 LOC, own lockfile, checked-in `target/`, never compiled | `tests/evaluation/src/` |
| A22 | `tests/evaluation/README.md` documents commands that cannot work | not a workspace member |
| A23 | Exit code 1 on any failure contradicts the 31% baseline model | `tests/evaluation/main.rs:231` |
| A24 | Doc drift: nonexistent `caro-eval` (✅ fixed), untraceable "93.1%" (README.md:34, CLAUDE.md:119 — P7's seed case) | CLAUDE.md, README.md |

## Appendix B: Orphaned Sub-Crate — Salvage Ideas Before Deletion

`tests/evaluation/src/` should be deleted under P6b (it has never compiled
in-tree; keeping 6,400 LOC of dead reference code misleads every future
reader). Ideas worth re-implementing in `src/evaluation/` first — as designs,
not code copies:

- `timeseries.rs` / `dashboard.rs` — per-run time-series records and trend
  rendering (the missing A9).
- `execution_cache.rs` — 24h-TTL generation cache keyed on
  `(test_id, backend, prompt_version)` (the missing A10).
- `prompts/{loader,registry,metadata}.rs` — prompt versioning as a first-class
  eval dimension (the missing A12).
- `safety_validator.rs` confusion-matrix metrics (precision/recall/F1) — the
  only place CSR-style metrics were ever implemented; belongs in the
  `SafetyEvaluator`.
- `issue_automation.rs` — auto-filing regressions as issues; re-home the idea
  in the automation layer (Hermes protocol), not the harness.
