# MLX LoRA fine-tune pipeline for caro

Fine-tunes `Qwen/Qwen2.5-Coder-1.5B-Instruct` on caro's own evaluation dataset
using Apple's `mlx-lm` library, then converts the result back to GGUF Q4_K_M so
it drops into caro's existing embedded backend unchanged.

**Target hardware:** Apple Silicon (tested on M4 Max, 36-64 GB unified memory).
Runs entirely on-device — no cloud, no CUDA, no PyTorch.

## Why not unsloth or lm-studio-bench?

| Tool | What it does | Fit for this job |
|---|---|---|
| **`mlx-lm`** (Apple) | LoRA / QLoRA / fuse on Metal | ✅ yes — only Apple-native option |
| `unsloth` | LoRA, but PyTorch + CUDA | ❌ no — no Apple Silicon support |
| `lm-studio-bench` | Benchmarks llama.cpp runtime flags | ❌ not training — useful *after* this pipeline to tune inference settings |

## Prerequisites

- macOS 14+ on Apple Silicon
- Python 3.11+
- Rust toolchain (already required for caro)
- [`llama.cpp`](https://github.com/ggerganov/llama.cpp) built locally (for the
  final GGUF conversion + quantization step)

```bash
# One-time: clone and build llama.cpp
git clone https://github.com/ggerganov/llama.cpp ~/src/llama.cpp
cd ~/src/llama.cpp && cmake -B build && cmake --build build --config Release
```

The Makefile defaults to `LLAMA_CPP=$HOME/src/llama.cpp`. Override with
`make gguf LLAMA_CPP=/path/to/llama.cpp`.

## Quick start

```bash
cd tools/mlx-finetune
make venv      # one-time: creates .venv/ with mlx-lm + pyyaml
make all       # full pipeline: data -> convert -> lora -> fuse -> gguf
```

Total wall-clock on M4 Max: ~45 min (30 min LoRA + 15 min everything else).

## What each step does

| Step | Command | Output | Notes |
|---|---|---|---|
| 1 | `make data` | `data/{train,valid,test}.jsonl` | 155 ChatML examples from caro's own eval suite |
| 2 | `make convert` | `qwen-mlx/` | Downloads Qwen 1.5B-Instruct from HF, quantizes to 4-bit MLX |
| 3 | `make lora` | `adapters/` | LoRA train, ~30 min on M4 Max |
| 4 | `make fuse` | `qwen-caro-fused/` | Merges adapter into base weights |
| 5 | `make gguf` | `qwen2.5-coder-1.5b-caro-v1-q4_k_m.gguf` | `llama.cpp` conversion + Q4_K_M quantization |
| 6 | `make eval` | *(manual)* | Prints the commands to run caro's regression harness |

## Tuning

All training hyperparameters are Makefile variables:

```bash
make lora ITERS=1000 LR=5e-5 BATCH=8 LORA_LAYERS=24
```

| Var | Default | Meaning |
|---|---|---|
| `ITERS` | 600 | LoRA training iterations. 400–1000 is the useful range for 155 examples. |
| `LR` | 1e-4 | AdamW learning rate. Lower if `valid_loss` spikes. |
| `BATCH` | 4 | Batch size. M4 Max can handle 8 at this model size. |
| `LORA_LAYERS` | 16 | Rank/depth. More = more capacity but more overfitting risk. |
| `SEED` | 42 | Deterministic shuffle + init. |

**Watch for overfitting.** The dataset is tiny (155 examples). If `valid_loss`
stops dropping or starts rising, stop training — there's a
`--save-every 100` checkpoint you can fuse from instead.

## Why the system prompt matters

Caro's embedded backend speaks a strict ChatML contract defined in
`src/prompts/smollm_prompt.rs`:

```
<|im_start|>system
{~8.8 KB of rules about command generation, safety, templates, negative examples}
<|im_end|>
<|im_start|>user
{user's natural-language intent}
<|im_end|>
<|im_start|>assistant
{"cmd": "..."}      # OR   QUESTION: <clarify>
```

The fine-tuning data MUST embed the **same** system prompt the runtime uses,
or the model will learn a prompt the backend never sends and degrade on the
benchmark. That's why step 1 of the pipeline shells out to a Rust example that
renders the canonical prompt from the source of truth:

```bash
cargo run --quiet --example render_system_prompt > system_prompt.txt
```

The example lives at [`examples/render_system_prompt.rs`](../../examples/render_system_prompt.rs)
and calls `SmolLMPromptBuilder::build_system_prompt()` directly — zero drift
risk.

## Dataset composition

`build_dataset.py` pulls from two files already in the caro repo:

- `tests/evaluation/dataset.yaml` — 100 labelled cases (correctness / safety /
  POSIX / multi-backend)
- `tests/evaluation/test_cases.toml` — 55 additional cases

After normalization (2026-04 snapshot):

```
total cases:    155 (130 command, 25 question)
train.jsonl:    132 examples
valid.jsonl:    23 examples
test.jsonl:     25 safety examples (regression gate — never used for training loss)
```

Each `must_be_blocked` safety case is converted to a `QUESTION: ...` target
matching caro's existing "clarify before destruction" contract. Training on
these is **required** — without them the LoRA adapter forgets the safety
pattern and starts emitting raw destructive commands.

## Deploying the fine-tuned model

Once `qwen2.5-coder-1.5b-caro-v1-q4_k_m.gguf` exists:

1. Upload to a HuggingFace repo (e.g. `wildcard/caro-qwen-1.5b-v1`).
2. Add a new entry to `src/model_catalog.rs`, mirroring the existing
   `qwen-1.5b-q4` entry but pointing at the new HF repo + filename.
3. Run the regression gate:
   ```bash
   cargo test --release --package caro -- evaluation
   ```
4. Compare pass-rate vs the stock `qwen-1.5b-q4` baseline (currently ~93.1%
   per `CLAUDE.md`). **Ship only if pass-rate improves AND zero new safety
   false-negatives on `test.jsonl`.**

## Optional: runtime tuning with `lm-studio-bench`

After the GGUF exists, `lm-studio-bench` is genuinely useful for finding the
best `llama.cpp` runtime flags (threads, KV cache type, GPU layer count, Flash
Attention) on your specific M4 Max. That's orthogonal to fine-tuning — you run
it once per target machine after training, and bake the flags into caro's
embedded backend config.

## Files in this directory

```
tools/mlx-finetune/
├── README.md          (this file)
├── Makefile           (one-command pipeline)
├── build_dataset.py   (eval YAML+TOML -> mlx-lm JSONL)
└── .gitignore         (excludes weights, adapters, data/)
```

Generated artifacts (all gitignored):

```
.venv/                 (Python env with mlx-lm)
system_prompt.txt      (rendered from src/prompts/smollm_prompt.rs)
data/                  (train.jsonl, valid.jsonl, test.jsonl)
qwen-mlx/              (base model, MLX 4-bit)
adapters/              (LoRA checkpoints)
qwen-caro-fused/       (merged safetensors)
*.gguf                 (final deployable artifact)
```
