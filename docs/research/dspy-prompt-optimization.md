# DSPy → caro: Prompt Optimization Learnings

> Research doc, 2026-09-11. What [stanfordnlp/dspy](https://github.com/stanfordnlp/dspy)
> teaches about caro's prompt + eval architecture, and a phased adoption roadmap.
> DSPy facts are pinned to **DSPy 3.3.0** (docs at [dspy.ai](https://dspy.ai)) and
> will drift; caro facts are pinned to `main` at the date above.

## Executive Summary

DSPy's thesis is **"programming — not prompting — language models"**: you declare
*what* a model call must do (a typed Signature), a framework renders that into a
model-specific prompt (an Adapter), and an **Optimizer** compiles the instructions
and few-shot demos against a metric and a small dataset. The prompt becomes a
*compiled, versioned artifact* rather than hand-edited source code.

caro's core quality bottleneck is exactly the loop DSPy automates: hand-tuning
system prompts for small local models (default Qwen2.5-Coder-1.5B Q4, SmolLM-135M
in CI) by eyeballing eval failures. Exploration shows caro already owns every
ingredient DSPy needs — ~290 labelled cases, a 5k-line evaluation harness, a
chi-square A/B engine, a versioned prompt registry, and per-platform few-shot
libraries — but none of them are connected into an optimization loop, and the
artifacts that *are* structured (`src/prompts/`) are not the ones that ship.

The smoking gun: three sources of truth disagree about caro's most basic query.

| Source | Query | Answer |
|---|---|---|
| Shipped embedded prompt (`src/backends/embedded/embedded_backend.rs:221`) | "list all files in the current directory" | `ls` |
| Dead few-shot library (`src/prompts/smollm_prompt.rs:561`) | "list all files" | `ls -a` |
| Eval dataset (`tests/evaluation/dataset.yaml`, `correctness-001`) | "list all files in the current directory" | `ls -la` |

Nothing in the build detects this. A compiled prompt would have been optimized
*against* the dataset, so the prompt and the ground truth cannot silently diverge.

**Eight learnings, ranked by leverage ÷ effort:**

| # | Learning | DSPy concept | caro gap it closes | Effort | Phase |
|---|---|---|---|---|---|
| 1 | Prompts are compiled, versioned artifacts | Signatures + `program.save()` | 8 divergent inline `format!` prompts, no versioning | M | 1→3 |
| 2 | Metrics return score **and** textual feedback | GEPA metric contract | Binary string-match, coarse `failure_reason` | S | 1 |
| 3 | Offline optimizer harness | BootstrapFewShot → GEPA | Manual `prompt-tuner` loop; the unmet goal of #517 | M | 2 |
| 4 | Bounded Refine on safety failure | `dspy.Refine` / `BestOfN` | Final safety block has no regeneration hook | S–M | 1 |
| 5 | Platform few-shot demos as data | `LabeledFewShot` | Dead gnu/bsd/busybox/posix sets; macOS advice on Linux | S–M | 2 |
| 6 | One output adapter, not eight parsers | Adapters | 4-tier JSON parse ladder duplicated per backend | M | 3 |
| 7 | Honest confidence | `BestOfN` / self-consistency | `confidence_score` is a per-backend constant | S–M | 1 |
| 8 | Distill compiled prompts into weights | `BootstrapFinetune` | Existing SFT export has no prompt-side input | L | later |

---

## 1. What DSPy is (as of 2026-09, v3.3.0)

### 1.1 The four primitives

| Primitive | What it is | Why it matters for caro |
|---|---|---|
| **Signature** | Typed input/output contract: `request: str = InputField()`, `cmd: str = OutputField()`. Docstring = task description. | One declaration replaces eight hand-written prompt bodies. |
| **Module** | Execution strategy over a signature: `Predict`, `ChainOfThought`, `ReAct`, **`Refine`**, **`BestOfN`**. | `Refine`/`BestOfN` are the pattern for "retry with feedback, bounded". |
| **Adapter** | Renders a signature into a model-specific prompt and parses the response: `ChatAdapter`, `JSONAdapter`, `XMLAdapter`. | Structured output is a framework concern, not something each backend begs for in prose. |
| **Optimizer** | Compiles instructions + demos against `(program, metric, trainset)`. | The part caro does by hand today. |

`dspy.Evaluate` runs the same metric the optimizer uses. **The optimizer and the
regression gate share one metric** — that is the discipline, not the library.

### 1.2 The optimizers

| Optimizer | Optimizes | Data needed | Notes |
|---|---|---|---|
| `LabeledFewShot` | picks k demos from labelled data | ~10 | Zero model calls. |
| `BootstrapFewShot` | mines *successful traces* into demos | 10–50 | Cheapest real win for small models. |
| `MIPROv2` | instructions **and** demos jointly | ~200 ideal, works with 50+ | Bayesian search over candidates. |
| `GEPA` (2025) | instructions via **reflective evolution** | small; sample-efficient | See §1.3. |
| `BootstrapFinetune` | model weights | traces from any of the above | Distills prompt gains into the model. |

### 1.3 GEPA in three steps

1. **Run & score** — the *student* model (caro's small local model) runs the program
   over the trainset; the metric scores each example and returns **textual
   feedback** (`dspy.Prediction(score=0.0, feedback="expected -type f, got none")`).
2. **Reflect** — a *reflection* model reads failing traces + feedback and proposes
   a rewritten instruction.
3. **Select** — candidates are evaluated; GEPA keeps a Pareto frontier rather than
   a single winner, so a prompt that fixes `find` without breaking `ls` survives.

The docs' headline result: a small model went 78.1% → 90.1% and beat an
*unoptimized* frontier model. That is caro's exact regime — a 1.5B model whose
ceiling is set by its prompt.

### 1.4 The Python → Rust bridge

`compiled.save("program.json", save_program=False)` writes **inspectable JSON**
containing the signature, the optimized instruction text, and the few-shot demos.
DSPy does not need to exist at runtime. The optimizer runs offline in Python at
dev time; the Rust binary embeds or loads the JSON. This is the whole integration
story — no Python in the shipped binary, no new runtime dependency.

There is also a Rust rewrite, [DSRs / `dspy-rs`](https://github.com/krypticmouse/DSRs)
(crates.io `dspy-rs` 0.7.x, Apache-2.0, beta, ships a GEPA example). It is
discussed and set aside in §5.

---

## 2. Where caro is today

All claims below were verified on `main` at the doc date.

### 2.1 Prompts are source code, eight times over

Every backend owns a private `create_system_prompt()`:
`src/backends/embedded/embedded_backend.rs:183`, `src/backends/remote/ollama.rs:83`,
`vllm.rs`, `exo.rs`, `mesh.rs`, `ai_horde.rs`, `claude.rs`, `openrouter.rs`. Only
`{shell}` and `{input}` are injected; everything else is a literal string.

- The embedded prompt hardcodes macOS advice ("Use BSD-compatible flags (macOS)")
  regardless of host OS, numbers its rules 1, 2, 3, 5, 6… (no 4), and carries
  ~25 inline few-shot pairs with no platform filtering.
- Ollama and vLLM share a much weaker 7-rule prompt: no BSD/GNU guidance, no
  examples, no docker/k8s section. A user switching `--backend` silently gets a
  different — worse — prompt for the same query.
- Specialization is **per backend, not per model**. Nothing dispatches on model
  name; `qwen2.5-coder:1.5b` on Ollama and `codellama:7b` on Ollama get the same
  prompt.
- No prompt version is pinned to the shipped binary. `caro --version` cannot tell
  you which prompt produced a command.

### 2.2 The prompt machinery that *is* structured is dead

`src/prompts/` (`smollm_prompt.rs` 870 lines, `capability_profile.rs` 1,197,
`command_templates.rs` 731, `validation.rs` 1,004) is re-exported from
`src/lib.rs` but never called by a backend. It contains real per-platform example
selection — `build_examples_section()` at `smollm_prompt.rs:526` dispatches to
`gnu_examples()` / `bsd_examples()` / `busybox_examples()` / `posix_examples()`
(:547–:655) plus `build_negative_examples()` (:451). **None of it reaches a model.**
The live exceptions are `build_minimal_prompt`, `ExplainerPromptBuilder`, and
`build_risk_judge_prompt`.

### 2.3 Eval exists but is not an optimization signal

- **Data (~290 cases)**: `tests/evaluation/dataset.yaml` (101: correctness 26,
  safety 25, posix 25, multi_backend 25; `validation_rule ∈ exact_match |
  command_equivalence | pattern_match | must_be_blocked | must_execute`;
  `difficulty` tagged), `tests/evaluation/test_cases.toml` (56),
  `datasets/**/*.json` (80), `.claude/beta-testing/test-cases.yaml` (~58).
- **Harness**: `src/evaluation/` (harness, models, correctness/safety/posix/
  consistency evaluators, `baseline.rs`, `sft_export.rs`). Invoked as
  `cargo test --test evaluation -- --backend <b> --format json --baseline <p>`.
  The `caro-eval` binary is still commented out ("WP07").
- **Metric**: `tests/evaluation/src/evaluator.rs::evaluate_correctness` is a
  string ladder — 1.0 exact / 0.95 whitespace-normalized / 0.90 flag-order /
  0.0. Nothing executes commands; nothing is semantic.
  `EvaluationResult.failure_reason` (`src/evaluation/models.rs:155`) exists and
  the rule evaluators fill it, but the strings are for humans reading a report,
  not for a reflection model — and nothing exports them.
- **A/B infrastructure**: `tests/evaluation/src/prompt_comparison.rs` has
  `chi_square_test`, `find_winner`, `should_rollback`, `generate_report` — used
  only by its own unit tests. `tests/evaluation/prompts/v1.0/` is a versioned
  registry (`metadata.yaml`: `target_models: [smollm, qwen, static_matcher]`,
  `baseline_pass_rate: 0.31`) whose prompts **do not match production**.
- **CI gate** (`.github/workflows/evaluation.yml`): baselines are hardcoded —
  static_matcher 31.0%, every LLM backend `0.0` "TBD", which *skips the check*
  (:78–:81). The harness runs under `|| true` (:45) and a crashed run reports
  `backend_available=false` and passes.
- **Prior intent**: issue [#517](https://github.com/wildcard/caro/issues/517)
  "WP10-11: Prompt Engineering Framework" envisioned exactly this —
  `--compare-prompts v1.0,v1.1,v1.2`, semantic-versioned prompts, automated
  rollback (`thoughts/shared/plans/evaluation-harness-milestone-summary.md:47–60`).
  It was **closed as completed on 2026-01-17**, the day it was opened, once the
  registry and the A/B engine landed. Its own "Current State" list — prompts
  hard-coded in backends, no version control, no systematic comparison — is
  still true on `main` today. The deliverables shipped; disconnected from
  production, the goal did not.
  `src/prompts/minimal.rs` records the only documented A/B run (default 45.5% vs
  minimal 18.2% on 11 cases, discarded) and cites `data/evals/default.yaml`,
  which does not exist.

### 2.4 Runtime feedback loops

The flow (`src/cli/mod.rs` → `src/agent/mod.rs`): static matcher first → LLM
`generate_initial` (:479) → `CommandValidator::validate` → on failure **one**
`repair_command` (:582) whose `build_repair_prompt` (:693) embeds numbered
validation errors → optional advisor/refine if `confidence_score < 0.8` (:338) →
a **second, independent** `SafetyValidator::validate_command` pass
(`src/cli/mod.rs:836`) → approval → execute.

What exists is good: the repair prompt is already a feedback-carrying retry.
What is missing:

- The final safety pass has **no regeneration hook** — a blocked command is a
  dead end even when the block reason ("uses `rm -rf`") is exactly the feedback a
  second attempt needs.
- `_max_iterations: 2` (`src/agent/mod.rs:77`) is underscore-prefixed and unused;
  repair is single-shot.
- `confidence_score` is a **constant per backend**: 0.85 embedded
  (`embedded_backend.rs:511`), 1.0 static (`static_matcher.rs:1901`), 0.8 Ollama,
  0.75 AI-Horde, 0.95 Claude. The `< 0.8` refinement gate is therefore decided by
  *which backend answered*, not by anything the model did.

### 2.5 The manual loop

`.claude/skills/prompt-tuner/SKILL.md` is the process: run `caro test`, eyeball
failures against a six-row symptom table, hand-edit the embedded `format!` string,
`cargo build --release`, compare two numbers, commit or revert. The file is 107
lines and ends mid-example with an unclosed code fence. It never mentions the
other seven backend prompts, `--prompt-style`, or the A/B harness — so tuning
embedded silently leaves Ollama/vLLM stale.

This is the loop DSPy replaces. Not because humans are bad at it, but because a
human can hold one prompt × one model × eleven cases in their head, and caro has
eight prompts × seven catalogued models × ~290 cases.

---

## 3. The eight learnings

Each entry: the DSPy concept → the caro gap → the concrete change → why it pays.

### L1 — Prompts are compiled, versioned artifacts, not source code  *(keystone, M)*

**DSPy**: a Signature is declared once; `program.save()` emits the optimized
instruction + demos as JSON; the runtime loads it.

**caro gap**: §2.1 — eight literal strings, no version, no model binding.

**Change**: introduce a *prompt artifact* — one JSON document per (backend-family,
model) pair — and a single Rust render path that turns an artifact into the
system prompt. Proposed shape (illustrative; ADR-017 owns the final schema):

```json
{
  "schema_version": 1,
  "artifact_id": "embedded/qwen2.5-coder-1.5b-q4/v1.1",
  "model": { "catalog_id": "qwen-1.5b-q4", "family": "qwen2.5-coder" },
  "compiled_with": {
    "tool": "dspy", "version": "3.3.0", "optimizer": "GEPA", "auto": "light",
    "trainset": "tests/evaluation/dataset.yaml#split=train",
    "source_commit": "<sha>"
  },
  "signature": { "inputs": ["request", "shell", "platform"], "outputs": ["cmd"] },
  "instructions": "…optimized instruction text…",
  "output_format": { "kind": "json", "schema": { "cmd": "string" } },
  "demos": {
    "gnu":     [{ "request": "list all files", "cmd": "ls -a" }],
    "bsd":     [],
    "busybox": [],
    "posix":   []
  },
  "eval": { "train_pass_rate": 0.62, "heldout_pass_rate": 0.58, "baseline_pass_rate": 0.455 }
}
```

The default artifact ships via `include_str!` so the binary stays self-contained;
a `--prompt-artifact <path>` override (mirroring today's `--prompt-style`) lets
the harness test candidates without a rebuild. `tests/evaluation/prompts/v1.0/`
becomes the registry these artifacts live in, and `metadata.yaml`'s
`target_models` becomes real rather than aspirational.

**Why it pays**: it collapses eight prompts into one rendering function, makes
"which prompt produced this?" answerable, and is the precondition for every
optimizer-driven learning below. It also gives the `prompt_comparison.rs` A/B
engine real inputs for the first time.

### L2 — Metrics return a score *and* textual feedback  *(S)*

**DSPy**: GEPA's metric contract is `score + feedback`. The feedback is what the
reflection model reasons over; a bare 0/1 tells it nothing.

**caro gap**: the string ladder is binary in effect, and `failure_reason` is
prose for a report.

**Change**: make every evaluator emit a *diagnostic* reason with a stable shape —
which rule failed, expected vs got, the flag/operand diff, the safety pattern id
that fired — and add a JSONL export of `(input, expected, actual, score,
feedback)` per run. This is ~200 lines inside `src/evaluation/evaluators/` and
does not change pass/fail semantics.

**Why it pays**: it is the prerequisite for GEPA, and it improves the human
failure reports immediately. The `sft_export.rs` module already proves the
"eval run → JSONL" pattern; this is the negative-example twin of it.

### L3 — An offline optimizer harness replaces the manual loop  *(M)*

**DSPy**: `optimizer.compile(program, trainset, valset)`.

**caro gap**: §2.5, and the unmet goal of #517.

**Change**: `tools/dspy-harness/` (beside the existing `tools/mlx-finetune/`),
Python, dev-time only. It (a) loads `dataset.yaml` into `dspy.Example`s, (b)
mirrors the Rust scoring ladder as the metric with L2-style feedback, (c) runs
`BootstrapFewShot` first (cheap, no reflection model) and `GEPA` second, against
a **local** student via Ollama, and (d) writes an L1 artifact. Sketch —
**untested, illustrative only**; the real script lands in Phase 2 with a recorded
run:

```python
# tools/dspy-harness/optimize.py — SKETCH, not yet run. Pin: dspy>=3.3,<4
import dspy, yaml
from pathlib import Path

class ShellCommand(dspy.Signature):
    """Convert a natural-language request into one safe POSIX shell command."""
    request: str = dspy.InputField()
    shell: str = dspy.InputField(desc="target shell, e.g. bash")
    platform: str = dspy.InputField(desc="gnu | bsd | busybox | posix")
    cmd: str = dspy.OutputField(desc="the command only, no prose")

def load_cases(path="tests/evaluation/dataset.yaml"):
    for c in yaml.safe_load(Path(path).read_text()):
        if c.get("validation_rule") in ("exact_match", "command_equivalence"):
            yield dspy.Example(request=c["input_request"], shell="bash",
                               platform="gnu", cmd=c["expected_command"]
                               ).with_inputs("request", "shell", "platform")

def metric(gold, pred, trace=None, pred_name=None, pred_trace=None):
    # Same ladder as tests/evaluation/src/evaluator.rs, plus feedback text.
    got, want = normalize(pred.cmd), normalize(gold.cmd)
    if got == want:
        return dspy.Prediction(score=1.0, feedback="exact match")
    return dspy.Prediction(score=0.0,
        feedback=f"expected `{want}`, got `{got}`; flag diff: {flag_diff(want, got)}")

dspy.configure(lm=dspy.LM("ollama_chat/qwen2.5-coder:1.5b", temperature=0.1, max_tokens=100))
train, dev = stratified_split(list(load_cases()))   # dev is NEVER the CI gate set
program = dspy.Predict(ShellCommand)
opt = dspy.GEPA(metric=metric, auto="light",
                reflection_lm=dspy.LM("ollama_chat/qwen2.5-coder:7b"))
compiled = opt.compile(program, trainset=train, valset=dev)
compiled.save("tests/evaluation/prompts/artifacts/embedded-qwen-1.5b.json",
              save_program=False)
```

Two design notes. First, the reflection model is a dev-time choice: a local 7B
keeps the whole loop offline and consistent with caro's privacy stance; a
frontier model is faster and is acceptable because the dataset is synthetic —
no user data is involved. Second, the exporter must translate DSPy's JSON into
the L1 schema; do not make the Rust side parse DSPy's internal format.

**Why it pays**: the manual loop scales as (prompts × models × cases); this
scales as "run the script per model". It also finishes what #517 started, with a stronger
design than the original `--compare-prompts` idea, because candidates are
*generated*, not hand-authored.

### L4 — Bounded `Refine` on safety failure  *(S–M)*

**DSPy**: `dspy.Refine(module, N, reward_fn, threshold)` re-runs a module with
the reward's feedback until the threshold is met or N is exhausted.

**caro gap**: §2.4 — the final safety block is a dead end.

**Change**: when `SafetyValidator::validate_command` blocks at `cli/mod.rs:836`,
feed the block reason through the existing `build_repair_prompt` for **one**
regeneration, re-validate, then fail closed. Use the dormant `_max_iterations`
as the bound. Static-matcher `Critical` blocks stay terminal — they are
deliberate today and stay deliberate.

**Why it pays**: the user asked for something legitimate ("clean up old logs")
and the model reached for `rm -rf`; one retry with "that pattern is blocked; use
`find … -delete` with explicit filters" converts a block into an answer. This is
the only runtime learning that is a quick win, and it reuses code that exists.

### L5 — Platform few-shot demos as data, not code  *(S–M)*

**DSPy**: `LabeledFewShot` — demos are a dataset the optimizer selects from.

**caro gap**: §2.2 — the gnu/bsd/busybox/posix sets are dead Rust.

**Change**: migrate the example tuples from `smollm_prompt.rs:547–:655` into the
L1 artifact's `demos` block (or a YAML sibling the harness reads), then delete
the dead functions. The optimizer warm-starts from these instead of an empty
demo set, and the render path picks the platform block from the detected
profile — which also fixes the "BSD flags on Linux" defect in the shipped prompt.

**Why it pays**: 22 curated pairs per platform already exist and are tested; they
just never reach a model. Cheapest possible accuracy win on non-macOS hosts.

### L6 — One output adapter instead of eight parsers  *(M)*

**DSPy**: Adapters own the "ask for structure, parse it back, retry on garbage"
loop centrally.

**caro gap**: every backend duplicates the four-tier parse ladder
(`embedded_backend.rs:271` and its copies) and only embedded has the parse-retry.

**Change**: a `CommandOutputAdapter` in `src/backends/mod.rs` that renders the
output-format instruction from the artifact and owns parse + bounded retry; each
backend calls it. Behavior-preserving refactor with the existing parser tests as
the guard.

**Why it pays**: removes ~7 copies of subtle code, gives remote backends the
retry embedded already has, and makes output-format changes a one-place edit.
Lower leverage than L1–L5, so it rides along with Phase 3.

### L7 — Honest confidence  *(S–M)*

**DSPy**: `BestOfN` / self-consistency derive confidence from agreement across
samples, not from a constant.

**caro gap**: §2.4 — per-backend constants make the `< 0.8` gate fake.

**Change**: derive `confidence_score` from observable signals: parse tier hit
(strict JSON = high, regex rescue = low), validator outcome (clean / warnings /
repaired), and later `n=2` agreement for the embedded backend when latency
allows. The threshold logic in `AgentLoop` does not change; its input becomes
real.

**Why it pays**: the advisor and refinement paths already exist and are gated on
this number. Today they fire by backend identity; after this they fire when the
model was actually unsure.

### L8 — Distill compiled prompts into weights  *(L, strategic)*

**DSPy**: `BootstrapFinetune` takes traces from a compiled program and fine-tunes
the student, so the prompt gains become weight gains.

**caro gap**: `src/evaluation/sft_export.rs` and `docs/ml/sft-data-pipeline.md`
already collect passing trajectories; they have no prompt-side input.

**Change**: feed the L3 harness's bootstrapped demos and optimized instruction
into the SFT positive set, so the fine-tune target is "the compiled behavior".

**Why it pays**: demos are the cheap distillation; fine-tuning is the expensive
one. Sequencing them this way means the fine-tune track starts from the best
prompt-level behavior rather than the hand-tuned one. Not before L1–L3 exist.

### Cross-cutting mandates

- **Split hygiene.** The optimizer must never see the CI gate set. Tag
  `dataset.yaml` cases `split: train | heldout` (stratified by category ×
  difficulty, roughly 60/41) before any compilation. A compiled prompt evaluated
  on its own trainset will look great and the regression gate will lie.
- **Per-model artifacts.** Compile SmolLM and Qwen separately; pin
  artifact ↔ model in `model_catalog.rs`. A prompt tuned for a 1.5B coder model
  is the wrong prompt for a 135M CI model, and vice versa.
- **Safety is never optimized.** The optimizer touches generation instructions
  and demos only. `src/safety/patterns.rs` and the two validation passes remain
  deterministic and unchanged. If a compiled prompt raises the pass rate by
  producing commands the validator then blocks, that is a *worse* prompt, and the
  metric must say so (L2 feedback: "blocked by pattern `rm_rf_root`").

---

## 4. Adoption roadmap

| Phase | Scope | PR shape | Touchpoints |
|---|---|---|---|
| **0** | This document. | `docs:` PR, one file. | `docs/research/dspy-prompt-optimization.md` |
| **1** — Rust-only quick wins | ADR-017 (artifact architecture, *Proposed*); L2 diagnostic feedback + JSONL export; L4 bounded safety regeneration; L7 derived confidence; `split:` tags in `dataset.yaml`; CI gate hygiene (drop `\|\| true`, fail on crashed runs). | 4–5 small independent PRs, each with a regression-guard test per `.claude/rules/feature-evidence.md`. | `docs/adr/ADR-017-*.md`, `src/evaluation/evaluators/*`, `src/agent/mod.rs`, `src/cli/mod.rs`, `src/backends/*` (confidence), `.github/workflows/evaluation.yml` |
| **2** — Harness | `tools/dspy-harness/` (loader, metric, exporter); migrate L5 demos to data; first `BootstrapFewShot` run per model with **recorded before/after held-out numbers**; replace the `0.0 TBD` CI baselines with those numbers; wire `prompt_comparison.rs` to two real artifacts. | One tooling PR + one data PR. Successor to #517 (closed 2026-01-17, goal unmet). Python dev tooling, not a runtime SDK — no build-spike needed. | `tools/dspy-harness/`, `tests/evaluation/prompts/`, `src/prompts/smollm_prompt.rs` (delete dead sets) |
| **3** — Runtime | `PromptArtifact` loader (`include_str!` default + `--prompt-artifact` override) replacing the eight `create_system_prompt` bodies; L6 shared adapter; artifact id surfaced in `caro --version`; GEPA run once L2 feedback is diagnostic. | 2–3 PRs behind a feature flag until the compiled artifact beats the hand-tuned prompt on held-out. | `src/backends/mod.rs`, `src/backends/*/`, `src/model_catalog.rs` |
| **3b** — Optional | `dspy-rs` build-spike if a Rust-native optimizer ever becomes worth it; L8 `BootstrapFinetune` tie-in. | Per `.claude/rules/external-sdk-integration.md`. | `Cargo.toml` (optional dep, off by default) |

**Exit criterion for Phase 3**: the compiled artifact for `qwen-1.5b-q4` beats the
current hand-tuned embedded prompt on the *held-out* split, on the same machine,
same seed, with the gap reported by `prompt_comparison::chi_square_test`. Until
then the hand-tuned prompt stays the default and the artifact is opt-in.

### Proposed follow-up issues (not yet filed)

1. `eval: emit diagnostic failure_reason + JSONL feedback export from all evaluators` (L2)
2. `agent: bounded regeneration when the final safety pass blocks` (L4)
3. `backends: derive confidence_score from parse tier and validation outcome` (L7)
4. `eval: add split: train|heldout tags to dataset.yaml (stratified)` (hygiene)
5. `ci: evaluation.yml — fail on crashed runs; replace TBD baselines` (hygiene)
6. `adr: ADR-017 versioned prompt artifacts` (L1)
7. `tools: dspy-harness phase 2 — BootstrapFewShot/GEPA against dataset.yaml` (L3, successor to #517)
8. `prompts: migrate platform demo sets from smollm_prompt.rs into artifact data` (L5)
9. `backends: shared CommandOutputAdapter replacing per-backend parse ladders` (L6)
10. `skills: repair prompt-tuner SKILL.md (truncated fence, stale dataset path, cover all backends)` (boy-scout)
11. `ml: feed compiled demos/instructions into sft_export positive set` (L8)

---

## 5. Considered and rejected

- **Taking `dspy-rs` as a runtime dependency.** It is beta, its API is still
  moving, and caro's runtime needs are render + parse + bounded retry — a few
  hundred lines of Rust, not a framework. The optimizer is the valuable part and
  it runs offline. Revisit only if per-user, on-device optimization becomes a
  goal; then it goes through the ≤100-LOC build-spike rule first.
- **Running DSPy at runtime (Python sidecar).** Violates the single-binary,
  offline, no-Python promise. Never.
- **Replacing `src/evaluation/` with `dspy.Evaluate`.** The Rust harness is the
  CI gate and must stay Rust. The harness *mirrors* the metric; it does not
  replace it. Metric drift between the two is a real risk — mitigate with a
  fixture of ~20 `(actual, expected, score)` triples asserted on both sides.
- **Big-bang rewrite of `src/prompts/`.** The dead machinery is large and
  well-intended, but resurrecting it wholesale re-creates the two-parallel-
  systems problem. Harvest the data (L5), delete the rest when Phase 3 lands.
- **Optimizing safety patterns.** Out of scope by design (§3, cross-cutting).

---

## 6. Risks and guardrails

| Risk | Guardrail |
|---|---|
| Overfitting to `dataset.yaml`; CI gate becomes meaningless | Held-out split is mandatory before the first compile; gate runs on held-out only. |
| Artifact drifts from the model that ships (catalog bump, quant change) | Artifact pins `model.catalog_id`; a catalog change without a recompile fails a contract test. |
| Metric mismatch between Python harness and Rust harness | Shared fixture asserted on both sides (see §5). |
| Reflection-model cost / privacy | Dev-time only, synthetic data only; local 7B by default. |
| A compiled prompt "wins" by generating commands the validator blocks | L2 feedback penalizes blocked output; safety is never in the optimizer's search space. |
| Two prompt systems again (artifact + legacy strings) | Phase 3 deletes the eight `create_system_prompt` bodies in the same PR that flips the default. |

This doc is internal tooling / evaluation work, so the five gates in
`.claude/rules/validation-discipline.md` do not apply. Phase 1–3 feature PRs are
subject to `.claude/rules/feature-evidence.md` (green CI link, runnable demo,
named regression guard).

---

## 7. References

- DSPy repository — https://github.com/stanfordnlp/dspy (v3.3.0 at doc date)
- DSPy docs — https://dspy.ai (signatures, modules, adapters, optimizers, `Evaluate`)
- GEPA getting-started — https://dspy.ai/getting-started/gepa-optimization/
- Saving/loading programs — https://dspy.ai/tutorials/saving/
- DSRs (`dspy-rs`) — https://github.com/krypticmouse/DSRs · https://crates.io/crates/dspy-rs
- caro issue #517 "WP10-11: Prompt Engineering Framework" — https://github.com/wildcard/caro/issues/517 (closed 2026-01-17 as completed; goal unmet, see §2.3)
- caro prior art: `tests/evaluation/src/prompt_comparison.rs`,
  `tests/evaluation/prompts/v1.0/metadata.yaml`, `src/prompts/minimal.rs`,
  `src/evaluation/sft_export.rs`, `docs/ml/sft-data-pipeline.md`
- Repo rules that shape the roadmap: `.claude/rules/external-sdk-integration.md`,
  `.claude/rules/feature-evidence.md`, `.claude/rules/validation-discipline.md`,
  `.claude/rules/adr-numbering.md`
