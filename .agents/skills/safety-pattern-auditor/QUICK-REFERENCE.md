# Safety Pattern Auditor - Quick Reference

## 1-Hour Pre-Release Audit

```bash
# Capture baseline (2 min)
./scripts/capture-safety-baseline.sh audit-$(date +%Y%m%d).json

# Run gap analyzer (5 min)
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs --min-severity critical

# Check regressions (3 min)
./scripts/check-safety-regressions.sh audit-$(date +%Y%m%d).json

# Review critical gaps (20 min)
# - Count P0 gaps (should be ≤ 3)
# - Document exceptions if >3

# Create mini-report (30 min)
# - Status: PASS/CONDITIONAL/FAIL
# - P0 gaps list
# - Action items
```

## Common Commands

```bash
# Full gap analysis
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs -o gaps.md

# Critical gaps only
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs --min-severity critical

# JSON output
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs --format json

# Specific detector
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs --detector argument

# Regression check
./scripts/check-safety-regressions.sh baseline.json

# Run tests
./target/release/caro test --backend static --suite test-cases.yaml
```

## Quick Priority Assessment

**P0 Criteria** (must fix before release):
- Severity: Critical
- Likelihood: High
- Exploit: Easy
- Coverage: None

**Count P0 gaps**:
```bash
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs --min-severity critical | grep "Critical" | wc -l
```

**Acceptable threshold**: ≤ 3 P0 gaps

## Release Decision Tree

```
Is regression check PASS?
  ├─ NO  → FAIL (do not release)
  └─ YES → Are there P0 gaps?
             ├─ 0-3 gaps  → PASS (safe to release)
             ├─ 4-10 gaps → CONDITIONAL (fix P0s first)
             └─ >10 gaps  → FAIL (major security review needed)
```

## 5-Phase Checklist

```
Phase 1: Baseline (10 min)
  ☐ Run capture script
  ☐ Review metrics
  ☐ Document starting point

Phase 2: Gap Analysis (20 min)
  ☐ Run gap analyzer
  ☐ Count gaps by type
  ☐ Extract critical gaps
  ☐ Categorize findings

Phase 3: Regression (15 min)
  ☐ Run regression checker
  ☐ Run test suite
  ☐ Check compilation
  ☐ Test false positives

Phase 4: Priority (30 min)
  ☐ Count by severity
  ☐ Assess each critical gap
  ☐ Create priority matrix
  ☐ Estimate effort

Phase 5: Report (30 min)
  ☐ Write executive summary
  ☐ Document findings
  ☐ List recommendations
  ☐ Save artifacts
  ☐ Share with team
```

## Gap Type Quick Reference

| Type | Example | Fix Complexity |
|------|---------|----------------|
| Argument Order | `rm .. -rf` | Simple |
| Path Variant | `rm -rf ../` | Simple |
| Wildcard | `rm -rf ./*` | Medium |
| Platform | `Remove-Item -Recurse` | Complex |

## Effort Estimation

- Simple gap: 30-60 minutes (most argument/path gaps)
- Medium gap: 1-2 hours (wildcards, multiple variants)
- Complex gap: 2-4 hours (platform equivalents, edge cases)

## File Locations

```
Baselines:     .claude/audits/[date]/baseline.json
Gap Reports:   .claude/audits/[date]/gaps.md
Audit Reports: .claude/audits/[date]/audit-report.md
Templates:     .claude/skills/safety-pattern-auditor/examples/
```

## Status Codes

- 🟢 **GOOD**: <5 critical gaps, no regressions, >90% test pass
- 🟡 **NEEDS ATTENTION**: 5-10 critical gaps, minor regressions
- 🔴 **CRITICAL**: >10 critical gaps, regressions found, <80% test pass

---

**Emergency Contact**: #safety-patterns channel
**Tools Help**: `./scripts/analyze-pattern-gaps.py --help`
**Full Guide**: `.claude/skills/safety-pattern-auditor/SKILL.md`
