# Cycle Documentation Template and Guidelines

This reference provides templates and guidelines for documenting beta test cycle results consistently and comprehensively.

## Purpose of Cycle Documentation

Cycle documentation serves multiple purposes:

1. **Progress Tracking**: Measure improvement over time
2. **Pattern Learning**: Identify what works and what doesn't
3. **Knowledge Transfer**: Help future developers understand decisions
4. **Reproducibility**: Enable others to understand and replicate fixes
5. **Milestone Validation**: Prove product delivers on promises

## Document Location

Store cycle documents in: `.claude/beta-testing/cycles/cycle-N-<description>.md`

**Naming Convention:**
- `cycle-0-baseline.md` - Initial baseline measurement
- `cycle-1-quick-wins.md` - First improvements
- `cycle-N-<key-achievement>.md` - Subsequent cycles

**Examples:**
- `cycle-7-batch-reordering.md`
- `cycle-8-category-completions.md`

## Required Sections

### 1. Frontmatter

```markdown
# Beta Test Cycle N: <Title>

**Date**: YYYY-MM-DD
**Version**: caro X.Y.Z (commit: <sha> or TBD)
**Backend**: static (StaticMatcher)
**Changes**: Brief summary of what was changed
```

### 2. Executive Summary

**Purpose**: Provide at-a-glance understanding of cycle results

**Required Content:**
- Overall pass rate change (before → after, percentage change)
- Key achievements (categories completed, milestone reached)
- Pattern count change (net increase/decrease)
- Complete categories count

**Template:**
```markdown
## Executive Summary

**Cycle N achieved X.X% overall pass rate**, <milestone achieved>.

**Key Results:**
- **Overall pass rate: X.X% → Y.Y%** (+Z.Z% improvement, +N tests) 🎯
- **Category Name: X.X% → Y.Y%** (+N tests) ⭐
- **Pattern count: N → M** (net change)
- **Complete categories: N** (list names)
```

**Example:**
```markdown
## Executive Summary

**Cycle 8 achieved 86.2% overall pass rate**, exceeding the 85% target.

**Key Results:**
- **Overall pass rate: 82.8% → 86.2%** (+3.4% improvement, +2 tests) 🎯
- **File Management: 94.7% → 100.0%** (COMPLETE! +1 test) ⭐
- **System Monitoring: 85.7% → 100.0%** (COMPLETE! +1 test) ⭐
- **Pattern count: 44 → 43** (net -1 after cleanup)
- **7 complete categories** (Git, DevOps, Network, Text Processing, Log Analysis, File Management, System Monitoring)
```

### 3. Improvements Made

**Purpose**: Detailed description of each fix implemented

**Required Content:**
- Fix category (pattern reordering, keyword adjustment, regex refinement, new pattern)
- Root cause analysis
- Solution applied
- Before/after comparison
- Impact assessment

**Template:**
```markdown
## Improvements Made

### 1. <Fix Type>: <Description>

**Issue**: <What was wrong>

**Root Cause**: <Why it was happening>

**Solution Applied**:
```rust
// Before
<code showing old pattern>

// After
<code showing new pattern>
```

**Rationale**: <Why this fix works>

**Impact**: ✅ Category +N tests, OR ❌ No improvement (explain why)
```

**Example:**
```markdown
## Improvements Made

### 1. Pattern 50 Fix: Japanese Filenames

**Issue**: Pattern 50 was not matching Japanese queries AND was over-matching English queries with false positives.

**Root Cause**: Pattern 50 had empty `required_keywords` and optional keywords `["find", "files", "search"]`. With empty required keywords, ALL queries were candidates. English queries like "delete all log files" matched because they contained optional keyword "files".

**Solution Applied**:
```rust
// Before
PatternEntry {
    required_keywords: vec![],
    optional_keywords: vec!["find".to_string(), "files".to_string()],
    regex_pattern: Some(Regex::new(r"[ぁ-んァ-ヶー一-龯]").unwrap()),
    // ...
}

// After
PatternEntry {
    required_keywords: vec![],
    optional_keywords: vec![],  // Empty to force regex-only matching
    regex_pattern: Some(Regex::new(r"[ぁ-んァ-ヶー一-龯]").unwrap()),
    // ...
}
```

**Rationale**: Removed ALL optional keywords to force regex-only matching. Pattern now ONLY matches if query contains Japanese characters.

**Impact**: ✅ File Management +1 test (i18n_001), fixed 3 false positives
```

### 4. Results Comparison

**Purpose**: Quantitative before/after metrics

**Required Tables:**
1. Overall metrics table
2. By-category breakdown table

**Template:**
```markdown
## Results Comparison

### Overall (All XX Test Cases)

| Metric | Cycle N-1 | Cycle N | Change |
|--------|-----------|---------|--------|
| **Pass Rate** | X.X% | **Y.Y%** | **+Z.Z%** 🎯 |
| **Passing Tests** | N | M | +K tests |
| **Failing Tests** | N | M | -K tests |
| **Pattern Count** | N | M | ±K patterns |

### By Category

| Category | Cycle N-1 | Cycle N | Change | Status |
|----------|-----------|---------|--------|--------|
| **Category 1** | X.X% | **Y.Y%** | **+N tests** | <emoji> <status> |
| **Category 2** | 100.0% | **100.0%** | maintained | 🎯 COMPLETE |
```

**Status Indicators:**
- 🎯 COMPLETE - 100% pass rate
- ⭐ Excellent - 80-99% pass rate
- ✅ Good - 60-79% pass rate
- ⚠️ Needs Work - 40-59% pass rate
- ❌ Critical - <40% pass rate or intentionally 0%

### 5. Target Achievement

**Purpose**: Compare planned goals vs actual results

**Template:**
```markdown
## Target Achievement

### Cycle N Targets vs Actuals

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Overall pass rate | X%+ | **Y.Y%** | ✅ **Exceeded** / ⚠️ **Close** / ❌ **Missed** |
| Category Name | 100% | **Y.Y%** | ✅ **Achieved** / ⚠️ **Progress** |

**Analysis**: <Explanation of results, why targets were met/missed, insights>
```

### 6. Pattern Impact Analysis

**Purpose**: Evaluate effectiveness of each fix

**Required Content:**
- Which fixes worked (and why)
- Which fixes didn't work (and why)
- Unexpected side effects
- Reordering success rate

**Template:**
```markdown
## Pattern Impact Analysis

### Successful Fixes (N/M = X% success rate)

**Fix 1: <Description>** ✅
- **Before**: <What happened before>
- **After**: <What happens now>
- **Result**: <Tests fixed, pass rate change>

### Unsuccessful Fixes (if any)

**Fix X: <Description>** ❌
- **Expected**: <What we hoped would happen>
- **Actual**: <What actually happened>
- **Root Cause**: <Why it didn't work>
- **Next Steps**: <How to fix in next cycle>
```

### 7. Pattern Distribution

**Purpose**: Show pattern allocation across categories

**Template:**
```markdown
## Pattern Distribution (XX total)

| Category | Patterns | Coverage | Notes |
|----------|----------|----------|-------|
| **Category 1** | N | XX.X% → YY.Y% | <Status> |
| **Category 2** | N | XX.X% → YY.Y% | <Status> |

### High ROI Changes (Cycle N)

1. **<Fix description>** (±N patterns → +M tests): <ROI calculation>
2. **<Fix description>** (±N patterns → +M tests): <ROI calculation>
```

**ROI Calculation:**
- Infinite ROI: 0 new patterns, +N tests (reorderings)
- High ROI: +1 pattern, +3+ tests
- Medium ROI: +1 pattern, +1-2 tests
- Low ROI: +1 pattern, +0 tests (needs adjustment)

### 8. What's Still Missing

**Purpose**: Identify remaining gaps for future cycles

**Template:**
```markdown
## What's Still Missing

### Zero Coverage Categories (N remaining)
1. **Category Name** (0/N tests) - <Reason>

### Partial Coverage Gaps

- **Category Name** (M/N passing, K still failing)
  - Pattern X issue - <Description>
  - Pattern Y issue - <Description>
  - Test failing: "<test name>"
  - Expected: `<expected command>`
  - Got: `<actual command>`
```

### 9. Lessons Learned

**Purpose**: Capture insights for future cycles

**Required Sections:**
- What worked exceptionally well (repeat these)
- What needs improvement (avoid these)
- Optimization opportunities (try these next)

**Template:**
```markdown
## Lessons Learned

### What Worked Exceptionally Well

1. **<Strategy>**: <Why it worked>
   - <Specific result 1>
   - <Specific result 2>

### What Needs Improvement

1. **<Strategy that didn't work>**: <Why it didn't work>
   - <Specific issue>
   - <What to try instead>

### Optimization Opportunities

1. **<Potential improvement>**: <Why it's promising>
   - <Expected impact>
   - <When to try it>
```

### 10. Next Steps

**Purpose**: Prioritize work for next cycle

**Template:**
```markdown
## Next Steps for Cycle N+1

### Priority 1: <Highest Priority Fix>
<Description of what to do>

**Expected Impact**: Category X% → Y% (+N tests)

### Priority 2: <Second Priority Fix>
<Description of what to do>

**Expected Impact**: Category X% → Y% (+N tests)

### Priority 3: <Third Priority Fix>
<Description of what to do>

**Expected Impact**: Overall X% → Y%
```

### 11. Detailed Test Results

**Purpose**: Provide granular test-level information

**Template:**
```markdown
## Detailed Test Results

### Newly Passing Tests (N)

| Test ID | Input | Expected Output | Category | Pattern | Fix |
|---------|-------|-----------------|----------|---------|-----|
| xx_NNN | <query> | `<command>` | <category> | Pattern N | <fix description> |

### Still Failing Tests (if analyzing)

| Test ID | Category | Issue | Root Cause |
|---------|----------|-------|------------|
| xx_NNN | <category> | <what's wrong> | <why it's failing> |
```

### 12. Cumulative Progress

**Purpose**: Show long-term trends across all cycles

**Template:**
```markdown
## Cumulative Progress (Cycle 0 → Cycle N)

| Metric | Cycle 0 | Cycle 1 | ... | Cycle N | Total Change |
|--------|---------|---------|-----|---------|--------------|
| **Pass Rate** | X.X% | Y.Y% | ... | **Z.Z%** | **+PPP%** 🚀 |
| **Passing Tests** | N | M | ... | P | +K tests |
| **Pattern Count** | N | M | ... | P | ±K patterns |
| **Completed Categories** | 0 | N | ... | **M** | +M categories |

**Completed Categories Timeline**:
- Cycle X: Category A (100%)
- Cycle Y: Category B (100%), Category C (100%)
- Cycle N: Category D (100%)

**Overall Achievement**: In N cycles, increased pass rate by X% (A% → B%) and completed M categories with P patterns
```

### 13. Conclusion

**Purpose**: Synthesize key takeaways

**Template:**
```markdown
## Conclusion

**Cycle N <successfully/achieved/exceeded expectations>**, achieving:
- X.X% overall pass rate (<met/exceeded> Y% target!)
- **100% pass rate for Category A** (<key achievement>)
- N complete categories (was M, gained K)
- Net ±K pattern count while improving pass rate by +N tests

**Key Achievement**: <Most important accomplishment>

**Pattern <Strategy> Success**: <Validation of approach used>

**Milestone**: At X.X%, we've <milestone description>. <Context about where we are>.

**Next Focus**:
1. **<Priority 1>** → Expected outcome
2. **<Priority 2>** → Expected outcome
3. **<Priority 3>** → Expected outcome
4. Target: Push toward **Y%+** in Cycle N+1

---

**Related**: Issue #395 (Beta Testing Cycles)
**Previous**: Cycle N-1 - <Title>
**Next**: Cycle N+1 - <Title>
```

## Writing Guidelines

### Tone and Style

**DO:**
- ✅ Use objective, data-driven language
- ✅ Include specific metrics (percentages, counts)
- ✅ Explain reasoning behind decisions
- ✅ Celebrate successes (with emojis when appropriate)
- ✅ Be honest about failures and lessons learned
- ✅ Use markdown formatting for readability

**DON'T:**
- ❌ Use vague language ("better", "improved" without numbers)
- ❌ Skip sections (every section serves a purpose)
- ❌ Omit root cause analysis
- ❌ Hide failures or make excuses
- ❌ Write walls of text (use tables, lists, code blocks)

### Metrics Presentation

**Percentages:**
- Always show before → after: "75.9% → 82.8%"
- Include absolute change: "+6.9 percentage points"
- Include count change when relevant: "+4 tests"

**Pattern Changes:**
- Show net change: "44 → 43 (net -1 after cleanup)"
- Explain removals: "(removed duplicate Pattern 48)"
- Clarify reorderings: "(Pattern 46 → Pattern 1)"

**Categories:**
- List complete categories: "Git, DevOps, Network, Text Processing, Log Analysis"
- Use status emojis: 🎯 COMPLETE, ⭐ Excellent, ⚠️ Needs Work
- Show progression: "5 → 7 complete categories"

### Code Examples

When showing before/after code:

```markdown
**Before (Cycle N-1, 44 patterns)**:
```rust
// Pattern 5: "files larger than 10MB" (general)
PatternEntry {
    required_keywords: vec!["file", "10mb"],
    // ...
},
```

**After (Cycle N, 43 patterns)**:
```rust
// Pattern 5: "PDF files larger than 10MB in Downloads" (SPECIFIC - moved from Pattern 41)
PatternEntry {
    required_keywords: vec!["pdf", "downloads"],
    // ...
},
// Pattern 6: "files larger than 10MB" (GENERAL - was Pattern 5)
PatternEntry {
    required_keywords: vec!["file", "10mb"],
    // ...
},
```
```

**Key elements:**
- Show cycle and pattern count in header
- Include pattern comments from code
- Show complete context (not just changed lines)
- Explain reordering in comments

### Root Cause Analysis

Every fix should include root cause analysis:

**Template:**
```markdown
**Issue**: <Observable problem>

**Root Cause**: <Why it was happening>
- <Technical detail 1>
- <Technical detail 2>
- Result: <What this caused>

**Solution**: <What we did to fix it>
```

**Example:**
```markdown
**Issue**: Pattern 50 was over-matching English queries

**Root Cause**: Pattern 50 had empty `required_keywords` but non-empty `optional_keywords`
- With empty required keywords, ALL queries were candidates
- Matching logic checks regex first, but falls back to keywords if regex doesn't match
- English queries like "delete all log files" matched via optional keyword "files"
- Result: 3 Dangerous Commands tests incorrectly matched Pattern 50

**Solution**: Removed all optional keywords to force regex-only matching
```

## Common Mistakes

### Mistake 1: Incomplete Metrics

❌ **Bad**: "Pass rate improved"

✅ **Good**: "Pass rate: 82.8% → 86.2% (+3.4 percentage points, +2 tests)"

### Mistake 2: No Root Cause

❌ **Bad**: "Fixed Pattern 50 by removing keywords"

✅ **Good**: "Pattern 50 had optional keywords causing false positives. Removed keywords to force regex-only matching, preventing English queries from matching Japanese pattern."

### Mistake 3: Missing Validation

❌ **Bad**: "Applied fix, should work"

✅ **Good**: "Applied fix, tested, verified:
- ✅ Target test now passes
- ✅ No regressions (all previously passing tests still pass)
- ✅ Pass rate improved from 82.8% to 86.2%"

### Mistake 4: Vague Next Steps

❌ **Bad**: "Improve patterns more in next cycle"

✅ **Good**: "Priority 1: Fix Pattern 3 vs 48 disk usage conflict
- Reorder Pattern 48 (4 keywords) before Pattern 3 (3 keywords)
- Expected Impact: System Monitoring 85.7% → 100% (+1 test)"

## Document Length

**Target**: 3,000-5,000 words for comprehensive cycles

**Sections by size:**
- Executive Summary: 100-200 words
- Improvements Made: 500-1,000 words (varies by fix count)
- Results Comparison: Tables (100-200 words text)
- Lessons Learned: 300-500 words
- Next Steps: 200-300 words
- Other sections: 100-300 words each

**If document exceeds 6,000 words**: Consider splitting detailed analysis into separate appendix file

## Checklist Before Publishing

- [ ] All required sections present
- [ ] Metrics are complete (before/after, change, percentages)
- [ ] Root cause explained for each fix
- [ ] Code examples show before/after
- [ ] Tables are formatted correctly
- [ ] Emojis used appropriately (not excessively)
- [ ] Lessons learned captured
- [ ] Next steps prioritized with expected impact
- [ ] Related issues/cycles linked at bottom
- [ ] Proofread for clarity and typos

## Example Structure

See `examples/cycle-analysis.md` for a complete, real-world example from Cycle 8 that follows all these guidelines.

## Summary

Good cycle documentation:
- **Tracks progress** with specific metrics
- **Explains reasoning** with root cause analysis
- **Validates approach** with before/after comparisons
- **Captures insights** for future cycles
- **Plans ahead** with prioritized next steps
- **Maintains history** with cumulative progress

Follow this template to create consistent, comprehensive cycle documentation that serves as both progress tracker and knowledge base for the project.
