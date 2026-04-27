# Caro ML/DS engineer session log

> **Owner:** caro-ml-ds-engineer (autonomous role; daily 12:05 AM PST)
> **Last updated:** 2026-04-27 by caro-ml-ds-engineer
> **External companion:** [`docs/fine-tune-pipeline.md`](../../docs/fine-tune-pipeline.md)

Append-only. One entry per loop pass. No-op passes are valid — log them with `Move: no-op` and a one-sentence rationale. The header date should match the most recent entry below.

## Format

```markdown
## YYYY-MM-DD — <one-line summary>
- **Move:** <category> — <slug>
- **Artifact:** <link to issue or PR>
- **Eval delta:** <pass-rate change vs. last run, or N/A>
- **Next:** <one-sentence pointer for tomorrow's pass>
```

Categories: `dataset` · `dataset-hook` · `fine-tune` · `eval-metric` · `model-selection` · `pipeline-infra` · `no-op`.

---

## 2026-04-27 — bootstrap pass

- **Move:** pipeline-infra — bootstrap fine-tune loop scaffolding
- **Artifact:** [#944](https://github.com/wildcard/caro/issues/944) (issue) ; PR: this one
- **Eval delta:** N/A
- **Next:** Tag the existing 100 dataset cases with shell-dialect / OS-family / input-register dimensions (data PR, S effort) — this surfaces coverage gaps so subsequent dataset-growth passes know what to grow.

## Queue (do NOT execute today — one move per pass)

These are the next-best moves identified during the bootstrap pass. Each daily pass picks the topmost still-applicable item, confirms it's still the right call against the latest inbox, and then files / implements:

1. **dataset** — Tag-coverage sweep across `tests/evaluation/dataset.yaml` (S)
2. **eval-metric** — Latency-on-M4-Max cold + warm cache metric in [`src/evaluation/harness.rs`](../../src/evaluation/harness.rs) (M)
3. **model-selection** — Qwen2.5-Coder-7B baseline-eval issue (M)
4. **pipeline-infra** — `ModelVariant` enum extension to support a model-identity axis in [`src/backends/embedded/common.rs`](../../src/backends/embedded/common.rs) (M)
5. **eval-metric** — Shell-portability cross-parse metric (M)
6. **eval-metric** — Execution-correctness sandbox metric (L — defer until 1–4 land)
