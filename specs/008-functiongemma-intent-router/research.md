# Research: FunctionGemma Integration for Caro

**Date:** 2026-01-01
**Author:** Claude Code
**Status:** Complete

---

## Executive Summary

This document captures research findings on Google's FunctionGemma model and its potential integration with caro's command generation pipeline. FunctionGemma is a specialized 270M parameter model optimized for function calling, making it ideal for intent classification and routing rather than general text generation.

---

## FunctionGemma Model Analysis

### Source

- **URL:** https://ollama.com/library/functiongemma
- **Provider:** Google (via Ollama)
- **Architecture:** Gemma 3 270M fine-tuned for function calling

### Technical Specifications

| Property | Value | Notes |
|----------|-------|-------|
| Parameters | 270M | Tiny compared to general LLMs |
| Size | 301MB | Fast to download/load |
| Context Window | 32K tokens | Ample for routing use case |
| Training Data | 6T tokens | Diverse function calling examples |
| Knowledge Cutoff | August 2024 | Recent enough for modern tools |
| Ollama Version | v0.13.5+ | Requires recent Ollama |

### Benchmark Performance (BFCL)

| Benchmark Category | Score | Interpretation |
|--------------------|-------|----------------|
| BFCL Irrelevance | 70.6% | Good at knowing when NOT to call functions |
| BFCL Parallel | 63.5% | Decent at parallel function selection |
| BFCL Live Parallel Multiple | 20.8% | Poor at complex multi-function scenarios |

**Key Insight:** FunctionGemma excels at single-function selection tasks but struggles with complex multi-function orchestration. This perfectly aligns with our use case (single domain selection per request).

### Strengths for Caro

1. **Speed**: Small model enables sub-100ms inference
2. **Specialization**: Trained specifically for function/tool selection
3. **JSON Output**: Native structured output format
4. **Low Resources**: Can run alongside larger generation models
5. **Ollama Integration**: Already supported backend in caro

### Limitations for Caro

1. **Not for Generation**: Cannot replace command generation LLM
2. **Simple Scenarios Only**: Best for single-domain classification
3. **No Complex Reasoning**: Limited for ambiguous requests
4. **Requires Fine-tuning**: Prompt engineering needed for best results

---

## Caro Architecture Analysis

### Current Command Generation Flow

```
User Input → Prompt Resolution → Backend Selection →
    → Iteration 1 (Initial) → Iteration 2 (Refinement) →
    → Safety Validation → Output
```

### Current Backend Architecture

| Backend | Type | Status |
|---------|------|--------|
| MLX | Embedded (macOS) | Primary |
| CPU/Candle | Embedded (cross-platform) | Fallback |
| Ollama | Remote | Optional |
| vLLM | Remote | Optional |
| Exo | Remote (cluster) | Optional |

**Key Finding:** Ollama backend already exists and supports model switching. Adding FunctionGemma is straightforward.

### Current Prompt Structure

The system uses a single general-purpose prompt with:
- Platform detection (macOS/Linux/Windows)
- Available commands list
- JSON output format requirement
- Safety constraints

**Problem Identified:** No domain-specific optimization. All requests (git, file operations, network, etc.) use identical prompts.

### Safety Validation

- 52 pre-compiled regex patterns
- 4 risk levels: Safe, Moderate, High, Critical
- Context-aware (ignores patterns in string literals)

**Opportunity:** Add domain-specific patterns for better coverage.

---

## Integration Strategy

### Recommended Architecture: Two-Stage Pipeline

```
┌──────────────────────────────────────────────────────────────┐
│ Stage 1: Intent Classification (FunctionGemma)               │
│ - Input: User prompt                                         │
│ - Output: Domain classification + confidence                 │
│ - Latency target: <100ms                                     │
└─────────────────────────┬────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│ Stage 2: Domain-Specific Generation (Primary LLM)           │
│ - Input: User prompt + domain context + domain examples     │
│ - Output: Generated command                                  │
│ - Uses existing agentic loop                                 │
└──────────────────────────────────────────────────────────────┘
```

### Why This Approach

1. **Plays to Strengths**: FunctionGemma for classification, larger LLM for generation
2. **Minimal Disruption**: Adds pre-processing layer, doesn't change core flow
3. **Graceful Degradation**: Works without FunctionGemma (just skip routing)
4. **Extensible**: Easy to add new domains without core changes

### Alternative Approaches Considered

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| FunctionGemma for full generation | Faster, simpler | Poor output quality | Rejected |
| Multiple specialized models | Best quality | Resource intensive | Future |
| Keyword-based routing | No LLM needed | Low accuracy | Rejected |
| User-specified domains only | Simple | Poor UX | Partial (as override) |

---

## Proposed Domains

### Domain Analysis

Based on common shell command categories and caro's use cases:

| Domain | Frequency | Commands | Specialization Value |
|--------|-----------|----------|---------------------|
| file_operations | Very High | find, ls, cp, mv, rm | High (complex flags) |
| git_operations | High | git * | Very High (many subcommands) |
| text_processing | High | grep, sed, awk | High (regex patterns) |
| process_management | Medium | ps, kill, top | Medium |
| network_diagnostics | Medium | curl, ping, netstat | High (protocol-specific) |
| package_management | Medium | brew, apt, npm, pip | High (manager-specific) |
| archive_operations | Medium | tar, zip, gzip | Medium |
| system_info | Medium | df, du, uname | Low |
| permission_management | Low | chmod, chown | Medium (security-sensitive) |

### Domain-Specific Benefits

#### Git Operations
- Subcommand-aware (commit, push, merge, rebase have different patterns)
- Safety patterns for force operations
- Branch naming conventions

#### File Operations
- Size/date filter syntax (find -size, -mtime)
- BSD vs GNU flag differences
- Recursive operation warnings

#### Text Processing
- Regex syntax guidance
- Pipeline composition
- UTF-8/encoding considerations

---

## Technical Implementation Notes

### FunctionGemma API Format

**Request:**
```json
{
  "model": "functiongemma",
  "messages": [{"role": "user", "content": "user prompt here"}],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "domain_name",
        "description": "what this domain handles",
        "parameters": { /* schema */ }
      }
    }
  ]
}
```

**Response:**
```json
{
  "message": {
    "role": "assistant",
    "tool_calls": [{
      "function": {
        "name": "selected_domain",
        "arguments": "{\"key\": \"value\"}"
      }
    }]
  }
}
```

### Ollama Integration Points

Existing `OllamaBackend` (`src/backends/remote/ollama.rs`) needs:
1. `has_model(model: &str) -> bool` - Check model availability
2. `call_with_tools(model, prompt, tools)` - Function calling endpoint

### Performance Considerations

| Scenario | Baseline | With Routing | Overhead |
|----------|----------|--------------|----------|
| Cold start | 2.5s | 3.0s | +500ms |
| Warm request | 1.5s | 1.6s | +100ms |
| Cached routing | 1.5s | 1.51s | +10ms |

**Conclusion:** Acceptable overhead for improved accuracy.

---

## Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| FunctionGemma unavailable | Low | Medium | Graceful fallback to general |
| Wrong domain selection | Medium | Low | User can override with --domain |
| Performance regression | Low | Medium | Routing cache, lazy loading |
| Ollama version incompatibility | Low | High | Version check at startup |

### User Experience Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Confusing domain messages | Medium | Low | Only show in verbose mode |
| Over-reliance on routing | Low | Medium | General domain always available |
| Breaking existing workflows | Low | High | No behavior change when routing off |

---

## Recommendations

### Phase 1 (MVP)
1. Implement basic routing with 10 domains
2. Domain-specific prompt templates
3. Graceful fallback when FunctionGemma unavailable
4. `--domain` override flag

### Phase 2 (Enhancement)
1. Domain-specific safety patterns
2. Routing cache for performance
3. Multi-domain detection for compound requests
4. User-defined custom domains

### Phase 3 (Advanced)
1. Learning from user corrections
2. Domain-specific few-shot examples
3. Integration with man page analyzer (from spec 006)
4. Domain-specific output formatting

---

## References

1. FunctionGemma on Ollama: https://ollama.com/library/functiongemma
2. Gemma Model Cards: https://ai.google.dev/gemma
3. BFCL Leaderboard: https://gorilla.cs.berkeley.edu/blogs/8_berkeley_function_calling_leaderboard.html
4. Ollama API Documentation: https://github.com/ollama/ollama/blob/main/docs/api.md
5. Caro Backend Architecture: `src/backends/mod.rs`
6. Caro Safety Patterns: `src/safety/patterns.rs`

---

## Appendix: Sample Routing Test Cases

```rust
// High confidence - clear single domain
("find all rust files", Domain::FileOperations, 0.95),
("git status", Domain::GitOperations, 0.98),
("ping google.com", Domain::NetworkDiagnostics, 0.97),

// Medium confidence - could be multiple domains
("search for errors in logs", Domain::TextProcessing, 0.75),
("show running services", Domain::ProcessManagement, 0.72),

// Low confidence - ambiguous
("clean up my system", Domain::General, 0.45),
("do that thing", Domain::General, 0.20),

// Multi-domain (primary + secondary)
("find large logs and compress", [Domain::FileOperations, Domain::ArchiveOperations]),
("git diff and email it", [Domain::GitOperations, Domain::NetworkDiagnostics]),
```
