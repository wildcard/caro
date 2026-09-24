# Safety Pattern Auditor Skill

**Systematic auditing of safety patterns to maintain high quality and security**

## What This Skill Does

Guides you through a comprehensive 5-phase audit process:

1. **Baseline Capture**: Document current state
2. **Gap Analysis**: Find missing coverage with automated tools
3. **Regression Testing**: Ensure no quality degradation
4. **Priority Assessment**: Triage and plan fixes
5. **Report & Recommendations**: Create actionable output

## When to Use

- **Pre-Release** (Mandatory): QA gate before shipping
- **Monthly** (Recommended): Regular security review
- **Post-Incident** (As-Needed): After vulnerability reports

## Duration

- Pre-Release Audit: 1 hour
- Monthly Full Audit: 2-3 hours
- Post-Incident Audit: 1-2 hours

## Quick Start

```bash
# Invoke the skill
/safety-pattern-auditor

# Or follow manual steps:
./scripts/capture-safety-baseline.sh baseline.json
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs -o gaps.md
./scripts/check-safety-regressions.sh baseline.json
```

## What You'll Create

- ✅ Baseline metrics (JSON)
- ✅ Gap analysis report (Markdown)
- ✅ Regression test results
- ✅ Priority matrix (P0/P1/P2/P3)
- ✅ Audit report with recommendations
- ✅ Historical trend analysis

## Success Criteria

Audit is complete when:
- All 5 phases executed
- No regressions found
- P0 gaps ≤ 3 (or documented)
- Report written with action items
- Artifacts saved for tracking

## Audit Checklist

```
☐ Phase 1: Baseline captured
☐ Phase 2: Gap analyzer run
☐ Phase 3: Regression tests passed
☐ Phase 4: Gaps prioritized
☐ Phase 5: Report written

☐ No regressions detected
☐ No false positives
☐ P0 gaps identified
☐ Action plan with owners
☐ Artifacts saved
```

## Tools Used

- **Baseline Capture**: `./scripts/capture-safety-baseline.sh`
- **Gap Analyzer**: `./scripts/analyze-pattern-gaps.py`
- **Regression Check**: `./scripts/check-safety-regressions.sh`
- **Test Runner**: `./target/release/caro test`

## Priority Framework

- **P0**: Fix immediately (Critical + Likely + Easy to exploit)
- **P1**: Fix this release (Critical or High + Likely)
- **P2**: Fix next release (Medium risk or Complex)
- **P3**: Backlog (Low priority, document only)

## Example Audit Report

See `examples/audit-report-template.md` for complete template.

## Integration

**Works with**:
- Gap Analyzer tool (automated gap detection)
- Regression checker (baseline comparison)
- Pattern developer skill (fix workflows)

**Provides**:
- Historical tracking
- Priority assessment
- Trend analysis
- Action planning

---

*Maintain high safety pattern quality through systematic auditing.*
