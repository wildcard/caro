# PRD-001: OLMo 3 Model Support

**Status**: Draft

**Date**: 2026-01-03

**Author**: caro maintainers

**Related ADR**: [ADR-004-olmo3-model-support](../adr/ADR-004-olmo3-model-support.md)

---

## Executive Summary

Add OLMo 3, Allen AI's latest open-source language model family, to caro's model catalog. OLMo 3 offers enhanced reasoning capabilities through its "Think" variant, making it ideal for complex shell command generation that requires multi-step thinking.

## Problem Statement

### Current State
caro supports various models (SmolLM 135M to Mistral 7B), but lacks models optimized for:
- Chain-of-thought reasoning for complex commands
- Explaining command logic step-by-step
- Handling multi-step shell pipelines

### User Pain Points
1. Complex commands sometimes lack clear reasoning
2. Multi-step pipelines can be error-prone without explanation
3. Users want transparency in how commands are generated

### Opportunity
OLMo 3's "Think" variant provides built-in chain-of-thought reasoning, enabling caro to:
- Generate more reliable complex commands
- Explain its reasoning to users
- Improve user trust and command accuracy

## Goals & Success Metrics

### Primary Goals
| Goal | Metric | Target |
|------|--------|--------|
| Improve complex command quality | Accuracy on 50-command test suite | ≥95% |
| Enable reasoning explanations | User can request "explain this command" | 100% supported |
| Maintain performance | Inference time for 7B model | <5s first inference |

### Non-Goals
- Replacing the default Qwen 1.5B model (evaluation only)
- Supporting 32B model in default catalog (too large)
- CI/CD usage (model too large)

## Target Users

### Primary Persona: Power User
- Uses caro daily for complex system administration
- Wants detailed explanations for unfamiliar commands
- Values accuracy over speed

### Secondary Persona: Developer
- Uses caro for code-related shell tasks
- Appreciates OLMo 3's strong coding benchmarks
- May use Think variant for complex git/build commands

## Feature Requirements

### P0 (Must Have)

| ID | Feature | Description | Acceptance Criteria |
|----|---------|-------------|---------------------|
| F1 | OLMo 3 7B Instruct support | Add model to catalog | Model loads and generates commands |
| F2 | Ollama backend integration | Support `ollama run olmo3` | Commands execute successfully |
| F3 | Model selection via config | `CARO_MODEL=olmo3-7b-instruct` | Model switches correctly |
| F4 | Documentation update | Add to MODEL_CATALOG.md | All fields documented |

### P1 (Should Have)

| ID | Feature | Description | Acceptance Criteria |
|----|---------|-------------|---------------------|
| F5 | OLMo 3 7B Think variant | Reasoning-enhanced model | Chain-of-thought visible in verbose mode |
| F6 | Explain command mode | `caro --explain "command"` | Returns reasoning steps |
| F7 | Model comparison tests | Benchmark vs current models | Results documented |

### P2 (Nice to Have)

| ID | Feature | Description | Acceptance Criteria |
|----|---------|-------------|---------------------|
| F8 | GGUF direct download | Native GGUF support when available | Downloads from HuggingFace |
| F9 | 32B documentation | Guide for power users | README with requirements |
| F10 | Auto-model selection | Choose Think for complex prompts | Heuristic-based switching |

## Technical Specifications

### Model Details

| Variant | Size | Context | Best For |
|---------|------|---------|----------|
| OLMo 3 7B Instruct | 4.5GB | 64K | Standard command generation |
| OLMo 3 7B Think | 4.5GB | 64K | Complex reasoning tasks |
| OLMo 3 32B Instruct | 19GB | 64K | Maximum quality (docs only) |
| OLMo 3 32B Think | 19GB | 64K | Maximum reasoning (docs only) |

### Integration Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      caro CLI                           │
├─────────────────────────────────────────────────────────┤
│                   Model Catalog                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │ Qwen 1.5B   │  │ Mistral 7B  │  │ OLMo 3 7B       │ │
│  │ (default)   │  │             │  │ Instruct/Think  │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────┤
│                   Backend Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │ GGUF/llama  │  │ MLX         │  │ Ollama          │ │
│  │             │  │ (Apple Si)  │  │ (OLMo 3)        │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Code Changes

```rust
// src/model_catalog.rs - Add new model entries

/// OLMo 3 7B Instruct - Reasoning-focused model
pub static OLMO3_7B_INSTRUCT: ModelInfo = ModelInfo {
    id: "olmo3-7b-instruct",
    name: "OLMo 3 7B Instruct",
    hf_repo: "allenai/OLMo-3-7B-Instruct",  // TBD
    filename: "olmo3-7b-instruct.gguf",      // TBD
    size_mb: 4500,
    size_category: ModelSize::Large,
    description: "Allen AI reasoning model, good for complex commands",
    mlx_optimized: false,
    ci_suitable: false,
};

/// OLMo 3 7B Think - Chain-of-thought reasoning
pub static OLMO3_7B_THINK: ModelInfo = ModelInfo {
    id: "olmo3-7b-think",
    name: "OLMo 3 7B Think",
    hf_repo: "allenai/OLMo-3-7B-Think",  // TBD
    filename: "olmo3-7b-think.gguf",      // TBD
    size_mb: 4500,
    size_category: ModelSize::Large,
    description: "Chain-of-thought reasoning for complex shell tasks",
    mlx_optimized: false,
    ci_suitable: false,
};
```

## User Experience

### Use Case 1: Standard Command Generation
```bash
# User switches to OLMo 3
export CARO_MODEL=olmo3-7b-instruct

# Normal usage
$ caro "find all files modified in the last hour"
find . -type f -mmin -60
```

### Use Case 2: Complex Reasoning (Think Variant)
```bash
export CARO_MODEL=olmo3-7b-think

$ caro "set up a cron job to backup my database every night at 2am"
# Output includes reasoning steps in verbose mode:
# Thinking: User wants a scheduled backup...
# Step 1: Determine cron syntax for 2am...
# Step 2: Construct backup command...

0 2 * * * /usr/bin/pg_dump mydb > /backup/mydb_$(date +\%Y\%m\%d).sql
```

### Use Case 3: Explain Mode
```bash
$ caro --explain "0 2 * * * /usr/bin/pg_dump mydb > /backup/mydb.sql"

This cron job:
1. Runs at 2:00 AM every day (0 2 * * *)
2. Uses pg_dump to export the PostgreSQL database 'mydb'
3. Saves the output to /backup/mydb.sql
4. Overwrites the previous backup each time
```

## Dependencies & Risks

### Dependencies
| Dependency | Owner | Risk Level |
|------------|-------|------------|
| Ollama availability | External | Low |
| GGUF model availability | Allen AI / Community | Medium |
| HuggingFace hosting | External | Low |

### Risks & Mitigations
| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Model underperforms on shell tasks | High | Medium | Extensive testing before promotion |
| GGUF not available | Medium | Medium | Use Ollama backend initially |
| Model size concerns | Low | Low | Document resource requirements |

## Timeline

| Phase | Deliverables | Duration |
|-------|--------------|----------|
| Phase 1 | Ollama integration, basic 7B Instruct | 1 sprint |
| Phase 2 | Think variant, explain mode | 1 sprint |
| Phase 3 | Benchmarking, documentation | 1 sprint |
| Phase 4 | Community feedback, iteration | Ongoing |

## Appendix

### Benchmark Data (from Ollama)

| Benchmark | OLMo 3 32B-Think | OLMo 3 7B-Think | Qwen 1.5B |
|-----------|------------------|-----------------|-----------|
| MATH | 96.1% | ~80%* | ~40%* |
| HumanEvalPlus | 91.4% | ~70%* | ~50%* |
| BigBenchHard | 89.8% | ~65%* | ~35%* |
| MMLU | 85.4% | ~60%* | ~45%* |

*Estimated based on scaling; actual 7B benchmarks pending

### Related Documents
- [ADR-004: OLMo 3 Model Support](../adr/ADR-004-olmo3-model-support.md)
- [Model Catalog Documentation](../MODEL_CATALOG.md)
- [OLMo 3 on Ollama](https://ollama.com/library/olmo-3)
