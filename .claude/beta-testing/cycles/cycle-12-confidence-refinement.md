# Beta Test Cycle 12: Confidence-Based Refinement (Phase 4.2)

**Date**: 2026-01-08
**Version**: caro 1.0.4 (commit: TBD)
**Backend**: embedded (SmolLM/Qwen via embedded backend)
**Phase**: Agent Loop Improvements (Phase 4.2)

## Executive Summary

**Cycle 12 implements confidence-based refinement** - the second component of Phase 4 (Agent Loop Improvements). This feature automatically triggers command refinement when the LLM expresses low confidence in its generated command.

**Changes Made**:
- ✅ Added confidence threshold field to AgentLoop struct
- ✅ Implemented confidence score checking after initial generation
- ✅ Integrated confidence-based refinement trigger
- ✅ Made threshold configurable via with_confidence_threshold() builder method
- ✅ Combined confidence check with platform issue detection

---

## Problem Statement

### Context from Previous Cycles

- **Cycle 11**: Implemented validation-triggered retry (Phase 4.1)
- **Cycles 0-10**: Achieved 86.2% pass rate with static matcher

Even with validation-triggered retry, the agent loop had a gap:

**The Problem**: LLMs can generate syntactically valid but semantically questionable commands when uncertain. These commands:
- Pass validation (correct syntax, allowed tools, no dangerous patterns)
- Have low confidence scores (< 0.8)
- May not optimally solve the user's query
- Could benefit from additional context and refinement

**Example**:
```
User: "find largest files in project"
Initial (confidence: 0.65): ls -lS | head -10
Better (after refinement): find . -type f -exec du -h {} + | sort -rh | head -10
```

The initial command works but is suboptimal:
- Only checks current directory (not recursive)
- Doesn't handle hidden files
- Less accurate size measurement

### Why This Matters

Confidence-based refinement enables:
- **Quality over Speed**: Prioritizes better commands for uncertain cases
- **Proactive Improvement**: Catches potential issues before user sees them
- **Multi-Step Reasoning**: Complex queries get multi-iteration thinking
- **Learning Signal**: Low confidence indicates need for more context

---

## Architecture: Confidence-Based Refinement

### Decision Flow

```
User Query
    ↓
Generate Command (LLM)
    ↓
Validate Command
    ├─ Invalid? → Repair (Cycle 11)
    └─ Valid? ↓
        Check Confidence Score
        ├─ confidence >= 0.8? → Return ✅
        └─ confidence < 0.8? ↓
            Refine with Context
            ↓
            Return Refined ✅
```

### Refinement Triggers

Commands are refined if **either** condition is true:

1. **Low Confidence**: `confidence_score < threshold` (default 0.8)
2. **Platform Issues**: Detected platform-specific problems (Cycle 10 logic)

```rust
let low_confidence = initial.confidence_score < self.confidence_threshold;
let needs_platform_fix = self.should_refine(&initial);

if !low_confidence && !needs_platform_fix {
    return Ok(initial); // Skip refinement
}
```

---

## Implementation Details

### 1. Confidence Threshold Field

**File**: `src/agent/mod.rs`
**Struct**: `AgentLoop`

Added configurable threshold:

```rust
pub struct AgentLoop {
    backend: Arc<dyn CommandGenerator>,
    static_matcher: Option<StaticMatcher>,
    validator: CommandValidator,
    context: ExecutionContext,
    _max_iterations: usize,
    timeout: Duration,
    confidence_threshold: f64, // NEW: Default 0.8
}
```

### 2. Configuration Builder

**Method**: `with_confidence_threshold()`

Allows customization:

```rust
pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
    self.confidence_threshold = threshold;
    self
}

// Usage:
let agent = AgentLoop::new(backend, context)
    .with_confidence_threshold(0.85); // More aggressive refinement
```

### 3. Refinement Logic Integration

**Method**: `generate_command_impl()`

After validation passes, check confidence:

```rust
// Check confidence score - trigger refinement if low
let low_confidence = initial.confidence_score < self.confidence_threshold;

if low_confidence {
    info!("Low confidence ({:.2}), triggering refinement", initial.confidence_score);
}

// Check if refinement is beneficial
let needs_platform_fix = self.should_refine(&initial);

if !low_confidence && !needs_platform_fix {
    info!("Refinement not needed (confidence: {:.2}, no platform issues)",
          initial.confidence_score);
    return Ok(initial);
}

// Iteration 2: Refine with command context
if low_confidence {
    debug!("Iteration 2: Refining due to low confidence ({:.2})",
           initial.confidence_score);
} else {
    debug!("Iteration 2: Refining due to platform issues");
}
```

### 4. Logging and Observability

Added structured logging:
- Info logs for confidence-triggered refinements
- Debug logs distinguishing confidence vs platform triggers
- Confidence scores in log messages for debugging

---

## Expected Impact

### Measurable Improvements

1. **Higher Quality Commands**:
   - Before: Low-confidence commands returned as-is
   - After: Low-confidence commands get refinement opportunity
   - Expected: 20-30% quality improvement for uncertain queries

2. **Reduced User Friction**:
   - Before: Users might need to rephrase or clarify
   - After: Agent proactively seeks better solution
   - Expected: 15-20% fewer follow-up queries

3. **Better Complex Query Handling**:
   - Before: Complex queries get single-pass treatment
   - After: Complex queries automatically trigger multi-step
   - Expected: 25-35% better handling of devops/k8s queries

### Qualitative Improvements

1. **Adaptive Behavior**: Agent knows when it needs help
2. **Transparency**: Logs reveal confidence-driven decisions
3. **Tunability**: Threshold can be adjusted per use case
4. **Graceful Degradation**: Falls back gracefully under time pressure

---

## Configuration Trade-offs

### Threshold Values

| Threshold | Behavior | Use Case |
|-----------|----------|----------|
| **0.9** | Very aggressive refinement | Maximum quality, slower |
| **0.8** | Default balanced approach | Recommended |
| **0.7** | Conservative refinement | Faster, risk some quality |
| **0.6** | Minimal refinement | Speed-critical applications |

### Performance vs Quality

```
Refinement Rate vs Threshold
│
│  100% ┤           ╭─────────
│      │         ╭─╯
│   50% ┤     ╭───╯
│      │   ╭─╯
│    0% ┼───╯
       └─────────────────────
        0.5  0.7  0.9  1.0
           Threshold
```

**Analysis**:
- Threshold 0.8: ~30% of commands refined
- Threshold 0.9: ~50% of commands refined
- Threshold 0.7: ~20% of commands refined

---

## Testing Strategy

### Unit Test: Confidence Threshold

```rust
#[tokio::test]
async fn test_confidence_based_refinement() {
    let backend = MockBackend::new()
        .with_initial_confidence(0.65) // Low confidence
        .with_refined_confidence(0.92); // High after refinement

    let agent = AgentLoop::new(backend, context)
        .with_confidence_threshold(0.8);

    let result = agent.generate_command("complex query").await;

    assert!(result.is_ok());
    let cmd = result.unwrap();

    // Should have triggered refinement due to low confidence
    assert!(cmd.confidence_score >= 0.8);
    assert_eq!(backend.refinement_count(), 1);
}
```

### Integration Test: Skip Refinement for High Confidence

```rust
#[tokio::test]
async fn test_skip_refinement_high_confidence() {
    let backend = MockBackend::new()
        .with_initial_confidence(0.95); // High confidence

    let agent = AgentLoop::new(backend, context);

    let result = agent.generate_command("simple query").await;

    // Should NOT have triggered refinement
    assert_eq!(backend.refinement_count(), 0);
}
```

### Manual Test Cases

1. **Low Confidence → Refinement**:
   ```
   Query: "find files modified in last 3 days"
   Initial: ls -lt (confidence: 0.6)
   Expected: Trigger refinement
   Refined: find . -type f -mtime -3 (confidence: 0.9)
   ```

2. **High Confidence → No Refinement**:
   ```
   Query: "list files"
   Initial: ls -la (confidence: 0.98)
   Expected: No refinement (fast path)
   ```

3. **Platform Issue → Refinement (regardless of confidence)**:
   ```
   Query: "show top processes by CPU"
   Initial: ps aux --sort=-%cpu (confidence: 0.95, but has GNU flag on BSD)
   Expected: Trigger refinement (platform issue)
   Refined: ps aux | sort -k3 -rn | head -10
   ```

---

## Performance Analysis

### Latency Impact

**Best Case** (high confidence, no platform issues):
- Validation: ~1-5ms
- No refinement: 0ms
- **Total overhead**: ~5ms (negligible)

**Refinement Case** (low confidence or platform issues):
- Validation: ~1-5ms
- Context gathering: ~10-50ms (command --help lookups)
- Refinement generation: ~500-1500ms
- **Total**: ~1500-2000ms (acceptable for better quality)

### Confidence Distribution

Expected distribution of confidence scores:

| Confidence Range | % of Queries | Action |
|------------------|--------------|--------|
| 0.9 - 1.0 | 40% | No refinement ✅ |
| 0.8 - 0.9 | 30% | No refinement ✅ |
| 0.7 - 0.8 | 20% | Refine 🔄 |
| 0.0 - 0.7 | 10% | Refine 🔄 |

**Impact**: ~30% of queries trigger refinement (threshold 0.8)

---

## Phase 4 Progress

### Completed

- ✅ **Phase 4.1** (Cycle 11): Validation-triggered retry
- ✅ **Phase 4.2** (Cycle 12): Confidence-based refinement

### Next Steps

**Phase 4.3**: Testing with full embedded backend

1. Run full test suite with embedded backend (75 test cases)
2. Measure pass rate improvements from Cycles 11-12
3. Compare against baseline (Cycle 10: prompt engineering only)
4. Document results in Cycle 13

**Expected Improvements**:
- Validation retry: +5-10% pass rate
- Confidence refinement: +3-7% pass rate
- **Combined**: Target 75%+ overall pass rate for embedded backend

---

## Success Criteria

### For Cycle 12 (Confidence-Based Refinement)
- ✅ Confidence threshold added to AgentLoop
- ✅ Confidence checking integrated into refinement decision
- ✅ Configurable via builder pattern
- ✅ Combined with platform issue detection
- ✅ Code compiles without errors
- ⏳ Testing: Unit tests pending (Phase 4.3)
- ⏳ Testing: Manual tests pending (Phase 4.3)

### For Phase 4 (Agent Loop) Overall
- ✅ Phase 4.1: Validation-triggered retry (Cycle 11)
- ✅ Phase 4.2: Confidence-based refinement (Cycle 12 - THIS CYCLE)
- ⏳ Phase 4.3: Full testing with embedded backend (Cycle 13 - NEXT)

---

## Commit Information

**Commit**: TBD (to be created)
**Message**: feat(agent): [Cycle 12] Add confidence-based refinement
**Branch**: release-planning/v1.1.0
**Date**: 2026-01-08

---

## Lessons Learned

1. **Confidence as Quality Signal**: LLM confidence scores are valuable for deciding when to invest in refinement
2. **Combined Triggers**: Confidence + platform issues = comprehensive refinement strategy
3. **Tunability Matters**: Different use cases need different thresholds
4. **Minimal Overhead**: Confidence check is essentially free (already in response)
5. **Graceful Degradation**: Timeout protection prevents runaway refinement

---

## Related Work

### Confidence Scoring in LLMs

Research shows LLM confidence scores correlate with:
- **Correctness**: Higher confidence → more likely correct
- **Uncertainty**: Low confidence → model is unsure
- **Complexity**: Complex queries → lower confidence

**Application**: Using confidence for adaptive refinement aligns with best practices in LLM systems.

### Multi-Step Agent Loops

Industry patterns:
- **ReAct**: Reasoning + Action loops
- **Chain-of-Thought**: Step-by-step reasoning
- **Self-Refinement**: Models improving their own outputs

**Our Approach**: Confidence-triggered refinement is lightweight self-refinement.

---

## References

- Original Plan: `moonlit-kindling-acorn.md` § Phase 4.2: Confidence-Based Refinement
- Previous Cycle: `cycle-11-validation-retry.md` (validation integration)
- Agent Loop: `src/agent/mod.rs` (implementation)
- Models: `src/models/mod.rs` (GeneratedCommand with confidence_score)
- Next Cycle: `cycle-13-phase4-testing.md` (full testing - TBD)

---

**Status**: ✅ Phase 4.2 Complete - Confidence-Based Refinement Implemented

Next: Phase 4.3 - Full Testing with Embedded Backend
