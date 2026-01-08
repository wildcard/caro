# Pattern Gap Analyzer - Design Document

**Created**: 2026-01-08
**Status**: Implementation Ready
**Part of**: Week 1 Implementation Plan - Day 2

---

## Overview

The Pattern Gap Analyzer is an automated tool that analyzes safety patterns in `src/safety/patterns.rs` and identifies missing variants that attackers could exploit.

**Purpose**: Detect gaps in pattern coverage before they become security vulnerabilities.

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                  analyze-pattern-gaps.py             │
│                    (CLI Entry Point)                 │
└──────────────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────┐
│                   Pattern Parser                      │
│  - Extracts patterns from patterns.rs                │
│  - Parses regex into components                      │
│  - Returns structured pattern data                   │
└──────────────────────────────────────────────────────┘
                          │
                ┌─────────┴─────────┐
                ▼                   ▼
┌───────────────────────┐  ┌───────────────────────┐
│  Argument Order       │  │  Path Variant         │
│  Detector             │  │  Detector             │
│  - Finds flag swaps   │  │  - Relative paths     │
│  - Finds missing args │  │  - Absolute paths     │
└───────────────────────┘  └───────────────────────┘
                │                   │
                ▼                   ▼
┌───────────────────────┐  ┌───────────────────────┐
│  Wildcard             │  │  Platform Equivalent  │
│  Detector             │  │  Detector             │
│  - * patterns         │  │  - GNU vs BSD         │
│  - ? patterns         │  │  - PowerShell equiv   │
└───────────────────────┘  └───────────────────────┘
                │
                ▼
┌──────────────────────────────────────────────────────┐
│                   Report Generator                    │
│  - Aggregates findings                               │
│  - Prioritizes by risk                               │
│  - Outputs markdown report                           │
└──────────────────────────────────────────────────────┘
```

---

## Components

### 1. Pattern Parser (`pattern_analyzer/parser.py`)

**Responsibility**: Extract and parse patterns from Rust source.

**Input**: `src/safety/patterns.rs`

**Output**: List of structured patterns:
```python
Pattern = {
    'regex': str,           # The actual regex pattern
    'description': str,     # What it blocks
    'risk_level': str,      # Critical/High/Medium
    'shell_specific': Optional[str],  # bash/zsh/powershell/fish
    'command_base': str,    # Extracted command (rm, dd, etc)
}
```

**Key Functions**:
- `parse_patterns_file(filepath: str) -> List[Pattern]`
- `extract_command_from_regex(regex: str) -> str`
- `normalize_regex(regex: str) -> str`

---

### 2. Argument Order Detector (`pattern_analyzer/argument_detector.py`)

**Responsibility**: Detect missing argument order permutations.

**Example Gap**:
```
Present:  rm -rf /path
Missing:  rm /path -rf  # Flag after argument
Missing:  rm -r -f /path  # Separate flags
```

**Algorithm**:
1. Extract flags from pattern: `-rf`, `-R`, etc.
2. Generate permutations:
   - Combined flags: `-rf`, `-fr`
   - Separate flags: `-r -f`, `-f -r`
   - Flags after args: `path -rf`
   - Flags before and after: `-r path -f`
3. Check if pattern covers each permutation
4. Report missing coverage

**Key Functions**:
- `detect_argument_order_gaps(pattern: Pattern) -> List[Gap]`
- `generate_flag_permutations(flags: List[str]) -> List[str]`
- `check_pattern_covers(pattern: Pattern, variant: str) -> bool`

---

### 3. Path Variant Detector (`pattern_analyzer/path_detector.py`)

**Responsibility**: Detect missing path representation variants.

**Example Gap**:
```
Present:  rm -rf ..
Missing:  rm -rf ../  # Trailing slash
Missing:  rm -rf ../. # With dot
Missing:  rm -rf /abs/path/..  # Absolute with parent
```

**Path Categories**:
1. **Relative paths**: `.`, `..`, `../../`, etc.
2. **Absolute paths**: `/`, `/etc`, `/var`, etc.
3. **Current directory**: `./*`, `./`, etc.
4. **Wildcards in paths**: `../*`, `/**`, etc.

**Key Functions**:
- `detect_path_gaps(pattern: Pattern) -> List[Gap]`
- `generate_path_variants(base_path: str) -> List[str]`
- `extract_paths_from_pattern(pattern: Pattern) -> List[str]`

---

### 4. Wildcard Detector (`pattern_analyzer/wildcard_detector.py`)

**Responsibility**: Detect missing wildcard pattern coverage.

**Example Gap**:
```
Present:  rm *
Missing:  rm ./*  # Explicit current dir
Missing:  rm ./*.txt  # With extension
Missing:  rm **  # Recursive glob
```

**Wildcard Types**:
1. `*` - Match anything
2. `?` - Match single char
3. `**` - Recursive match (some shells)
4. `[...]` - Character class
5. `{a,b}` - Brace expansion

**Key Functions**:
- `detect_wildcard_gaps(pattern: Pattern) -> List[Gap]`
- `generate_wildcard_variants(base: str) -> List[str]`
- `check_shell_glob_support(shell: str, glob: str) -> bool`

---

### 5. Platform Equivalent Detector (`pattern_analyzer/platform_detector.py`)

**Responsibility**: Detect missing cross-platform command equivalents.

**Example Gap**:
```
Present:  rm -rf (POSIX)
Missing:  Remove-Item -Recurse -Force (PowerShell)
Missing:  del /s /q (Windows CMD)
```

**Platform Mappings**:
```python
PLATFORM_EQUIVALENTS = {
    'rm': {
        'posix': ['rm', 'unlink'],
        'powershell': ['Remove-Item', 'ri', 'rm', 'del'],
        'cmd': ['del', 'erase', 'rd', 'rmdir'],
    },
    'dd': {
        'posix': ['dd'],
        'powershell': ['dd', 'Write-Output'],  # Rare
        'cmd': None,  # No direct equivalent
    },
    # ... more mappings
}
```

**Key Functions**:
- `detect_platform_gaps(pattern: Pattern) -> List[Gap]`
- `get_platform_equivalents(command: str) -> Dict[str, List[str]]`
- `check_pattern_platform_coverage(pattern: Pattern) -> Dict[str, bool]`

---

## Gap Data Structure

```python
Gap = {
    'type': str,  # 'argument_order' | 'path_variant' | 'wildcard' | 'platform'
    'severity': str,  # 'critical' | 'high' | 'medium' | 'low'
    'original_pattern': str,
    'missing_variant': str,
    'example_command': str,  # Concrete example that would bypass
    'recommendation': str,   # Suggested regex fix
    'affected_command': str, # Base command (rm, dd, etc)
}
```

---

## CLI Interface

```bash
# Basic usage
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs

# Output to file
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs -o gaps.md

# Filter by severity
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs --min-severity high

# Filter by detector
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs --detector argument

# JSON output
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs --format json
```

**Output Format** (Markdown):
```markdown
# Safety Pattern Gap Analysis

**Generated**: 2026-01-08 13:45:00 UTC
**Patterns Analyzed**: 52
**Gaps Found**: 15 (8 critical, 5 high, 2 medium)

---

## Critical Gaps (8)

### Gap 1: Argument Order - rm command
**Severity**: Critical
**Original**: `rm\s+-[rfRF]+\s+\.\./?`
**Missing**: `rm .. -rf` (flags after path)
**Example**: `rm ../sensitive -rf`
**Recommendation**: Add pattern: `rm\s+\.\./?.*-[rfRF]+`

---

## High Gaps (5)

...
```

---

## Implementation Priority

### Phase 1 (Day 2 Morning):
1. ✅ Parser module (1.5h)
2. ✅ Argument detector (1h)

### Phase 2 (Day 2 Afternoon):
3. ✅ Path detector (1h)
4. ✅ Wildcard detector (30min)
5. ✅ Platform detector (1h)
6. ✅ CLI + Report generation (1h)

---

## Testing Strategy

### Unit Tests (`scripts/tests/test_pattern_gap_analyzer.py`):

```python
def test_parser_extracts_patterns():
    patterns = parse_patterns_file('test_patterns.rs')
    assert len(patterns) > 0
    assert patterns[0]['regex'] is not None

def test_argument_detector_finds_gaps():
    pattern = {'regex': r'rm\s+-rf\s+\.\./'}
    gaps = detect_argument_order_gaps(pattern)
    assert any(g['missing_variant'] == 'rm .. -rf' for g in gaps)

def test_path_detector_finds_variants():
    pattern = {'regex': r'rm\s+-rf\s+\.\.'}
    gaps = detect_path_gaps(pattern)
    assert any('../' in g['missing_variant'] for g in gaps)
```

### Integration Test:
```bash
# Should find known gaps in current patterns
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs | grep -c "CRITICAL"
# Expected: > 0 (we know there are gaps)
```

---

## Error Handling

1. **File Not Found**: Clear error message with expected path
2. **Invalid Regex**: Skip pattern, log warning, continue
3. **Parse Errors**: Show line number, continue with next pattern
4. **No Patterns Found**: Exit with error code 1

---

## Performance Considerations

- **Target**: Analyze 52 patterns in < 2 seconds
- **Optimization**: Cache regex compilation
- **Parallelization**: Not needed for 52 patterns
- **Memory**: Keep full report in memory (< 1MB expected)

---

## Future Enhancements (Out of Scope for Week 1)

1. **Auto-fix mode**: Generate corrected regex automatically
2. **CI Integration**: Fail PR if critical gaps introduced
3. **Historical tracking**: Track gap trends over time
4. **Interactive mode**: Review and apply fixes interactively
5. **Machine learning**: Learn patterns from test failures

---

## Success Criteria

✅ Detects all 4 gap types (argument, path, wildcard, platform)
✅ Produces actionable recommendations
✅ Runs in < 2 seconds
✅ CLI is easy to use
✅ Output is clear and prioritized
✅ Integration tests pass
✅ Can be run in CI/CD

---

## Related Documents

- Week 1 Plan: `.claude/recommendations/one-week-implementation-plan.md`
- TDD Workflow: `CONTRIBUTING.md` (Safety Pattern Development section)
- Contribution Workflow: `.claude/workflows/add-safety-pattern.md`
