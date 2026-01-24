# ADR-013: Pre-Processing Pipeline Architecture

| **Status**     | Proposed                            |
|----------------|-------------------------------------|
| **Date**       | January 2026                        |
| **Authors**    | Caro Maintainers                    |
| **Supersedes** | N/A                                 |
| **Relates To** | ADR-001 (LLM Inference Architecture)|

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [Decision Drivers](#decision-drivers)
4. [Design Philosophy](#design-philosophy)
5. [Architecture Overview](#architecture-overview)
6. [Component Design](#component-design)
7. [Parallelization Strategy](#parallelization-strategy)
8. [Lazy Evaluation & Early Exit](#lazy-evaluation--early-exit)
9. [Memory Model Integration](#memory-model-integration)
10. [Performance Budget](#performance-budget)
11. [Consequences](#consequences)
12. [Future Direction](#future-direction)

---

## Executive Summary

This document establishes the architectural decision for Caro's **Pre-Processing Pipeline**—a parallelized, multi-stage system that prepares context for the main LLM inference. The core insight is that the **decoder model (command generation) is the bottleneck we must protect**. By investing in intelligent pre-processing, we enable one-shot inference with maximum context quality.

**Core Tenets:**
- **Protect the inference**: The coder model should hit once with everything it needs
- **Parallelize pre-processing**: Use Tokio to run independent preparation tasks concurrently
- **Lazy evaluation**: Break early when we can answer without full processing
- **Domain-aware optimization**: Run only relevant rules and load only relevant context
- **Speed is a feature**: Nobody wants a slow terminal assistant

---

## Context and Problem Statement

### The Challenge

Caro's command generation involves multiple concerns:

1. **Intent Understanding**: What is the user trying to accomplish?
2. **Context Gathering**: What platform, tools, and environment are relevant?
3. **Safety Validation**: Is this command potentially dangerous?
4. **Command Generation**: What shell command accomplishes this task?

Currently, these concerns are handled sequentially, with the same general-purpose prompt used for all requests regardless of domain.

### The Insight

**The decoder model is expensive.** Every round-trip to the LLM costs hundreds of milliseconds. Our goal should be to:

1. **Prepare maximum context** before hitting the model
2. **Hit the model once** with a high-quality, domain-specific prompt
3. **Minimize post-processing** because needing it means we failed at pre-processing

### The Opportunity

By introducing a lightweight pre-processing layer (using FunctionGemma for intent classification), we can:

- Route to domain-specific prompts
- Load only relevant context
- Run only applicable safety rules
- Potentially answer before hitting the main model (early exit)

---

## Decision Drivers

### Primary Drivers

1. **Latency**: Users expect sub-2-second response times
2. **Accuracy**: Domain-specific context improves generation quality
3. **Safety**: Domain-aware validation catches more dangerous patterns
4. **Resource Efficiency**: Don't load context or run rules that aren't needed
5. **One-Shot Goal**: Minimize LLM round-trips

### Secondary Drivers

- Extensibility (easy to add new domains and rules)
- Debuggability (clear flow for understanding failures)
- Graceful degradation (works without optional components)

---

## Design Philosophy

### The Three-Phase Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           THE THREE PHASES                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  PHASE 1: PRE-PROCESSING              PHASE 2: INFERENCE                │
│  (Parallelizable)                     (Protected)                        │
│  ┌─────────────────────────┐          ┌──────────────────────┐          │
│  │ • Intent Classification │          │ • Single LLM Call    │          │
│  │ • Context Gathering     │    ──►   │ • Domain-Optimized   │          │
│  │ • Rule Selection        │          │ • One-Shot Goal      │          │
│  │ • Early Exit Check      │          └──────────────────────┘          │
│  └─────────────────────────┘                    │                        │
│          │                                      │                        │
│          ▼                                      ▼                        │
│  ┌─────────────────────────┐          ┌──────────────────────┐          │
│  │ EARLY EXIT (if safe)    │          │  PHASE 3: POST       │          │
│  │ • Block dangerous cmd   │          │  (Minimize This)     │          │
│  │ • Cached response       │          │ • Safety validation  │          │
│  │ • Clarification needed  │          │ • Format output      │          │
│  └─────────────────────────┘          └──────────────────────┘          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Philosophy Principles

#### 1. "Protect the Inference"

The decoder model (Qwen2.5-Coder) is our most expensive operation. Every decision should ask: **"Does this help us hit the model once with maximum context?"**

- Pre-processing: Maximize context quality → One-shot success
- Post-processing: Indicates pre-processing failure → Minimize

#### 2. "Lazy is Good"

Good programs break early. If we can answer without full processing:

- **Dangerous pattern detected**: Block immediately, don't generate
- **Cached response**: Return immediately, don't regenerate
- **Clarification needed**: Ask user, don't guess
- **No generation needed**: Explain why, don't waste cycles

#### 3. "Parallelism is Free (Almost)"

With Tokio's async runtime, we can run independent tasks concurrently:

```rust
// Bad: Sequential
let intent = classify_intent(&input).await?;
let context = gather_context(&input).await?;
let rules = select_rules(&input).await?;

// Good: Parallel
let (intent, context, rules) = tokio::try_join!(
    classify_intent(&input),
    gather_context(&input),
    select_rules(&input),
)?;
```

The wall-clock time is the **max** of parallel tasks, not the **sum**.

#### 4. "Domain-Aware Everything"

Once we know the domain, everything becomes more focused:

| Without Domain | With Domain |
|---------------|-------------|
| Load all 10 prompt templates | Load 1-2 relevant templates |
| Check 80+ safety patterns | Check 10-15 domain patterns |
| Include examples from all domains | Include domain-specific examples |
| General platform rules | Domain-specific platform rules |

---

## Architecture Overview

### Pipeline Stages

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        PRE-PROCESSING PIPELINE                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  User Input: "find large log files and delete them"                     │
│       │                                                                  │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    STAGE 1: PARALLEL ANALYSIS                    │   │
│  │              (All run concurrently via tokio::join!)             │   │
│  │                                                                   │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐    │   │
│  │  │   Intent     │  │   Context    │  │   Early Safety      │    │   │
│  │  │   Router     │  │   Scanner    │  │   Check             │    │   │
│  │  │ (Gemma)      │  │              │  │                     │    │   │
│  │  │              │  │ • Platform   │  │ • Critical patterns │    │   │
│  │  │ → Domain:    │  │ • Available  │  │ • Quick blocklist   │    │   │
│  │  │   file_ops   │  │   commands   │  │ • Obvious dangers   │    │   │
│  │  │ → Confidence │  │ • Working    │  │                     │    │   │
│  │  │   0.94       │  │   directory  │  │ → BLOCK if found    │    │   │
│  │  └──────────────┘  └──────────────┘  └─────────────────────┘    │   │
│  │         │                  │                    │                │   │
│  └─────────┼──────────────────┼────────────────────┼────────────────┘   │
│            │                  │                    │                     │
│            ▼                  ▼                    ▼                     │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    STAGE 2: EARLY EXIT GATE                      │   │
│  │                                                                   │   │
│  │  Check: Can we answer WITHOUT hitting the coder model?           │   │
│  │                                                                   │   │
│  │  • Critical danger detected? → BLOCK (exit early)                │   │
│  │  • Cached response exists?   → RETURN (exit early)               │   │
│  │  • Need clarification?       → ASK (exit early)                  │   │
│  │  • Low intent confidence?    → FALLBACK (continue with general)  │   │
│  │                                                                   │   │
│  │  If all pass → CONTINUE to inference                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│            │                                                             │
│            ▼                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    STAGE 3: CONTEXT ASSEMBLY                     │   │
│  │                    (Domain-Aware Loading)                        │   │
│  │                                                                   │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐    │   │
│  │  │   Domain     │  │   Rule       │  │   Example           │    │   │
│  │  │   Prompt     │  │   Selection  │  │   Selection         │    │   │
│  │  │              │  │              │  │                     │    │   │
│  │  │ Load only:   │  │ Load only:   │  │ Load only:          │    │   │
│  │  │ • file_ops   │  │ • file rules │  │ • file examples     │    │   │
│  │  │ • archive    │  │ • rm/delete  │  │ • find + rm         │    │   │
│  │  │   (related)  │  │   patterns   │  │   examples          │    │   │
│  │  └──────────────┘  └──────────────┘  └─────────────────────┘    │   │
│  │                                                                   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│            │                                                             │
│            ▼                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                OUTPUT: Optimized Generation Context              │   │
│  │                                                                   │   │
│  │  {                                                                │   │
│  │    domain: FileOperations,                                       │   │
│  │    prompt_template: "file-operations.toml",                      │   │
│  │    platform_context: { os: "linux", commands: [...] },           │   │
│  │    safety_rules: [12 file-specific patterns],                    │   │
│  │    examples: [5 file operation examples],                        │   │
│  │    user_input: "find large log files and delete them"           │   │
│  │  }                                                                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                    ┌───────────────────────────────┐
                    │   INFERENCE (Coder Model)     │
                    │   • Single call               │
                    │   • Domain-optimized prompt   │
                    │   • High-quality context      │
                    │   → One-shot generation       │
                    └───────────────────────────────┘
```

### Component Responsibilities

| Component | Stage | Parallelizable | Can Exit Early | Output |
|-----------|-------|----------------|----------------|--------|
| Intent Router | 1 | Yes | No | Domain + confidence |
| Context Scanner | 1 | Yes | No | Platform info |
| Early Safety | 1 | Yes | Yes (block) | Risk assessment |
| Cache Check | 2 | N/A | Yes (return) | Cached response |
| Clarifier | 2 | N/A | Yes (ask) | Questions |
| Domain Prompt | 3 | Yes | No | Prompt template |
| Rule Selector | 3 | Yes | No | Applicable rules |
| Example Selector | 3 | Yes | No | Few-shot examples |

---

## Component Design

### Intent Router (FunctionGemma)

**Purpose**: Classify user intent into command domains.

**Input**: Raw user prompt
**Output**: Primary domain, secondary domains, confidence score

```rust
pub struct IntentRouter {
    client: FunctionGemmaClient,
    registry: DomainRegistry,
    cache: LruCache<String, RoutingResult>,
}

impl IntentRouter {
    /// Classify user intent - designed for parallel execution
    pub async fn classify(&self, input: &str) -> Result<RoutingResult> {
        // Check cache first (fast path)
        if let Some(cached) = self.cache.get(input) {
            return Ok(cached.clone());
        }

        // Call FunctionGemma
        let result = self.client.call_function(
            input,
            &self.registry.to_function_definitions(),
        ).await?;

        // Parse and cache
        let routing = self.parse_result(result)?;
        self.cache.insert(input.to_string(), routing.clone());
        Ok(routing)
    }
}
```

### Context Scanner

**Purpose**: Gather platform and environment context.

**Input**: None (reads from system)
**Output**: Platform info, available commands, environment

```rust
pub struct ContextScanner {
    cached_context: OnceCell<PlatformContext>,
}

impl ContextScanner {
    /// Scan system context - designed for parallel execution
    pub async fn scan(&self) -> Result<PlatformContext> {
        // Context rarely changes, cache aggressively
        if let Some(ctx) = self.cached_context.get() {
            return Ok(ctx.clone());
        }

        // Parallel sub-scans
        let (os_info, commands, env) = tokio::try_join!(
            self.scan_os_info(),
            self.scan_available_commands(),
            self.scan_environment(),
        )?;

        let context = PlatformContext { os_info, commands, env };
        let _ = self.cached_context.set(context.clone());
        Ok(context)
    }
}
```

### Early Safety Check

**Purpose**: Quick detection of critical dangerous patterns.

**Input**: User prompt
**Output**: Early block decision or continue

```rust
pub struct EarlySafetyChecker {
    critical_patterns: Vec<CompiledPattern>,
}

impl EarlySafetyChecker {
    /// Quick safety scan - designed for parallel execution
    /// Returns early block or continue signal
    pub fn check(&self, input: &str) -> EarlySafetyResult {
        // Only check CRITICAL patterns (fast)
        // Domain-specific patterns checked in Stage 3
        for pattern in &self.critical_patterns {
            if pattern.is_match(input) {
                return EarlySafetyResult::Block {
                    reason: pattern.message.clone(),
                    severity: RiskLevel::Critical,
                };
            }
        }

        // Also check for obvious dangerous keywords
        if self.contains_dangerous_intent(input) {
            return EarlySafetyResult::RequiresConfirmation {
                reason: "Request appears to involve destructive operations".into(),
            };
        }

        EarlySafetyResult::Continue
    }

    fn contains_dangerous_intent(&self, input: &str) -> bool {
        let dangerous_phrases = [
            "delete everything",
            "remove all",
            "format disk",
            "wipe",
            "destroy",
        ];
        dangerous_phrases.iter().any(|p| input.to_lowercase().contains(p))
    }
}
```

### Rule Selector

**Purpose**: Select only applicable safety rules for the domain.

**Input**: Domain classification
**Output**: Filtered safety patterns

```rust
pub struct RuleSelector {
    rules_by_domain: HashMap<Domain, Vec<SafetyPattern>>,
    global_rules: Vec<SafetyPattern>,
}

impl RuleSelector {
    /// Select rules relevant to domain - reduces validation time
    pub fn select(&self, routing: &RoutingResult) -> Vec<&SafetyPattern> {
        let mut rules = Vec::new();

        // Always include global critical rules
        rules.extend(self.global_rules.iter().filter(|r|
            r.risk_level == RiskLevel::Critical
        ));

        // Add primary domain rules
        if let Some(domain_rules) = self.rules_by_domain.get(&routing.primary_domain) {
            rules.extend(domain_rules.iter());
        }

        // Add secondary domain rules (with lower priority)
        for domain in &routing.secondary_domains {
            if let Some(domain_rules) = self.rules_by_domain.get(domain) {
                rules.extend(domain_rules.iter());
            }
        }

        rules
    }
}
```

---

## Parallelization Strategy

### Tokio-Based Concurrency

```rust
use tokio::try_join;

pub async fn pre_process(input: &str) -> Result<GenerationContext> {
    // STAGE 1: Parallel analysis
    let (intent_result, platform_context, early_safety) = try_join!(
        intent_router.classify(input),
        context_scanner.scan(),
        async { Ok(early_safety_checker.check(input)) },
    )?;

    // STAGE 2: Early exit gate
    match early_safety {
        EarlySafetyResult::Block { reason, severity } => {
            return Err(Error::SafetyBlock { reason, severity });
        }
        EarlySafetyResult::RequiresConfirmation { reason } => {
            return Err(Error::ConfirmationRequired { reason });
        }
        EarlySafetyResult::Continue => {}
    }

    // Check cache
    if let Some(cached) = cache.get(input, &intent_result.primary_domain) {
        return Ok(cached);
    }

    // STAGE 3: Parallel context assembly (domain-aware)
    let (prompt_template, safety_rules, examples) = try_join!(
        domain_loader.load_prompt(&intent_result),
        async { Ok(rule_selector.select(&intent_result)) },
        example_selector.select(&intent_result),
    )?;

    Ok(GenerationContext {
        domain: intent_result.primary_domain,
        prompt_template,
        platform_context,
        safety_rules,
        examples,
        user_input: input.to_string(),
    })
}
```

### Dependency Graph

```
User Input
    │
    ├──────────────┬──────────────┐  ← PARALLEL (Stage 1)
    ▼              ▼              ▼
Intent         Context        Early
Router         Scanner        Safety
    │              │              │
    └──────────────┴──────────────┘
                   │
                   ▼
            Early Exit Gate ──────► EXIT if blocked/cached
                   │
                   │ (domain known)
    ┌──────────────┼──────────────┐  ← PARALLEL (Stage 3)
    ▼              ▼              ▼
  Domain        Rule          Example
  Prompt       Selector       Selector
    │              │              │
    └──────────────┴──────────────┘
                   │
                   ▼
         Generation Context ──────► Inference
```

### Resource Considerations

| Approach | Memory | Latency | CPU |
|----------|--------|---------|-----|
| Sequential | Lower | Higher | Bursty |
| Parallel | Higher | Lower | Spread |

**Trade-off**: We accept slightly higher memory for significantly lower latency. This is appropriate for a CLI tool where:
- Requests are infrequent (human-paced)
- Response time directly impacts UX
- Resources are available between requests

---

## Lazy Evaluation & Early Exit

### Early Exit Opportunities

```rust
pub enum EarlyExitDecision {
    /// Command is critically dangerous - block immediately
    Block {
        reason: String,
        severity: RiskLevel,
        suggestion: Option<String>,
    },

    /// Found in cache - return immediately
    Cached {
        response: GeneratedCommand,
        cache_age: Duration,
    },

    /// Need more information - ask user
    Clarify {
        questions: Vec<Question>,
        reason: String,
    },

    /// Request doesn't need generation
    NoGenerationNeeded {
        explanation: String,
    },

    /// Continue to inference
    Continue,
}
```

### Decision Flow

```rust
impl PreProcessor {
    async fn check_early_exit(
        &self,
        input: &str,
        intent: &RoutingResult,
        safety: &EarlySafetyResult,
    ) -> EarlyExitDecision {
        // 1. Critical safety - highest priority
        if let EarlySafetyResult::Block { reason, severity } = safety {
            return EarlyExitDecision::Block {
                reason: reason.clone(),
                severity: *severity,
                suggestion: self.get_safe_alternative(input),
            };
        }

        // 2. Cache check - avoid redundant work
        if let Some(cached) = self.cache.get(input, &intent.primary_domain) {
            return EarlyExitDecision::Cached {
                response: cached,
                cache_age: self.cache.age(input),
            };
        }

        // 3. Low confidence - need clarification
        if intent.confidence < 0.4 {
            return EarlyExitDecision::Clarify {
                questions: self.generate_clarification_questions(input),
                reason: "Could not determine your intent clearly".into(),
            };
        }

        // 4. No generation needed (meta-commands)
        if self.is_meta_request(input) {
            return EarlyExitDecision::NoGenerationNeeded {
                explanation: self.handle_meta_request(input),
            };
        }

        // 5. Continue to inference
        EarlyExitDecision::Continue
    }
}
```

### Rule Optimization (Lazy Patterns)

```rust
pub struct LazyRuleEngine {
    // Rules organized by domain for fast lookup
    domain_rules: HashMap<Domain, Vec<CompiledPattern>>,

    // Global rules that always apply
    global_critical: Vec<CompiledPattern>,

    // Expensive patterns to check only if simple checks pass
    complex_patterns: Vec<CompiledPattern>,
}

impl LazyRuleEngine {
    /// Validate with early termination
    pub fn validate(&self, command: &str, domain: Domain) -> ValidationResult {
        // Level 1: Critical patterns (always check, very fast)
        for pattern in &self.global_critical {
            if pattern.is_match(command) {
                return ValidationResult::critical_block(pattern);
            }
        }

        // Level 2: Domain-specific patterns (if we know domain)
        if let Some(rules) = self.domain_rules.get(&domain) {
            for pattern in rules {
                if pattern.is_match(command) {
                    return ValidationResult::domain_match(pattern);
                }
            }
        }

        // Level 3: Complex patterns (expensive, only if needed)
        // Skip if we already found issues
        for pattern in &self.complex_patterns {
            if pattern.is_match(command) {
                return ValidationResult::complex_match(pattern);
            }
        }

        ValidationResult::safe()
    }
}
```

---

## Memory Model Integration

### Context Memory Architecture

The pre-processing pipeline integrates with Caro's planned memory model:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          MEMORY MODEL                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌───────────────────┐  ┌───────────────────┐  ┌──────────────────┐    │
│  │  Session Memory   │  │   Tool Memory     │  │   Pattern Memory │    │
│  │                   │  │                   │  │                  │    │
│  │  • Recent cmds    │  │  • Which tools    │  │  • User's cmd    │    │
│  │  • Domain prefs   │  │    work well for  │  │    preferences   │    │
│  │  • Context cues   │  │    this intent    │  │  • Common tasks  │    │
│  │                   │  │  • Tool flags     │  │  • Corrections   │    │
│  └───────────────────┘  │    by platform    │  └──────────────────┘    │
│           │             └───────────────────┘           │               │
│           │                      │                      │               │
│           └──────────────────────┼──────────────────────┘               │
│                                  │                                       │
│                                  ▼                                       │
│                    ┌─────────────────────────────┐                      │
│                    │   Pre-Processing Pipeline   │                      │
│                    │                             │                      │
│                    │  Uses memory for:           │                      │
│                    │  • Smarter routing          │                      │
│                    │  • Better context           │                      │
│                    │  • Personalized rules       │                      │
│                    └─────────────────────────────┘                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Memory-Enhanced Routing

```rust
impl IntentRouter {
    pub async fn classify_with_memory(
        &self,
        input: &str,
        memory: &SessionMemory,
    ) -> Result<RoutingResult> {
        // Check if user has domain preference
        if let Some(preferred) = memory.domain_preference_for(input) {
            return Ok(RoutingResult {
                primary_domain: preferred,
                confidence: 0.95,
                source: RoutingSource::Memory,
                ..Default::default()
            });
        }

        // Check for pattern match with previous commands
        if let Some(pattern) = memory.similar_request_pattern(input) {
            return Ok(RoutingResult {
                primary_domain: pattern.domain,
                confidence: 0.85,
                source: RoutingSource::PatternMatch,
                ..Default::default()
            });
        }

        // Fall back to FunctionGemma
        self.classify(input).await
    }
}
```

### Tool Context Memory

```rust
impl ContextScanner {
    pub async fn scan_with_memory(
        &self,
        domain: Domain,
        memory: &ToolMemory,
    ) -> Result<EnrichedContext> {
        let base_context = self.scan().await?;

        // Enrich with learned tool preferences
        let preferred_tools = memory.preferred_tools_for(domain);
        let known_flags = memory.flag_compatibility_for(domain);

        Ok(EnrichedContext {
            platform: base_context,
            preferred_tools,
            known_flags,
            // Include only relevant tools based on memory
            focused_commands: self.filter_by_relevance(
                &base_context.commands,
                &preferred_tools,
            ),
        })
    }
}
```

---

## Performance Budget

### Latency Allocation

| Phase | Budget | Components |
|-------|--------|------------|
| **Pre-Processing** | 200ms | Intent + Context + Safety (parallel) |
| **Context Assembly** | 50ms | Domain loading + Rule selection |
| **Inference** | 1500ms | Coder model generation |
| **Post-Processing** | 50ms | Final safety + formatting |
| **Total** | **1800ms** | End-to-end target |

### Pre-Processing Breakdown

| Component | Sequential | Parallel | Budget |
|-----------|------------|----------|--------|
| Intent Router | 100ms | 100ms | 100ms |
| Context Scanner | 50ms | parallel | - |
| Early Safety | 10ms | parallel | - |
| Cache Check | 1ms | sequential | 1ms |
| Domain Prompt | 20ms | parallel | 20ms |
| Rule Selection | 5ms | parallel | - |
| Example Selection | 20ms | parallel | - |
| **Total** | 206ms | **121ms** | **150ms** |

### Early Exit Performance

| Scenario | Time | Skipped |
|----------|------|---------|
| Critical block | <15ms | Inference, post-processing |
| Cache hit | <5ms | All stages |
| Clarification | <50ms | Inference |
| Normal flow | ~1800ms | Nothing |

---

## Consequences

### Positive

1. **Lower Latency**: Parallel pre-processing reduces wall-clock time
2. **Better Accuracy**: Domain-specific context improves generation
3. **Enhanced Safety**: Domain-aware rules catch more patterns
4. **Resource Efficiency**: Lazy evaluation skips unnecessary work
5. **One-Shot Success**: Better prep means fewer retries
6. **Memory Integration**: Foundation for learning user patterns

### Negative

1. **Complexity**: More components to maintain
2. **Memory Usage**: Parallel tasks consume more memory
3. **FunctionGemma Dependency**: Optional but provides best routing
4. **Caching Complexity**: Multiple cache layers to manage

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| FunctionGemma latency | Low | Medium | Cache routing decisions |
| Parallel task failure | Low | High | Graceful degradation |
| Cache staleness | Medium | Low | TTL-based invalidation |
| Memory pressure | Low | Medium | Resource limits |

---

## Alternatives Considered

### Alternative 1: Sequential Processing

**Description**: Keep current sequential flow, no parallelization
**Pros**: Simpler, less resource usage
**Cons**: Higher latency, missed optimization opportunity
**Decision**: Rejected - latency is critical for UX

### Alternative 2: Full FunctionGemma Generation

**Description**: Use FunctionGemma for command generation too
**Pros**: Lower resource usage, faster inference
**Cons**: Poor generation quality (not trained for this)
**Decision**: Rejected - quality is essential

### Alternative 3: Keyword-Based Routing

**Description**: Use regex/keyword matching instead of LLM routing
**Pros**: No LLM dependency, very fast
**Cons**: Low accuracy, poor handling of natural language
**Decision**: Rejected - accuracy matters more

### Alternative 4: Client-Side Only Routing

**Description**: Route based on first word/command detection
**Pros**: Instant, no model needed
**Cons**: Misses intent ("help me find" vs "find command")
**Decision**: Partial - use as fast-path optimization

---

## Implementation Roadmap

### Phase 1: Foundation

1. Implement parallel pre-processing skeleton
2. Add basic early exit gates
3. Integrate FunctionGemma for intent routing
4. Add domain-specific prompt loading

### Phase 2: Optimization

1. Implement lazy rule evaluation
2. Add routing cache
3. Integrate with context scanner
4. Performance benchmarking

### Phase 3: Memory Integration

1. Add session memory tracking
2. Implement tool preference learning
3. Pattern-based routing enhancement
4. Memory-aware context assembly

### Phase 4: Polish

1. Fallback paths for all components
2. Comprehensive error handling
3. Metrics and observability
4. Documentation

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| End-to-end latency (p95) | 2.5s | 1.8s |
| First-shot accuracy | 60% | 80% |
| Early exit rate | 0% | 15% |
| Cache hit rate | 0% | 30% |
| Pre-processing overhead | N/A | <150ms |

---

## References

- [ADR-001: LLM Inference Architecture](./001-llm-inference-architecture.md)
- [Spec 008: FunctionGemma Intent Router](../../specs/008-functiongemma-intent-router/spec.md)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [The Lazy Programmer's Guide to Optimization](https://martinfowler.com/articles/lean-performance.html)

---

*This ADR was authored in January 2026 and reflects the planned pre-processing architecture for Caro's command generation pipeline.*
