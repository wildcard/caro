---
name: qa-bundle-validation
description: Orchestrate systematic beta testing of model bundles with multiple profiles, collect structured results, and generate QA sign-off report
---

# QA Bundle Validation Command

This command orchestrates comprehensive beta testing of model bundles using multiple beta tester profiles, following the Beta Testing Playbook methodology.

## Purpose

Systematically validate model bundles across diverse user personas to ensure:
- Bundle structure integrity (binary + models + licenses)
- Cross-platform compatibility
- Offline installation success
- Model loading from bundle works correctly
- User experience quality across different environments

## Usage

```bash
/qa-bundle-validation version=vX.Y.Z profiles=<selection> bundles=<scope>
```

### Parameters

| Parameter | Required | Options | Description |
|-----------|----------|---------|-------------|
| `version` | Yes | `vX.Y.Z` | GitHub release version tag |
| `profiles` | No (default: `minimal`) | `all`, `original`, `new`, `minimal`, `custom:bt_001,bt_003` | Beta tester profile selection |
| `bundles` | No (default: `sample`) | `all`, `qwen`, `smollm`, `sample` | Which bundles to test |

### Profile Selection Options

- `all`: All 10 profiles (bt_001 through bt_010)
- `original`: Original 5 profiles (bt_001-bt_005)
- `new`: New 5 profiles (bt_006-bt_010)
- `minimal`: Minimal set (Terminal Novice, Power User, Windows Dev) - bt_001, bt_002, bt_004
- `custom:bt_001,bt_003,bt_007`: Specific profiles by ID

### Bundle Scope Options

- `all`: Test all 10 bundles (5 platforms × 2 models)
- `qwen`: Test only Qwen 1.5B bundles (5 platforms)
- `smollm`: Test only SmolLM 135M bundles (5 platforms)
- `sample`: Test 2 representative bundles (Linux amd64 + macOS Silicon with Qwen)

## Workflow

### Phase 1: Pre-Flight Checks

Before dispatching any testers, verify prerequisites:

1. **Verify Release Exists**
   ```bash
   gh release view $VERSION --json tagName,createdAt
   ```
   - Confirm release tag exists on GitHub
   - Note release date for timeline tracking

2. **Verify Bundle Assets**
   ```bash
   gh release view $VERSION --json assets -q '.assets[] | select(.name | contains("with-")) | .name'
   ```
   - Expected: 10 bundles (or subset based on `bundles` parameter)
   - Check bundle naming: `caro-VERSION-PLATFORM-with-MODEL.tar.gz`
   - Verify checksums exist: `.sha256` files for each bundle

3. **Check Bundle Sizes**
   ```bash
   gh release view $VERSION --json assets -q '.assets[] | select(.name | contains("with-")) | "\(.name): \(.size / 1024 / 1024 | floor)MB"'
   ```
   - Qwen bundles: ~1.0-1.1 GB each
   - SmolLM bundles: ~130-150 MB each

4. **Load Known Issues**
   - Read `.claude/skills/quality-engineer-manager/references/known-issues.md`
   - Note any known bundle-specific issues
   - Prepare to skip testing known failures

**Output Phase 1 Summary:**
```markdown
## Pre-Flight Checks: $VERSION

✅ Release exists: $VERSION (created: $DATE)
✅ Bundle assets verified: X/10 bundles present
✅ Bundle sizes correct (Qwen: ~1.1GB, SmolLM: ~145MB)
ℹ️  Known issues: X documented (none blocking bundle testing)

**Ready to dispatch beta testers.**
```

### Phase 2: Profile Selection & Assignment

Based on the `profiles` parameter, select and assign bundle-profile combinations:

**Minimal Testing** (`profiles=minimal`):
- bt_001 (Terminal Novice): macOS Silicon with Qwen
- bt_002 (Power User): Linux amd64 with Qwen
- bt_004 (Windows Dev): Windows amd64 with SmolLM

**Standard Testing** (`profiles=original`):
- All original 5 profiles, each assigned a relevant platform/model combo

**Comprehensive Testing** (`profiles=all`):
- All 10 profiles, covering all bundle variations

**Assignment Strategy:**
1. Match profile OS to bundle platform
2. Vary model sizes across profiles (test both Qwen and SmolLM)
3. For custom selections, distribute bundles to maximize coverage

**Output Phase 2 Summary:**
```markdown
## Beta Tester Assignment

Testing Configuration:
- Profiles selected: X
- Bundles to test: Y
- Total test sessions: Z

| Profile | OS | Bundle | Model | Size |
|---------|----|----|-------|------|
| bt_001 (Terminal Novice) | macOS | macos-silicon-with-qwen-1.5b | Qwen 1.5B | 1.04 GB |
| bt_002 (Power User) | Linux | linux-amd64-with-qwen-1.5b | Qwen 1.5B | 1.04 GB |
| ... | ... | ... | ... | ... |

**Dispatching testers...**
```

### Phase 3: Test Execution

For each assigned profile-bundle combination:

1. **Invoke Beta Tester Skill**
   ```
   /unbiased-beta-tester profile=<profile-id> os=<os> docker=<true/false> test=bundle-installation
   ```

2. **Test Scenarios** (each tester should execute):

   a. **Bundle Download**
      - Download bundle from GitHub release
      - Verify checksum matches `.sha256` file
      - Check bundle size matches expected

   b. **Bundle Extraction**
      - Extract tar.gz to test directory
      - Verify structure:
        ```
        caro-VERSION-PLATFORM-with-MODEL/
        ├── caro (or caro.exe)
        ├── models/
        │   └── <model-file>.gguf
        └── licenses/
            ├── LICENSE-Qwen2.5-0.5B-Instruct (or SmolLM-135M-Instruct)
            └── THIRD_PARTY_NOTICES.txt
        ```

   c. **Offline Installation**
      - Move binary to `~/bin` or `$HOME/.local/bin`
      - Update PATH
      - Verify `caro --version` works
      - **No network access after this step**

   d. **Model Loading Test**
      - Run `caro "test command"` without network
      - Verify model loads from local bundle
      - Check command generation works
      - Ensure no network calls are made

   e. **License Validation**
      - Verify LICENSE files are present
      - Check THIRD_PARTY_NOTICES.txt content
      - Confirm attribution is complete

3. **Evidence Collection** (per tester):
   - Exact commands run
   - Full stdout/stderr output
   - Tool versions (`caro --version`, `uname -a`)
   - Bundle extraction logs
   - Model loading output
   - Any errors encountered

4. **Structured Results Format** (per tester):
   ```yaml
   tester_id: bt_###
   profile_name: "[Name (Role)]"
   bundle_tested: "caro-VERSION-PLATFORM-with-MODEL.tar.gz"
   test_date: "YYYY-MM-DD"

   results:
     bundle_download:
       status: PASS | FAIL
       duration_seconds: X
       notes: "[Observations]"

     bundle_extraction:
       status: PASS | FAIL
       structure_valid: true | false
       files_present:
         - binary: true | false
         - models: true | false
         - licenses: true | false

     offline_installation:
       status: PASS | FAIL
       install_location: "$HOME/bin"
       path_updated: true | false

     model_loading:
       status: PASS | FAIL
       model_loaded: true | false
       network_calls: 0 (expected)
       first_command: SUCCESS | FAILED

     license_compliance:
       status: PASS | FAIL
       apache_license: true | false
       third_party_notices: true | false
       attribution_complete: true | false

   issues_found:
     - severity: P0 | P1 | P2 | P3
       title: "[Brief title]"
       description: "[Detailed description]"
       reproduction_steps: []
       workaround: "[Workaround if exists]" | "None"

   overall_verdict: PASS | CONDITIONAL_PASS | FAIL
   confidence: HIGH | MEDIUM | LOW
   ```

**Output Phase 3 Progress:**
```markdown
## Test Execution Progress

- [ ] bt_001 (Terminal Novice) → macOS Silicon Qwen bundle
- [x] bt_002 (Power User) → Linux amd64 Qwen bundle (PASS)
- [ ] bt_004 (Windows Dev) → Windows amd64 SmolLM bundle
...

**Running tests in parallel...**
```

### Phase 4: Results Aggregation

After all testers complete:

1. **Collect All Results**
   - Parse structured results from each tester
   - Create test coverage matrix

2. **Coverage Matrix**
   ```markdown
   | Bundle | bt_001 | bt_002 | bt_004 | ... | Coverage |
   |--------|--------|--------|--------|-----|----------|
   | linux-amd64-qwen | - | ✅ | - | - | 1/X |
   | macos-silicon-qwen | ✅ | - | - | - | 1/X |
   | windows-amd64-smollm | - | - | ⚠️ | - | 1/X |
   ...
   ```

   **Legend**:
   - ✅ PASS
   - ⚠️ PARTIAL (worked with friction)
   - ❌ FAIL
   - - Not tested

3. **Issue Aggregation**
   - Group issues by severity (P0, P1, P2, P3)
   - Identify patterns across testers
   - Link related issues

4. **Pattern Detection**
   - Look for issues affecting multiple platforms
   - Note tester-specific vs universal problems
   - Identify documentation gaps

**Output Phase 4 Summary:**
```markdown
## Results Summary

### Test Coverage
- Profiles tested: X/10
- Bundles tested: Y/10
- Test scenarios: Z total
- Overall pass rate: N%

### Issue Breakdown by Severity
- **P0 (Critical)**: 0 issues
- **P1 (High)**: 1 issue
- **P2 (Medium)**: 3 issues
- **P3 (Low)**: 2 issues

### Common Patterns Detected
1. [Pattern description]
2. [Pattern description]
...
```

### Phase 5: Issue Analysis & Categorization

For each issue found:

1. **Categorize by Severity**

   **P0 (Critical)**:
   - Bundle doesn't extract properly
   - Binary doesn't run on target platform
   - Model files missing or corrupted
   - Crashes on model loading
   - **Action**: STOP release, hotfix required

   **P1 (High)**:
   - Model loading slow but eventually works
   - Binary requires specific glibc version not documented
   - License files incomplete
   - **Action**: Document workaround or hotfix

   **P2 (Medium)**:
   - Bundle extraction shows warnings
   - Model path handling quirks
   - Documentation unclear
   - **Action**: Document and schedule for next release

   **P3 (Low)**:
   - Minor cosmetic issues
   - Performance optimizations possible
   - **Action**: Backlog

2. **Root Cause Analysis**
   - Determine if issue is bundle-specific or systemic
   - Check if related to known issues
   - Identify reproduction steps

3. **Impact Assessment**
   - How many platforms affected?
   - How many users affected?
   - Is there a workaround?

**Output Phase 5:**
```markdown
## Issue Analysis

### P0 Issues (Critical)
None

### P1 Issues (High)
**Issue #1**: Windows SmolLM bundle extraction warning
- **Affects**: Windows users only
- **Root Cause**: Path separator handling
- **Workaround**: Extract with 7-Zip instead of Windows built-in
- **Recommendation**: Document in release notes

### P2 Issues (Medium)
...
```

### Phase 6: Sign-Off Decision

Use the QE Manager sign-off matrix:

| Condition | Decision | Action |
|-----------|----------|--------|
| No P0/P1, all critical paths working | ✅ **SHIP IT** | Announce bundles ready |
| P0 found, quick fix available | 🔧 **HOTFIX** | Rebuild bundles |
| P0 found, complex fix needed | ❌ **ABORT** | Defer bundling |
| Multiple P1 issues | ⚠️ **CONDITIONAL** | Assess workarounds |

**Sign-Off Checklist:**
- [ ] All selected profiles tested
- [ ] No P0 issues discovered
- [ ] P1 issues have documented workarounds
- [ ] P2/P3 issues documented
- [ ] Test coverage acceptable (>70% of selected bundles)
- [ ] Results documented in QA-SIGNOFF report

**Output Phase 6:**
```markdown
## QA Sign-Off Decision

**Decision**: ✅ **APPROVED**

**Rationale**:
- All X profile tests completed
- No P0/P1 blocking issues
- P2 issues have reasonable workarounds
- Test coverage: Y% of bundles validated

**Recommendation**: Announce model bundles as ready for download.

**Next Steps**:
1. Update release notes with bundle availability
2. Document P1 workarounds in known-issues.md
3. File GitHub issues for P2/P3 improvements
```

### Phase 7: Documentation Updates

1. **Update Known Issues Database**
   - Add any new bundle-specific issues
   - Reference bundle version and platforms affected

2. **Update Release Notes**
   - Add bundle availability notice
   - Note any platform-specific workarounds
   - Link to bundle download instructions

3. **Create QA Sign-Off Artifact**
   - Save complete report to `.claude/QA-SIGNOFF-BUNDLES-vX.Y.Z.md`
   - Include all test results, coverage matrix, and decisions

4. **Update Beta Tester Profiles** (if needed)
   - Add new test scenarios discovered
   - Update profile configurations based on learnings

**Output Phase 7:**
```markdown
## Documentation Complete

- [x] Known issues updated (X new entries)
- [x] QA sign-off report saved: QA-SIGNOFF-BUNDLES-vX.Y.Z.md
- [x] Release notes updated with bundle availability
- [ ] GitHub issues filed for P2/P3 items (optional)

**Bundle validation complete for $VERSION.**
```

## Examples

### Example 1: Minimal Validation (3 Profiles)

```bash
/qa-bundle-validation version=v1.0.4 profiles=minimal bundles=sample
```

**Output:**
```
# QA Bundle Validation: v1.0.4 (Minimal)

## Phase 1: Pre-Flight Checks
✅ Release v1.0.4 exists (created: 2026-01-07)
✅ Bundle assets: 2/2 sample bundles verified
✅ Sizes correct (Qwen: 1.04GB)

## Phase 2: Assignment
- bt_001 (Terminal Novice) → macOS Silicon Qwen
- bt_002 (Power User) → Linux amd64 Qwen

## Phase 3: Test Execution
[Invokes 2 beta tester sessions]

## Phase 4-6: Results & Sign-Off
Coverage: 2/10 bundles (20%)
Issues: 0 P0, 0 P1, 1 P2

**Decision**: ✅ APPROVED (sample validation)

Recommendation: Run comprehensive testing before major announcement.
```

### Example 2: Comprehensive Validation (All Profiles)

```bash
/qa-bundle-validation version=v1.0.4 profiles=all bundles=all
```

**Output:**
```
# QA Bundle Validation: v1.0.4 (Comprehensive)

## Phase 1: Pre-Flight Checks
✅ All 10 bundles verified

## Phase 2: Assignment
All 10 profiles assigned across 10 bundles

## Phase 3: Test Execution
[Invokes 10 beta tester sessions in parallel]

## Phase 4-6: Results & Sign-Off
Coverage: 10/10 bundles (100%)
Issues: 0 P0, 1 P1, 3 P2, 2 P3

**Decision**: ⚠️ **CONDITIONAL PASS**

P1 Issue requires documentation before announcement.
```

### Example 3: New Profiles Only (Post-Release)

```bash
/qa-bundle-validation version=v1.0.4 profiles=new bundles=all
```

Tests with bt_006-bt_010 to validate:
- Data scientist workflows (bt_006)
- Japanese/Unicode handling (bt_007)
- Fish shell compatibility (bt_008)
- Accessibility (bt_009)
- Offline/legacy systems (bt_010)

## Integration with QE Manager

The QE Manager skill automatically invokes this command during Phase 2 (Beta Testing) for bundle releases:

```
# In QE Manager workflow
/quality-engineer-manager action=beta-test version=vX.Y.Z

# QE Manager internally calls:
/qa-bundle-validation version=vX.Y.Z profiles=original bundles=sample

# Then escalates to comprehensive if issues found:
/qa-bundle-validation version=vX.Y.Z profiles=all bundles=all
```

## Best Practices

1. **Start with Minimal**
   - Run minimal validation first for fast feedback
   - Escalate to comprehensive only if issues found

2. **Test Sample Before All**
   - `bundles=sample` catches most issues
   - Save time by testing representative bundles first

3. **Match Profiles to Risk**
   - Installation changes → Test bt_001, bt_003, bt_010
   - Data/ML changes → Test bt_006
   - i18n changes → Test bt_007

4. **Document Everything**
   - Every friction point is a learning opportunity
   - Update known-issues.md immediately
   - Share insights with team

5. **Iterate on Failure**
   - Don't batch multiple rounds of testing
   - Fix P0/P1 issues immediately
   - Re-test only affected bundles

## Troubleshooting

### Issue: "Release not found"
- Verify version format: `vX.Y.Z` (with leading 'v')
- Check GitHub release exists: `gh release view vX.Y.Z`

### Issue: "No bundles found"
- Bundles are created post-release
- Wait for bundle workflow to complete
- Check workflow: `gh run list --workflow=bundle.yml`

### Issue: "Tester invocation failed"
- Verify profile ID is valid (bt_001-bt_010)
- Check unbiased-beta-tester skill is installed
- Ensure OS matches bundle platform

### Issue: "Test coverage too low"
- Increase profile selection (minimal → original → all)
- Add more bundle testing (sample → all)
- Consider custom profile selection for specific issues

## References

- **Beta Testing Playbook**: `.claude/skills/quality-engineer-manager/references/beta-testing-playbook.md`
- **Beta Tester Profiles**: `.claude/skills/unbiased-beta-tester/examples/preset-profiles.md`
- **Known Issues**: `.claude/skills/quality-engineer-manager/references/known-issues.md`
- **QE Manager Skill**: `.claude/skills/quality-engineer-manager/SKILL.md`
