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

### Issue #3: Telemetry Notice Behavior Inconsistent

**Status**: PARTIALLY CONFIRMED - Mixed Results
**Discovered By**: Terminal Novice (Day 1), Power User (Day 2)
**Severity**: P2 (Medium - Inconsistent UX)

**Description**:
Telemetry notice behavior is inconsistent between test sessions:
- **Day 1 (Terminal Novice)**: No notice appeared on first run
- **Day 2 (Power User)**: Notice appeared on EVERY command

**Expected**:
Notice should appear once per session (or first run only)

**Actual Behavior - Scenario A (Day 1)**:
First run shows command output directly without any telemetry notice.

**Actual Behavior - Scenario B (Day 2)**:
Notice appears on EVERY single command invocation:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  Telemetry & Privacy
[Full notice repeated every time]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Command: [generated command]
```

**Impact**:
- Notice repetition is extremely annoying for power users
- Clutters output, breaks workflows
- Makes tool unusable for frequent use

**Workaround**:
None - behavior cannot be controlled

**Fix Plan**:
- Implement proper first-run detection
- Show notice ONCE per session maximum
- Add flag to suppress notice after acknowledgment

**Testing Impact**:
- Note whether notice appears for you
- Count how many times it repeats
- Test if it persists across shell sessions

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

### Issue #5: Telemetry Notice Pollutes JSON Output

**Status**: CONFIRMED - Do Not Report Again
**Discovered By**: SRE/DevOps tester (Day 4)
**Severity**: P2 (Medium - Breaks scripting)

**Description**:
When using `--output json`, the telemetry notice is printed to stdout along with the JSON output, making the output unparseable without filtering.

```bash
# This fails to parse:
caro --output json "list files" | jq
# jq: parse error: Invalid numeric literal at line 3, column 0
```

**Expected**:
- Telemetry notice should go to stderr
- JSON output should go to stdout (clean, parseable)

**Actual**:
Both telemetry notice and JSON are written to stdout, requiring filtering:
```bash
caro --output json "list files" 2>&1 | grep -A100 "^{" | jq
```

**Impact**:
- Breaks CI/CD pipelines expecting clean JSON
- Prevents direct piping to jq/other JSON tools
- Forces complex filtering workarounds

**Workaround**:
```bash
# Filter to extract JSON only
caro --output json "query" 2>&1 | grep -A100 "^{" | jq -r '.generated_command'
```

**Fix Plan**:
Redirect telemetry notice to stderr, keep JSON on stdout

**Testing Impact**:
- Use workaround for JSON parsing in scripts
- All JSON tests require filtering

---

### Issue #6: Inconsistent Exit Codes for Safety Violations

**Status**: CONFIRMED - Do Not Report Again
**Discovered By**: SRE/DevOps tester (Day 4)
**Severity**: P2 (Medium - Inconsistent behavior)

**Description**:
Different dangerous commands return different exit codes, making it impossible to reliably detect safety violations in scripts.

**Test Results**:
| Query | Behavior | Exit Code | Expected |
|-------|----------|-----------|----------|
| "delete everything" | Blocked with error | 1 | ✅ Correct |
| "remove all files recursively" | Refused (echo message) | 0 | ❌ Should be 1 |
| "kill all processes" | Blocked with error | 1 | ✅ Correct |

**Expected**:
All safety violations should return non-zero exit code (1)

**Actual**:
Some refusals return exit code 0, preventing detection in scripts

**Impact**:
- Scripts cannot reliably detect command generation failures
- CI/CD pipelines may proceed with unsafe commands
- False sense of success when command was refused

**Workaround**:
Parse JSON output for `blocked_reason` field instead of relying on exit codes

**Fix Plan**:
Standardize all safety violations to return exit code 1

**Testing Impact**:
- Check exit codes for all safety tests
- Use JSON parsing for reliable detection

---

### Issue #7: Complex Multi-Step Queries Simplified

**Status**: CONFIRMED - Do Not Report Again
**Discovered By**: SRE/DevOps tester (Day 5)
**Severity**: P3 (Low - Model limitation)

**Description**:
Long queries with multiple requirements (filter + count + sort + exclude) are simplified to only handle the first requirement.

**Example**:
Query (75 words):
```
"I need to find all Python files that were modified in the last 7 days
and contain the word 'import requests' somewhere in their content, then
I want to count how many such files exist and list their full paths
sorted by modification time with the newest first, excluding any files
in the __pycache__ or .venv directories"
```

Generated command:
```bash
find . -name "*.py" -type f -mtime -7
```

**Missing**: grep filter, count, sort, exclusions

**Expected**:
Generate multi-step pipeline or prompt user to break into steps

**Actual**:
Simplified to first requirement only

**Impact**:
- Users must manually break complex requests into multiple commands
- May give impression that tool "didn't understand" the full request

**Workaround**:
Break complex queries into sequential simple queries

**Fix Plan**:
- Add multi-step command generation
- OR provide feedback: "Query too complex, try breaking into steps"

**Testing Impact**:
- Expect simple command output for complex queries
- Break workflows into multiple queries

---

### Issue #8: Help Output Lists Non-Existent Subcommands

**Status**: CONFIRMED - Do Not Report Again
**Discovered By**: Terminal Novice tester (Cycle 3)
**Severity**: P2 (Medium - Minor confusion)

**Description**:
The `caro --help` output lists `assess` and `telemetry` as available subcommands, but they don't function as subcommands in v1.1.0-beta.1. Instead, they are interpreted as natural language prompts and generate shell commands.

**Example**:
```bash
# Help output shows:
$ caro --help
Commands:
  doctor      Show system diagnostics
  assess      Assess system capabilities
  test        Run test suite
  telemetry   Manage telemetry settings

# But running them:
$ caro assess
Command: ps aux | sort -nrk 3,3

$ caro telemetry status
Command: systemctl status telemetry
```

**Expected**:
Either:
- Option A: Subcommands work as documented in help output
- Option B: Help output doesn't list non-functional subcommands

**Actual**:
Help lists subcommands but they generate shell commands instead

**Impact**:
- Users checking `--help` before reading docs get confused
- Creates mismatch between help output and actual behavior
- Documentation warns not to use these commands, but help suggests they exist

**Workaround**:
Read the beta testing instructions which correctly document that these commands don't exist

**Fix Plan**:
- Option A: Implement `assess` and `telemetry` as proper subcommands (higher effort)
- Option B: Remove from help output until implemented (lower effort)

**Testing Impact**:
- Users should follow documentation instead of help output
- Help output may not reflect actual available features in beta.1

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
1. Add telemetry subcommands OR remove from docs (Issue #1)
2. Add `assess` subcommand OR update docs to use `doctor` (Issue #2)
3. Fix telemetry notice first-run detection (Issue #3)
4. Update installation guide with sudo-free option (Issue #4)
5. Redirect telemetry notice to stderr for clean JSON output (Issue #5)
6. Standardize exit codes for all safety violations (Issue #6)
7. Add multi-step command generation OR complexity feedback (Issue #7)

---

**For Questions**: File a GitHub Discussion or contact beta@caro.sh

**Version**: 1.0
**Applies To**: v1.1.0-beta.1 only
