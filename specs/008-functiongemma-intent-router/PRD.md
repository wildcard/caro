# Product Requirements Document: FunctionGemma Intent Router

**Document Version:** 1.0
**Date:** January 2026
**Status:** Draft
**Authors:** Caro Product Team

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Problem Statement](#problem-statement)
3. [Product Vision](#product-vision)
4. [Core Philosophy](#core-philosophy)
5. [User Personas](#user-personas)
6. [Requirements](#requirements)
7. [Performance Requirements](#performance-requirements)
8. [User Experience](#user-experience)
9. [Success Metrics](#success-metrics)
10. [Roadmap](#roadmap)
11. [Dependencies](#dependencies)
12. [Appendices](#appendices)

---

## Executive Summary

### The Opportunity

Caro converts natural language to shell commands. Today, every request uses the same general-purpose approach regardless of whether you're asking about git, file operations, or network diagnostics. This is like using a Swiss Army knife when you have access to specialized tools.

### The Solution

Introduce **FunctionGemma**, a lightweight intent classification model, as the first stage of a parallelized pre-processing pipeline. This enables:

1. **Domain-aware generation**: Specialized prompts for different command types
2. **Faster responses**: Parallel pre-processing, lazy rule evaluation
3. **Smarter safety**: Domain-specific danger patterns
4. **One-shot success**: Better preparation means fewer retries

### The Impact

| Metric | Before | After |
|--------|--------|-------|
| Response time (p95) | 2.5s | 1.8s |
| First-shot accuracy | 60% | 80% |
| Safety coverage | 52 patterns | 80+ patterns |
| Context efficiency | 4K tokens | 2K tokens |

---

## Problem Statement

### Current Pain Points

#### 1. One-Size-Fits-All Generation

**Problem**: All requests use identical prompts regardless of domain.

**Impact**:
- Git commands generated with file operation examples
- Network diagnostics missing protocol-specific guidance
- Package management ignoring manager-specific syntax

**User Quote**: *"It keeps suggesting `apt install` on my Mac. Every time."*

#### 2. Slow Response Times

**Problem**: Sequential processing adds latency.

**Impact**:
- Users wait 2-3 seconds for simple commands
- Feels sluggish compared to actually typing commands
- Power users revert to manual command writing

**User Quote**: *"By the time it responds, I've already written the command myself."*

#### 3. Generic Safety Checks

**Problem**: Safety validation runs all 52 patterns for every command.

**Impact**:
- Git-specific dangers (force push) not caught
- File operations over-flagged on false positives
- Network commands missing privilege checks

**User Quote**: *"It warned me about `rm` in a Python string but didn't catch `git push --force`."*

#### 4. Context Bloat

**Problem**: Prompts include examples and rules from all domains.

**Impact**:
- Larger context → slower inference
- Irrelevant examples confuse the model
- Wasted tokens on inapplicable rules

---

## Product Vision

### The Three-Phase Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CARO COMMAND GENERATION PIPELINE                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  PHASE 1: PRE-PROCESSING (Parallelized, <200ms)                 │   │
│   │                                                                  │   │
│   │  "Prepare everything the coder model needs"                     │   │
│   │                                                                  │   │
│   │  • Intent classification (What domain?)                         │   │
│   │  • Context gathering (What platform? What tools?)               │   │
│   │  • Early safety (Any obvious dangers?)                          │   │
│   │  • Rule selection (Which patterns apply?)                       │   │
│   │                                                                  │   │
│   │  ► Can EXIT EARLY: Block, cache hit, clarification needed      │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                     │
│                                    ▼                                     │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  PHASE 2: INFERENCE (Protected, ~1500ms)                        │   │
│   │                                                                  │   │
│   │  "Hit the coder model ONCE with maximum context"                │   │
│   │                                                                  │   │
│   │  • Domain-specific prompt                                       │   │
│   │  • Relevant examples only                                       │   │
│   │  • Platform-aware constraints                                   │   │
│   │  • One-shot goal: get it right the first time                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                     │
│                                    ▼                                     │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  PHASE 3: POST-PROCESSING (Minimal, <50ms)                      │   │
│   │                                                                  │   │
│   │  "Validate and format—if we're here often, we failed earlier"  │   │
│   │                                                                  │   │
│   │  • Final safety validation                                      │   │
│   │  • Output formatting                                            │   │
│   │  • Cache population                                             │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Vision Statement

> **Caro should be faster to use than remembering the command yourself.**

This means:
- Response in under 2 seconds
- Right command on the first try
- Smart enough to know when to ask vs. guess
- Safe enough to trust

---

## Core Philosophy

### Principle 1: Speed is a Feature

**Nobody wants a slow terminal assistant.**

Every design decision must answer: *"Does this make us faster?"*

- Parallelize everything parallelizable
- Cache aggressively
- Break early when possible
- Lazy-load what we don't need

**Metric**: End-to-end latency < 1.8s (p95)

### Principle 2: Protect the Inference

**The decoder model is the bottleneck.**

The coder model (Qwen2.5-Coder) takes ~1.5s per call. Our job is to:

- Prepare maximum context before hitting it
- Hit it once with a high-quality prompt
- Avoid round-trips at all costs

**Metric**: One-shot success rate > 80%

### Principle 3: Good Programs Are Lazy

**Break early, load late, do only what's needed.**

- If command is dangerous → block immediately, don't generate
- If response is cached → return immediately, don't regenerate
- If clarification needed → ask immediately, don't guess
- If domain is known → load only that domain's rules

**Metric**: Early exit rate > 15%

### Principle 4: Domain Awareness Wins

**Knowing the domain unlocks optimizations everywhere.**

- Prompts: Domain-specific examples and constraints
- Safety: Domain-specific danger patterns
- Context: Domain-relevant tools and flags
- Caching: Domain-aware cache keys

**Metric**: Domain classification accuracy > 95%

---

## User Personas

### Primary Persona: The Pragmatic Developer

**Name**: Alex
**Role**: Full-stack developer
**Experience**: 5+ years
**Platform**: macOS (Apple Silicon)

**Goals**:
- Get working commands quickly
- Trust the safety checks
- Not have to remember obscure flags

**Frustrations**:
- Waiting more than a couple seconds
- Commands that don't work on Mac
- Safety warnings for safe commands

**Quote**: *"I know what I want to do, I just can't remember the exact syntax."*

### Secondary Persona: The DevOps Engineer

**Name**: Sam
**Role**: Site Reliability Engineer
**Experience**: 8+ years
**Platform**: Linux (remote servers)

**Goals**:
- Generate commands for remote systems
- Higher confidence in safety checks
- Audit trail for generated commands

**Frustrations**:
- Platform mismatches (Mac → Linux)
- Missing domain-specific warnings
- No explanation of why command was chosen

**Quote**: *"I need to trust this before I run it on prod."*

### Tertiary Persona: The Command-Line Learner

**Name**: Jordan
**Role**: Junior developer
**Experience**: 1 year
**Platform**: Various

**Goals**:
- Learn proper command syntax
- Understand what commands do
- Build command-line confidence

**Frustrations**:
- Not knowing which tool to use
- Confusing error messages
- No context for why one approach vs another

**Quote**: *"I don't even know what command to search for."*

---

## Requirements

### Functional Requirements

#### FR-001: Intent Classification

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-001.1 | System MUST classify user intent into one of 10 predefined domains | P0 |
| FR-001.2 | System MUST return confidence score (0.0-1.0) with classification | P0 |
| FR-001.3 | System MUST support multiple domain detection for compound requests | P1 |
| FR-001.4 | System MUST cache routing decisions for identical inputs | P1 |
| FR-001.5 | System MUST support manual domain override via `--domain` flag | P0 |

#### FR-002: Parallel Pre-Processing

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-002.1 | System MUST run intent classification, context scanning, and early safety checks in parallel | P0 |
| FR-002.2 | System MUST complete pre-processing within 200ms (p95) | P0 |
| FR-002.3 | System MUST use Tokio for async parallelization | P0 |
| FR-002.4 | System MUST gracefully degrade if any parallel task fails | P0 |

#### FR-003: Early Exit Gates

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-003.1 | System MUST block critical dangerous patterns before inference | P0 |
| FR-003.2 | System MUST return cached responses without regeneration | P1 |
| FR-003.3 | System MUST request clarification for low-confidence intents (<0.4) | P1 |
| FR-003.4 | System MUST skip inference for meta-commands (help, list domains) | P2 |

#### FR-004: Domain-Specific Context

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-004.1 | System MUST load domain-specific prompt templates | P0 |
| FR-004.2 | System MUST select only domain-relevant safety patterns | P0 |
| FR-004.3 | System MUST include domain-specific examples in prompts | P1 |
| FR-004.4 | System MUST apply domain-specific platform rules | P1 |

#### FR-005: Domain Safety Patterns

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-005.1 | System MUST define safety patterns for each domain | P0 |
| FR-005.2 | Git domain MUST detect force-push and history-rewriting operations | P0 |
| FR-005.3 | File domain MUST detect recursive deletion patterns | P0 |
| FR-005.4 | Network domain MUST detect backdoor and privilege patterns | P0 |
| FR-005.5 | Package domain MUST detect system-breaking operations | P1 |

### Non-Functional Requirements

#### NFR-001: Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-001.1 | End-to-end latency (p50) | < 1.5s |
| NFR-001.2 | End-to-end latency (p95) | < 1.8s |
| NFR-001.3 | Pre-processing overhead | < 200ms |
| NFR-001.4 | Intent classification (warm) | < 100ms |
| NFR-001.5 | Cache lookup | < 5ms |

#### NFR-002: Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-002.1 | Intent classification accuracy | > 95% |
| NFR-002.2 | First-shot success rate | > 80% |
| NFR-002.3 | Safety pattern coverage | > 80 patterns |
| NFR-002.4 | Graceful degradation rate | 100% |

#### NFR-003: Resource Usage

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-003.1 | Memory overhead (pre-processing) | < 50MB |
| NFR-003.2 | FunctionGemma model size | ~300MB |
| NFR-003.3 | Cache size limit | Configurable |

---

## Performance Requirements

### Latency Budget

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       LATENCY BUDGET (1800ms total)                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  PRE-PROCESSING: 200ms                                          │   │
│  │  ├── Intent Router (FunctionGemma): 100ms                       │   │
│  │  ├── Context Scanner: 50ms (parallel)                           │   │
│  │  ├── Early Safety Check: 10ms (parallel)                        │   │
│  │  └── Domain Loading: 40ms                                       │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  INFERENCE: 1500ms                                               │   │
│  │  ├── Prompt construction: 10ms                                  │   │
│  │  ├── Model inference: 1450ms                                    │   │
│  │  └── Response parsing: 40ms                                     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  POST-PROCESSING: 100ms                                          │   │
│  │  ├── Safety validation: 50ms                                    │   │
│  │  ├── Output formatting: 20ms                                    │   │
│  │  └── Cache update: 30ms                                         │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Early Exit Scenarios

| Scenario | Time | Savings |
|----------|------|---------|
| Critical safety block | 15ms | 1785ms (99%) |
| Cache hit | 5ms | 1795ms (99.7%) |
| Clarification prompt | 50ms | 1750ms (97%) |
| Normal flow | 1800ms | 0ms |

### Parallelization Savings

| Approach | Time | Savings |
|----------|------|---------|
| Sequential pre-processing | 260ms | Baseline |
| Parallel pre-processing | 120ms | 140ms (54%) |

---

## User Experience

### Command-Line Interface

#### Normal Flow

```bash
$ caro "find all rust files larger than 1MB"

[Router] Domain: file_operations (0.94)
[Safety] ✓ No dangerous patterns detected

Command: find . -name "*.rs" -size +1M

Execute? (y/n/e[dit]):
```

#### Domain Override

```bash
$ caro --domain git "show recent activity"

[Manual] Domain: git_operations
[Safety] ✓ No dangerous patterns detected

Command: git log --oneline -n 10 --graph

Execute? (y/n/e[dit]):
```

#### Safety Block (Early Exit)

```bash
$ caro "delete everything in root"

[Router] Domain: file_operations (0.91)
[Safety] ✗ CRITICAL: Recursive root deletion detected

This command would delete system files:
  rm -rf /

Suggestion: Be more specific about what to delete.
Example: caro "delete all .log files in ~/logs"
```

#### Clarification (Early Exit)

```bash
$ caro "clean up my system"

[Router] Confidence too low (0.32)

I need more information:

1. What do you want to clean?
   a) Old files
   b) Large files
   c) Cache/temp files
   d) Unused packages

2. Where?
   a) Home directory
   b) Specific folder
   c) System-wide

Your choice:
```

#### Multi-Domain Detection

```bash
$ caro "find large logs and compress them"

[Router] Domains detected:
  Primary: file_operations (0.87)
  Secondary: archive_operations (0.72)

[Safety] ⚠ Warning: Compression will replace original files

Command: find . -name "*.log" -size +100M -exec gzip {} \;

Note: Original files will be replaced with .gz versions.

Execute? (y/n/e[dit]):
```

### Verbose Mode

```bash
$ caro -v "find rust files"

[Pre-Processing] Starting parallel analysis...
  ├─ Intent Router: file_operations (0.94) [87ms]
  ├─ Context Scanner: linux/gnu, 23 commands [34ms]
  └─ Early Safety: passed [8ms]
[Pre-Processing] Completed in 92ms

[Context Assembly]
  ├─ Prompt: file-operations.toml
  ├─ Safety Rules: 12 patterns loaded
  └─ Examples: 5 file operation examples

[Inference] Generating command...
[Inference] Completed in 1423ms

[Post-Processing]
  ├─ Safety Validation: passed (0 warnings)
  └─ Cached for future use

Total time: 1547ms

Command: find . -name "*.rs"
```

### Domain Listing

```bash
$ caro --list-domains

Available Domains:
  file_operations      File and directory manipulation
  git_operations       Git version control
  network_diagnostics  Network testing and information
  process_management   Process control and monitoring
  text_processing      Text search and manipulation
  package_management   Package installation (brew, apt, npm, pip)
  archive_operations   Compression and archiving
  system_info          System information queries
  permission_management File permissions and ownership
  general              General commands (fallback)

Usage: caro --domain <domain> "your request"
```

---

## Success Metrics

### Key Performance Indicators

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| **Latency (p95)** | 2.5s | 1.8s | End-to-end response time |
| **First-Shot Accuracy** | 60% | 80% | Commands that work without modification |
| **Routing Accuracy** | N/A | 95% | Correct domain classification |
| **Early Exit Rate** | 0% | 15% | Requests resolved without inference |
| **Cache Hit Rate** | 0% | 30% | Requests served from cache |
| **Safety Coverage** | 52 | 80+ | Total safety patterns |

### User Satisfaction Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Task Completion Rate | > 90% | Users get working command |
| Trust Score | > 4.0/5 | Post-execution survey |
| Return Usage | > 70% | Users who use caro again within 7 days |
| Domain Accuracy Satisfaction | > 90% | Users who agree with domain choice |

### Operational Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| FunctionGemma Availability | > 99% | Model responds successfully |
| Graceful Degradation | 100% | Fallback when component fails |
| Memory Usage | < 2GB | Peak during inference |
| Cold Start Time | < 3s | First command after launch |

---

## Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Goal**: Basic intent routing with FunctionGemma

**Deliverables**:
- [ ] IntentRouter implementation
- [ ] DomainRegistry with 10 domains
- [ ] FunctionGemma Ollama integration
- [ ] Basic parallel pre-processing
- [ ] `--domain` override flag

**Success Criteria**:
- Routing accuracy > 90%
- Pre-processing < 200ms

### Phase 2: Domain Context (Weeks 3-4)

**Goal**: Domain-specific prompts and examples

**Deliverables**:
- [ ] Domain prompt templates (10 files)
- [ ] Domain-specific examples
- [ ] Context assembly pipeline
- [ ] Integration with existing agentic loop

**Success Criteria**:
- First-shot accuracy improvement > 10%
- Context reduction > 30%

### Phase 3: Safety Enhancement (Weeks 5-6)

**Goal**: Domain-aware safety validation

**Deliverables**:
- [ ] Domain-specific safety patterns (~30 new)
- [ ] Lazy rule evaluation
- [ ] Early safety exit gate
- [ ] Enhanced warning messages

**Success Criteria**:
- Total patterns > 80
- Early block rate > 5%

### Phase 4: Optimization (Weeks 7-8)

**Goal**: Performance tuning and caching

**Deliverables**:
- [ ] Routing cache implementation
- [ ] Response caching
- [ ] Performance benchmarking
- [ ] Memory optimization

**Success Criteria**:
- Latency (p95) < 1.8s
- Cache hit rate > 20%

### Phase 5: Memory Integration (Future)

**Goal**: Learning from user patterns

**Deliverables**:
- [ ] Session memory tracking
- [ ] Tool preference learning
- [ ] Pattern-based routing enhancement
- [ ] Personalized examples

**Success Criteria**:
- Return user accuracy > 85%
- Memory-enhanced routing > 10% faster

---

## Dependencies

### Required

| Dependency | Version | Purpose |
|------------|---------|---------|
| Ollama | v0.13.5+ | FunctionGemma hosting |
| FunctionGemma model | latest | Intent classification |
| Tokio | 1.x | Async parallelization |

### Optional (Graceful Degradation)

| Dependency | Fallback |
|------------|----------|
| FunctionGemma unavailable | Use general-purpose routing |
| Ollama unavailable | Skip intent routing entirely |
| Domain cache miss | Load domain on demand |

### Integration Points

| System | Integration |
|--------|-------------|
| Existing Ollama backend | Extend for function calling |
| Safety validator | Add domain filtering |
| Agentic loop | Inject domain context |
| Configuration | Add router settings |

---

## Appendices

### Appendix A: Domain Definitions

| Domain | Keywords | Commands | Safety Focus |
|--------|----------|----------|--------------|
| file_operations | find, list, copy, move, delete | find, ls, cp, mv, rm | Recursive deletion |
| git_operations | git, commit, push, branch | git | Force operations |
| network_diagnostics | ping, curl, connect, port | ping, curl, netstat | Backdoors |
| process_management | process, kill, running | ps, kill, top | System processes |
| text_processing | search, grep, find text | grep, sed, awk | None specific |
| package_management | install, package, brew | brew, apt, npm, pip | System packages |
| archive_operations | compress, zip, tar | tar, zip, gzip | Overwrite |
| system_info | disk, memory, info | df, du, uname | None specific |
| permission_management | chmod, permission | chmod, chown | Privilege escalation |
| general | (fallback) | (all) | All patterns |

### Appendix B: Safety Pattern Categories

| Category | Count | Domains |
|----------|-------|---------|
| Critical (always block) | 15 | All |
| High (require confirmation) | 25 | Domain-specific |
| Moderate (warn only) | 20 | Domain-specific |
| Low (log only) | 20 | Domain-specific |

### Appendix C: FunctionGemma Tool Schema

```json
{
  "name": "file_operations",
  "description": "File and directory operations",
  "parameters": {
    "type": "object",
    "properties": {
      "operation": {
        "type": "string",
        "enum": ["find", "list", "copy", "move", "remove", "create"]
      },
      "target": {
        "type": "string",
        "description": "Files or directories to operate on"
      }
    },
    "required": ["operation", "target"]
  }
}
```

---

**Document Status**: Draft - Pending Review
**Next Steps**: Technical review → Stakeholder alignment → Implementation kickoff
