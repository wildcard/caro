# Safety Pattern Developer Skill

**A disciplined TDD workflow for adding safety patterns to Caro**

## What This Skill Does

Guides you through the complete Test-Driven Development (TDD) cycle for adding safety patterns:

1. **Identify**: Document the threat and risk level
2. **Red**: Write tests FIRST (they fail)
3. **Green**: Implement pattern (tests pass)
4. **Verify**: Confirm 100% pass rate
5. **Refactor**: Check regressions + false positives
6. **Commit**: Document and commit with tests

## When to Use

- Adding a new dangerous command pattern
- Fixing a gap found by the gap analyzer
- Responding to a security vulnerability
- Improving existing pattern coverage

## Quick Start

```bash
# Invoke the skill
/safety-pattern-developer

# Or use directly
claude-code --skill safety-pattern-developer
```

The skill will guide you through each phase with:
- Clear action items
- Code examples
- Verification steps
- Success criteria

## Duration

- **Simple pattern**: 30-45 minutes
- **Complex pattern**: 1-2 hours

## Prerequisites

- Caro repository cloned
- Rust toolchain installed (`cargo build` works)
- Understanding of the dangerous command you want to block

## What You'll Create

By the end of the workflow, you'll have:

- ✅ Threat documentation
- ✅ 5+ test cases (YAML)
- ✅ Pattern implementation (Rust)
- ✅ 100% test pass rate
- ✅ Zero regressions
- ✅ Zero false positives
- ✅ Complete documentation
- ✅ Proper git commit

## Examples

See complete walkthroughs in `examples/`:

1. **`example-rm-rf-parent.md`**: Parent directory deletion
   - Duration: 45 minutes
   - Test cases: 5
   - Gap closed: GAP-003

2. **`example-dd-reverse.md`**: dd argument order attack
   - Duration: 1 hour
   - Test cases: 8
   - Patterns: 2 (both argument orders)
   - Gap closed: GAP-012

## The 6-Phase Workflow

### Phase 1: Threat Identification (5 min)
Document what command you're blocking and why.

### Phase 2: Write Tests FIRST (15 min) [RED]
Create test YAML with 5+ test cases. Run tests - they must FAIL.

### Phase 3: Implement Pattern (15 min) [GREEN]
Add DangerPattern to patterns.rs. Pattern must compile.

### Phase 4: Verify Tests Pass (5 min) [GREEN]
Run tests again - they must ALL PASS (100%).

### Phase 5: Full Test Suite (10 min) [REFACTOR]
- Check regressions (existing tests still pass)
- Test false positives (safe commands allowed)
- Run gap analyzer (verify gap closed)

### Phase 6: Document & Commit (10 min)
Add code comments, update test metadata, create descriptive commit.

## Success Criteria

Before marking complete, verify ALL of:

- [ ] All 6 phases completed in order
- [ ] Threat documented
- [ ] 5+ test cases written BEFORE implementation
- [ ] All tests pass (100%)
- [ ] No regressions
- [ ] No false positives
- [ ] Pattern documented in code
- [ ] Committed with tests

## Common Pitfalls

### ❌ Pattern too broad
**Symptom**: Blocks safe commands (false positives)
**Fix**: Narrow regex, add more context

### ❌ Pattern too specific
**Symptom**: Gap analyzer still finds variants
**Fix**: Use character classes, optional groups

### ❌ Missing flag orders
**Symptom**: `rm -rf path` blocked but `rm path -rf` not
**Fix**: Use alternation or make flags optional

### ❌ Regex syntax errors
**Symptom**: Pattern doesn't compile
**Fix**: Use raw strings `r"pattern"`, escape special chars

### ❌ Incomplete test coverage
**Symptom**: Pattern works but gaps remain
**Fix**: Run gap analyzer, add tests for each gap

## Tools & Resources

**Gap Analyzer**:
```bash
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs
```

**Regression Checker**:
```bash
./scripts/check-safety-regressions.sh /tmp/baseline.json
```

**Test Runner**:
```bash
./target/release/caro test --backend static --suite <test-file>.yaml
```

**Related Documentation**:
- TDD Workflow: `CONTRIBUTING.md` (Safety Pattern Development)
- Contribution Guide: `.claude/workflows/add-safety-pattern.md`
- Gap Analyzer Design: `.claude/recommendations/gap-analyzer-design.md`

## Getting Help

**If stuck**:
1. Review examples in `examples/`
2. Check CONTRIBUTING.md for detailed workflows
3. Run gap analyzer for hints
4. Ask in #safety-patterns channel

**Questions**:
- "How do I test false positives?" → See Phase 5
- "My pattern is too complex" → Consider splitting into 2 patterns
- "Tests pass but gaps remain" → Re-run gap analyzer
- "Regex syntax error" → Use https://regex101.com to test

## Metrics & Impact

**From Example Walkthroughs**:
- Average duration: 50 minutes
- Test coverage: 6.5 cases average
- Pass rate: 100%
- Regressions: 0
- False positives: 0

**Lives Saved**:
- Parent deletion: Prevents project loss
- dd disk wipe: Prevents data loss
- Combined: Protects against catastrophic user errors

---

## Skill Metadata

- **Version**: 1.0.0
- **Created**: 2026-01-08
- **Part of**: Week 1 Implementation Plan - Day 3
- **Related Skills**: safety-pattern-auditor, backend-safety-integrator
- **Agents**: N/A (manual skill)

---

*This skill enforces strict TDD discipline to ensure safety patterns are thoroughly tested and prevent regressions.*
