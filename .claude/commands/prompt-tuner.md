---
description: Improve embedded LLM system prompt based on evaluation test failures
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/prompt-tuner` | Full tuning cycle: test -> analyze -> improve -> verify |
| `/prompt-tuner --baseline` | Just run baseline tests, no changes |
| `/prompt-tuner --analyze` | Analyze last test results without re-running |

---

## What This Command Does

`/prompt-tuner` systematically improves the embedded backend's system prompt to increase command generation accuracy. It follows a data-driven approach:

1. **Measure** current accuracy with evaluation tests
2. **Analyze** failure patterns to identify root causes
3. **Improve** system prompt with targeted fixes
4. **Verify** improvement by re-running tests
5. **Commit** if significant improvement achieved

---

## Workflow

### Step 1: Baseline Measurement

First, ensure release build is current:

```bash
cargo build --release 2>&1 | tail -3
```

Run evaluation tests with embedded backend:

```bash
./target/release/caro test --backend embedded
```

**Record these metrics:**
- Overall: X/Y (Z%)
- By category: Website Claim, Natural Variant, Edge Case
- Failed test cases list

### Step 2: Failure Analysis

For each failed test, identify the pattern:

| Pattern | Example Got vs Expected | Fix Strategy |
|---------|-------------------------|--------------|
| Wrong path | `find /` vs `find .` | Add rule: "Use current directory '.'" |
| GNU flags | `--max-depth` on macOS | Add rule: "BSD-compatible only" |
| Missing filter | No `-name "*.py"` | Add rule: "Include all filters" |
| Time error | `-mtime -1` vs `-mtime 1` | Add mtime documentation |
| Order diff | `-type f -name` vs `-name -type f` | **Equivalent** - not a real failure |

**Important**: Flag ordering and quote style differences are usually semantically equivalent!

### Step 3: Improve System Prompt

Edit the system prompt in:

```
src/backends/embedded/embedded_backend.rs
```

Look for function `create_system_prompt()`.

**Improvement strategies:**
1. Add explicit rules for each failure pattern
2. Include concrete examples matching test cases
3. Use numbered priority rules
4. Keep prompt concise (small models struggle with verbosity)

### Step 4: Verify Improvement

Build and re-test:

```bash
cargo build --release && ./target/release/caro test --backend embedded
```

**Compare results:**
- Did overall accuracy improve by >10%?
- Any category regressions?
- Are remaining failures semantic equivalents?

### Step 5: Commit or Iterate

**If improved significantly:**
```bash
git add src/backends/embedded/embedded_backend.rs
git commit -m "feat(prompt): Improve embedded accuracy from X% to Y%"
```

**If not improved:**
- Revert changes: `git checkout src/backends/embedded/embedded_backend.rs`
- Try different approach
- Consider if test expectations need adjustment

---

## Success Metrics

| Accuracy | Assessment | Recommendation |
|----------|------------|----------------|
| < 50% | Poor | Major prompt rewrite needed |
| 50-70% | Acceptable | Add targeted rules |
| 70-85% | Good | Fine-tune edge cases |
| > 85% | Excellent | Likely remaining are equivalents |

---

## Tips

1. **One change at a time** - easier to debug regressions
2. **Check equivalence** - some "failures" produce identical results
3. **BSD not GNU** - macOS uses BSD utilities
4. **Minimal examples** - too many confuse small models
5. **Test specific patterns** - use `--force-llm` flag for targeted testing

---

## Example Session

```bash
# Run baseline
./target/release/caro test --backend embedded
# Result: 27.3% (3/11)

# Analyze failures - see path issues, GNU flags, missing filters

# Edit src/backends/embedded/embedded_backend.rs
# Add rules for:
# - Always use "." not "/"
# - BSD-compatible flags
# - Include all filters
# - mtime semantics

# Rebuild and verify
cargo build --release
./target/release/caro test --backend embedded
# Result: 72.7% (8/11) - significant improvement!

# Commit
git add src/backends/embedded/embedded_backend.rs
git commit -m "feat(prompt): Improve embedded accuracy from 27% to 73%"
```
