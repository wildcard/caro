# Feature Specification: FunctionGemma + LM Studio Backend Integration

**Feature Branch**: `008-functiongemma-lmstudio-backend`
**Created**: 2026-01-04
**Status**: Draft
**Input**: Fine-tune FunctionGemma for custom tool calls, convert to GGUF, serve via LM Studio

## Executive Summary

Integrate Google's FunctionGemma (270M parameters) as a fine-tunable, locally-served backend for caro. This enables users to fine-tune a purpose-built function-calling model on their own data using Unsloth, convert it to GGUF format, and serve it locally via LM Studio's OpenAI-compatible API.

---

## Quick Guidelines
- Focus on WHAT users need and WHY
- Avoid HOW to implement (implementation details in plan.md)
- Written for business stakeholders and contributors

---

## Problem Statement

### Current Limitations

1. **Generic Models**: Current embedded models (Qwen 2.5 Coder) are general-purpose code models, not optimized specifically for shell command generation
2. **No Customization Path**: Users cannot fine-tune models on their specific workflows or organizational command patterns
3. **Model Size Trade-offs**: Larger models provide better quality but slower inference; smaller models sacrifice accuracy

### Opportunity

FunctionGemma addresses these limitations:
- **Purpose-Built**: Designed specifically for function/tool calling (exactly what caro does)
- **Tiny but Capable**: 270M parameters - fast inference on any hardware
- **Fine-Tunable**: Unsloth enables 2x faster fine-tuning with reduced VRAM
- **Local Serving**: LM Studio provides zero-config OpenAI-compatible API

---

## User Scenarios & Testing

### Primary User Stories

**Story 1: Power User Fine-Tuning**
As a DevOps engineer, I want to fine-tune FunctionGemma on my organization's shell command patterns so that caro generates commands that match our infrastructure conventions.

**Story 2: Privacy-Conscious Local Serving**
As a security-conscious developer, I want to run a fine-tuned model locally via LM Studio so that my prompts never leave my machine while getting customized results.

**Story 3: Resource-Constrained Environment**
As a developer with limited hardware, I want to use a 270M parameter model that runs fast on CPU so that I can use caro without a GPU.

### Acceptance Scenarios

1. **Given** LM Studio is running with FunctionGemma loaded, **When** user runs `caro "list docker containers"`, **Then** caro connects to LM Studio API and returns a generated command

2. **Given** user has fine-tuned FunctionGemma on custom data, **When** user imports the GGUF into LM Studio, **Then** the custom model is available for caro to use

3. **Given** LM Studio is not running, **When** user attempts to use LM Studio backend, **Then** caro falls back to embedded model with clear notification

4. **Given** user configures LM Studio backend in `~/.caro/config.toml`, **When** LM Studio is available, **Then** caro automatically uses it as preferred backend

5. **Given** user runs `caro --backend lmstudio`, **When** LM Studio returns a command, **Then** response includes function-call formatted output parsed correctly

### Edge Cases

- What happens when LM Studio is running but no model is loaded?
- How does caro handle LM Studio's streaming responses?
- What error messages appear when the model returns non-JSON responses?
- How are tool-calling responses parsed vs. plain text responses?
- What happens when LM Studio runs on a non-default port?

---

## Requirements

### Functional Requirements

**Backend Integration**

- **FR-001**: System MUST support connecting to LM Studio's OpenAI-compatible API at configurable endpoint (default: `http://localhost:1234/v1`)
- **FR-002**: System MUST detect LM Studio availability via `/v1/models` endpoint health check
- **FR-003**: System MUST support both `/v1/completions` and `/v1/chat/completions` endpoints
- **FR-004**: System MUST parse FunctionGemma's function-calling response format
- **FR-005**: System MUST fall back to embedded backend when LM Studio unavailable

**Configuration**

- **FR-006**: Users MUST be able to configure LM Studio endpoint via `~/.caro/config.toml`
- **FR-007**: Users MUST be able to specify LM Studio backend via CLI flag `--backend lmstudio`
- **FR-008**: System MUST support custom model identifier configuration for LM Studio
- **FR-009**: System MUST provide sensible defaults (localhost:1234, no auth required)

**Fine-Tuning Support**

- **FR-010**: Documentation MUST include step-by-step guide for fine-tuning FunctionGemma with Unsloth
- **FR-011**: Documentation MUST include GGUF conversion instructions (Q8_0, Q4_K_M quantization options)
- **FR-012**: Documentation MUST include LM Studio import and serving instructions
- **FR-013**: System MUST provide example training dataset format for shell command generation

**Response Handling**

- **FR-014**: System MUST parse FunctionGemma's tool-call JSON format: `{"name": "function_name", "arguments": {...}}`
- **FR-015**: System MUST handle both streaming and non-streaming responses
- **FR-016**: System MUST validate and sanitize commands extracted from tool-call responses
- **FR-017**: System MUST apply safety validation to all commands regardless of source

**Error Handling**

- **FR-018**: System MUST detect and report LM Studio connection failures with actionable messages
- **FR-019**: System MUST handle timeout scenarios (default: 30 seconds)
- **FR-020**: System MUST provide diagnostic info when model returns malformed responses
- **FR-021**: System MUST gracefully handle "no model loaded" state in LM Studio

### Non-Functional Requirements

**Performance**

- **NFR-001**: LM Studio backend availability check MUST complete within 500ms
- **NFR-002**: Command generation via LM Studio MUST complete within 10 seconds for 270M model
- **NFR-003**: System MUST cache LM Studio availability status for 60 seconds

**Usability**

- **NFR-004**: Error messages MUST include specific troubleshooting steps (e.g., "Run `lms server start`")
- **NFR-005**: Backend status MUST be visible in verbose mode output
- **NFR-006**: Fine-tuning documentation MUST be completable by intermediate developers in under 2 hours

**Compatibility**

- **NFR-007**: System MUST work with LM Studio versions 0.2.0 and above
- **NFR-008**: System MUST support GGUF models quantized with Q4_K_M, Q8_0, and F16
- **NFR-009**: System MUST work on macOS, Linux, and Windows where LM Studio is available

---

## Key Entities

### LM Studio Backend
Represents the connection to LM Studio's local inference server. Includes endpoint URL, model identifier, connection status, and response parser for function-calling format.

### FunctionGemma Model
Google's 270M parameter model designed for function/tool calling. Trained on function-calling datasets, outputs structured JSON for tool invocations.

### Fine-Tuning Pipeline
The workflow for customizing FunctionGemma using Unsloth:
1. Prepare training data (shell command examples)
2. Run Unsloth fine-tuning (LoRA adapters)
3. Merge and convert to GGUF
4. Import into LM Studio

### Training Dataset
Collection of shell command generation examples in the format:
- Input: Natural language description
- Output: Shell command with explanation
- Tools: Available shell utilities/commands

### GGUF Model Artifact
Quantized model file compatible with llama.cpp and LM Studio. Supports various quantization levels (Q4_K_M for size, Q8_0 for quality, F16 for full precision).

---

## Integration Architecture

```
                    Fine-Tuning Pipeline (One-Time)
                    ================================

  Training Data    →    Unsloth    →    GGUF    →    LM Studio
  (JSON examples)       (Colab)        Export        Import


                    Runtime Inference
                    =================

  User Prompt → caro CLI → LM Studio API → FunctionGemma → Command
                   ↓              ↓
            (fallback)    OpenAI-compatible
                   ↓         /v1/chat/completions
           Embedded Model         ↓
                          Tool-call JSON response
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Inference latency | < 2s on M1 Mac | Time from prompt to command |
| Fine-tuning time | < 1 hour (500 steps) | Colab notebook runtime |
| Model size (Q4_K_M) | < 200MB | GGUF file size |
| Command accuracy | > 90% on test set | Correct command generation |
| User adoption | 100+ fine-tuned models | Community reports |

---

## Dependencies & Assumptions

### Dependencies
- LM Studio desktop application (user-installed)
- Google Colab (for fine-tuning, free tier sufficient)
- Unsloth library (open source, MIT license)
- Existing caro remote backend infrastructure (FR-001 through FR-005 leverage existing patterns)

### Assumptions
- Users have basic familiarity with running local LLM servers
- LM Studio maintains OpenAI-compatible API stability
- FunctionGemma remains available on Hugging Face
- Unsloth maintains compatibility with FunctionGemma architecture

### Out of Scope
- Built-in fine-tuning UI within caro
- Automatic model download/management for LM Studio
- Training data collection from user sessions
- Distributed fine-tuning across multiple machines

---

## Competitive Advantage

| Feature | caro + FunctionGemma | Competitors |
|---------|---------------------|-------------|
| Model size | 270M (tiny) | 1B-7B typical |
| Fine-tuning | 1hr on free Colab | Often requires paid GPU |
| Function calling | Native support | Prompt engineering |
| Local serving | LM Studio (free) | Often requires paid APIs |
| Customization | Full control | Limited/none |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| LM Studio API changes | Low | Medium | Version pin, fallback to embedded |
| FunctionGemma deprecated | Low | High | Document alternative models (Phi-3, Gemma 2) |
| Unsloth compatibility breaks | Medium | Medium | Pin versions in documentation |
| Users struggle with fine-tuning | Medium | Medium | Provide pre-made training datasets |

---

## Timeline Recommendation

**Phase 1: LM Studio Backend** (Core integration)
- Implement LmStudioBackend following existing remote backend patterns
- Add configuration support
- Write integration tests

**Phase 2: Documentation** (Enable fine-tuning)
- Create fine-tuning guide with Unsloth
- Provide example training dataset
- Document GGUF conversion and import

**Phase 3: Community** (Adoption)
- Publish pre-fine-tuned model on Hugging Face
- Create video tutorial
- Gather community feedback

---

## Review & Acceptance Checklist

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## References

- [LM Studio Blog: FunctionGemma + Unsloth](https://lmstudio.ai/blog/functiongemma-unsloth)
- [Unsloth Colab Notebook](https://colab.research.google.com/github/unslothai/notebooks/blob/main/nb/FunctionGemma_(270M)-LMStudio.ipynb)
- [FunctionGemma on Hugging Face](https://huggingface.co/unsloth/functiongemma-270m-it)
- [LM Studio Developer Docs](https://lmstudio.ai/docs/developer)
- [caro ADR-001: LLM Inference Architecture](../../docs/adr/001-llm-inference-architecture.md)

---

**Next Steps**: Run `/plan` to generate implementation plan, then `/implement` for task execution.
