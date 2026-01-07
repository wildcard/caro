# Beta Test Cycle 0: Baseline Results

**Date**: 2026-01-07
**Version**: caro X.Y.Z (commit: 15b7b40)
**Backend**: static (StaticMatcher)
**Test Suite**: .claude/beta-testing/test-cases.yaml
**Total Available Test Cases**: 58 (loaded from YAML)

## Executive Summary

Established baseline metrics for command generation quality using static pattern matcher across 58 test cases organized into 8 categories. Results show **10.3% overall pass rate** with significant gaps in all categories except file management.

**Key Findings:**
- Only 6 patterns in static matcher vs 58 test cases (10% coverage)
- All 52 failures due to missing static patterns (100% static miss rate)
- File Management has best coverage (26.3%) but still needs expansion
- System Monitoring, DevOps, and Network categories have near-zero coverage
- Profile-based filtering working correctly (bt_001: 16 tests, bt_002: 31 tests, bt_005: 27 tests)

---

## Overall Results (No Profile Filter)

### Summary

| Metric | Value |
|--------|-------|
| **Total Tests** | 58 |
| **Passed** | 6 |
| **Failed** | 52 |
| **Pass Rate** | **10.3%** |
| **Backend** | static (StaticMatcher) |

### Results by Category

| Category | Passed | Total | Pass Rate |
|----------|--------|-------|-----------|
| **File Management** | 5 | 19 | **26.3%** ✓ |
| **System Monitoring** | 1 | 7 | 14.3% |
| **DevOps/Kubernetes** | 0 | 5 | 0.0% |
| **Text Processing** | 0 | 7 | 0.0% |
| **Git Version Control** | 0 | 3 | 0.0% |
| **Dangerous Commands** | 0 | 8 | 0.0% |
| **Log Analysis** | 0 | 4 | 0.0% |
| **Network Operations** | 0 | 5 | 0.0% |

---

## Profile-Specific Results

### bt_001: Alex (Terminal Novice)

**Environment**: macOS, zsh
**Focus**: Basic file operations, beginner-friendly examples

| Metric | Value |
|--------|-------|
| **Total Tests** | 16 (filtered) |
| **Passed** | 4 |
| **Failed** | 12 |
| **Pass Rate** | **25.0%** |

**Category Breakdown:**
- File Management: 4/14 (28.6%)
- Text Processing: 0/1 (0.0%)
- Dangerous Commands: 0/1 (0.0%)

**Notable**: Highest pass rate of all profiles due to focus on basic file operations.

---

### bt_002: Jordan (Power User)

**Environment**: Linux Ubuntu 22.04, zsh
**Focus**: Advanced features, system monitoring, git commands, text processing

| Metric | Value |
|--------|-------|
| **Total Tests** | 31 (filtered) |
| **Passed** | 4 |
| **Failed** | 27 |
| **Pass Rate** | **12.9%** |

**Category Breakdown:**
- File Management: 3/8 (37.5%)
- System Monitoring: 1/7 (14.3%)
- Git Version Control: 0/3 (0.0%)
- Text Processing: 0/4 (0.0%)
- Log Analysis: 0/3 (0.0%)
- Network Operations: 0/3 (0.0%)
- DevOps/Kubernetes: 0/2 (0.0%)
- Dangerous Commands: 0/1 (0.0%)

**Notable**: Broader test coverage across advanced categories, revealing gaps in non-file-management areas.

---

### bt_005: Taylor (SRE/Ops Engineer)

**Environment**: Linux Ubuntu 22.04, bash
**Focus**: DevOps workflows, Kubernetes, dangerous command safety, log analysis

| Metric | Value |
|--------|-------|
| **Total Tests** | 27 (filtered) |
| **Passed** | 1 |
| **Failed** | 26 |
| **Pass Rate** | **3.7%** ⚠️ |

**Category Breakdown:**
- System Monitoring: 1/6 (16.7%)
- DevOps/Kubernetes: 0/5 (0.0%)
- Network Operations: 0/4 (0.0%)
- Log Analysis: 0/4 (0.0%)
- Text Processing: 0/1 (0.0%)
- Dangerous Commands: 0/7 (0.0%)

**Notable**: Lowest pass rate due to focus on advanced DevOps/SRE commands with no static pattern coverage.

---

## Failure Analysis

### Root Cause Distribution

| Root Cause | Count | Percentage |
|------------|-------|------------|
| **Static Miss** (No pattern match) | 52 | **100%** |
| LLM Hallucination | 0 | 0% |
| Platform Mismatch | 0 | 0% |
| Parse Failure | 0 | 0% |
| Timeout | 0 | 0% |

**Analysis**: Every single failure is due to missing static patterns. The static matcher only has 6 patterns but the test suite requires 58 different command types.

---

## Passing Test Cases (6 total)

These are the **only** patterns currently supported by the static matcher:

| ID | Input | Expected Output | Category |
|----|-------|-----------------|----------|
| fm_001 | list all files modified today | `find . -type f -mtime 0` | File Management |
| fm_002 | find large files over 100MB | `find . -type f -size +100M` | File Management |
| fm_003 | find python files modified last week | `find . -name "*.py" -type f -mtime -7` | File Management |
| fm_004 | list all files modified today | `find . -type f -mtime 0` | File Management |
| fm_006 | show all files modified today | `find . -type f -mtime 0` | File Management |
| sm_002 | show disk usage by folder | `du -sh */ | sort -rh | head -10` | System Monitoring |

**Pattern**: 5 out of 6 are file management, primarily using `find` with `-mtime` and `-size` flags.

---

## Top 10 Critical Failures (By Priority)

Based on test category priority (P0 > P1 > P2) and profile assignment:

### 1. File Management - Size Filters (P1)
**Failing Tests**:
- "list all files larger than 10MB" → `find . -type f -size +10M`
- "list all files larger than 1GB" → `find . -type f -size +1G`
- "Find all files larger than 1GB" → `find . -type f -size +1G -exec ls -lh {} \;`

**Issue**: Static matcher has pattern for `+100M` but not `+10M` or `+1G`
**Fix**: Expand size filter patterns to handle common sizes (1M, 5M, 10M, 50M, 100M, 500M, 1G, 10G)

### 2. File Management - Extension + Time Filters (P1)
**Failing Tests**:
- "find python files modified in the last 7 days" → `find . -name "*.py" -type f -mtime -7`
- "List all PNG images modified in the last 7 days" → `find . -name '*.png' -type f -mtime -7`

**Issue**: Static matcher has pattern for Python files with `-mtime -7` but not generalized for other extensions
**Fix**: Generalize pattern to match any file extension with time filters

### 3. File Management - Time Filters (Minutes) (P1)
**Failing Tests**:
- "find files changed in last hour" → `find . -type f -mmin -60`
- "show files modified in the last hour" → `find . -type f -mmin -60`

**Issue**: Static matcher has `-mtime` patterns but not `-mmin`
**Fix**: Add `-mmin` patterns for minute-based time filters

### 4. System Monitoring - Process Management (P1)
**Failing Tests**:
- "Show top 10 memory-consuming processes" → `ps aux --sort=-%mem | head -n 11`
- "Monitor CPU usage in real-time" → `top -b -n 1 | head -n 20`

**Issue**: No process monitoring patterns exist
**Fix**: Add patterns for `ps`, `top`, process sorting

### 5. System Monitoring - Port/Network (P1)
**Failing Tests**:
- "Check which process is using port 8080" → `lsof -i :8080`

**Issue**: No `lsof` patterns exist
**Fix**: Add patterns for `lsof` with port specifications

### 6. Text Processing - grep/awk (P2)
**Failing Tests**:
- "search for TODO in all python files" → `grep -r "TODO" --include="*.py" .`
- "Extract IPs from log file" → `grep -oE '\b([0-9]{1,3}\.){3}[0-9]{1,3}\b' /var/log/syslog`

**Issue**: No text processing patterns exist
**Fix**: Add patterns for `grep`, `awk`, `sed` with common use cases

### 7. DevOps/Kubernetes (P2)
**Failing Tests**:
- All 5 DevOps/Kubernetes tests fail (0% pass rate)

**Issue**: No `kubectl`, `docker`, `terraform` patterns exist
**Fix**: Add patterns for common DevOps commands

### 8. Network Operations (P2)
**Failing Tests**:
- All 5 network operation tests fail (0% pass rate)

**Issue**: No `ping`, `ss`, `wget`, `curl` patterns exist
**Fix**: Add patterns for basic network diagnostics

### 9. Git Version Control (P2)
**Failing Tests**:
- All 3 git tests fail (0% pass rate)

**Issue**: No `git` patterns exist
**Fix**: Add patterns for common git operations

### 10. Dangerous Commands (P2)
**Failing Tests**:
- All 8 dangerous command tests fail (0% pass rate)

**Issue**: Safety validation system not integrated with static matcher
**Fix**: Integrate safety validator to block dangerous patterns

---

## Improvement Opportunities from PRs/Issues

Based on existing PRs and design docs, identified opportunities:

### From Issue #274: Command Validation Pipeline
- Integrate `CommandValidator` into test runner
- Use validation errors to identify safety issues
- Test safety blocking for dangerous commands category

### From Issue #152: Local Directory Context Awareness
- Add patterns that use current directory context
- Test with `pwd`, `ls`, directory-specific commands

### From Issue #147: Machine Resource Assessment
- Add patterns for resource monitoring (`free`, `df`, `iostat`)
- Relevant for System Monitoring category

### From Design Docs
- **Command Help Caching**: Cache `--help` output for common commands
- **Multi-step Refinement**: Implement validation-triggered retry loop
- **Chain-of-Thought Prompting**: Add reasoning before command generation

---

## Recommended Actions for Cycle 1 (Quick Wins)

### Priority 1: Expand Static Matcher (Target: 70%+ pass rate for File Management)

**File**: `src/backends/static_matcher.rs`

Add 15 new patterns:
1. File size filters: +1M, +5M, +10M, +50M, +500M, +1G, +5G, +10G
2. Time filters with minutes: `-mmin -5`, `-mmin -15`, `-mmin -30`, `-mmin -60`
3. Extension + time combinations: `*.{js,ts,md,txt,log,json}` + `-mtime -N`
4. Directory-specific finds: `find ~/Downloads`, `find ~/Documents`
5. Execute actions: `-exec ls -lh {} \;`, `-exec rm {} \;` (with safety validation)

**Expected Impact**: File Management 26.3% → 70%+

### Priority 2: Fix LLM Temperature (Target: More deterministic output)

**File**: `src/backends/embedded/embedded_backend.rs`

Change temperature from 0.7 to 0.1:
```rust
// Current: temperature: 0.7
// Fix: temperature: 0.1
```

**Expected Impact**: Reduce variability in LLM fallback (for future cycles when LLM is tested)

### Priority 3: Add Few-Shot Examples (Target: 60%+ pass rate for System Monitoring)

**File**: `src/prompts/smollm_prompt.rs`

Add 10-15 examples for:
- Process monitoring: `ps aux`, `top`, `htop`
- Port checking: `lsof -i :PORT`, `ss -tulpn`
- Disk usage: `du -sh`, `df -h`

**Expected Impact**: System Monitoring 14.3% → 60%+ (when LLM backend is used)

---

## Target Pass Rates

| Category | Baseline | Cycle 1 Target | Cycle 2 Target | Final Target |
|----------|----------|----------------|----------------|--------------|
| **File Management** | 26.3% | **70%+** | 80%+ | 90%+ |
| **System Monitoring** | 14.3% | 40%+ | **60%+** | 75%+ |
| **Text Processing** | 0.0% | 30%+ | **50%+** | 70%+ |
| **DevOps/Kubernetes** | 0.0% | 20%+ | 40%+ | **60%+** |
| **Git Version Control** | 0.0% | 50%+ | **70%+** | 85%+ |
| **Network Operations** | 0.0% | 30%+ | **50%+** | 70%+ |
| **Log Analysis** | 0.0% | 30%+ | **50%+** | 70%+ |
| **Dangerous Commands** | 0.0% | Block all | Block all | **Block all** |
| **Overall** | **10.3%** | **50%+** | **65%+** | **75%+** |

---

## Next Steps

1. **Immediate** (Cycle 1): Expand static matcher with 15+ new patterns (Priority 1 failures)
2. **Short-term** (Cycle 2): Fix temperature, add few-shot examples, add chain-of-thought prompting
3. **Medium-term** (Cycle 3): Implement validation-triggered retry loop
4. **Long-term** (Cycle 4+): Intent classification, command help caching, model selection by complexity

---

## Test Infrastructure Status

✅ **Working**:
- YAML test suite loading (`--suite` flag)
- Profile filtering (`--profile` flag)
- Test runner with category grouping
- Pass/fail metrics by category
- Static matcher backend

⚠️ **Needs Improvement**:
- LLM backend testing (temperature too high)
- Safety validation integration
- Retry loop for failed commands
- Performance metrics (latency, throughput)

❌ **Missing**:
- Automated CI testing
- Regression tracking across cycles
- Visual diff tool for command comparison
- Platform-specific test variants (GNU/BSD/Windows)

---

## Conclusion

Baseline testing confirms that the static matcher covers only 10.3% of test cases, with significant gaps in all categories. The primary issue is **insufficient static pattern coverage** (6 patterns vs 58 test cases needed).

**Critical Path for Improvement**:
1. Expand static matcher patterns (Cycle 1)
2. Improve prompt engineering and temperature (Cycle 2)
3. Add validation-retry loop (Cycle 3)
4. Advanced features: intent classification, help caching (Cycle 4+)

**Success Criteria**: Achieve 75%+ overall pass rate while maintaining 100% block rate for dangerous commands.

---

**Related**: Issue #395 (Beta Testing Cycles)
**Next Cycle**: Cycle 1 - Quick Wins (Expand Static Matcher)
