# Caro Beta Testing Test Cases

This directory contains all natural language query examples extracted from the caro website, organized for systematic beta testing cycles.

## Overview

**Total Test Cases**: 75 curated examples
**Source**: Website (website/src/**/*.astro, *.tsx)
**Version**: 1.0.0
**Last Updated**: 2025-01-07

## Quick Start

1. **Review test-cases.yaml** - All website examples with expected outputs
2. **Select a profile** - Choose appropriate tester persona (see profiles below)
3. **Run tests manually** - Execute each test case and record results
4. **Store results** - Save outcomes in `cycles/YYYY-MM-DD-profile-name.md`

## File Structure

```
.claude/beta-testing/
├── README.md              # This file
├── test-cases.yaml        # All test cases with profile assignments
└── cycles/                # Test cycle results (one file per run)
    └── 2025-01-07-bt_001.md
```

## Test Categories

| Category | Count | Description | Primary Profile |
|----------|-------|-------------|----------------|
| **file_management** | 15 | File operations (find, ls, du) | bt_001 (Novice) |
| **system_monitoring** | 7 | Process/system monitoring (ps, top, lsof) | bt_002 (Power User) |
| **git_version_control** | 3 | Git and version control | bt_002 (Power User) |
| **log_analysis** | 4 | Log parsing (grep, journalctl, awk) | bt_005 (SRE) |
| **network_operations** | 5 | Network tools (ping, ss, wget) | bt_005 (SRE) |
| **devops_kubernetes** | 5 | DevOps tools (kubectl, docker, terraform) | bt_005 (SRE) |
| **text_processing** | 6 | Text manipulation (sed, awk, grep) | bt_002 (Power User) |
| **dangerous_commands** | 8 | Safety testing for risky operations | bt_005 (SRE) |

## Beta Tester Profiles

### bt_001 - Alex (Terminal Novice)
- **Skill**: Novice - copies commands from tutorials
- **Platform**: macOS 14.3, zsh
- **Focus**: Basic installation, first-time user experience, documentation clarity
- **Patience**: Low (5 min, gives up after 1 failure)
- **Best For**: Simple file operations, onboarding flows, beginner-friendly examples
- **Test Categories**: file_management basics

### bt_002 - Jordan (Power User)
- **Skill**: Expert - shell scripts, advanced tooling
- **Platform**: Linux Ubuntu 22.04, zsh
- **Focus**: Advanced features, CI/CD integration, complex commands, scripting
- **Patience**: High (30 min, 5 attempts)
- **Best For**: System monitoring, git commands, text processing, advanced use cases
- **Test Categories**: system_monitoring, git_version_control, text_processing

### bt_003 - Sam (Corporate IT)
- **Skill**: Advanced
- **Platform**: Windows 11 Enterprise, PowerShell
- **Focus**: Security assessment, offline installation, corporate proxy, restricted environment
- **Patience**: High (60 min)
- **Best For**: Enterprise scenarios, security-focused use cases, air-gapped testing
- **Test Categories**: dangerous_commands (safety validation)

### bt_004 - Casey (Windows Developer)
- **Skill**: Intermediate
- **Platform**: Windows 11 Pro, PowerShell
- **Focus**: Windows-native installation, PowerShell command generation
- **Patience**: Medium (20 min)
- **Best For**: Windows-specific workflows, PowerShell syntax, cross-platform testing
- **Test Categories**: file_management, text_processing

### bt_005 - Taylor (SRE/Ops Engineer)
- **Skill**: Expert
- **Platform**: Linux Ubuntu 22.04, bash
- **Focus**: CI/CD pipeline integration, automation, machine-readable output, idempotent operations
- **Patience**: Very High (60 min, 10 attempts)
- **Best For**: DevOps workflows, Kubernetes, dangerous command safety, log analysis
- **Test Categories**: devops_kubernetes, log_analysis, network_operations, dangerous_commands

### bt_006 - Riley (Data Scientist)
- **Skill**: Intermediate (Python expert, shell beginner)
- **Platform**: Linux Ubuntu 22.04, bash
- **Focus**: Data processing, CSV/JSON manipulation, batch file operations
- **Patience**: Medium (15 min)
- **Best For**: Data transformation, file manipulation, Python-related workflows
- **Test Categories**: text_processing, file_management (with Python focus)

### bt_007 - Yuki (Japanese Developer)
- **Skill**: Advanced
- **Platform**: macOS 14.3, zsh
- **Focus**: Japanese i18n, UTF-8 encoding, Unicode filename handling
- **Patience**: Medium (20 min)
- **Best For**: International/Unicode scenarios, non-ASCII filenames
- **Test Categories**: i18n-specific tests

### bt_008 - Morgan (Fish Shell User)
- **Skill**: Expert
- **Platform**: macOS 14.3, fish
- **Focus**: Fish shell compatibility, non-POSIX syntax, environment variables
- **Patience**: High (30 min)
- **Best For**: Non-POSIX shell scenarios, Fish-specific syntax
- **Test Categories**: All categories (Fish compatibility check)

### bt_009 - Jamie (Accessibility User)
- **Skill**: Advanced
- **Platform**: macOS 14.3, zsh (VoiceOver screen reader)
- **Focus**: Screen reader compatibility, keyboard-only navigation, accessible output
- **Patience**: High (30 min)
- **Best For**: Accessible CLI design, output without ASCII art
- **Test Categories**: All categories (accessibility check)

### bt_010 - Chris (SSH-Only Remote)
- **Skill**: Expert
- **Platform**: Linux CentOS 7 (EOL), bash, SSH-only, airgapped
- **Focus**: Offline operation, legacy systems, old glibc compatibility
- **Patience**: Very High (60 min)
- **Best For**: Legacy system support, offline usage, restricted environments
- **Test Categories**: All categories (legacy compatibility check)

## How to Run a Beta Testing Cycle

### Step 1: Choose Profile

Select the appropriate tester profile based on what you want to test:
- **bt_001**: First-time user experience, basic examples
- **bt_002**: Advanced features, power user workflows
- **bt_005**: DevOps/SRE scenarios, safety features
- **bt_003/bt_004**: Windows compatibility
- **bt_007**: i18n/Unicode handling
- **bt_008**: Fish shell compatibility
- **bt_009**: Accessibility
- **bt_010**: Legacy system support

### Step 2: Filter Test Cases

From `test-cases.yaml`, extract tests where the profile appears as primary or secondary:

```bash
# Example: Get all tests for bt_001
grep -A 10 "primary_profile: \"bt_001\"" test-cases.yaml

# Or use yq if available:
yq '.test_cases[] | select(.primary_profile == "bt_001" or .secondary_profiles[] == "bt_001")' test-cases.yaml
```

### Step 3: Execute Tests

For each test case:

1. **Run caro with the input**:
   ```bash
   caro "list all files modified today"
   ```

2. **Compare output** to `expected_output` in YAML

3. **Record result**:
   - ✅ PASS: Output matches exactly
   - ⚠️ PARTIAL: Output functionally equivalent but different
   - ❌ FAIL: Output incorrect or error occurred
   - 🚫 BLOCKED: For dangerous commands that should be blocked

4. **Capture evidence**:
   - Exact command run
   - Full output (stdout + stderr)
   - caro version
   - System info (OS, shell)

### Step 4: Record Results

Create a result file in `cycles/`:

```markdown
# Beta Test Results: bt_001 (Alex - Terminal Novice)

**Date**: 2025-01-07
**Profile**: bt_001 - Alex (Terminal Novice)
**Environment**:
- OS: macOS 14.3
- Shell: zsh
- caro version: X.Y.Z
- Backend: embedded/mlx

## Test Results

### fm_001 - list all files modified today
- **Input**: `caro "list all files modified today"`
- **Expected**: `find . -type f -mtime 0`
- **Actual**: `find . -type f -mtime 0`
- **Result**: ✅ PASS
- **Notes**: Perfect match, instant response

### fm_002 - find large files over 100MB
- **Input**: `caro "find large files over 100MB"`
- **Expected**: `find . -type f -size +100M`
- **Actual**: `find . -type f -size +100M`
- **Result**: ✅ PASS

...

## Summary

- Total Tests: 15
- Passed: 14
- Partial: 1
- Failed: 0
- Blocked: 0 (N/A for this profile)

## Issues Found

1. [Issue description if any]
2. ...

## Observations

- Installation was straightforward
- Safety warnings were clear
- ...
```

Save as: `cycles/2025-01-07-bt_001-alex.md`

## Priority Testing Matrix

### P0 - Must Test Every Release (Website Core Examples)

These **MUST** generate exact matches (100% pass rate):

| Test ID | Input | Category |
|---------|-------|----------|
| fm_001 | "list all files modified today" | file_management |
| fm_002 | "find large files over 100MB" | file_management |
| sm_001 | "show disk usage by folder" | system_monitoring |
| fm_003 | "find python files modified last week" | file_management |

**Test With**: bt_001 (Novice) - these are homepage examples

### P1 - High Priority (Common Use Cases)

Test these with multiple profiles to ensure broad compatibility:

- All **file_management** tests → bt_001, bt_002
- All **system_monitoring** tests → bt_002, bt_005
- All **dangerous_commands** tests → bt_005, bt_003

### P2 - Medium Priority (Platform-Specific)

- **devops_kubernetes** → bt_005
- **git_version_control** → bt_002, bt_006
- **log_analysis** → bt_005
- **network_operations** → bt_005

### P3 - Lower Priority (Edge Cases)

- Unicode/i18n → bt_007
- Fish shell → bt_008
- Accessibility → bt_009
- Legacy systems → bt_010

## Automation Integration

While manual testing is valuable, these test cases can also be used for automation:

### With `caro test` CLI (If Implemented)

```bash
# Run all tests for a profile
caro test --profile bt_001

# Run tests by category
caro test --category file_management

# Run P0 tests only
caro test --priority p0
```

### With Existing Eval Framework

The test cases in this YAML can be converted to the format used by `src/eval/mod.rs`:

```rust
let suite = EvalSuite::from_yaml(".claude/beta-testing/test-cases.yaml")?;
let results = suite.run_tests(&backend).await?;
results.print_summary();
```

## Tips for Beta Testers

### For Novice Testers (bt_001)
- Start with P0 tests (homepage examples)
- Don't skip failures - document confusion
- Note where documentation is unclear
- Stop after 1-2 failures (per profile patience level)

### For Expert Testers (bt_002, bt_005)
- Try variations of inputs (synonyms, rephrasing)
- Test edge cases
- Check JSON output modes
- Test pipeline integration
- Verify error messages are actionable

### For Platform-Specific Testers (bt_003, bt_004, bt_010)
- Focus on platform differences (GNU vs BSD, Windows paths)
- Test offline scenarios
- Check compatibility with restricted environments
- Document any platform-specific issues

### For Specialized Testers (bt_007, bt_008, bt_009)
- Focus on your specialization (i18n, Fish, accessibility)
- Test boundary cases in your domain
- Provide feedback on specialized workflows

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-01-07 | Initial compilation of 75 test cases from website |

## Future Enhancements

- [ ] Add JSON schema validation for test case YAML
- [ ] Create automated test runner script
- [ ] Integrate with GitHub Actions for CI testing
- [ ] Add platform-specific test variants (GNU/BSD/Windows)
- [ ] Create visual diff tool for comparing outputs
- [ ] Add regression tracking (compare cycles over time)

## Questions?

See also:
- `.claude/skills/unbiased-beta-tester/` - Beta tester skill documentation
- `src/eval/mod.rs` - Evaluation framework source code
- `ROADMAP.md` - Beta testing cycles and Issue #395 tracking
