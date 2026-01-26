# ADR-004: FunctionGemma + LM Studio Backend Integration

**Status**: Proposed

**Date**: 2026-01-04

**Authors**: Caro Maintainers

**Target**: Community

## Context

Caro currently supports multiple inference backends (MLX/llama.cpp for embedded, Ollama, vLLM, Exo for remote), but all rely on general-purpose code models like Qwen 2.5 Coder. These models excel at code generation but were not specifically designed for the function-calling/tool-use pattern that caro employs (natural language → structured shell command).

A new class of purpose-built function-calling models has emerged, notably Google's **FunctionGemma** (270M parameters). Combined with:
- **Unsloth**: Fine-tuning framework that reduces training time by 2x and VRAM usage significantly
- **LM Studio**: Desktop application for local LLM serving with OpenAI-compatible API

This creates an opportunity for users to:
1. Fine-tune a tiny, purpose-built model on their specific shell command patterns
2. Serve it locally with zero cloud dependencies
3. Achieve fast inference due to the small model size

### The Problem

| Issue | Impact |
|-------|--------|
| Generic models waste capacity on irrelevant tasks | Slower inference, larger memory footprint |
| No customization path for users | Cannot adapt to organization-specific commands |
| Large models required for quality | Excludes resource-constrained environments |
| Fine-tuning is complex and expensive | Barrier to entry for most users |

### The Opportunity

FunctionGemma + Unsloth + LM Studio addresses all of these:

| Solution | Benefit |
|----------|---------|
| 270M parameters, function-calling native | Fast inference, purpose-built architecture |
| Unsloth fine-tuning on free Colab | Accessible customization for everyone |
| LM Studio local serving | Privacy, no API costs, offline capability |
| GGUF format with quantization | Small files (< 200MB), runs on CPU |

## Decision

We will add **LM Studio as a new remote backend** for caro, with first-class support for FunctionGemma and comprehensive documentation for the fine-tuning workflow.

### Implementation Approach

```
src/backends/remote/
├── mod.rs          # Add lm_studio export
├── ollama.rs       # Existing
├── vllm.rs         # Existing
├── exo.rs          # Existing
└── lm_studio.rs    # NEW: LM Studio backend
```

The LM Studio backend will:

1. **Connect to LM Studio's OpenAI-compatible API** (`http://localhost:1234/v1`)
2. **Support both completions and chat endpoints** for flexibility
3. **Parse FunctionGemma's tool-calling response format** natively
4. **Fall back to embedded backend** when LM Studio unavailable
5. **Integrate with existing backend selection logic** in `cli/mod.rs`

### Configuration

```toml
# ~/.caro/config.toml
[backend.lm_studio]
enabled = true
base_url = "http://localhost:1234"
model = "functiongemma-270m-finetuned"
timeout_seconds = 30
```

### Backend Priority Order (Updated)

```
1. User CLI flag (--backend lmstudio|ollama|vllm|exo|embedded)
2. Configuration file preference
3. Auto-detection:
   a. Exo Cluster (localhost:52415)
   b. LM Studio (localhost:1234)  # NEW
   c. Ollama (localhost:11434)
   d. vLLM (localhost:8000)
   e. Embedded (always available)
```

## Rationale

### Why FunctionGemma?

| Factor | FunctionGemma | Qwen 2.5 Coder 1.5B | CodeLlama 7B |
|--------|---------------|---------------------|--------------|
| **Parameters** | 270M | 1.5B | 7B |
| **Purpose** | Function calling | General code | General code |
| **Memory (Q4)** | ~200MB | ~1.1GB | ~4GB |
| **Inference speed** | Very fast | Fast | Moderate |
| **Fine-tuning time** | ~1 hour (Colab) | ~4 hours | ~8+ hours |
| **Fine-tuning cost** | Free (Colab T4) | Requires A100 | Requires A100+ |

FunctionGemma is **5.5x smaller** than our current default model while being specifically designed for the task caro performs.

### Why LM Studio?

| Factor | LM Studio | Ollama | vLLM |
|--------|-----------|--------|------|
| **Setup complexity** | GUI, drag-drop | CLI, somewhat complex | Server config required |
| **GGUF import** | Native, one-click | Modelfile required | Not supported |
| **API compatibility** | OpenAI-compatible | Custom + OpenAI | OpenAI-compatible |
| **Target users** | Developers, beginners | CLI power users | Production teams |
| **Cross-platform** | macOS, Windows, Linux | macOS, Linux | Linux primarily |

LM Studio's **zero-config import** of custom GGUF models makes it ideal for users who fine-tune their own models.

### Why Unsloth for Fine-Tuning?

| Factor | Unsloth | Standard HF Training | PEFT/LoRA alone |
|--------|---------|---------------------|-----------------|
| **Speed** | 2x faster | Baseline | 1.5x faster |
| **VRAM usage** | 50% reduction | High | 70% of baseline |
| **Colab T4 support** | Yes | Often OOM | Sometimes |
| **GGUF export** | Built-in | Manual conversion | Manual conversion |
| **Learning curve** | Low (notebooks) | High | Medium |

Unsloth's **Google Colab notebooks** enable fine-tuning on the free tier, democratizing model customization.

## Consequences

### Benefits

1. **Smallest viable model**: 270M parameters runs fast on any hardware, including CPU-only
2. **Purpose-built architecture**: Native function-calling means better accuracy per parameter
3. **User customization**: Fine-tuning enables organization-specific command patterns
4. **Zero cost**: Free Colab for training, free LM Studio for serving
5. **Privacy**: All inference local, no data leaves the machine
6. **Fast iteration**: Quick fine-tuning cycles (< 1 hour) enable experimentation
7. **Existing patterns**: LM Studio backend follows established remote backend architecture

### Trade-offs

1. **Additional dependency**: Users must install LM Studio (optional, with embedded fallback)
2. **Documentation burden**: Must maintain fine-tuning guides as ecosystem evolves
3. **Model versioning**: Fine-tuned models are user-managed, not auto-updated
4. **Debugging complexity**: Custom models may produce unexpected outputs
5. **Support scope**: Cannot support arbitrary user fine-tuning issues

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| LM Studio discontinues development | Low | High | API is OpenAI-compatible; can switch to alternatives |
| FunctionGemma deprecated by Google | Low | Medium | Architecture works with any function-calling model |
| Unsloth breaks compatibility | Medium | Low | Pin versions in docs; users keep working models |
| Users create unsafe fine-tuned models | Medium | Medium | Safety validation runs on ALL backends |
| Poor fine-tuning leads to bad UX | Medium | Medium | Provide pre-made training datasets and examples |

## Alternatives Considered

### Alternative 1: Built-in Fine-Tuning CLI

- **Description**: Add `caro finetune` command with integrated training
- **Pros**: Seamless UX, no external tools needed
- **Cons**: Massive binary size, requires CUDA/Metal deps, complex maintenance
- **Why rejected**: Violates single-binary philosophy; fine-tuning is one-time, not runtime

### Alternative 2: Ollama with Custom Models

- **Description**: Use Ollama's custom Modelfile for FunctionGemma
- **Pros**: Already supported backend, familiar to users
- **Cons**: Modelfile creation is complex, GGUF import less intuitive than LM Studio
- **Why rejected**: Higher friction for the target use case (custom fine-tuned models)

### Alternative 3: Direct llama.cpp Server

- **Description**: Have users run llama.cpp server directly
- **Pros**: Lightweight, no GUI dependency
- **Cons**: CLI-only, manual setup, no model management UI
- **Why rejected**: LM Studio provides better UX for the same underlying technology

### Alternative 4: Cloud Fine-Tuning Services

- **Description**: Integrate with Hugging Face AutoTrain or Modal
- **Pros**: No local GPU needed, professional tooling
- **Cons**: Costs money, data leaves machine, requires accounts
- **Why rejected**: Conflicts with local-first, privacy-focused philosophy

## Implementation Notes

### Phase 1: LM Studio Backend (Core)

1. Create `src/backends/remote/lm_studio.rs` following Ollama/vLLM patterns
2. Implement `CommandGenerator` trait with OpenAI-compatible API calls
3. Add function-calling response parser for FunctionGemma format
4. Update `cli/mod.rs` backend selection to include LM Studio
5. Add configuration support in `config.rs`
6. Write integration tests with mock LM Studio server

### Phase 2: Documentation

1. Create `/docs/guides/functiongemma-finetuning.md`
2. Include Unsloth Colab notebook walkthrough
3. Document GGUF conversion options (Q4_K_M, Q8_0, F16)
4. Provide LM Studio import instructions
5. Create example training dataset for shell commands

### Phase 3: Community Resources

1. Publish pre-fine-tuned FunctionGemma on Hugging Face (`caro-community/functiongemma-shell`)
2. Create video tutorial for end-to-end workflow
3. Add troubleshooting guide for common issues
4. Gather community fine-tuned models in awesome-caro repository

### API Integration

```rust
// LM Studio uses OpenAI-compatible API
POST /v1/chat/completions
{
  "model": "functiongemma-270m-finetuned",
  "messages": [
    {"role": "system", "content": "You are a shell command generator..."},
    {"role": "user", "content": "list all running docker containers"}
  ],
  "tools": [...],  // Optional: structured tool definitions
  "temperature": 0.1,
  "max_tokens": 100
}
```

### Response Parsing

FunctionGemma returns tool calls in OpenAI format:

```json
{
  "choices": [{
    "message": {
      "tool_calls": [{
        "function": {
          "name": "execute_shell",
          "arguments": "{\"command\": \"docker ps\"}"
        }
      }]
    }
  }]
}
```

Or plain text when not using tools:

```json
{
  "choices": [{
    "message": {
      "content": "{\"cmd\": \"docker ps\"}"
    }
  }]
}
```

The backend must handle both formats gracefully.

## Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Backend adoption | 500+ users in 6 months | Telemetry opt-in |
| Fine-tuning guide completions | 100+ | Community reports |
| Community fine-tuned models | 20+ on HuggingFace | Search/tags |
| Inference latency | < 2s on M1 Mac | Automated benchmarks |
| User satisfaction | > 4/5 stars | GitHub discussions |

## References

- [LM Studio Blog: FunctionGemma + Unsloth](https://lmstudio.ai/blog/functiongemma-unsloth)
- [Unsloth Colab Notebook](https://colab.research.google.com/github/unslothai/notebooks/blob/main/nb/FunctionGemma_(270M)-LMStudio.ipynb)
- [FunctionGemma on Hugging Face](https://huggingface.co/unsloth/functiongemma-270m-it)
- [LM Studio Developer Docs](https://lmstudio.ai/docs/developer)
- [ADR-001: LLM Inference Architecture](001-llm-inference-architecture.md)
- [Spec 004: Remote Backend Support](../../specs/004-implement-ollama-and/spec.md)

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-04 | Caro Maintainers | Initial draft |
