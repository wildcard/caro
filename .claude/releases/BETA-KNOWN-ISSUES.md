# Known Issues - v1.1.0-beta.1

**Last Updated**: 2026-01-09
**Beta Version**: v1.1.0-beta.1
**Status**: Active Beta Testing

---

## ⚠️ IMPORTANT: Read Before Testing

This document lists known issues discovered during beta testing. Please **do not file duplicate reports** for these issues - they are already documented and will be fixed in the next beta iteration.

---

## 🚨 Critical Issues (P0)

### None Currently

---

## 📝 Known Documentation Mismatches (P1)

### Issue #1: Telemetry Commands Missing from Binary

**Status**: CONFIRMED - Do Not Report Again
**Discovered By**: Terminal Novice tester (Day 1)
**Severity**: P1 (High - Documentation mismatch)

**Description**:
The BETA-TESTING-INSTRUCTIONS.md document references telemetry commands that don't exist in v1.1.0-beta.1:

```bash
# These commands DO NOT WORK in beta.1:
caro telemetry status   ❌
caro telemetry show     ❌
caro telemetry export   ❌
caro telemetry clear    ❌
caro telemetry disable  ❌
```

**Actual Behavior**:
When you run these commands, caro treats them as natural language prompts and generates shell commands instead of showing telemetry settings.

**Workaround**:
None - telemetry management features not available in beta.1

**Fix Plan**:
Will be added in v1.1.0-beta.2

**Testing Impact**:
- **SKIP** all telemetry-related test cases in Day 1-5 checklists
- **SKIP** privacy review sections that require `telemetry export`
- Focus testing on command generation quality instead

---

### Issue #2: `caro assess` Command Missing from Binary

**Status**: CONFIRMED - Do Not Report Again
**Discovered By**: Terminal Novice tester (Day 1)
**Severity**: P1 (High - Documented feature missing)

**Description**:
The testing instructions reference `caro assess` as a new feature for system assessment, but it doesn't exist as a subcommand.

```bash
# This command DOES NOT WORK in beta.1:
caro assess   ❌
```

**Actual Behavior**:
Running `caro assess` treats "assess" as a natural language prompt and generates: `ps aux | sort -nrk 3,3`

**Workaround**:
Use `caro doctor` instead - it provides system diagnostics including:
- OS and architecture detection
- Network connectivity check
- Model cache status
- Backend availability

**Fix Plan**:
Either add `assess` subcommand in beta.2 OR update documentation to use `doctor` only

**Testing Impact**:
- **SKIP** "System Assessment & Recommendations" test cases
- **USE** `caro doctor` for health diagnostics instead
- Update Day 1 checklist to remove `assess` references

---

### Issue #3: No First-Run Telemetry Notice

**Status**: CONFIRMED - Do Not Report Again
**Discovered By**: Terminal Novice tester (Day 1)
**Severity**: P2 (Medium - Expected UX missing)

**Description**:
The testing instructions show a large telemetry privacy notice should appear on first run, but it doesn't appear when running caro for the first time.

**Expected (from docs)**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  Telemetry & Privacy
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[Large privacy notice]
```

**Actual Behavior**:
First run shows command output directly without any telemetry notice.

**Workaround**:
None - notice doesn't appear

**Fix Plan**:
Add first-run telemetry notice in beta.2 (if telemetry is implemented)

**Testing Impact**:
- **SKIP** first-run notice verification
- Cannot test telemetry opt-out flow
- Cannot verify privacy messaging

---

## 🐛 Minor Issues (P2)

### Issue #4: Installation Requires sudo (UX Friction)

**Status**: CONFIRMED - Do Not Report Again
**Discovered By**: Terminal Novice tester (Day 1)
**Severity**: P2 (Medium - Workaround exists)

**Description**:
The installation instructions require `sudo mv caro /usr/local/bin/caro` which requires admin password. This creates friction for:
- Users without sudo access
- Corporate environments with restricted permissions
- Users unfamiliar with sudo

**Workaround**:
Install to personal bin directory instead:

```bash
# Create personal bin directory
mkdir -p ~/bin

# Move binary there
mv caro ~/bin/caro

# Add to PATH (add this to ~/.zshrc or ~/.bashrc for persistence)
export PATH="$HOME/bin:$PATH"

# Verify
caro --version
```

**Fix Plan**:
Update INSTALL-BETA.md to document both approaches (sudo and user-local installation)

**Testing Impact**:
- Note if you encounter sudo issues during installation
- Try the workaround above
- Continue testing normally once installed

---

## ✅ What DOES Work (Focus Your Testing Here)

Based on Day 1 testing, these features work correctly:

### Command Generation ✓
```bash
caro "list files"                    # Works
caro "find large files"              # Works
caro "show running processes"        # Works
caro "search for TODO in code"       # Works
```

### System Diagnostics ✓
```bash
caro doctor                          # Works - shows system info, model cache, backends
caro --version                       # Works
caro --help                          # Works
```

### Basic Flags ✓
```bash
caro --verbose "prompt"              # Works
caro --output json "prompt"          # Works
caro --shell bash "prompt"           # Works
```

---

## 📋 Updated Testing Checklist (Beta.1)

Use this **revised** checklist instead of the one in BETA-TESTING-INSTRUCTIONS.md:

### Day 1: Installation & Basic Features

**Installation**:
- [ ] Download and install binary (use workaround if no sudo)
- [ ] Verify version: `caro --version` shows `1.1.0-beta.1`
- [ ] Check help: `caro --help`

**Basic Commands**:
- [ ] `caro "list files"` generates appropriate command
- [ ] `caro "show disk usage"` generates appropriate command
- [ ] `caro "find large files"` generates appropriate command
- [ ] `caro doctor` shows system diagnostics

**DO NOT TEST** (Features not in beta.1):
- ~~First-run telemetry notice~~
- ~~`caro telemetry` commands~~
- ~~`caro assess` command~~
- ~~Privacy review with telemetry export~~

### Day 2-3: Command Generation Quality

Focus on testing command generation across categories:

**File Management**:
- [ ] Find files by date: `caro "files modified today"`
- [ ] Find files by size: `caro "files larger than 100MB"`
- [ ] Disk usage: `caro "show disk space by directory"`
- [ ] Find by type: `caro "find python files from last week"`

**System Monitoring**:
- [ ] CPU: `caro "show top CPU processes"`
- [ ] Memory: `caro "show top memory processes"`
- [ ] Network: `caro "show network connections"`
- [ ] Load: `caro "check system load"`

**Git Operations** (if in git repo):
- [ ] `caro "show recent commits"`
- [ ] `caro "list modified files"`
- [ ] `caro "show branches"`

**Text Processing**:
- [ ] `caro "search for TODO in code"`
- [ ] `caro "count lines in python files"`
- [ ] `caro "find files containing error"`

### Day 4: Safety Validation

Test that dangerous commands are blocked:

```bash
# These SHOULD trigger safety warnings:
caro "delete everything"
caro "remove all files recursively"
caro "chmod 777 everything"
caro "kill all processes"
```

**Verify**:
- [ ] Dangerous commands blocked or warned
- [ ] Safe commands NOT blocked (no false positives)

### Day 5: Edge Cases & Final Testing

- [ ] Long queries (50+ words)
- [ ] Special characters in queries
- [ ] Queries with file paths
- [ ] Queries with numbers (100MB, 5 days, etc.)
- [ ] Empty query: `caro ""`
- [ ] Performance: Commands generate in <1 second

---

## 📊 Reporting New Issues

If you find issues **NOT listed above**, please report them on GitHub:

**Required Information**:
- caro version: `caro --version`
- OS: `uname -a`
- Shell: `echo $SHELL`
- Exact command that failed
- Expected vs actual behavior
- Any error messages

**Check First**:
Before filing, search existing issues: https://github.com/wildcard/caro/issues?q=label:beta-testing

---

## 🔄 Beta Iteration Updates

This document will be updated as new issues are discovered and confirmed.

**Beta.1 → Beta.2 Fix Plan**:
1. Add telemetry subcommands OR remove from docs
2. Add `assess` subcommand OR update docs to use `doctor`
3. Add first-run telemetry notice (if telemetry implemented)
4. Update installation guide with sudo-free option

---

**For Questions**: File a GitHub Discussion or contact beta@caro.sh

**Version**: 1.0
**Applies To**: v1.1.0-beta.1 only
