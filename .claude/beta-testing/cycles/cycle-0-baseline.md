# Cycle 0: Baseline Testing Results

**Date**: 2026-01-08
**Backend**: Static Matcher
**Total Test Cases**: 11 (from default eval suite)
**Overall Pass Rate**: 5/11 (45.5%)

---

## Pass Rates by Category

| Category | Passed | Total | Pass Rate |
|----------|--------|-------|-----------|
| Website Claim | 4 | 4 | 100.0% ✓ |
| Natural Variant | 1 | 5 | 20.0% |
| Edge Case | 0 | 2 | 0.0% |

---

## Successful Test Cases (5/11)

### Website Claims (4/4) - **100% PASS RATE** ✓
These are the P0 tests from website documentation:

1. ✅ **"show largest files"** → `find . -type f -exec ls -lh {} + | sort -k5 -rh | head -10`
2. ✅ **"files modified in the last 7 days"** → `find . -type f -mtime -7`
3. ✅ **"list all python files"** → `find . -name "*.py" -type f`
4. ✅ **"files larger than 10MB"** → `find . -type f -size +10M`

### Natural Variants (1/5) - 20% pass rate
5. ✅ **"show me big files"** → Matched to largest files pattern

---

## Failed Test Cases (6/11)

### Natural Variant Failures (4/5)

#### 1. "files changed today"
- **Expected**: `find . -type f -mtime 0`
- **Error**: Backend is not available: No static pattern match found
- **Root Cause**: **Static miss** - Pattern not in static matcher
- **Priority**: High (common time-based query)

#### 2. "list files bigger than 100 megabytes"
- **Expected**: `find . -type f -size +100M`
- **Got**: `find . -type f -size +10M`
- **Root Cause**: **Size threshold mismatch** - Static pattern only has 10M threshold
- **Priority**: Medium (needs parameterized matching)

#### 3. "disk space used by each folder"
- **Expected**: `du -sh */ | sort -rh | head -10`
- **Error**: Backend is not available: No static pattern match found
- **Root Cause**: **Static miss** - du command pattern not in matcher
- **Priority**: High (common disk usage query)

#### 4. "python files from the last 7 days"
- **Expected**: `find . -name "*.py" -type f -mtime -7`
- **Got**: `find . -type f -mtime -7`
- **Root Cause**: **Partial match** - Matched time pattern but lost file extension
- **Priority**: High (compound query with extension + time)

### Edge Case Failures (2/2)

#### 5. "files modified yesterday"
- **Expected**: `find . -type f -mtime 1`
- **Error**: Backend is not available: No static pattern match found
- **Root Cause**: **Static miss** - Specific time offset (yesterday = mtime 1) not in matcher
- **Priority**: Medium (less common than "today")

#### 6. "large javascript files over 50MB"
- **Expected**: `find . -name "*.js" -type f -size +50M`
- **Got**: `find . -type f -size +50M`
- **Root Cause**: **Partial match** - Matched size pattern but lost file extension
- **Priority**: High (compound query with extension + size)

---

## Root Cause Distribution

| Root Cause | Count | Percentage |
|------------|-------|------------|
| Static miss (no pattern) | 4 | 66.7% |
| Partial match (compound query) | 2 | 33.3% |

**Key Insight**: 100% of failures are due to static matcher limitations, not LLM quality issues.

---

## Next Steps (Cycle 1)

Focus on expanding static patterns to address the 6 failures identified above.

**Target**: Increase Natural Variant pass rate from 20% to 70%+

