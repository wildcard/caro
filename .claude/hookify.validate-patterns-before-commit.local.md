---
name: validate-patterns-before-commit
enabled: true
event: bash
pattern: git\s+commit
action: warn
---

⚠️ **Safety Pattern Validation Required Before Commit**

You're about to commit changes. Before proceeding, verify:

## Validation Checklist

**If `src/safety/patterns.rs` was modified:**

1. **✅ Pattern Compilation Check**
   ```bash
   cargo build --lib --quiet 2>&1 | grep -i "error"
   ```
   - Must compile without errors
   - Regex patterns must be valid

2. **✅ Pattern Coverage Check**
   ```bash
   # Check for common gaps (if gap analyzer exists)
   ./scripts/analyze-pattern-gaps.py src/safety/patterns.rs 2>/dev/null || echo "Gap analyzer not available - manual review needed"
   ```

3. **✅ Test Suite Verification**
   ```bash
   cargo test --lib safety::patterns --quiet
   ```
   - All pattern tests must pass

4. **✅ No Regressions**
   ```bash
   ./target/release/caro test --backend static --suite .claude/beta-testing/test-cases.yaml 2>/dev/null
   ```
   - Verify baseline pass rate maintained

## Quick Validation Script

Run this before committing pattern changes:
```bash
# Quick validation
echo "=== Checking pattern compilation ==="
cargo build --lib --quiet && echo "✅ Patterns compile" || echo "❌ Compilation failed"

echo "=== Checking for staged patterns.rs ==="
git diff --cached --name-only | grep -q "src/safety/patterns.rs" && echo "⚠️  patterns.rs is staged - validation required" || echo "✅ No pattern changes"
```

## Why This Matters

**Pattern compilation errors** can break the safety validation system entirely.
**Pattern gaps** leave users vulnerable to dangerous commands.
**Regressions** can block safe commands or allow dangerous ones through.

**Recommendation**: Run validation checks before this commit, or commit without patterns.rs changes first.
