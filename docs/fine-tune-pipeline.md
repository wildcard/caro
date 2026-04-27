# Caro Fine-Tune Pipeline

> **Owner:** [caro-ml-ds-engineer](../.claude/agents/ml-ds-engineer.md) (autonomous role; daily 12:05 AM PST via [`/ml-fine-tune-loop`](../.claude/commands/ml-fine-tune-loop.md))
> **Last updated:** 2026-04-27
> **Tracking issue:** [#944](https://github.com/wildcard/caro/issues/944) — bootstrap

## Mission

Deliver, through small reproducible experiments, the best open-source language model for natural-language → POSIX-shell-command translation that runs on a developer's **MacBook Pro M4 Max with 48 GB unified memory** while leaving headroom for the user's editor + browser + simulator.

End-to-end ownership: dataset → base-model selection → fine-tune (LoRA/QLoRA on mlx-lm) → eval → packaging into Caro's embedded backend (GGUF Q4_K_M default, Q6_K opt-in for accuracy).

## Hardware envelope (M4 Max 48 GB)

| Slice | Use |
|---|---|
| ~6–8 GB | OS + dev environment baseline |
| ~12–16 GB | model weights at 4-bit for 7B–14B |
| ~4–6 GB | KV cache headroom for 4k-context queries |
| ~8 GB | LoRA training scratch (gradients + optimizer state) on QLoRA |
| remainder | the user's actual work — never starve it |

If a candidate model needs >24 GB resident at inference time, it is not a Caro candidate. Caro is meant to coexist with the developer's normal toolchain, not own the machine.

## Candidate base-model shortlist

| Model | Params | 4-bit GGUF size | License | Bake-off pass-rate | Status |
|---|---|---|---|---|---|
| Qwen2.5-Coder-7B | 7.6B | ~4.5 GB | Apache 2.0 | TBD | candidate |
| Llama-3.1-8B-Instruct | 8B | ~5 GB | Llama Community | TBD | candidate |
| DeepSeek-Coder-V2-Lite-16B | 16B (MoE, 2.4B active) | ~9 GB | DeepSeek License | TBD | candidate |
| Granite-3.0-8B-Code | 8B | ~5 GB | Apache 2.0 | TBD | candidate |
| SmolLM-1.7B | 1.7B | ~1 GB | Apache 2.0 | TBD | sanity baseline |
| SmolLM-135M | 135M | ~80 MB | Apache 2.0 | (current shipped, smoke only) | smoke |

Re-evaluate quarterly and on every notable OSS coder-model release.

## Dataset coverage

Source: [`tests/evaluation/dataset.yaml`](../tests/evaluation/dataset.yaml) (100 hand-curated cases as of 2026-04-27).

| Dimension | Tags | Coverage today |
|---|---|---|
| Difficulty | easy / medium / hard | ✅ tagged (40 / 40 / 20) |
| Category | correctness / safety / posix / multi_backend | ✅ tagged |
| Shell dialect | bash / zsh / fish / sh | ❌ untagged |
| OS family | macos / linux / bsd | ❌ untagged |
| Intent | file-ops / git / networking / process / system / ai-pipelines | ⚠️ partial via category |
| Danger level | safe / blocked / borderline | ✅ paired via safety harness |
| Input register | terse / verbose / non-native-EN | ❌ untagged |

The four `❌`/`⚠️` rows are the active growth surface.

## Eval metrics

**Implemented** (in [`src/evaluation/harness.rs`](../src/evaluation/harness.rs)):
- pass / fail per case
- regression threshold against a baseline run

**Planned** (each filed as its own issue when its turn comes):
- Execution-correctness sandbox — does the generated command actually do what was asked when run safely
- Shell-portability cross-parse — bash answer parses cleanly under `sh` / `zsh`
- Latency on M4 Max — cold and warm-cache inference time
- Safety-validator pass rate paired with correctness — no false negatives slipping past

## Recent experiments

_(empty — first pass)_

## Packaging targets

When a fine-tune wins, rollout is:
1. Add the variant to `ModelVariant` in [`src/backends/embedded/common.rs`](../src/backends/embedded/common.rs)
2. Update the MLX path in [`src/backends/embedded/mlx.rs`](../src/backends/embedded/mlx.rs)
3. Wire a download script (model on Hugging Face under an OSI-compatible license)
4. Update this doc + ship a model card

Default ship format: **GGUF Q4_K_M**. Opt-in: **Q6_K**. Never ship Q8 (no measurable gain on this task; doubles disk).

## Open questions for the user

_(none today)_

## See also

- Sub-agent persona: [`.claude/agents/ml-ds-engineer.md`](../.claude/agents/ml-ds-engineer.md)
- Interactive skill: [`.claude/commands/caro.ml.md`](../.claude/commands/caro.ml.md)
- Daily loop: [`.claude/commands/ml-fine-tune-loop.md`](../.claude/commands/ml-fine-tune-loop.md)
- Internal session log: [`.claude/memory/ml-session-log.md`](../.claude/memory/ml-session-log.md)
- Tracking: [issues labelled `ml`](https://github.com/wildcard/caro/issues?q=is%3Aissue+label%3Aml)
