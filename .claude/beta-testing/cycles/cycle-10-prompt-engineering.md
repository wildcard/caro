# Beta Test Cycle 10: Prompt Engineering Improvements (Phase 3)

**Date**: 2026-01-08
**Version**: caro 1.0.4 (commit: 27fc5f1)
**Backend**: embedded (SmolLM/Qwen via embedded backend)
**Phase**: Prompt Engineering (as per original plan)

## Executive Summary

**Cycle 10 implements comprehensive prompt engineering improvements** to enhance the embedded backend's (LLM-based) command generation quality. These changes focus on three key areas:

1. **Chain-of-Thought Prompting**: Explicit 5-step reasoning process
2. **Negative Examples**: Platform-specific examples of what NOT to generate
3. **Expanded Few-Shot Examples**: 3x more diverse training examples (8 → 25+ per platform)

**Changes Made**:
- ✅ Added explicit chain-of-thought decision procedure (5 steps)
- ✅ Added negative examples section with platform-specific mistakes
- ✅ Expanded GNU examples from 8 to 25+ examples
- ✅ Expanded BSD examples from 8 to 23+ examples
- ✅ Temperature already set to 0.1 (from previous work)

---

## Problem Statement

### Context from Previous Cycles

Cycles 0-9 achieved **86.2% pass rate** for the **static matcher** backend, which handles instant, deterministic command generation for well-known patterns.

However, the **embedded backend** (LLM-based fallback for novel queries) had several quality issues:

1. **No explicit reasoning process** - Model jumped directly to output
2. **Platform confusion** - Generated GNU flags on BSD systems
3. **Sparse training examples** - Only 8 examples per platform
4. **No negative feedback** - Model never saw what NOT to do
5. **Temperature too high** - 0.7 caused variability (already fixed to 0.1)

### Why This Matters

The embedded backend handles:
- Novel queries not in static patterns
- Complex multi-step operations
- Queries requiring context awareness
- Edge cases and variations

Quality improvements here directly impact **user satisfaction** when they ask questions beyond the static patterns.

---

## Changes Implemented

### 1. Chain-of-Thought Prompting

**Before** (Cycle 9):
```
DECISION PROCEDURE:
1. Parse intent -> determine category
2. Select template
3. Fill template
4. Verify flags
5. Output JSON
```

**After** (Cycle 10):
```
DECISION PROCEDURE (THINK step by step):

THINK through these steps mentally BEFORE generating output:

STEP 1: CATEGORIZE the user intent
   - "list/show files" -> LISTING
   - "find files with condition" -> FILTERING (use find)
   - ...

STEP 2: CHECK platform constraints
   - What OS am I running on? (see CAPABILITY_PROFILE)
   - Which flags are supported? (FIND_PRINTF, SORT_H, etc.)
   - GNU, BSD, or POSIX mode?

STEP 3: SELECT appropriate template
   - Find matching template in TEMPLATES section
   - Ensure template uses only supported flags
   - Check for platform-specific alternatives

STEP 4: FILL template with user parameters
   - Extract file patterns, size constraints, time ranges
   - Substitute into template command
   - Verify syntax correctness

STEP 5: VALIDATE then OUTPUT
   - All flags in CAPABILITY_PROFILE? If no -> adapt
   - Command safe or needs confirmation?
   - Then output ONLY: {"cmd": "final_command"}
```

**Rationale**: Small models (SmolLM-135M, Qwen-1.5B) benefit from explicit reasoning scaffolding. This helps them:
- Consider platform constraints before generating
- Check capabilities systematically
- Avoid jumping to incorrect conclusions

---

### 2. Negative Examples

Added comprehensive "what NOT to do" section with platform-specific mistakes:

**BSD-specific negative examples**:
```
❌ BAD (GNU flags on BSD): {"cmd": "ps aux --sort=-%mem"}
✅ GOOD (BSD compatible): {"cmd": "ps aux | sort -k4 -rn | head -10"}
Why: BSD ps doesn't support --sort flag

❌ BAD (GNU find printf): {"cmd": "find . -printf '%T@ %p\n' | sort -nr"}
✅ GOOD (BSD compatible): {"cmd": "find . -exec stat -f '%m %N' {} + | sort -nr"}
Why: BSD find doesn't support -printf
```

**GNU/Linux-specific negative examples**:
```
❌ BAD (missing -h flag): {"cmd": "du -s */ | sort -rn"}
✅ GOOD (human readable): {"cmd": "du -sh */ | sort -rh"}
Why: Always use human-readable sizes with -h when available

❌ BAD (inefficient nested loops): {"cmd": "for f in $(find .); do wc -l $f; done"}
✅ GOOD (use xargs): {"cmd": "find . -type f | xargs wc -l"}
Why: xargs is much faster for bulk operations
```

**Common mistakes (all platforms)**:
```
❌ BAD (multiple commands in JSON): {"cmd": "ls -la\ncd ..\nls"}
✅ GOOD (single command or pipeline): {"cmd": "ls -la"}
Why: Output must be ONE command or pipeline

❌ BAD (command with explanation): {"cmd": "find . -name '*.log' # finds log files"}
✅ GOOD (command only): {"cmd": "find . -name '*.log'"}
Why: No comments in JSON output
```

**Rationale**: Learning from mistakes is powerful. Showing the model explicit counter-examples helps it avoid common errors, especially platform-specific flag incompatibilities.

---

### 3. Expanded Few-Shot Examples

**Before** (Cycle 9): 8 examples per platform
**After** (Cycle 10): 25+ examples per platform (3x increase)

**New example categories added**:

1. **Process Monitoring** (3 examples):
   - "show top 10 memory-consuming processes"
   - "show top 10 CPU-consuming processes"
   - "find all running python processes"

2. **Text Search** (3 examples):
   - "find files containing error in logs"
   - "search for pattern in all js files"
   - Additional variations on grep usage

3. **Network Operations** (2 examples):
   - "show all listening ports"
   - "find process using port 8080"

4. **Counting and Statistics** (2 examples):
   - "count number of log files"
   - Additional wc -l variations

5. **Git Operations** (2 examples):
   - "show recent commits"
   - "show uncommitted changes"

6. **Additional File Management** (3 examples):
   - "find files modified yesterday"
   - "find files larger than 50MB"
   - "find javascript files"

**Rationale**: More diverse examples give the model better coverage of common patterns. The original 8 examples were heavily biased toward file operations; the expanded set covers system monitoring, text processing, networking, and git.

---

## Technical Details

### File Modified
- `src/prompts/smollm_prompt.rs`

### Methods Added
- `build_negative_examples()` - Platform-specific counter-examples

### Methods Modified
- `build_decision_procedure()` - Added chain-of-thought steps
- `build_system_prompt()` - Integrated negative examples section
- `gnu_examples()` - Expanded from 8 to 25+ examples
- `bsd_examples()` - Expanded from 8 to 23+ examples

### Lines Changed
- +150 lines added
- -8 lines removed
- Net: +142 lines

---

## Expected Impact

### Measurable Improvements

1. **Platform Compatibility**: Reduced GNU-on-BSD and BSD-on-GNU errors
   - Before: Model often confused platform flags
   - After: Explicit negative examples + platform checks in reasoning

2. **Command Quality**: Fewer malformed or inefficient commands
   - Before: No guidance on efficiency patterns
   - After: Negative examples show inefficient patterns to avoid

3. **Category Coverage**: Better handling of non-file-management queries
   - Before: 8 examples heavily file-focused
   - After: 25+ examples across 7 categories

### Qualitative Improvements

1. **Reasoning Transparency**: Chain-of-thought makes model's process clearer
2. **Error Avoidance**: Negative examples teach from mistakes
3. **Confidence**: More examples = better model confidence on edge cases

---

## Next Steps (Phase 4: Agent Loop)

The prompt improvements in Cycle 10 enhance the **model's ability to generate good commands**. Phase 4 will focus on the **agent loop's ability to repair bad commands**:

1. **Validation-Triggered Retry** (Phase 4.1):
   - Integrate CommandValidator with generation loop
   - Auto-repair commands that fail validation
   - Use RepairPromptBuilder for targeted fixes

2. **Confidence-Based Refinement** (Phase 4.2):
   - Extract confidence scores from LLM responses
   - Trigger refinement for low-confidence outputs
   - Multi-step agent process for complex queries

3. **Testing** (Phase 4.3):
   - Run full test suite with embedded backend
   - Measure improvement on complex queries (devops, text processing)
   - Compare against Cycle 9 baseline

---

## Success Criteria

### For Cycle 10 (Prompt Engineering)
- ✅ Chain-of-thought reasoning added
- ✅ Negative examples for all platforms
- ✅ 3x more few-shot examples
- ✅ Code compiles without errors

### For Cycle 11 (Agent Loop Testing)
- Measure embedded backend pass rate on test suite
- Target: Improvement on categories where static matcher fails
- Focus: devops_kubernetes, text_processing, complex queries

---

## Commit Information

**Commit**: 27fc5f1
**Message**: feat(prompts): [Cycle 2] Add chain-of-thought prompting and negative examples
**Branch**: release-planning/v1.1.0
**Date**: 2026-01-08

---

## Lessons Learned

1. **Prompt engineering is high-leverage**: Small prompt changes can have large impact on model behavior
2. **Negative examples are powerful**: Showing what NOT to do is as important as showing what to do
3. **Platform awareness is critical**: Platform-specific negative examples address real pain points
4. **Chain-of-thought helps small models**: Explicit reasoning scaffolding improves quality

---

## References

- Original Plan: `moonlit-kindling-acorn.md` § Phase 3: Prompt Engineering
- Previous Cycle: `cycle-9-final-milestone.md` (86.2% static matcher pass rate)
- Static Matcher Code: `src/backends/static_matcher.rs`
- Prompt Code: `src/prompts/smollm_prompt.rs`
