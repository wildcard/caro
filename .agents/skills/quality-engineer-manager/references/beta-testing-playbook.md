# Beta Testing Playbook

Procedures for coordinating beta testing of caro releases using the unbiased-beta-tester skill.

## Overview

Beta testing validates real-world usage scenarios before production release by simulating diverse user personas testing the product "in the wild."

**Key Principle**: Test with unbiased testers who have no internal knowledge of the codebase or development decisions.

---

## When to Use Beta Testing

### Required For:
- **Minor versions** (X.Y.0): New features, major improvements
- **First release** of any new functionality
- **Breaking changes** or API modifications
- **Installation flow changes**
- **Cross-platform compatibility** concerns

### Optional For:
- **Patch versions** (X.Y.Z): Bug fixes only
- **Hotfixes**: Use critical path testing instead
- **Documentation-only** releases

---

## Test Phases

### Phase 1: Artifact Verification

**Before dispatching testers**, verify release artifacts are available:

```bash
# Verify crates.io publication
cargo search caro --limit 1
curl -s https://crates.io/api/v1/crates/caro/X.Y.Z | jq -r '.version.num'

# Verify GitHub release created
gh release view vX.Y.Z --json tagName

# Verify platform binaries
gh release view vX.Y.Z --json assets -q '.assets[] | .name'

# Verify model bundles (if applicable)
gh release view vX.Y.Z --json assets -q '.assets[] | select(.name | contains("with-")) | .name'
```

**Expected Artifacts**:
- Package on crates.io
- GitHub release page
- 5 platform binaries + checksums
- 10 model bundles (optional, post-release)

---

### Phase 2: Profile Selection

Select 3-5 beta tester profiles based on release scope and risk:

#### Selection Matrix

| Release Type | Recommended Profiles | Rationale |
|--------------|---------------------|-----------|
| **Minor version** (new features) | Terminal Novice, Power User, Windows Dev | Cover beginner to expert, test cross-platform |
| **Patch version** (bug fixes) | Power User, SRE/Ops | Fast validation of fixes |
| **Major version** (breaking changes) | All 5 original + 3 new profiles | Maximum coverage |
| **Installation changes** | Terminal Novice, Corporate IT | Test constrained environments |
| **Model/ML changes** | Data Scientist, Power User | Domain-specific validation |
| **i18n/Unicode changes** | Japanese Developer, Accessibility | Specialized testing |

#### Original Profiles (bt_001 - bt_005)

| ID | Name | Use Case |
|----|------|----------|
| bt_001 | Alex (Terminal Novice) | First-time CLI user, tests onboarding |
| bt_002 | Jordan (Power User) | Expert shell user, tests advanced features |
| bt_003 | Sam (Corporate IT) | Locked-down environment, proxy, no sudo |
| bt_004 | Casey (Windows Dev) | Windows-specific testing, PowerShell |
| bt_005 | Taylor (SRE/Ops) | CI/CD integration, automation |

#### New Profiles (bt_006 - bt_010) - Added v1.0.4+

| ID | Name | Use Case |
|----|------|----------|
| bt_006 | Riley (Data Scientist) | Python/conda ecosystem, data commands |
| bt_007 | Yuki (Japanese Dev) | Unicode/i18n, non-English experience |
| bt_008 | Morgan (Fish Shell) | Non-POSIX shell compatibility |
| bt_009 | Jamie (Accessibility) | Screen reader, keyboard-only |
| bt_010 | Chris (SSH-Only) | Offline, legacy systems, user-space |

---

### Phase 3: Test Execution

#### Invoke Beta Testers

**Basic syntax**:
```
/unbiased-beta-tester profile=<profile-id> os=<os> test=<scenario>
```

**Example Invocations**:

```bash
# Test first-run experience with novice
/unbiased-beta-tester profile=terminal-novice os=macos test=first-run

# Test cargo installation with expert
/unbiased-beta-tester profile=power-user os=linux test=cargo-install

# Test Windows binary download
/unbiased-beta-tester profile=windows-dev os=windows test=binary-download

# Test offline bundle installation
/unbiased-beta-tester profile=ssh-only os=linux test=offline-bundle

# Test data processing commands
/unbiased-beta-tester profile=data-scientist os=linux test=data-commands
```

#### Test Scenarios by Profile

**Terminal Novice (bt_001)**:
- First-run experience
- Installation via curl | bash
- Basic command generation
- Error message clarity
- Documentation findability

**Power User (bt_002)**:
- Cargo installation
- Advanced command generation
- Shell completion
- JSON output mode
- CI/CD integration

**Corporate IT (bt_003)**:
- Proxy configuration
- Certificate validation
- No-sudo installation
- Firewall restrictions
- Offline model management

**Windows Dev (bt_004)**:
- Windows binary download
- PowerShell compatibility
- Path handling (backslashes)
- winget/chocolatey installation
- WSL fallback

**SRE/Ops (bt_005)**:
- Docker container usage
- Non-interactive mode
- Machine-readable output
- Log aggregation
- Health check/doctor command

**Data Scientist (bt_006)**:
- Conda environment compatibility
- Data processing commands
- CSV/JSON manipulation
- Jupyter integration
- GPU command generation

**Japanese Developer (bt_007)**:
- Japanese filename handling
- UTF-8 encoding
- Error messages with non-English
- Locale compatibility
- Input method compatibility

**Fish Shell User (bt_008)**:
- Shell detection
- Fish syntax compatibility
- Environment variable handling
- Completion generation
- `set` vs `export` handling

**Accessibility User (bt_009)**:
- Screen reader compatibility
- Keyboard-only navigation
- VoiceOver/NVDA testing
- High contrast support
- No ASCII art blocking

**SSH-Only Remote (bt_010)**:
- User-space installation
- Offline operation
- Old glibc compatibility
- Binary portability
- Airgapped model setup

---

### Phase 4: Results Collection

#### Structured Results Format

For each tester, collect:

```yaml
tester_id: bt_###
profile_name: "[Name (Role)]"
os_platform: "[OS and version]"
test_scenario: "[What was tested]"

results:
  installation:
    status: PASS | FAIL | PARTIAL
    duration_minutes: X
    friction_points:
      - "[Observed issue 1]"
      - "[Observed issue 2]"

  first_run:
    status: PASS | FAIL | PARTIAL
    model_download: SUCCESS | FAILED
    first_command: SUCCESS | FAILED
    error_handling: GOOD | POOR

  use_case_1:
    name: "[Use case name]"
    status: PASS | FAIL | PARTIAL
    notes: "[Observations]"

  use_case_2:
    name: "[Use case name]"
    status: PASS | FAIL | PARTIAL
    notes: "[Observations]"

issues_found:
  - severity: P0 | P1 | P2 | P3
    title: "[Brief title]"
    description: "[Detailed description]"
    reproduction_steps:
      - "[Step 1]"
      - "[Step 2]"
    workaround: "[Workaround if exists]" | "None"
    affected_platforms: "[Platforms]"

recommendations:
  - "[Actionable improvement 1]"
  - "[Actionable improvement 2]"

overall_verdict: PASS | CONDITIONAL_PASS | FAIL
confidence: HIGH | MEDIUM | LOW
```

#### Aggregation Across Testers

Create a test coverage matrix:

| Bundle/Scenario | bt_001 | bt_002 | bt_003 | bt_004 | bt_005 | Coverage |
|-----------------|--------|--------|--------|--------|--------|----------|
| Cargo install   | -      | ✅     | ⚠️     | -      | ✅     | 3/5      |
| Binary download | ✅     | -      | -      | ✅     | -      | 2/5      |
| First run       | ✅     | ✅     | ✅     | ✅     | ✅     | 5/5      |
| Offline use     | -      | -      | ✅     | -      | ⚠️     | 2/5      |
| Data commands   | -      | ✅     | -      | -      | -      | 1/5      |

**Legend**:
- ✅ PASS
- ⚠️ PARTIAL (worked with friction)
- ❌ FAIL
- - Not tested

---

### Phase 5: Issue Analysis

#### Categorize by Severity

**P0 (Critical)**:
- Blocks primary use case
- Affects majority of users
- No workaround exists
- **Action**: STOP RELEASE or immediate hotfix

**P1 (High)**:
- Breaks common workflow
- Affects significant user segment
- Workaround exists but difficult
- **Action**: Consider hotfix or schedule for next patch

**P2 (Medium)**:
- Degrades experience
- Affects minority of users
- Reasonable workaround available
- **Action**: Document and schedule for next minor

**P3 (Low)**:
- Minor polish issue
- Edge case or cosmetic
- **Action**: Backlog, low priority

#### Pattern Detection

Look for patterns across testers:

**Example Patterns**:
- 3/5 testers confused by error message → Documentation issue
- 2/3 Windows testers hit path issue → Platform-specific bug
- All novices gave up at step X → Onboarding friction
- Fish + Windows testers only → Shell-specific syntax

---

### Phase 6: Sign-Off Decision

Use the QE Manager sign-off matrix:

| Condition | Decision | Action |
|-----------|----------|--------|
| No P0/P1, all critical paths working | ✅ **SHIP IT** | Announce release, close milestone |
| P0 found, quick fix available (<2 hours) | 🔧 **HOTFIX** | Trigger hotfix workflow |
| P0 found, complex fix needed (>4 hours) | ❌ **ABORT** | Yank release, create fix milestone |
| Multiple P1 issues | ⚠️ **CONDITIONAL** | Assess workarounds, user impact |

#### Sign-Off Checklist

- [ ] At least 3 beta tester profiles tested
- [ ] All critical paths (install, first-run, basic command) validated
- [ ] No P0 issues discovered
- [ ] P1 issues have documented workarounds (if any)
- [ ] P2/P3 issues documented in known-issues.md
- [ ] Test results documented in QA-SIGNOFF-vX.Y.Z.md

---

## Integration with QE Manager

### Automatic Invocation

The QE Manager skill automatically invokes beta testing during Phase 2 of the release validation workflow.

**Command**:
```
/quality-engineer-manager action=beta-test version=vX.Y.Z profiles=<selection>
```

**Profile Selection Options**:
- `all` - All 10 profiles (original + new)
- `original` - Original 5 profiles only (bt_001-bt_005)
- `new` - New 5 profiles only (bt_006-bt_010)
- `minimal` - Minimal set (Novice, Power User, Windows Dev)
- `custom:bt_001,bt_003,bt_007` - Specific profiles

### Bundle Validation Workflow

For releases with model bundles, use specialized bundle validation:

```
/qa-bundle-validation version=vX.Y.Z profiles=new bundles=all
```

This dispatches testers to validate:
- Bundle structure (binary + models + licenses)
- Offline installation
- Model loading from bundle
- Cross-platform compatibility
- License compliance

---

## Best Practices

### Before Testing

1. **Load Known Issues**: Review `known-issues.md` to avoid reporting known problems
2. **Set Expectations**: Testers should simulate real users, not debug experts
3. **Document Scope**: Clearly define what scenarios to test
4. **Prepare Artifacts**: Ensure all artifacts are available before dispatching

### During Testing

1. **Observe Don't Fix**: Let testers struggle - friction points are valuable data
2. **Capture Verbatim**: Record exact errors, commands, and observations
3. **Note Timing**: Track how long tasks take (time-to-first-success)
4. **Follow Their Lead**: Don't force testers down specific paths

### After Testing

1. **Aggregate Quickly**: Analyze results within 2 hours of completion
2. **Prioritize Ruthlessly**: Don't let P3 issues delay P0 decisions
3. **Document Everything**: Add to known-issues.md immediately
4. **Share Learnings**: Update skills with new test scenarios

---

## Common Testing Mistakes

### ❌ Don't Do This:
- Test only on your own machine
- Use only expert testers
- Skip testing if "it works for me"
- Test with debug logs enabled (users don't have those)
- Use internal documentation (users see public docs only)
- Fix issues during testing (defeats the purpose)

### ✅ Do This:
- Test on fresh environments
- Use diverse persona profiles
- Test the actual release artifacts
- Simulate real user constraints
- Follow public documentation only
- Document issues for later fixing

---

## Metrics to Track

### Test Coverage Metrics
- Profiles tested: X/10
- Scenarios covered: Y total
- Platform combinations: Z
- Critical path validation: 100% (required)

### Quality Metrics
- P0 issues found: 0 (target)
- P1 issues found: ≤2 (acceptable)
- P2/P3 issues found: (informational)
- Time to first success: <10 minutes (target)

### Success Rate
- Installation success: ≥95%
- First command success: ≥90%
- Use case completion: ≥80%

---

## References

- **Beta Tester Skill**: `../../unbiased-beta-tester/SKILL.md`
- **Beta Tester Profiles**: `../../unbiased-beta-tester/examples/preset-profiles.md`
- **Known Issues Reference**: `../../unbiased-beta-tester/references/known-issues-reference.md`
- **QE Manager Skill**: `../SKILL.md`
- **Release Runbook**: `release-runbook.md`
- **Hotfix Workflow**: `hotfix-workflow.md`

---

## Quick Reference: Profile Selection

```bash
# Minimal testing (3 profiles)
profiles="terminal-novice,power-user,windows-dev"

# Standard testing (5 original profiles)
profiles="terminal-novice,power-user,corporate-it,windows-dev,sre-ops"

# Comprehensive testing (all 10 profiles)
profiles="all"

# Specialized testing (new profiles)
profiles="data-scientist,japanese-dev,fish-shell,accessibility,ssh-only"

# Custom selection
profiles="terminal-novice,japanese-dev,ssh-only"
```

---

## Emergency Skip Procedure

**ONLY use if absolutely necessary** (e.g., critical security hotfix):

```bash
# Skip beta testing (use with caution!)
export QE_SKIP_BETA_TESTING=true

# Document why in QA sign-off
echo "BETA TESTING SKIPPED: [Reason]" >> QA-SIGNOFF-vX.Y.Z.md

# Note: Still requires CI/CD validation
```

**When to Skip**:
- Security vulnerability with active exploits
- P0 hotfix with verified fix
- Rollback to previous known-good version

**Never Skip For**:
- New features
- Breaking changes
- "We're in a hurry"
- "It's just a small change"
