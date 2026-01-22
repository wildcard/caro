# Fine-tuning Alternatives for Shell Command Generation

> Research document exploring alternatives to Unsloth for distilling small models with shell command instructions.

## TL;DR Recommendations

| Use Case | Recommended Tool | Why |
|----------|-----------------|-----|
| **Quick iteration on Mac** | MLX | Native Apple Silicon, LoRA/QLoRA, no code needed |
| **Production training** | Axolotl | Multi-GPU, comprehensive features, well-maintained |
| **Memory constrained** | Unsloth | 80% less memory, 2-5x faster |
| **Factual accuracy** | Lamini Memory Tuning | 95%+ accuracy, handles 100K+ facts |
| **End-to-end distillation** | Distil Labs CLI | Purpose-built for small models |
| **Rust inference** | Train elsewhere → GGUF/Candle | Rust training is immature |

---

## 1. Python-Based Frameworks (Most Mature)

### 1.1 Unsloth (What You Know)
- **Best for**: Memory-constrained environments, single GPU
- **Performance**: 2-5x faster, 80% less memory than Flash Attention 2
- **Supports**: Llama 3.x, Mistral, Phi, Gemma
- **Limitation**: No multi-GPU training
- **URL**: https://github.com/unslothai/unsloth

### 1.2 Axolotl ⭐ Recommended for Production
- **Best for**: Comprehensive fine-tuning with multi-GPU support
- **Features**:
  - YAML-based config (no code)
  - Sample packing for efficiency
  - DeepSpeed & FSDP integration
  - Supports QLoRA, LoRA, full fine-tuning
  - Reward modeling (Jan 2025)
  - Quantization Aware Training (May 2025)
- **2025 Updates**: Llama 4, Multimodal, Sequence Parallelism
- **URL**: https://github.com/axolotl-ai-cloud/axolotl

```bash
# Example: Fine-tune Qwen for shell commands
accelerate launch -m axolotl.cli.train examples/qwen/qlora.yml
```

### 1.3 TRL + PEFT (HuggingFace Native)
- **Best for**: Integration with HuggingFace ecosystem
- **Features**:
  - SFT, DPO, RLHF, PPO training
  - Native PEFT integration (LoRA, QLoRA, Spectrum)
  - Liger Kernels for optimization
- **URL**: https://github.com/huggingface/trl

```bash
# QLoRA fine-tuning
python trl/scripts/sft.py \
    --model_name_or_path Qwen/Qwen2-0.5B \
    --dataset_name your-shell-dataset \
    --load_in_4bit --use_peft \
    --lora_r 32 --lora_alpha 16 \
    --output_dir shell-command-model
```

### 1.4 Lamini Memory Tuning ⭐ For High Accuracy
- **Best for**: When you need 95%+ accuracy on factual/structured outputs
- **How it works**: Mixture of Memory Experts (MoME) - millions of LoRA adapters
- **Results**: 95% accuracy vs 50% with traditional fine-tuning
- **Scaling**: Works with 10 to 100,000+ examples
- **Cost**: Smaller models with high accuracy = lower inference costs
- **URL**: https://docs.lamini.ai/tuning/memory_tuning/

> **For Caro**: Shell commands are highly structured and factual. Memory Tuning could eliminate hallucinations like wrong flags or non-POSIX syntax.

---

## 2. Apple Silicon Optimized

### 2.1 MLX (Apple's Framework) ⭐ Best for Mac Development
- **Best for**: Local development on Apple Silicon
- **Features**:
  - Native M1/M2/M3/M4/M5 optimization
  - LoRA and full fine-tuning
  - Quantized training (QLoRA)
  - No code needed via mlx_lm CLI
  - WWDC 2025: Neural Accelerator support (4x speedup on M5)
- **Memory requirements**:
  - 7B QLoRA: ~6-7GB
  - 7B LoRA: ~14GB
  - 7B Full: ~28GB
- **URL**: https://github.com/ml-explore/mlx-lm

```bash
# Fine-tune Mistral-7B for shell commands
mlx_lm.lora \
    --model mistralai/Mistral-7B-v0.1 \
    --data ./caro-training-data \
    --train --iters 1000

# Fuse adapter back
mlx_lm.fuse --model mistralai/Mistral-7B-v0.1 --adapter-path ./adapters
```

---

## 3. Rust Ecosystem (Inference-First)

### 3.1 Burn
- **Status**: Training is **experimental** (2025 roadmap)
- **Good for**: Inference, ONNX import, WebAssembly
- **Training**: Dynamic-shape graphs still maturing
- **URL**: https://burn.dev/

**Verdict**: Not ready for production fine-tuning.

### 3.2 Candle (HuggingFace)
- **Status**: **Inference-first**, training experimental
- **Good for**: Rust inference, quantized models, WASM deployment
- **Supports**: LLaMA, Mistral, Phi, Gemma, StarCoder
- **Training**: Basic autodiff exists but not production-ready
- **URL**: https://github.com/huggingface/candle

**Verdict**: Use for inference after training elsewhere.

### 3.3 mistral.rs
- **Status**: **Inference only** (no training support)
- **Performance**: Near llama.cpp speeds with pure Rust
- **Features**: OpenAI-compatible API, CUDA/Metal, automatic tensor parallelism
- **URL**: https://github.com/EricLBuehler/mistral.rs

**Verdict**: Excellent for Caro's inference, but can't train with it.

### 3.4 Hybrid Approach for Rust Projects ⭐ Recommended
```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Train with     │ ──▶ │  Convert to     │ ──▶ │  Inference in   │
│  Axolotl/MLX    │     │  GGUF format    │     │  Rust (mistral.rs/│
│  (Python)       │     │                 │     │  Candle)         │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

---

## 4. llama.cpp (CPU Fine-tuning)

- **Best for**: Fine-tuning on CPU without GPU
- **How**: Works directly with quantized GGUF files
- **Limitation**: Very slow (days/weeks on CPU)
- **Output**: LoRA adapter in `.bin` format, mergeable with base model
- **URL**: https://github.com/ggml-org/llama.cpp

```bash
# Fine-tune GGUF model on CPU
./finetune \
    --model-base models/qwen2-0.5b-q4_k_m.gguf \
    --train-data ./shell-commands.txt \
    --lora-out ./shell-lora.bin
```

---

## 5. Distillation-Specific Tools

### 5.1 Distil Labs CLI ⭐ Purpose-Built for SLMs
- **Best for**: End-to-end small model distillation
- **Models**: 100M to 8B parameters
- **Tasks**: Classification, NER, QA, function calling
- **Example**: [GitAra](https://github.com/distil-labs/distil-gitara) - git command model
- **Results**: 3B model matches 120B teacher (25x smaller)
- **URL**: https://www.distillabs.ai/

> **For Caro**: GitAra is literally the same use case (git commands). Their approach has been validated.

### 5.2 OpenAI Distillation API
- **Best for**: Using GPT-4o as teacher
- **Features**: Built-in evaluation, dataset generation
- **Cost**: Pay for API calls during distillation
- **URL**: https://platform.openai.com/docs/guides/distillation

### 5.3 MiniLLM Framework
- **Paper**: Reverse KLD minimization for language generation
- **Best for**: Research-grade distillation with theoretical backing
- **URL**: https://arxiv.org/pdf/2306.08543

---

## 6. Synthetic Data Generation

Your Caro evaluation data is already well-structured. To expand it:

### 6.1 Meta Synthetic Data Kit ⭐
```bash
pip install synthetic-data-kit

# Generate QA pairs from docs
synthetic-data-kit ingest ./shell-docs/
synthetic-data-kit create ./data/ --type qa
synthetic-data-kit curate ./data/ --threshold 8.0
synthetic-data-kit save-as ./curated/ --format alpaca
```

### 6.2 NVIDIA Nemotron-4 340B
- Open models for synthetic data generation
- Base + Instruct + Reward models
- Commercial use allowed

### 6.3 Evidently AI (Open Source)
- Generate test inputs with customizable user profiles
- Good for RAG evaluation datasets

---

## 7. Recommended Pipeline for Caro

Based on your requirements (shell commands, Caro data, potentially Rust integration):

### Option A: Fast Iteration (Mac)
```
Caro Eval Data → MLX Fine-tune → GGUF Export → Caro Inference (mistral.rs)
```

### Option B: Production Quality
```
Caro Eval Data → Expand with Synthetic Data Kit → Axolotl (QLoRA) → GGUF → Caro
```

### Option C: Maximum Accuracy
```
Caro Eval Data → Lamini Memory Tuning → Export → Caro Inference
```

### Option D: Proven Same-Domain Approach
```
Follow Distil Labs GitAra approach → Adapt for shell commands
```

---

## 8. Caro Data Format Compatibility

Your current evaluation format:
```json
{
  "prompt": "list all files in the current directory",
  "expected_command": "ls",
  "category": "file_operations",
  "posix_compliant": true
}
```

**Conversion to training formats**:

| Format | Conversion |
|--------|-----------|
| Alpaca | `{"instruction": prompt, "output": expected_command}` |
| ChatML | `[{"role": "user", "content": prompt}, {"role": "assistant", "content": expected_command}]` |
| ShareGPT | `{"conversations": [{"from": "human", "value": prompt}, {"from": "gpt", "value": expected_command}]}` |

---

## 9. Comparison Matrix

| Tool | Training | Multi-GPU | Rust | Memory | Speed | Shell-Specific |
|------|----------|-----------|------|--------|-------|----------------|
| Unsloth | ✅ | ❌ | ❌ | ⭐⭐⭐ | ⭐⭐⭐ | ❌ |
| Axolotl | ✅ | ✅ | ❌ | ⭐⭐ | ⭐⭐ | ❌ |
| MLX | ✅ | ❌ | ❌ | ⭐⭐⭐ | ⭐⭐ | ❌ |
| Lamini | ✅ | ✅ | ❌ | ⭐⭐ | ⭐⭐ | ❌ |
| Burn | ⚠️ | ⚠️ | ✅ | ⭐⭐ | ⭐⭐ | ❌ |
| Candle | ⚠️ | ❌ | ✅ | ⭐⭐ | ⭐⭐⭐ | ❌ |
| Distil Labs | ✅ | ✅ | ❌ | ⭐⭐ | ⭐⭐ | ✅ (GitAra) |
| llama.cpp | ✅ | ❌ | ❌ | ⭐⭐⭐ | ⭐ | ❌ |

---

## 10. Key Insights

1. **Rust training isn't ready**: Both Burn and Candle are inference-first. Plan for Python training → Rust inference.

2. **Distil Labs solved your exact problem**: GitAra distills git commands. Their approach is validated.

3. **Lamini for accuracy**: If POSIX compliance and correct flags matter (they do for Caro), Memory Tuning's 95% accuracy is compelling.

4. **MLX for iteration speed**: On Apple Silicon, MLX lets you iterate quickly without cloud costs.

5. **Your eval data is training data**: The Caro evaluation JSON format is easily converted to any training format.

---

## Sources

- [Burn ML Framework](https://burn.dev/)
- [Candle](https://github.com/huggingface/candle)
- [MLX-LM](https://github.com/ml-explore/mlx-lm)
- [Axolotl](https://github.com/axolotl-ai-cloud/axolotl)
- [TRL + PEFT](https://huggingface.co/docs/trl)
- [Lamini Memory Tuning](https://docs.lamini.ai/tuning/memory_tuning/)
- [Distil Labs](https://www.distillabs.ai/)
- [mistral.rs](https://github.com/EricLBuehler/mistral.rs)
- [Meta Synthetic Data Kit](https://github.com/meta-llama/synthetic-data-kit)
- [Fine-tuning LLMs in 2025](https://www.philschmid.de/fine-tune-llms-in-2025)
- [Modal Fine-tuning Guide](https://modal.com/blog/fine-tuning-llms)
